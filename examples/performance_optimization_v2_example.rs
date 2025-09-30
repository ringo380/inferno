//! Performance Optimization Command Examples - New Architecture

use anyhow::Result;
use inferno::{
    cli::performance_optimization_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Performance Optimization Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: Analyze performance
    println!("Example 1: Analyze performance");
    let cmd = OptimizeAnalyze::new(
        config.clone(),
        "inference_engine".to_string(),
        60,
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: Auto-tune with grid search
    println!("Example 2: Auto-tune with grid search");
    let cmd = OptimizeTune::new(
        config.clone(),
        "batch_processing".to_string(),
        100,
        "grid".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: Auto-tune with bayesian
    println!("Example 3: Auto-tune with bayesian");
    let cmd = OptimizeTune::new(
        config.clone(),
        "cache_system".to_string(),
        50,
        "bayesian".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: Apply optimizations (dry run)
    println!("Example 4: Apply optimizations (dry run)");
    let cmd = OptimizeApply::new(
        config.clone(),
        "api_server".to_string(),
        "recommendations.json".to_string(),
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: Apply optimizations (live)
    println!("Example 5: Apply optimizations (live)");
    let cmd = OptimizeApply::new(
        config.clone(),
        "api_server".to_string(),
        "recommendations.json".to_string(),
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: Monitor performance
    println!("Example 6: Monitor performance");
    let cmd = OptimizeMonitor::new(
        config.clone(),
        "worker_pool".to_string(),
        5,
        Some(80.0),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 7: Generate HTML report
    println!("Example 7: Generate HTML report");
    let cmd = OptimizeReport::new(
        config.clone(),
        "system".to_string(),
        "7d".to_string(),
        "html".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 8: Validate optimizations
    println!("Example 8: Validate optimizations");
    let cmd = OptimizeValidate::new(
        config.clone(),
        "cache".to_string(),
        "baseline.json".to_string(),
        10.0,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    println!("=== All examples completed successfully ===");
    Ok(())
}