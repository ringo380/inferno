//! Bench Command - New Architecture
//!
//! This module demonstrates the migration of the bench command to the new
//! CLI architecture with Command trait, pipeline, and middleware support.
//!
//! Benchmarks model inference performance with warmup and statistical analysis.

use crate::backends::{Backend, BackendType, InferenceParams};
use crate::config::Config;
use crate::interfaces::cli::{Command, CommandContext, CommandOutput};
use crate::models::ModelManager;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::time::{Duration, Instant};
use tracing::info;

// ============================================================================
// BenchCommand - Model performance benchmarking
// ============================================================================

/// Benchmark model inference performance
pub struct BenchCommand {
    config: Config,
    model: String,
    iterations: u32,
    prompt: Option<String>,
    tokens: u32,
    warmup: u32,
    backend: Option<BackendType>,
}

impl BenchCommand {
    pub fn new(
        config: Config,
        model: String,
        iterations: u32,
        prompt: Option<String>,
        tokens: u32,
        warmup: u32,
        backend: Option<BackendType>,
    ) -> Self {
        Self {
            config,
            model,
            iterations,
            prompt,
            tokens,
            warmup,
            backend,
        }
    }
}

#[async_trait]
impl Command for BenchCommand {
    fn name(&self) -> &str {
        "bench"
    }

    fn description(&self) -> &str {
        "Benchmark model inference performance"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // Validate model name
        if self.model.is_empty() {
            anyhow::bail!("Model name cannot be empty");
        }

        // Validate iterations
        if self.iterations == 0 {
            anyhow::bail!("Iterations must be greater than 0");
        }

        if self.iterations > 1000 {
            anyhow::bail!("Iterations must be 1000 or less");
        }

        // Validate tokens
        if self.tokens == 0 {
            anyhow::bail!("Tokens must be greater than 0");
        }

        if self.tokens > 10000 {
            anyhow::bail!("Tokens must be 10000 or less (for reasonable benchmark times)");
        }

        // Warmup is optional, can be 0
        if self.warmup > 100 {
            anyhow::bail!("Warmup iterations must be 100 or less");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Starting benchmark for model: {}", self.model);

        // Resolve model
        let model_manager = ModelManager::new(&self.config.models_dir);
        let model_info = model_manager.resolve_model(&self.model).await?;

        // Determine backend
        let backend_type = self
            .backend
            .or_else(|| BackendType::from_model_path(&model_info.path))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No suitable backend found for model: {}",
                    model_info.path.display()
                )
            })?;

        let mut backend = Backend::new(backend_type, &self.config.backend_config)?;

        // Load model
        if !ctx.json_output {
            println!("Loading model: {}", model_info.name);
        }
        let load_start = Instant::now();
        backend.load_model(&model_info).await?;
        let load_time = load_start.elapsed();

        if !ctx.json_output {
            println!("Model loaded in: {:?}\n", load_time);
        }

        // Prepare prompt
        let prompt = self
            .prompt
            .clone()
            .unwrap_or_else(|| "The quick brown fox jumps over the lazy dog.".to_string());

        let inference_params = InferenceParams {
            max_tokens: self.tokens,
            temperature: 0.7,
            top_p: 0.9,
            stream: false,
            stop_sequences: vec![],
            seed: None,
        };

        // Print configuration
        if !ctx.json_output {
            self.print_config(&model_info.name, &backend_type, &prompt);
        }

        // Run warmup
        let warmup_stats = if self.warmup > 0 {
            Some(
                self.run_warmup(&mut backend, &prompt, &inference_params, ctx)
                    .await?,
            )
        } else {
            None
        };

        // Run benchmark
        let bench_stats = self
            .run_benchmark(&mut backend, &prompt, &inference_params, ctx)
            .await?;

        // Print results
        if !ctx.json_output {
            self.print_results(&bench_stats);
        }

        // Build structured output
        let result_json = json!({
            "model": model_info.name,
            "backend": backend_type.to_string(),
            "load_time_ms": load_time.as_millis(),
            "config": {
                "iterations": self.iterations,
                "warmup": self.warmup,
                "max_tokens": self.tokens,
                "prompt_length": prompt.len(),
            },
            "warmup": warmup_stats.as_ref().map(|s| json!({
                "iterations": s.iterations,
                "total_time_ms": s.total_time.as_millis(),
                "mean_ms": s.mean.as_millis(),
            })),
            "benchmark": {
                "iterations": bench_stats.iterations,
                "total_time_ms": bench_stats.total_time.as_millis(),
                "total_tokens": bench_stats.total_tokens,
                "throughput_tokens_per_sec": bench_stats.throughput_tokens_per_sec,
                "statistics": {
                    "min_ms": bench_stats.min.as_millis(),
                    "max_ms": bench_stats.max.as_millis(),
                    "mean_ms": bench_stats.mean.as_millis(),
                    "median_ms": bench_stats.median.as_millis(),
                    "min_tokens_per_sec": self.tokens as f64 / bench_stats.min.as_secs_f64(),
                    "max_tokens_per_sec": self.tokens as f64 / bench_stats.max.as_secs_f64(),
                    "mean_tokens_per_sec": bench_stats.mean_tokens_per_sec,
                    "median_tokens_per_sec": self.tokens as f64 / bench_stats.median.as_secs_f64(),
                },
                "performance_rating": bench_stats.performance_rating.clone(),
            },
            "memory": bench_stats.memory_info.as_ref().map(|m| json!({
                "used_gb": m.used_gb,
                "total_gb": m.total_gb,
            })),
        });

        Ok(CommandOutput::success_with_data(
            format!(
                "Benchmark completed: {:.1} tokens/sec (mean over {} iterations)",
                bench_stats.mean_tokens_per_sec, self.iterations
            ),
            result_json,
        ))
    }
}

