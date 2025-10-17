//! Streaming Enhancements & Optimization
//!
//! Provides Server-Sent Events, compression, token batching, keep-alive, and timeout handling

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Compression format options
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum CompressionFormat {
    /// No compression
    None,
    /// gzip compression
    Gzip,
    /// deflate compression
    Deflate,
    /// Brotli compression
    Brotli,
}

impl CompressionFormat {
    /// Parse from Accept-Encoding header
    pub fn from_accept_encoding(header: &str) -> Vec<Self> {
        let mut formats = Vec::new();

        if header.contains("gzip") {
            formats.push(CompressionFormat::Gzip);
        }
        if header.contains("deflate") {
            formats.push(CompressionFormat::Deflate);
        }
        if header.contains("br") {
            formats.push(CompressionFormat::Brotli);
        }

        if formats.is_empty() {
            formats.push(CompressionFormat::None);
        }

        formats
    }

    /// Get content-encoding header value
    pub fn header_value(&self) -> &'static str {
        match self {
            CompressionFormat::None => "",
            CompressionFormat::Gzip => "gzip",
            CompressionFormat::Deflate => "deflate",
            CompressionFormat::Brotli => "br",
        }
    }
}

/// Server-Sent Events configuration
#[derive(Debug, Clone)]
pub struct SSEConfig {
    /// Event type for token events
    pub event_type: String,
    /// Event type for completion
    pub completion_event_type: String,
    /// Event type for errors
    pub error_event_type: String,
    /// Event type for keep-alive
    pub keepalive_event_type: String,
}

impl Default for SSEConfig {
    fn default() -> Self {
        Self {
            event_type: "token".to_string(),
            completion_event_type: "complete".to_string(),
            error_event_type: "error".to_string(),
            keepalive_event_type: "heartbeat".to_string(),
        }
    }
}

/// Server-Sent Event message
#[derive(Debug, Clone)]
pub struct SSEMessage {
    pub event: String,
    pub data: String,
    pub id: Option<String>,
    pub retry: Option<u32>,
}

impl SSEMessage {
    /// Create new SSE message
    pub fn new(event: String, data: String) -> Self {
        Self {
            event,
            data,
            id: None,
            retry: None,
        }
    }

    /// Add event ID
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// Add retry time (milliseconds)
    pub fn with_retry(mut self, retry_ms: u32) -> Self {
        self.retry = Some(retry_ms);
        self
    }

    /// Serialize to SSE format
    pub fn to_sse_format(&self) -> String {
        let mut output = format!("event: {}\n", self.event);
        output.push_str(&format!("data: {}\n", self.data));

        if let Some(id) = &self.id {
            output.push_str(&format!("id: {}\n", id));
        }

        if let Some(retry) = self.retry {
            output.push_str(&format!("retry: {}\n", retry));
        }

        output.push('\n');
        output
    }
}

/// Token batcher for streaming optimization
#[derive(Debug)]
pub struct TokenBatcher {
    /// Batch size (default 2-3 tokens)
    batch_size: usize,
    /// Maximum wait time before flushing
    max_wait_ms: Duration,
    /// Current buffer
    buffer: Vec<String>,
    /// Last flush time
    last_flush: Instant,
}

impl TokenBatcher {
    /// Create new token batcher
    pub fn new(batch_size: usize, max_wait_ms: u64) -> Self {
        Self {
            batch_size,
            max_wait_ms: Duration::from_millis(max_wait_ms),
            buffer: Vec::with_capacity(batch_size),
            last_flush: Instant::now(),
        }
    }

    /// Add token to buffer
    pub fn add_token(&mut self, token: String) {
        self.buffer.push(token);
    }

    /// Check if buffer should be flushed
    pub fn should_flush(&self) -> bool {
        self.buffer.len() >= self.batch_size || self.last_flush.elapsed() > self.max_wait_ms
    }

    /// Flush buffer and return batched tokens
    pub fn flush(&mut self) -> String {
        let batched = self.buffer.join("");
        self.buffer.clear();
        self.last_flush = Instant::now();
        batched
    }

    /// Get buffer size
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

/// Timeout manager for streaming operations
#[derive(Debug, Clone)]
pub struct TimeoutManager {
    /// Inference timeout (default 5 minutes)
    inference_timeout: Duration,
    /// Token generation timeout (default 30 seconds)
    token_timeout: Duration,
    /// Start time
    start_time: Instant,
    /// Last token time
    last_token_time: Instant,
}

impl TimeoutManager {
    /// Create new timeout manager
    pub fn new(inference_timeout_secs: u64, token_timeout_secs: u64) -> Self {
        let now = Instant::now();
        Self {
            inference_timeout: Duration::from_secs(inference_timeout_secs),
            token_timeout: Duration::from_secs(token_timeout_secs),
            start_time: now,
            last_token_time: now,
        }
    }

