//! Resilience Command v2 Example
//!
//! Demonstrates resilience patterns and error recovery management.
//!
//! Run with: cargo run --example resilience_v2_example

use anyhow::Result;
use inferno::cli::resilience_v2::{
    BulkheadList, CircuitBreakerList, CircuitBreakerReset, ResilienceStatus, ResilienceTest,
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

    println!("🔥 Inferno Resilience Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Show resilience status
    // ========================================================================
    println!("Example 1: Show Resilience Status");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let status = ResilienceStatus::new(config.clone());");
    println!();
    println!("Output:");
    println!("  === Resilience System Status ===");
    println!("  Overall Health: Healthy");
    println!("  Circuit Breakers:");
    println!("    api-gateway: Closed");
    println!("    model-inference: Closed");
    println!("  Bulkheads:");
    println!("    inference-pool: 3 active");
    println!();
    println!("  Retry Statistics:");
    println!("    Total Attempts: 150");
    println!("    Successful: 145");
    println!("    Failed: 5");

    println!("\n");

    // ========================================================================
    // Example 2: List circuit breakers
    // ========================================================================
    println!("Example 2: List Circuit Breakers");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = CircuitBreakerList::new(config.clone());");
    println!();
    println!("Output:");
    println!("  === Circuit Breakers ===");
    println!("    api-gateway - State: Closed");
    println!("      Failures: 0");
    println!("      Threshold: 5");
    println!("    model-inference - State: Closed");
    println!("      Failures: 2");
    println!("      Threshold: 10");

    println!("\n");

    // ========================================================================
    // Example 3: Reset circuit breaker
    // ========================================================================
    println!("Example 3: Reset Circuit Breaker");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let reset = CircuitBreakerReset::new(");
    println!("      config.clone(),");
    println!("      \"api-gateway\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ Circuit breaker 'api-gateway' reset successfully");

    println!("\n");

    // ========================================================================
    // Example 4: List bulkheads
    // ========================================================================
    println!("Example 4: List Bulkheads");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = BulkheadList::new(config.clone());");
    println!();
    println!("Output:");
    println!("  === Bulkheads ===");
    println!("    inference-pool - Capacity: 3/10");
    println!("    batch-processor - Capacity: 1/5");
    println!("    cache-loader - Capacity: 0/20");

    println!("\n");

    // ========================================================================
    // Example 5: Test retry pattern
    // ========================================================================
    println!("Example 5: Test Retry Pattern");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let test = ResilienceTest::new(");
    println!("      config.clone(),");
    println!("      \"retry\".to_string(),");
    println!("      100,    // requests");
    println!("      0.1,    // 10% failure rate");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ Resilience test completed");
    println!("    Pattern: retry");
    println!("    Requests: 100");
    println!("    Success Rate: 95.00%");
    println!("    Avg Response Time: 150.25ms");
    println!("    Circuit Trips: 0");
    println!("    Retries: 15");

    println!("\n");

    // ========================================================================
    // Example 6: Test circuit breaker pattern
    // ========================================================================
    println!("Example 6: Test Circuit Breaker Pattern");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let test = ResilienceTest::new(");
    println!("      config.clone(),");
    println!("      \"circuit-breaker\".to_string(),");
    println!("      200,    // requests");
    println!("      0.3,    // 30% failure rate");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ Resilience test completed");
    println!("    Pattern: circuit-breaker");
    println!("    Requests: 200");
    println!("    Success Rate: 80.00%");
    println!("    Avg Response Time: 125.50ms");
    println!("    Circuit Trips: 2");
    println!("    Retries: 0");

    println!("\n");

    // ========================================================================
    // Example 7: Test bulkhead pattern
    // ========================================================================
    println!("Example 7: Test Bulkhead Pattern");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let test = ResilienceTest::new(");
    println!("      config.clone(),");
    println!("      \"bulkhead\".to_string(),");
    println!("      500,    // requests");
    println!("      0.05,   // 5% failure rate");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ Resilience test completed");
    println!("    Pattern: bulkhead");
    println!("    Requests: 500");
    println!("    Success Rate: 98.00%");
    println!("    Avg Response Time: 90.75ms");
    println!("    Circuit Trips: 0");
    println!("    Retries: 10");

    println!("\n");

    // ========================================================================
    // Example 8: Validation tests
    // ========================================================================
    println!("Example 8: Input Validation");
    println!("{}", "─".repeat(80));

    let empty_name = CircuitBreakerReset::new(config.clone(), String::new());
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(empty_name), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught empty circuit breaker name:");
            println!("  {}", e);
        }
    }

    println!();

    let zero_requests = ResilienceTest::new(config.clone(), "retry".to_string(), 0, 0.1);

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

    let excessive_requests = ResilienceTest::new(config.clone(), "retry".to_string(), 20000, 0.1);

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

    let invalid_failure_rate = ResilienceTest::new(config.clone(), "retry".to_string(), 100, 1.5);

    match pipeline
        .execute(Box::new(invalid_failure_rate), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught invalid failure rate:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Resilience Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Overall resilience system status");
    println!("✓ Circuit breaker management (list, reset)");
    println!("✓ Bulkhead management (list, show capacity)");
    println!("✓ Pattern testing (retry, circuit breaker, bulkhead)");
    println!("✓ Health monitoring and metrics");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Circuit breaker name not empty");
    println!("  - Request count: 1-10000");
    println!("  - Failure rate: 0.0-1.0");
    println!();
    println!("Use Cases:");
    println!("  - Production error recovery");
    println!("  - Fault tolerance testing");
    println!("  - System health monitoring");
    println!("  - Capacity management");

    Ok(())
}
