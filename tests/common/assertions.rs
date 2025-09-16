/// Custom assertions for integration tests

use anyhow::Result;
use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, SystemTime},
};

/// Performance assertion utilities
pub struct PerformanceAssertions;

impl PerformanceAssertions {
    /// Assert that operation completed within expected time
    pub fn assert_duration_within(
        actual: Duration,
        expected: Duration,
        tolerance_percent: f64,
        operation: &str,
    ) -> Result<()> {
        let tolerance = expected.mul_f64(tolerance_percent / 100.0);
        let min_duration = expected.saturating_sub(tolerance);
        let max_duration = expected + tolerance;

        if actual < min_duration || actual > max_duration {
            return Err(anyhow::anyhow!(
                "{} took {:?}, expected {:?} ± {}%",
                operation,
                actual,
                expected,
                tolerance_percent
            ));
        }

        Ok(())
    }

    /// Assert throughput meets minimum requirement
    pub fn assert_min_throughput(
        operations: u64,
        duration: Duration,
        min_ops_per_sec: f64,
        operation_type: &str,
    ) -> Result<()> {
        let actual_throughput = operations as f64 / duration.as_secs_f64();

        if actual_throughput < min_ops_per_sec {
            return Err(anyhow::anyhow!(
                "{} throughput {:.2} ops/sec is below minimum {:.2} ops/sec",
                operation_type,
                actual_throughput,
                min_ops_per_sec
            ));
        }

        Ok(())
    }

    /// Assert latency percentiles are within bounds
    pub fn assert_latency_percentiles(
        latencies: &[Duration],
        p95_max: Duration,
        p99_max: Duration,
        operation: &str,
    ) -> Result<()> {
        if latencies.is_empty() {
            return Err(anyhow::anyhow!("No latency data for {}", operation));
        }

        let mut sorted_latencies = latencies.to_vec();
        sorted_latencies.sort();

        let p95_index = (sorted_latencies.len() as f64 * 0.95) as usize;
        let p99_index = (sorted_latencies.len() as f64 * 0.99) as usize;

        let p95_latency = sorted_latencies.get(p95_index.min(sorted_latencies.len() - 1))
            .unwrap_or(&Duration::ZERO);
        let p99_latency = sorted_latencies.get(p99_index.min(sorted_latencies.len() - 1))
            .unwrap_or(&Duration::ZERO);

        if *p95_latency > p95_max {
            return Err(anyhow::anyhow!(
                "{} P95 latency {:?} exceeds maximum {:?}",
                operation,
                p95_latency,
                p95_max
            ));
        }

        if *p99_latency > p99_max {
            return Err(anyhow::anyhow!(
                "{} P99 latency {:?} exceeds maximum {:?}",
                operation,
                p99_latency,
                p99_max
            ));
        }

        Ok(())
    }

    /// Assert error rate is below threshold
    pub fn assert_error_rate_below(
        total_operations: u64,
        failed_operations: u64,
        max_error_rate_percent: f64,
        operation: &str,
    ) -> Result<()> {
        if total_operations == 0 {
            return Err(anyhow::anyhow!("No operations recorded for {}", operation));
        }

        let error_rate = (failed_operations as f64 / total_operations as f64) * 100.0;

        if error_rate > max_error_rate_percent {
            return Err(anyhow::anyhow!(
                "{} error rate {:.2}% exceeds maximum {:.2}%",
                operation,
                error_rate,
                max_error_rate_percent
            ));
        }

        Ok(())
    }

    /// Assert memory usage is within bounds
    pub fn assert_memory_usage_within(
        peak_memory_mb: f64,
        max_memory_mb: f64,
        operation: &str,
    ) -> Result<()> {
        if peak_memory_mb > max_memory_mb {
            return Err(anyhow::anyhow!(
                "{} peak memory usage {:.2}MB exceeds maximum {:.2}MB",
                operation,
                peak_memory_mb,
                max_memory_mb
            ));
        }

        Ok(())
    }
}

