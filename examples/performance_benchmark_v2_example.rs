//! Performance Benchmark Command Examples - New Architecture

use anyhow::Result;
use inferno::{
    cli::performance_benchmark_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Performance Benchmark Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: Run CPU benchmark
    println!("Example 1: Run CPU benchmark");
    let cmd = BenchmarkRun::new(config.clone(), "cpu".to_string(), 100, true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: Run memory benchmark
    println!("Example 2: Run memory benchmark");
    let cmd = BenchmarkRun::new(config.clone(), "memory".to_string(), 50, false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: Run all benchmarks
    println!("Example 3: Run all benchmarks");
    let cmd = BenchmarkRun::new(config.clone(), "all".to_string(), 100, true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: Compare results
    println!("Example 4: Compare results");
    let cmd = BenchmarkCompare::new(
        config.clone(),
        "baseline.json".to_string(),
        "current.json".to_string(),
        Some(5.0),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: Establish baseline
    println!("Example 5: Establish baseline");
    let cmd = BenchmarkBaseline::new(
        config.clone(),
        "baseline.json".to_string(),
        vec!["gguf".to_string(), "onnx".to_string()],
        60,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: Generate HTML report
    println!("Example 6: Generate HTML report");
    let cmd = BenchmarkReport::new(
        config.clone(),
        "results.json".to_string(),
        "html".to_string(),
        Some("reports/benchmark.html".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 7: Generate JSON report
    println!("Example 7: Generate JSON report");
    let cmd = BenchmarkReport::new(
        config.clone(),
        "results.json".to_string(),
        "json".to_string(),
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 8: Profile performance
    println!("Example 8: Profile performance");
    let cmd = BenchmarkProfile::new(config.clone(), "inference_engine".to_string(), 30, true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    println!("=== All examples completed successfully ===");
    Ok(())
}
