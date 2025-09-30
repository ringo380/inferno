//! Advanced Cache Command Examples - New Architecture
//!
//! This example demonstrates the usage of advanced cache commands in the v2 architecture.

use anyhow::Result;
use inferno::{
    cli::advanced_cache_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Advanced Cache Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: Basic cache statistics
    println!("Example 1: Basic cache statistics");
    let cmd = CacheStats::new(config.clone(), false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: Detailed cache statistics
    println!("Example 2: Detailed cache statistics");
    let cmd = CacheStats::new(config.clone(), true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: Warmup with popular strategy
    println!("Example 3: Warmup with popular strategy");
    let cmd = CacheWarmup::new(config.clone(), "popular".to_string(), None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: Warmup with recent strategy and pattern
    println!("Example 4: Warmup with recent strategy and pattern");
    let cmd = CacheWarmup::new(
        config.clone(),
        "recent".to_string(),
        Some("model-*".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: Warmup with predicted strategy
    println!("Example 5: Warmup with predicted strategy");
    let cmd = CacheWarmup::new(config.clone(), "predicted".to_string(), None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: Evict with LRU policy
    println!("Example 6: Evict with LRU policy");
    let cmd = CacheEvict::new(config.clone(), "lru".to_string(), None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 7: Evict with size policy and target
    println!("Example 7: Evict with size policy and target");
    let cmd = CacheEvict::new(config.clone(), "size".to_string(), Some(1_073_741_824));
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 8: Evict with LFU policy
    println!("Example 8: Evict with LFU policy");
    let cmd = CacheEvict::new(config.clone(), "lfu".to_string(), None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 9: Get cache configuration
    println!("Example 9: Get cache configuration");
    let cmd = CacheConfig::new(config.clone(), "get".to_string(), None, None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 10: Set max size
    println!("Example 10: Set max size");
    let cmd = CacheConfig::new(
        config.clone(),
        "set".to_string(),
        Some(4_294_967_296),
        None,
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 11: Set TTL
    println!("Example 11: Set TTL");
    let cmd = CacheConfig::new(config.clone(), "set".to_string(), None, Some(7200), None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 12: Enable compression
    println!("Example 12: Enable compression");
    let cmd = CacheConfig::new(config.clone(), "set".to_string(), None, None, Some(true));
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 13: Analyze 1 hour
    println!("Example 13: Analyze 1 hour");
    let cmd = CacheAnalyze::new(config.clone(), "1h".to_string());
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 14: Analyze 24 hours
    println!("Example 14: Analyze 24 hours");
    let cmd = CacheAnalyze::new(config.clone(), "24h".to_string());
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 15: Analyze 7 days
    println!("Example 15: Analyze 7 days");
    let cmd = CacheAnalyze::new(config.clone(), "7d".to_string());
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 16: Clean expired entries
    println!("Example 16: Clean expired entries");
    let cmd = CacheClean::new(config.clone(), "expired".to_string());
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 17: Clean unused entries
    println!("Example 17: Clean unused entries");
    let cmd = CacheClean::new(config.clone(), "unused".to_string());
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 18: Clean all entries
    println!("Example 18: Clean all entries");
    let cmd = CacheClean::new(config.clone(), "all".to_string());
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 19: Compact cache
    println!("Example 19: Compact cache");
    let cmd = CacheClean::new(config.clone(), "compact".to_string());
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 20: Validation - Invalid strategy
    println!("Example 20: Validation - Invalid strategy");
    let cmd = CacheWarmup::new(config.clone(), "invalid".to_string(), None);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 21: Validation - Size policy without target
    println!("Example 21: Validation - Size policy without target");
    let cmd = CacheEvict::new(config.clone(), "size".to_string(), None);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 22: Validation - Set without parameters
    println!("Example 22: Validation - Set without parameters");
    let cmd = CacheConfig::new(config.clone(), "set".to_string(), None, None, None);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    println!("=== All examples completed successfully ===");
    Ok(())
}