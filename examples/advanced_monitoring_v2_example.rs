//! Advanced Monitoring Command Examples - New Architecture
//!
//! This example demonstrates the usage of advanced monitoring commands in the v2 architecture.

use anyhow::Result;
use inferno::{
    cli::advanced_monitoring_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Advanced Monitoring Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: Start monitoring in foreground
    println!("Example 1: Start monitoring in foreground");
    let cmd = MonitoringStart::new(config.clone(), 9090, 3000, false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: Start as daemon
    println!("Example 2: Start as daemon");
    let cmd = MonitoringStart::new(config.clone(), 9090, 3001, true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: Basic status
    println!("Example 3: Basic status");
    let cmd = MonitoringStatus::new(config.clone(), false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: Detailed status
    println!("Example 4: Detailed status");
    let cmd = MonitoringStatus::new(config.clone(), true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: List all alerts
    println!("Example 5: List all alerts");
    let cmd = MonitoringAlerts::new(config.clone(), "list".to_string(), None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: List critical alerts
    println!("Example 6: List critical alerts");
    let cmd = MonitoringAlerts::new(
        config.clone(),
        "list".to_string(),
        None,
        Some("critical".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 7: Add alert
    println!("Example 7: Add alert");
    let cmd = MonitoringAlerts::new(
        config.clone(),
        "add".to_string(),
        Some("high_memory".to_string()),
        Some("warning".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 8: Silence alert
    println!("Example 8: Silence alert");
    let cmd = MonitoringAlerts::new(
        config.clone(),
        "silence".to_string(),
        Some("maintenance_alert".to_string()),
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 9: List targets
    println!("Example 9: List targets");
    let cmd = MonitoringTargets::new(config.clone(), "list".to_string(), None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 10: Add target
    println!("Example 10: Add target");
    let cmd = MonitoringTargets::new(
        config.clone(),
        "add".to_string(),
        Some("http://localhost:8080/metrics".to_string()),
        Some("job=api,env=prod".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 11: Check target health
    println!("Example 11: Check target health");
    let cmd = MonitoringTargets::new(
        config.clone(),
        "health".to_string(),
        Some("http://localhost:8080/metrics".to_string()),
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 12: Remove target
    println!("Example 12: Remove target");
    let cmd = MonitoringTargets::new(
        config.clone(),
        "remove".to_string(),
        Some("http://localhost:8082/metrics".to_string()),
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 13: Show 1 hour metrics
    println!("Example 13: Show 1 hour metrics");
    let cmd = MonitoringMetrics::new(config.clone(), "1h".to_string(), None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 14: Show 24h metrics with query
    println!("Example 14: Show 24h metrics with query");
    let cmd = MonitoringMetrics::new(
        config.clone(),
        "24h".to_string(),
        Some("http_requests_total".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 15: Show 7 day metrics
    println!("Example 15: Show 7 day metrics");
    let cmd = MonitoringMetrics::new(config.clone(), "7d".to_string(), None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 16: Basic health check
    println!("Example 16: Basic health check");
    let cmd = MonitoringHealth::new(config.clone(), false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 17: Comprehensive health check
    println!("Example 17: Comprehensive health check");
    let cmd = MonitoringHealth::new(config.clone(), true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 18: Validation - Zero metrics port
    println!("Example 18: Validation - Zero metrics port");
    let cmd = MonitoringStart::new(config.clone(), 0, 3000, false);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 19: Validation - Same ports
    println!("Example 19: Validation - Same ports");
    let cmd = MonitoringStart::new(config.clone(), 3000, 3000, false);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 20: Validation - Invalid alert action
    println!("Example 20: Validation - Invalid alert action");
    let cmd = MonitoringAlerts::new(config.clone(), "invalid".to_string(), None, None);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 21: Validation - Add alert without name
    println!("Example 21: Validation - Add alert without name");
    let cmd = MonitoringAlerts::new(config.clone(), "add".to_string(), None, None);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 22: Validation - Invalid time range
    println!("Example 22: Validation - Invalid time range");
    let cmd = MonitoringMetrics::new(config.clone(), "invalid".to_string(), None);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    println!("=== All examples completed successfully ===");
    Ok(())
}
