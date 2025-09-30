//! Metrics Command v2 Example
//!
//! Demonstrates the new CLI architecture for the metrics command.
//! Shows metrics export in JSON, Prometheus, and snapshot formats.
//!
//! Run with: cargo run --example metrics_v2_example

use anyhow::Result;
use inferno::cli::metrics_v2::{MetricsJson, MetricsPrometheus, MetricsSnapshot};
use inferno::config::Config;
use inferno::core::config::ConfigBuilder;
use inferno::interfaces::cli::{
    CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Metrics Command v2 Examples\n");

    // Create configuration
    let config = Config::default();
    let _core_config = ConfigBuilder::new().build_unchecked();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Export metrics as JSON
    // ========================================================================
    println!("Example 1: Export Metrics as JSON");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let json_cmd = MetricsJson::new(config.clone());");
    println!();
    println!("Expected output format:");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "inference": {
                "total_requests": 1523,
                "successful_requests": 1498,
                "failed_requests": 25,
                "total_tokens": 245678,
                "average_latency_ms": 342.5,
                "requests_per_second": 12.3
            },
            "models": {
                "llama-2-7b-chat": {
                    "requests": 845,
                    "tokens_generated": 134567,
                    "average_latency_ms": 298.2,
                    "cache_hits": 734,
                    "cache_misses": 111
                },
                "mistral-7b-instruct": {
                    "requests": 653,
                    "tokens_generated": 111111,
                    "average_latency_ms": 402.1,
                    "cache_hits": 589,
                    "cache_misses": 64
                }
            },
            "system": {
                "cpu_usage_percent": 45.2,
                "memory_usage_mb": 8456.3,
                "uptime_seconds": 86400
            }
        }))?
    );

    println!("\n");

    // ========================================================================
    // Example 2: Export metrics in Prometheus format
    // ========================================================================
    println!("Example 2: Export Metrics in Prometheus Format");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let prom_cmd = MetricsPrometheus::new(config.clone());");
    println!();
    println!("Expected output format (Prometheus text format):");
    println!("  # HELP inferno_inference_requests_total Total number of inference requests");
    println!("  # TYPE inferno_inference_requests_total counter");
    println!("  inferno_inference_requests_total 1523");
    println!();
    println!("  # HELP inferno_inference_latency_seconds Inference latency in seconds");
    println!("  # TYPE inferno_inference_latency_seconds histogram");
    println!(
        "  inferno_inference_latency_seconds_bucket{{le=\"0.1\"}} 234"
    );
    println!(
        "  inferno_inference_latency_seconds_bucket{{le=\"0.5\"}} 987"
    );
    println!(
        "  inferno_inference_latency_seconds_bucket{{le=\"1.0\"}} 1432"
    );
    println!(
        "  inferno_inference_latency_seconds_bucket{{le=\"+Inf\"}} 1523"
    );
    println!("  inferno_inference_latency_seconds_sum 521.234");
    println!("  inferno_inference_latency_seconds_count 1523");
    println!();
    println!(
        "  # HELP inferno_model_requests_total Total requests per model"
    );
    println!("  # TYPE inferno_model_requests_total counter");
    println!(
        "  inferno_model_requests_total{{model=\"llama-2-7b-chat\"}} 845"
    );
    println!(
        "  inferno_model_requests_total{{model=\"mistral-7b-instruct\"}} 653"
    );

    println!("\n");

    // ========================================================================
    // Example 3: Metrics snapshot (compact)
    // ========================================================================
    println!("Example 3: Metrics Snapshot (Compact)");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let snapshot_cmd = MetricsSnapshot::new(config.clone(), false);");
    println!();
    println!("Compact output (single line JSON):");
    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "inference": { "total": 1523, "success": 1498, "failed": 25 },
            "latency": { "avg_ms": 342.5, "p50_ms": 298.0, "p95_ms": 789.0, "p99_ms": 1234.0 },
            "system": { "cpu": 45.2, "memory_mb": 8456.3, "uptime": 86400 }
        }))?
    );

    println!("\n");

    // ========================================================================
    // Example 4: Metrics snapshot (pretty)
    // ========================================================================
    println!("Example 4: Metrics Snapshot (Pretty)");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let snapshot_pretty = MetricsSnapshot::new(config.clone(), true);");
    println!();
    println!("Pretty-printed output:");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "inference": {
                "total_requests": 1523,
                "successful_requests": 1498,
                "failed_requests": 25,
                "success_rate": 98.36
            },
            "latency": {
                "average_ms": 342.5,
                "median_ms": 298.0,
                "p95_ms": 789.0,
                "p99_ms": 1234.0,
                "min_ms": 87.2,
                "max_ms": 2341.8
            },
            "throughput": {
                "requests_per_second": 12.3,
                "tokens_per_second": 1987.4
            },
            "system": {
                "cpu_usage_percent": 45.2,
                "memory_usage_mb": 8456.3,
                "memory_total_mb": 16384.0,
                "uptime_seconds": 86400
            }
        }))?
    );

    println!("\n");

    // ========================================================================
    // Example 5: Run actual commands
    // ========================================================================
    println!("Example 5: Execute Actual Commands");
    println!("{}", "─".repeat(80));

    println!("Running MetricsSnapshot with pretty=true...");
    let snapshot_cmd = MetricsSnapshot::new(config.clone(), true);
    let mut ctx = CommandContext::new(config.clone());

    match pipeline.execute(Box::new(snapshot_cmd), &mut ctx).await {
        Ok(output) => {
            println!("✓ {}", output.message);
            if let Some(data) = output.data {
                println!("Snapshot captured at: {}", data["timestamp"].as_str().unwrap());
                println!("Pretty format: {}", data["pretty"]);
            }
        }
        Err(e) => {
            eprintln!("✗ Failed: {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 6: Use cases and integration
    // ========================================================================
    println!("Example 6: Integration Use Cases");
    println!("{}", "─".repeat(80));
    println!("Common integration scenarios:");
    println!();
    println!("1. Prometheus Monitoring:");
    println!("   inferno metrics prometheus | curl --data-binary @- http://pushgateway:9091/metrics/job/inferno");
    println!();
    println!("2. JSON Logging:");
    println!("   inferno metrics json >> /var/log/inferno-metrics.jsonl");
    println!();
    println!("3. Real-time Dashboard:");
    println!("   while true; do");
    println!("     inferno metrics snapshot --pretty");
    println!("     sleep 5");
    println!("   done");
    println!();
    println!("4. Alerting:");
    println!("   inferno metrics json | jq '.inference.failed_requests' | \\");
    println!("     xargs -I {{}} bash -c '[ {{}} -gt 100 ] && alert-script.sh'");
    println!();
    println!("5. CI/CD Integration:");
    println!("   inferno metrics json > metrics.json");
    println!("   # Upload to monitoring service");

    println!("\n");

    // ========================================================================
    // Example 7: Metrics categories
    // ========================================================================
    println!("Example 7: Available Metrics Categories");
    println!("{}", "─".repeat(80));
    println!("Inference Metrics:");
    println!("  - total_requests: Total inference requests");
    println!("  - successful_requests: Successfully completed requests");
    println!("  - failed_requests: Failed requests");
    println!("  - total_tokens: Total tokens generated");
    println!("  - requests_per_second: Throughput rate");
    println!();
    println!("Latency Metrics:");
    println!("  - average_latency_ms: Mean latency");
    println!("  - median_latency_ms: 50th percentile");
    println!("  - p95_latency_ms: 95th percentile");
    println!("  - p99_latency_ms: 99th percentile");
    println!("  - min_latency_ms: Minimum observed");
    println!("  - max_latency_ms: Maximum observed");
    println!();
    println!("Model Metrics (per model):");
    println!("  - requests: Request count");
    println!("  - tokens_generated: Token count");
    println!("  - average_latency_ms: Model-specific latency");
    println!("  - cache_hits: Cache hit count");
    println!("  - cache_misses: Cache miss count");
    println!();
    println!("System Metrics:");
    println!("  - cpu_usage_percent: CPU utilization");
    println!("  - memory_usage_mb: Memory usage");
    println!("  - memory_total_mb: Total memory");
    println!("  - uptime_seconds: System uptime");
    println!();
    println!("Cache Metrics:");
    println!("  - hit_rate: Cache hit ratio");
    println!("  - eviction_count: Total evictions");
    println!("  - total_models: Cached models");

    println!("\n");

    // ========================================================================
    // Example 8: Format comparison
    // ========================================================================
    println!("Example 8: Format Comparison");
    println!("{}", "─".repeat(80));
    println!("JSON Format:");
    println!("  ✓ Easy to parse programmatically");
    println!("  ✓ Hierarchical structure");
    println!("  ✓ Rich data types (strings, numbers, arrays)");
    println!("  ✓ Perfect for logging and storage");
    println!("  ✓ Compatible with most monitoring tools");
    println!();
    println!("Prometheus Format:");
    println!("  ✓ Industry-standard monitoring format");
    println!("  ✓ Direct Prometheus integration");
    println!("  ✓ Efficient for time-series data");
    println!("  ✓ Works with Grafana and other tools");
    println!("  ✓ Labels for multi-dimensional queries");
    println!();
    println!("Snapshot Format:");
    println!("  ✓ Point-in-time metrics capture");
    println!("  ✓ Detailed breakdown of all metrics");
    println!("  ✓ Pretty-print for human readability");
    println!("  ✓ Includes timestamp for tracking");
    println!("  ✓ Ideal for debugging and analysis");

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Metrics Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Export metrics in JSON format");
    println!("✓ Export metrics in Prometheus format");
    println!("✓ Create detailed metrics snapshots");
    println!("✓ Pretty-print option for readability");
    println!("✓ Structured output for automation");
    println!("✓ Timestamp tracking");
    println!("✓ Multiple metric categories (inference, latency, system, cache)");
    println!("✓ Per-model metrics tracking");
    println!("✓ Integration-ready formats");
    println!("✓ Middleware support (logging, metrics)");
    println!();
    println!("Metric Categories:");
    println!("  - Inference: Requests, success/failure rates, throughput");
    println!("  - Latency: Average, percentiles (p50, p95, p99), min/max");
    println!("  - Models: Per-model statistics and cache performance");
    println!("  - System: CPU, memory, uptime");
    println!("  - Cache: Hit rates, evictions, model count");
    println!();
    println!("Export Formats:");
    println!("  - JSON: Structured data for logging and storage");
    println!("  - Prometheus: Time-series monitoring format");
    println!("  - Snapshot: Point-in-time detailed metrics");
    println!();
    println!("Use Cases:");
    println!("  - Monitor application performance");
    println!("  - Integrate with Prometheus/Grafana");
    println!("  - Log metrics for historical analysis");
    println!("  - Alert on performance degradation");
    println!("  - Track model usage and efficiency");
    println!("  - Debug performance issues");
    println!();
    println!("Note: This is a focused migration covering core metrics export.");
    println!("Full metrics functionality (HTTP server) remains available through");
    println!("the original metrics module.");

    Ok(())
}