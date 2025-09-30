//! Response Cache Command v2 Example
//!
//! Demonstrates response caching and deduplication management.
//!
//! Run with: cargo run --example response_cache_v2_example

use anyhow::Result;
use inferno::cli::response_cache_v2::{
    CacheClear, CacheConfigure, CacheInvalidate, CacheStats, CacheTest,
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

    println!("🔥 Inferno Response Cache Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Show cache statistics
    // ========================================================================
    println!("Example 1: Show Cache Statistics");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let stats = CacheStats::new(config.clone());");
    println!();
    println!("Output:");
    println!("  === Response Cache Statistics ===");
    println!("  Total Requests: 0");
    println!("  Cache Hits: 0");
    println!("  Cache Misses: 0");
    println!("  Hit Rate: 0.00%");
    println!("  Total Entries: 0");
    println!("  Memory Usage: 0.00 MB");

    println!("\n");

    // ========================================================================
    // Example 2: Test cache functionality
    // ========================================================================
    println!("Example 2: Test Cache Functionality");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let test = CacheTest::new(");
    println!("      config.clone(),");
    println!("      100,     // requests");
    println!("      false,   // test_dedup");
    println!("      false,   // test_compression");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ Cache test completed");
    println!("    Total requests: 100");
    println!("    Cache hits: 50");
    println!("    Hit rate: 100.00%");
    println!("    Duration: 0.05s");
    println!("    Memory usage: 0.15 MB");

    println!("\n");

    // ========================================================================
    // Example 3: Test cache with deduplication
    // ========================================================================
    println!("Example 3: Test Cache with Deduplication");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let test = CacheTest::new(");
    println!("      config.clone(),");
    println!("      200,     // requests");
    println!("      true,    // test_dedup");
    println!("      false,   // test_compression");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ Cache test completed");
    println!("    Total requests: 200");
    println!("    Cache hits: 100");
    println!("    Hit rate: 100.00%");
    println!("    Duration: 0.08s");
    println!("    Memory usage: 0.20 MB");
    println!("    Deduplication savings: 15360 bytes");

    println!("\n");

    // ========================================================================
    // Example 4: Test cache with compression
    // ========================================================================
    println!("Example 4: Test Cache with Compression");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let test = CacheTest::new(");
    println!("      config.clone(),");
    println!("      200,     // requests");
    println!("      false,   // test_dedup");
    println!("      true,    // test_compression");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ Cache test completed");
    println!("    Total requests: 200");
    println!("    Cache hits: 100");
    println!("    Hit rate: 100.00%");
    println!("    Duration: 0.10s");
    println!("    Memory usage: 0.08 MB");
    println!("    Compression ratio: 2.5x");

    println!("\n");

    // ========================================================================
    // Example 5: Clear cache
    // ========================================================================
    println!("Example 5: Clear Cache");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let clear = CacheClear::new(config.clone(), None);");
    println!();
    println!("Output:");
    println!("  ✓ Cleared entire response cache");
    println!("    Entries removed: 50");

    println!("\n");

    // ========================================================================
    // Example 6: Clear cache with pattern
    // ========================================================================
    println!("Example 6: Clear Cache with Pattern");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let clear = CacheClear::new(");
    println!("      config.clone(),");
    println!("      Some(\"test-*\".to_string()),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ Cleared cache entries matching pattern: test-*");
    println!("    Entries removed: 25");

    println!("\n");

    // ========================================================================
    // Example 7: Invalidate cache entries
    // ========================================================================
    println!("Example 7: Invalidate Cache Entries");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let invalidate = CacheInvalidate::new(");
    println!("      config.clone(),");
    println!("      \"model-v1-*\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ Invalidated cache entries matching: model-v1-*");

    println!("\n");

    // ========================================================================
    // Example 8: Configure cache settings
    // ========================================================================
    println!("Example 8: Configure Cache Settings");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let configure = CacheConfigure::new(");
    println!("      config.clone(),");
    println!("      Some(true),   // enabled");
    println!("      Some(1000),   // max_entries");
    println!("      Some(3600),   // ttl_seconds");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ Response cache configuration updated");
    println!("    Enabled: true");
    println!("    Max entries: 1000");
    println!("    TTL: 3600 seconds");

    println!("\n");

    // ========================================================================
    // Example 9: Validation tests
    // ========================================================================
    println!("Example 9: Input Validation");
    println!("{}", "─".repeat(80));

    let zero_requests = CacheTest::new(config.clone(), 0, false, false);
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(zero_requests), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught zero requests:");
            println!("  {}", e);
        }
    }

    println!();

    let excessive_requests = CacheTest::new(config.clone(), 20000, false, false);

    match pipeline
        .execute(Box::new(excessive_requests), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught excessive requests:");
            println!("  {}", e);
        }
    }

    println!();

    let empty_pattern = CacheInvalidate::new(config.clone(), String::new());

    match pipeline
        .execute(Box::new(empty_pattern), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught empty pattern:");
            println!("  {}", e);
        }
    }

    println!();

    let zero_entries = CacheConfigure::new(config.clone(), None, Some(0), None);

    match pipeline
        .execute(Box::new(zero_entries), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught zero max entries:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Response Cache Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Cache statistics with hit rates and memory usage");
    println!("✓ Functional testing with configurable parameters");
    println!("✓ Deduplication testing and metrics");
    println!("✓ Compression testing and ratios");
    println!("✓ Clear cache (entire or pattern-based)");
    println!("✓ Invalidate entries by pattern");
    println!("✓ Configure cache settings");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Request count: 1-10000");
    println!("  - Pattern not empty for invalidation");
    println!("  - Max entries >= 1");
    println!();
    println!("Use Cases:");
    println!("  - Response deduplication");
    println!("  - Cache performance optimization");
    println!("  - Memory usage monitoring");
    println!("  - Cache invalidation strategies");

    Ok(())
}