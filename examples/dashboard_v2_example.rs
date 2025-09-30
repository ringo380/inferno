//! Dashboard Command Examples - New Architecture
//!
//! This example demonstrates the usage of dashboard commands in the v2 architecture.

use anyhow::Result;
use inferno::{
    cli::dashboard_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Dashboard Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: Start dashboard on default port
    println!("Example 1: Start dashboard on default port");
    let cmd = DashboardStart::new(
        config.clone(),
        "127.0.0.1".to_string(),
        8080,
        false,
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: Start with authentication
    println!("Example 2: Start with authentication");
    let cmd = DashboardStart::new(
        config.clone(),
        "0.0.0.0".to_string(),
        8080,
        true,
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: Start as daemon
    println!("Example 3: Start as daemon");
    let cmd = DashboardStart::new(
        config.clone(),
        "127.0.0.1".to_string(),
        8080,
        false,
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: Basic status
    println!("Example 4: Basic status");
    let cmd = DashboardStatus::new(
        config.clone(),
        "http://localhost:8080".to_string(),
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: Detailed status
    println!("Example 5: Detailed status");
    let cmd = DashboardStatus::new(
        config.clone(),
        "http://localhost:8080".to_string(),
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: Get configuration
    println!("Example 6: Get configuration");
    let cmd = DashboardConfig::new(config.clone(), "get".to_string(), None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 7: Enable authentication
    println!("Example 7: Enable authentication");
    let cmd = DashboardConfig::new(config.clone(), "set".to_string(), Some(true), None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 8: Set dark theme
    println!("Example 8: Set dark theme");
    let cmd = DashboardConfig::new(
        config.clone(),
        "set".to_string(),
        None,
        Some("dark".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 9: Initialize configuration
    println!("Example 9: Initialize configuration");
    let cmd = DashboardConfig::new(config.clone(), "init".to_string(), None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 10: Export metrics as JSON
    println!("Example 10: Export metrics as JSON");
    let cmd = DashboardExport::new(
        config.clone(),
        "metrics".to_string(),
        "json".to_string(),
        "24h".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 11: Export logs as CSV
    println!("Example 11: Export logs as CSV");
    let cmd = DashboardExport::new(
        config.clone(),
        "logs".to_string(),
        "csv".to_string(),
        "7d".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 12: Export all data as HTML
    println!("Example 12: Export all data as HTML");
    let cmd = DashboardExport::new(
        config.clone(),
        "all".to_string(),
        "html".to_string(),
        "30d".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 13: Validation - Zero port
    println!("Example 13: Validation - Zero port");
    let cmd = DashboardStart::new(
        config.clone(),
        "127.0.0.1".to_string(),
        0,
        false,
        false,
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 14: Validation - Empty address
    println!("Example 14: Validation - Empty address");
    let cmd = DashboardStart::new(config.clone(), "".to_string(), 8080, false, false);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 15: Validation - Set config without params
    println!("Example 15: Validation - Set config without params");
    let cmd = DashboardConfig::new(config.clone(), "set".to_string(), None, None);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 16: Validation - Invalid export type
    println!("Example 16: Validation - Invalid export type");
    let cmd = DashboardExport::new(
        config.clone(),
        "invalid".to_string(),
        "json".to_string(),
        "24h".to_string(),
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    println!("=== All examples completed successfully ===");
    Ok(())
}