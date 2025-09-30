//! Cache Command v2 Example
//!
//! Demonstrates the new CLI architecture for the cache command.
//! Shows cache statistics, warmup, clearing, and configuration operations.
//!
//! Run with: cargo run --example cache_v2_example

use anyhow::Result;
use inferno::cache::WarmupStrategy;
use inferno::cli::cache_v2::{CacheClear, CacheConfigure, CacheStats, CacheWarmup};
use inferno::config::Config;
use inferno::core::config::ConfigBuilder;
use inferno::interfaces::cli::{
    CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Cache Command v2 Examples\n");

    // Create configuration
    let config = Config::default();
    let _core_config = ConfigBuilder::new().build_unchecked();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Show cache statistics
    // ========================================================================
    println!("Example 1: Cache Statistics");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let stats_cmd = CacheStats::new(config.clone());");
    println!();
    println!("Expected output:");
    println!("  === Model Cache Statistics ===");
    println!("  Total Models: 3");
    println!("  Memory Usage: 8456.23 MB");
    println!("  Hit Rate: 87.50%");
    println!("  Miss Rate: 12.50%");
    println!("  Evictions: 5");
    println!("  Warmups: 8");
    println!();
    println!("  Active Models:");
    println!("    - llama-2-7b-chat.Q4_0.gguf");
    println!("    - mistral-7b-instruct.Q5_0.gguf");
    println!("    - codellama-13b.Q4_K_M.gguf");

    println!("\n");

    // ========================================================================
    // Example 2: Warmup specific models
    // ========================================================================
    println!("Example 2: Warm Up Specific Models");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let warmup_cmd = CacheWarmup::new(");
    println!("      config.clone(),");
    println!("      vec![");
    println!("          \"llama-2-7b-chat.gguf\".to_string(),");
    println!("          \"mistral-7b-instruct.gguf\".to_string(),");
    println!("      ],");
    println!("      Some(WarmupStrategy::Priority),");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  Warming up 2 models...");
    println!("  ✓ Warmed up model: llama-2-7b-chat.gguf");
    println!("  ✓ Warmed up model: mistral-7b-instruct.gguf");
    println!("  Warmup completed in 45.2s");
    println!();
    println!("  Final cache status:");
    println!("    Active models: 2");
    println!("    Memory usage: 8456.23 MB");

    println!("\n");

    // ========================================================================
    // Example 3: Automatic warmup
    // ========================================================================
    println!("Example 3: Automatic Warmup");
    println!("{}", "─".repeat(80));
    println!("Usage example (empty models list triggers automatic warmup):");
    println!("  let auto_warmup = CacheWarmup::new(");
    println!("      config.clone(),");
    println!("      vec![],  // Empty - uses automatic warmup");
    println!("      Some(WarmupStrategy::UsageBased),");
    println!("  );");
    println!();
    println!("Warmup strategies:");
    println!("  UsageBased    - Warm up most frequently used models");
    println!("  Predictive    - Use ML to predict which models will be needed");
    println!("  SizeOptimized - Prioritize smaller models for faster warmup");
    println!("  Priority      - Use configured priority list");
    println!("  Hybrid        - Combine multiple strategies");

    println!("\n");

    // ========================================================================
    // Example 4: Clear specific model
    // ========================================================================
    println!("Example 4: Clear Specific Model");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let clear_cmd = CacheClear::new(");
    println!("      config.clone(),");
    println!("      Some(\"old-model.gguf\".to_string()),");
    println!("      false,  // force");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  ✓ Cleared model: old-model.gguf");
    println!("  Remaining models: 2");
    println!("  Memory usage: 5832.10 MB");

    println!("\n");

    // ========================================================================
    // Example 5: Clear all models
    // ========================================================================
    println!("Example 5: Clear All Models");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let clear_all = CacheClear::new(");
    println!("      config.clone(),");
    println!("      None,   // clear all");
    println!("      false,  // force");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  ✓ Cleared all cached models");
    println!("  Remaining models: 0");
    println!("  Memory usage: 0.00 MB");

    println!("\n");

    // ========================================================================
    // Example 6: Always-warm protection
    // ========================================================================
    println!("Example 6: Always-Warm Model Protection");
    println!("{}", "─".repeat(80));

    let mut protected_config = config.clone();
    protected_config.cache.always_warm = vec!["important-model.gguf".to_string()];

    let clear_protected = CacheClear::new(
        protected_config.clone(),
        Some("important-model.gguf".to_string()),
        false, // Not forcing
    );
    let ctx_protected = CommandContext::new(protected_config);

    match pipeline
        .execute(Box::new(clear_protected), &mut ctx_protected.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation correctly protected always-warm model:");
            println!("  {}", e);
        }
    }

    println!();
    println!("To force clear always-warm models:");
    println!("  CacheClear::new(config, Some(\"model\".to_string()), true)");

    println!("\n");

    // ========================================================================
    // Example 7: Configure cache settings
    // ========================================================================
    println!("Example 7: Configure Cache Settings");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let configure_cmd = CacheConfigure::new(");
    println!("      Some(10),         // max_models");
    println!("      Some(16384),      // max_memory_mb (16GB)");
    println!("      Some(3600),       // ttl_seconds (1 hour)");
    println!("      Some(true),       // enable warmup");
    println!("      Some(WarmupStrategy::Hybrid),");
    println!("      Some(vec![\"llama-2-7b.gguf\".to_string()]),");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  === Cache Configuration Update ===");
    println!("  Max models: 10");
    println!("  Max memory: 16384 MB");
    println!("  TTL: 3600 seconds");
    println!("  Warmup enabled: true");
    println!("  Warmup strategy: Hybrid");
    println!("  Always warm: [\"llama-2-7b.gguf\"]");
    println!();
    println!("  Note: Configuration changes require restart to take effect.");
    println!("  Update your config.toml file with these values.");

    println!("\n");

    // ========================================================================
    // Example 8: Validation examples
    // ========================================================================
    println!("Example 8: Input Validation");
    println!("{}", "─".repeat(80));

    // Test zero max_models
    let invalid_config = CacheConfigure::new(Some(0), None, None, None, None, None);
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(invalid_config), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught zero max_models:");
            println!("  {}", e);
        }
    }

    println!();

    // Test excessive max_models
    let excessive_models = CacheConfigure::new(Some(200), None, None, None, None, None);

    match pipeline
        .execute(Box::new(excessive_models), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught excessive max_models:");
            println!("  {}", e);
        }
    }

    println!();

    // Test excessive TTL
    let excessive_ttl = CacheConfigure::new(None, None, Some(50_000_000), None, None, None);

    match pipeline
        .execute(Box::new(excessive_ttl), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught excessive TTL:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 9: JSON output mode
    // ========================================================================
    println!("Example 9: JSON Output Mode");
    println!("{}", "─".repeat(80));
    println!("With json_output=true, structured data is returned:");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "cache_stats": {
                "total_models": 3,
                "memory_usage_mb": 8456.23,
                "hit_rate": 0.875,
                "miss_rate": 0.125,
                "eviction_count": 5,
                "warmup_count": 8,
                "active_models": [
                    "llama-2-7b-chat.Q4_0.gguf",
                    "mistral-7b-instruct.Q5_0.gguf"
                ]
            },
            "cache_config": {
                "max_cached_models": 10,
                "max_memory_mb": 16384,
                "model_ttl_seconds": 3600,
                "enable_warmup": true,
                "warmup_strategy": "Hybrid",
                "always_warm": ["llama-2-7b.gguf"]
            }
        }))?
    );

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Cache Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Show comprehensive cache statistics");
    println!("✓ Display cache configuration settings");
    println!("✓ Warm up specific models");
    println!("✓ Automatic warmup with multiple strategies");
    println!("✓ Clear specific models from cache");
    println!("✓ Clear all cached models");
    println!("✓ Configure cache settings");
    println!("✓ Always-warm model protection");
    println!("✓ Input validation (ranges, limits)");
    println!("✓ Structured JSON output");
    println!("✓ Human-readable progress and results");
    println!("✓ Middleware support (logging, metrics)");
    println!();
    println!("Validation Checks:");
    println!("  - Max models range (1-100)");
    println!("  - Max memory range (1-1,000,000 MB)");
    println!("  - TTL range (1-31,536,000 seconds)");
    println!("  - Always-warm model protection");
    println!();
    println!("Warmup Strategies:");
    println!("  - UsageBased: Most frequently used models");
    println!("  - Predictive: ML-based prediction");
    println!("  - SizeOptimized: Prioritize smaller models");
    println!("  - Priority: Use configured priority list");
    println!("  - Hybrid: Combine multiple strategies");
    println!();
    println!("Use Cases:");
    println!("  - Monitor cache performance and memory usage");
    println!("  - Pre-load frequently used models");
    println!("  - Free memory by clearing unused models");
    println!("  - Configure cache limits and behavior");
    println!("  - Protect critical models from eviction");
    println!();
    println!("Note: This is a focused migration covering core cache operations.");
    println!("Full cache functionality (benchmark, monitor, export) remains");
    println!("available through the original cache module.");

    Ok(())
}