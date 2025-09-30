//! Observability Command v2 Example
//!
//! Demonstrates observability stack management for metrics, tracing, and dashboards.
//!
//! Run with: cargo run --example observability_v2_example

use anyhow::Result;
use inferno::cli::observability_v2::{
    ObservabilityExport, ObservabilityInit, ObservabilityMetrics, ObservabilityStatus,
    ObservabilityTracing,
};
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

    println!("🔥 Inferno Observability Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Show observability status
    // ========================================================================
    println!("Example 1: Show Observability Status");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let status = ObservabilityStatus::new(config.clone());");
    println!();
    println!("Output:");
    println!("  === Observability Status ===");
    println!("  Prometheus: ✓ Enabled");
    println!("  OpenTelemetry: ✗ Disabled");
    println!("  Grafana: ✗ Disabled");
    println!("  ");
    println!("  Metrics Collected: 1234");
    println!("  Traces Recorded: 567");

    println!("\n");

    // ========================================================================
    // Example 2: Initialize with Prometheus only
    // ========================================================================
    println!("Example 2: Initialize with Prometheus Only");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let init = ObservabilityInit::new(");
    println!("      config.clone(),");
    println!("      true,     // prometheus");
    println!("      false,    // otel");
    println!("      false,    // grafana");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Initializing Observability Stack ===");
    println!("  ✓ Prometheus metrics enabled");

    println!("\n");

    // ========================================================================
    // Example 3: Initialize full stack
    // ========================================================================
    println!("Example 3: Initialize Full Stack");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let init = ObservabilityInit::new(");
    println!("      config.clone(),");
    println!("      true,     // prometheus");
    println!("      true,     // otel");
    println!("      true,     // grafana");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Initializing Observability Stack ===");
    println!("  ✓ Prometheus metrics enabled");
    println!("  ✓ OpenTelemetry tracing enabled");
    println!("  ✓ Grafana dashboards enabled");

    println!("\n");

    // ========================================================================
    // Example 4: Show all metrics
    // ========================================================================
    println!("Example 4: Show All Metrics");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let metrics = ObservabilityMetrics::new(");
    println!("      config.clone(),");
    println!("      None,     // no filter");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Current Metrics ===");
    println!("  ");
    println!("  inference_requests_total: 1234");
    println!("  inference_latency_ms: 234");
    println!("  cache_hits_total: 567");
    println!("  cache_misses_total: 123");

    println!("\n");

    // ========================================================================
    // Example 5: Filter metrics
    // ========================================================================
    println!("Example 5: Filter Metrics");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let metrics = ObservabilityMetrics::new(");
    println!("      config.clone(),");
    println!("      Some(\"cache_*\".to_string()),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Current Metrics ===");
    println!("  Filter: cache_*");
    println!("  ");
    println!("  cache_hits_total: 567");
    println!("  cache_misses_total: 123");

    println!("\n");

    // ========================================================================
    // Example 6: Enable tracing
    // ========================================================================
    println!("Example 6: Enable Distributed Tracing");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let tracing = ObservabilityTracing::new(");
    println!("      config.clone(),");
    println!("      true,     // enabled");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Distributed Tracing ===");
    println!("  Status: Enabled");
    println!("  Backend: OpenTelemetry");
    println!("  Traces Recorded: 567");

    println!("\n");

    // ========================================================================
    // Example 7: Export metrics
    // ========================================================================
    println!("Example 7: Export Metrics");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let export = ObservabilityExport::new(");
    println!("      config.clone(),");
    println!("      Some(PathBuf::from(\"/tmp/metrics.json\")),");
    println!("      None,");
    println!("      \"json\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Exporting Observability Data ===");
    println!("  Format: json");
    println!("  Metrics: \"/tmp/metrics.json\"");

    println!("\n");

    // ========================================================================
    // Example 8: Export both metrics and traces
    // ========================================================================
    println!("Example 8: Export Metrics and Traces");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let export = ObservabilityExport::new(");
    println!("      config.clone(),");
    println!("      Some(PathBuf::from(\"/tmp/metrics.json\")),");
    println!("      Some(PathBuf::from(\"/tmp/traces.json\")),");
    println!("      \"json\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Exporting Observability Data ===");
    println!("  Format: json");
    println!("  Metrics: \"/tmp/metrics.json\"");
    println!("  Traces: \"/tmp/traces.json\"");

    println!("\n");

    // ========================================================================
    // Example 9: Validation tests
    // ========================================================================
    println!("Example 9: Input Validation");
    println!("{}", "─".repeat(80));

    let no_targets = ObservabilityExport::new(config.clone(), None, None, "json".to_string());
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(no_targets), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught missing export targets:");
            println!("  {}", e);
        }
    }

    println!();

    let invalid_format = ObservabilityExport::new(
        config.clone(),
        Some(PathBuf::from("/tmp/metrics.json")),
        None,
        "invalid".to_string(),
    );

    match pipeline
        .execute(Box::new(invalid_format), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught invalid format:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Observability Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Stack status monitoring");
    println!("✓ Prometheus metrics integration");
    println!("✓ OpenTelemetry tracing support");
    println!("✓ Grafana dashboard management");
    println!("✓ Metrics filtering and querying");
    println!("✓ Data export (JSON, Prometheus, OTLP)");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - At least one export target required");
    println!("  - Format must be: json, prometheus, or otlp");
    println!();
    println!("Use Cases:");
    println!("  - Production monitoring");
    println!("  - Performance analysis");
    println!("  - Distributed tracing");
    println!("  - Dashboard creation");

    Ok(())
}
