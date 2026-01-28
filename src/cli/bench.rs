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
    // Pre-execution validation
    validate_args(&args)?;

    info!("Starting benchmark for model: {}", args.model);

    let model_manager = ModelManager::new(&config.models_dir);
    let model_info = model_manager.resolve_model(&args.model).await?;

    let backend_type = args
        .backend
        .or_else(|| BackendType::from_model_path(&model_info.path))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No suitable backend found for model: {}",
                model_info.path.display()
            )
        })?;

    let mut backend = Backend::new(backend_type, &config.backend_config)?;

    println!("Loading model: {}", model_info.name);
    let load_start = Instant::now();
    backend.load_model(&model_info).await?;
    let load_time = load_start.elapsed();

    println!("Model loaded in: {:?}", load_time);
    println!();

    let prompt = args
        .prompt
        .unwrap_or_else(|| "The quick brown fox jumps over the lazy dog.".to_string());

    let inference_params = InferenceParams {
        max_tokens: args.tokens,
        temperature: 0.7,
        top_k: 40,
        top_p: 0.9,
        stream: false,
        stop_sequences: vec![],
        seed: None,
    };

    println!("Benchmark Configuration:");
    println!("  Model: {}", model_info.name);
    println!("  Backend: {}", backend_type);
    println!("  Iterations: {}", args.iterations);
    println!("  Warmup: {}", args.warmup);
    println!("  Max tokens: {}", args.tokens);
    println!(
        "  Prompt: {}",
        if prompt.len() > 50 {
            format!("{}...", &prompt[..50])
        } else {
            prompt.clone()
        }
    );
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
    let mean = calculate_mean(&durations);

    let total_tokens_per_sec = total_tokens as f64 / total_time.as_secs_f64();
    let mean_tokens_per_sec = args.tokens as f64 / mean.as_secs_f64();

    println!("\nBenchmark Results:");
    println!("==================");
    println!("Total time: {:?}", total_time);
    println!("Total tokens: {}", total_tokens);
    println!("Throughput: {:.1} tokens/sec", total_tokens_per_sec);
    println!();
    println!("Per-iteration statistics:");
    println!(
        "  Min:    {:?} ({:.1} tok/s)",
        min,
        args.tokens as f64 / min.as_secs_f64()
    );
    println!(
        "  Max:    {:?} ({:.1} tok/s)",
        max,
        args.tokens as f64 / max.as_secs_f64()
    );
    println!("  Mean:   {:?} ({:.1} tok/s)", mean, mean_tokens_per_sec);
    println!(
        "  Median: {:?} ({:.1} tok/s)",
        median,
        args.tokens as f64 / median.as_secs_f64()
    );
    println!();

    // Performance classification
    let performance_rating = classify_performance(mean_tokens_per_sec);
    println!("Performance: {}", performance_rating);

    // Memory usage estimation
    if let Ok(memory_info) = get_memory_info() {
        println!("Estimated memory usage: {:.1} GB", memory_info.used_gb);
    }

    Ok(())
}

/// Validate benchmark arguments before execution
fn validate_args(args: &BenchArgs) -> Result<()> {
    // Validate model name
    if args.model.is_empty() {
        anyhow::bail!("Model name cannot be empty");
    }

    // Validate iterations
    if args.iterations == 0 {
        anyhow::bail!("Iterations must be greater than 0");
    }

    if args.iterations > 1000 {
        anyhow::bail!("Iterations must be 1000 or less to ensure reasonable benchmark times");
    }

    // Validate tokens
    if args.tokens == 0 {
        anyhow::bail!("Tokens must be greater than 0");
    }

    if args.tokens > 10000 {
        anyhow::bail!("Tokens must be 10000 or less to ensure reasonable benchmark times");
    }

    // Warmup is optional (can be 0), but cap at reasonable limit
    if args.warmup > 100 {
        anyhow::bail!("Warmup iterations must be 100 or less");
    }

    Ok(())
}

