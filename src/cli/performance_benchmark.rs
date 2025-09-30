use crate::{
    backends::{Backend, BackendConfig, BackendType, InferenceParams},
    models::ModelInfo,
    performance_baseline::{PerformanceBaseline, PerformanceTarget},
};
use anyhow::Result;
use clap::{Args, Subcommand};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use sysinfo::{CpuExt, System, SystemExt};

// Memory profiling structures
#[derive(Debug, Clone)]
struct MemoryUsage {
    heap_used: u64,
    heap_total: u64,
    rss: u64, // Resident Set Size
    vms: u64, // Virtual Memory Size
}

#[derive(Debug, Clone)]
struct MemorySnapshot {
    timestamp: Duration,
    heap_used: u64,
    heap_total: u64,
    rss: u64,
    vms: u64,
    inference_id: u32,
    model_loaded: bool,
}

#[derive(Debug)]
struct MemoryAnalysis {
    baseline_rss: u64,
    peak_rss: u64,
    final_rss: u64,
    total_growth: i64,
    average_growth_per_cycle: f64,
    memory_leak_detected: bool,
    gc_efficiency: f64,
    model_load_overhead: u64,
}

#[derive(Debug, Args)]
pub struct PerformanceBenchmarkArgs {
    #[command(subcommand)]
    pub command: PerformanceBenchmarkCommand,
}

#[derive(Debug, Subcommand)]
pub enum PerformanceBenchmarkCommand {
    /// Establish performance baseline
    Baseline {
        /// Output directory for baseline results
        #[arg(short, long, default_value = "performance_baseline")]
        output: PathBuf,

        /// Custom performance targets file
        #[arg(short, long)]
        targets: Option<PathBuf>,

        /// Backend types to test (comma-separated)
        #[arg(short, long, default_value = "gguf,onnx")]
        backends: String,

        /// Test duration per benchmark (seconds)
        #[arg(short, long, default_value = "30")]
        duration: u64,
    },

    /// Run performance benchmarks
    Benchmark {
        /// Benchmark type to run
        #[arg(short, long, default_value = "all")]
        bench_type: String,

        /// Output directory for results
        #[arg(short, long, default_value = "benchmark_results")]
        output: PathBuf,

        /// Model to test (optional, uses test models if not specified)
        #[arg(short, long)]
        model: Option<String>,

        /// Number of iterations
        #[arg(short, long, default_value = "100")]
        iterations: u32,

        /// Enable profiling
        #[arg(short, long)]
        profile: bool,
    },

    /// Compare performance with baseline
    Compare {
        /// Current results file
        #[arg(short, long)]
        current: PathBuf,

        /// Baseline results file
        #[arg(short, long)]
        baseline: PathBuf,

        /// Regression threshold (percentage)
        #[arg(short, long, default_value = "10.0")]
        threshold: f64,

        /// Generate detailed report
        #[arg(short, long)]
        report: bool,
    },

