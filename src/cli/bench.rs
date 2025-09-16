use crate::backends::{Backend, BackendType, InferenceParams};
use crate::config::Config;
use crate::models::ModelManager;
use anyhow::Result;
use clap::Args;
use std::time::{Duration, Instant};
use tracing::info;

#[derive(Args)]
pub struct BenchArgs {
    #[arg(short, long, help = "Model file path or name")]
    pub model: String,

    #[arg(short, long, help = "Number of iterations", default_value = "10")]
    pub iterations: u32,

    #[arg(long, help = "Prompt for text generation benchmarks")]
    pub prompt: Option<String>,

    #[arg(long, help = "Number of tokens to generate", default_value = "100")]
    pub tokens: u32,

    #[arg(long, help = "Warmup iterations", default_value = "3")]
    pub warmup: u32,

    #[arg(long, help = "Backend to use", value_enum)]
    pub backend: Option<BackendType>,

    #[arg(long, help = "Enable detailed per-iteration output")]
    pub verbose: bool,
}

pub async fn execute(args: BenchArgs, config: &Config) -> Result<()> {
    info!("Starting benchmark for model: {}", args.model);

    let model_manager = ModelManager::new(&config.models_dir);
    let model_info = model_manager.resolve_model(&args.model).await?;

    let backend_type = args.backend.unwrap_or_else(|| {
        BackendType::from_model_path(&model_info.path)
    });

    let mut backend = Backend::new(backend_type, &config.backend_config)?;

    println!("Loading model: {}", model_info.name);
    let load_start = Instant::now();
    backend.load_model(&model_info).await?;
    let load_time = load_start.elapsed();

    println!("Model loaded in: {:?}", load_time);
    println!();

    let prompt = args.prompt.unwrap_or_else(|| {
        "The quick brown fox jumps over the lazy dog.".to_string()
    });

    let inference_params = InferenceParams {
        max_tokens: args.tokens,
        temperature: 0.7,
        top_p: 0.9,
        stream: false,
    };

    println!("Benchmark Configuration:");
    println!("  Model: {}", model_info.name);
    println!("  Backend: {}", backend_type);
    println!("  Iterations: {}", args.iterations);
    println!("  Warmup: {}", args.warmup);
    println!("  Max tokens: {}", args.tokens);
    println!("  Prompt: {}", if prompt.len() > 50 {
        format!("{}...", &prompt[..50])
    } else {
        prompt.clone()
    });
    println!();

    // Warmup
    if args.warmup > 0 {
        println!("Warming up ({} iterations)...", args.warmup);
        for i in 1..=args.warmup {
            let start = Instant::now();
            let _ = backend.infer(&prompt, &inference_params).await?;
            let duration = start.elapsed();
            if args.verbose {
                println!("  Warmup {}: {:?}", i, duration);
            }
        }
        println!("Warmup completed.\n");
    }

    // Benchmark
    println!("Running benchmark...");
    let mut durations = Vec::new();
    let mut total_tokens = 0u32;

    let bench_start = Instant::now();

    for i in 1..=args.iterations {
        let start = Instant::now();
        let result = backend.infer(&prompt, &inference_params).await?;
        let duration = start.elapsed();

        let token_count = estimate_token_count(&result);
        total_tokens += token_count;
        durations.push(duration);

        if args.verbose {
            println!(
                "  Iteration {}: {:?} ({} tokens, {:.1} tok/s)",
                i,
                duration,
                token_count,
                token_count as f64 / duration.as_secs_f64()
            );
        }
    }

    let total_time = bench_start.elapsed();

    // Statistics
    durations.sort();
    let min = durations[0];
    let max = durations[durations.len() - 1];
    let median = durations[durations.len() / 2];
    let mean: Duration = Duration::from_nanos(
        durations.iter().map(|d| d.as_nanos()).sum::<u128>() as u64 / durations.len() as u64
    );

    let total_tokens_per_sec = total_tokens as f64 / total_time.as_secs_f64();
    let mean_tokens_per_sec = args.tokens as f64 / mean.as_secs_f64();

    println!("\nBenchmark Results:");
    println!("==================");
    println!("Total time: {:?}", total_time);
    println!("Total tokens: {}", total_tokens);
    println!("Throughput: {:.1} tokens/sec", total_tokens_per_sec);
    println!();
    println!("Per-iteration statistics:");
    println!("  Min:    {:?} ({:.1} tok/s)", min, args.tokens as f64 / min.as_secs_f64());
    println!("  Max:    {:?} ({:.1} tok/s)", max, args.tokens as f64 / max.as_secs_f64());
    println!("  Mean:   {:?} ({:.1} tok/s)", mean, mean_tokens_per_sec);
    println!("  Median: {:?} ({:.1} tok/s)", median, args.tokens as f64 / median.as_secs_f64());
    println!();

    // Performance classification
    if mean_tokens_per_sec > 100.0 {
        println!("Performance: Excellent (>100 tok/s)");
    } else if mean_tokens_per_sec > 50.0 {
        println!("Performance: Good (50-100 tok/s)");
    } else if mean_tokens_per_sec > 20.0 {
        println!("Performance: Fair (20-50 tok/s)");
    } else {
        println!("Performance: Needs improvement (<20 tok/s)");
    }

    // Memory usage estimation
    if let Ok(memory_info) = get_memory_info() {
        println!("Estimated memory usage: {:.1} GB", memory_info.used_gb);
    }

    Ok(())
}

fn estimate_token_count(text: &str) -> u32 {
    // Rough estimation: ~4 characters per token for English text
    (text.len() as f32 / 4.0).ceil() as u32
}

fn get_memory_info() -> Result<MemoryInfo> {
    use sysinfo::{System, SystemExt};
    let mut sys = System::new_all();
    sys.refresh_memory();

    Ok(MemoryInfo {
        used_gb: sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
        total_gb: sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
    })
}

struct MemoryInfo {
    used_gb: f64,
    #[allow(dead_code)]
    total_gb: f64,
}