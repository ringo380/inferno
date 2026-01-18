//! Profiling System Integration Tests

use inferno::infrastructure::profiling::*;
use std::time::Duration;

#[test]
fn test_basic_profiling_workflow() {
    let collector = ProfileCollector::new(100);

    let mut profile = InferenceProfile::new("req_1".to_string(), "llama-7b".to_string(), 256, 128);

    let tokenize = OperationProfile::new("tokenization".to_string(), Duration::from_millis(10));
    let inference = OperationProfile::new("inference".to_string(), Duration::from_millis(800));
    let detokenize = OperationProfile::new("detokenization".to_string(), Duration::from_millis(5));

    profile.add_phase(tokenize);
    profile.add_phase(inference);
    profile.add_phase(detokenize);
    profile.set_total_time(Duration::from_millis(815));

    collector.record_profile(profile.clone()).unwrap();
    assert_eq!(collector.len().unwrap(), 1);
}

#[test]
fn test_multiple_profiles_collection() {
    let collector = ProfileCollector::new(50);

    for i in 0..30 {
        let mut profile = InferenceProfile::new(
            format!("req_{}", i),
            "model".to_string(),
            100 + i as u32,
            50 + (i as u32 / 2),
        );
        profile.set_total_time(Duration::from_millis(500 + (i * 10) as u64));
        collector.record_profile(profile).unwrap();
    }

    assert_eq!(collector.len().unwrap(), 30);
}

#[test]
fn test_statistics_aggregator() {
    let mut agg = StatisticsAggregator::new();

    for i in 1..=10 {
        agg.record_phase("tokenization".to_string(), (i * 5) as f32);
    }

    let stats = agg.get_phase_stats("tokenization").unwrap();
    assert_eq!(stats.count, 10);
    assert!(stats.duration_ms.mean > 0.0);
}

#[test]
fn test_anomaly_detection() {
    let mut agg = StatisticsAggregator::new();
    agg.update_baseline("latency".to_string(), 100.0);

    let anomaly = agg.detect_anomalies("latency", 105.0, 10.0).unwrap();
    assert!(!anomaly.is_anomaly);

    let anomaly = agg.detect_anomalies("latency", 150.0, 10.0).unwrap();
    assert!(anomaly.is_anomaly);
}

#[test]
fn test_benchmark_comparison() {
    let baseline = BenchmarkResult {
        scenario_name: "test".to_string(),
        batch_size: 32,
        model_id: "model".to_string(),
        throughput_tokens_per_sec: 100.0,
        avg_latency_ms: 100.0,
        p99_latency_ms: 150.0,
        memory_peak_mb: 4096,
        iterations: 100,
    };

    let current = BenchmarkResult {
        scenario_name: "test".to_string(),
        batch_size: 32,
        model_id: "model".to_string(),
        throughput_tokens_per_sec: 110.0,
        avg_latency_ms: 95.0,
        p99_latency_ms: 140.0,
        memory_peak_mb: 4096,
        iterations: 100,
    };

    let comparison = BenchmarkComparison::new(baseline, current);
    assert!(comparison.throughput_change_percent > 0.0);
}

#[test]
fn test_benchmark_report() {
    let mut report = BenchmarkReport::new("Test".to_string());

    report.add_benchmark(BenchmarkResult {
        scenario_name: "test".to_string(),
        batch_size: 32,
        model_id: "model".to_string(),
        throughput_tokens_per_sec: 100.0,
        avg_latency_ms: 100.0,
        p99_latency_ms: 150.0,
        memory_peak_mb: 4096,
        iterations: 100,
    });

    let summary = report.summary();
    assert_eq!(summary.total_benchmarks, 1);
}

#[test]
fn test_html_report_generation() {
    let mut report = BenchmarkReport::new("Test".to_string());

    report.add_benchmark(BenchmarkResult {
        scenario_name: "test".to_string(),
        batch_size: 32,
        model_id: "model".to_string(),
        throughput_tokens_per_sec: 100.0,
        avg_latency_ms: 100.0,
        p99_latency_ms: 150.0,
        memory_peak_mb: 4096,
        iterations: 100,
    });

    let html = HTMLReportGenerator::generate(&report);
    assert!(html.contains("Test"));
}