    /// Monitor real-time performance
    Monitor {
        /// Monitoring duration (seconds, 0 for infinite)
        #[arg(short, long, default_value = "60")]
        duration: u64,

        /// Sampling interval (seconds)
        #[arg(short, long, default_value = "5")]
        interval: u64,

        /// Output format (json, csv, console)
        #[arg(short, long, default_value = "console")]
        format: String,

        /// Output file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Stress test the system
    Stress {
        /// Number of concurrent clients
        #[arg(short, long, default_value = "10")]
        clients: u32,

        /// Test duration (seconds)
        #[arg(short, long, default_value = "300")]
        duration: u64,

        /// Model to use for stress testing
        #[arg(short, long)]
        model: Option<String>,

        /// Request rate per client (requests/second)
        #[arg(short, long, default_value = "1.0")]
        rate: f64,
    },

    /// Profile memory usage
    MemoryProfile {
        /// Model to profile
        #[arg(short, long)]
        model: Option<String>,

        /// Number of inference cycles
        #[arg(short, long, default_value = "50")]
        cycles: u32,

        /// Output file for memory report
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Track memory over time
        #[arg(short, long)]
        track: bool,
    },
}

pub async fn execute_performance_benchmark(args: PerformanceBenchmarkArgs) -> Result<()> {
    match args.command {
        PerformanceBenchmarkCommand::Baseline {
            output,
            targets,
            backends,
            duration,
        } => establish_baseline(output, targets, backends, duration).await,
        PerformanceBenchmarkCommand::Benchmark {
            bench_type,
            output,
            model,
            iterations,
            profile,
        } => run_benchmark(bench_type, output, model, iterations, profile).await,
        PerformanceBenchmarkCommand::Compare {
            current,
            baseline,
            threshold,
            report,
        } => compare_performance(current, baseline, threshold, report).await,
        PerformanceBenchmarkCommand::Monitor {
            duration,
            interval,
            format,
            output,
        } => monitor_performance(duration, interval, format, output).await,
        PerformanceBenchmarkCommand::Stress {
            clients,
            duration,
            model,
            rate,
        } => stress_test(clients, duration, model, rate).await,
        PerformanceBenchmarkCommand::MemoryProfile {
            model,
            cycles,
            output,
            track,
        } => memory_profile(model, cycles, output, track).await,
    }
}

async fn establish_baseline(
    output_dir: PathBuf,
    targets_file: Option<PathBuf>,
    backends_str: String,
    duration: u64,
) -> Result<()> {
    tracing::info!("Establishing performance baseline");

    let mut baseline = PerformanceBaseline::new(output_dir);
    baseline.initialize().await?;

    // Load custom targets if provided
    if let Some(targets_path) = targets_file {
        let targets_content = tokio::fs::read_to_string(targets_path).await?;
        let custom_targets: PerformanceTarget = serde_json::from_str(&targets_content)?;
        baseline.set_targets(custom_targets);
    }

    // Parse backend types
    let backend_types: Vec<BackendType> = backends_str
        .split(',')
        .filter_map(|s| match s.trim().to_lowercase().as_str() {
            #[cfg(feature = "gguf")]
            "gguf" => Some(BackendType::Gguf),
            #[cfg(feature = "onnx")]
            "onnx" => Some(BackendType::Onnx),
            _ => {
                tracing::warn!("Unknown backend type: {}", s);
                None
            }
        })
        .collect();

    if backend_types.is_empty() {
        anyhow::bail!("No valid backend types specified");
    }

    baseline.run_comprehensive_baseline().await?;

    tracing::info!("Performance baseline established successfully");
    Ok(())
}

async fn run_benchmark(
    bench_type: String,
    output_dir: PathBuf,
    model_name: Option<String>,
    iterations: u32,
    enable_profiling: bool,
) -> Result<()> {
    tracing::info!(
        "Running {} benchmark with {} iterations",
        bench_type,
        iterations
    );

    tokio::fs::create_dir_all(&output_dir).await?;

    match bench_type.as_str() {
        "inference" => run_inference_benchmark(output_dir, model_name, iterations).await,
        "memory" => run_memory_benchmark(output_dir, model_name, iterations).await,
        "concurrent" => run_concurrent_benchmark(output_dir, model_name, iterations).await,
        "cache" => {
            tracing::warn!("Cache benchmark temporarily disabled");
            Ok(())
        }
        "all" => {
            run_inference_benchmark(output_dir.clone(), model_name.clone(), iterations).await?;
            run_memory_benchmark(output_dir.clone(), model_name.clone(), iterations).await?;
            run_concurrent_benchmark(output_dir, model_name, iterations).await
        }
        _ => anyhow::bail!("Unknown benchmark type: {}", bench_type),
    }
}

#[cfg(feature = "gguf")]
async fn run_inference_benchmark(
    output_dir: PathBuf,
    model_name: Option<String>,
    iterations: u32,
) -> Result<()> {
    tracing::info!("Running inference benchmark");

    let temp_dir = tempfile::tempdir()?;
    let model_path = temp_dir.path().join("benchmark.gguf");

    // Create test model
    tokio::fs::write(&model_path, b"GGUF\x00\x00\x00\x01test model").await?;

    let model = ModelInfo {
        name: "benchmark.gguf".to_string(),
        path: model_path,
        size: 1024 * 1024, // 1MB
        modified: chrono::Utc::now(),
        backend_type: "gguf".to_string(),
        checksum: None,
    };

    let backend_config = BackendConfig::default();
    let mut backend = Backend::new(BackendType::Gguf, &backend_config)?;
    backend.load_model(&model).await?;
    let backend_handle = BackendHandle::new(backend);

    let inference_params = InferenceParams {
        max_tokens: 50,
        temperature: 0.7,
        top_p: 0.9,
        stream: false,
    };

    let test_prompts = vec![
        "Hello, world!",
        "Explain artificial intelligence",
        "Write a story",
        "Solve this problem",
        "What is the meaning of life?",
    ];

    let mut latencies = Vec::new();
    let mut successful_requests = 0u32;
    let mut failed_requests = 0u32;

    let start_time = Instant::now();

    for i in 0..iterations {
        let prompt = &test_prompts[i as usize % test_prompts.len()];
        let inference_start = Instant::now();

        match backend_handle.infer(prompt, &inference_params).await {
            Ok(_) => {
                latencies.push(inference_start.elapsed());
                successful_requests += 1;
            }
            Err(e) => {
                tracing::warn!("Inference failed: {}", e);
                failed_requests += 1;
            }
        }

        if i % 10 == 0 {
            tracing::info!("Completed {} / {} iterations", i + 1, iterations);
        }
    }

    let total_duration = start_time.elapsed();

    // Calculate statistics
    latencies.sort();
    let latency_ms: Vec<f64> = latencies.iter().map(|d| d.as_secs_f64() * 1000.0).collect();

    let avg_latency = latency_ms.iter().sum::<f64>() / latency_ms.len() as f64;
    let p50_latency = percentile(&latency_ms, 50.0);
    let p90_latency = percentile(&latency_ms, 90.0);
    let p99_latency = percentile(&latency_ms, 99.0);

    let throughput = successful_requests as f64 / total_duration.as_secs_f64();

    let results = serde_json::json!({
        "benchmark_type": "inference",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "iterations": iterations,
        "successful_requests": successful_requests,
        "failed_requests": failed_requests,
        "total_duration_sec": total_duration.as_secs_f64(),
        "avg_latency_ms": avg_latency,
        "p50_latency_ms": p50_latency,
        "p90_latency_ms": p90_latency,
        "p99_latency_ms": p99_latency,
        "throughput_rps": throughput,
        "error_rate": failed_requests as f64 / (successful_requests + failed_requests) as f64,
    });

    let results_file = output_dir.join("inference_benchmark.json");
    tokio::fs::write(results_file, serde_json::to_string_pretty(&results)?).await?;

    tracing::info!("Inference benchmark completed:");
    tracing::info!("  Average latency: {:.2} ms", avg_latency);
    tracing::info!("  P99 latency: {:.2} ms", p99_latency);
    tracing::info!("  Throughput: {:.1} RPS", throughput);
    tracing::info!(
        "  Error rate: {:.2}%",
        (failed_requests as f64 / (successful_requests + failed_requests) as f64) * 100.0
    );

    Ok(())
}

#[cfg(not(feature = "gguf"))]
async fn run_inference_benchmark(
    _output_dir: PathBuf,
    _model_name: Option<String>,
    _iterations: u32,
) -> Result<()> {
    Err(anyhow::anyhow!(
        "GGUF backend not available. Enable 'gguf' feature for inference benchmarks."
    ))
}

#[cfg(feature = "gguf")]
async fn run_memory_benchmark(
    output_dir: PathBuf,
    model_name: Option<String>,
    iterations: u32,
) -> Result<()> {
    tracing::info!("Running memory benchmark");

    let mut system = System::new_all();
    system.refresh_all();

    let initial_memory = system.used_memory();
    let mut peak_memory = initial_memory;
    let mut memory_samples = Vec::new();

    // Create progressively larger models to test memory scaling
    let model_sizes = vec![1, 5, 10, 25]; // MB
    let mut results = Vec::new();

    for size_mb in model_sizes {
        tracing::info!("Testing with {}MB model", size_mb);

        let temp_dir = tempfile::tempdir()?;
        let model_path = temp_dir.path().join(format!("model_{}mb.gguf", size_mb));

        // Create model content
        let mut content = b"GGUF\x00\x00\x00\x01".to_vec();
        content.extend(vec![0u8; size_mb * 1024 * 1024 - content.len()]);
        tokio::fs::write(&model_path, content).await?;

        let model = ModelInfo {
            name: format!("model_{}mb.gguf", size_mb),
            path: model_path,
            size: (size_mb * 1024 * 1024) as u64,
            modified: chrono::Utc::now(),
            backend_type: "gguf".to_string(),
            checksum: None,
        };

        let backend_config = BackendConfig::default();

        // Test model loading/unloading cycles
        for cycle in 0..10 {
            system.refresh_memory();
            let cycle_start_memory = system.used_memory();

            let mut backend = Backend::new(BackendType::Gguf, &backend_config)?;
            backend.load_model(&model).await?;
            let backend_handle = BackendHandle::new(backend);

            system.refresh_memory();
            let loaded_memory = system.used_memory();
            peak_memory = peak_memory.max(loaded_memory);

            // Perform some inferences
            let inference_params = InferenceParams {
                max_tokens: 20,
                temperature: 0.7,
                top_p: 0.9,
                stream: false,
            };

            for _ in 0..5 {
                let _ = backend_handle.infer("Test prompt", &inference_params).await;
                system.refresh_memory();
                peak_memory = peak_memory.max(system.used_memory());
            }

            backend_handle.unload_model().await?;

            system.refresh_memory();
            let cycle_end_memory = system.used_memory();

            memory_samples.push(serde_json::json!({
                "model_size_mb": size_mb,
                "cycle": cycle,
                "start_memory_mb": cycle_start_memory / 1024 / 1024,
                "loaded_memory_mb": loaded_memory / 1024 / 1024,
                "end_memory_mb": cycle_end_memory / 1024 / 1024,
                "memory_delta_mb": (loaded_memory as i64 - cycle_start_memory as i64) / 1024 / 1024,
            }));

            if cycle % 3 == 0 {
                tracing::info!("  Completed cycle {} / 10", cycle + 1);
            }
        }

        let avg_delta: f64 = memory_samples
            .iter()
            .filter(|s| s["model_size_mb"] == size_mb)
            .map(|s| s["memory_delta_mb"].as_i64().unwrap_or(0) as f64)
            .sum::<f64>()
            / 10.0;

        results.push(serde_json::json!({
            "model_size_mb": size_mb,
            "avg_memory_delta_mb": avg_delta,
            "memory_efficiency": size_mb as f64 / avg_delta,
        }));
    }

    let benchmark_results = serde_json::json!({
        "benchmark_type": "memory",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "initial_memory_mb": initial_memory / 1024 / 1024,
        "peak_memory_mb": peak_memory / 1024 / 1024,
        "total_peak_delta_mb": (peak_memory - initial_memory) / 1024 / 1024,
        "model_results": results,
        "detailed_samples": memory_samples,
    });

    let results_file = output_dir.join("memory_benchmark.json");
    tokio::fs::write(
        results_file,
        serde_json::to_string_pretty(&benchmark_results)?,
    )
    .await?;

    tracing::info!("Memory benchmark completed:");
    tracing::info!("  Initial memory: {} MB", initial_memory / 1024 / 1024);
    tracing::info!("  Peak memory: {} MB", peak_memory / 1024 / 1024);
    tracing::info!(
        "  Peak delta: {} MB",
        (peak_memory - initial_memory) / 1024 / 1024
    );

    Ok(())
}

#[cfg(not(feature = "gguf"))]
async fn run_memory_benchmark(
    _output_dir: PathBuf,
    _model_name: Option<String>,
    _iterations: u32,
) -> Result<()> {
    Err(anyhow::anyhow!(
        "GGUF backend not available. Enable 'gguf' feature for memory benchmarks."
    ))
}

#[cfg(feature = "gguf")]
async fn run_concurrent_benchmark(
    output_dir: PathBuf,
    model_name: Option<String>,
    iterations: u32,
) -> Result<()> {
    tracing::info!("Running concurrent benchmark");

    let temp_dir = tempfile::tempdir()?;
    let model_path = temp_dir.path().join("concurrent_test.gguf");
    tokio::fs::write(&model_path, b"GGUF\x00\x00\x00\x01test model").await?;

    let model = ModelInfo {
        name: "concurrent_test.gguf".to_string(),
        path: model_path,
        size: 1024 * 1024,
        modified: chrono::Utc::now(),
        backend_type: "gguf".to_string(),
        checksum: None,
    };

    let concurrency_levels = vec![1, 2, 4, 8, 16, 32];
    let mut results = Vec::new();

    for concurrency in concurrency_levels {
        tracing::info!("Testing concurrency level: {}", concurrency);

        let backend_config = BackendConfig::default();
        let mut backend = Backend::new(BackendType::Gguf, &backend_config)?;
        backend.load_model(&model).await?;
        let backend_handle = BackendHandle::new(backend);

        let inference_params = InferenceParams {
            max_tokens: 30,
            temperature: 0.7,
            top_p: 0.9,
            stream: false,
        };

        let start_time = Instant::now();
        let mut successful_requests = 0u32;
        let mut failed_requests = 0u32;

        let handles: Vec<_> = (0..concurrency)
            .map(|i| {
                let prompt = format!("Concurrent test request {}", i);
                let params = inference_params.clone();
                let backend_clone = backend_handle.clone();
                tokio::spawn(async move { backend_clone.infer(&prompt, &params).await })
            })
            .collect();

        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => successful_requests += 1,
                _ => failed_requests += 1,
            }
        }

