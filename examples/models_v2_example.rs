//! Models Command v2 Example
//!
//! Demonstrates the new CLI architecture for the models command.
//! Shows how to use Command trait, pipeline, middleware, and structured output.
//!
//! Run with: cargo run --example models_v2_example

use anyhow::Result;
use inferno::cli::models_v2::{ModelsInfo, ModelsList, ModelsValidate};
use inferno::config::Config;
use inferno::core::config::ConfigBuilder;
use inferno::interfaces::cli::{CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for log output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Models Command v2 Examples\n");

    // Create configuration
    let config = Config::default();
    let _core_config = ConfigBuilder::new().build_unchecked();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: List all models (human-readable output)
    // ========================================================================
    println!("Example 1: List All Models (Human-Readable)");
    println!("{}", "─".repeat(80));

    let list_cmd = ModelsList::new(config.clone());
    let mut ctx = CommandContext::new(config.clone());
    ctx.verbose = false;
    ctx.json_output = false;

    match pipeline.execute(Box::new(list_cmd), &mut ctx).await {
        Ok(output) => {
            println!("\n✓ Command completed: {}", output.message);
            if let Some(data) = output.data {
                println!("  Models found: {}", data["count"]);
            }
        }
        Err(e) => {
            eprintln!("✗ Command failed: {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 2: List all models (JSON output)
    // ========================================================================
    println!("Example 2: List All Models (JSON Output)");
    println!("{}", "─".repeat(80));

    let list_cmd_json = ModelsList::new(config.clone());
    let mut ctx_json = CommandContext::new(config.clone());
    ctx_json.verbose = false;
    ctx_json.json_output = true;

    match pipeline.execute(Box::new(list_cmd_json), &mut ctx_json).await {
        Ok(output) => {
            println!("✓ Command completed: {}", output.message);
            if let Some(data) = output.data {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&data).unwrap_or_default()
                );
            }
        }
        Err(e) => {
            eprintln!("✗ Command failed: {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 3: Get info about a specific model
    // ========================================================================
    println!("Example 3: Model Info (if models exist)");
    println!("{}", "─".repeat(80));

    // Try to get info about first model (if any exist)
    let list_cmd_first = ModelsList::new(config.clone());
    let mut ctx_first = CommandContext::new(config.clone());
    ctx_first.json_output = true;

    if let Ok(output) = pipeline.execute(Box::new(list_cmd_first), &mut ctx_first).await {
        if let Some(data) = output.data {
            if let Some(models) = data["models"].as_array() {
                if let Some(first_model) = models.first() {
                    if let Some(model_name) = first_model["name"].as_str() {
                        println!("Getting info for model: {}", model_name);

                        let info_cmd = ModelsInfo::new(config.clone(), model_name.to_string());
                        let mut info_ctx = CommandContext::new(config.clone());
                        info_ctx.verbose = true;

                        match pipeline.execute(Box::new(info_cmd), &mut info_ctx).await {
                            Ok(info_output) => {
                                println!("\n✓ {}", info_output.message);
                            }
                            Err(e) => {
                                eprintln!("✗ Failed to get model info: {}", e);
                            }
                        }
                    }
                } else {
                    println!("No models available to show info for");
                }
            }
        }
    }

    println!("\n");

    // ========================================================================
    // Example 4: Validate a model (if path provided)
    // ========================================================================
    println!("Example 4: Model Validation");
    println!("{}", "─".repeat(80));

    // Check if there's a model file to validate
    let test_model_path = PathBuf::from("test_models/test.gguf");
    if test_model_path.exists() {
        println!("Validating: {}", test_model_path.display());

        let validate_cmd = ModelsValidate::new(test_model_path);
        let mut validate_ctx = CommandContext::new(config.clone());
        validate_ctx.verbose = true;

        match pipeline.execute(Box::new(validate_cmd), &mut validate_ctx).await {
            Ok(val_output) => {
                if val_output.success {
                    println!("\n✓ {}", val_output.message);
                } else {
                    println!("\n⚠ {}", val_output.message);
                }

                if let Some(data) = val_output.data {
                    if let Some(errors) = data["errors"].as_array() {
                        if !errors.is_empty() {
                            println!("  Errors: {:?}", errors);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("✗ Validation error: {}", e);
            }
        }
    } else {
        println!("No test model found at: {}", test_model_path.display());
        println!("Skipping validation example");
    }

    println!("\n");

    // ========================================================================
    // Example 5: Demonstrate error handling
    // ========================================================================
    println!("Example 5: Error Handling (Invalid Path)");
    println!("{}", "─".repeat(80));

    let invalid_path = PathBuf::from("/nonexistent/model.gguf");
    let validate_invalid = ModelsValidate::new(invalid_path.clone());
    let mut error_ctx = CommandContext::new(config.clone());

    match pipeline.execute(Box::new(validate_invalid), &mut error_ctx).await {
        Ok(output) => {
            println!("Unexpected success: {}", output.message);
        }
        Err(e) => {
            println!("✓ Error handling worked correctly:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: New CLI Architecture Benefits");
    println!("{}", "═".repeat(80));
    println!("✓ Structured output with CommandOutput");
    println!("✓ Middleware support (logging, metrics, validation)");
    println!("✓ Consistent error handling");
    println!("✓ JSON and human-readable output modes");
    println!("✓ Validation before execution");
    println!("✓ Testable command implementations");
    println!("✓ Context propagation (verbose, dry-run, etc.)");

    Ok(())
}