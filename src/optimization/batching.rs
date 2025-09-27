// Dynamic batching module for Inferno AI/ML platform
// Provides intelligent request batching and scheduling for improved throughput

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time::{Duration, Instant};
use uuid::Uuid;

/// Batching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchingConfig {
    pub enabled: bool,
    pub max_batch_size: usize,
    pub max_wait_time_ms: u64,
    pub min_batch_size: usize,
    pub adaptive_batching: bool,
    pub priority_levels: usize,
    pub sequence_length_grouping: bool,
    pub padding_strategy: PaddingStrategy,
    pub throughput_target: f64,
}

impl Default for BatchingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_batch_size: 32,
            max_wait_time_ms: 50,
            min_batch_size: 1,
            adaptive_batching: true,
            priority_levels: 3,
            sequence_length_grouping: true,
            padding_strategy: PaddingStrategy::LeftPadding,
            throughput_target: 1000.0, // requests per second
        }
    }
}

/// Padding strategies for batching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaddingStrategy {
    LeftPadding,
    RightPadding,
    NoPadding,
    DynamicPadding,
}

/// Request priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
}

/// Batch request representation
#[derive(Debug)]
pub struct BatchRequest {
    pub id: Uuid,
    pub input: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub priority: Priority,
    pub sequence_length: usize,
    pub received_at: Instant,
    pub response_sender: Option<tokio::sync::oneshot::Sender<Result<String>>>,
}

impl BatchRequest {
    pub fn new(
        input: String,
        priority: Priority,
    ) -> (Self, tokio::sync::oneshot::Receiver<Result<String>>) {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let request = Self {
            id: Uuid::new_v4(),
            sequence_length: input.len(),
            input,
            max_tokens: None,
            temperature: None,
            priority,
            received_at: Instant::now(),
            response_sender: Some(tx),
        };
        (request, rx)
    }
}

/// Batch of requests ready for processing
#[derive(Debug)]
pub struct Batch {
    pub id: Uuid,
    pub requests: Vec<BatchRequest>,
    pub created_at: Instant,
    pub estimated_processing_time: Duration,
}

impl Batch {
    pub fn new(requests: Vec<BatchRequest>) -> Self {
        let estimated_time = Duration::from_millis(requests.len() as u64 * 10); // Rough estimate
        Self {
            id: Uuid::new_v4(),
            requests,
            created_at: Instant::now(),
            estimated_processing_time: estimated_time,
        }
    }

    pub fn size(&self) -> usize {
        self.requests.len()
    }

    pub fn avg_sequence_length(&self) -> f64 {
        if self.requests.is_empty() {
            return 0.0;
        }
        self.requests
            .iter()
            .map(|r| r.sequence_length)
            .sum::<usize>() as f64
            / self.requests.len() as f64
    }
}

/// Batching metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatchingMetrics {
    pub throughput_improvement: f64,
    pub efficiency_ratio: f64,
    pub avg_batch_size: f64,
    pub avg_wait_time_ms: f64,
    pub total_requests_processed: u64,
    pub total_batches_processed: u64,
    pub requests_per_second: f64,
}

/// Dynamic batcher implementation
pub struct DynamicBatcher {
    config: BatchingConfig,
    metrics: Arc<RwLock<BatchingMetrics>>,
    request_queues: Arc<RwLock<HashMap<Priority, VecDeque<BatchRequest>>>>,
    batch_sender: mpsc::UnboundedSender<Batch>,
    batch_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<Batch>>>>,
    processing_semaphore: Arc<Semaphore>,
    adaptive_params: Arc<RwLock<AdaptiveParams>>,
}

#[derive(Debug, Clone)]
struct AdaptiveParams {
    current_batch_size: usize,
    current_wait_time: Duration,
    recent_throughput: f64,
    last_adjustment: Instant,
}

impl DynamicBatcher {
    /// Create new dynamic batcher
    pub async fn new(config: BatchingConfig) -> Result<Self> {
        let (batch_sender, batch_receiver) = mpsc::unbounded_channel();

        let mut request_queues = HashMap::new();
        request_queues.insert(Priority::Low, VecDeque::new());
        request_queues.insert(Priority::Normal, VecDeque::new());
        request_queues.insert(Priority::High, VecDeque::new());

        let adaptive_params = AdaptiveParams {
            current_batch_size: config.max_batch_size,
            current_wait_time: Duration::from_millis(config.max_wait_time_ms),
            recent_throughput: 0.0,
            last_adjustment: Instant::now(),
        };

        Ok(Self {
            config,
            metrics: Arc::new(RwLock::new(BatchingMetrics::default())),
            request_queues: Arc::new(RwLock::new(request_queues)),
            batch_sender,
            batch_receiver: Arc::new(RwLock::new(Some(batch_receiver))),
            processing_semaphore: Arc::new(Semaphore::new(10)), // Max 10 concurrent batches
            adaptive_params: Arc::new(RwLock::new(adaptive_params)),
        })
    }