    /// Check if inference timeout exceeded
    pub fn is_inference_timeout(&self) -> bool {
        self.start_time.elapsed() > self.inference_timeout
    }

    /// Check if token timeout exceeded
    pub fn is_token_timeout(&self) -> bool {
        self.last_token_time.elapsed() > self.token_timeout
    }

    /// Record token received
    pub fn record_token(&mut self) {
        self.last_token_time = Instant::now();
    }

    /// Get elapsed inference time
    pub fn elapsed_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get time since last token
    pub fn time_since_last_token_ms(&self) -> u64 {
        self.last_token_time.elapsed().as_millis() as u64
    }
}

/// Keep-alive mechanism for detecting dead connections
#[derive(Debug)]
pub struct KeepAlive {
    /// Keep-alive interval
    interval: Duration,
    /// Last keep-alive sent
    last_sent: Instant,
    /// Keep-alive counter
    count: u32,
}

impl KeepAlive {
    /// Create new keep-alive manager
    pub fn new(interval_secs: u64) -> Self {
        Self {
            interval: Duration::from_secs(interval_secs),
            last_sent: Instant::now(),
            count: 0,
        }
    }

    /// Check if keep-alive should be sent
    pub fn should_send_keepalive(&self) -> bool {
        self.last_sent.elapsed() > self.interval
    }

    /// Send keep-alive
    pub fn send_keepalive(&mut self) -> u32 {
        self.last_sent = Instant::now();
        self.count += 1;
        self.count
    }

    /// Get keep-alive count
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Reset (typically on token received)
    pub fn reset(&mut self) {
        self.last_sent = Instant::now();
    }
}

/// Streaming configuration combining all enhancements
#[derive(Debug, Clone)]
pub struct StreamingOptimizationConfig {
    pub compression: CompressionFormat,
    pub sse_config: SSEConfig,
    pub batch_size: usize,
    pub batch_max_wait_ms: u64,
    pub inference_timeout_secs: u64,
    pub token_timeout_secs: u64,
    pub keepalive_interval_secs: u64,
    pub tcp_nodelay: bool,
}

impl Default for StreamingOptimizationConfig {
    fn default() -> Self {
        Self {
            compression: CompressionFormat::None,
            sse_config: SSEConfig::default(),
            batch_size: 3,
            batch_max_wait_ms: 50,
            inference_timeout_secs: 300,
            token_timeout_secs: 30,
            keepalive_interval_secs: 30,
            tcp_nodelay: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_format_parsing() {
        let formats = CompressionFormat::from_accept_encoding("gzip, deflate, br");
        assert!(formats.contains(&CompressionFormat::Gzip));
        assert!(formats.contains(&CompressionFormat::Deflate));
        assert!(formats.contains(&CompressionFormat::Brotli));
    }

    #[test]
    fn test_sse_message_formatting() {
        let msg = SSEMessage::new("token".to_string(), "Hello".to_string())
            .with_id("123".to_string())
            .with_retry(1000);

        let formatted = msg.to_sse_format();
        assert!(formatted.contains("event: token"));
        assert!(formatted.contains("data: Hello"));
        assert!(formatted.contains("id: 123"));
        assert!(formatted.contains("retry: 1000"));
    }

    #[test]
    fn test_token_batcher() {
        let mut batcher = TokenBatcher::new(3, 100);

        assert!(!batcher.should_flush());

        batcher.add_token("Hello".to_string());
        batcher.add_token(" ".to_string());
        assert!(!batcher.should_flush());

        batcher.add_token("World".to_string());
        assert!(batcher.should_flush());

        let batched = batcher.flush();
        assert_eq!(batched, "Hello World");
        assert!(batcher.is_empty());
    }

    #[test]
    fn test_token_batcher_timeout() {
        let mut batcher = TokenBatcher::new(100, 50); // Very small timeout

        batcher.add_token("token1".to_string());
        assert!(!batcher.should_flush());

        std::thread::sleep(Duration::from_millis(100));
        assert!(batcher.should_flush()); // Should flush due to timeout
    }

    #[test]
    fn test_timeout_manager() {
        let tm = TimeoutManager::new(5, 1);

        assert!(!tm.is_inference_timeout());
        assert!(!tm.is_token_timeout());

        std::thread::sleep(Duration::from_millis(1100));
        assert!(tm.is_token_timeout());
    }

    #[test]
    fn test_keepalive() {
        let mut ka = KeepAlive::new(1);

        assert!(!ka.should_send_keepalive());

        std::thread::sleep(Duration::from_millis(1100));
        assert!(ka.should_send_keepalive());

        ka.send_keepalive();
        assert_eq!(ka.count(), 1);
        assert!(!ka.should_send_keepalive());
    }

    #[test]
    fn test_default_config() {
        let config = StreamingOptimizationConfig::default();
        assert_eq!(config.batch_size, 3);
        assert_eq!(config.inference_timeout_secs, 300);
        assert!(config.tcp_nodelay);
    }
}
