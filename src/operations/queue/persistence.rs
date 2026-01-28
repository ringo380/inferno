//! Queue Persistence & Recovery
//!
//! This module handles persistent storage of queue state to survive restarts
//! and provides graceful shutdown mechanisms.

use crate::operations::queue::priority_queue::RequestMetadata;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Queue state snapshot for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStateSnapshot {
    pub timestamp_ms: u64,
    pub version: u32,
    pub pending_requests: Vec<RequestMetadata>,
    pub metrics: SnapshotMetrics,
}

/// Metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetrics {
    pub total_queued: u64,
    pub total_processed: u64,
    pub avg_queue_depth: f64,
}

/// Queue persistence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    pub enabled: bool,
    pub checkpoint_path: PathBuf,
    pub compression_level: u32, // 1-22 for zstd
    pub auto_checkpoint_interval_secs: u64,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        let checkpoint_path = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".inferno/queue_state.bin")
        } else {
            PathBuf::from(".inferno/queue_state.bin")
        };

        Self {
            enabled: true,
            checkpoint_path,
            compression_level: 3,
            auto_checkpoint_interval_secs: 300, // 5 minutes
        }
    }
}

impl PersistenceConfig {
    /// Create with custom checkpoint path
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.checkpoint_path = path;
        self
    }

    /// Set compression level (1-22)
    pub fn with_compression_level(mut self, level: u32) -> Self {
        self.compression_level = level.max(1).min(22);
        self
    }

    /// Enable/disable persistence
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Queue persistence manager
pub struct QueuePersistence {
    config: PersistenceConfig,
    last_checkpoint_secs: u64,
}

impl QueuePersistence {
    /// Create new persistence manager
    pub fn new(config: PersistenceConfig) -> Self {
        Self {
            config,
            last_checkpoint_secs: Self::current_timestamp_secs(),
        }
    }

    /// Save queue state to disk
    pub fn save_checkpoint(&mut self, snapshot: &QueueStateSnapshot) -> anyhow::Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Create parent directory if needed
        if let Some(parent) = self.config.checkpoint_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Serialize to JSON
        let json_data = serde_json::to_vec(snapshot)?;

        // Compress with zstd
        let compressed =
            zstd::encode_all(json_data.as_slice(), self.config.compression_level as i32)?;

        let compressed_size = compressed.len();

        // Write to disk
        std::fs::write(&self.config.checkpoint_path, compressed)?;

        self.last_checkpoint_secs = Self::current_timestamp_secs();

        tracing::info!(
            "Queue checkpoint saved: {} pending requests, {} bytes",
            snapshot.pending_requests.len(),
            compressed_size
        );

        Ok(())
    }

    /// Load queue state from disk
    pub fn load_checkpoint(&self) -> anyhow::Result<Option<QueueStateSnapshot>> {
        if !self.config.enabled || !self.config.checkpoint_path.exists() {
            return Ok(None);
        }

        // Read compressed data
        let compressed = std::fs::read(&self.config.checkpoint_path)?;

        // Decompress with zstd
        let json_data = zstd::decode_all(compressed.as_slice())?;

        // Deserialize
        let snapshot: QueueStateSnapshot = serde_json::from_slice(&json_data)?;

        tracing::info!(
            "Queue checkpoint loaded: {} pending requests from {}ms ago",
            snapshot.pending_requests.len(),
            Self::current_timestamp_ms() - snapshot.timestamp_ms
        );

        Ok(Some(snapshot))
    }

    /// Check if checkpoint is needed
    pub fn should_checkpoint(&self, force: bool) -> bool {
        if !self.config.enabled {
            return false;
        }

        if force {
            return true;
        }

        let elapsed_secs = Self::current_timestamp_secs() - self.last_checkpoint_secs;
        elapsed_secs >= self.config.auto_checkpoint_interval_secs
    }

    /// Get current timestamp in milliseconds
    fn current_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Get current timestamp in seconds
    fn current_timestamp_secs() -> u64 {
        Self::current_timestamp_ms() / 1000
    }

    /// Get checkpoint file path
    pub fn checkpoint_path(&self) -> &Path {
        &self.config.checkpoint_path
    }

    /// Delete checkpoint file
    pub fn delete_checkpoint(&self) -> anyhow::Result<()> {
        if self.config.checkpoint_path.exists() {
            std::fs::remove_file(&self.config.checkpoint_path)?;
            tracing::info!("Queue checkpoint deleted");
        }
        Ok(())
    }
}

