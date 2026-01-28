//! Performance Statistics & Analytics
//!
//! Aggregates profiling data into actionable insights with percentile analysis
//! and time-window based statistics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Duration statistics with percentiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationStats {
    /// Minimum duration in milliseconds
    pub min: f32,
    /// 50th percentile (median)
    pub p50: f32,
    /// 95th percentile
    pub p95: f32,
    /// 99th percentile
    pub p99: f32,
    /// Maximum duration
    pub max: f32,
    /// Mean/average duration
    pub mean: f32,
    /// Standard deviation
    pub stddev: f32,
}

impl Default for DurationStats {
    fn default() -> Self {
        Self {
            min: 0.0,
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
            max: 0.0,
            mean: 0.0,
            stddev: 0.0,
        }
    }
}

impl DurationStats {
    /// Calculate statistics from a list of durations
    pub fn from_durations(durations: &[f32]) -> Self {
        if durations.is_empty() {
            return Self::default();
        }

        let mut sorted = durations.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let sum: f32 = sorted.iter().sum();
        let mean = sum / sorted.len() as f32;

        // Percentiles - use proper median calculation for even-length arrays
        let p50 = if sorted.len().is_multiple_of(2) {
            let mid = sorted.len() / 2;
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let p95_idx = ((sorted.len() as f32 * 0.95) as usize).min(sorted.len() - 1);
        let p95 = sorted[p95_idx];

        let p99_idx = ((sorted.len() as f32 * 0.99) as usize).min(sorted.len() - 1);
        let p99 = sorted[p99_idx];

        // Standard deviation
        let variance = sorted.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / sorted.len() as f32;
        let stddev = variance.sqrt();

        Self {
            min,
            p50,
            p95,
            p99,
            max,
            mean,
            stddev,
        }
    }
}

/// Per-phase statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseStats {
    /// Phase name (tokenization, inference, detokenization)
    pub phase: String,
    /// Number of observations
    pub count: u32,
    /// Duration statistics
    pub duration_ms: DurationStats,
    /// GPU memory statistics (if available)
    pub gpu_memory_mb: Option<DurationStats>,
    /// GPU utilization statistics (if available)
    pub gpu_utilization_percent: Option<DurationStats>,
    /// Throughput in tokens per second (for inference phase)
    pub throughput_tokens_per_sec: f32,
}

/// Time window aggregation types
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum TimeWindow {
    OneMinute,
    FiveMinutes,
    OneHour,
    AllTime,
}

impl TimeWindow {
    /// Get window duration in seconds
    pub fn seconds(&self) -> u64 {
        match self {
            TimeWindow::OneMinute => 60,
            TimeWindow::FiveMinutes => 300,
            TimeWindow::OneHour => 3600,
            TimeWindow::AllTime => u64::MAX,
        }
    }
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetection {
    pub metric_name: String,
    pub is_anomaly: bool,
    pub current_value: f32,
    pub baseline_value: f32,
    pub deviation_percent: f32,
}

/// Performance statistics aggregator
#[derive(Debug, Clone)]
pub struct StatisticsAggregator {
    /// Per-phase durations
    phase_durations: HashMap<String, Vec<f32>>,
    /// Per-model statistics
    per_model_stats: HashMap<String, Vec<f32>>,
    /// Per-priority stats
    per_priority_stats: HashMap<u8, Vec<f32>>,
    /// Baseline for anomaly detection
    baseline_stats: HashMap<String, f32>,
}

impl StatisticsAggregator {
    /// Create a new aggregator
    pub fn new() -> Self {
        Self {
            phase_durations: HashMap::new(),
            per_model_stats: HashMap::new(),
            per_priority_stats: HashMap::new(),
            baseline_stats: HashMap::new(),
        }
    }

    /// Record phase duration
    pub fn record_phase(&mut self, phase_name: String, duration_ms: f32) {
        self.phase_durations
            .entry(phase_name)
            .or_default()
            .push(duration_ms);
    }

    /// Record model-specific measurement
    pub fn record_model(&mut self, model_id: String, duration_ms: f32) {
        self.per_model_stats
            .entry(model_id)
            .or_default()
            .push(duration_ms);
    }

    /// Record priority-level measurement
    pub fn record_priority(&mut self, priority: u8, duration_ms: f32) {
        self.per_priority_stats
            .entry(priority)
            .or_default()
            .push(duration_ms);
    }

    /// Get statistics for a phase
    pub fn get_phase_stats(&self, phase_name: &str) -> Option<PhaseStats> {
        let durations = self.phase_durations.get(phase_name)?;

        if durations.is_empty() {
            return None;
        }

        Some(PhaseStats {
            phase: phase_name.to_string(),
            count: durations.len() as u32,
            duration_ms: DurationStats::from_durations(durations),
            gpu_memory_mb: None,
            gpu_utilization_percent: None,
            throughput_tokens_per_sec: 0.0,
        })
    }

    /// Get all phase statistics
    pub fn get_all_phase_stats(&self) -> Vec<PhaseStats> {
        self.phase_durations
            .keys()
            .filter_map(|phase| self.get_phase_stats(phase))
            .collect()
    }