fn estimate_token_count(text: &str) -> u32 {
    // Rough estimation: ~4 characters per token for English text
    (text.len() as f32 / 4.0).ceil() as u32
}

fn calculate_mean(durations: &[Duration]) -> Duration {
    let total_nanos: u128 = durations.iter().map(|d| d.as_nanos()).sum();
    Duration::from_nanos((total_nanos / durations.len() as u128) as u64)
}

fn classify_performance(tokens_per_sec: f64) -> String {
    if tokens_per_sec > 100.0 {
        "Excellent (>100 tok/s)".to_string()
    } else if tokens_per_sec > 50.0 {
        "Good (50-100 tok/s)".to_string()
    } else if tokens_per_sec > 20.0 {
        "Fair (20-50 tok/s)".to_string()
    } else {
        "Needs improvement (<20 tok/s)".to_string()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_mean() {
        let durations = vec![
            Duration::from_millis(100),
            Duration::from_millis(200),
            Duration::from_millis(300),
        ];
        let mean = calculate_mean(&durations);
        assert_eq!(mean.as_millis(), 200);
    }

    #[test]
    fn test_estimate_token_count() {
        let text = "Hello, world!"; // 13 characters
        let count = estimate_token_count(text);
        assert_eq!(count, 4); // 13 / 4 = 3.25, ceil = 4
    }

    #[test]
    fn test_classify_performance() {
        assert_eq!(classify_performance(150.0), "Excellent (>100 tok/s)");
        assert_eq!(classify_performance(75.0), "Good (50-100 tok/s)");
        assert_eq!(classify_performance(35.0), "Fair (20-50 tok/s)");
        assert_eq!(classify_performance(10.0), "Needs improvement (<20 tok/s)");
    }

    #[test]
    fn test_validate_args_empty_model() {
        let args = BenchArgs {
            model: String::new(),
            iterations: 10,
            prompt: None,
            tokens: 100,
            warmup: 3,
            backend: None,
            verbose: false,
        };
        let result = validate_args(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Model name cannot be empty"));
    }

    #[test]
    fn test_validate_args_zero_iterations() {
        let args = BenchArgs {
            model: "test-model".to_string(),
            iterations: 0,
            prompt: None,
            tokens: 100,
            warmup: 3,
            backend: None,
            verbose: false,
        };
        let result = validate_args(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Iterations must be greater than 0"));
    }

    #[test]
    fn test_validate_args_too_many_iterations() {
        let args = BenchArgs {
            model: "test-model".to_string(),
            iterations: 1001,
            prompt: None,
            tokens: 100,
            warmup: 3,
            backend: None,
            verbose: false,
        };
        let result = validate_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("1000 or less"));
    }

    #[test]
    fn test_validate_args_zero_tokens() {
        let args = BenchArgs {
            model: "test-model".to_string(),
            iterations: 10,
            prompt: None,
            tokens: 0,
            warmup: 3,
            backend: None,
            verbose: false,
        };
        let result = validate_args(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Tokens must be greater than 0"));
    }

    #[test]
    fn test_validate_args_too_many_tokens() {
        let args = BenchArgs {
            model: "test-model".to_string(),
            iterations: 10,
            prompt: None,
            tokens: 10001,
            warmup: 3,
            backend: None,
            verbose: false,
        };
        let result = validate_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("10000 or less"));
    }

    #[test]
    fn test_validate_args_too_many_warmup() {
        let args = BenchArgs {
            model: "test-model".to_string(),
            iterations: 10,
            prompt: None,
            tokens: 100,
            warmup: 101,
            backend: None,
            verbose: false,
        };
        let result = validate_args(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Warmup iterations must be 100 or less"));
    }

    #[test]
    fn test_validate_args_valid() {
        let args = BenchArgs {
            model: "test-model".to_string(),
            iterations: 10,
            prompt: Some("test prompt".to_string()),
            tokens: 100,
            warmup: 3,
            backend: None,
            verbose: true,
        };
        let result = validate_args(&args);
        assert!(result.is_ok());
    }
}