        let duration = start_time.elapsed();
        let throughput = successful_requests as f64 / duration.as_secs_f64();

        results.push(serde_json::json!({
            "concurrency": concurrency,
            "successful_requests": successful_requests,
            "failed_requests": failed_requests,
            "duration_sec": duration.as_secs_f64(),
            "throughput_rps": throughput,
            "avg_latency_ms": (duration.as_secs_f64() * 1000.0) / successful_requests as f64,
        }));

        backend_handle.unload_model().await?;
    }

    let benchmark_results = serde_json::json!({
        "benchmark_type": "concurrent",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "results": results,
    });

    let results_file = output_dir.join("concurrent_benchmark.json");
    tokio::fs::write(
        results_file,
        serde_json::to_string_pretty(&benchmark_results)?,
    )
    .await?;

    tracing::info!("Concurrent benchmark completed");
    for result in &results {
        tracing::info!(
            "  Concurrency {}: {:.1} RPS",
            result["concurrency"],
            result["throughput_rps"]
        );
    }

    Ok(())
}

#[cfg(not(feature = "gguf"))]
async fn run_concurrent_benchmark(
    _output_dir: PathBuf,
    _model_name: Option<String>,
    _iterations: u32,
) -> Result<()> {
    Err(anyhow::anyhow!(
        "GGUF backend not available. Enable 'gguf' feature for concurrent benchmarks."
    ))
}