    /// Submit request for batching
    pub async fn submit_request(
        &self,
        input: String,
        priority: Priority,
    ) -> Result<tokio::sync::oneshot::Receiver<Result<String>>> {
        let (request, receiver) = BatchRequest::new(input, priority);

        // Add to appropriate priority queue
        {
            let mut queues = self.request_queues.write().await;
            queues.get_mut(&priority).unwrap().push_back(request);
        }

        tracing::debug!(
            "Request submitted for batching with priority: {:?}",
            priority
        );
        Ok(receiver)
    }

    /// Start the batching process
    pub async fn start_batching(&self) -> Result<()> {
        let batcher = self.clone();
        tokio::spawn(async move {
            batcher.batch_processing_loop().await;
        });

        let processor = Arc::new(self.clone());
        let processor_clone = Arc::clone(&processor);
        tokio::spawn(async move {
            if let Some(receiver) = processor.batch_receiver.write().await.take() {
                DynamicBatcher::batch_execution_loop(processor_clone, receiver).await;
            }
        });

        tracing::info!("Dynamic batching started");
        Ok(())
    }

    /// Main batching loop
    async fn batch_processing_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_millis(10));

        loop {
            interval.tick().await;

            if let Some(batch) = self.try_create_batch().await {
                if let Err(e) = self.batch_sender.send(batch) {
                    tracing::error!("Failed to send batch: {}", e);
                    break;
                }
            }

            // Adaptive parameter adjustment
            if self.config.adaptive_batching {
                self.adjust_adaptive_parameters().await;
            }
        }
    }

    /// Try to create a batch from queued requests
    async fn try_create_batch(&self) -> Option<Batch> {
        let mut queues = self.request_queues.write().await;
        let adaptive_params = self.adaptive_params.read().await;

        let mut batch_requests = Vec::new();
        let max_batch_size = adaptive_params.current_batch_size;

        // Process high priority first, then normal, then low
        for priority in [Priority::High, Priority::Normal, Priority::Low] {
            if let Some(queue) = queues.get_mut(&priority) {
                while batch_requests.len() < max_batch_size && !queue.is_empty() {
                    // Check if we should wait for more requests
                    if batch_requests.len() < self.config.min_batch_size {
                        if let Some(oldest_request) = queue.front() {
                            if oldest_request.received_at.elapsed()
                                < adaptive_params.current_wait_time
                            {
                                continue; // Wait for more requests
                            }
                        }
                    }

                    if let Some(request) = queue.pop_front() {
                        batch_requests.push(request);
                    }
                }

                if batch_requests.len() >= max_batch_size {
                    break;
                }
            }
        }

        if !batch_requests.is_empty() && self.should_create_batch(&batch_requests, &adaptive_params)
        {
            // Group by sequence length if enabled
            if self.config.sequence_length_grouping {
                batch_requests = self.group_by_sequence_length(batch_requests);
            }

            Some(Batch::new(batch_requests))
        } else {
            None
        }
    }

    /// Determine if a batch should be created
    fn should_create_batch(
        &self,
        requests: &[BatchRequest],
        adaptive_params: &AdaptiveParams,
    ) -> bool {
        if requests.is_empty() {
            return false;
        }

        // Always create if we have enough requests
        if requests.len() >= adaptive_params.current_batch_size {
            return true;
        }

        // Create if minimum batch size is met and timeout exceeded
        if requests.len() >= self.config.min_batch_size {
            if let Some(oldest) = requests.iter().min_by_key(|r| r.received_at) {
                return oldest.received_at.elapsed() >= adaptive_params.current_wait_time;
            }
        }

        // Force batch if any request is too old
        requests
            .iter()
            .any(|r| r.received_at.elapsed() > adaptive_params.current_wait_time * 2)
    }

    /// Group requests by similar sequence length
    fn group_by_sequence_length(&self, mut requests: Vec<BatchRequest>) -> Vec<BatchRequest> {
        // Sort by sequence length for better batching efficiency
        requests.sort_by_key(|r| r.sequence_length);

        // Take the largest group with similar lengths
        if requests.len() <= 1 {
            return requests;
        }

        // Find the best contiguous group with similar sequence lengths
        let mut best_start = 0;
        let mut best_len = 1;
        let mut current_start = 0;
        let mut current_len = 1;

        for i in 1..requests.len() {
            let length_diff =
                (requests[i].sequence_length as i32 - requests[i - 1].sequence_length as i32).abs();

            // Group if length difference is within 20% or 50 tokens
            let prev_length = requests[i - 1].sequence_length;
            if length_diff <= (prev_length as f32 * 0.2) as i32 || length_diff <= 50 {
                current_len += 1;
            } else {
                if current_len > best_len {
                    best_start = current_start;
                    best_len = current_len;
                }
                current_start = i;
                current_len = 1;
            }
        }

        // Check the last group
        if current_len > best_len {
            best_start = current_start;
            best_len = current_len;
        }

        // Extract the best group
        requests
            .into_iter()
            .skip(best_start)
            .take(best_len)
            .collect()
    }

    /// Batch execution loop
    async fn batch_execution_loop(
        batcher: Arc<Self>,
        mut receiver: mpsc::UnboundedReceiver<Batch>,
    ) {
        while let Some(batch) = receiver.recv().await {
            let batcher_clone = Arc::clone(&batcher);

            tokio::spawn(async move {
                let permit = batcher_clone.processing_semaphore.acquire().await.unwrap();
                let _permit = permit;
                batcher_clone.process_batch(batch).await;
            });
        }
    }

    /// Process a batch of requests
    async fn process_batch(&self, batch: Batch) {
        let start_time = Instant::now();
        let batch_size = batch.size();

        tracing::debug!("Processing batch {} with {} requests", batch.id, batch_size);

        // Simulate batch processing (replace with actual inference)
        let batch_results = self.execute_batch_inference(&batch).await;

        // Send responses back to clients
        for (request, result) in batch.requests.into_iter().zip(batch_results) {
            if let Some(sender) = request.response_sender {
                let _ = sender.send(result);
            }
        }

        // Update metrics
        let processing_time = start_time.elapsed();
        self.update_metrics(batch_size, processing_time).await;

        tracing::debug!("Batch processing completed in {:?}", processing_time);
    }

    /// Execute batch inference (mock implementation)
    async fn execute_batch_inference(&self, batch: &Batch) -> Vec<Result<String>> {
        // Simulate batch processing time based on batch size and sequence length
        let avg_seq_len = batch.avg_sequence_length();
        let processing_time =
            Duration::from_millis((batch.size() as f64 * avg_seq_len * 0.1) as u64);

        tokio::time::sleep(processing_time).await;

        // Generate mock responses
        batch
            .requests
            .iter()
            .map(|request| {
                Ok(format!(
                    "Batch response for request {}: {}",
                    request.id, request.input
                ))
            })
            .collect()
    }

    /// Update batching metrics
    async fn update_metrics(&self, batch_size: usize, processing_time: Duration) {
        let mut metrics = self.metrics.write().await;

        metrics.total_batches_processed += 1;
        metrics.total_requests_processed += batch_size as u64;

        // Update running averages
        let total_batches = metrics.total_batches_processed as f64;
        metrics.avg_batch_size =
            (metrics.avg_batch_size * (total_batches - 1.0) + batch_size as f64) / total_batches;

        let processing_time_ms = processing_time.as_millis() as f64;
        metrics.avg_wait_time_ms =
            (metrics.avg_wait_time_ms * (total_batches - 1.0) + processing_time_ms) / total_batches;

        // Calculate requests per second
        if processing_time.as_secs_f64() > 0.0 {
            let current_rps = batch_size as f64 / processing_time.as_secs_f64();
            metrics.requests_per_second = (metrics.requests_per_second * 0.9) + (current_rps * 0.1);
            // Exponential moving average
        }

        // Calculate efficiency ratio (actual vs ideal throughput)
        let ideal_throughput = self.config.throughput_target;
        metrics.efficiency_ratio = metrics.requests_per_second / ideal_throughput;

        // Calculate throughput improvement (vs single request processing)
        metrics.throughput_improvement = metrics.avg_batch_size;
    }

    /// Adjust adaptive parameters based on performance
    async fn adjust_adaptive_parameters(&self) {
        let mut adaptive_params = self.adaptive_params.write().await;
        let metrics = self.metrics.read().await;

        // Only adjust every 5 seconds
        if adaptive_params.last_adjustment.elapsed() < Duration::from_secs(5) {
            return;
        }

        let current_throughput = metrics.requests_per_second;
        let target_throughput = self.config.throughput_target;

        tracing::debug!(
            "Adjusting adaptive parameters: current_throughput={:.2}, target={:.2}",
            current_throughput,
            target_throughput
        );

        // Adjust batch size based on throughput
        if current_throughput < target_throughput * 0.8 {
            // Increase batch size to improve throughput
            adaptive_params.current_batch_size =
                (adaptive_params.current_batch_size + 2).min(self.config.max_batch_size);
        } else if current_throughput > target_throughput * 1.2 {
            // Decrease batch size to reduce latency
            adaptive_params.current_batch_size =
                (adaptive_params.current_batch_size.saturating_sub(1))
                    .max(self.config.min_batch_size);
        }

        // Adjust wait time based on efficiency
        if metrics.efficiency_ratio < 0.7 {
            // Increase wait time to allow larger batches
            adaptive_params.current_wait_time = (adaptive_params.current_wait_time
                + Duration::from_millis(5))
            .min(Duration::from_millis(self.config.max_wait_time_ms));
        } else if metrics.efficiency_ratio > 1.3 {
            // Decrease wait time to reduce latency
            adaptive_params.current_wait_time = adaptive_params
                .current_wait_time
                .saturating_sub(Duration::from_millis(2))
                .max(Duration::from_millis(1));
        }

        adaptive_params.recent_throughput = current_throughput;
        adaptive_params.last_adjustment = Instant::now();

        tracing::debug!(
            "Adaptive parameters adjusted: batch_size={}, wait_time={:?}",
            adaptive_params.current_batch_size,
            adaptive_params.current_wait_time
        );
    }

    /// Get current batching metrics
    pub async fn get_metrics(&self) -> BatchingMetrics {
        self.metrics.read().await.clone()
    }

    /// Benchmark batching performance
    pub async fn benchmark(&self, _model_path: &str, num_requests: usize) -> Result<f64> {
        tracing::info!("Benchmarking batching with {} requests", num_requests);

        let start_time = Instant::now();

        // Submit test requests
        let mut receivers = Vec::new();
        for i in 0..num_requests {
            let priority = match i % 3 {
                0 => Priority::High,
                1 => Priority::Normal,
                _ => Priority::Low,
            };

            let receiver = self
                .submit_request(format!("test request {}", i), priority)
                .await?;
            receivers.push(receiver);
        }

        // Wait for all responses
        for receiver in receivers {
            let _ = receiver.await;
        }

        let total_time = start_time.elapsed();
        let throughput = num_requests as f64 / total_time.as_secs_f64();

        tracing::info!(
            "Batch benchmark completed: {:.2} requests/second",
            throughput
        );
        Ok(throughput / 100.0) // Return as performance multiplier
    }

    /// Get queue status
    pub async fn get_queue_status(&self) -> HashMap<String, usize> {
        let queues = self.request_queues.read().await;
        let mut status = HashMap::new();

        for (priority, queue) in queues.iter() {
            status.insert(format!("{:?}", priority), queue.len());
        }

        status
    }
}

impl Clone for DynamicBatcher {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            metrics: Arc::clone(&self.metrics),
            request_queues: Arc::clone(&self.request_queues),
            batch_sender: self.batch_sender.clone(),
            batch_receiver: Arc::clone(&self.batch_receiver),
            processing_semaphore: Arc::clone(&self.processing_semaphore),
            adaptive_params: Arc::clone(&self.adaptive_params),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batcher_creation() {
        let config = BatchingConfig::default();
        let batcher = DynamicBatcher::new(config).await;
        assert!(batcher.is_ok());
    }

    #[tokio::test]
    async fn test_request_submission() {
        let config = BatchingConfig::default();
        let batcher = DynamicBatcher::new(config).await.unwrap();

        let receiver = batcher
            .submit_request("test input".to_string(), Priority::Normal)
            .await;
        assert!(receiver.is_ok());
    }

    #[tokio::test]
    async fn test_batch_creation() {
        let requests = vec![
            BatchRequest::new("test 1".to_string(), Priority::Normal).0,
            BatchRequest::new("test 2".to_string(), Priority::High).0,
        ];

        let batch = Batch::new(requests);
        assert_eq!(batch.size(), 2);
        assert!(batch.avg_sequence_length() > 0.0);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }
}