/// Data integrity assertion utilities
pub struct DataAssertions;

impl DataAssertions {
    /// Assert two collections contain the same elements (order independent)
    pub fn assert_collections_equivalent<T>(
        actual: &[T],
        expected: &[T],
        description: &str,
    ) -> Result<()>
    where
        T: PartialEq + std::fmt::Debug,
    {
        if actual.len() != expected.len() {
            return Err(anyhow::anyhow!(
                "{}: length mismatch - actual: {}, expected: {}",
                description,
                actual.len(),
                expected.len()
            ));
        }

        for item in expected {
            if !actual.contains(item) {
                return Err(anyhow::anyhow!(
                    "{}: missing expected item: {:?}",
                    description,
                    item
                ));
            }
        }

        Ok(())
    }

    /// Assert JSON contains expected structure
    pub fn assert_json_structure(
        json: &serde_json::Value,
        required_fields: &[&str],
        optional_fields: &[&str],
        description: &str,
    ) -> Result<()> {
        // Check required fields
        for field in required_fields {
            if !json.get(field).is_some() {
                return Err(anyhow::anyhow!(
                    "{}: missing required field '{}'",
                    description,
                    field
                ));
            }
        }

        // Check that no unexpected fields exist (beyond required and optional)
        if let Some(obj) = json.as_object() {
            let all_allowed: std::collections::HashSet<_> = required_fields
                .iter()
                .chain(optional_fields.iter())
                .collect();

            for key in obj.keys() {
                if !all_allowed.contains(&key.as_str()) {
                    return Err(anyhow::anyhow!(
                        "{}: unexpected field '{}'",
                        description,
                        key
                    ));
                }
            }
        }

        Ok(())
    }

    /// Assert string matches pattern
    pub fn assert_string_matches_pattern(
        actual: &str,
        pattern: &regex::Regex,
        description: &str,
    ) -> Result<()> {
        if !pattern.is_match(actual) {
            return Err(anyhow::anyhow!(
                "{}: string '{}' does not match pattern '{}'",
                description,
                actual,
                pattern.as_str()
            ));
        }

        Ok(())
    }

    /// Assert file exists and has expected properties
    pub fn assert_file_properties(
        path: &PathBuf,
        min_size: Option<u64>,
        max_size: Option<u64>,
        expected_extension: Option<&str>,
    ) -> Result<()> {
        if !path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {:?}", path));
        }

        if !path.is_file() {
            return Err(anyhow::anyhow!("Path is not a file: {:?}", path));
        }

        let metadata = std::fs::metadata(path)?;
        let file_size = metadata.len();

        if let Some(min) = min_size {
            if file_size < min {
                return Err(anyhow::anyhow!(
                    "File {:?} size {} bytes is below minimum {} bytes",
                    path,
                    file_size,
                    min
                ));
            }
        }

        if let Some(max) = max_size {
            if file_size > max {
                return Err(anyhow::anyhow!(
                    "File {:?} size {} bytes exceeds maximum {} bytes",
                    path,
                    file_size,
                    max
                ));
            }
        }

        if let Some(ext) = expected_extension {
            let actual_ext = path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            if actual_ext != ext {
                return Err(anyhow::anyhow!(
                    "File {:?} has extension '{}', expected '{}'",
                    path,
                    actual_ext,
                    ext
                ));
            }
        }

        Ok(())
    }

    /// Assert directory contains expected files
    pub fn assert_directory_contains(
        dir_path: &PathBuf,
        expected_files: &[&str],
        exact_match: bool,
    ) -> Result<()> {
        if !dir_path.exists() {
            return Err(anyhow::anyhow!("Directory does not exist: {:?}", dir_path));
        }

        if !dir_path.is_dir() {
            return Err(anyhow::anyhow!("Path is not a directory: {:?}", dir_path));
        }

        let entries: Vec<String> = std::fs::read_dir(dir_path)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    e.file_name().to_str().map(|s| s.to_string())
                })
            })
            .collect();

        for expected_file in expected_files {
            if !entries.contains(&expected_file.to_string()) {
                return Err(anyhow::anyhow!(
                    "Directory {:?} missing expected file '{}'",
                    dir_path,
                    expected_file
                ));
            }
        }

        if exact_match && entries.len() != expected_files.len() {
            return Err(anyhow::anyhow!(
                "Directory {:?} contains {} files, expected exactly {}",
                dir_path,
                entries.len(),
                expected_files.len()
            ));
        }

        Ok(())
    }
}

