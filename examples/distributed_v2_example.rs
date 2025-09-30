//! Distributed Command v2 Example
//!
//! Demonstrates distributed inference and worker pool management.
//!
//! Run with: cargo run --example distributed_v2_example

use anyhow::Result;
use inferno::cli::distributed_v2::{
    DistributedBenchmark, DistributedStart, DistributedStats, DistributedTest,
};
use inferno::config::Config;
use inferno::interfaces::cli::{
    CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Distributed Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Start distributed server
    // ========================================================================
    println!("Example 1: Start Distributed Server");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let start = DistributedStart::new(");
    println!("      config.clone(),");
    println!("      4,        // workers");
    println!("      None,     // no preload model");
    println!("      true,     // load balancing enabled");
    println!("      8,        // max concurrent per worker");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Distributed Inference Server Started ===");
    println!("  Workers: 4");
    println!("  Max Concurrent per Worker: 8");
    println!("  Load Balancing: true");
    println!("  ");
    println!("  Server is running. Press Ctrl+C to stop.");

    println!("\n");

    // ========================================================================
    // Example 2: Start with preloaded model
    // ========================================================================
    println!("Example 2: Start with Preloaded Model");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let start = DistributedStart::new(");
    println!("      config.clone(),");
    println!("      8,        // workers");
    println!("      Some(\"llama-2-7b\".to_string()),");
    println!("      true,     // load balancing enabled");
    println!("      16,       // max concurrent per worker");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Distributed Inference Server Started ===");
    println!("  Workers: 8");
    println!("  Max Concurrent per Worker: 16");
    println!("  Load Balancing: true");
    println!("  Preloading Model: llama-2-7b");

    println!("\n");

    // ========================================================================
    // Example 3: Run benchmark
    // ========================================================================
    println!("Example 3: Run Distributed Benchmark");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let benchmark = DistributedBenchmark::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      10,       // concurrent clients");
    println!("      5,        // requests per client");
    println!("      \"Hello, world!\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Benchmark Results ===");
    println!("  Model: llama-2-7b");
    println!("  Total Duration: 12.345s");
    println!("  Total Requests: 50");
    println!("  Successful: 48");
    println!("  Failed: 2");
    println!("  Success Rate: 96.00%");
    println!("  Throughput: 3.89 req/s");
    println!("  Total Tokens: 2400");

    println!("\n");

    // ========================================================================
    // Example 4: Show worker statistics
    // ========================================================================
    println!("Example 4: Show Worker Statistics");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let stats = DistributedStats::new(config.clone());");
    println!();
    println!("Output:");
    println!("  === Distributed Inference Statistics ===");
    println!("  Total Workers: 4");
    println!("  Active Workers: 4");
    println!("  Total Requests: 150");
    println!("  Successful Requests: 147");
    println!("  Failed Requests: 3");
    println!("  Success Rate: 98.00%");
    println!("  Average Response Time: 234ms");

    println!("\n");

    // ========================================================================
    // Example 5: Test single inference
    // ========================================================================
    println!("Example 5: Test Single Inference");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let test = DistributedTest::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      \"Hello, world!\".to_string(),");
    println!("      false,    // no streaming");
    println!("      100,      // max tokens");
    println!("      0.7,      // temperature");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Inference Test Results ===");
    println!("  Model: llama-2-7b");
    println!("  Input: Hello, world!");
    println!("  Response: Hello! How can I help you today?");
    println!("  Tokens Generated: 8");
    println!("  Duration: 234ms");

    println!("\n");

    // ========================================================================
    // Example 6: Test with streaming
    // ========================================================================
    println!("Example 6: Test with Streaming");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let test = DistributedTest::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      \"Explain quantum computing\".to_string(),");
    println!("      true,     // streaming enabled");
    println!("      200,      // max tokens");
    println!("      0.8,      // temperature");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Inference Test Results ===");
    println!("  Model: llama-2-7b");
    println!("  Input: Explain quantum computing");
    println!("  Response: Quantum computing is...");
    println!("  Tokens Generated: 50");
    println!("  Duration: 567ms");

    println!("\n");

    // ========================================================================
    // Example 7: High concurrency benchmark
    // ========================================================================
    println!("Example 7: High Concurrency Benchmark");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let benchmark = DistributedBenchmark::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      50,       // high concurrent clients");
    println!("      10,       // requests per client");
    println!("      \"Test\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Benchmark Results ===");
    println!("  Model: llama-2-7b");
    println!("  Total Duration: 45.678s");
    println!("  Total Requests: 500");
    println!("  Successful: 490");
    println!("  Failed: 10");
    println!("  Success Rate: 98.00%");
    println!("  Throughput: 10.73 req/s");
    println!("  Total Tokens: 24500");

    println!("\n");

    // ========================================================================
    // Example 8: Validation tests
    // ========================================================================
    println!("Example 8: Input Validation");
    println!("{}", "─".repeat(80));

    let too_many_workers = DistributedStart::new(config.clone(), 100, None, true, 8);
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(too_many_workers), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught too many workers:");
            println!("  {}", e);
        }
    }

    println!();

    let empty_model = DistributedBenchmark::new(
        config.clone(),
        String::new(),
        10,
        5,
        "Hello".to_string(),
    );

    match pipeline
        .execute(Box::new(empty_model), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught empty model name:");
            println!("  {}", e);
        }
    }

    println!();

    let invalid_temperature = DistributedTest::new(
        config.clone(),
        "test-model".to_string(),
        "Hello".to_string(),
        false,
        100,
        3.0,
    );

    match pipeline
        .execute(Box::new(invalid_temperature), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught invalid temperature:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Distributed Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Start distributed inference server");
    println!("✓ Configure worker pools");
    println!("✓ Enable load balancing");
    println!("✓ Preload models for faster response");
    println!("✓ Run performance benchmarks");
    println!("✓ Test single inference requests");
    println!("✓ Stream inference output");
    println!("✓ Monitor worker statistics");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Worker count <= 32");
    println!("  - Max concurrent > 0 and <= 100");
    println!("  - Model name not empty");
    println!("  - Prompt not empty");
    println!("  - Temperature between 0.0 and 2.0");
    println!();
    println!("Use Cases:");
    println!("  - High-throughput inference");
    println!("  - Load distribution across workers");
    println!("  - Performance testing");
    println!("  - Production scalability");

    Ok(())
}