//! Deployment Command Examples - New Architecture

use anyhow::Result;
use inferno::{
    cli::deployment_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Deployment Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: Deploy to dev environment
    println!("Example 1: Deploy to dev environment");
    let cmd = DeploymentDeploy::new(config.clone(), "dev".to_string(), 3, "rolling".to_string());
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: Deploy to staging with blue-green
    println!("Example 2: Deploy to staging with blue-green");
    let cmd = DeploymentDeploy::new(
        config.clone(),
        "staging".to_string(),
        5,
        "blue-green".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: Deploy to prod with canary
    println!("Example 3: Deploy to prod with canary");
    let cmd = DeploymentDeploy::new(config.clone(), "prod".to_string(), 10, "canary".to_string());
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: Rollback to previous version
    println!("Example 4: Rollback to previous version");
    let cmd = DeploymentRollback::new(config.clone(), "prod".to_string(), None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: Rollback to specific revision
    println!("Example 5: Rollback to specific revision");
    let cmd = DeploymentRollback::new(config.clone(), "staging".to_string(), Some(5));
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: Check status
    println!("Example 6: Check status");
    let cmd = DeploymentStatus::new(config.clone(), "prod".to_string(), false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 7: Check detailed status
    println!("Example 7: Check detailed status");
    let cmd = DeploymentStatus::new(config.clone(), "staging".to_string(), true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 8: Scale deployment
    println!("Example 8: Scale deployment");
    let cmd = DeploymentScale::new(config.clone(), "prod".to_string(), 15);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 9: List config
    println!("Example 9: List config");
    let cmd = DeploymentConfig::new(
        config.clone(),
        "list".to_string(),
        "prod".to_string(),
        None,
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 10: Get config value
    println!("Example 10: Get config value");
    let cmd = DeploymentConfig::new(
        config.clone(),
        "get".to_string(),
        "prod".to_string(),
        Some("image".to_string()),
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 11: Set config value
    println!("Example 11: Set config value");
    let cmd = DeploymentConfig::new(
        config.clone(),
        "set".to_string(),
        "prod".to_string(),
        Some("replicas".to_string()),
        Some("10".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    println!("=== All examples completed successfully ===");
    Ok(())
}
