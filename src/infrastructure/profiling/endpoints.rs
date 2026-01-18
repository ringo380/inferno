//! Profiling API Endpoints for Dashboard
//!
//! Provides REST API endpoints for profiling data visualization

use crate::infrastructure::profiling::{
    AnomalyDetection, BenchmarkResult, InferenceProfile, PhaseStats,
};
use serde::{Deserialize, Serialize};

/// Request to export profiling data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub time_range_minutes: Option<u32>,
}

/// Export format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    CSV,
    JSON,
    PDF,
}

/// Response for recent profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProfilesResponse {
    pub profiles: Vec<InferenceProfile>,
    pub count: u32,
    pub timestamp: u64,
}

/// Response for profiling statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStatsResponse {
    pub stats: Vec<PhaseStats>,
    pub window: String,
    pub sample_count: u32,
}

/// Response for timeline data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineResponse {
    pub data_points: Vec<TimelinePoint>,
    pub window_minutes: u32,
}

/// Single timeline data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    pub timestamp: u64,
    pub throughput_tokens_per_sec: f32,
    pub avg_latency_ms: f32,
    pub gpu_memory_mb: u32,
}

/// Response for anomalies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomaliesResponse {
    pub anomalies: Vec<AnomalyInfo>,
    pub timestamp: u64,
}

/// Individual anomaly information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyInfo {
    pub metric: String,
    pub severity: AnomalySeverity,
    pub message: String,
    pub deviation_percent: f32,
}

/// Anomaly severity level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Response for benchmark comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResponse {
    pub baseline: BenchmarkResult,
    pub current: BenchmarkResult,
    pub throughput_delta_percent: f32,
    pub latency_delta_percent: f32,
    pub memory_delta_percent: f32,
    pub status: ComparisonStatus,
}

/// Comparison status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonStatus {
    Regression,
    Improvement,
    NoChange,
}

/// Export response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    pub format: String,
    pub file_size_bytes: u64,
    pub record_count: u32,
    pub download_url: String,
}

/// Performance metrics for gauge display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceGaugeResponse {
    pub throughput_tokens_per_sec: f32,
    pub avg_latency_ms: f32,
    pub p99_latency_ms: f32,
    pub gpu_utilization_percent: f32,
    pub status: GaugeStatus,
}

/// Gauge status indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GaugeStatus {
    Excellent,
    Good,
    Warning,
    Critical,
}

impl GaugeStatus {
    /// Determine status from metrics
    pub fn from_metrics(p99_latency: f32, throughput: f32) -> Self {
        match (p99_latency, throughput) {
            (lat, _) if lat < 50.0 => GaugeStatus::Excellent,
            (lat, _) if lat < 150.0 => GaugeStatus::Good,
            (lat, _) if lat < 500.0 => GaugeStatus::Warning,
            _ => GaugeStatus::Critical,
        }
    }
}

/// Phase breakdown response (pie chart data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseBreakdownResponse {
    pub phases: Vec<PhaseBreakdownItem>,
    pub total_time_ms: f32,
}

/// Phase breakdown item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseBreakdownItem {
    pub phase: String,
    pub duration_ms: f32,
    pub percentage: f32,
}

/// Latency histogram response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyHistogramResponse {
    pub buckets: Vec<HistogramBucket>,
    pub p50: f32,
    pub p95: f32,
    pub p99: f32,
}

/// Histogram bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    pub range_ms: String,
    pub count: u32,
    pub percentage: f32,
}

/// Model comparison response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelComparisonResponse {
    pub models: Vec<ModelMetrics>,
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub model_id: String,
    pub avg_throughput_tokens_per_sec: f32,
    pub avg_latency_ms: f32,
    pub efficiency_score: f32, // throughput / GPU memory
    pub inference_count: u32,
}

/// Health check endpoint response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingHealthResponse {
    pub status: HealthStatus,
    pub profiles_collected: u32,
    pub anomalies_detected: u32,
    pub last_profile_timestamp: u64,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gauge_status_determination() {
        assert_eq!(
            GaugeStatus::from_metrics(30.0, 100.0),
            GaugeStatus::Excellent
        );
        assert_eq!(GaugeStatus::from_metrics(100.0, 100.0), GaugeStatus::Good);
        assert_eq!(GaugeStatus::from_metrics(300.0, 50.0), GaugeStatus::Warning);
        assert_eq!(
            GaugeStatus::from_metrics(600.0, 10.0),
            GaugeStatus::Critical
        );
    }

    #[test]
    fn test_phase_breakdown() {
        let response = PhaseBreakdownResponse {
            phases: vec![
                PhaseBreakdownItem {
                    phase: "tokenization".to_string(),
                    duration_ms: 10.0,
                    percentage: 1.2,
                },
                PhaseBreakdownItem {
                    phase: "inference".to_string(),
                    duration_ms: 800.0,
                    percentage: 97.5,
                },
                PhaseBreakdownItem {
                    phase: "detokenization".to_string(),
                    duration_ms: 10.0,
                    percentage: 1.2,
                },
            ],
            total_time_ms: 820.0,
        };

        assert_eq!(response.phases.len(), 3);
        assert!((response.total_time_ms - 820.0).abs() < 0.1);
    }

    #[test]
    fn test_model_comparison() {
        let response = ModelComparisonResponse {
            models: vec![
                ModelMetrics {
                    model_id: "llama-7b".to_string(),
                    avg_throughput_tokens_per_sec: 100.0,
                    avg_latency_ms: 100.0,
                    efficiency_score: 0.025,
                    inference_count: 50,
                },
                ModelMetrics {
                    model_id: "mistral-7b".to_string(),
                    avg_throughput_tokens_per_sec: 110.0,
                    avg_latency_ms: 90.0,
                    efficiency_score: 0.027,
                    inference_count: 60,
                },
            ],
        };

        assert_eq!(response.models.len(), 2);
    }
}
