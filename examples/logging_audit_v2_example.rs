//! Logging and Audit Command Examples - New Architecture
//!
//! This example demonstrates the usage of logging and audit commands in the v2 architecture.

use anyhow::Result;
use inferno::{
    cli::logging_audit_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Logging and Audit Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: List all audit events
    println!("Example 1: List all audit events");
    let cmd = AuditEvents::new(config.clone(), "list".to_string(), None, None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: Search events by type
    println!("Example 2: Search events by type");
    let cmd = AuditEvents::new(
        config.clone(),
        "search".to_string(),
        Some("USER_LOGIN".to_string()),
        Some("24h".to_string()),
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: Search events by user
    println!("Example 3: Search events by user");
    let cmd = AuditEvents::new(
        config.clone(),
        "search".to_string(),
        None,
        Some("7d".to_string()),
        Some("admin".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: Export audit events
    println!("Example 4: Export audit events");
    let cmd = AuditEvents::new(
        config.clone(),
        "export".to_string(),
        None,
        Some("30d".to_string()),
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: Get logging configuration
    println!("Example 5: Get logging configuration");
    let cmd = LoggingConfig::new(config.clone(), "get".to_string(), None, None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: Set log level
    println!("Example 6: Set log level");
    let cmd = LoggingConfig::new(
        config.clone(),
        "set".to_string(),
        Some("debug".to_string()),
        None,
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 7: Set log format
    println!("Example 7: Set log format");
    let cmd = LoggingConfig::new(
        config.clone(),
        "set".to_string(),
        None,
        Some("json".to_string()),
        None,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 8: Set log output
    println!("Example 8: Set log output");
    let cmd = LoggingConfig::new(
        config.clone(),
        "set".to_string(),
        None,
        None,
        Some("stdout,file".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 9: Generate SOC2 compliance report
    println!("Example 9: Generate SOC2 compliance report");
    let cmd = ComplianceReport::new(config.clone(), "soc2".to_string(), "30d".to_string(), false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 10: Detailed GDPR compliance report
    println!("Example 10: Detailed GDPR compliance report");
    let cmd = ComplianceReport::new(config.clone(), "gdpr".to_string(), "90d".to_string(), true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 11: HIPAA compliance report
    println!("Example 11: HIPAA compliance report");
    let cmd =
        ComplianceReport::new(config.clone(), "hipaa".to_string(), "30d".to_string(), false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 12: List all security alerts
    println!("Example 12: List all security alerts");
    let cmd = SecurityAlerts::new(config.clone(), "list".to_string(), None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 13: List critical security alerts
    println!("Example 13: List critical security alerts");
    let cmd = SecurityAlerts::new(
        config.clone(),
        "list".to_string(),
        Some("critical".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 14: Acknowledge security alerts
    println!("Example 14: Acknowledge security alerts");
    let cmd = SecurityAlerts::new(config.clone(), "acknowledge".to_string(), None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 15: Get retention policies
    println!("Example 15: Get retention policies");
    let cmd = RetentionPolicy::new(config.clone(), "get".to_string(), None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 16: Set audit log retention
    println!("Example 16: Set audit log retention");
    let cmd = RetentionPolicy::new(
        config.clone(),
        "set".to_string(),
        Some("audit".to_string()),
        Some(365),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 17: Apply retention policy
    println!("Example 17: Apply retention policy");
    let cmd = RetentionPolicy::new(config.clone(), "apply".to_string(), None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 18: Export to JSON
    println!("Example 18: Export to JSON");
    let cmd = AuditExport::new(
        config.clone(),
        "json".to_string(),
        Some("/tmp/audit.json".to_string()),
        "24h".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 19: Export to CSV
    println!("Example 19: Export to CSV");
    let cmd = AuditExport::new(
        config.clone(),
        "csv".to_string(),
        None,
        "7d".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 20: Quick integrity check
    println!("Example 20: Quick integrity check");
    let cmd = AuditIntegrity::new(config.clone(), "quick".to_string());
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 21: Full integrity verification
    println!("Example 21: Full integrity verification");
    let cmd = AuditIntegrity::new(config.clone(), "full".to_string());
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 22: Validation - Invalid action
    println!("Example 22: Validation - Invalid action");
    let cmd = AuditEvents::new(config.clone(), "invalid".to_string(), None, None, None);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 23: Validation - Set config without params
    println!("Example 23: Validation - Set config without params");
    let cmd = LoggingConfig::new(config.clone(), "set".to_string(), None, None, None);
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 24: Validation - Invalid compliance standard
    println!("Example 24: Validation - Invalid compliance standard");
    let cmd = ComplianceReport::new(
        config.clone(),
        "invalid".to_string(),
        "30d".to_string(),
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