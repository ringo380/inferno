//! Run Command v2 Example
//!
//! Demonstrates the new CLI architecture for the run command.
//! Shows single inference, batch processing, and streaming modes.
//!
//! Run with: cargo run --example run_v2_example

use anyhow::Result;
use inferno::cli::run_v2::RunCommand;
use inferno::config::Config;
use inferno::core::config::ConfigBuilder;
use inferno::interfaces::cli::{CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware};
use inferno::io::{InputFormat, OutputFormat};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Run Command v2 Examples\n");

    // Create configuration
    let config = Config::default();
    let _core_config = ConfigBuilder::new().build_unchecked();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Simple text generation with prompt
    // ========================================================================
    println!("Example 1: Simple Text Generation");
    println!("{}", "─".repeat(80));

    let run_cmd = RunCommand::new(
        config.clone(),
        "test-model".to_string(),
        Some("What is the meaning of life?".to_string()),
        None,
        None,
        InputFormat::Text,
        OutputFormat::Text,
        50,    // max_tokens
        0.7,   // temperature
        0.9,   // top_p
        false, // stream
        false, // batch
        None,  // backend
    );

    let mut ctx = CommandContext::new(config.clone());
    ctx.verbose = false;
    ctx.json_output = false;

    match pipeline.execute(Box::new(run_cmd), &mut ctx).await {
        Ok(output) => {
            println!("\n✓ {}", output.message);
            if let Some(data) = output.data {
                if let Some(elapsed) = data["elapsed_ms"].as_u64() {
                    println!("  Time: {}ms", elapsed);
                }
                if let Some(metrics) = data["metrics"].as_object() {
                    if let Some(tps) = metrics["tokens_per_second"].as_f64() {
                        println!("  Speed: {:.2} tokens/sec", tps);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Command failed: {}", e);
            println!("  Note: This example requires a model to be available");
        }
    }

    println!("\n");

    // ========================================================================
    // Example 2: JSON output mode
    // ========================================================================
    println!("Example 2: JSON Output Mode");
    println!("{}", "─".repeat(80));

    let run_cmd_json = RunCommand::new(
        config.clone(),
        "test-model".to_string(),
        Some("Hello, world!".to_string()),
        None,
        None,
        InputFormat::Text,
        OutputFormat::Json,
        20,
        0.7,
        0.9,
        false,
        false,
        None,
    );

    let mut ctx_json = CommandContext::new(config.clone());
    ctx_json.json_output = true;

    match pipeline.execute(Box::new(run_cmd_json), &mut ctx_json).await {
        Ok(output) => {
            if let Some(data) = output.data {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&data).unwrap_or_default()
                );
            }
        }
        Err(e) => {
            eprintln!("✗ Failed: {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 3: Streaming mode (if model available)
    // ========================================================================
    println!("Example 3: Streaming Mode");
    println!("{}", "─".repeat(80));
    println!("Note: Streaming mode would display tokens in real-time");

    let run_cmd_stream = RunCommand::new(
        config.clone(),
        "test-model".to_string(),
        Some("Count to 5".to_string()),
        None,
        None,
        InputFormat::Text,
        OutputFormat::Text,
        30,
        0.7,
        0.9,
        true,  // stream enabled
        false,
        None,
    );

    let mut ctx_stream = CommandContext::new(config.clone());

    match pipeline.execute(Box::new(run_cmd_stream), &mut ctx_stream).await {
        Ok(output) => {
            println!("\n✓ {}", output.message);
        }
        Err(e) => {
            eprintln!("✗ Streaming failed: {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 4: Batch processing (with mock input file)
    // ========================================================================
    println!("Example 4: Batch Processing");
    println!("{}", "─".repeat(80));

    // Create a temporary batch input file
    let temp_dir = tempfile::tempdir()?;
    let input_file = temp_dir.path().join("batch_input.jsonl");
    let output_file = temp_dir.path().join("batch_output.jsonl");

    let batch_content = r#"{"prompt": "What is 2+2?"}
{"prompt": "What is 3+3?"}
{"prompt": "What is 4+4?"}"#;

    tokio::fs::write(&input_file, batch_content).await?;
    println!("Created temporary batch input: {}", input_file.display());

    let run_cmd_batch = RunCommand::new(
        config.clone(),
        "test-model".to_string(),
        None,
        Some(input_file.clone()),
        Some(output_file.clone()),
        InputFormat::Json,
        OutputFormat::JsonLines,
        20,
        0.7,
        0.9,
        false,
        true,  // batch mode enabled
        None,
    );

    let mut ctx_batch = CommandContext::new(config.clone());

    match pipeline.execute(Box::new(run_cmd_batch), &mut ctx_batch).await {
        Ok(output) => {
            println!("\n✓ {}", output.message);
            if let Some(data) = output.data {
                println!(
                    "  Success Rate: {:.1}%",
                    data["success_rate"].as_f64().unwrap_or(0.0)
                );
            }
        }
        Err(e) => {
            eprintln!("✗ Batch processing failed: {}", e);
            eprintln!("  This is expected without a real model");
        }
    }

    // Cleanup
    drop(temp_dir);

    println!("\n");

    // ========================================================================
    // Example 5: Validation demonstration
    // ========================================================================
    println!("Example 5: Input Validation");
    println!("{}", "─".repeat(80));

    // Test with invalid parameters
    let invalid_cmd = RunCommand::new(
        config.clone(),
        "".to_string(), // Empty model name - should fail validation
        None,
        None,
        None,
        InputFormat::Text,
        OutputFormat::Text,
        0,  // Invalid: max_tokens = 0
        3.0, // Invalid: temperature > 2.0
        0.9,
        false,
        false,
        None,
    );

    let mut ctx_invalid = CommandContext::new(config.clone());

    match pipeline.execute(Box::new(invalid_cmd), &mut ctx_invalid).await {
        Ok(_) => {
            println!("Unexpected success");
        }
        Err(e) => {
            println!("✓ Validation correctly caught errors:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Run Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Single inference with prompt or file input");
    println!("✓ Batch processing from JSON/JSONL/CSV files");
    println!("✓ Streaming token output for real-time display");
    println!("✓ Comprehensive parameter validation");
    println!("✓ Structured output with metrics");
    println!("✓ JSON and human-readable output modes");
    println!("✓ Middleware support (logging, metrics)");
    println!("✓ Automatic backend selection");
    println!("✓ Progress tracking for batch jobs");
    println!("✓ Error handling with retry logic");

    Ok(())
}