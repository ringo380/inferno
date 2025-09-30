//! Validate Command v2 Example
//!
//! Demonstrates the new CLI architecture for the validate command.
//! Shows file validation, directory validation, checksum validation, and deep validation.
//!
//! Run with: cargo run --example validate_v2_example

use anyhow::Result;
use inferno::cli::validate_v2::ValidateCommand;
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

    println!("🔥 Inferno Validate Command v2 Examples\n");

    // Create configuration
    let config = Config::default();
    let _core_config = ConfigBuilder::new().build_unchecked();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Validate a config file (create temporary TOML)
    // ========================================================================
    println!("Example 1: Config File Validation");
    println!("{}", "─".repeat(80));

    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join("test.toml");

    let config_content = r#"
[models]
directory = "/models"

[server]
host = "127.0.0.1"
port = 8080
"#;
    tokio::fs::write(&config_path, config_content).await?;

    let validate_cmd = ValidateCommand::new(
        config.clone(),
        config_path.clone(),
        false, // checksum
        false, // deep
    );

    let mut ctx = CommandContext::new(config.clone());
    ctx.set_verbosity(1);
    ctx.json_output = false;

    match pipeline.execute(Box::new(validate_cmd), &mut ctx).await {
        Ok(output) => {
            println!("\n✓ {}", output.message);
            if let Some(data) = output.data {
                if ctx.verbose {
                    println!("  Details: {}", serde_json::to_string_pretty(&data).unwrap_or_default());
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Validation failed: {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 2: Validate with JSON output
    // ========================================================================
    println!("Example 2: JSON Output Mode");
    println!("{}", "─".repeat(80));

    let validate_cmd_json = ValidateCommand::new(
        config.clone(),
        config_path.clone(),
        false,
        false,
    );

    let mut ctx_json = CommandContext::new(config.clone());
    ctx_json.json_output = true;

    match pipeline.execute(Box::new(validate_cmd_json), &mut ctx_json).await {
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
    // Example 3: Validate invalid config (syntax error)
    // ========================================================================
    println!("Example 3: Invalid Config Detection");
    println!("{}", "─".repeat(80));

    let invalid_config_path = temp_dir.path().join("invalid.toml");
    let invalid_content = r#"
[models
directory = "/models"  # Missing closing bracket
"#;
    tokio::fs::write(&invalid_config_path, invalid_content).await?;

    let validate_invalid = ValidateCommand::new(
        config.clone(),
        invalid_config_path.clone(),
        false,
        false,
    );

    let mut ctx_invalid = CommandContext::new(config.clone());

    match pipeline.execute(Box::new(validate_invalid), &mut ctx_invalid).await {
        Ok(output) => {
            if output.success {
                println!("Unexpected success");
            } else {
                println!("✓ Validation correctly detected errors:");
                if let Some(data) = output.data {
                    if let Some(errors) = data.get("errors") {
                        println!("  Errors: {}", serde_json::to_string_pretty(errors).unwrap_or_default());
                    }
                }
            }
        }
        Err(e) => {
            println!("✓ Validation error caught: {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 4: Validate with checksum
    // ========================================================================
    println!("Example 4: Checksum Validation");
    println!("{}", "─".repeat(80));

    let validate_checksum = ValidateCommand::new(
        config.clone(),
        config_path.clone(),
        true,  // Enable checksum validation
        false,
    );

    let mut ctx_checksum = CommandContext::new(config.clone());
    ctx_checksum.set_verbosity(1);

    match pipeline.execute(Box::new(validate_checksum), &mut ctx_checksum).await {
        Ok(output) => {
            println!("\n✓ {}", output.message);
            if let Some(data) = output.data {
                if let Some(checksum) = data["details"].get("checksum") {
                    println!("  Checksum: {}", checksum);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed: {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 5: Directory validation (create test directory)
    // ========================================================================
    println!("Example 5: Directory Validation");
    println!("{}", "─".repeat(80));

    let models_dir = temp_dir.path().join("models");
    tokio::fs::create_dir(&models_dir).await?;

    // Create some test files
    tokio::fs::write(models_dir.join("model1.gguf"), b"fake gguf content").await?;
    tokio::fs::write(models_dir.join("model2.onnx"), b"fake onnx content").await?;
    tokio::fs::write(models_dir.join("readme.txt"), b"readme content").await?;

    let validate_dir = ValidateCommand::new(
        config.clone(),
        models_dir.clone(),
        false,
        false,
    );

    let mut ctx_dir = CommandContext::new(config.clone());

    match pipeline.execute(Box::new(validate_dir), &mut ctx_dir).await {
        Ok(output) => {
            println!("\n✓ {}", output.message);
            if let Some(data) = output.data {
                if let Some(model_count) = data["details"].get("model_count") {
                    println!("  Models found: {}", model_count);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed: {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 6: Validation errors (non-existent file)
    // ========================================================================
    println!("Example 6: Path Validation");
    println!("{}", "─".repeat(80));

    let nonexistent_path = PathBuf::from("/nonexistent/model.gguf");
    let validate_error = ValidateCommand::new(
        config.clone(),
        nonexistent_path.clone(),
        false,
        false,
    );

    let mut ctx_error = CommandContext::new(config.clone());

    match pipeline.execute(Box::new(validate_error), &mut ctx_error).await {
        Ok(_) => {
            println!("Unexpected success");
        }
        Err(e) => {
            println!("✓ Validation correctly caught error:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 7: Deep validation note
    // ========================================================================
    println!("Example 7: Deep Validation (Note)");
    println!("{}", "─".repeat(80));
    println!("Deep validation requires a real model file to test loading and inference.");
    println!("When used with --deep flag:");
    println!("  1. Validates file format and metadata");
    println!("  2. Attempts to load model with appropriate backend");
    println!("  3. Runs a test inference to verify model works");
    println!("  4. Reports any loading or inference errors");
    println!();
    println!("Example usage:");
    println!("  let validate_deep = ValidateCommand::new(");
    println!("      config,");
    println!("      PathBuf::from(\"path/to/model.gguf\"),");
    println!("      true,  // checksum");
    println!("      true,  // deep validation");
    println!("  );");

    println!("\n");

    // Cleanup
    drop(temp_dir);

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Validate Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Config file validation (TOML syntax checking)");
    println!("✓ Model file validation (GGUF, ONNX)");
    println!("✓ Directory validation (batch validate all models)");
    println!("✓ Checksum validation (SHA256 computation)");
    println!("✓ Deep validation (load and test inference)");
    println!("✓ Structured output with detailed results");
    println!("✓ JSON and human-readable output modes");
    println!("✓ Comprehensive error detection");
    println!("✓ Middleware support (logging, metrics)");
    println!("✓ Validation before execution pattern");

    Ok(())
}