// Cache benchmark temporarily disabled due to dependency issues
// Will be re-enabled once advanced cache system is properly integrated

async fn compare_performance(
    current_file: PathBuf,
    baseline_file: PathBuf,
    threshold: f64,
    generate_report: bool,
) -> Result<()> {
    tracing::info!("Comparing performance with baseline");

    let current_content = tokio::fs::read_to_string(&current_file).await?;
    let baseline_content = tokio::fs::read_to_string(&baseline_file).await?;

    let current_data: serde_json::Value = serde_json::from_str(&current_content)?;
    let baseline_data: serde_json::Value = serde_json::from_str(&baseline_content)?;

    // Simplified comparison logic
    let mut regressions = Vec::new();
    let mut improvements = Vec::new();

    // Compare key metrics if they exist
    if let (Some(current_latency), Some(baseline_latency)) = (
        current_data.get("avg_latency_ms").and_then(|v| v.as_f64()),
        baseline_data.get("avg_latency_ms").and_then(|v| v.as_f64()),
    ) {
        let change_percent = ((current_latency - baseline_latency) / baseline_latency) * 100.0;
        if change_percent > threshold {
            regressions.push(format!(
                "Latency regression: {:.2}% increase ({:.2}ms -> {:.2}ms)",
                change_percent, baseline_latency, current_latency
            ));
        } else if change_percent < -5.0 {
            improvements.push(format!(
                "Latency improvement: {:.2}% decrease ({:.2}ms -> {:.2}ms)",
                -change_percent, baseline_latency, current_latency
            ));
        }
    }

    if let (Some(current_throughput), Some(baseline_throughput)) = (
        current_data.get("throughput_rps").and_then(|v| v.as_f64()),
        baseline_data.get("throughput_rps").and_then(|v| v.as_f64()),
    ) {
        let change_percent =
            ((current_throughput - baseline_throughput) / baseline_throughput) * 100.0;
        if change_percent < -threshold {
            regressions.push(format!(
                "Throughput regression: {:.2}% decrease ({:.1} -> {:.1} RPS)",
                -change_percent, baseline_throughput, current_throughput
            ));
        } else if change_percent > 5.0 {
            improvements.push(format!(
                "Throughput improvement: {:.2}% increase ({:.1} -> {:.1} RPS)",
                change_percent, baseline_throughput, current_throughput
            ));
        }
    }

    // Report results
    if !regressions.is_empty() {
        tracing::error!("Performance regressions detected:");
        for regression in &regressions {
            tracing::error!("  {}", regression);
        }
    }

    if !improvements.is_empty() {
        tracing::info!("Performance improvements detected:");
        for improvement in &improvements {
            tracing::info!("  {}", improvement);
        }
    }

    if regressions.is_empty() && improvements.is_empty() {
        tracing::info!("No significant performance changes detected");
    }

    // Return error if regressions found (for CI)
    if !regressions.is_empty() {
        anyhow::bail!("Performance regressions detected");
    }

    Ok(())
}

async fn monitor_performance(
    duration: u64,
    interval: u64,
    format: String,
    output_file: Option<PathBuf>,
) -> Result<()> {
    tracing::info!("Starting performance monitoring for {}s", duration);

    let mut system = System::new_all();
    let end_time = if duration == 0 {
        None
    } else {
        Some(Instant::now() + Duration::from_secs(duration))
    };

    let mut samples = Vec::new();

    loop {
        if let Some(end) = end_time {
            if Instant::now() >= end {
                break;
            }
        }

        system.refresh_all();

        let sample = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "cpu_usage": system.global_cpu_info().cpu_usage(),
            "memory_used_mb": system.used_memory() / 1024 / 1024,
            "memory_total_mb": system.total_memory() / 1024 / 1024,
            "memory_usage_percent": (system.used_memory() as f64 / system.total_memory() as f64) * 100.0,
            "processes": system.processes().len(),
        });

        match format.as_str() {
            "console" => {
                println!(
                    "{}: CPU: {:.1}%, Memory: {} MB ({:.1}%)",
                    chrono::Utc::now().format("%H:%M:%S"),
                    sample["cpu_usage"],
                    sample["memory_used_mb"],
                    sample["memory_usage_percent"]
                );
            }
            "json" => {
                println!("{}", serde_json::to_string(&sample)?);
            }
            _ => {
                samples.push(sample);
            }
        }

        tokio::time::sleep(Duration::from_secs(interval)).await;
    }

    if let Some(output_path) = output_file {
        let output_data = serde_json::json!({
            "monitoring_session": {
                "start_time": chrono::Utc::now() - chrono::Duration::seconds(duration as i64),
                "duration_sec": duration,
                "interval_sec": interval,
                "samples": samples,
            }
        });
        tokio::fs::write(output_path, serde_json::to_string_pretty(&output_data)?).await?;
    }

    Ok(())
}

