//! Batch Command v2 Example
//!
//! Demonstrates the new CLI architecture for the batch processing command.
//! Shows bulk inference processing with various configurations.
//!
//! Run with: cargo run --example batch_v2_example

use anyhow::Result;
use inferno::batch::BatchOutputFormat;
use inferno::cli::batch_v2::BatchProcess;
use inferno::config::Config;
use inferno::core::config::ConfigBuilder;
use inferno::interfaces::cli::{
    CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Batch Command v2 Examples\n");

    // Create configuration
    let config = Config::default();
    let _core_config = ConfigBuilder::new().build_unchecked();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Basic batch processing
    // ========================================================================
    println!("Example 1: Basic Batch Processing");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let batch_cmd = BatchProcess::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b-chat.gguf\".to_string(),");
    println!("      PathBuf::from(\"inputs.jsonl\"),");
    println!("      Some(PathBuf::from(\"outputs.jsonl\")),");
    println!("      BatchOutputFormat::JsonLines,");
    println!("      512,    // max_tokens");
    println!("      0.7,    // temperature");
    println!("      0.9,    // top_p");
    println!("      4,      // concurrency");
    println!("      300,    // timeout (seconds)");
    println!("      3,      // retries");
    println!("      100,    // checkpoint interval");
    println!("      false,  // continue_on_error");
    println!("      false,  // shuffle");
    println!("      false,  // enable_metrics");
    println!("      None,   // resume checkpoint");
    println!("      false,  // dry_run");
    println!("      None,   // backend");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  Loading model...");
    println!("  Model loaded in 12.3s");
    println!("  Estimated 1000 items to process");
    println!("  Output will be saved to: outputs.jsonl");
    println!("  [Progress indicators...]");
    println!();
    println!("  === Batch Processing Summary ===");
    println!("  Input file: inputs.jsonl");
    println!("  Model: llama-2-7b-chat.gguf");
    println!("  Total items: 1000");
    println!("  Completed: 998");
    println!("  Failed: 2");
    println!("  Skipped: 0");
    println!("  Success rate: 99.8%");
    println!("  Processing time: 4m 32s");
    println!("  Average rate: 3.67 items/second");
    println!("  Output saved to: outputs.jsonl");
    println!();
    println!("  ✅ Batch processing completed successfully!");

    println!("\n");

    // ========================================================================
    // Example 2: High concurrency processing
    // ========================================================================
    println!("Example 2: High Concurrency Processing");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let high_concurrency = BatchProcess::new(");
    println!("      config.clone(),");
    println!("      \"mistral-7b-instruct.gguf\".to_string(),");
    println!("      PathBuf::from(\"large_dataset.jsonl\"),");
    println!("      Some(PathBuf::from(\"results.jsonl\")),");
    println!("      BatchOutputFormat::JsonLines,");
    println!("      256,    // shorter max_tokens for speed");
    println!("      0.5,    // lower temperature for consistency");
    println!("      0.95,   // top_p");
    println!("      16,     // high concurrency");
    println!("      180,    // shorter timeout");
    println!("      2,      // fewer retries");
    println!("      50,     // more frequent checkpoints");
    println!("      true,   // continue on error");
    println!("      true,   // shuffle for load balancing");
    println!("      true,   // enable metrics");
    println!("      None,");
    println!("      false,");
    println!("      None,");
    println!("  );");
    println!();
    println!("Benefits:");
    println!("  - 16 concurrent requests maximize throughput");
    println!("  - Shuffling improves load distribution");
    println!("  - Continue on error ensures completion");
    println!("  - Frequent checkpoints for recovery");
    println!("  - Metrics collection for analysis");

    println!("\n");

    // ========================================================================
    // Example 3: CSV/TSV input processing
    // ========================================================================
    println!("Example 3: CSV/TSV Input Processing");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let csv_batch = BatchProcess::new(");
    println!("      config.clone(),");
    println!("      \"codellama-13b.gguf\".to_string(),");
    println!("      PathBuf::from(\"prompts.csv\"),");
    println!("      Some(PathBuf::from(\"completions.csv\")),");
    println!("      BatchOutputFormat::Csv,  // Match input format");
    println!("      1024,   // more tokens for code");
    println!("      0.2,    // low temp for code generation");
    println!("      0.95,");
    println!("      8,");
    println!("      600,    // longer timeout for complex code");
    println!("      3,");
    println!("      100,");
    println!("      false,");
    println!("      false,");
    println!("      false,");
    println!("      None,");
    println!("      false,");
    println!("      None,");
    println!("  );");
    println!();
    println!("Input CSV format:");
    println!("  id,prompt");
    println!("  1,\"Write a function to reverse a string\"");
    println!("  2,\"Implement binary search in Python\"");
    println!("  3,\"Create a REST API endpoint\"");
    println!();
    println!("Output CSV format:");
    println!("  id,prompt,completion,status");
    println!("  1,\"Write a...\",\"def reverse_string(s)...\",\"success\"");
    println!("  2,\"Implement...\",\"def binary_search(arr)...\",\"success\"");

    println!("\n");

    // ========================================================================
    // Example 4: JSON array input
    // ========================================================================
    println!("Example 4: JSON Array Input");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let json_batch = BatchProcess::new(");
    println!("      config.clone(),");
    println!("      \"phi-2.gguf\".to_string(),");
    println!("      PathBuf::from(\"questions.json\"),");
    println!("      Some(PathBuf::from(\"answers.json\")),");
    println!("      BatchOutputFormat::Json,  // Array output");
    println!("      512,");
    println!("      0.7,");
    println!("      0.9,");
    println!("      4,");
    println!("      300,");
    println!("      3,");
    println!("      100,");
    println!("      false,");
    println!("      false,");
    println!("      false,");
    println!("      None,");
    println!("      false,");
    println!("      None,");
    println!("  );");
    println!();
    println!("Input JSON format:");
    println!("  [");
    println!("    {{\"id\": 1, \"content\": \"What is Rust?\"}},");
    println!("    {{\"id\": 2, \"content\": \"Explain async/await\"}}");
    println!("  ]");
    println!();
    println!("Output JSON format:");
    println!("  [");
    println!("    {{");
    println!("      \"id\": 1,");
    println!("      \"input\": \"What is Rust?\",");
    println!("      \"output\": \"Rust is a systems programming language...\",");
    println!("      \"status\": \"success\"");
    println!("    }}");
    println!("  ]");

    println!("\n");

    // ========================================================================
    // Example 5: Resume from checkpoint
    // ========================================================================
    println!("Example 5: Resume from Checkpoint");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let resume_batch = BatchProcess::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b-chat.gguf\".to_string(),");
    println!("      PathBuf::from(\"large_inputs.jsonl\"),");
    println!("      Some(PathBuf::from(\"outputs.jsonl\")),");
    println!("      BatchOutputFormat::JsonLines,");
    println!("      512,");
    println!("      0.7,");
    println!("      0.9,");
    println!("      4,");
    println!("      300,");
    println!("      3,");
    println!("      100,");
    println!("      true,   // continue on error");
    println!("      false,");
    println!("      true,   // metrics enabled");
    println!("      Some(PathBuf::from(\".checkpoint_500.json\")),  // Resume point");
    println!("      false,");
    println!("      None,");
    println!("  );");
    println!();
    println!("Scenario:");
    println!("  - Previous run processed 500/10000 items and crashed");
    println!("  - Checkpoint saved at item 500");
    println!("  - Resume picks up from item 501");
    println!("  - No duplicate processing");
    println!("  - Full recovery from interruption");

    println!("\n");

    // ========================================================================
    // Example 6: Dry run validation
    // ========================================================================
    println!("Example 6: Dry Run Validation");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let dry_run = BatchProcess::new(");
    println!("      config.clone(),");
    println!("      \"model.gguf\".to_string(),");
    println!("      PathBuf::from(\"test_inputs.jsonl\"),");
    println!("      None,");
    println!("      BatchOutputFormat::JsonLines,");
    println!("      512, 0.7, 0.9, 4, 300, 3, 100,");
    println!("      false, false, false, None,");
    println!("      true,   // dry_run = true");
    println!("      None,");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  Validating batch inputs (dry run mode)");
    println!("  ✓ Successfully parsed 100 inputs from test_inputs.jsonl");
    println!("  Sample inputs:");
    println!("    1: What is the capital of France? (...)");
    println!("    2: Explain quantum computing (...)");
    println!("    3: Write a poem about spring (...)");
    println!("    ... and 97 more");
    println!("  ✓ Batch validation complete - ready for processing");
    println!();
    println!("Use cases:");
    println!("  - Validate input file format");
    println!("  - Check parsing without running inference");
    println!("  - Preview input samples");
    println!("  - Test configuration before long runs");

    println!("\n");

    // ========================================================================
    // Example 7: Continue on error
    // ========================================================================
    println!("Example 7: Continue on Error");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let error_tolerant = BatchProcess::new(");
    println!("      config.clone(),");
    println!("      \"model.gguf\".to_string(),");
    println!("      PathBuf::from(\"mixed_inputs.jsonl\"),");
    println!("      Some(PathBuf::from(\"results.jsonl\")),");
    println!("      BatchOutputFormat::JsonLines,");
    println!("      512, 0.7, 0.9, 4, 300, 3, 100,");
    println!("      true,   // continue_on_error = true");
    println!("      false, false, None, false, None,");
    println!("  );");
    println!();
    println!("Behavior:");
    println!("  - Individual item failures don't stop processing");
    println!("  - Failed items included in output with error details");
    println!("  - Full batch completes regardless of errors");
    println!("  - Error summary provided at end");
    println!();
    println!("Example output entry for failed item:");
    println!("  {{");
    println!("    \"id\": 42,");
    println!("    \"input\": \"problematic input\",");
    println!("    \"status\": \"failed\",");
    println!("    \"error\": \"Timeout exceeded\"");
    println!("  }}");

    println!("\n");

    // ========================================================================
    // Example 8: Validation examples
    // ========================================================================
    println!("Example 8: Input Validation");
    println!("{}", "─".repeat(80));

    // Test empty model
    let invalid_model = BatchProcess::new(
        config.clone(),
        String::new(),
        PathBuf::from("test.json"),
        None,
        BatchOutputFormat::JsonLines,
        512,
        0.7,
        0.9,
        4,
        300,
        3,
        100,
        false,
        false,
        false,
        None,
        false,
        None,
    );
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(invalid_model), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught empty model name:");
            println!("  {}", e);
        }
    }

    println!();

    // Test invalid temperature
    let invalid_temp = BatchProcess::new(
        config.clone(),
        "model.gguf".to_string(),
        PathBuf::from("test.json"),
        None,
        BatchOutputFormat::JsonLines,
        512,
        3.0, // Invalid
        0.9,
        4,
        300,
        3,
        100,
        false,
        false,
        false,
        None,
        false,
        None,
    );

    match pipeline
        .execute(Box::new(invalid_temp), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught invalid temperature:");
            println!("  {}", e);
        }
    }

    println!();

    // Test excessive concurrency
    let excessive_concurrency = BatchProcess::new(
        config.clone(),
        "model.gguf".to_string(),
        PathBuf::from("test.json"),
        None,
        BatchOutputFormat::JsonLines,
        512,
        0.7,
        0.9,
        200, // Excessive
        300,
        3,
        100,
        false,
        false,
        false,
        None,
        false,
        None,
    );

    match pipeline
        .execute(Box::new(excessive_concurrency), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught excessive concurrency:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 9: JSON output mode
    // ========================================================================
    println!("Example 9: JSON Output Mode");
    println!("{}", "─".repeat(80));
    println!("With json_output=true, structured data is returned:");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "input_file": "inputs.jsonl",
            "output_file": "outputs.jsonl",
            "model": "llama-2-7b-chat.gguf",
            "total_items": 1000,
            "completed_items": 998,
            "failed_items": 2,
            "skipped_items": 0,
            "success_rate": 99.8,
            "average_rate": 3.67,
            "duration_seconds": 272.5,
            "config": {
                "concurrency": 4,
                "max_tokens": 512,
                "temperature": 0.7,
                "top_p": 0.9,
                "continue_on_error": false
            }
        }))?
    );

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Batch Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Process multiple inputs in batch mode");
    println!("✓ Multiple input formats (JSON, JSONL, CSV, TSV)");
    println!("✓ Multiple output formats (JSON, JSONL, CSV, TSV)");
    println!("✓ Configurable concurrency (1-128 parallel requests)");
    println!("✓ Automatic checkpointing for recovery");
    println!("✓ Resume from checkpoint after interruption");
    println!("✓ Retry failed requests with exponential backoff");
    println!("✓ Continue on error option");
    println!("✓ Input shuffling for load balancing");
    println!("✓ Metrics collection and reporting");
    println!("✓ Dry run validation");
    println!("✓ Progress tracking and rate estimation");
    println!("✓ Comprehensive parameter validation");
    println!("✓ Structured JSON output");
    println!("✓ Human-readable summaries");
    println!("✓ Middleware support (logging, metrics)");
    println!();
    println!("Validation Checks:");
    println!("  - Model name not empty");
    println!("  - Input file exists");
    println!("  - Output directory exists");
    println!("  - Max tokens range (1-32768)");
    println!("  - Temperature range (0.0-2.0)");
    println!("  - Top-p range (0.0-1.0)");
    println!("  - Concurrency range (1-128)");
    println!("  - Timeout at least 1 second");
    println!("  - Checkpoint interval at least 1");
    println!();
    println!("Performance Features:");
    println!("  - Parallel processing with configurable concurrency");
    println!("  - Load balancing via input shuffling");
    println!("  - Automatic retry with backoff");
    println!("  - Checkpointing every N items");
    println!("  - Resume capability");
    println!("  - Progress estimation");
    println!();
    println!("Use Cases:");
    println!("  - Bulk inference on large datasets");
    println!("  - Data annotation and labeling");
    println!("  - Text classification at scale");
    println!("  - Code generation pipelines");
    println!("  - Question answering systems");
    println!("  - Content generation workflows");
    println!("  - ETL data enrichment");
    println!("  - Model evaluation on test sets");

    Ok(())
}
