use crate::backends::{Backend, InferenceParams, TokenStream};
use crate::InfernoError;
use anyhow::Result;
use async_stream;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid;

/// Configuration for real-time streaming inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Maximum number of concurrent streams
    pub max_concurrent_streams: usize,
    /// Buffer size per stream
    pub buffer_size: usize,
    /// Enable backpressure handling
    pub enable_backpressure: bool,
    /// Timeout for individual token generation (milliseconds)
    pub token_timeout_ms: u64,
    /// Maximum time for a complete response (seconds)
    pub max_response_time_seconds: u64,
    /// Enable real-time metrics collection
    pub enable_metrics: bool,
    /// Heartbeat interval for connection health (milliseconds)
    pub heartbeat_interval_ms: u64,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_concurrent_streams: 10,
            buffer_size: 100,
            enable_backpressure: true,
            token_timeout_ms: 30000, // 30 seconds
            max_response_time_seconds: 300, // 5 minutes
            enable_metrics: true,
            heartbeat_interval_ms: 30000, // 30 seconds
        }
    }
}

/// Real-time streaming metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingMetrics {
    pub active_streams: usize,
    pub total_streams_created: u64,
    pub total_tokens_streamed: u64,
    pub average_tokens_per_second: f32,
    pub average_latency_ms: f32,
    pub errors_count: u64,
    pub buffer_overflows: u64,
    pub timeouts: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for StreamingMetrics {
    fn default() -> Self {
        Self {
            active_streams: 0,
            total_streams_created: 0,
            total_tokens_streamed: 0,
            average_tokens_per_second: 0.0,
            average_latency_ms: 0.0,
            errors_count: 0,
            buffer_overflows: 0,
            timeouts: 0,
            last_updated: chrono::Utc::now(),
        }
    }
}

/// Stream state for tracking individual stream health
#[derive(Debug, Clone)]
pub struct StreamState {
    pub stream_id: String,
    pub created_at: Instant,
    pub last_token_at: Option<Instant>,
    pub tokens_generated: u64,
    pub errors: u64,
    pub status: StreamStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamStatus {
    Active,
    Completed,
    Error,
    Timeout,
    Cancelled,
}

/// Enhanced streaming manager with real-time capabilities
pub struct StreamingManager {
    config: StreamingConfig,
    metrics: Arc<Mutex<StreamingMetrics>>,
    active_streams: Arc<Mutex<Vec<StreamState>>>,
    metrics_broadcast: broadcast::Sender<StreamingMetrics>,
}

impl StreamingManager {
    pub fn new(config: StreamingConfig) -> Self {
        let (metrics_broadcast, _) = broadcast::channel(100);

        Self {
            config,
            metrics: Arc::new(Mutex::new(StreamingMetrics::default())),
            active_streams: Arc::new(Mutex::new(Vec::new())),
            metrics_broadcast,
        }
    }

    /// Start the streaming manager with background tasks
    pub async fn start(&self) -> Result<()> {
        info!("Starting streaming manager with enhanced real-time capabilities");

        // Start metrics collection task
        if self.config.enable_metrics {
            self.start_metrics_collection().await?;
        }

        // Start health monitoring task
        self.start_health_monitoring().await?;

        Ok(())
    }