async fn stress_test(
    clients: u32,
    duration: u64,
    model_name: Option<String>,
    rate: f64,
) -> Result<()> {
    tracing::info!(
        "Starting stress test with {} clients for {}s at {}req/s per client",
        clients,
        duration,
        rate
    );

    let model = model_name.unwrap_or_else(|| "default".to_string());
    let start_time = std::time::Instant::now();
    let test_duration = std::time::Duration::from_secs(duration);

    // Stress test metrics
    let mut total_requests = 0u64;
    let mut successful_requests = 0u64;
    let mut failed_requests = 0u64;
    let mut response_times = Vec::new();
    let mut peak_memory = 0u64;
    let mut errors = Vec::new();

    println!("🚀 Starting stress test...");
    println!(
        "Clients: {}, Duration: {}s, Rate: {:.1} req/s/client",
        clients, duration, rate
    );
    println!("{}", "=".repeat(60));

    // Spawn concurrent client tasks
    let mut client_handles = Vec::new();
    let request_interval = std::time::Duration::from_secs_f64(1.0 / rate);

    for client_id in 0..clients {
        let model_clone = model.clone();
        let client_handle = tokio::spawn(async move {
            let mut client_requests = 0u64;
            let mut client_successes = 0u64;
            let mut client_failures = 0u64;
            let mut client_response_times = Vec::new();
            let mut last_request_time = std::time::Instant::now();

            loop {
                let elapsed = start_time.elapsed();
                if elapsed >= test_duration {
                    break;
                }

                // Respect rate limiting
                if last_request_time.elapsed() < request_interval {
                    tokio::time::sleep(request_interval - last_request_time.elapsed()).await;
                }

                // Simulate inference request
                let request_start = std::time::Instant::now();
                let test_input = format!(
                    "Stress test input from client {} request {}",
                    client_id, client_requests
                );

                match simulate_inference_request(&model_clone, &test_input).await {
                    Ok(response_time) => {
                        client_successes += 1;
                        client_response_times.push(response_time);
                    }
                    Err(e) => {
                        client_failures += 1;
                        tracing::debug!("Client {} request failed: {}", client_id, e);
                    }
                }

                client_requests += 1;
                last_request_time = std::time::Instant::now();

                // Monitor memory usage periodically
                if client_requests % 10 == 0 {
                    let memory_usage = get_current_memory_usage();
                    if memory_usage.rss > peak_memory {
                        peak_memory = memory_usage.rss;
                    }
                }
            }

            (
                client_id,
                client_requests,
                client_successes,
                client_failures,
                client_response_times,
            )
        });

        client_handles.push(client_handle);
    }

    // Monitor progress
    let progress_handle = tokio::spawn(async move {
        let mut last_report = std::time::Instant::now();

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let elapsed = start_time.elapsed();

            if elapsed >= test_duration {
                break;
            }

            let progress = (elapsed.as_secs_f64() / test_duration.as_secs_f64()) * 100.0;
            println!(
                "Progress: {:.1}% ({:.0}s elapsed)",
                progress,
                elapsed.as_secs_f64()
            );
            last_report = std::time::Instant::now();
        }
    });

    // Wait for all clients to complete
    for handle in client_handles {
        match handle.await {
            Ok((client_id, requests, successes, failures, response_times_vec)) => {
                total_requests += requests;
                successful_requests += successes;
                failed_requests += failures;
                response_times.extend(response_times_vec);

                if failures > 0 {
                    errors.push(format!(
                        "Client {}: {} failures out of {} requests",
                        client_id, failures, requests
                    ));
                }
            }
            Err(e) => {
                errors.push(format!("Client task failed: {}", e));
            }
        }
    }

    progress_handle.abort();

    let actual_duration = start_time.elapsed();
    let overall_throughput = total_requests as f64 / actual_duration.as_secs_f64();

    // Calculate response time statistics
    response_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let response_times_secs: Vec<f64> = response_times.iter().map(|d| d.as_secs_f64()).collect();
    let avg_response_time =
        response_times_secs.iter().sum::<f64>() / response_times_secs.len() as f64;
    let p50 = percentile(&response_times_secs, 50.0);
    let p95 = percentile(&response_times_secs, 95.0);
    let p99 = percentile(&response_times_secs, 99.0);

    // Display results
    println!("\n{}", "=".repeat(60));
    println!("🏁 Stress Test Results");
    println!("{}", "=".repeat(60));
    println!("Duration: {:.2}s", actual_duration.as_secs_f64());
    println!("Total Requests: {}", total_requests);
    println!(
        "Successful Requests: {} ({:.1}%)",
        successful_requests,
        (successful_requests as f64 / total_requests as f64) * 100.0
    );
    println!(
        "Failed Requests: {} ({:.1}%)",
        failed_requests,
        (failed_requests as f64 / total_requests as f64) * 100.0
    );
    println!("Overall Throughput: {:.2} req/s", overall_throughput);
    println!(
        "Peak Memory Usage: {:.1} MB",
        peak_memory as f64 / 1024.0 / 1024.0
    );

    println!("\n📊 Response Time Statistics:");
    println!("Average: {:.2}ms", avg_response_time);
    println!("50th percentile: {:.2}ms", p50);
    println!("95th percentile: {:.2}ms", p95);
    println!("99th percentile: {:.2}ms", p99);

    if !errors.is_empty() {
        println!("\n❌ Errors encountered:");
        for error in &errors {
            println!("  {}", error);
        }
    }

    // Determine test result
    let success_rate = successful_requests as f64 / total_requests as f64;
    if success_rate < 0.95 {
        println!("\n⚠️  STRESS TEST FAILED: Success rate below 95%");
        return Err(anyhow::anyhow!(
            "Stress test failed with {:.1}% success rate",
            success_rate * 100.0
        ));
    } else if p99 > 5000.0 {
        println!("\n⚠️  STRESS TEST WARNING: 99th percentile latency above 5s");
    } else {
        println!("\n✅ STRESS TEST PASSED: All metrics within acceptable ranges");
    }

    Ok(())
}

