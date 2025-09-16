/// Common test utilities and fixtures for Inferno integration tests
///
/// This module provides shared test utilities, mock data generators, and helper functions
/// that can be used across all integration test files to ensure consistency and reduce
/// code duplication.

pub mod fixtures;
pub mod mocks;
pub mod helpers;
pub mod assertions;

use anyhow::Result;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tempfile::TempDir;
use tokio::fs;
use uuid::Uuid;

// Re-export commonly used test utilities
pub use fixtures::*;
pub use mocks::*;
pub use helpers::*;
pub use assertions::*;

/// Standard test environment configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Enable debug logging in tests
    pub debug_logging: bool,
    /// Timeout for individual test operations
    pub operation_timeout_secs: u64,
    /// Timeout for test cleanup
    pub cleanup_timeout_secs: u64,
    /// Maximum memory usage for test operations (MB)
    pub max_memory_mb: u64,
    /// Maximum number of concurrent operations
    pub max_concurrency: usize,
    /// Enable performance monitoring during tests
    pub enable_performance_monitoring: bool,
    /// Custom test metadata
    pub metadata: HashMap<String, String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            debug_logging: true,
            operation_timeout_secs: 30,
            cleanup_timeout_secs: 10,
            max_memory_mb: 1024,
            max_concurrency: 10,
            enable_performance_monitoring: true,
            metadata: HashMap::new(),
        }
    }
}

/// Test run context that tracks test execution state
pub struct TestContext {
    pub config: TestConfig,
    pub test_id: String,
    pub temp_dir: TempDir,
    pub start_time: SystemTime,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TestContext {
    /// Create a new test context with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(TestConfig::default())
    }

    /// Create a new test context with custom configuration
    pub fn with_config(config: TestConfig) -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let test_id = Uuid::new_v4().to_string();

        Ok(Self {
            config,
            test_id: test_id.clone(),
            temp_dir,
            start_time: SystemTime::now(),
            metadata: HashMap::from([
                ("test_id".to_string(), serde_json::Value::String(test_id)),
                ("start_time".to_string(), serde_json::Value::String(
                    humantime::format_rfc3339_seconds(SystemTime::now()).to_string()
                )),
            ]),
        })
    }

    /// Get a path within the test temporary directory
    pub fn temp_path(&self, relative_path: &str) -> PathBuf {
        self.temp_dir.path().join(relative_path)
    }

    /// Create a directory in the test temporary space
    pub async fn create_temp_dir(&self, name: &str) -> Result<PathBuf> {
        let path = self.temp_path(name);
        fs::create_dir_all(&path).await?;
        Ok(path)
    }

    /// Add metadata to the test context
    pub fn add_metadata(&mut self, key: &str, value: serde_json::Value) {
        self.metadata.insert(key.to_string(), value);
    }

    /// Get the test execution duration
    pub fn duration(&self) -> Duration {
        self.start_time.elapsed().unwrap_or(Duration::ZERO)
    }

    /// Check if debug logging is enabled
    pub fn is_debug(&self) -> bool {
        self.config.debug_logging
    }
}

/// Test result tracking for performance analysis
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_id: String,
    pub test_name: String,
    pub success: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub metrics: HashMap<String, f64>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TestResult {
    pub fn success(test_id: String, test_name: String, duration: Duration) -> Self {
        Self {
            test_id,
            test_name,
            success: true,
            duration,
            error_message: None,
            metrics: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn failure(test_id: String, test_name: String, duration: Duration, error: String) -> Self {
        Self {
            test_id,
            test_name,
            success: false,
            duration,
            error_message: Some(error),
            metrics: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_metric(&mut self, name: &str, value: f64) {
        self.metrics.insert(name.to_string(), value);
    }
}

/// Macro for creating test functions with automatic result tracking
#[macro_export]
macro_rules! integration_test {
    ($test_name:ident, $test_fn:expr) => {
        #[tokio::test]
        async fn $test_name() -> anyhow::Result<()> {
            use $crate::common::{TestContext, TestResult};

            let ctx = TestContext::new()?;
            let test_id = ctx.test_id.clone();
            let test_name = stringify!($test_name).to_string();
            let start_time = std::time::Instant::now();

            let result = $test_fn(ctx).await;
            let duration = start_time.elapsed();

            match result {
                Ok(_) => {
                    if std::env::var("INFERNO_TEST_VERBOSE").is_ok() {
                        println!("✓ {} completed in {:?}", test_name, duration);
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("✗ {} failed after {:?}: {}", test_name, duration, e);
                    Err(e)
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_creation() -> Result<()> {
        let ctx = TestContext::new()?;
        assert!(!ctx.test_id.is_empty());
        assert!(ctx.temp_dir.path().exists());
        Ok(())
    }

    #[tokio::test]
    async fn test_temp_directory_creation() -> Result<()> {
        let ctx = TestContext::new()?;
        let test_dir = ctx.create_temp_dir("test_subdir").await?;
        assert!(test_dir.exists());
        assert!(test_dir.is_dir());
        Ok(())
    }

    #[tokio::test]
    async fn test_metadata_handling() -> Result<()> {
        let mut ctx = TestContext::new()?;
        ctx.add_metadata("test_key", serde_json::Value::String("test_value".to_string()));

        assert!(ctx.metadata.contains_key("test_key"));
        assert_eq!(ctx.metadata["test_key"], serde_json::Value::String("test_value".to_string()));
        Ok(())
    }
}