    /// Create an enhanced streaming inference session
    pub async fn create_enhanced_stream(
        &self,
        backend: &mut Backend,
        input: &str,
        params: &InferenceParams,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamingToken, InfernoError>> + Send>>> {
        let stream_id = uuid::Uuid::new_v4().to_string();

        // Check stream limits
        {
            let active_streams = self.active_streams.lock()
                .map_err(|e| anyhow::anyhow!("Active streams mutex poisoned: {}", e))?;
            if active_streams.len() >= self.config.max_concurrent_streams {
                return Err(InfernoError::StreamingLimit(
                    "Maximum concurrent streams reached".to_string()
                ).into());
            }
        }

        info!("Creating enhanced stream: {}", stream_id);

        // Create stream state
        let stream_state = StreamState {
            stream_id: stream_id.clone(),
            created_at: Instant::now(),
            last_token_at: None,
            tokens_generated: 0,
            errors: 0,
            status: StreamStatus::Active,
        };

        // Add to active streams
        {
            let mut active_streams = self.active_streams.lock()
                .expect("Active streams mutex poisoned in create_enhanced_stream");
            active_streams.push(stream_state);
        }

        // Update metrics
        {
            let mut metrics = self.metrics.lock()
                .expect("Metrics mutex poisoned in create_enhanced_stream");
            metrics.active_streams += 1;
            metrics.total_streams_created += 1;
        }

        // Get the base token stream from backend
        let base_stream = backend.infer_stream(input, params).await?;

        // Create enhanced stream with monitoring and buffering
        let enhanced_stream = self.create_monitored_stream(
            stream_id.clone(),
            base_stream,
        ).await;

        Ok(Box::pin(enhanced_stream))
    }

    /// Create a monitored stream with backpressure and error handling
    async fn create_monitored_stream(
        &self,
        stream_id: String,
        mut base_stream: TokenStream,
    ) -> impl Stream<Item = Result<StreamingToken, InfernoError>> {
        let config = self.config.clone();
        let metrics = self.metrics.clone();
        let active_streams = self.active_streams.clone();
        let metrics_broadcast = self.metrics_broadcast.clone();

        async_stream::stream! {
            let mut buffer = VecDeque::new();
            let mut last_activity = Instant::now();
            let stream_start = Instant::now();
            let mut tokens_generated = 0u64;

            // Create timeout for overall response
            let response_timeout = Duration::from_secs(config.max_response_time_seconds);

            loop {
                // Check for overall timeout
                if stream_start.elapsed() > response_timeout {
                    warn!("Stream {} timed out after {}s", stream_id, config.max_response_time_seconds);

                    // Update stream state
                    Self::update_stream_status(&active_streams, &stream_id, StreamStatus::Timeout);

                    // Update metrics
                    {
                        let mut metrics_guard = metrics.lock()
                            .expect("Metrics mutex poisoned during timeout in stream");
                        metrics_guard.timeouts += 1;
                        metrics_guard.active_streams = metrics_guard.active_streams.saturating_sub(1);
                    }

                    yield Err(InfernoError::Timeout("Stream response timeout".to_string()));
                    break;
                }

                // Try to get next token with timeout
                let token_timeout = Duration::from_millis(config.token_timeout_ms);

                match tokio::time::timeout(token_timeout, base_stream.next()).await {
                    Ok(Some(token_result)) => {
                        last_activity = Instant::now();

                        match token_result {
                            Ok(token) => {
                                tokens_generated += 1;

                                // Update stream state
                                Self::update_stream_activity(&active_streams, &stream_id, tokens_generated);

                                // Create enhanced token with metadata
                                let streaming_token = StreamingToken {
                                    content: token,
                                    stream_id: stream_id.clone(),
                                    token_index: tokens_generated,
                                    timestamp: chrono::Utc::now(),
                                    latency_ms: stream_start.elapsed().as_millis() as u32,
                                };

                                // Handle backpressure if enabled
                                if config.enable_backpressure && buffer.len() >= config.buffer_size {
                                    warn!("Buffer overflow for stream {}, dropping oldest tokens", stream_id);

                                    // Update metrics
                                    {
                                        let mut metrics_guard = metrics.lock()
                                            .expect("Metrics mutex poisoned during buffer overflow in stream");
                                        metrics_guard.buffer_overflows += 1;
                                    }

                                    buffer.pop_front();
                                }

                                buffer.push_back(streaming_token);

                                // Yield token from buffer
                                if let Some(buffered_token) = buffer.pop_front() {
                                    yield Ok(buffered_token);
                                }

                                // Update metrics
                                {
                                    let mut metrics_guard = metrics.lock()
                                        .expect("Metrics mutex poisoned during token count in stream");
                                    metrics_guard.total_tokens_streamed += 1;

                                    // Update averages
                                    let elapsed_secs = stream_start.elapsed().as_secs_f32();
                                    if elapsed_secs > 0.0 {
                                        metrics_guard.average_tokens_per_second =
                                            tokens_generated as f32 / elapsed_secs;
                                    }

                                    metrics_guard.average_latency_ms =
                                        stream_start.elapsed().as_millis() as f32 / tokens_generated as f32;

                                    metrics_guard.last_updated = chrono::Utc::now();
                                }
                            }
                            Err(e) => {
                                error!("Token generation error in stream {}: {}", stream_id, e);

                                // Update stream state
                                Self::update_stream_error(&active_streams, &stream_id);

                                // Update metrics
                                {
                                    let mut metrics_guard = metrics.lock()
                                        .expect("Metrics mutex poisoned during error count in stream");
                                    metrics_guard.errors_count += 1;
                                }

                                yield Err(e);
                            }
                        }
                    }
                    Ok(None) => {
                        // Stream completed normally
                        info!("Stream {} completed successfully with {} tokens", stream_id, tokens_generated);

                        // Flush remaining buffer
                        while let Some(buffered_token) = buffer.pop_front() {
                            yield Ok(buffered_token);
                        }

                        // Update stream state
                        Self::update_stream_status(&active_streams, &stream_id, StreamStatus::Completed);

                        // Update metrics
                        {
                            let mut metrics_guard = metrics.lock()
                                .expect("Metrics mutex poisoned during cleanup in stream");
                            metrics_guard.active_streams = metrics_guard.active_streams.saturating_sub(1);

                            // Broadcast updated metrics
                            let _ = metrics_broadcast.send(metrics_guard.clone());
                        }

                        break;
                    }
                    Err(_) => {
                        // Token timeout
                        warn!("Token timeout in stream {} after {}ms", stream_id, config.token_timeout_ms);

                        // Check if we should continue or abort
                        if last_activity.elapsed() > Duration::from_millis(config.token_timeout_ms * 2) {
                            // Too long without activity, abort stream
                            Self::update_stream_status(&active_streams, &stream_id, StreamStatus::Timeout);

                            {
                                let mut metrics_guard = metrics.lock()
                                    .expect("Metrics mutex poisoned during timeout in cleanup in stream");
                                metrics_guard.timeouts += 1;
                                metrics_guard.active_streams = metrics_guard.active_streams.saturating_sub(1);
                            }

                            yield Err(InfernoError::Timeout("Token generation timeout".to_string()));
                            break;
                        }

                        // Send heartbeat token to keep connection alive
                        yield Ok(StreamingToken {
                            content: "".to_string(), // Empty content for heartbeat
                            stream_id: stream_id.clone(),
                            token_index: 0, // Special index for heartbeat
                            timestamp: chrono::Utc::now(),
                            latency_ms: 0,
                        });
                    }
                }
            }

            debug!("Enhanced stream {} finished", stream_id);
        }
    }

    /// Start metrics collection background task
    async fn start_metrics_collection(&self) -> Result<()> {
        let metrics = self.metrics.clone();
        let active_streams = self.active_streams.clone();
        let broadcast = self.metrics_broadcast.clone();
        let interval_ms = self.config.heartbeat_interval_ms;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(interval_ms));

            loop {
                interval.tick().await;

                // Update and broadcast metrics
                {
                    let active_streams_guard = active_streams.lock()
                        .expect("Active streams mutex poisoned during metrics update");
                    let mut metrics_guard = metrics.lock()
                        .expect("Metrics mutex poisoned during metrics update");

                    metrics_guard.active_streams = active_streams_guard.len();
                    metrics_guard.last_updated = chrono::Utc::now();

                    // Broadcast updated metrics
                    let _ = broadcast.send(metrics_guard.clone());
                }
            }
        });

