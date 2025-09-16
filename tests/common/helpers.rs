/// Helper functions for integration tests

use anyhow::Result;
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{fs, time::timeout};

/// File system utilities for tests
pub struct FsHelpers;

impl FsHelpers {
    /// Create a directory structure for testing
    pub async fn create_test_dirs(base_path: &PathBuf, dirs: &[&str]) -> Result<Vec<PathBuf>> {
        let mut created_dirs = Vec::new();

        for dir_name in dirs {
            let dir_path = base_path.join(dir_name);
            fs::create_dir_all(&dir_path).await?;
            created_dirs.push(dir_path);
        }

        Ok(created_dirs)
    }

    /// Wait for a file to exist with timeout
    pub async fn wait_for_file(path: &PathBuf, timeout_duration: Duration) -> Result<bool> {
        let start = Instant::now();

        while start.elapsed() < timeout_duration {
            if path.exists() {
                return Ok(true);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        Ok(false)
    }

    /// Wait for a directory to contain a minimum number of files
    pub async fn wait_for_files_in_dir(
        dir_path: &PathBuf,
        min_count: usize,
        timeout_duration: Duration,
    ) -> Result<bool> {
        let start = Instant::now();

        while start.elapsed() < timeout_duration {
            if let Ok(mut entries) = fs::read_dir(dir_path).await {
                let mut count = 0;
                while let Ok(Some(_)) = entries.next_entry().await {
                    count += 1;
                }
                if count >= min_count {
                    return Ok(true);
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(false)
    }

    /// Get file count in directory
    pub async fn count_files_in_dir(dir_path: &PathBuf) -> Result<usize> {
        let mut count = 0;
        if let Ok(mut entries) = fs::read_dir(dir_path).await {
            while let Ok(Some(_)) = entries.next_entry().await {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Get total size of files in directory
    pub async fn get_dir_size(dir_path: &PathBuf) -> Result<u64> {
        let mut total_size = 0;
        if let Ok(mut entries) = fs::read_dir(dir_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
                    total_size += metadata.len();
                }
            }
        }
        Ok(total_size)
    }

    /// Clean up directory by removing all files
    pub async fn clean_dir(dir_path: &PathBuf) -> Result<()> {
        if dir_path.exists() {
            let mut entries = fs::read_dir(dir_path).await?;
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(path).await?;
                } else if path.is_dir() {
                    fs::remove_dir_all(path).await?;
                }
            }
        }
        Ok(())
    }
}

/// Timing and performance utilities
pub struct TimingHelpers;

impl TimingHelpers {
    /// Measure execution time of an async operation
    pub async fn measure_async<F, T, Fut>(operation: F) -> (T, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start = Instant::now();
        let result = operation().await;
        let duration = start.elapsed();
        (result, duration)
    }

    /// Wait for a condition to be true with timeout
    pub async fn wait_for_condition<F, Fut>(
        mut condition: F,
        timeout_duration: Duration,
        check_interval: Duration,
    ) -> Result<bool>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start = Instant::now();

        while start.elapsed() < timeout_duration {
            if condition().await {
                return Ok(true);
            }
            tokio::time::sleep(check_interval).await;
        }

        Ok(false)
    }

    /// Run operation with timeout
    pub async fn with_timeout<F, T, Fut>(
        operation: F,
        timeout_duration: Duration,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        timeout(timeout_duration, operation()).await?
    }

    /// Retry operation with exponential backoff
    pub async fn retry_with_backoff<F, T, Fut>(
        mut operation: F,
        max_retries: usize,
        initial_delay: Duration,
        max_delay: Duration,
    ) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut delay = initial_delay;
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        tokio::time::sleep(delay).await;
                        delay = std::cmp::min(delay * 2, max_delay);
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }
}

/// Concurrency testing utilities
pub struct ConcurrencyHelpers;

impl ConcurrencyHelpers {
    /// Run multiple async operations concurrently
    pub async fn run_concurrent<F, T, Fut>(
        operations: Vec<F>,
    ) -> Vec<Result<T>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        let tasks: Vec<_> = operations
            .into_iter()
            .map(|op| tokio::spawn(op()))
            .collect();

        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(join_error) => results.push(Err(anyhow::anyhow!("Task join error: {}", join_error))),
            }
        }