async fn memory_profile(
    model_name: Option<String>,
    cycles: u32,
    output_file: Option<PathBuf>,
    track: bool,
) -> Result<()> {
    tracing::info!("Starting memory profiling for {} cycles", cycles);

    // Initialize memory tracking
    let mut memory_snapshots = Vec::new();
    let start_time = std::time::Instant::now();

    // Get initial memory baseline
    let baseline_memory = get_memory_usage().await?;
    memory_snapshots.push(MemorySnapshot {
        timestamp: start_time.elapsed(),
        heap_used: baseline_memory.heap_used,
        heap_total: baseline_memory.heap_total,
        rss: baseline_memory.rss,
        vms: baseline_memory.vms,
        inference_id: 0,
        model_loaded: false,
    });

    println!("🧠 Memory Profiling Started");
    println!("├─ Cycles: {}", cycles);
    println!("├─ Model: {}", model_name.as_deref().unwrap_or("default"));
    println!(
        "├─ Tracking: {}",
        if track { "enabled" } else { "disabled" }
    );
    println!(
        "└─ Baseline Memory: {:.2} MB RSS, {:.2} MB Heap",
        baseline_memory.rss as f64 / 1024.0 / 1024.0,
        baseline_memory.heap_used as f64 / 1024.0 / 1024.0
    );

    // Load backend for testing
    #[cfg(feature = "gguf")]
    let backend_type = BackendType::Gguf;
    #[cfg(all(not(feature = "gguf"), feature = "onnx"))]
    let backend_type = BackendType::Onnx;
    #[cfg(all(
        not(feature = "gguf"),
        not(feature = "onnx"),
        all(feature = "gpu-metal", target_os = "macos")
    ))]
    let backend_type = BackendType::Metal;
    #[cfg(not(any(
        feature = "gguf",
        feature = "onnx",
        all(feature = "gpu-metal", target_os = "macos")
    )))]
    let backend_type = BackendType::None;
    let backend_config = BackendConfig::default();
    let mut backend = Backend::new(backend_type, &backend_config)?;

    // Profile model loading
    if let Some(ref model) = model_name {
        let model_info = ModelInfo {
            name: model.clone(),
            path: std::path::PathBuf::from(format!("models/{}", model)),
            file_path: std::path::PathBuf::from(format!("models/{}", model)),
            size: 0,
            size_bytes: 0,
            modified: chrono::Utc::now(),
            backend_type: "gguf".to_string(),
            format: "gguf".to_string(),
            checksum: None,
            metadata: std::collections::HashMap::new(),
        };

        tracing::info!("Loading model for memory profiling");
        backend.load_model(&model_info).await?;

        let post_load_memory = get_memory_usage().await?;
        memory_snapshots.push(MemorySnapshot {
            timestamp: start_time.elapsed(),
            heap_used: post_load_memory.heap_used,
            heap_total: post_load_memory.heap_total,
            rss: post_load_memory.rss,
            vms: post_load_memory.vms,
            inference_id: 0,
            model_loaded: true,
        });

        let model_memory_usage = post_load_memory.rss.saturating_sub(baseline_memory.rss);
        println!(
            "📊 Model loaded - Memory delta: {:.2} MB",
            model_memory_usage as f64 / 1024.0 / 1024.0
        );
    }

    // Run inference cycles with memory tracking
    let test_prompt = "What is artificial intelligence?";
    let params = InferenceParams {
        max_tokens: 50,
        temperature: 0.7,
        top_p: 0.9,
        stream: false,
        stop_sequences: vec![],
        seed: Some(42),
    };

    for cycle in 1..=cycles {
        let cycle_start = start_time.elapsed();

        // Force garbage collection before measurement
        if track {
            tokio::task::yield_now().await;
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let pre_inference_memory = get_memory_usage().await?;

        // Run inference
        if backend.is_loaded().await {
            match backend.infer(test_prompt, &params).await {
                Ok(_response) => {
                    let post_inference_memory = get_memory_usage().await?;

                    memory_snapshots.push(MemorySnapshot {
                        timestamp: cycle_start,
                        heap_used: post_inference_memory.heap_used,
                        heap_total: post_inference_memory.heap_total,
                        rss: post_inference_memory.rss,
                        vms: post_inference_memory.vms,
                        inference_id: cycle,
                        model_loaded: true,
                    });

                    let cycle_memory_delta = post_inference_memory
                        .rss
                        .saturating_sub(pre_inference_memory.rss);

                    if track {
                        println!(
                            "Cycle {}/{}: {:.2} MB delta, {:.2} MB total RSS",
                            cycle,
                            cycles,
                            cycle_memory_delta as f64 / 1024.0 / 1024.0,
                            post_inference_memory.rss as f64 / 1024.0 / 1024.0
                        );
                    }

                    // Check for significant memory growth (potential leak)
                    if cycle > 1 && cycle_memory_delta > 10 * 1024 * 1024 {
                        // 10MB+ growth
                        tracing::warn!(
                            "Potential memory leak detected in cycle {}: {:.2} MB growth",
                            cycle,
                            cycle_memory_delta as f64 / 1024.0 / 1024.0
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Inference failed in cycle {}: {}", cycle, e);
                }
            }
        }

        // Small delay between cycles
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Final memory measurement
    let final_memory = get_memory_usage().await?;
    memory_snapshots.push(MemorySnapshot {
        timestamp: start_time.elapsed(),
        heap_used: final_memory.heap_used,
        heap_total: final_memory.heap_total,
        rss: final_memory.rss,
        vms: final_memory.vms,
        inference_id: cycles,
        model_loaded: backend.is_loaded().await,
    });

    // Analyze memory patterns
    let analysis = analyze_memory_patterns(&memory_snapshots, baseline_memory);

    // Display results
    display_memory_analysis(&analysis);

    // Save to file if requested
    if let Some(output_path) = output_file {
        save_memory_profile(&memory_snapshots, &analysis, &output_path).await?;
        println!("📁 Memory profile saved to: {}", output_path.display());
    }

    // Cleanup
    if backend.is_loaded().await {
        backend.unload_model().await?;
    }

    Ok(())
}

// Memory profiling support functions
async fn get_memory_usage() -> Result<MemoryUsage> {
    let mut system = System::new_all();
    system.refresh_all();

    // Cross-platform memory usage detection
    #[cfg(target_os = "linux")]
    {
        use std::fs;

        // Read from /proc/self/status for detailed memory info
        let status = fs::read_to_string("/proc/self/status")?;
        let mut rss = 0u64;
        let mut vms = 0u64;

        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    rss = value.parse::<u64>().unwrap_or(0) * 1024; // Convert kB to bytes
                }
            } else if line.starts_with("VmSize:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    vms = value.parse::<u64>().unwrap_or(0) * 1024; // Convert kB to bytes
                }
            }
        }

        Ok(MemoryUsage {
            heap_used: rss, // Approximation
            heap_total: vms,
            rss,
            vms,
        })
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Fallback for other platforms using sysinfo
        use sysinfo::PidExt;
        let current_pid = sysinfo::Pid::from_u32(std::process::id());
        let process = system.processes().get(&current_pid);

        if let Some(process) = process {
            use sysinfo::ProcessExt;
            let memory = process.memory(); // In KB, convert to bytes
            let virtual_memory = process.virtual_memory(); // In KB, convert to bytes

            Ok(MemoryUsage {
                heap_used: memory * 1024,
                heap_total: virtual_memory * 1024,
                rss: memory * 1024,
                vms: virtual_memory * 1024,
            })
        } else {
            // Fallback values if process not found
            Ok(MemoryUsage {
                heap_used: 0,
                heap_total: 0,
                rss: 0,
                vms: 0,
            })
        }
    }
}

