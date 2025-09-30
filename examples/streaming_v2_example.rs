//! Streaming Command v2 Example
//!
//! Demonstrates the new CLI architecture for streaming operations.
//! Shows benchmark testing and configuration export.
//!
//! Run with: cargo run --example streaming_v2_example

use anyhow::Result;
use inferno::cli::streaming_v2::{ConfigFormat, StreamingBenchmark, StreamingConfigExport};
use inferno::config::Config;
use inferno::core::config::ConfigBuilder;
use inferno::interfaces::cli::{
    CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Streaming Command v2 Examples\n");

    // Create configuration
    let config = Config::default();
    let _core_config = ConfigBuilder::new().build_unchecked();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Basic streaming benchmark
    // ========================================================================
    println!("Example 1: Basic Streaming Benchmark");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let bench = StreamingBenchmark::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b-chat.gguf\".to_string(),");
    println!("      5,      // concurrent streams");
    println!("      \"What is the capital of France?\".to_string(),");
    println!("      30,     // duration in seconds");
    println!("  );");
    println!();
    println!("Output:");
    println!("  🚀 Starting streaming benchmark");
    println!("  Model: llama-2-7b-chat.gguf");
    println!("  Concurrent streams: 5");
    println!("  Duration: 30s");
    println!("  📊 Active: 5, Total tokens: 1234, Avg tok/s: 41.1");
    println!("  🏁 Benchmark Results:");
    println!("  Total streams created: 150");
    println!("  Total tokens generated: 12450");
    println!("  Average tokens/second: 415.0");

    println!("\n");

    // ========================================================================
    // Example 2: High concurrency benchmark
    // ========================================================================
    println!("Example 2: High Concurrency Benchmark");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let high_concurrency = StreamingBenchmark::new(");
    println!("      config.clone(),");
    println!("      \"mistral-7b-instruct.gguf\".to_string(),");
    println!("      20,     // 20 concurrent streams");
    println!("      \"Write a short story\".to_string(),");
    println!("      60,     // 1 minute test");
    println!("  );");
    println!();
    println!("Benefits:");
    println!("  - Tests system limits");
    println!("  - Identifies bottlenecks");
    println!("  - Measures throughput");
    println!("  - Validates stability");

    println!("\n");

    // ========================================================================
    // Example 3: Export JSON config
    // ========================================================================
    println!("Example 3: Export JSON Configuration");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let export_json = StreamingConfigExport::new(");
    println!("      ConfigFormat::Json,");
    println!("      Some(PathBuf::from(\"streaming-config.json\")),");
    println!("  );");
    println!();
    println!("Output file content:");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "max_concurrent_streams": 50,
            "enable_metrics": true,
            "heartbeat_interval_ms": 30000,
            "buffer_size": 1024,
            "timeout_ms": 300000
        }))?
    );

    println!("\n");

    // ========================================================================
    // Example 4: Export to stdout
    // ========================================================================
    println!("Example 4: Export to Stdout");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let stdout_export = StreamingConfigExport::new(");
    println!("      ConfigFormat::Yaml,");
    println!("      None,  // stdout");
    println!("  );");
    println!();
    println!("Useful for:");
    println!("  - Piping to other tools");
    println!("  - Quick config inspection");
    println!("  - Template generation");

    println!("\n");

    // ========================================================================
    // Example 5: Validation tests
    // ========================================================================
    println!("Example 5: Input Validation");
    println!("{}", "─".repeat(80));

    let invalid_concurrent = StreamingBenchmark::new(
        config.clone(),
        "model.gguf".to_string(),
        200,
        "test".to_string(),
        30,
    );
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(invalid_concurrent), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught excessive concurrency:");
            println!("  {}", e);
        }
    }

    println!();

    let excessive_duration = StreamingBenchmark::new(
        config.clone(),
        "model.gguf".to_string(),
        5,
        "test".to_string(),
        5000,
    );

    match pipeline
        .execute(Box::new(excessive_duration), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught excessive duration:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Streaming Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Streaming performance benchmarking");
    println!("✓ Concurrent stream testing (1-100 streams)");
    println!("✓ Duration-based testing");
    println!("✓ Real-time metrics monitoring");
    println!("✓ Configuration export (JSON/YAML/TOML)");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Model name validation");
    println!("  - Prompt not empty");
    println!("  - Concurrency limits (1-100)");
    println!("  - Duration limits (1-3600s)");
    println!("  - Output directory validation");
    println!();
    println!("Note: Interactive and server modes remain available");
    println!("through the original streaming module.");

    Ok(())
}