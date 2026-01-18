//! Flow Control & Backpressure Handling for WebSocket Streaming
//!
//! Implements adaptive backpressure handling to manage slow clients and prevent
//! memory exhaustion during high-throughput streaming scenarios.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Backpressure status indicators
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum BackpressureLevel {
    /// Normal operation, buffer has capacity
    Healthy,
    /// Buffer approaching limit, apply gentle backpressure
    Moderate,
    /// Buffer nearly full, apply aggressive backpressure
    Critical,
}

/// Flow control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowControlConfig {
    /// Maximum messages in buffer before applying backpressure (default: 1000)
    pub max_pending_messages: u32,
    /// Threshold for moderate backpressure (default: 70% of max)
    pub moderate_threshold_percent: u32,
    /// Threshold for critical backpressure (default: 90% of max)
    pub critical_threshold_percent: u32,
    /// Maximum unacknowledged tokens in flight (default: 10000)
    pub max_unacked_tokens: u32,
    /// Acknowledgment timeout in seconds (default: 30)
    pub ack_timeout_secs: u64,
    /// Inference timeout in seconds (default: 300)
    pub inference_timeout_secs: u64,
    /// Keep-alive interval in seconds (default: 30)
    pub keepalive_interval_secs: u64,
}

impl Default for FlowControlConfig {
    fn default() -> Self {
        Self {
            max_pending_messages: 1000,
            moderate_threshold_percent: 70,
            critical_threshold_percent: 90,
            max_unacked_tokens: 10_000,
            ack_timeout_secs: 30,
            inference_timeout_secs: 300,
            keepalive_interval_secs: 30,
        }
    }
}

/// Per-stream flow control state
#[derive(Debug, Clone)]
pub struct StreamFlowControl {
    config: FlowControlConfig,
    pending_messages: Arc<AtomicU32>,
    unacked_tokens: Arc<AtomicU32>,
    last_ack_timestamp: Arc<std::sync::Mutex<Instant>>,
    stream_start_time: Instant,
}

impl StreamFlowControl {
    /// Create new flow control for a stream
    pub fn new(config: FlowControlConfig) -> Self {
        Self {
            config,
            pending_messages: Arc::new(AtomicU32::new(0)),
            unacked_tokens: Arc::new(AtomicU32::new(0)),
            last_ack_timestamp: Arc::new(std::sync::Mutex::new(Instant::now())),
            stream_start_time: Instant::now(),
        }
    }

    /// Check current backpressure level
    pub fn check_backpressure(&self) -> BackpressureLevel {
        let pending = self.pending_messages.load(Ordering::Relaxed);
        let max = self.config.max_pending_messages;

        let critical_threshold =
            (max as f32 * self.config.critical_threshold_percent as f32 / 100.0) as u32;
        let moderate_threshold =
            (max as f32 * self.config.moderate_threshold_percent as f32 / 100.0) as u32;

        if pending >= critical_threshold {
            BackpressureLevel::Critical
        } else if pending >= moderate_threshold {
            BackpressureLevel::Moderate
        } else {
            BackpressureLevel::Healthy
        }
    }

    /// Add pending message to buffer
    pub fn add_message(&self) -> Result<(), String> {
        let pending = self.pending_messages.fetch_add(1, Ordering::SeqCst);

        if pending >= self.config.max_pending_messages {
            self.pending_messages.fetch_sub(1, Ordering::SeqCst);
            return Err("Buffer full, backpressure triggered".to_string());
        }

        Ok(())
    }

    /// Mark message as sent
    pub fn message_sent(&self) {
        self.pending_messages.fetch_sub(1, Ordering::SeqCst);
    }

    /// Add tokens to unacknowledged counter
    pub fn add_tokens(&self, count: u32) -> Result<(), String> {
        let total = self.unacked_tokens.fetch_add(count, Ordering::SeqCst) + count;

        if total > self.config.max_unacked_tokens {
            self.unacked_tokens.fetch_sub(count, Ordering::SeqCst);
            return Err("Token limit exceeded, backpressure triggered".to_string());
        }

        Ok(())
    }

    /// Acknowledge received tokens
    pub fn ack_tokens(&self, count: u32) {
        self.unacked_tokens.fetch_sub(
            count.min(self.unacked_tokens.load(Ordering::Relaxed)),
            Ordering::SeqCst,
        );

        if let Ok(mut last_ack) = self.last_ack_timestamp.lock() {
            *last_ack = Instant::now();
        }
    }

    /// Check if acknowledgment timeout exceeded
    pub fn is_ack_timeout(&self) -> bool {
        if let Ok(last_ack) = self.last_ack_timestamp.lock() {
            last_ack.elapsed() > Duration::from_secs(self.config.ack_timeout_secs)
        } else {
            false
        }
    }

    /// Check if inference timeout exceeded
    pub fn is_inference_timeout(&self) -> bool {
        self.stream_start_time.elapsed() > Duration::from_secs(self.config.inference_timeout_secs)
    }

    /// Get current buffer utilization percentage
    pub fn buffer_utilization_percent(&self) -> u32 {
        let pending = self.pending_messages.load(Ordering::Relaxed);
        ((pending as f32 / self.config.max_pending_messages as f32) * 100.0) as u32
    }

    /// Get current unacknowledged token count
    pub fn unacked_token_count(&self) -> u32 {
        self.unacked_tokens.load(Ordering::Relaxed)
    }

    /// Get elapsed time since stream start in seconds
    pub fn elapsed_secs(&self) -> u64 {
        self.stream_start_time.elapsed().as_secs()
    }
}