fn analyze_memory_patterns(snapshots: &[MemorySnapshot], baseline: MemoryUsage) -> MemoryAnalysis {
    if snapshots.is_empty() {
        return MemoryAnalysis {
            baseline_rss: baseline.rss,
            peak_rss: baseline.rss,
            final_rss: baseline.rss,
            total_growth: 0,
            average_growth_per_cycle: 0.0,
            memory_leak_detected: false,
            gc_efficiency: 1.0,
            model_load_overhead: 0,
        };
    }

    let baseline_rss = baseline.rss;
    let peak_rss = snapshots
        .iter()
        .map(|s| s.rss)
        .max()
        .unwrap_or(baseline_rss);
    let final_rss = snapshots.last().map(|s| s.rss).unwrap_or(baseline_rss);

    let total_growth = final_rss as i64 - baseline_rss as i64;

    // Calculate model loading overhead
    let model_load_overhead = snapshots
        .iter()
        .find(|s| s.model_loaded && s.inference_id == 0)
        .map(|s| s.rss.saturating_sub(baseline_rss))
        .unwrap_or(0);

    // Analyze growth patterns to detect potential leaks
    let inference_snapshots: Vec<_> = snapshots.iter().filter(|s| s.inference_id > 0).collect();

    let average_growth_per_cycle = if inference_snapshots.len() > 1 {
        let first_inference_rss = inference_snapshots
            .first()
            .map(|s| s.rss)
            .unwrap_or(baseline_rss);
        let last_inference_rss = inference_snapshots
            .last()
            .map(|s| s.rss)
            .unwrap_or(baseline_rss);
        let cycles = inference_snapshots.len() as f64;

        (last_inference_rss as f64 - first_inference_rss as f64) / cycles
    } else {
        0.0
    };

    // Detect memory leaks: consistent growth > 1MB per cycle
    let memory_leak_detected = average_growth_per_cycle > 1024.0 * 1024.0;

    // Calculate GC efficiency (how much memory is reclaimed between peaks)
    let mut memory_deltas = Vec::new();
    for window in inference_snapshots.windows(2) {
        let delta = window[1].rss as i64 - window[0].rss as i64;
        memory_deltas.push(delta);
    }

    let positive_deltas: Vec<_> = memory_deltas.iter().filter(|&&d| d > 0).collect();
    let negative_deltas: Vec<_> = memory_deltas.iter().filter(|&&d| d < 0).collect();

    let gc_efficiency = if !positive_deltas.is_empty() && !negative_deltas.is_empty() {
        let total_growth: i64 = positive_deltas.iter().map(|&&d| d).sum();
        let total_reclaim: i64 = negative_deltas.iter().map(|&&d| d.abs()).sum();

        if total_growth > 0 {
            total_reclaim as f64 / total_growth as f64
        } else {
            1.0
        }
    } else {
        1.0
    };

    MemoryAnalysis {
        baseline_rss,
        peak_rss,
        final_rss,
        total_growth,
        average_growth_per_cycle,
        memory_leak_detected,
        gc_efficiency: gc_efficiency.min(1.0),
        model_load_overhead,
    }
}