/// System state assertion utilities
pub struct SystemAssertions;

impl SystemAssertions {
    /// Assert component is in expected state
    pub fn assert_component_state<T>(
        actual_state: &T,
        expected_state: &T,
        component_name: &str,
    ) -> Result<()>
    where
        T: PartialEq + std::fmt::Debug,
    {
        if actual_state != expected_state {
            return Err(anyhow::anyhow!(
                "{} state mismatch - actual: {:?}, expected: {:?}",
                component_name,
                actual_state,
                expected_state
            ));
        }

        Ok(())
    }

    /// Assert metrics are within expected ranges
    pub fn assert_metrics_within_range(
        metrics: &HashMap<String, f64>,
        expected_ranges: &HashMap<String, (f64, f64)>,
        component_name: &str,
    ) -> Result<()> {
        for (metric_name, (min_val, max_val)) in expected_ranges {
            if let Some(&actual_val) = metrics.get(metric_name) {
                if actual_val < *min_val || actual_val > *max_val {
                    return Err(anyhow::anyhow!(
                        "{} metric '{}' value {} is outside expected range [{}, {}]",
                        component_name,
                        metric_name,
                        actual_val,
                        min_val,
                        max_val
                    ));
                }
            } else {
                return Err(anyhow::anyhow!(
                    "{} missing required metric '{}'",
                    component_name,
                    metric_name
                ));
            }
        }

        Ok(())
    }

    /// Assert cache statistics are reasonable
    pub fn assert_cache_stats_reasonable(
        hits: u64,
        misses: u64,
        min_hit_rate: f64,
        cache_name: &str,
    ) -> Result<()> {
        let total = hits + misses;
        if total == 0 {
            return Err(anyhow::anyhow!(
                "{} cache has no recorded operations",
                cache_name
            ));
        }

        let hit_rate = hits as f64 / total as f64;
        if hit_rate < min_hit_rate {
            return Err(anyhow::anyhow!(
                "{} cache hit rate {:.2}% is below minimum {:.2}%",
                cache_name,
                hit_rate * 100.0,
                min_hit_rate * 100.0
            ));
        }

        Ok(())
    }

    /// Assert audit trail completeness
    pub fn assert_audit_trail_complete(
        events: &[inferno::audit::AuditEvent],
        expected_event_types: &[inferno::audit::EventType],
        min_events_per_type: usize,
    ) -> Result<()> {
        for expected_type in expected_event_types {
            let count = events
                .iter()
                .filter(|e| std::mem::discriminant(&e.event_type) == std::mem::discriminant(expected_type))
                .count();

            if count < min_events_per_type {
                return Err(anyhow::anyhow!(
                    "Audit trail missing sufficient events of type {:?} - found {}, expected at least {}",
                    expected_type,
                    count,
                    min_events_per_type
                ));
            }
        }

        Ok(())
    }

    /// Assert time-based consistency
    pub fn assert_time_consistency(
        events: &[(SystemTime, &str)],
        max_time_drift: Duration,
    ) -> Result<()> {
        if events.len() < 2 {
            return Ok(()); // Cannot check consistency with fewer than 2 events
        }

        let mut previous_time = events[0].0;
        for (current_time, event_name) in &events[1..] {
            if *current_time < previous_time {
                return Err(anyhow::anyhow!(
                    "Time inconsistency detected: event '{}' has earlier timestamp than previous event",
                    event_name
                ));
            }

            let time_diff = current_time.duration_since(previous_time)
                .unwrap_or(Duration::ZERO);

            if time_diff > max_time_drift {
                return Err(anyhow::anyhow!(
                    "Time drift too large for event '{}': {:?} > {:?}",
                    event_name,
                    time_diff,
                    max_time_drift
                ));
            }

            previous_time = *current_time;
        }

        Ok(())
    }
}