/// Health check status for queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueHealthStatus {
    pub status: HealthStatus,
    pub queue_depth: usize,
    pub active_workers: usize,
    pub avg_wait_ms: f64,
    pub gpu_memory_free_mb: u32,
    pub last_update_ms_ago: u64,
    pub checkpoint_available: bool,
    pub timestamp_ms: u64,
}

/// Health status levels
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
}

impl QueueHealthStatus {
    /// Create health status
    pub fn new(
        queue_depth: usize,
        active_workers: usize,
        avg_wait_ms: f64,
        gpu_memory_free_mb: u32,
        checkpoint_available: bool,
    ) -> Self {
        let status = match (queue_depth, avg_wait_ms, gpu_memory_free_mb) {
            (_, _, gpu) if gpu < 256 => HealthStatus::Critical,
            (depth, _, _) if depth > 5000 => HealthStatus::Critical,
            (_, wait, _) if wait > 1000.0 => HealthStatus::Critical,
            (depth, wait, _) if depth > 2000 || wait > 500.0 => HealthStatus::Degraded,
            _ => HealthStatus::Healthy,
        };

        Self {
            status,
            queue_depth,
            active_workers,
            avg_wait_ms,
            gpu_memory_free_mb,
            last_update_ms_ago: 0,
            checkpoint_available,
            timestamp_ms: Self::current_timestamp_ms(),
        }
    }

    fn current_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Shutdown coordinator for graceful queue shutdown
pub struct ShutdownCoordinator {
    graceful_timeout_secs: u64,
}

impl ShutdownCoordinator {
    /// Create new shutdown coordinator
    pub fn new(graceful_timeout_secs: u64) -> Self {
        Self {
            graceful_timeout_secs,
        }
    }

    /// Log shutdown stats
    pub fn log_shutdown_stats(&self, processed: u64, pending: usize, elapsed_secs: u64) {
        tracing::info!(
            "Queue shutdown: {} processed, {} pending, {}s elapsed (timeout: {}s)",
            processed,
            pending,
            elapsed_secs,
            self.graceful_timeout_secs
        );

        if elapsed_secs > self.graceful_timeout_secs {
            tracing::warn!(
                "Shutdown timeout exceeded: forced termination of {} pending requests",
                pending
            );
        }
    }

    /// Check if timeout exceeded
    pub fn is_timeout_exceeded(&self, elapsed_secs: u64) -> bool {
        elapsed_secs > self.graceful_timeout_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::queue::Priority;
    use std::fs;

    #[test]
    fn test_persistence_config() {
        let config = PersistenceConfig::default();
        assert!(config.enabled);
        assert_eq!(config.compression_level, 3);
    }

    #[test]
    fn test_queue_health_status() {
        let health = QueueHealthStatus::new(100, 4, 250.0, 1024, true);
        assert_eq!(health.status, HealthStatus::Healthy);

        let health = QueueHealthStatus::new(6000, 2, 600.0, 512, true);
        assert_eq!(health.status, HealthStatus::Critical);

        let health = QueueHealthStatus::new(3000, 4, 400.0, 512, true);
        assert_eq!(health.status, HealthStatus::Degraded);
    }

    #[test]
    fn test_shutdown_coordinator() {
        let coordinator = ShutdownCoordinator::new(30);

        assert!(!coordinator.is_timeout_exceeded(20));
        assert!(coordinator.is_timeout_exceeded(40));
    }

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = QueueStateSnapshot {
            timestamp_ms: 1234567890,
            version: 1,
            pending_requests: vec![RequestMetadata::new(
                "test_req".to_string(),
                "user".to_string(),
                Priority::Normal,
                "model".to_string(),
            )],
            metrics: SnapshotMetrics {
                total_queued: 100,
                total_processed: 95,
                avg_queue_depth: 5.0,
            },
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: QueueStateSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.timestamp_ms, snapshot.timestamp_ms);
        assert_eq!(deserialized.pending_requests.len(), 1);
    }

    #[test]
    fn test_persistence_disabled() {
        let config = PersistenceConfig::default().with_enabled(false);
        let mut persistence = QueuePersistence::new(config);

        let snapshot = QueueStateSnapshot {
            timestamp_ms: 1234567890,
            version: 1,
            pending_requests: vec![],
            metrics: SnapshotMetrics {
                total_queued: 0,
                total_processed: 0,
                avg_queue_depth: 0.0,
            },
        };

        // Should succeed silently
        assert!(persistence.save_checkpoint(&snapshot).is_ok());
    }

    #[test]
    fn test_compression_levels() {
        for level in [1, 3, 10, 22] {
            let config = PersistenceConfig::default().with_compression_level(level);
            assert!(config.compression_level >= 1 && config.compression_level <= 22);
        }
    }
}
