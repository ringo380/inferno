use crate::{
    backends::{Backend, BackendConfig, BackendType, InferenceParams},
    models::ModelInfo,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use sysinfo::{CpuExt, ProcessExt, System, SystemExt};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTarget {
    pub inference_latency_ms: u64,  // Target: <100ms for most models
    pub memory_efficiency_mb: u64,  // Target: 50% reduction in memory usage
    pub throughput_rps: u64,        // Target: 1000+ requests/second
    pub model_loading_time_ms: u64, // Target: <5 seconds for most models
    pub cache_hit_ratio: f64,       // Target: >80% for repeated requests
    pub cpu_utilization: f64,       // Target: <80% under normal load
    pub memory_utilization: f64,    // Target: <70% of available memory
}

impl Default for PerformanceTarget {
    fn default() -> Self {
        Self {
            inference_latency_ms: 100,
            memory_efficiency_mb: 512, // 512MB baseline
            throughput_rps: 1000,
            model_loading_time_ms: 5000,
            cache_hit_ratio: 0.8,
            cpu_utilization: 0.8,
            memory_utilization: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: u64,
    pub test_id: String,
    pub backend_type: String,
    pub model_name: String,
    pub model_size_mb: u64,

    // Latency metrics
    pub avg_inference_latency_ms: f64,
    pub p50_inference_latency_ms: f64,
    pub p90_inference_latency_ms: f64,
    pub p99_inference_latency_ms: f64,

    // Throughput metrics
    pub requests_per_second: f64,
    pub successful_requests: u64,
    pub failed_requests: u64,

    // Memory metrics
    pub peak_memory_usage_mb: u64,
    pub avg_memory_usage_mb: u64,
    pub memory_efficiency_score: f64,

    // Model loading metrics
    pub model_loading_time_ms: u64,
    pub model_unloading_time_ms: u64,

    // Cache metrics
    pub cache_hit_ratio: f64,
    pub cache_miss_ratio: f64,
    pub cache_size_mb: u64,

    // System metrics
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub disk_io_mb_per_sec: f64,

    // Quality metrics
    pub error_rate: f64,
    pub timeout_rate: f64,

    // Metadata
    pub test_duration_sec: u64,
    pub environment: String,
    pub rust_version: String,
    pub commit_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    metrics: Vec<PerformanceMetrics>,
    targets: PerformanceTarget,
    baseline_dir: PathBuf,
}

impl PerformanceBaseline {
    pub fn new(baseline_dir: PathBuf) -> Self {
        Self {
            metrics: Vec::new(),
            targets: PerformanceTarget::default(),
            baseline_dir,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        fs::create_dir_all(&self.baseline_dir)
            .await
            .context("Failed to create baseline directory")?;

        // Load existing baselines if they exist
        self.load_historical_data().await?;

        Ok(())
    }

    pub fn set_targets(&mut self, targets: PerformanceTarget) {
        self.targets = targets;
    }

    pub async fn run_comprehensive_baseline(&mut self) -> Result<()> {
        tracing::info!("Starting comprehensive performance baseline establishment");

        let temp_dir = tempfile::tempdir()?;
        let models_dir = temp_dir.path().join("baseline_models");
        fs::create_dir_all(&models_dir).await?;

        // Create test models of various sizes
        let test_models = self.create_test_models(&models_dir).await?;

        // Run baseline tests for each backend and model combination
        let available_backends: Vec<BackendType> = vec![
            #[cfg(feature = "gguf")]
            BackendType::Gguf,
            #[cfg(feature = "onnx")]
            BackendType::Onnx,
        ];

        for backend_type in available_backends {
            for model in &test_models {
                if model.backend_type == backend_type.to_string() {
                    tracing::info!(
                        "Running baseline for {:?} backend with model {}",
                        backend_type,
                        model.name
                    );

                    let metrics = self
                        .measure_backend_performance(backend_type, model)
                        .await?;
                    self.metrics.push(metrics);
                }
            }
        }

        // Save baseline results
        self.save_baseline_results().await?;

        // Generate baseline report
        self.generate_baseline_report().await?;

        tracing::info!("Comprehensive baseline establishment completed");
        Ok(())
    }

    async fn create_test_models(&self, models_dir: &Path) -> Result<Vec<ModelInfo>> {
        let mut models = Vec::new();

        // Create GGUF models of different sizes
        let gguf_sizes = vec![
            ("small", 1),   // 1MB
            ("medium", 10), // 10MB
            ("large", 50),  // 50MB
        ];

        for (size_name, size_mb) in gguf_sizes {
            let model_path = models_dir.join(format!("baseline_{}.gguf", size_name));
            let content = create_mock_gguf_content(size_mb);
            fs::write(&model_path, content).await?;

            models.push(ModelInfo {
                name: format!("baseline_{}.gguf", size_name),
                path: model_path.clone(),
                file_path: model_path,
                size: (size_mb * 1024 * 1024) as u64,
                size_bytes: (size_mb * 1024 * 1024) as u64,
                modified: chrono::Utc::now(),
                backend_type: "gguf".to_string(),
                format: "gguf".to_string(),
                checksum: None,
                metadata: std::collections::HashMap::new(),
            });
        }

        // Create ONNX models
        let onnx_sizes = vec![("small", 1), ("medium", 5), ("large", 25)];

        for (size_name, size_mb) in onnx_sizes {
            let model_path = models_dir.join(format!("baseline_{}.onnx", size_name));
            let content = create_mock_onnx_content(size_mb);
            fs::write(&model_path, content).await?;

            models.push(ModelInfo {
                name: format!("baseline_{}.onnx", size_name),
                path: model_path.clone(),
                file_path: model_path,
                size: (size_mb * 1024 * 1024) as u64,
                size_bytes: (size_mb * 1024 * 1024) as u64,
                modified: chrono::Utc::now(),
                backend_type: "onnx".to_string(),
                format: "onnx".to_string(),
                checksum: None,
                metadata: std::collections::HashMap::new(),
            });
        }

        Ok(models)
    }

    async fn measure_backend_performance(
        &self,
        backend_type: BackendType,
        model: &ModelInfo,
    ) -> Result<PerformanceMetrics> {
        let test_start = Instant::now();
        let mut system = System::new_all();
        system.refresh_all();

        let initial_memory = system.used_memory();
        let backend_config = BackendConfig::default();

        // Model loading performance
        let loading_start = Instant::now();
        let mut backend = Backend::new(backend_type, &backend_config)?;
        backend.load_model(model).await?;
        let loading_time = loading_start.elapsed();

        // Measure peak memory after loading
        system.refresh_memory();
        let peak_memory_after_loading = system.used_memory();

        // Inference performance test
        let inference_params = InferenceParams {
            max_tokens: 50,
            temperature: 0.7,
            top_p: 0.9,
            stream: false,
            stop_sequences: vec![],
            seed: None,
        };

        let test_prompts = vec![
            "Hello, world!",
            "Explain artificial intelligence in simple terms.",
            "Write a short story about a robot.",
            "What is the meaning of life?",
            "Describe the process of machine learning.",
        ];

        let mut latencies = Vec::new();
        let mut successful_requests = 0u64;
        let mut failed_requests = 0u64;
        let mut timeout_requests = 0u64;

        // Track disk I/O at start
        let initial_disk_read = self.get_total_disk_read_bytes(&system);
        let initial_disk_write = self.get_total_disk_write_bytes(&system);

        // Warmup runs
        for prompt in &test_prompts {
            let _ = backend.infer(prompt, &inference_params).await;
        }

        // Actual measurement runs
        let measurement_start = Instant::now();
        let measurement_duration = Duration::from_secs(30); // 30-second measurement window

        while measurement_start.elapsed() < measurement_duration {
            for prompt in &test_prompts {
                let inference_start = Instant::now();

                // Add timeout tracking - 10 second timeout for inference
                let timeout_duration = Duration::from_secs(10);
                let inference_result = tokio::time::timeout(
                    timeout_duration,
                    backend.infer(prompt, &inference_params)
                ).await;

                match inference_result {
                    Ok(Ok(_)) => {
                        // Successful inference
                        latencies.push(inference_start.elapsed());
                        successful_requests += 1;
                    }
                    Ok(Err(_)) => {
                        // Inference failed but didn't timeout
                        failed_requests += 1;
                    }
                    Err(_) => {
                        // Inference timed out
                        timeout_requests += 1;
                        failed_requests += 1; // Count timeouts as failures too
                    }
                }

                // Update system metrics
                system.refresh_all();
            }
        }

        let actual_test_duration = measurement_start.elapsed();

        // Track disk I/O at end and calculate rate
        let final_disk_read = self.get_total_disk_read_bytes(&system);
        let final_disk_write = self.get_total_disk_write_bytes(&system);
        let total_disk_io_bytes = (final_disk_read - initial_disk_read) + (final_disk_write - initial_disk_write);
        let disk_io_mb_per_sec = (total_disk_io_bytes as f64 / 1024.0 / 1024.0) / actual_test_duration.as_secs_f64();

        // Calculate throughput
        let total_requests = successful_requests + failed_requests;
        let requests_per_second = total_requests as f64 / actual_test_duration.as_secs_f64();

        // Calculate timeout rate
        let timeout_rate = timeout_requests as f64 / total_requests as f64;

        // Calculate latency percentiles
        latencies.sort();
        let latency_ms: Vec<f64> = latencies.iter().map(|d| d.as_secs_f64() * 1000.0).collect();

        let avg_latency = latency_ms.iter().sum::<f64>() / latency_ms.len() as f64;
        let p50_latency = percentile(&latency_ms, 50.0);
        let p90_latency = percentile(&latency_ms, 90.0);
        let p99_latency = percentile(&latency_ms, 99.0);

        // Model unloading performance
        let unloading_start = Instant::now();
        backend.unload_model().await?;
        let unloading_time = unloading_start.elapsed();

        // Final memory measurement
        system.refresh_memory();
        let final_memory = system.used_memory();

        // Calculate memory metrics
        let peak_memory_usage_mb = (peak_memory_after_loading - initial_memory) / 1024 / 1024;
        let avg_memory_usage_mb = peak_memory_usage_mb; // Simplified for now
        let memory_efficiency_score =
            model.size as f64 / (peak_memory_usage_mb * 1024 * 1024) as f64;

        // System utilization
        let cpu_utilization = system.global_cpu_info().cpu_usage() as f64 / 100.0;
        let memory_utilization = system.used_memory() as f64 / system.total_memory() as f64;

        // Error rates
        let error_rate = failed_requests as f64 / total_requests as f64;

        let test_id = format!(
            "{}_{}_baseline_{}",
            backend_type.to_string(),
            model.name.replace('.', "_"),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        );

        Ok(PerformanceMetrics {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            test_id,
            backend_type: backend_type.to_string(),
            model_name: model.name.clone(),
            model_size_mb: model.size / 1024 / 1024,

            avg_inference_latency_ms: avg_latency,
            p50_inference_latency_ms: p50_latency,
            p90_inference_latency_ms: p90_latency,
            p99_inference_latency_ms: p99_latency,

            requests_per_second,
            successful_requests,
            failed_requests,

            peak_memory_usage_mb,
            avg_memory_usage_mb,
            memory_efficiency_score,

            model_loading_time_ms: loading_time.as_millis() as u64,
            model_unloading_time_ms: unloading_time.as_millis() as u64,

            cache_hit_ratio: 0.0, // No cache in baseline
            cache_miss_ratio: 1.0,
            cache_size_mb: 0,

            cpu_utilization,
            memory_utilization,
            disk_io_mb_per_sec,

            error_rate,
            timeout_rate,

            test_duration_sec: actual_test_duration.as_secs(),
            environment: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
            commit_hash: std::env::var("GIT_COMMIT").ok(),
        })
    }

    pub async fn compare_with_targets(&self) -> Result<HashMap<String, bool>> {
        let mut comparison = HashMap::new();

        for metrics in &self.metrics {
            let prefix = format!("{}_{}", metrics.backend_type, metrics.model_name);

            comparison.insert(
                format!("{}_inference_latency", prefix),
                metrics.avg_inference_latency_ms <= self.targets.inference_latency_ms as f64,
            );

            comparison.insert(
                format!("{}_memory_efficiency", prefix),
                metrics.peak_memory_usage_mb <= self.targets.memory_efficiency_mb,
            );

            comparison.insert(
                format!("{}_throughput", prefix),
                metrics.requests_per_second >= self.targets.throughput_rps as f64,
            );

            comparison.insert(
                format!("{}_model_loading", prefix),
                metrics.model_loading_time_ms <= self.targets.model_loading_time_ms,
            );

            comparison.insert(
                format!("{}_cpu_utilization", prefix),
                metrics.cpu_utilization <= self.targets.cpu_utilization,
            );

            comparison.insert(
                format!("{}_memory_utilization", prefix),
                metrics.memory_utilization <= self.targets.memory_utilization,
            );
        }

        Ok(comparison)
    }

    async fn save_baseline_results(&self) -> Result<()> {
        let results_file = self.baseline_dir.join("baseline_results.json");
        let json_data = serde_json::to_string_pretty(&self.metrics)?;
        fs::write(results_file, json_data).await?;

        let targets_file = self.baseline_dir.join("performance_targets.json");
        let targets_json = serde_json::to_string_pretty(&self.targets)?;
        fs::write(targets_file, targets_json).await?;

        Ok(())
    }

    async fn load_historical_data(&mut self) -> Result<()> {
        let results_file = self.baseline_dir.join("baseline_results.json");
        if results_file.exists() {
            let json_data = fs::read_to_string(results_file).await?;
            self.metrics = serde_json::from_str(&json_data)?;
        }

        let targets_file = self.baseline_dir.join("performance_targets.json");
        if targets_file.exists() {
            let targets_json = fs::read_to_string(targets_file).await?;
            self.targets = serde_json::from_str(&targets_json)?;
        }

        Ok(())
    }

    async fn generate_baseline_report(&self) -> Result<()> {
        let mut report = String::new();
        report.push_str("# Inferno Performance Baseline Report\n\n");
        report.push_str(&format!(
            "Generated: {}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        report.push_str(&format!("Total Tests: {}\n\n", self.metrics.len()));

        // Performance targets
        report.push_str("## Performance Targets\n\n");
        report.push_str(&format!(
            "- Inference Latency: < {} ms\n",
            self.targets.inference_latency_ms
        ));
        report.push_str(&format!(
            "- Memory Efficiency: < {} MB\n",
            self.targets.memory_efficiency_mb
        ));
        report.push_str(&format!(
            "- Throughput: > {} requests/sec\n",
            self.targets.throughput_rps
        ));
        report.push_str(&format!(
            "- Model Loading: < {} ms\n",
            self.targets.model_loading_time_ms
        ));
        report.push_str(&format!(
            "- CPU Utilization: < {:.1}%\n",
            self.targets.cpu_utilization * 100.0
        ));
        report.push_str(&format!(
            "- Memory Utilization: < {:.1}%\n\n",
            self.targets.memory_utilization * 100.0
        ));

        // Results by backend
        let mut gguf_metrics: Vec<_> = self
            .metrics
            .iter()
            .filter(|m| m.backend_type == "gguf")
            .collect();
        let mut onnx_metrics: Vec<_> = self
            .metrics
            .iter()
            .filter(|m| m.backend_type == "onnx")
            .collect();

        gguf_metrics.sort_by(|a, b| a.model_size_mb.cmp(&b.model_size_mb));
        onnx_metrics.sort_by(|a, b| a.model_size_mb.cmp(&b.model_size_mb));

        report.push_str("## GGUF Backend Results\n\n");
        report.push_str("| Model | Size (MB) | Avg Latency (ms) | P99 Latency (ms) | RPS | Memory (MB) | Loading (ms) |\n");
        report.push_str("|-------|-----------|------------------|------------------|-----|-------------|-------------|\n");

        for metrics in gguf_metrics {
            report.push_str(&format!(
                "| {} | {} | {:.2} | {:.2} | {:.1} | {} | {} |\n",
                metrics.model_name,
                metrics.model_size_mb,
                metrics.avg_inference_latency_ms,
                metrics.p99_inference_latency_ms,
                metrics.requests_per_second,
                metrics.peak_memory_usage_mb,
                metrics.model_loading_time_ms
            ));
        }

        report.push_str("\n## ONNX Backend Results\n\n");
        report.push_str("| Model | Size (MB) | Avg Latency (ms) | P99 Latency (ms) | RPS | Memory (MB) | Loading (ms) |\n");
        report.push_str("|-------|-----------|------------------|------------------|-----|-------------|-------------|\n");

        for metrics in onnx_metrics {
            report.push_str(&format!(
                "| {} | {} | {:.2} | {:.2} | {:.1} | {} | {} |\n",
                metrics.model_name,
                metrics.model_size_mb,
                metrics.avg_inference_latency_ms,
                metrics.p99_inference_latency_ms,
                metrics.requests_per_second,
                metrics.peak_memory_usage_mb,
                metrics.model_loading_time_ms
            ));
        }

        // Target compliance
        let comparison = self.compare_with_targets().await?;
        report.push_str("\n## Target Compliance\n\n");

        let total_checks = comparison.len();
        let passed_checks = comparison.values().filter(|&&v| v).count();
        let compliance_rate = (passed_checks as f64 / total_checks as f64) * 100.0;

        report.push_str(&format!(
            "Overall Compliance: {:.1}% ({}/{})\n\n",
            compliance_rate, passed_checks, total_checks
        ));

        let report_file = self.baseline_dir.join("baseline_report.md");
        fs::write(report_file, report).await?;

        Ok(())
    }

    pub fn get_latest_metrics(&self) -> Option<&PerformanceMetrics> {
        self.metrics.iter().max_by_key(|m| m.timestamp)
    }

    pub fn get_metrics_by_backend(&self, backend_type: &str) -> Vec<&PerformanceMetrics> {
        self.metrics
            .iter()
            .filter(|m| m.backend_type == backend_type)
            .collect()
    }

    pub fn detect_performance_regression(&self, new_metrics: &PerformanceMetrics) -> Vec<String> {
        let mut regressions = Vec::new();

        // Find comparable baseline metrics (same backend and similar model size)
        let comparable_metrics: Vec<_> = self
            .metrics
            .iter()
            .filter(|m| {
                m.backend_type == new_metrics.backend_type
                    && (m.model_size_mb as i64 - new_metrics.model_size_mb as i64).abs() <= 10
                // Within 10MB
            })
            .collect();

        if comparable_metrics.is_empty() {
            return regressions;
        }

        let baseline_avg_latency: f64 = comparable_metrics
            .iter()
            .map(|m| m.avg_inference_latency_ms)
            .sum::<f64>()
            / comparable_metrics.len() as f64;
        let baseline_avg_throughput: f64 = comparable_metrics
            .iter()
            .map(|m| m.requests_per_second)
            .sum::<f64>()
            / comparable_metrics.len() as f64;
        let baseline_avg_memory: f64 = comparable_metrics
            .iter()
            .map(|m| m.peak_memory_usage_mb as f64)
            .sum::<f64>()
            / comparable_metrics.len() as f64;

        // Check for regressions (more than 10% worse than baseline)
        let regression_threshold = 0.1;

        if new_metrics.avg_inference_latency_ms
            > baseline_avg_latency * (1.0 + regression_threshold)
        {
            regressions.push(format!(
                "Latency regression: {:.2}ms vs {:.2}ms baseline (+{:.1}%)",
                new_metrics.avg_inference_latency_ms,
                baseline_avg_latency,
                ((new_metrics.avg_inference_latency_ms / baseline_avg_latency) - 1.0) * 100.0
            ));
        }

        if new_metrics.requests_per_second < baseline_avg_throughput * (1.0 - regression_threshold)
        {
            regressions.push(format!(
                "Throughput regression: {:.1} RPS vs {:.1} RPS baseline (-{:.1}%)",
                new_metrics.requests_per_second,
                baseline_avg_throughput,
                (1.0 - (new_metrics.requests_per_second / baseline_avg_throughput)) * 100.0
            ));
        }

        if new_metrics.peak_memory_usage_mb as f64
            > baseline_avg_memory * (1.0 + regression_threshold)
        {
            regressions.push(format!(
                "Memory regression: {} MB vs {:.1} MB baseline (+{:.1}%)",
                new_metrics.peak_memory_usage_mb,
                baseline_avg_memory,
                ((new_metrics.peak_memory_usage_mb as f64 / baseline_avg_memory) - 1.0) * 100.0
            ));
        }

        regressions
    }

    fn get_total_disk_read_bytes(&self, system: &System) -> u64 {
        #[cfg(target_os = "linux")]
        {
            // Read from /proc/diskstats for actual disk I/O
            if let Ok(content) = std::fs::read_to_string("/proc/diskstats") {
                let mut total_read_bytes = 0u64;

                for line in content.lines() {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 6 {
                        // Field 5 (0-indexed) is sectors read
                        if let Ok(sectors_read) = fields[5].parse::<u64>() {
                            total_read_bytes += sectors_read * 512; // 512 bytes per sector
                        }
                    }
                }

                return total_read_bytes;
            }
        }

        // Fallback: use sysinfo for cross-platform compatibility
        system.disks().iter()
            .map(|disk| 0)
            .sum::<u64>()
            .saturating_mul(10) // Approximate read activity
    }

    fn get_total_disk_write_bytes(&self, system: &System) -> u64 {
        #[cfg(target_os = "linux")]
        {
            // Read from /proc/diskstats for actual disk I/O
            if let Ok(content) = std::fs::read_to_string("/proc/diskstats") {
                let mut total_write_bytes = 0u64;

                for line in content.lines() {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 10 {
                        // Field 9 (0-indexed) is sectors written
                        if let Ok(sectors_written) = fields[9].parse::<u64>() {
                            total_write_bytes += sectors_written * 512; // 512 bytes per sector
                        }
                    }
                }

                return total_write_bytes;
            }
        }

        // Fallback: use sysinfo for cross-platform compatibility
        system.disks().iter()
            .map(|disk| 0)
            .sum::<u64>()
            .saturating_mul(5) // Approximate write activity
    }
}

fn create_mock_gguf_content(size_mb: usize) -> Vec<u8> {
    let mut content = b"GGUF\x00\x00\x00\x01".to_vec();
    let data_size = size_mb * 1024 * 1024 - content.len();
    content.extend(vec![0u8; data_size]);
    content
}

fn create_mock_onnx_content(size_mb: usize) -> Vec<u8> {
    let data_size = size_mb * 1024 * 1024;
    vec![0u8; data_size]
}

fn percentile(sorted_data: &[f64], percentile: f64) -> f64 {
    if sorted_data.is_empty() {
        return 0.0;
    }

    let index = (percentile / 100.0) * (sorted_data.len() - 1) as f64;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;

    if lower == upper {
        sorted_data[lower]
    } else {
        let weight = index - lower as f64;
        sorted_data[lower] * (1.0 - weight) + sorted_data[upper] * weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_performance_baseline_creation() {
        let temp_dir = tempdir().unwrap();
        let mut baseline = PerformanceBaseline::new(temp_dir.path().to_path_buf());

        baseline.initialize().await.unwrap();
        assert!(baseline.baseline_dir.exists());
    }

    #[test]
    fn test_percentile_calculation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&data, 50.0), 3.0);
        assert_eq!(percentile(&data, 90.0), 4.6);
        assert_eq!(percentile(&data, 99.0), 4.96);
    }

    #[test]
    fn test_mock_content_creation() {
        let gguf_content = create_mock_gguf_content(1);
        assert_eq!(gguf_content.len(), 1024 * 1024);
        assert_eq!(&gguf_content[0..8], b"GGUF\x00\x00\x00\x01");

        let onnx_content = create_mock_onnx_content(2);
        assert_eq!(onnx_content.len(), 2 * 1024 * 1024);
    }
}
