//! Performance Profiling & Benchmarking Module
//!
//! Provides comprehensive performance analysis with:
//! - Per-operation profiling (tokenization, inference, detokenization)
//! - Statistical analysis with percentiles
//! - Anomaly detection
//! - Trend analysis

pub mod profiler;
pub mod stats;

pub use profiler::{
    OperationProfile, InferenceProfile, PhaseTimer, ProfileCollector, AverageMetrics,
};

pub use stats::{
    DurationStats, PhaseStats, TimeWindow, AnomalyDetection, StatisticsAggregator, TrendDirection,
};
