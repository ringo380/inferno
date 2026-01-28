//! Benchmark Report Generation
//!
//! Creates comprehensive benchmark reports comparing performance across scenarios

use serde::{Deserialize, Serialize};

/// Benchmark scenario result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub scenario_name: String,
    pub batch_size: usize,
    pub model_id: String,
    pub throughput_tokens_per_sec: f32,
    pub avg_latency_ms: f32,
    pub p99_latency_ms: f32,
    pub memory_peak_mb: u32,
    pub iterations: u32,
}

/// Benchmark comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub baseline: BenchmarkResult,
    pub current: BenchmarkResult,
    pub throughput_change_percent: f32,
    pub latency_change_percent: f32,
    pub memory_change_percent: f32,
}

impl BenchmarkComparison {
    /// Create comparison between two benchmarks
    pub fn new(baseline: BenchmarkResult, current: BenchmarkResult) -> Self {
        let throughput_change = if baseline.throughput_tokens_per_sec > 0.0 {
            ((current.throughput_tokens_per_sec - baseline.throughput_tokens_per_sec)
                / baseline.throughput_tokens_per_sec)
                * 100.0
        } else {
            0.0
        };

        let latency_change = if baseline.avg_latency_ms > 0.0 {
            ((current.avg_latency_ms - baseline.avg_latency_ms) / baseline.avg_latency_ms) * 100.0
        } else {
            0.0
        };

        let memory_change = if baseline.memory_peak_mb > 0 {
            ((current.memory_peak_mb as f32 - baseline.memory_peak_mb as f32)
                / baseline.memory_peak_mb as f32)
                * 100.0
        } else {
            0.0
        };

        Self {
            baseline,
            current,
            throughput_change_percent: throughput_change,
            latency_change_percent: latency_change,
            memory_change_percent: memory_change,
        }
    }

    /// Check if there's a regression
    pub fn has_regression(&self, threshold_percent: f32) -> bool {
        self.latency_change_percent > threshold_percent
            || (self.throughput_change_percent < -threshold_percent)
            || (self.memory_change_percent > threshold_percent)
    }

    /// Check if there's an improvement
    pub fn has_improvement(&self, threshold_percent: f32) -> bool {
        self.latency_change_percent < -threshold_percent
            || (self.throughput_change_percent > threshold_percent)
    }
}

/// Complete benchmark report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub title: String,
    pub timestamp: u64,
    pub duration_secs: u64,
    pub system_info: SystemInfo,
    pub benchmarks: Vec<BenchmarkResult>,
    pub comparisons: Vec<BenchmarkComparison>,
}

impl BenchmarkReport {
    /// Create new benchmark report
    pub fn new(title: String) -> Self {
        Self {
            title,
            timestamp: Self::current_timestamp(),
            duration_secs: 0,
            system_info: SystemInfo::default(),
            benchmarks: Vec::new(),
            comparisons: Vec::new(),
        }
    }

    /// Add benchmark result
    pub fn add_benchmark(&mut self, result: BenchmarkResult) {
        self.benchmarks.push(result);
    }

    /// Add comparison
    pub fn add_comparison(&mut self, comparison: BenchmarkComparison) {
        self.comparisons.push(comparison);
    }

    /// Generate summary statistics
    pub fn summary(&self) -> ReportSummary {
        if self.benchmarks.is_empty() {
            return ReportSummary::default();
        }

        let avg_throughput = self
            .benchmarks
            .iter()
            .map(|b| b.throughput_tokens_per_sec)
            .sum::<f32>()
            / self.benchmarks.len() as f32;
        let avg_latency = self
            .benchmarks
            .iter()
            .map(|b| b.avg_latency_ms)
            .sum::<f32>()
            / self.benchmarks.len() as f32;
        let peak_memory = self
            .benchmarks
            .iter()
            .map(|b| b.memory_peak_mb)
            .max()
            .unwrap_or(0);

        let regressions = self
            .comparisons
            .iter()
            .filter(|c| c.has_regression(5.0))
            .count();
        let improvements = self
            .comparisons
            .iter()
            .filter(|c| c.has_improvement(5.0))
            .count();

        ReportSummary {
            total_benchmarks: self.benchmarks.len() as u32,
            avg_throughput_tokens_per_sec: avg_throughput,
            avg_latency_ms: avg_latency,
            peak_memory_mb: peak_memory,
            regressions_detected: regressions as u32,
            improvements_detected: improvements as u32,
        }
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

/// System information for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub gpu_model: String,
    pub gpu_memory_gb: u32,
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub ram_gb: u32,
    pub os: String,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            gpu_model: "Unknown".to_string(),
            gpu_memory_gb: 0,
            cpu_model: "Unknown".to_string(),
            cpu_cores: 0,
            ram_gb: 0,
            os: std::env::consts::OS.to_string(),
        }
    }
}