        results
    }

    /// Run operations with controlled concurrency
    pub async fn run_with_concurrency_limit<F, T, Fut>(
        operations: Vec<F>,
        max_concurrent: usize,
    ) -> Vec<Result<T>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        use tokio::sync::Semaphore;

        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let tasks: Vec<_> = operations
            .into_iter()
            .map(|op| {
                let semaphore = semaphore.clone();
                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    op().await
                })
            })
            .collect();

        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(join_error) => results.push(Err(anyhow::anyhow!("Task join error: {}", join_error))),
            }
        }

        results
    }

    /// Stress test by running operations repeatedly
    pub async fn stress_test<F, T, Fut>(
        operation: F,
        num_iterations: usize,
        max_concurrent: usize,
    ) -> (Vec<Result<T>>, Duration)
    where
        F: Fn() -> Fut + Send + Sync + Clone,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        let start = Instant::now();

        let operations: Vec<_> = (0..num_iterations)
            .map(|_| operation.clone())
            .collect();

        let results = Self::run_with_concurrency_limit(operations, max_concurrent).await;
        let duration = start.elapsed();

        (results, duration)
    }
}

/// Network and API testing utilities
pub struct NetworkHelpers;

impl NetworkHelpers {
    /// Find an available port for testing
    pub fn find_available_port() -> Result<u16> {
        use std::net::{TcpListener, SocketAddr};

        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        Ok(addr.port())
    }

    /// Wait for a TCP port to be available
    pub async fn wait_for_port(port: u16, timeout_duration: Duration) -> Result<bool> {
        use tokio::net::TcpStream;

        let start = Instant::now();

        while start.elapsed() < timeout_duration {
            match TcpStream::connect(format!("127.0.0.1:{}", port)).await {
                Ok(_) => return Ok(true),
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }

        Ok(false)
    }

    /// Create HTTP client for testing
    pub fn create_test_client() -> reqwest::Client {
        reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client")
    }
}

/// Data validation utilities
pub struct ValidationHelpers;

impl ValidationHelpers {
    /// Validate JSON structure
    pub fn validate_json_structure(
        json: &serde_json::Value,
        required_fields: &[&str],
    ) -> Result<()> {
        for field in required_fields {
            if !json.get(field).is_some() {
                return Err(anyhow::anyhow!("Missing required field: {}", field));
            }
        }
        Ok(())
    }

    /// Validate numeric range
    pub fn validate_numeric_range(value: f64, min: f64, max: f64, field_name: &str) -> Result<()> {
        if value < min || value > max {
            return Err(anyhow::anyhow!(
                "{} value {} is out of range [{}, {}]",
                field_name,
                value,
                min,
                max
            ));
        }
        Ok(())
    }

    /// Validate string is not empty
    pub fn validate_non_empty_string(value: &str, field_name: &str) -> Result<()> {
        if value.trim().is_empty() {
            return Err(anyhow::anyhow!("{} cannot be empty", field_name));
        }
        Ok(())
    }

    /// Validate file extension
    pub fn validate_file_extension(path: &PathBuf, expected_extensions: &[&str]) -> Result<()> {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if expected_extensions.contains(&ext) {
                return Ok(());
            }
        }

        Err(anyhow::anyhow!(
            "File {:?} does not have expected extension (expected one of: {:?})",
            path,
            expected_extensions
        ))
    }
}

/// Resource monitoring utilities
pub struct ResourceHelpers;

impl ResourceHelpers {
    /// Monitor memory usage during operation
    pub async fn monitor_memory_usage<F, T, Fut>(
        operation: F,
        monitoring_interval: Duration,
    ) -> (T, Vec<f64>)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let memory_samples = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let samples_clone = memory_samples.clone();