    /// Detect anomalies by comparing to baseline
    pub fn detect_anomalies(
        &self,
        metric_name: &str,
        current_value: f32,
        threshold_percent: f32,
    ) -> Option<AnomalyDetection> {
        let baseline = *self.baseline_stats.get(metric_name)?;

        if baseline == 0.0 {
            return None;
        }

        let deviation = ((current_value - baseline) / baseline * 100.0).abs();
        let is_anomaly = deviation > threshold_percent;

        Some(AnomalyDetection {
            metric_name: metric_name.to_string(),
            is_anomaly,
            current_value,
            baseline_value: baseline,
            deviation_percent: deviation,
        })
    }

    /// Update baseline statistics
    pub fn update_baseline(&mut self, metric_name: String, value: f32) {
        self.baseline_stats.insert(metric_name, value);
    }

    /// Get percentile value from collected data
    pub fn get_percentile(&self, phase_name: &str, percentile: f32) -> Option<f32> {
        let durations = self.phase_durations.get(phase_name)?;
        if durations.is_empty() {
            return None;
        }

        let mut sorted = durations.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let idx = ((sorted.len() as f32 * percentile / 100.0) as usize).min(sorted.len() - 1);
        Some(sorted[idx])
    }

    /// Calculate trend direction
    pub fn calculate_trend(&self, metric_name: &str) -> Option<TrendDirection> {
        let values = if let Some(vals) = self.phase_durations.get(metric_name) {
            vals.clone()
        } else if let Some(vals) = self.per_model_stats.get(metric_name) {
            vals.clone()
        } else {
            return None;
        };

        if values.len() < 2 {
            return None;
        }

        let mid = values.len() / 2;
        let first_half: f32 = values[..mid].iter().sum::<f32>() / mid as f32;
        let second_half: f32 = values[mid..].iter().sum::<f32>() / (values.len() - mid) as f32;

        let change_percent = ((second_half - first_half) / first_half * 100.0).abs();

        if change_percent < 5.0 {
            return Some(TrendDirection::Stable(change_percent));
        }

        if second_half > first_half {
            Some(TrendDirection::Increasing(change_percent))
        } else {
            Some(TrendDirection::Decreasing(change_percent))
        }
    }

    /// Clear all statistics
    pub fn clear(&mut self) {
        self.phase_durations.clear();
        self.per_model_stats.clear();
        self.per_priority_stats.clear();
    }
}

impl Default for StatisticsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Trend direction with magnitude
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing(f32), // percentage change
    Decreasing(f32), // percentage change
    Stable(f32),     // percentage deviation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_stats_from_data() {
        let durations = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];
        let stats = DurationStats::from_durations(&durations);

        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 100.0);
        assert!((stats.mean - 55.0).abs() < 0.1);
        assert_eq!(stats.p50, 55.0); // Median
    }

    #[test]
    fn test_percentile_calculation() {
        let durations = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let stats = DurationStats::from_durations(&durations);

        assert!(stats.p95 >= 9.0);
        assert!(stats.p99 >= 9.0);
    }

    #[test]
    fn test_statistics_aggregator() {
        let mut agg = StatisticsAggregator::new();

        // Record tokenization times
        for i in 1..=10 {
            agg.record_phase("tokenization".to_string(), (i * 10) as f32);
        }

        let stats = agg.get_phase_stats("tokenization").unwrap();
        assert_eq!(stats.count, 10);
        assert!(stats.duration_ms.min > 0.0);
        assert!(stats.duration_ms.max > 0.0);
    }

    #[test]
    fn test_anomaly_detection() {
        let mut agg = StatisticsAggregator::new();

        agg.update_baseline("latency".to_string(), 100.0);

        // Current value is 20% higher
        let anomaly = agg.detect_anomalies("latency", 120.0, 15.0);
        assert!(anomaly.is_some());
        assert!(anomaly.unwrap().is_anomaly);

        // Within threshold
        let anomaly = agg.detect_anomalies("latency", 105.0, 10.0);
        assert!(anomaly.is_some());
        assert!(!anomaly.unwrap().is_anomaly);
    }

    #[test]
    fn test_trend_detection() {
        let mut agg = StatisticsAggregator::new();

        // Increasing trend
        for i in 0..20 {
            if i < 10 {
                agg.record_phase("latency".to_string(), 100.0);
            } else {
                agg.record_phase("latency".to_string(), 150.0);
            }
        }

        let trend = agg.calculate_trend("latency");
        assert!(trend.is_some());
        if let Some(TrendDirection::Increasing(pct)) = trend {
            assert!(pct > 0.0);
        } else {
            panic!("Expected increasing trend");
        }
    }

    #[test]
    fn test_percentile_queries() {
        let mut agg = StatisticsAggregator::new();

        for i in 1..=100 {
            agg.record_phase("inference".to_string(), i as f32);
        }

        let p95 = agg.get_percentile("inference", 95.0).unwrap();
        assert!(p95 >= 95.0);

        let p50 = agg.get_percentile("inference", 50.0).unwrap();
        assert!(p50 >= 50.0 && p50 <= 51.0); // Close to 50
    }

    #[test]
    fn test_per_model_stats() {
        let mut agg = StatisticsAggregator::new();

        // Model 1 times
        agg.record_model("llama-7b".to_string(), 100.0);
        agg.record_model("llama-7b".to_string(), 120.0);

        // Model 2 times
        agg.record_model("mistral-7b".to_string(), 80.0);
        agg.record_model("mistral-7b".to_string(), 90.0);

        assert!(agg.per_model_stats.get("llama-7b").unwrap().len() == 2);
        assert!(agg.per_model_stats.get("mistral-7b").unwrap().len() == 2);
    }
}
