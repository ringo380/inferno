//! Real-time token streaming with tokio channels
//!
//! This module provides infrastructure for streaming tokens as they're generated,
//! enabling real-time feedback for users via WebSocket, SSE, or desktop UI.

use tokio::sync::mpsc;

/// Configuration for streaming behavior
#[derive(Clone, Debug)]
pub struct StreamConfig {
    /// Channel buffer size (tokens queued before blocking)
    pub buffer_size: usize,
    /// Whether to include timing metadata
    pub include_timing: bool,
    /// Maximum tokens per second (rate limiting, 0 = unlimited)
    pub max_tokens_per_sec: u32,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 64,
            include_timing: false,
            max_tokens_per_sec: 0, // Unlimited
        }
    }
}

/// A single token with optional metadata
#[derive(Clone, Debug)]
pub struct StreamToken {
    /// The token string content
    pub content: String,
    /// Token sequence number (0-indexed)
    pub sequence: u32,
    /// Detokenization succeeded
    pub is_valid: bool,
    /// Optional timing info (timestamp in ms since stream start)
    pub timestamp_ms: Option<u64>,
}

impl StreamToken {
    /// Create a valid token
    pub fn new(content: String, sequence: u32) -> Self {
        Self {
            content,
            sequence,
            is_valid: true,
            timestamp_ms: None,
        }
    }

    /// Create an invalid token (detokenization failed)
    pub fn invalid(sequence: u32) -> Self {
        Self {
            content: String::new(),
            sequence,
            is_valid: false,
            timestamp_ms: None,
        }
    }

    /// Set timing information
    pub fn with_timing(mut self, timestamp_ms: u64) -> Self {
        self.timestamp_ms = Some(timestamp_ms);
        self
    }
}

/// Receiver end of a streaming token channel
pub type StreamReceiver = mpsc::Receiver<StreamToken>;

/// Sender end of a streaming token channel
pub type StreamSender = mpsc::Sender<StreamToken>;

/// Create a new streaming channel with configuration
pub fn create_stream_channel(config: StreamConfig) -> (StreamSender, StreamReceiver) {
    mpsc::channel(config.buffer_size)
}

/// Statistics for a completed stream
#[derive(Clone, Debug, Default)]
pub struct StreamStats {
    /// Total tokens generated
    pub total_tokens: u32,
    /// Total valid tokens (successfully detokenized)
    pub valid_tokens: u32,
    /// Total invalid tokens (detokenization failed)
    pub invalid_tokens: u32,
    /// Total stream duration in milliseconds
    pub duration_ms: u64,
    /// Average tokens per second
    pub tokens_per_second: f32,
}

impl StreamStats {
    /// Create new stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record token generation
    pub fn record_token(&mut self, is_valid: bool) {
        self.total_tokens += 1;
        if is_valid {
            self.valid_tokens += 1;
        } else {
            self.invalid_tokens += 1;
        }
    }

    /// Calculate tokens per second
    pub fn finalize(&mut self) {
        if self.duration_ms > 0 {
            self.tokens_per_second =
                (self.total_tokens as f64 / (self.duration_ms as f64 / 1000.0)) as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_token_creation() {
        let token = StreamToken::new("hello".to_string(), 0);
        assert_eq!(token.content, "hello");
        assert_eq!(token.sequence, 0);
        assert!(token.is_valid);
    }

    #[test]
    fn test_stream_token_invalid() {
        let token = StreamToken::invalid(5);
        assert_eq!(token.sequence, 5);
        assert!(!token.is_valid);
    }

    #[test]
    fn test_stream_token_with_timing() {
        let token = StreamToken::new("test".to_string(), 0).with_timing(100);
        assert_eq!(token.timestamp_ms, Some(100));
    }

    #[test]
    fn test_stream_stats() {
        let mut stats = StreamStats::new();
        stats.record_token(true);
        stats.record_token(true);
        stats.record_token(false);
        stats.duration_ms = 1000;
        stats.finalize();

        assert_eq!(stats.total_tokens, 3);
        assert_eq!(stats.valid_tokens, 2);
        assert_eq!(stats.invalid_tokens, 1);
        assert!((stats.tokens_per_second - 3.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_create_stream_channel() {
        let config = StreamConfig {
            buffer_size: 10,
            ..Default::default()
        };
        let (tx, mut rx) = create_stream_channel(config);

        // Send a token
        let token = StreamToken::new("hello".to_string(), 0);
        tx.send(token.clone()).await.unwrap();

        // Receive the token
        let received = rx.recv().await.unwrap();
        assert_eq!(received.content, "hello");
        assert_eq!(received.sequence, 0);
    }

    #[tokio::test]
    async fn test_stream_channel_buffer() {
        let config = StreamConfig {
            buffer_size: 2,
            ..Default::default()
        };
        let (tx, mut rx) = create_stream_channel(config);

        // Send multiple tokens within buffer
        tx.send(StreamToken::new("1".to_string(), 0)).await.unwrap();
        tx.send(StreamToken::new("2".to_string(), 1)).await.unwrap();

        // Receive them
        assert_eq!(rx.recv().await.unwrap().content, "1");
        assert_eq!(rx.recv().await.unwrap().content, "2");
    }
}
