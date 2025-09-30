//! Monitoring Command v2 Example
//!
//! Demonstrates real-time performance monitoring and alerting.
//!
//! Run with: cargo run --example monitoring_v2_example

use anyhow::Result;
use inferno::cli::monitoring_v2::{
    MonitoringAlerts, MonitoringConfigure, MonitoringReport, MonitoringStatus, MonitoringTrends,
};
use inferno::config::Config;
use inferno::interfaces::cli::{
    CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware,
};
use inferno::monitoring::AlertSeverity;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Monitoring Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Show monitoring status
    // ========================================================================
    println!("Example 1: Show Monitoring Status");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let status = MonitoringStatus::new(config.clone());");
    println!();
    println!("Output:");
    println!("  === Monitoring Status ===");
    println!("  Monitoring Active: true");
    println!("  Alert Count: 3");
    println!("  Total Metrics: 15432");
    println!("  Uptime: 2h 15m");
    println!("  ");
    println!("  ⚠️  Active Alerts:");
    println!("    - [High] Response time exceeded threshold");
    println!("    - [Medium] CPU usage above 80%");
    println!("    - [Low] Cache hit rate below target");

    println!("\n");

    // ========================================================================
    // Example 2: List active alerts
    // ========================================================================
    println!("Example 2: List Active Alerts");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let alerts = MonitoringAlerts::new(");
    println!("      config.clone(),");
    println!("      false,    // don't show resolved");
    println!("      None,     // all severities");
    println!("      20,       // limit");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Monitoring Alerts ===");
    println!("  Total Alerts: 3");
    println!("  ");
    println!("  Alert ID: alert-001");
    println!("    Severity: High");
    println!("    Message: Response time exceeded threshold");
    println!("    Timestamp: 2025-09-29T10:15:00Z");
    println!("    Status: Active");

    println!("\n");

    // ========================================================================
    // Example 3: Filter alerts by severity
    // ========================================================================
    println!("Example 3: Filter Alerts by Severity");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let alerts = MonitoringAlerts::new(");
    println!("      config.clone(),");
    println!("      false,");
    println!("      Some(AlertSeverity::Critical),");
    println!("      10,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Monitoring Alerts ===");
    println!("  Total Alerts: 1");
    println!("  ");
    println!("  Alert ID: alert-003");
    println!("    Severity: Critical");
    println!("    Message: System overload detected");
    println!("    Timestamp: 2025-09-29T10:20:00Z");
    println!("    Status: Active");

    println!("\n");

    // ========================================================================
    // Example 4: Configure monitoring thresholds
    // ========================================================================
    println!("Example 4: Configure Monitoring Thresholds");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let configure = MonitoringConfigure::new(");
    println!("      config.clone(),");
    println!("      Some(1000),   // max response time (ms)");
    println!("      Some(10.0),   // min throughput");
    println!("      Some(5.0),    // max error rate");
    println!("      Some(4096),   // max memory (MB)");
    println!("      Some(80.0),   // max CPU");
    println!("      Some(70.0),   // min cache hit rate");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Monitoring Configuration ===");
    println!("  Max Response Time: 1000ms");
    println!("  Min Throughput: 10 req/s");
    println!("  Max Error Rate: 5%");
    println!("  Max Memory: 4096 MB");
    println!("  Max CPU: 80%");
    println!("  Min Cache Hit Rate: 70%");

    println!("\n");

    // ========================================================================
    // Example 5: Show performance trends
    // ========================================================================
    println!("Example 5: Show Performance Trends");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let trends = MonitoringTrends::new(");
    println!("      config.clone(),");
    println!("      24,       // last 24 hours");
    println!("      None,     // all models");
    println!("      5,        // group by 5 minutes");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Performance Trends ===");
    println!("  Time Period: 24 hours");
    println!("  Group By: 5 minutes");
    println!("  ");
    println!("  Trend Summary:");
    println!("    Average Response Time: 234ms");
    println!("    Average Throughput: 12.34 req/s");
    println!("    Error Rate: 1.23%");
    println!("    Peak Requests: 150");

    println!("\n");

    // ========================================================================
    // Example 6: Trends for specific model
    // ========================================================================
    println!("Example 6: Trends for Specific Model");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let trends = MonitoringTrends::new(");
    println!("      config.clone(),");
    println!("      12,       // last 12 hours");
    println!("      Some(\"llama-2-7b\".to_string()),");
    println!("      10,       // group by 10 minutes");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Performance Trends ===");
    println!("  Time Period: 12 hours");
    println!("  Model: llama-2-7b");
    println!("  Group By: 10 minutes");
    println!("  ");
    println!("  Trend Summary:");
    println!("    Average Response Time: 187ms");
    println!("    Average Throughput: 15.67 req/s");
    println!("    Error Rate: 0.89%");
    println!("    Peak Requests: 120");

    println!("\n");

    // ========================================================================
    // Example 7: Generate basic report
    // ========================================================================
    println!("Example 7: Generate Basic Report");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let report = MonitoringReport::new(");
    println!("      config.clone(),");
    println!("      24,       // last 24 hours");
    println!("      false,    // not detailed");
    println!("      false,    // no recommendations");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Performance Report ===");
    println!("  Report Period: 24 hours");
    println!("  ");
    println!("  Summary:");
    println!("    Total Requests: 10543");
    println!("    Successful Requests: 10412");
    println!("    Failed Requests: 131");
    println!("    Success Rate: 98.76%");
    println!("    Average Response Time: 245ms");
    println!("    Average Throughput: 12.15 req/s");

    println!("\n");

    // ========================================================================
    // Example 8: Detailed report with recommendations
    // ========================================================================
    println!("Example 8: Detailed Report with Recommendations");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let report = MonitoringReport::new(");
    println!("      config.clone(),");
    println!("      72,       // last 72 hours");
    println!("      true,     // detailed");
    println!("      true,     // with recommendations");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Performance Report ===");
    println!("  Report Period: 72 hours");
    println!("  ");
    println!("  Summary:");
    println!("    Total Requests: 31629");
    println!("    Successful Requests: 31234");
    println!("    Failed Requests: 395");
    println!("    Success Rate: 98.75%");
    println!("    Average Response Time: 253ms");
    println!("    Average Throughput: 12.23 req/s");
    println!("  ");
    println!("  Detailed Breakdown:");
    println!("    Peak Hour: 2025-09-28 14:00");
    println!("    Memory Usage: 2048 MB");
    println!("    CPU Usage: 65.43%");
    println!("  ");
    println!("  Recommendations:");
    println!("    • Consider increasing worker count during peak hours");
    println!("    • Enable response caching to improve throughput");
    println!("    • Optimize model loading for faster response times");

    println!("\n");

    // ========================================================================
    // Example 9: Validation tests
    // ========================================================================
    println!("Example 9: Input Validation");
    println!("{}", "─".repeat(80));

    let zero_limit = MonitoringAlerts::new(config.clone(), false, None, 0);
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(zero_limit), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught zero limit:");
            println!("  {}", e);
        }
    }

    println!();

    let invalid_error_rate = MonitoringConfigure::new(
        config.clone(),
        None,
        None,
        Some(150.0),
        None,
        None,
        None,
    );

    match pipeline
        .execute(Box::new(invalid_error_rate), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught invalid error rate:");
            println!("  {}", e);
        }
    }

    println!();

    let too_long_trends = MonitoringTrends::new(config.clone(), 200, None, 5);

    match pipeline
        .execute(Box::new(too_long_trends), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught excessive time period:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Monitoring Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Real-time monitoring status");
    println!("✓ Alert management and filtering");
    println!("✓ Configurable thresholds");
    println!("✓ Performance trend analysis");
    println!("✓ Automated report generation");
    println!("✓ Detailed breakdowns and recommendations");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Alert limit > 0 and <= 1000");
    println!("  - Error rate, CPU, cache hit rate: 0.0-100.0%");
    println!("  - Trend hours > 0 and <= 168 (1 week)");
    println!("  - Report hours > 0 and <= 720 (30 days)");
    println!("  - Group by minutes > 0 and <= 1440 (1 day)");
    println!();
    println!("Use Cases:");
    println!("  - Real-time system health monitoring");
    println!("  - Performance optimization");
    println!("  - Alert management");
    println!("  - Capacity planning");

    Ok(())
}