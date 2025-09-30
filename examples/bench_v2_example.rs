//! Bench Command v2 Example
//!
//! Demonstrates the new CLI architecture for the bench command.
//! Shows performance benchmarking with warmup, statistics, and structured output.
//!
//! Run with: cargo run --example bench_v2_example

use anyhow::Result;
use inferno::cli::bench_v2::BenchCommand;
use inferno::config::Config;
use inferno::core::config::ConfigBuilder;
use inferno::interfaces::cli::{CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Bench Command v2 Examples\n");

    // Create configuration
    let config = Config::default();
    let _core_config = ConfigBuilder::new().build_unchecked();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Basic benchmark (requires a real model)
    // ========================================================================
    println!("Example 1: Basic Benchmark");
    println!("{}", "─".repeat(80));
    println!("Note: This example requires a real model file to run.");
    println!("Usage example:");
    println!("  let bench_cmd = BenchCommand::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b.gguf\".to_string(),  // model name");
    println!("      10,                              // iterations");
    println!("      Some(\"Hello, world!\".to_string()),  // prompt");
    println!("      100,                             // max tokens");
    println!("      3,                               // warmup iterations");
    println!("      None,                            // auto-detect backend");
    println!("  );");
    println!();
    println!("When executed, this would:");
    println!("  1. Load the model");
    println!("  2. Run 3 warmup iterations");
    println!("  3. Run 10 benchmark iterations");
    println!("  4. Calculate statistics (min, max, mean, median)");
    println!("  5. Report tokens/second and performance rating");

    println!("\n");

    // ========================================================================
    // Example 2: Validation demonstration
    // ========================================================================
    println!("Example 2: Input Validation");
    println!("{}", "─".repeat(80));

    // Test with invalid parameters
    let invalid_bench = BenchCommand::new(
        config.clone(),
        "".to_string(), // Empty model name - should fail
        0,              // Zero iterations - should fail
        None,
        10000,
        3,
        None,
    );

    let mut ctx_invalid = CommandContext::new(config.clone());

    match pipeline.execute(Box::new(invalid_bench), &mut ctx_invalid).await {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation correctly caught errors:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 3: Validation - too many iterations
    // ========================================================================
    println!("Example 3: Iteration Limit Validation");
    println!("{}", "─".repeat(80));

    let too_many_iters = BenchCommand::new(
        config.clone(),
        "test-model".to_string(),
        2000, // Over 1000 limit - should fail
        None,
        100,
        3,
        None,
    );

    let mut ctx_iters = CommandContext::new(config.clone());

    match pipeline.execute(Box::new(too_many_iters), &mut ctx_iters).await {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught iteration limit:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 4: JSON output mode
    // ========================================================================
    println!("Example 4: JSON Output Mode");
    println!("{}", "─".repeat(80));
    println!("When run with json_output = true, the command returns structured data:");
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "model": "llama-2-7b.gguf",
        "backend": "Gguf",
        "load_time_ms": 1234,
        "config": {
            "iterations": 10,
            "warmup": 3,
            "max_tokens": 100,
            "prompt_length": 27
        },
        "warmup": {
            "iterations": 3,
            "total_time_ms": 856,
            "mean_ms": 285
        },
        "benchmark": {
            "iterations": 10,
            "total_time_ms": 2847,
            "total_tokens": 980,
            "throughput_tokens_per_sec": 344.3,
            "statistics": {
                "min_ms": 245,
                "max_ms": 312,
                "mean_ms": 284,
                "median_ms": 281,
                "min_tokens_per_sec": 320.5,
                "max_tokens_per_sec": 408.2,
                "mean_tokens_per_sec": 352.1,
                "median_tokens_per_sec": 355.9
            },
            "performance_rating": "Excellent (>100 tok/s)"
        },
        "memory": {
            "used_gb": 4.2,
            "total_gb": 16.0
        }
    }))?);

    println!("\n");

    // ========================================================================
    // Example 5: Verbose output mode
    // ========================================================================
    println!("Example 5: Verbose Output Mode");
    println!("{}", "─".repeat(80));
    println!("When run with verbosity = 1, shows per-iteration details:");
    println!();
    println!("Warming up (3 iterations)...");
    println!("  Warmup 1: 287ms");
    println!("  Warmup 2: 283ms");
    println!("  Warmup 3: 285ms");
    println!("Warmup completed.");
    println!();
    println!("Running benchmark...");
    println!("  Iteration 1: 284ms (98 tokens, 345.1 tok/s)");
    println!("  Iteration 2: 281ms (97 tokens, 345.2 tok/s)");
    println!("  Iteration 3: 289ms (99 tokens, 342.6 tok/s)");
    println!("  ...");

    println!("\n");

    // ========================================================================
    // Example 6: Warmup behavior
    // ========================================================================
    println!("Example 6: Warmup Iterations");
    println!("{}", "─".repeat(80));
    println!("Warmup iterations help stabilize performance before benchmarking:");
    println!("  - First iteration: often slower (model loading, cache warming)");
    println!("  - Warmup iterations: prepare the system");
    println!("  - Benchmark iterations: measure stable performance");
    println!();
    println!("Example with warmup=0 (no warmup):");
    println!("  let no_warmup = BenchCommand::new(");
    println!("      config, \"model\".to_string(), 10, None, 100, 0, None");
    println!("  );");
    println!();
    println!("Example with warmup=5 (5 warmup iterations):");
    println!("  let with_warmup = BenchCommand::new(");
    println!("      config, \"model\".to_string(), 10, None, 100, 5, None");
    println!("  );");

    println!("\n");

    // ========================================================================
    // Example 7: Custom prompts
    // ========================================================================
    println!("Example 7: Custom Prompts");
    println!("{}", "─".repeat(80));
    println!("You can provide custom prompts for benchmarking:");
    println!();
    println!("Short prompt (fast iteration):");
    println!("  BenchCommand::new(");
    println!("      config, \"model\".to_string(), 20,");
    println!("      Some(\"Hello\".to_string()),  // short prompt");
    println!("      50, 3, None");
    println!("  );");
    println!();
    println!("Long prompt (slower iteration, more realistic):");
    println!("  BenchCommand::new(");
    println!("      config, \"model\".to_string(), 10,");
    println!("      Some(\"Explain quantum computing...\".to_string()),");
    println!("      200, 3, None");
    println!("  );");
    println!();
    println!("Default prompt (if None provided):");
    println!("  \"The quick brown fox jumps over the lazy dog.\"");

    println!("\n");

    // ========================================================================
    // Example 8: Backend selection
    // ========================================================================
    println!("Example 8: Backend Selection");
    println!("{}", "─".repeat(80));
    println!("Backends are auto-detected from model file extension:");
    println!("  - .gguf files → GGUF backend");
    println!("  - .onnx files → ONNX backend");
    println!();
    println!("Or explicitly specify backend:");
    println!("  use inferno::backends::BackendType;");
    println!();
    println!("  BenchCommand::new(");
    println!("      config, \"model.gguf\".to_string(), 10, None, 100, 3,");
    println!("      Some(BackendType::Gguf)  // explicit backend");
    println!("  );");

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Bench Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Configurable benchmark iterations");
    println!("✓ Warmup iterations for stable measurements");
    println!("✓ Custom prompts for realistic testing");
    println!("✓ Backend auto-detection or manual selection");
    println!("✓ Statistical analysis (min, max, mean, median)");
    println!("✓ Tokens per second throughput calculation");
    println!("✓ Performance rating classification");
    println!("✓ Memory usage estimation");
    println!("✓ Structured JSON output");
    println!("✓ Verbose per-iteration reporting");
    println!("✓ Comprehensive validation (iterations, tokens, warmup)");
    println!("✓ Middleware support (logging, metrics)");
    println!();
    println!("Use Cases:");
    println!("  - Compare model performance across backends");
    println!("  - Measure optimization impact");
    println!("  - Establish performance baselines");
    println!("  - Monitor performance regression");
    println!("  - Hardware comparison and selection");

    Ok(())
}