        Ok(())
    }

    /// Start health monitoring background task
    async fn start_health_monitoring(&self) -> Result<()> {
        let active_streams = self.active_streams.clone();
        let timeout_threshold = Duration::from_millis(self.config.token_timeout_ms * 3);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds

            loop {
                interval.tick().await;

                // Check for stale streams
                {
                    let mut streams = active_streams.lock()
                        .expect("Active streams mutex poisoned during cleanup");
                    let now = Instant::now();

                    streams.retain(|stream| {
                        let is_stale = match stream.last_token_at {
                            Some(last_token) => now.duration_since(last_token) > timeout_threshold,
                            None => now.duration_since(stream.created_at) > timeout_threshold,
                        };

                        if is_stale && stream.status == StreamStatus::Active {
                            warn!("Detected stale stream: {}", stream.stream_id);
                            false // Remove stale stream
                        } else {
                            true // Keep active stream
                        }
                    });
                }
            }
        });

        Ok(())
    }

    /// Get current streaming metrics
    pub fn get_metrics(&self) -> StreamingMetrics {
        self.metrics.lock()
            .expect("Metrics mutex poisoned in get_metrics")
            .clone()
    }

    /// Subscribe to real-time metrics updates
    pub fn subscribe_to_metrics(&self) -> broadcast::Receiver<StreamingMetrics> {
        self.metrics_broadcast.subscribe()
    }

    /// Get active stream states
    pub fn get_active_streams(&self) -> Vec<StreamState> {
        self.active_streams.lock()
            .expect("Active streams mutex poisoned in get_active_streams")
            .clone()
    }

    /// Helper methods for stream state management
    fn update_stream_status(
        active_streams: &Arc<Mutex<Vec<StreamState>>>,
        stream_id: &str,
        status: StreamStatus,
    ) {
        let mut streams = active_streams.lock()
            .expect("Active streams mutex poisoned in update_stream_status");
        if let Some(stream) = streams.iter_mut().find(|s| s.stream_id == stream_id) {
            stream.status = status;
        }
    }

    fn update_stream_activity(
        active_streams: &Arc<Mutex<Vec<StreamState>>>,
        stream_id: &str,
        tokens_generated: u64,
    ) {
        let mut streams = active_streams.lock()
            .expect("Active streams mutex poisoned in update_stream_status");
        if let Some(stream) = streams.iter_mut().find(|s| s.stream_id == stream_id) {
            stream.last_token_at = Some(Instant::now());
            stream.tokens_generated = tokens_generated;
        }
    }

    fn update_stream_error(
        active_streams: &Arc<Mutex<Vec<StreamState>>>,
        stream_id: &str,
    ) {
        let mut streams = active_streams.lock()
            .expect("Active streams mutex poisoned in update_stream_status");
        if let Some(stream) = streams.iter_mut().find(|s| s.stream_id == stream_id) {
            stream.errors += 1;
            stream.status = StreamStatus::Error;
        }
    }
}

/// Enhanced streaming token with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingToken {
    pub content: String,
    pub stream_id: String,
    pub token_index: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub latency_ms: u32,
}

impl StreamingToken {
    /// Check if this is a heartbeat token
    pub fn is_heartbeat(&self) -> bool {
        self.token_index == 0 && self.content.is_empty()
    }
}