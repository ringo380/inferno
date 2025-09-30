//! Multi-Tenancy Command Examples - New Architecture

use anyhow::Result;
use inferno::{
    cli::multi_tenancy_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Multi-Tenancy Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: Create enterprise tenant
    println!("Example 1: Create enterprise tenant");
    let cmd = TenantCreate::new(
        config.clone(),
        "acme-corp".to_string(),
        "enterprise".to_string(),
        Some("admin@acme.com".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: List all tenants
    println!("Example 2: List all tenants");
    let cmd = TenantList::new(config.clone(), None, false, 100);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: List pro tier tenants
    println!("Example 3: List pro tier tenants");
    let cmd = TenantList::new(config.clone(), Some("pro".to_string()), true, 50);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: Get isolation settings
    println!("Example 4: Get isolation settings");
    let cmd = TenantIsolation::new(
        config.clone(),
        "tenant-123".to_string(),
        "get".to_string(),
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: Set strict isolation
    println!("Example 5: Set strict isolation");
    let cmd = TenantIsolation::new(
        config.clone(),
        "tenant-123".to_string(),
        "set".to_string(),
        Some("strict".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: Get quotas
    println!("Example 6: Get quotas");
    let cmd = TenantQuotas::new(
        config.clone(),
        "tenant-123".to_string(),
        "get".to_string(),
        None,
        None,
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 7: Set CPU and memory quotas
    println!("Example 7: Set CPU and memory quotas");
    let cmd = TenantQuotas::new(
        config.clone(),
        "tenant-123".to_string(),
        "set".to_string(),
        Some(16),
        Some(32),
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 8: Reset quotas
    println!("Example 8: Reset quotas");
    let cmd = TenantQuotas::new(
        config.clone(),
        "tenant-123".to_string(),
        "reset".to_string(),
        None,
        None,
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 9: Show 24h metrics
    println!("Example 9: Show 24h metrics");
    let cmd = TenantMetrics::new(
        config.clone(),
        "tenant-123".to_string(),
        "24h".to_string(),
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 10: Detailed 7-day metrics
    println!("Example 10: Detailed 7-day metrics");
    let cmd = TenantMetrics::new(
        config.clone(),
        "tenant-123".to_string(),
        "7d".to_string(),
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 11: Delete with backup
    println!("Example 11: Delete with backup");
    let cmd = TenantDelete::new(
        config.clone(),
        "tenant-old".to_string(),
        false,
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 12: Force delete
    println!("Example 12: Force delete");
    let cmd = TenantDelete::new(
        config.clone(),
        "tenant-test".to_string(),
        true,
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 13: Validation - Empty tenant name
    println!("Example 13: Validation - Empty tenant name");
    let cmd = TenantCreate::new(config.clone(), "".to_string(), "pro".to_string(), None);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 14: Validation - Invalid tier
    println!("Example 14: Validation - Invalid tier");
    let cmd = TenantCreate::new(
        config.clone(),
        "test".to_string(),
        "invalid".to_string(),
        None,
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 15: Validation - Delete without confirmation
    println!("Example 15: Validation - Delete without confirmation");
    let cmd = TenantDelete::new(config.clone(), "tenant-123".to_string(), false, false);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    println!("=== All examples completed successfully ===");
    Ok(())
}