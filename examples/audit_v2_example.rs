//! Audit Command v2 Example
//!
//! Demonstrates the new CLI architecture for audit operations.
//! Shows query, statistics, export, and configuration commands.
//!
//! Run with: cargo run --example audit_v2_example

use anyhow::Result;
use chrono::{Duration, Utc};
use inferno::audit::{EventType, ExportFormat, Severity, SortField, SortOrder};
use inferno::cli::audit_v2::{AuditConfigure, AuditExport, AuditQueryCmd, AuditStats};
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

    println!("🔥 Inferno Audit Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Basic audit query
    // ========================================================================
    println!("Example 1: Basic Audit Query");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let query = AuditQueryCmd::new(");
    println!("      config.clone(),");
    println!("      None,                    // event_types");
    println!("      None,                    // severities");
    println!("      None,                    // actors");
    println!("      None,                    // resources");
    println!("      None,                    // start_time");
    println!("      None,                    // end_time");
    println!("      100,                     // limit");
    println!("      0,                       // offset");
    println!("      SortField::Timestamp,");
    println!("      SortOrder::Descending,");
    println!("      None,                    // search");
    println!("  );");
    println!();
    println!("Output:");
    println!("  Found 42 audit events");
    println!();
    println!("  TIMESTAMP            EVENT_TYPE      SEVERITY   ACTOR                DESCRIPTION");
    println!("  {}", "-".repeat(95));
    println!("  2025-09-29 10:30:15  ModelLoaded     Info       system               Model llama-2-7b loaded succ...");
    println!("  2025-09-29 10:29:42  InferenceStart  Info       user-123             Inference started with max_t...");
    println!("  2025-09-29 10:29:30  ConfigChanged   Warning    admin-456            Configuration updated: backe...");

    println!("\n");

    // ========================================================================
    // Example 2: Filtered query by event type and severity
    // ========================================================================
    println!("Example 2: Filtered Query by Event Type and Severity");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let filtered_query = AuditQueryCmd::new(");
    println!("      config.clone(),");
    println!("      Some(vec![");
    println!("          EventType::ModelLoaded,");
    println!("          EventType::ModelUnloaded,");
    println!("      ]),");
    println!("      Some(vec![");
    println!("          Severity::Warning,");
    println!("          Severity::Error,");
    println!("      ]),");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("      50,");
    println!("      0,");
    println!("      SortField::Timestamp,");
    println!("      SortOrder::Descending,");
    println!("      None,");
    println!("  );");
    println!();
    println!("Use Cases:");
    println!("  - Find model loading errors");
    println!("  - Track configuration warnings");
    println!("  - Monitor security events");
    println!("  - Investigate authentication failures");

    println!("\n");

    // ========================================================================
    // Example 3: Time range query
    // ========================================================================
    println!("Example 3: Time Range Query");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let start_time = Utc::now() - Duration::hours(24);");
    println!("  let end_time = Utc::now();");
    println!("  let time_query = AuditQueryCmd::new(");
    println!("      config.clone(),");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("      Some(start_time),");
    println!("      Some(end_time),");
    println!("      1000,");
    println!("      0,");
    println!("      SortField::Timestamp,");
    println!("      SortOrder::Ascending,");
    println!("      None,");
    println!("  );");
    println!();
    println!("Time Range Options:");
    println!("  - Last 24 hours: Utc::now() - Duration::hours(24)");
    println!("  - Last week: Utc::now() - Duration::days(7)");
    println!("  - Last month: Utc::now() - Duration::days(30)");
    println!("  - Custom range: Specify exact start and end times");

    println!("\n");

    // ========================================================================
    // Example 4: Search query
    // ========================================================================
    println!("Example 4: Search Query");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let search_query = AuditQueryCmd::new(");
    println!("      config.clone(),");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("      100,");
    println!("      0,");
    println!("      SortField::Timestamp,");
    println!("      SortOrder::Descending,");
    println!("      Some(\"GPU\".to_string()),");
    println!("  );");
    println!();
    println!("Search Examples:");
    println!("  - \"GPU\" - Find GPU-related events");
    println!("  - \"error\" - Find all error messages");
    println!("  - \"llama\" - Find events related to llama models");
    println!("  - \"authentication\" - Find auth events");

    println!("\n");

    // ========================================================================
    // Example 5: Audit statistics
    // ========================================================================
    println!("Example 5: Audit Statistics");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let stats = AuditStats::new(");
    println!("      config.clone(),");
    println!("      24,  // last 24 hours");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Audit Statistics ===");
    println!("  Time range: last 24 hours");
    println!("  Total events: 1,234");
    println!("  Average events/hour: 51.4");
    println!();
    println!("  Events by type:");
    println!("    InferenceStart: 456");
    println!("    InferenceComplete: 450");
    println!("    ModelLoaded: 89");
    println!("    ConfigChanged: 23");
    println!();
    println!("  Events by severity:");
    println!("    Info: 1,100");
    println!("    Warning: 120");
    println!("    Error: 14");

    println!("\n");

    // ========================================================================
    // Example 6: Export to JSON
    // ========================================================================
    println!("Example 6: Export to JSON");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let export_json = AuditExport::new(");
    println!("      config.clone(),");
    println!("      PathBuf::from(\"audit-export.json\"),");
    println!("      ExportFormat::Json,");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("      Some(5000),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ Exported 4,567 events to audit-export.json");
    println!("  Format: Json");
    println!();
    println!("Use Cases:");
    println!("  - Compliance reporting");
    println!("  - Log analysis tools");
    println!("  - Data archival");
    println!("  - Integration with SIEM systems");

    println!("\n");

    // ========================================================================
    // Example 7: Export with filters
    // ========================================================================
    println!("Example 7: Export with Filters");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let start_time = Utc::now() - Duration::days(7);");
    println!("  let filtered_export = AuditExport::new(");
    println!("      config.clone(),");
    println!("      PathBuf::from(\"errors-last-week.json\"),");
    println!("      ExportFormat::Json,");
    println!("      Some(vec![");
    println!("          EventType::Error,");
    println!("          EventType::SecurityViolation,");
    println!("      ]),");
    println!("      Some(start_time),");
    println!("      None,");
    println!("      None,");
    println!("  );");
    println!();
    println!("Benefits:");
    println!("  - Focus on specific event types");
    println!("  - Smaller export files");
    println!("  - Faster processing");
    println!("  - Targeted analysis");

    println!("\n");

    // ========================================================================
    // Example 8: Show configuration
    // ========================================================================
    println!("Example 8: Show Audit Configuration");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let configure = AuditConfigure::new(config.clone());");
    println!();
    println!("Output:");
    println!("  === Audit Configuration ===");
    println!("  Enabled: true");
    println!("  Storage path: /var/log/inferno/audit");
    println!("  Max file size: 100 MB");
    println!("  Max files: 10");
    println!("  Retention days: 90");
    println!("  Compression: true");
    println!("  Encryption: true");
    println!();
    println!("Configuration Sources:");
    println!("  1. CLI arguments (highest priority)");
    println!("  2. Environment variables");
    println!("  3. Config file");
    println!("  4. Defaults (lowest priority)");

    println!("\n");

    // ========================================================================
    // Example 9: Pagination
    // ========================================================================
    println!("Example 9: Pagination");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  // First page");
    println!("  let page1 = AuditQueryCmd::new(");
    println!("      config.clone(),");
    println!("      None, None, None, None, None, None,");
    println!("      100,  // limit");
    println!("      0,    // offset");
    println!("      SortField::Timestamp,");
    println!("      SortOrder::Descending,");
    println!("      None,");
    println!("  );");
    println!();
    println!("  // Second page");
    println!("  let page2 = AuditQueryCmd::new(");
    println!("      config.clone(),");
    println!("      None, None, None, None, None, None,");
    println!("      100,  // limit");
    println!("      100,  // offset");
    println!("      SortField::Timestamp,");
    println!("      SortOrder::Descending,");
    println!("      None,");
    println!("  );");

    println!("\n");

    // ========================================================================
    // Example 10: Validation tests
    // ========================================================================
    println!("Example 10: Input Validation");
    println!("{}", "─".repeat(80));

    let invalid_limit = AuditQueryCmd::new(
        config.clone(),
        None,
        None,
        None,
        None,
        None,
        None,
        0, // Invalid
        0,
        SortField::Timestamp,
        SortOrder::Descending,
        None,
    );
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(invalid_limit), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught zero limit:");
            println!("  {}", e);
        }
    }

    println!();

    let excessive_limit = AuditQueryCmd::new(
        config.clone(),
        None,
        None,
        None,
        None,
        None,
        None,
        20000, // Invalid
        0,
        SortField::Timestamp,
        SortOrder::Descending,
        None,
    );

    match pipeline
        .execute(Box::new(excessive_limit), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught excessive limit:");
            println!("  {}", e);
        }
    }

    println!();

    let invalid_time_range = AuditQueryCmd::new(
        config.clone(),
        None,
        None,
        None,
        None,
        Some(Utc::now()),
        Some(Utc::now() - Duration::hours(1)), // End before start
        100,
        0,
        SortField::Timestamp,
        SortOrder::Descending,
        None,
    );

    match pipeline
        .execute(Box::new(invalid_time_range), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught invalid time range:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Audit Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Query audit events with comprehensive filters");
    println!("✓ Filter by event type, severity, actor, resource");
    println!("✓ Time range filtering (start/end times)");
    println!("✓ Full-text search across event descriptions");
    println!("✓ Sorting and pagination support");
    println!("✓ Statistics generation (event counts by type/severity)");
    println!("✓ Export to multiple formats (JSON, CSV, etc.)");
    println!("✓ Configuration display");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Limit range (1-10000)");
    println!("  - Time range validation (end > start)");
    println!("  - Output directory validation");
    println!("  - Stats time range (1-8760 hours)");
    println!();
    println!("Note: Advanced features (Monitor, Tail, Search) remain available");
    println!("through the original audit module.");

    Ok(())
}
