//! Performance Profiling & Benchmarking Module
//!
//! Provides comprehensive performance analysis with:
//! - Per-operation profiling (tokenization, inference, detokenization)
//! - Statistical analysis with percentiles
//! - Anomaly detection
//! - Trend analysis
//! - Benchmark report generation

pub mod profiler;
pub mod stats;
pub mod benchmark_report;

pub use profiler::{
    OperationProfile, InferenceProfile, PhaseTimer, ProfileCollector, AverageMetrics,
};

pub use stats::{
    DurationStats, PhaseStats, TimeWindow, AnomalyDetection, StatisticsAggregator, TrendDirection,
};

pub use benchmark_report::{
    BenchmarkResult, BenchmarkComparison, BenchmarkReport, HTMLReportGenerator, SystemInfo, ReportSummary,
};
