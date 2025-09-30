//! Federated Learning Command Examples - New Architecture
//!
//! This example demonstrates the usage of federated learning commands in the v2 architecture.

use anyhow::Result;
use inferno::{
    cli::federated_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Federated Learning Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: Start coordinator node
    println!("Example 1: Start coordinator node");
    let cmd = FederatedStart::new(
        config.clone(),
        "coordinator".to_string(),
        8090,
        None,
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: Start participant node
    println!("Example 2: Start participant node");
    let cmd = FederatedStart::new(
        config.clone(),
        "participant".to_string(),
        8091,
        Some("http://coordinator:8090".to_string()),
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: Start node as daemon
    println!("Example 3: Start node as daemon");
    let cmd = FederatedStart::new(
        config.clone(),
        "both".to_string(),
        8090,
        None,
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: Check cluster status
    println!("Example 4: Check cluster status");
    let cmd = FederatedStatus::new(config.clone(), None, false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: Check specific node status
    println!("Example 5: Check specific node status");
    let cmd = FederatedStatus::new(config.clone(), Some("node-001".to_string()), false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: Watch mode (stub)
    println!("Example 6: Watch mode (stub)");
    let cmd = FederatedStatus::new(config.clone(), None, true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 7: Start training round with defaults
    println!("Example 7: Start training round with defaults");
    let cmd = FederatedRoundStart::new(config.clone(), None, None, None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 8: Start round with participant limits
    println!("Example 8: Start round with participant limits");
    let cmd = FederatedRoundStart::new(config.clone(), Some(3), Some(10), None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 9: Start round with strategy
    println!("Example 9: Start round with federated averaging strategy");
    let cmd = FederatedRoundStart::new(
        config.clone(),
        Some(5),
        Some(15),
        Some(3600),
        Some("federated_averaging".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 10: Secure aggregation strategy
    println!("Example 10: Start round with secure aggregation");
    let cmd = FederatedRoundStart::new(
        config.clone(),
        Some(5),
        Some(10),
        None,
        Some("secure_aggregation".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 11: List all participants
    println!("Example 11: List all participants");
    let cmd = FederatedParticipants::new(
        config.clone(),
        "list".to_string(),
        None,
        None,
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 12: Get participant info
    println!("Example 12: Get participant info");
    let cmd = FederatedParticipants::new(
        config.clone(),
        "info".to_string(),
        Some("node-001".to_string()),
        None,
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 13: Register new participant
    println!("Example 13: Register new participant");
    let cmd = FederatedParticipants::new(
        config.clone(),
        "register".to_string(),
        None,
        Some("192.168.1.20:8091".to_string()),
        Some(0.85),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 14: Remove participant
    println!("Example 14: Remove participant");
    let cmd = FederatedParticipants::new(
        config.clone(),
        "remove".to_string(),
        Some("node-004".to_string()),
        None,
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 15: Deploy model to all edge devices
    println!("Example 15: Deploy model to all edge devices");
    let cmd = FederatedEdgeDeploy::new(
        config.clone(),
        "llama-federated-v16".to_string(),
        None,
        None,
        false,
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 16: Deploy with specific targets
    println!("Example 16: Deploy with specific targets");
    let cmd = FederatedEdgeDeploy::new(
        config.clone(),
        "llama-federated-v16".to_string(),
        Some("edge-001,edge-002,edge-003".to_string()),
        Some("push".to_string()),
        true,
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 17: Test communication
    println!("Example 17: Test communication");
    let cmd = FederatedTest::new(config.clone(), true, false, false, false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 18: Test aggregation
    println!("Example 18: Test aggregation");
    let cmd = FederatedTest::new(config.clone(), false, true, false, false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 19: Comprehensive test suite
    println!("Example 19: Comprehensive test suite");
    let cmd = FederatedTest::new(config.clone(), false, false, false, true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 20: Validation - Invalid role
    println!("Example 20: Validation - Invalid role");
    let cmd = FederatedStart::new(
        config.clone(),
        "invalid".to_string(),
        8090,
        None,
        false,
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 21: Validation - Participant without coordinator
    println!("Example 21: Validation - Participant without coordinator");
    let cmd = FederatedStart::new(
        config.clone(),
        "participant".to_string(),
        8091,
        None,
        false,
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 22: Validation - Invalid aggregation strategy
    println!("Example 22: Validation - Invalid aggregation strategy");
    let cmd = FederatedRoundStart::new(
        config.clone(),
        Some(5),
        Some(10),
        None,
        Some("invalid_strategy".to_string()),
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    println!("=== All examples completed successfully ===");
    Ok(())
}