/// Connection pool for managing multiple WebSocket connections
#[derive(Debug, Clone)]
pub struct ConnectionPool {
    /// Maximum concurrent connections
    max_connections: u32,
    /// Current active connections
    active_connections: Arc<AtomicU32>,
    /// Global pending message count
    global_pending_messages: Arc<AtomicU32>,
}

impl ConnectionPool {
    /// Create new connection pool
    pub fn new(max_connections: u32) -> Self {
        Self {
            max_connections,
            active_connections: Arc::new(AtomicU32::new(0)),
            global_pending_messages: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Try to acquire connection slot
    pub fn acquire_connection(&self) -> Result<ConnectionGuard, String> {
        let active = self.active_connections.fetch_add(1, Ordering::SeqCst);

        if active >= self.max_connections {
            self.active_connections.fetch_sub(1, Ordering::SeqCst);
            return Err("Connection pool full".to_string());
        }

        Ok(ConnectionGuard {
            active_connections: self.active_connections.clone(),
        })
    }

    /// Get active connection count
    pub fn active_count(&self) -> u32 {
        self.active_connections.load(Ordering::Relaxed)
    }

    /// Get pool utilization percentage
    pub fn utilization_percent(&self) -> u32 {
        let active = self.active_connections.load(Ordering::Relaxed);
        ((active as f32 / self.max_connections as f32) * 100.0) as u32
    }

    /// Get global pending message count
    pub fn pending_messages(&self) -> u32 {
        self.global_pending_messages.load(Ordering::Relaxed)
    }

    /// Add to global pending count
    pub fn add_pending(&self) {
        self.global_pending_messages.fetch_add(1, Ordering::SeqCst);
    }

    /// Remove from global pending count
    pub fn remove_pending(&self) {
        self.global_pending_messages.fetch_sub(1, Ordering::SeqCst);
    }
}

/// RAII guard for connection slots
pub struct ConnectionGuard {
    active_connections: Arc<AtomicU32>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_control_creation() {
        let config = FlowControlConfig::default();
        let fc = StreamFlowControl::new(config);

        assert_eq!(fc.check_backpressure(), BackpressureLevel::Healthy);
        assert_eq!(fc.buffer_utilization_percent(), 0);
    }

    #[test]
    fn test_backpressure_levels() {
        let config = FlowControlConfig {
            max_pending_messages: 100,
            moderate_threshold_percent: 70,
            critical_threshold_percent: 90,
            ..Default::default()
        };

        let fc = StreamFlowControl::new(config);

        // Add 60 messages - should be healthy
        for _ in 0..60 {
            let _ = fc.add_message();
        }
        assert_eq!(fc.check_backpressure(), BackpressureLevel::Healthy);

        // Add 15 more messages - should be moderate
        for _ in 0..15 {
            let _ = fc.add_message();
        }
        assert_eq!(fc.check_backpressure(), BackpressureLevel::Moderate);

        // Add 15 more messages - should be critical
        for _ in 0..15 {
            let _ = fc.add_message();
        }
        assert_eq!(fc.check_backpressure(), BackpressureLevel::Critical);
    }

    #[test]
    fn test_buffer_overflow() {
        let config = FlowControlConfig {
            max_pending_messages: 10,
            ..Default::default()
        };

        let fc = StreamFlowControl::new(config);

        // Fill buffer
        for _ in 0..10 {
            assert!(fc.add_message().is_ok());
        }

        // Next addition should fail
        assert!(fc.add_message().is_err());
    }

    #[test]
    fn test_token_management() {
        let config = FlowControlConfig {
            max_unacked_tokens: 1000,
            ..Default::default()
        };

        let fc = StreamFlowControl::new(config);

        // Add tokens
        assert!(fc.add_tokens(500).is_ok());
        assert_eq!(fc.unacked_token_count(), 500);

        // Acknowledge some
        fc.ack_tokens(200);
        assert_eq!(fc.unacked_token_count(), 300);

        // Add more up to limit
        assert!(fc.add_tokens(700).is_ok());
        assert_eq!(fc.unacked_token_count(), 1000);

        // Try to exceed limit
        assert!(fc.add_tokens(1).is_err());
    }

    #[test]
    fn test_connection_pool() {
        let pool = ConnectionPool::new(3);

        // Acquire connections
        let conn1 = pool.acquire_connection().unwrap();
        assert_eq!(pool.active_count(), 1);

        let conn2 = pool.acquire_connection().unwrap();
        assert_eq!(pool.active_count(), 2);

        let conn3 = pool.acquire_connection().unwrap();
        assert_eq!(pool.active_count(), 3);

        // Pool is full
        assert!(pool.acquire_connection().is_err());

        // Drop one connection
        drop(conn1);
        assert_eq!(pool.active_count(), 2);

        // Can acquire again
        let _conn4 = pool.acquire_connection().unwrap();
        assert_eq!(pool.active_count(), 3);
    }

    #[test]
    fn test_timeout_detection() {
        let config = FlowControlConfig {
            ack_timeout_secs: 1,
            ..Default::default()
        };

        let fc = StreamFlowControl::new(config);
        assert!(!fc.is_ack_timeout());

        std::thread::sleep(Duration::from_millis(1100));
        assert!(fc.is_ack_timeout());

        fc.ack_tokens(0); // Reset timeout
        assert!(!fc.is_ack_timeout());
    }

    #[test]
    fn test_utilization_percent() {
        let pool = ConnectionPool::new(100);

        assert_eq!(pool.utilization_percent(), 0);

        let _conn1 = pool.acquire_connection().unwrap();
        assert_eq!(pool.utilization_percent(), 1);

        for _ in 0..49 {
            let _ = pool.acquire_connection().unwrap();
        }
        assert_eq!(pool.utilization_percent(), 50);
    }
}