        // Start memory monitoring
        let monitoring_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(monitoring_interval);
            loop {
                interval.tick().await;
                let memory_usage = Self::get_current_memory_usage();
                samples_clone.lock().await.push(memory_usage);
            }
        });

        // Run the operation
        let result = operation().await;

        // Stop monitoring
        monitoring_task.abort();

        let samples = memory_samples.lock().await.clone();
        (result, samples)
    }

    /// Get current memory usage (simplified for testing)
    fn get_current_memory_usage() -> f64 {
        // In a real implementation, this would read from system metrics
        // For testing, we return a mock value
        100.0 // MB
    }

    /// Check if system has sufficient resources for test
    pub fn check_system_resources(min_memory_mb: u64, min_disk_space_mb: u64) -> Result<()> {
        // In a real implementation, this would check actual system resources
        // For testing, we assume resources are sufficient
        if min_memory_mb > 16384 {
            return Err(anyhow::anyhow!("Test requires too much memory: {} MB", min_memory_mb));
        }

        if min_disk_space_mb > 10240 {
            return Err(anyhow::anyhow!("Test requires too much disk space: {} MB", min_disk_space_mb));
        }

        Ok(())
    }
}

/// Logging and debugging utilities
pub struct LoggingHelpers;

impl LoggingHelpers {
    /// Initialize test logging
    pub fn init_test_logging() {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "info");
        }

        let _ = tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .try_init();
    }

    /// Log test progress
    pub fn log_test_progress(test_name: &str, progress: &str) {
        if std::env::var("INFERNO_TEST_VERBOSE").is_ok() {
            println!("[{}] {}", test_name, progress);
        }
    }

    /// Log performance metrics
    pub fn log_performance_metrics(test_name: &str, metrics: &std::collections::HashMap<String, f64>) {
        if std::env::var("INFERNO_TEST_VERBOSE").is_ok() {
            println!("[{}] Performance Metrics:", test_name);
            for (metric, value) in metrics {
                println!("  {}: {:.2}", metric, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_fs_helpers() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path().to_path_buf();

        // Test directory creation
        let dirs = FsHelpers::create_test_dirs(&base_path, &["test1", "test2", "test3"]).await?;
        assert_eq!(dirs.len(), 3);
        for dir in &dirs {
            assert!(dir.exists());
        }

        // Test file count
        let count = FsHelpers::count_files_in_dir(&base_path).await?;
        assert_eq!(count, 3); // 3 directories

        Ok(())
    }

    #[tokio::test]
    async fn test_timing_helpers() -> Result<()> {
        // Test measure_async
        let (result, duration) = TimingHelpers::measure_async(|| async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            42
        }).await;

        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(90)); // Allow some tolerance

        // Test wait_for_condition
        let mut counter = 0;
        let condition_met = TimingHelpers::wait_for_condition(
            || async {
                counter += 1;
                counter >= 3
            },
            Duration::from_secs(1),
            Duration::from_millis(10),
        ).await?;

        assert!(condition_met);
        assert_eq!(counter, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_concurrency_helpers() -> Result<()> {
        // Test concurrent operations
        let operations = vec![
            || async { tokio::time::sleep(Duration::from_millis(50)).await; Ok(1) },
            || async { tokio::time::sleep(Duration::from_millis(50)).await; Ok(2) },
            || async { tokio::time::sleep(Duration::from_millis(50)).await; Ok(3) },
        ];

        let results = ConcurrencyHelpers::run_concurrent(operations).await;
        assert_eq!(results.len(), 3);
        for result in results {
            assert!(result.is_ok());
        }

        Ok(())
    }

    #[test]
    fn test_validation_helpers() -> Result<()> {
        // Test JSON validation
        let json = serde_json::json!({
            "name": "test",
            "value": 42
        });

        ValidationHelpers::validate_json_structure(&json, &["name", "value"])?;

        let result = ValidationHelpers::validate_json_structure(&json, &["name", "missing"]);
        assert!(result.is_err());

        // Test numeric range validation
        ValidationHelpers::validate_numeric_range(5.0, 0.0, 10.0, "test_value")?;

        let result = ValidationHelpers::validate_numeric_range(15.0, 0.0, 10.0, "test_value");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_network_helpers() -> Result<()> {
        let port = NetworkHelpers::find_available_port()?;
        assert!(port > 0);
        assert!(port < 65536);

        Ok(())
    }
}