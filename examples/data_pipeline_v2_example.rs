//! Data Pipeline Command v2 Example
//!
//! Demonstrates ETL data pipeline management for model training workflows.
//!
//! Run with: cargo run --example data_pipeline_v2_example

use anyhow::Result;
use inferno::cli::data_pipeline_v2::*;
use inferno::config::Config;
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

    println!("🔥 Inferno Data Pipeline Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Create a batch pipeline
    // ========================================================================
    println!("Example 1: Create a Batch Pipeline");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let create = DataPipelineCreate::new(");
    println!("      config.clone(),");
    println!("      \"training-pipeline\".to_string(),");
    println!("      Some(\"Model training ETL pipeline\".to_string()),");
    println!("      \"batch\".to_string(),");
    println!("      Some(PathBuf::from(\"/config/pipeline.yaml\")),");
    println!("      false,    // don't auto-start");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Creating Data Pipeline ===");
    println!("  Name: training-pipeline");
    println!("  Type: batch");
    println!("  Description: Model training ETL pipeline");
    println!("  Configuration: \"/config/pipeline.yaml\"");
    println!("  ");
    println!("  ✓ Pipeline created successfully");
    println!("  Pipeline ID: pipe-abc123");

    println!("\n");

    // ========================================================================
    // Example 2: Create a streaming pipeline
    // ========================================================================
    println!("Example 2: Create a Streaming Pipeline");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let create = DataPipelineCreate::new(");
    println!("      config.clone(),");
    println!("      \"realtime-inference\".to_string(),");
    println!("      Some(\"Real-time data processing\".to_string()),");
    println!("      \"streaming\".to_string(),");
    println!("      None,");
    println!("      true,     // auto-start");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Creating Data Pipeline ===");
    println!("  Name: realtime-inference");
    println!("  Type: streaming");
    println!("  Description: Real-time data processing");
    println!("  Auto-start: true");
    println!("  ");
    println!("  ✓ Pipeline created successfully");
    println!("  Pipeline ID: pipe-xyz789");
    println!("  ");
    println!("  Starting pipeline...");
    println!("  ✓ Pipeline started");

    println!("\n");

    // ========================================================================
    // Example 3: Create a hybrid pipeline
    // ========================================================================
    println!("Example 3: Create a Hybrid Pipeline");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let create = DataPipelineCreate::new(");
    println!("      config.clone(),");
    println!("      \"hybrid-etl\".to_string(),");
    println!("      Some(\"Batch + streaming hybrid\".to_string()),");
    println!("      \"hybrid\".to_string(),");
    println!("      Some(PathBuf::from(\"/config/hybrid.yaml\")),");
    println!("      false,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Creating Data Pipeline ===");
    println!("  Name: hybrid-etl");
    println!("  Type: hybrid");
    println!("  Description: Batch + streaming hybrid");
    println!("  Configuration: \"/config/hybrid.yaml\"");
    println!("  ");
    println!("  ✓ Pipeline created successfully");
    println!("  Pipeline ID: pipe-hybrid1");

    println!("\n");

    // ========================================================================
    // Example 4: List all pipelines
    // ========================================================================
    println!("Example 4: List All Pipelines");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = DataPipelineList::new(");
    println!("      config.clone(),");
    println!("      None,     // no status filter");
    println!("      None,     // no type filter");
    println!("      false,    // not detailed");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Data Pipelines ===");
    println!("  ");
    println!("  Pipeline: pipe-abc123");
    println!("    Name: training-pipeline");
    println!("    Type: batch");
    println!("    Status: stopped");
    println!("  ");
    println!("  Pipeline: pipe-xyz789");
    println!("    Name: realtime-inference");
    println!("    Type: streaming");
    println!("    Status: running");
    println!("  ");
    println!("  Pipeline: pipe-hybrid1");
    println!("    Name: hybrid-etl");
    println!("    Type: hybrid");
    println!("    Status: stopped");
    println!("  ");
    println!("  Total Pipelines: 3");

    println!("\n");

    // ========================================================================
    // Example 5: List with status filter
    // ========================================================================
    println!("Example 5: List Running Pipelines");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = DataPipelineList::new(");
    println!("      config.clone(),");
    println!("      Some(\"running\".to_string()),");
    println!("      None,");
    println!("      false,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Data Pipelines ===");
    println!("  Filter: status=running");
    println!("  ");
    println!("  Pipeline: pipe-xyz789");
    println!("    Name: realtime-inference");
    println!("    Type: streaming");
    println!("    Status: running");
    println!("  ");
    println!("  Total Pipelines: 1");

    println!("\n");

    // ========================================================================
    // Example 6: List with type filter
    // ========================================================================
    println!("Example 6: List Batch Pipelines");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = DataPipelineList::new(");
    println!("      config.clone(),");
    println!("      None,");
    println!("      Some(\"batch\".to_string()),");
    println!("      true,     // detailed");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Data Pipelines ===");
    println!("  Filter: type=batch");
    println!("  ");
    println!("  Pipeline: pipe-abc123");
    println!("    Name: training-pipeline");
    println!("    Type: batch");
    println!("    Status: stopped");
    println!("    Created: 2025-09-29T10:00:00Z");
    println!("    Last Run: 2025-09-28T15:30:00Z");
    println!("    Total Runs: 45");
    println!("  ");
    println!("  Total Pipelines: 1");

    println!("\n");

    // ========================================================================
    // Example 7: Start a pipeline
    // ========================================================================
    println!("Example 7: Start a Pipeline");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let start = DataPipelineStart::new(");
    println!("      config.clone(),");
    println!("      \"pipe-abc123\".to_string(),");
    println!("      false,    // don't wait for completion");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Starting Data Pipeline ===");
    println!("  Pipeline ID: pipe-abc123");
    println!("  Name: training-pipeline");
    println!("  ");
    println!("  ✓ Pipeline started successfully");
    println!("  Status: running");

    println!("\n");

    // ========================================================================
    // Example 8: Start with wait
    // ========================================================================
    println!("Example 8: Start Pipeline and Wait");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let start = DataPipelineStart::new(");
    println!("      config.clone(),");
    println!("      \"pipe-abc123\".to_string(),");
    println!("      true,     // wait for completion");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Starting Data Pipeline ===");
    println!("  Pipeline ID: pipe-abc123");
    println!("  Name: training-pipeline");
    println!("  ");
    println!("  ✓ Pipeline started successfully");
    println!("  ");
    println!("  Waiting for completion...");
    println!("  Progress: [████████████████████] 100%");
    println!("  ");
    println!("  ✓ Pipeline completed successfully");
    println!("  Duration: 5m 23s");

    println!("\n");

    // ========================================================================
    // Example 9: Get pipeline status
    // ========================================================================
    println!("Example 9: Get Pipeline Status");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let status = DataPipelineStatus::new(");
    println!("      config.clone(),");
    println!("      \"pipe-abc123\".to_string(),");
    println!("      false,    // no metrics");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Pipeline Status ===");
    println!("  Pipeline ID: pipe-abc123");
    println!("  Name: training-pipeline");
    println!("  Type: batch");
    println!("  Status: running");
    println!("  ");
    println!("  Current Run:");
    println!("    Started: 2025-09-29T14:30:00Z");
    println!("    Progress: 65%");
    println!("    Current Stage: transformation");

    println!("\n");

    // ========================================================================
    // Example 10: Get status with metrics
    // ========================================================================
    println!("Example 10: Get Status with Metrics");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let status = DataPipelineStatus::new(");
    println!("      config.clone(),");
    println!("      \"pipe-abc123\".to_string(),");
    println!("      true,     // include metrics");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Pipeline Status ===");
    println!("  Pipeline ID: pipe-abc123");
    println!("  Name: training-pipeline");
    println!("  Type: batch");
    println!("  Status: running");
    println!("  ");
    println!("  Current Run:");
    println!("    Started: 2025-09-29T14:30:00Z");
    println!("    Progress: 65%");
    println!("    Current Stage: transformation");
    println!("  ");
    println!("  Metrics:");
    println!("    Records Processed: 650,000");
    println!("    Records Failed: 125");
    println!("    Throughput: 5,200 records/sec");
    println!("    Data Volume: 12.5 GB");

    println!("\n");

    // ========================================================================
    // Example 11: Stop pipeline gracefully
    // ========================================================================
    println!("Example 11: Stop Pipeline Gracefully");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let stop = DataPipelineStop::new(");
    println!("      config.clone(),");
    println!("      \"pipe-abc123\".to_string(),");
    println!("      false,    // graceful");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Stopping Data Pipeline ===");
    println!("  Pipeline ID: pipe-abc123");
    println!("  Mode: Graceful");
    println!("  ");
    println!("  Waiting for current stage to complete...");
    println!("  ");
    println!("  ✓ Pipeline stopped successfully");

    println!("\n");

    // ========================================================================
    // Example 12: Force stop pipeline
    // ========================================================================
    println!("Example 12: Force Stop Pipeline");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let stop = DataPipelineStop::new(");
    println!("      config.clone(),");
    println!("      \"pipe-xyz789\".to_string(),");
    println!("      true,     // force");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Stopping Data Pipeline ===");
    println!("  Pipeline ID: pipe-xyz789");
    println!("  Mode: Force");
    println!("  ");
    println!("  ⚠️  Forcing immediate stop");
    println!("  ");
    println!("  ✓ Pipeline stopped successfully");

    println!("\n");

    // ========================================================================
    // Example 13: Validate pipeline configuration
    // ========================================================================
    println!("Example 13: Validate Pipeline Configuration");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let validate = DataPipelineValidate::new(");
    println!("      config.clone(),");
    println!("      None,     // use config file");
    println!("      Some(PathBuf::from(\"/config/pipeline.yaml\")),");
    println!("      false,    // not strict");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Validating Pipeline Configuration ===");
    println!("  Configuration File: \"/config/pipeline.yaml\"");
    println!("  ");
    println!("  ✓ Configuration is valid");
    println!("  ");
    println!("  Validation Results:");
    println!("    - Stages: 4 (extract, transform, validate, load)");
    println!("    - Data sources: 2 valid");
    println!("    - Dependencies: All satisfied");

    println!("\n");

    // ========================================================================
    // Example 14: Strict validation
    // ========================================================================
    println!("Example 14: Strict Validation Mode");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let validate = DataPipelineValidate::new(");
    println!("      config.clone(),");
    println!("      Some(\"pipe-abc123\".to_string()),");
    println!("      None,");
    println!("      true,     // strict mode");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Validating Pipeline Configuration ===");
    println!("  Pipeline ID: pipe-abc123");
    println!("  Mode: Strict");
    println!("  ");
    println!("  ⚠️  Validation warnings:");
    println!("    - Stage 'transform': Missing error handling");
    println!("    - Stage 'load': No retry configuration");
    println!("  ");
    println!("  ✗ Strict validation failed");

    println!("\n");

    // ========================================================================
    // Example 15: Get pipeline metrics
    // ========================================================================
    println!("Example 15: Get Pipeline Metrics");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let metrics = DataPipelineMetrics::new(");
    println!("      config.clone(),");
    println!("      \"pipe-abc123\".to_string(),");
    println!("      24,       // last 24 hours");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Pipeline Metrics ===");
    println!("  Pipeline ID: pipe-abc123");
    println!("  Time Range: Last 24 hours");
    println!("  ");
    println!("  Execution Statistics:");
    println!("    Total Runs: 48");
    println!("    Successful: 45");
    println!("    Failed: 3");
    println!("    Success Rate: 93.75%");
    println!("  ");
    println!("  Performance:");
    println!("    Average Duration: 5m 42s");
    println!("    Average Throughput: 4,850 records/sec");
    println!("    Total Data Processed: 580 GB");

    println!("\n");

    // ========================================================================
    // Example 16: Extended time range metrics
    // ========================================================================
    println!("Example 16: Extended Metrics (30 days)");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let metrics = DataPipelineMetrics::new(");
    println!("      config.clone(),");
    println!("      \"pipe-abc123\".to_string(),");
    println!("      720,      // 30 days");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Pipeline Metrics ===");
    println!("  Pipeline ID: pipe-abc123");
    println!("  Time Range: Last 30 days");
    println!("  ");
    println!("  Execution Statistics:");
    println!("    Total Runs: 1,440");
    println!("    Successful: 1,398");
    println!("    Failed: 42");
    println!("    Success Rate: 97.08%");
    println!("  ");
    println!("  Performance:");
    println!("    Average Duration: 5m 38s");
    println!("    Average Throughput: 4,920 records/sec");
    println!("    Total Data Processed: 17.2 TB");
    println!("  ");
    println!("  Trends:");
    println!("    Performance: ▲ +5% (improving)");
    println!("    Reliability: ▲ +2% (improving)");

    println!("\n");

    // ========================================================================
    // Example 17: Validation tests
    // ========================================================================
    println!("Example 17: Input Validation");
    println!("{}", "─".repeat(80));

    let invalid_type = DataPipelineCreate::new(
        config.clone(),
        "test".to_string(),
        None,
        "invalid".to_string(),
        None,
        false,
    );
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(invalid_type), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught invalid pipeline type:");
            println!("  {}", e);
        }
    }

    println!();

    let invalid_time = DataPipelineMetrics::new(config.clone(), "pipe-1".to_string(), 800);

    match pipeline
        .execute(Box::new(invalid_time), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught excessive time range:");
            println!("  {}", e);
        }
    }

    println!();

    let no_target = DataPipelineValidate::new(config.clone(), None, None, false);

    match pipeline
        .execute(Box::new(no_target), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught missing validation target:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Data Pipeline Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Pipeline creation (batch/streaming/hybrid)");
    println!("✓ Pipeline listing with filters");
    println!("✓ Pipeline execution control");
    println!("✓ Status monitoring with metrics");
    println!("✓ Configuration validation (normal + strict)");
    println!("✓ Historical metrics and trends");
    println!("✓ Graceful and force stop");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Pipeline type: batch, streaming, hybrid");
    println!("  - Status filter: running, stopped, paused, failed, completed");
    println!("  - Type filter: batch, streaming, hybrid");
    println!("  - Metrics time range: 1-720 hours (30 days)");
    println!("  - Config file must exist if specified");
    println!();
    println!("Use Cases:");
    println!("  - ETL data processing for model training");
    println!("  - Real-time data streaming pipelines");
    println!("  - Hybrid batch + streaming workflows");
    println!("  - Data quality validation and monitoring");

    Ok(())
}