/// Macro for creating custom assertions
#[macro_export]
macro_rules! assert_eventually {
    ($condition:expr, $timeout:expr, $message:expr) => {
        {
            let start = std::time::Instant::now();
            let timeout_duration = $timeout;
            let mut last_error = None;

            while start.elapsed() < timeout_duration {
                match $condition {
                    Ok(_) => break,
                    Err(e) => {
                        last_error = Some(e);
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }

            if let Some(error) = last_error {
                return Err(anyhow::anyhow!("{}: {}", $message, error));
            }
        }
    };
}

/// Macro for asserting approximate equality of floating point numbers
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $tolerance:expr) => {
        {
            let diff = ($left - $right).abs();
            if diff > $tolerance {
                return Err(anyhow::anyhow!(
                    "Assertion failed: {} ≈ {} (tolerance: {}), actual difference: {}",
                    $left,
                    $right,
                    $tolerance,
                    diff
                ));
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_performance_assertions() -> Result<()> {
        // Test duration assertion
        let actual = Duration::from_millis(150);
        let expected = Duration::from_millis(100);

        PerformanceAssertions::assert_duration_within(actual, expected, 60.0, "test_op")?;

        let result = PerformanceAssertions::assert_duration_within(actual, expected, 30.0, "test_op");
        assert!(result.is_err());

        // Test throughput assertion
        PerformanceAssertions::assert_min_throughput(100, Duration::from_secs(1), 90.0, "test_ops")?;

        let result = PerformanceAssertions::assert_min_throughput(100, Duration::from_secs(1), 110.0, "test_ops");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_data_assertions() -> Result<()> {
        // Test collection equivalence
        let actual = vec![1, 2, 3];
        let expected = vec![3, 1, 2];
        DataAssertions::assert_collections_equivalent(&actual, &expected, "test_collection")?;

        let wrong = vec![1, 2, 4];
        let result = DataAssertions::assert_collections_equivalent(&actual, &wrong, "test_collection");
        assert!(result.is_err());

        // Test JSON structure
        let json = serde_json::json!({
            "name": "test",
            "value": 42,
            "optional": "data"
        });

        DataAssertions::assert_json_structure(
            &json,
            &["name", "value"],
            &["optional"],
            "test_json"
        )?;

        let result = DataAssertions::assert_json_structure(
            &json,
            &["name", "missing"],
            &[],
            "test_json"
        );
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_file_assertions() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "Hello, world!")?;

        // Test file properties
        DataAssertions::assert_file_properties(&test_file, Some(10), Some(20), Some("txt"))?;

        let result = DataAssertions::assert_file_properties(&test_file, Some(20), None, None);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_system_assertions() -> Result<()> {
        // Test component state
        let actual_state = "running";
        let expected_state = "running";
        SystemAssertions::assert_component_state(&actual_state, &expected_state, "test_component")?;

        let wrong_state = "stopped";
        let result = SystemAssertions::assert_component_state(&actual_state, &wrong_state, "test_component");
        assert!(result.is_err());

        // Test metrics ranges
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage".to_string(), 75.0);
        metrics.insert("memory_usage".to_string(), 512.0);

        let mut ranges = HashMap::new();
        ranges.insert("cpu_usage".to_string(), (0.0, 100.0));
        ranges.insert("memory_usage".to_string(), (0.0, 1024.0));

        SystemAssertions::assert_metrics_within_range(&metrics, &ranges, "test_component")?;

        ranges.insert("cpu_usage".to_string(), (80.0, 100.0));
        let result = SystemAssertions::assert_metrics_within_range(&metrics, &ranges, "test_component");
        assert!(result.is_err());

        Ok(())
    }
}