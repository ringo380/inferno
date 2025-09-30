//! API Gateway Command Examples - New Architecture
//!
//! This example demonstrates the usage of API gateway commands in the v2 architecture.

use anyhow::Result;
use inferno::{
    cli::api_gateway_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== API Gateway Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: Start gateway on default port
    println!("Example 1: Start gateway on default port");
    let cmd = ApiGatewayStart::new(
        config.clone(),
        8080,
        "0.0.0.0".to_string(),
        false,
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: Start as daemon with rate limiting disabled
    println!("Example 2: Start as daemon with rate limiting disabled");
    let cmd = ApiGatewayStart::new(
        config.clone(),
        8080,
        "127.0.0.1".to_string(),
        true,
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: Check gateway status
    println!("Example 3: Check gateway status");
    let cmd = ApiGatewayStatus::new(config.clone(), false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: Detailed status
    println!("Example 4: Detailed status");
    let cmd = ApiGatewayStatus::new(config.clone(), true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: List all routes
    println!("Example 5: List all routes");
    let cmd = ApiGatewayRoutes::new(config.clone(), "list".to_string(), None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: Add new route
    println!("Example 6: Add new route");
    let cmd = ApiGatewayRoutes::new(
        config.clone(),
        "add".to_string(),
        Some("/api/v1/models".to_string()),
        Some("http://backend1:8080".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 7: Remove route
    println!("Example 7: Remove route");
    let cmd = ApiGatewayRoutes::new(
        config.clone(),
        "remove".to_string(),
        Some("/api/v1/deprecated".to_string()),
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 8: List rate limit rules
    println!("Example 8: List rate limit rules");
    let cmd = ApiGatewayRateLimit::new(config.clone(), "list".to_string(), None, None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 9: Add rate limit rule
    println!("Example 9: Add rate limit rule");
    let cmd = ApiGatewayRateLimit::new(
        config.clone(),
        "add".to_string(),
        Some("api_limit".to_string()),
        Some(100),
        Some(60),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 10: Enable rate limiting
    println!("Example 10: Enable rate limiting");
    let cmd = ApiGatewayRateLimit::new(config.clone(), "enable".to_string(), None, None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 11: Disable rate limiting
    println!("Example 11: Disable rate limiting");
    let cmd = ApiGatewayRateLimit::new(config.clone(), "disable".to_string(), None, None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 12: List backend services
    println!("Example 12: List backend services");
    let cmd = ApiGatewayServices::new(config.clone(), "list".to_string(), None, None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 13: Add backend service
    println!("Example 13: Add backend service");
    let cmd = ApiGatewayServices::new(
        config.clone(),
        "add".to_string(),
        Some("inference-service".to_string()),
        Some("http://localhost:8081".to_string()),
        Some("/health".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 14: Check service health
    println!("Example 14: Check service health");
    let cmd = ApiGatewayServices::new(
        config.clone(),
        "health".to_string(),
        Some("backend1".to_string()),
        None,
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 15: Show metrics for 1 hour
    println!("Example 15: Show metrics for 1 hour");
    let cmd = ApiGatewayMetrics::new(config.clone(), "1h".to_string(), false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 16: Detailed metrics for 24 hours
    println!("Example 16: Detailed metrics for 24 hours");
    let cmd = ApiGatewayMetrics::new(config.clone(), "24h".to_string(), true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 17: Weekly metrics
    println!("Example 17: Weekly metrics");
    let cmd = ApiGatewayMetrics::new(config.clone(), "7d".to_string(), false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 18: Validation - Zero port
    println!("Example 18: Validation - Zero port");
    let cmd = ApiGatewayStart::new(config.clone(), 0, "0.0.0.0".to_string(), false, false);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 19: Validation - Invalid time range
    println!("Example 19: Validation - Invalid time range");
    let cmd = ApiGatewayMetrics::new(config.clone(), "invalid".to_string(), false);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 20: Validation - Add route without backend
    println!("Example 20: Validation - Add route without backend");
    let cmd = ApiGatewayRoutes::new(
        config.clone(),
        "add".to_string(),
        Some("/api/test".to_string()),
        None,
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    println!("=== All examples completed successfully ===");
    Ok(())
}