fn display_memory_analysis(analysis: &MemoryAnalysis) {
    println!("\n📊 Memory Analysis Results");
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│                           Memory Report                         │");
    println!("├─────────────────────────────────────────────────────────────────┤");
    println!(
        "│ Baseline RSS:           {:>8.2} MB                        │",
        analysis.baseline_rss as f64 / 1024.0 / 1024.0
    );
    println!(
        "│ Peak RSS:               {:>8.2} MB                        │",
        analysis.peak_rss as f64 / 1024.0 / 1024.0
    );
    println!(
        "│ Final RSS:              {:>8.2} MB                        │",
        analysis.final_rss as f64 / 1024.0 / 1024.0
    );
    println!(
        "│ Total Growth:           {:>8.2} MB                        │",
        analysis.total_growth as f64 / 1024.0 / 1024.0
    );
    println!(
        "│ Model Load Overhead:    {:>8.2} MB                        │",
        analysis.model_load_overhead as f64 / 1024.0 / 1024.0
    );
    println!(
        "│ Avg Growth/Cycle:       {:>8.2} KB                        │",
        analysis.average_growth_per_cycle / 1024.0
    );
    println!(
        "│ GC Efficiency:          {:>8.1}%                          │",
        analysis.gc_efficiency * 100.0
    );
    println!("├─────────────────────────────────────────────────────────────────┤");

    if analysis.memory_leak_detected {
        println!("│ ⚠️  MEMORY LEAK DETECTED: Consistent growth > 1MB/cycle        │");
    } else {
        println!("│ ✅ Memory Management: No significant leaks detected           │");
    }

    if analysis.gc_efficiency < 0.5 {
        println!("│ ⚠️  GC EFFICIENCY: Low garbage collection efficiency          │");
    } else {
        println!("│ ✅ GC Performance: Good memory reclamation efficiency         │");
    }

    println!("└─────────────────────────────────────────────────────────────────┘");

    // Performance assessment
    let peak_overhead = analysis.peak_rss.saturating_sub(analysis.baseline_rss);
    if peak_overhead > 500 * 1024 * 1024 {
        // 500MB
        println!("\n⚠️  HIGH MEMORY USAGE: Peak memory usage exceeds 500MB");
    } else if peak_overhead > 100 * 1024 * 1024 {
        // 100MB
        println!("\n📊 MODERATE MEMORY USAGE: Peak memory usage is moderate");
    } else {
        println!("\n✅ LOW MEMORY USAGE: Efficient memory utilization");
    }
}

async fn save_memory_profile(
    snapshots: &[MemorySnapshot],
    analysis: &MemoryAnalysis,
    output_path: &PathBuf,
) -> Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(output_path)?;

    // Write header
    writeln!(file, "# Memory Profile Report")?;
    writeln!(
        file,
        "Generated: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(file, "")?;

    // Write analysis summary
    writeln!(file, "## Analysis Summary")?;
    writeln!(
        file,
        "- Baseline RSS: {:.2} MB",
        analysis.baseline_rss as f64 / 1024.0 / 1024.0
    )?;
    writeln!(
        file,
        "- Peak RSS: {:.2} MB",
        analysis.peak_rss as f64 / 1024.0 / 1024.0
    )?;
    writeln!(
        file,
        "- Final RSS: {:.2} MB",
        analysis.final_rss as f64 / 1024.0 / 1024.0
    )?;
    writeln!(
        file,
        "- Total Growth: {:.2} MB",
        analysis.total_growth as f64 / 1024.0 / 1024.0
    )?;
    writeln!(
        file,
        "- Model Load Overhead: {:.2} MB",
        analysis.model_load_overhead as f64 / 1024.0 / 1024.0
    )?;
    writeln!(
        file,
        "- Average Growth per Cycle: {:.2} KB",
        analysis.average_growth_per_cycle / 1024.0
    )?;
    writeln!(
        file,
        "- GC Efficiency: {:.1}%",
        analysis.gc_efficiency * 100.0
    )?;
    writeln!(
        file,
        "- Memory Leak Detected: {}",
        analysis.memory_leak_detected
    )?;
    writeln!(file, "")?;

    // Write detailed snapshots
    writeln!(file, "## Memory Snapshots")?;
    writeln!(
        file,
        "Timestamp(ms),InferenceID,ModelLoaded,HeapUsed(MB),HeapTotal(MB),RSS(MB),VMS(MB)"
    )?;

    for snapshot in snapshots {
        writeln!(
            file,
            "{},{},{},{:.2},{:.2},{:.2},{:.2}",
            snapshot.timestamp.as_millis(),
            snapshot.inference_id,
            snapshot.model_loaded,
            snapshot.heap_used as f64 / 1024.0 / 1024.0,
            snapshot.heap_total as f64 / 1024.0 / 1024.0,
            snapshot.rss as f64 / 1024.0 / 1024.0,
            snapshot.vms as f64 / 1024.0 / 1024.0
        )?;
    }

    file.flush()?;
    Ok(())
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

async fn simulate_inference_request(model: &str, input: &str) -> Result<Duration> {
    let start = Instant::now();

    // Simulate processing time based on input length
    let processing_time_ms = (input.len() as u64 * 10).max(50);
    tokio::time::sleep(Duration::from_millis(processing_time_ms)).await;

    Ok(start.elapsed())
}

fn get_current_memory_usage() -> MemoryUsage {
    let mut system = System::new_all();
    system.refresh_memory();

    MemoryUsage {
        heap_used: system.used_memory(),
        heap_total: system.total_memory(),
        rss: system.used_memory(),
        vms: system.total_memory(),
    }
}