/// Report summary statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReportSummary {
    pub total_benchmarks: u32,
    pub avg_throughput_tokens_per_sec: f32,
    pub avg_latency_ms: f32,
    pub peak_memory_mb: u32,
    pub regressions_detected: u32,
    pub improvements_detected: u32,
}

/// HTML report generator
pub struct HTMLReportGenerator;

impl HTMLReportGenerator {
    /// Generate HTML report from benchmark report
    pub fn generate(report: &BenchmarkReport) -> String {
        let summary = report.summary();

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; margin: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        h1, h2 {{ color: #333; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 15px; margin-bottom: 30px; }}
        .metric {{ background: #f9f9f9; padding: 15px; border-radius: 6px; border-left: 4px solid #007bff; }}
        .metric-value {{ font-size: 24px; font-weight: bold; color: #007bff; }}
        .metric-label {{ color: #666; font-size: 12px; margin-top: 5px; text-transform: uppercase; }}
        table {{ width: 100%; border-collapse: collapse; margin-bottom: 20px; }}
        th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }}
        th {{ background: #f0f0f0; font-weight: 600; }}
        tr:hover {{ background: #f9f9f9; }}
        .positive {{ color: #28a745; }}
        .negative {{ color: #dc3545; }}
        .info {{ background: #e7f3ff; padding: 10px; border-radius: 4px; margin-bottom: 20px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{}</h1>
        <div class="info">
            Generated: {} | Duration: {}s | System: {} {}
        </div>

        <h2>Summary</h2>
        <div class="summary">
            <div class="metric">
                <div class="metric-value">{:.1}</div>
                <div class="metric-label">Avg Throughput (tokens/sec)</div>
            </div>
            <div class="metric">
                <div class="metric-value">{:.1}ms</div>
                <div class="metric-label">Avg Latency</div>
            </div>
            <div class="metric">
                <div class="metric-value">{}MB</div>
                <div class="metric-label">Peak Memory</div>
            </div>
            <div class="metric">
                <div class="metric-value">{}</div>
                <div class="metric-label">Benchmarks Run</div>
            </div>
        </div>

        <h2>Benchmarks</h2>
        <table>
            <tr>
                <th>Scenario</th>
                <th>Model</th>
                <th>Batch</th>
                <th>Throughput</th>
                <th>Avg Latency</th>
                <th>P99 Latency</th>
                <th>Memory</th>
            </tr>
            {}
        </table>

        {}
    </div>
</body>
</html>"#,
            report.title,
            report.title,
            Self::format_timestamp(report.timestamp),
            report.duration_secs,
            report.system_info.cpu_model,
            report.system_info.gpu_model,
            summary.avg_throughput_tokens_per_sec,
            summary.avg_latency_ms,
            summary.peak_memory_mb,
            summary.total_benchmarks,
            Self::format_benchmark_table(&report.benchmarks),
            Self::format_comparisons_section(&report.comparisons)
        )
    }

    fn format_timestamp(timestamp: u64) -> String {
        use std::time::SystemTime;

        if let Some(time) =
            SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(timestamp))
        {
            return format!("{:?}", time);
        }

        "Unknown".to_string()
    }

    fn format_benchmark_table(benchmarks: &[BenchmarkResult]) -> String {
        benchmarks
            .iter()
            .map(|b| {
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:.1}</td><td>{:.1}ms</td><td>{:.1}ms</td><td>{}MB</td></tr>",
                    b.scenario_name, b.model_id, b.batch_size, b.throughput_tokens_per_sec, b.avg_latency_ms, b.p99_latency_ms, b.memory_peak_mb
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_comparisons_section(comparisons: &[BenchmarkComparison]) -> String {
        if comparisons.is_empty() {
            return String::new();
        }

        let table = comparisons
            .iter()
            .map(|c| {
                let throughput_class = if c.throughput_change_percent > 0.0 {
                    "positive"
                } else {
                    "negative"
                };
                let latency_class = if c.latency_change_percent < 0.0 {
                    "positive"
                } else {
                    "negative"
                };

                format!(
                    "<tr><td>{}</td><td class=\"{}\">{:.1}%</td><td class=\"{}\">{:.1}%</td><td>{:.1}%</td></tr>",
                    c.current.scenario_name,
                    throughput_class,
                    c.throughput_change_percent,
                    latency_class,
                    c.latency_change_percent,
                    c.memory_change_percent
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "<h2>Comparisons vs Baseline</h2><table><tr><th>Scenario</th><th>Throughput Δ</th><th>Latency Δ</th><th>Memory Δ</th></tr>{}</table>",
            table
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_comparison_improvement() {
        let baseline = BenchmarkResult {
            scenario_name: "test".to_string(),
            batch_size: 1,
            model_id: "model".to_string(),
            throughput_tokens_per_sec: 100.0,
            avg_latency_ms: 100.0,
            p99_latency_ms: 150.0,
            memory_peak_mb: 1024,
            iterations: 100,
        };

        let current = BenchmarkResult {
            scenario_name: "test".to_string(),
            batch_size: 1,
            model_id: "model".to_string(),
            throughput_tokens_per_sec: 120.0, // 20% improvement
            avg_latency_ms: 80.0,             // 20% improvement
            p99_latency_ms: 120.0,
            memory_peak_mb: 1024,
            iterations: 100,
        };

        let comparison = BenchmarkComparison::new(baseline, current);
        assert!(comparison.has_improvement(15.0));
    }

    #[test]
    fn test_benchmark_comparison_regression() {
        let baseline = BenchmarkResult {
            scenario_name: "test".to_string(),
            batch_size: 1,
            model_id: "model".to_string(),
            throughput_tokens_per_sec: 100.0,
            avg_latency_ms: 100.0,
            p99_latency_ms: 150.0,
            memory_peak_mb: 1024,
            iterations: 100,
        };

        let current = BenchmarkResult {
            scenario_name: "test".to_string(),
            batch_size: 1,
            model_id: "model".to_string(),
            throughput_tokens_per_sec: 80.0, // 20% regression
            avg_latency_ms: 130.0,           // 30% regression
            p99_latency_ms: 180.0,
            memory_peak_mb: 1280, // 25% increase
            iterations: 100,
        };

        let comparison = BenchmarkComparison::new(baseline, current);
        assert!(comparison.has_regression(5.0));
    }

    #[test]
    fn test_benchmark_report_generation() {
        let mut report = BenchmarkReport::new("Test Report".to_string());

        report.add_benchmark(BenchmarkResult {
            scenario_name: "throughput_test".to_string(),
            batch_size: 32,
            model_id: "llama-7b".to_string(),
            throughput_tokens_per_sec: 150.0,
            avg_latency_ms: 200.0,
            p99_latency_ms: 250.0,
            memory_peak_mb: 8192,
            iterations: 100,
        });

        let summary = report.summary();
        assert_eq!(summary.total_benchmarks, 1);
        assert!((summary.avg_throughput_tokens_per_sec - 150.0).abs() < 0.1);
    }

    #[test]
    fn test_html_generation() {
        let mut report = BenchmarkReport::new("Performance Test".to_string());
        report.add_benchmark(BenchmarkResult {
            scenario_name: "test".to_string(),
            batch_size: 1,
            model_id: "model".to_string(),
            throughput_tokens_per_sec: 100.0,
            avg_latency_ms: 100.0,
            p99_latency_ms: 150.0,
            memory_peak_mb: 1024,
            iterations: 100,
        });

        let html = HTMLReportGenerator::generate(&report);
        assert!(html.contains("Performance Test"));
        assert!(html.contains("100.0"));
        assert!(html.contains("Benchmark"));
    }
}
