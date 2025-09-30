//! Model Versioning Command Examples - New Architecture
//!
//! This example demonstrates the usage of model versioning commands in the v2 architecture.

use anyhow::Result;
use inferno::{
    cli::model_versioning_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Model Versioning Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: Create new model version
    println!("Example 1: Create new model version");
    let cmd = ModelVersionCreate::new(
        config.clone(),
        "llama-7b".to_string(),
        "v1.0".to_string(),
        PathBuf::from("/tmp/model.gguf"),
        Some("Initial release".to_string()),
        vec!["production".to_string(), "verified".to_string()],
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: Create version without metadata
    println!("Example 2: Create version without metadata");
    let cmd = ModelVersionCreate::new(
        config.clone(),
        "llama-13b".to_string(),
        "v2.0".to_string(),
        PathBuf::from("/tmp/model2.gguf"),
        None,
        vec![],
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: List all model versions
    println!("Example 3: List all model versions");
    let cmd = ModelVersionList::new(config.clone(), None, None, 50);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: List versions for specific model
    println!("Example 4: List versions for specific model");
    let cmd = ModelVersionList::new(config.clone(), Some("llama-7b".to_string()), None, 20);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: List only active versions
    println!("Example 5: List only active versions");
    let cmd = ModelVersionList::new(config.clone(), None, Some("active".to_string()), 100);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: Deploy with blue-green strategy
    println!("Example 6: Deploy with blue-green strategy");
    let cmd = ModelVersionDeploy::new(
        config.clone(),
        "llama-7b-v1.0".to_string(),
        "production".to_string(),
        "blue_green".to_string(),
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 7: Canary deployment with 10% traffic
    println!("Example 7: Canary deployment with 10% traffic");
    let cmd = ModelVersionDeploy::new(
        config.clone(),
        "llama-7b-v1.1".to_string(),
        "production".to_string(),
        "canary".to_string(),
        Some(10),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 8: Rolling deployment with 50% traffic
    println!("Example 8: Rolling deployment with 50% traffic");
    let cmd = ModelVersionDeploy::new(
        config.clone(),
        "llama-13b-v2.0".to_string(),
        "staging".to_string(),
        "rolling".to_string(),
        Some(50),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 9: Immediate deployment
    println!("Example 9: Immediate deployment");
    let cmd = ModelVersionDeploy::new(
        config.clone(),
        "llama-7b-v1.2".to_string(),
        "development".to_string(),
        "immediate".to_string(),
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 10: Rollback to previous version
    println!("Example 10: Rollback to previous version");
    let cmd = ModelVersionRollback::new(
        config.clone(),
        "production".to_string(),
        None,
        Some("Critical bug found".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 11: Rollback to specific version
    println!("Example 11: Rollback to specific version");
    let cmd = ModelVersionRollback::new(
        config.clone(),
        "staging".to_string(),
        Some("llama-7b-v1.0".to_string()),
        Some("Performance regression".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 12: Compare two versions
    println!("Example 12: Compare two versions");
    let cmd = ModelVersionCompare::new(
        config.clone(),
        "llama-7b-v1.0".to_string(),
        "llama-7b-v1.1".to_string(),
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 13: Detailed comparison
    println!("Example 13: Detailed comparison");
    let cmd = ModelVersionCompare::new(
        config.clone(),
        "llama-13b-v2.0".to_string(),
        "llama-13b-v1.5".to_string(),
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 14: Validate with smoke tests
    println!("Example 14: Validate with smoke tests");
    let cmd = ModelVersionValidate::new(
        config.clone(),
        "llama-7b-v1.0".to_string(),
        Some("smoke".to_string()),
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 15: Comprehensive validation
    println!("Example 15: Comprehensive validation");
    let cmd = ModelVersionValidate::new(
        config.clone(),
        "llama-13b-v2.0".to_string(),
        Some("comprehensive".to_string()),
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 16: Validate skipping benchmarks
    println!("Example 16: Validate skipping benchmarks");
    let cmd = ModelVersionValidate::new(
        config.clone(),
        "llama-7b-v1.1".to_string(),
        Some("regression".to_string()),
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 17: Export with metadata only
    println!("Example 17: Export with metadata only");
    let cmd = ModelVersionExport::new(
        config.clone(),
        "llama-7b-v1.0".to_string(),
        PathBuf::from("/tmp/export.tar.gz"),
        true,
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 18: Export with full lineage
    println!("Example 18: Export with full lineage");
    let cmd = ModelVersionExport::new(
        config.clone(),
        "llama-13b-v2.0".to_string(),
        PathBuf::from("/tmp/full-export.tar.gz"),
        true,
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 19: Validation - Empty model name
    println!("Example 19: Validation - Empty model name");
    let cmd = ModelVersionCreate::new(
        config.clone(),
        "".to_string(),
        "v1.0".to_string(),
        PathBuf::from("/tmp/model.gguf"),
        None,
        vec![],
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 20: Validation - Invalid deployment strategy
    println!("Example 20: Validation - Invalid deployment strategy");
    let cmd = ModelVersionDeploy::new(
        config.clone(),
        "llama-7b-v1.0".to_string(),
        "prod".to_string(),
        "invalid_strategy".to_string(),
        None,
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 21: Validation - Same versions in comparison
    println!("Example 21: Validation - Same versions in comparison");
    let cmd = ModelVersionCompare::new(
        config.clone(),
        "v1.0".to_string(),
        "v1.0".to_string(),
        false,
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    println!("=== All examples completed successfully ===");
    Ok(())
}