impl BenchCommand {
    /// Print benchmark configuration
    fn print_config(&self, model_name: &str, backend_type: &BackendType, prompt: &str) {
        println!("Benchmark Configuration:");
        println!("  Model: {}", model_name);
        println!("  Backend: {}", backend_type);
        println!("  Iterations: {}", self.iterations);
        println!("  Warmup: {}", self.warmup);
        println!("  Max tokens: {}", self.tokens);
        println!(
            "  Prompt: {}",
            if prompt.len() > 50 {
                format!("{}...", &prompt[..50])
            } else {
                prompt.to_string()
            }
        );
        println!();
    }

    /// Run warmup iterations
    async fn run_warmup(
        &self,
        backend: &mut Backend,
        prompt: &str,
        params: &InferenceParams,
        ctx: &CommandContext,
    ) -> Result<BenchStats> {
        if !ctx.json_output {
            println!("Warming up ({} iterations)...", self.warmup);
        }

        let mut durations = Vec::new();
        let start = Instant::now();

        for i in 1..=self.warmup {
            let iter_start = Instant::now();
            let _ = backend.infer(prompt, params).await?;
            let duration = iter_start.elapsed();
            durations.push(duration);

            if ctx.is_verbose() && !ctx.json_output {
                println!("  Warmup {}: {:?}", i, duration);
            }
        }

        if !ctx.json_output {
            println!("Warmup completed.\n");
        }

        let total_time = start.elapsed();
        let mean = calculate_mean(&durations);

        Ok(BenchStats {
            iterations: self.warmup,
            durations,
            total_time,
            total_tokens: 0, // Not tracked for warmup
            throughput_tokens_per_sec: 0.0,
            min: Duration::default(),
            max: Duration::default(),
            mean,
            median: Duration::default(),
            mean_tokens_per_sec: 0.0,
            performance_rating: String::new(),
            memory_info: None,
        })
    }

    /// Run benchmark iterations
    async fn run_benchmark(
        &self,
        backend: &mut Backend,
        prompt: &str,
        params: &InferenceParams,
        ctx: &CommandContext,
    ) -> Result<BenchStats> {
        if !ctx.json_output {
            println!("Running benchmark...");
        }

        let mut durations = Vec::new();
        let mut total_tokens = 0u32;
        let bench_start = Instant::now();

        for i in 1..=self.iterations {
            let start = Instant::now();
            let result = backend.infer(prompt, params).await?;
            let duration = start.elapsed();

            let token_count = estimate_token_count(&result);
            total_tokens += token_count;
            durations.push(duration);

            if ctx.is_verbose() && !ctx.json_output {
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

        // Calculate statistics
        let mut sorted_durations = durations.clone();
        sorted_durations.sort();

        let min = sorted_durations[0];
        let max = sorted_durations[sorted_durations.len() - 1];
        let median = sorted_durations[sorted_durations.len() / 2];
        let mean = calculate_mean(&durations);

        let throughput_tokens_per_sec = total_tokens as f64 / total_time.as_secs_f64();
        let mean_tokens_per_sec = self.tokens as f64 / mean.as_secs_f64();

        let performance_rating = classify_performance(mean_tokens_per_sec);
        let memory_info = get_memory_info().ok();

        Ok(BenchStats {
            iterations: self.iterations,
            durations,
            total_time,
            total_tokens,
            throughput_tokens_per_sec,
            min,
            max,
            mean,
            median,
            mean_tokens_per_sec,
            performance_rating,
            memory_info,
        })
    }

    /// Print benchmark results
    fn print_results(&self, stats: &BenchStats) {
        println!("\nBenchmark Results:");
        println!("==================");
        println!("Total time: {:?}", stats.total_time);
        println!("Total tokens: {}", stats.total_tokens);
        println!(
            "Throughput: {:.1} tokens/sec",
            stats.throughput_tokens_per_sec
        );
        println!();
        println!("Per-iteration statistics:");
        println!(
            "  Min:    {:?} ({:.1} tok/s)",
            stats.min,
            self.tokens as f64 / stats.min.as_secs_f64()
        );
        println!(
            "  Max:    {:?} ({:.1} tok/s)",
            stats.max,
            self.tokens as f64 / stats.max.as_secs_f64()
        );
        println!(
            "  Mean:   {:?} ({:.1} tok/s)",
            stats.mean, stats.mean_tokens_per_sec
        );
        println!(
            "  Median: {:?} ({:.1} tok/s)",
            stats.median,
            self.tokens as f64 / stats.median.as_secs_f64()
        );
        println!();
        println!("Performance: {}", stats.performance_rating);

        if let Some(ref memory) = stats.memory_info {
            println!("Estimated memory usage: {:.1} GB", memory.used_gb);
        }
    }
}

// ============================================================================
// Helper Types and Functions
// ============================================================================

#[derive(Debug)]
struct BenchStats {
    iterations: u32,
    durations: Vec<Duration>,
    total_time: Duration,
    total_tokens: u32,
    throughput_tokens_per_sec: f64,
    min: Duration,
    max: Duration,
    mean: Duration,
    median: Duration,
    mean_tokens_per_sec: f64,
    performance_rating: String,
    memory_info: Option<MemoryInfo>,
}

#[derive(Debug)]
struct MemoryInfo {
    used_gb: f64,
    total_gb: f64,
}

fn calculate_mean(durations: &[Duration]) -> Duration {
    let total_nanos: u128 = durations.iter().map(|d| d.as_nanos()).sum();
    Duration::from_nanos((total_nanos / durations.len() as u128) as u64)
}

fn estimate_token_count(text: &str) -> u32 {
    // Rough estimation: ~4 characters per token for English text
    (text.len() as f32 / 4.0).ceil() as u32
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
}
