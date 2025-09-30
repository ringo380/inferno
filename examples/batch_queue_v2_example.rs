//! Batch Queue Command v2 Example
//!
//! Demonstrates advanced batch processing with job queues and scheduling.
//!
//! Run with: cargo run --example batch_queue_v2_example

use anyhow::Result;
use inferno::cli::batch_queue_v2::*;
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

    println!("🔥 Inferno Batch Queue Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Create a job queue
    // ========================================================================
    println!("Example 1: Create a Job Queue");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let create = BatchQueueCreate::new(");
    println!("      config.clone(),");
    println!("      \"production-queue\".to_string(),");
    println!("      \"Production Batch Queue\".to_string(),");
    println!("      Some(\"High-priority production workloads\".to_string()),");
    println!("      4,     // max concurrent jobs");
    println!("      1000,  // max queue size");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Creating Job Queue ===");
    println!("  Queue ID: production-queue");
    println!("  Name: Production Batch Queue");
    println!("  Description: High-priority production workloads");
    println!("  Max Concurrent: 4");
    println!("  Max Size: 1000");
    println!("  ");
    println!("  ✓ Queue created successfully");

    println!("\n");

    // ========================================================================
    // Example 2: Submit a job
    // ========================================================================
    println!("Example 2: Submit a Job to Queue");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let submit = BatchQueueSubmit::new(");
    println!("      config.clone(),");
    println!("      \"production-queue\".to_string(),");
    println!("      \"Image Classification Batch\".to_string(),");
    println!("      PathBuf::from(\"/data/inputs.jsonl\"),");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      \"high\".to_string(),");
    println!("      Some(PathBuf::from(\"/data/outputs.jsonl\")),");
    println!("      2,      // concurrency");
    println!("      60,     // timeout minutes");
    println!("      3,      // max retries");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Submitting Job ===");
    println!("  Queue: production-queue");
    println!("  Job ID: job-abc123");
    println!("  Name: Image Classification Batch");
    println!("  Model: llama-2-7b");
    println!("  Priority: high");
    println!("  Input: \"/data/inputs.jsonl\"");
    println!("  Output: \"/data/outputs.jsonl\"");
    println!("  Concurrency: 2");
    println!("  Timeout: 60 minutes");
    println!("  Max Retries: 3");
    println!("  ");
    println!("  ✓ Job submitted successfully");
    println!("  Status: Queued");

    println!("\n");

    // ========================================================================
    // Example 3: List all queues
    // ========================================================================
    println!("Example 3: List All Queues");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = BatchQueueListQueues::new(");
    println!("      config.clone(),");
    println!("      false,    // not detailed");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Job Queues ===");
    println!("  ");
    println!("  Queue: queue-1");
    println!("    Name: Production Queue");
    println!("    Running: 4");
    println!("    Pending: 2");
    println!("  ");
    println!("  Queue: queue-2");
    println!("    Name: Staging Queue");
    println!("    Running: 2");
    println!("    Pending: 1");
    println!("  ");
    println!("  Total Queues: 2");

    println!("\n");

    // ========================================================================
    // Example 4: List queues with details
    // ========================================================================
    println!("Example 4: List Queues with Details");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = BatchQueueListQueues::new(");
    println!("      config.clone(),");
    println!("      true,     // detailed");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Job Queues ===");
    println!("  ");
    println!("  Queue: queue-1");
    println!("    Name: Production Queue");
    println!("    Running: 4");
    println!("    Pending: 2");
    println!("    Total Jobs: 1000");
    println!("    Status: Active");

    println!("\n");

    // ========================================================================
    // Example 5: List jobs in a queue
    // ========================================================================
    println!("Example 5: List Jobs in a Queue");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = BatchQueueListJobs::new(");
    println!("      config.clone(),");
    println!("      \"production-queue\".to_string(),");
    println!("      None,     // no status filter");
    println!("      100,      // limit");
    println!("      false,    // not detailed");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Jobs in Queue: production-queue ===");
    println!("  Limit: 100");
    println!("  ");
    println!("  Job: job-001");
    println!("    Name: Batch process 1");
    println!("    Status: running");
    println!("  ");
    println!("  Job: job-002");
    println!("    Name: Batch process 2");
    println!("    Status: queued");
    println!("  ");
    println!("  Total Jobs: 2");

    println!("\n");

    // ========================================================================
    // Example 6: Filter jobs by status
    // ========================================================================
    println!("Example 6: Filter Jobs by Status");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = BatchQueueListJobs::new(");
    println!("      config.clone(),");
    println!("      \"production-queue\".to_string(),");
    println!("      Some(\"running\".to_string()),");
    println!("      50,");
    println!("      true,     // detailed");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Jobs in Queue: production-queue ===");
    println!("  Filter: status=running");
    println!("  Limit: 50");
    println!("  ");
    println!("  Job: job-001");
    println!("    Name: Batch process 1");
    println!("    Status: running");
    println!("    Progress: 65%");
    println!("    Created: 2025-09-29T10:00:00Z");

    println!("\n");

    // ========================================================================
    // Example 7: Get job status
    // ========================================================================
    println!("Example 7: Get Job Status");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let status = BatchQueueJobStatus::new(");
    println!("      config.clone(),");
    println!("      \"production-queue\".to_string(),");
    println!("      \"job-001\".to_string(),");
    println!("      false,    // no progress");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Job Status ===");
    println!("  Queue: production-queue");
    println!("  Job ID: job-001");
    println!("  Status: running");
    println!("  ");
    println!("  Created: 2025-09-29T10:00:00Z");
    println!("  Started: 2025-09-29T10:01:00Z");
    println!("  Runtime: 15m 23s");

    println!("\n");

    // ========================================================================
    // Example 8: Get job status with progress
    // ========================================================================
    println!("Example 8: Get Job Status with Progress");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let status = BatchQueueJobStatus::new(");
    println!("      config.clone(),");
    println!("      \"production-queue\".to_string(),");
    println!("      \"job-001\".to_string(),");
    println!("      true,     // show progress");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Job Status ===");
    println!("  Queue: production-queue");
    println!("  Job ID: job-001");
    println!("  Status: running");
    println!("  ");
    println!("  Progress:");
    println!("    65% complete");
    println!("    650 / 1000 items processed");
    println!("    Estimated time remaining: 5m 30s");
    println!("  ");
    println!("  Created: 2025-09-29T10:00:00Z");
    println!("  Started: 2025-09-29T10:01:00Z");
    println!("  Runtime: 15m 23s");

    println!("\n");

    // ========================================================================
    // Example 9: Cancel a job
    // ========================================================================
    println!("Example 9: Cancel a Job");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let cancel = BatchQueueCancel::new(");
    println!("      config.clone(),");
    println!("      \"production-queue\".to_string(),");
    println!("      \"job-002\".to_string(),");
    println!("      false,    // graceful");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Cancelling Job ===");
    println!("  Queue: production-queue");
    println!("  Job ID: job-002");
    println!("  Mode: Graceful");
    println!("  ");
    println!("  Waiting for current task to complete...");
    println!("  ");
    println!("  ✓ Job cancelled successfully");

    println!("\n");

    // ========================================================================
    // Example 10: Force cancel a job
    // ========================================================================
    println!("Example 10: Force Cancel a Job");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let cancel = BatchQueueCancel::new(");
    println!("      config.clone(),");
    println!("      \"production-queue\".to_string(),");
    println!("      \"job-003\".to_string(),");
    println!("      true,     // force");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Cancelling Job ===");
    println!("  Queue: production-queue");
    println!("  Job ID: job-003");
    println!("  Mode: Force");
    println!("  ");
    println!("  ⚠️  Forcing immediate cancellation");
    println!("  ");
    println!("  ✓ Job cancelled successfully");

    println!("\n");

    // ========================================================================
    // Example 11: Get queue metrics
    // ========================================================================
    println!("Example 11: Get Queue Metrics");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let metrics = BatchQueueMetrics::new(");
    println!("      config.clone(),");
    println!("      Some(\"production-queue\".to_string()),");
    println!("      false,    // not historical");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Queue Metrics ===");
    println!("  Queue: production-queue");
    println!("  ");
    println!("  Total Jobs: 150");
    println!("  Completed: 120");
    println!("  Failed: 5");
    println!("  Running: 10");
    println!("  Queued: 15");
    println!("  ");
    println!("  Success Rate: 96.0%");
    println!("  Average Processing Time: 12.5 minutes");

    println!("\n");

    // ========================================================================
    // Example 12: Get historical metrics
    // ========================================================================
    println!("Example 12: Get Historical Metrics");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let metrics = BatchQueueMetrics::new(");
    println!("      config.clone(),");
    println!("      None,     // all queues");
    println!("      true,     // historical");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Queue Metrics ===");
    println!("  All Queues");
    println!("  ");
    println!("  Total Jobs: 150");
    println!("  Completed: 120");
    println!("  Failed: 5");
    println!("  Running: 10");
    println!("  Queued: 15");
    println!("  ");
    println!("  Success Rate: 96.0%");
    println!("  Average Processing Time: 12.5 minutes");
    println!("  ");
    println!("  Historical Trends:");
    println!("    Last Hour: 45 jobs");
    println!("    Last Day: 890 jobs");
    println!("    Last Week: 5234 jobs");

    println!("\n");

    // ========================================================================
    // Example 13: Validation tests
    // ========================================================================
    println!("Example 13: Input Validation");
    println!("{}", "─".repeat(80));

    let invalid_concurrent = BatchQueueCreate::new(
        config.clone(),
        "test-queue".to_string(),
        "Test".to_string(),
        None,
        0,
        1000,
    );
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(invalid_concurrent), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught invalid concurrent jobs:");
            println!("  {}", e);
        }
    }

    println!();

    let invalid_status = BatchQueueListJobs::new(
        config.clone(),
        "queue-1".to_string(),
        Some("invalid".to_string()),
        100,
        false,
    );

    match pipeline
        .execute(Box::new(invalid_status), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught invalid status filter:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Batch Queue Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Job queue creation and management");
    println!("✓ Job submission with priorities");
    println!("✓ Queue and job listing");
    println!("✓ Job status tracking with progress");
    println!("✓ Job cancellation (graceful and force)");
    println!("✓ Queue metrics and statistics");
    println!("✓ Historical trends");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Max concurrent jobs: 1-100");
    println!("  - Max queue size: 1-10000");
    println!("  - Job concurrency: 1-32");
    println!("  - Timeout: 1-1440 minutes (24 hours)");
    println!("  - Max retries: <= 10");
    println!("  - Status filter: queued, running, completed, failed, cancelled");
    println!("  - Job list limit: 1-1000");
    println!();
    println!("Use Cases:");
    println!("  - Large batch processing workloads");
    println!("  - Scheduled jobs and recurring tasks");
    println!("  - Priority-based job management");
    println!("  - Production monitoring and metrics");

    Ok(())
}
