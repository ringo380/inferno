//! Performance Profiling & Benchmarking Module
//!
//! Provides comprehensive performance analysis with:
//! - Per-operation profiling (tokenization, inference, detokenization)
//! - Statistical analysis with percentiles
//! - Anomaly detection
//! - Trend analysis
//! - Benchmark report generation

pub mod benchmark_report;
pub mod endpoints;
pub mod profiler;
pub mod stats;

pub use profiler::{
    AverageMetrics, InferenceProfile, OperationProfile, PhaseTimer, ProfileCollector,
};

pub use stats::{
    AnomalyDetection, DurationStats, PhaseStats, StatisticsAggregator, TimeWindow, TrendDirection,
};

pub use benchmark_report::{
    BenchmarkComparison, BenchmarkReport, BenchmarkResult, HTMLReportGenerator, ReportSummary,
    SystemInfo,
};

pub use endpoints::{
    AnomaliesResponse, ComparisonResponse, ExportResponse, LatencyHistogramResponse,
    ModelComparisonResponse, PerformanceGaugeResponse, PhaseBreakdownResponse,
    ProfileStatsResponse, ProfilingHealthResponse, RecentProfilesResponse, TimelineResponse,
};
