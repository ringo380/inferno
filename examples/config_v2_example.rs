//! Config Command v2 Example
//!
//! Demonstrates the new CLI architecture for the config command.
//! Shows configuration management with show, init, and validate subcommands.
//!
//! Run with: cargo run --example config_v2_example

use anyhow::Result;
use inferno::cli::config_v2::{ConfigInit, ConfigShow, ConfigValidate};
use inferno::config::Config;
use inferno::core::config::ConfigBuilder;
use inferno::interfaces::cli::{CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Config Command v2 Examples\n");

    // Create configuration
    let config = Config::default();
    let _core_config = ConfigBuilder::new().build_unchecked();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Show current configuration
    // ========================================================================
    println!("Example 1: Show Current Configuration");
    println!("{}", "─".repeat(80));

    let show_cmd = ConfigShow::new(config.clone());
    let mut ctx = CommandContext::new(config.clone());
    ctx.json_output = false;

    match pipeline.execute(Box::new(show_cmd), &mut ctx).await {
        Ok(output) => {
            println!("\n✓ {}", output.message);
        }
        Err(e) => {
            eprintln!("✗ Failed: {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 2: Show configuration in JSON format
    // ========================================================================
    println!("Example 2: JSON Output Mode");
    println!("{}", "─".repeat(80));

    let show_json_cmd = ConfigShow::new(config.clone());
    let mut ctx_json = CommandContext::new(config.clone());
    ctx_json.json_output = true;

    match pipeline.execute(Box::new(show_json_cmd), &mut ctx_json).await {
        Ok(output) => {
            if let Some(data) = output.data {
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            }
        }
        Err(e) => {
            eprintln!("✗ Failed: {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 3: Initialize configuration file
    // ========================================================================
    println!("Example 3: Initialize Configuration File");
    println!("{}", "─".repeat(80));

    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join("test_config.toml");

    let init_cmd = ConfigInit::new(Some(config_path.clone()));
    let mut ctx_init = CommandContext::new(config.clone());

    match pipeline.execute(Box::new(init_cmd), &mut ctx_init).await {
        Ok(output) => {
            println!("\n✓ {}", output.message);
            if let Some(data) = output.data {
                println!("  Path: {}", data["path"].as_str().unwrap_or("unknown"));
            }

            // Verify file was created
            if config_path.exists() {
                println!("  File verification: ✓ exists");
                let content = tokio::fs::read_to_string(&config_path).await?;
                println!("  File size: {} bytes", content.len());
            }
        }
        Err(e) => {
            eprintln!("✗ Failed: {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 4: Validate configuration file
    // ========================================================================
    println!("Example 4: Validate Configuration File");
    println!("{}", "─".repeat(80));

    let validate_cmd = ConfigValidate::new(Some(config_path.clone()));
    let mut ctx_validate = CommandContext::new(config.clone());

    match pipeline.execute(Box::new(validate_cmd), &mut ctx_validate).await {
        Ok(output) => {
            println!("\n✓ {}", output.message);
            if let Some(data) = output.data {
                if let Some(warnings) = data["warnings"].as_array() {
                    if !warnings.is_empty() {
                        println!("  Warnings: {} found", warnings.len());
                    }
                }
                if let Some(errors) = data["errors"].as_array() {
                    if !errors.is_empty() {
                        println!("  Errors: {} found", errors.len());
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed: {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 5: Validate non-existent file (error handling)
    // ========================================================================
    println!("Example 5: Validate Non-Existent File");
    println!("{}", "─".repeat(80));

    let nonexistent_path = PathBuf::from("/nonexistent/config.toml");
    let validate_missing = ConfigValidate::new(Some(nonexistent_path.clone()));
    let mut ctx_missing = CommandContext::new(config.clone());

    match pipeline.execute(Box::new(validate_missing), &mut ctx_missing).await {
        Ok(_) => {
            println!("Unexpected success");
        }
        Err(e) => {
            println!("✓ Validation correctly detected missing file:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 6: Initialize with existing file (error handling)
    // ========================================================================
    println!("Example 6: Initialize With Existing File");
    println!("{}", "─".repeat(80));

    let init_duplicate = ConfigInit::new(Some(config_path.clone()));
    let mut ctx_duplicate = CommandContext::new(config.clone());

    match pipeline.execute(Box::new(init_duplicate), &mut ctx_duplicate).await {
        Ok(_) => {
            println!("Unexpected success");
        }
        Err(e) => {
            println!("✓ Initialization correctly detected existing file:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 7: Invalid configuration (create and validate)
    // ========================================================================
    println!("Example 7: Invalid Configuration Detection");
    println!("{}", "─".repeat(80));
    println!("Note: This example demonstrates validation of config values.");
    println!();
    println!("Example invalid configurations:");
    println!("  - Backend context_size = 0");
    println!("  - Backend batch_size = 0");
    println!("  - Server port = 0");
    println!("  - Server port > 65535");
    println!("  - Non-existent models directory");
    println!();
    println!("The validator checks these conditions and reports errors/warnings.");

    // Cleanup
    drop(temp_dir);

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Config Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Show current configuration (TOML format)");
    println!("✓ Initialize default configuration file");
    println!("✓ Validate configuration files");
    println!("✓ Comprehensive validation (values, paths, ranges)");
    println!("✓ Error detection and reporting");
    println!("✓ Warning messages for non-critical issues");
    println!("✓ Structured JSON output");
    println!("✓ Path customization (default or custom paths)");
    println!("✓ Middleware support (logging, metrics)");
    println!();
    println!("Config Validation Checks:");
    println!("  - TOML syntax correctness");
    println!("  - Models directory existence");
    println!("  - Backend configuration values (context_size, batch_size)");
    println!("  - Server configuration values (port range)");
    println!("  - File permissions and accessibility");
    println!();
    println!("Use Cases:");
    println!("  - Display current settings");
    println!("  - Generate starter configuration");
    println!("  - Verify configuration before deployment");
    println!("  - Troubleshoot configuration issues");
    println!("  - Automated configuration testing");

    Ok(())
}