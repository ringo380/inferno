//! Serve Command v2 Example
//!
//! Demonstrates HTTP API server management for local inference endpoints.
//!
//! Run with: cargo run --example serve_v2_example

use anyhow::Result;
use inferno::cli::serve_v2::*;
use inferno::config::Config;
use inferno::interfaces::cli::{
    CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Serve Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Start basic HTTP server
    // ========================================================================
    println!("Example 1: Start Basic HTTP Server");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let start = ServeStart::new(");
    println!("      config.clone(),");
    println!("      \"127.0.0.1:8080\".parse().unwrap(),");
    println!("      None,     // no startup model");
    println!("      false,    // not distributed");
    println!("      0,        // auto workers");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Starting HTTP API Server ===");
    println!("  Bind Address: 127.0.0.1:8080");
    println!("  Mode: Single-process");
    println!("  ");
    println!("  Available Endpoints:");
    println!("    GET  /              - Server information");
    println!("    GET  /health        - Health check");
    println!("    GET  /metrics       - Prometheus metrics");
    println!("    GET  /v1/models     - List models (OpenAI-compatible)");
    println!("    POST /v1/chat/completions - Chat completions");
    println!("    POST /v1/completions      - Text completions");
    println!("    POST /v1/embeddings       - Embeddings");
    println!("    WS   /ws/stream           - WebSocket streaming");

    println!("\n");

    // ========================================================================
    // Example 2: Start with preloaded model
    // ========================================================================
    println!("Example 2: Start with Preloaded Model");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let start = ServeStart::new(");
    println!("      config.clone(),");
    println!("      \"0.0.0.0:8080\".parse().unwrap(),");
    println!("      Some(\"llama-2-7b\".to_string()),");
    println!("      false,");
    println!("      0,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Starting HTTP API Server ===");
    println!("  Bind Address: 0.0.0.0:8080");
    println!("  Startup Model: llama-2-7b");
    println!("  Mode: Single-process");
    println!("  ");
    println!("  Available Endpoints:");
    println!("    [same as Example 1]");

    println!("\n");

    // ========================================================================
    // Example 3: Start with distributed inference
    // ========================================================================
    println!("Example 3: Start with Distributed Inference");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let start = ServeStart::new(");
    println!("      config.clone(),");
    println!("      \"0.0.0.0:8080\".parse().unwrap(),");
    println!("      None,");
    println!("      true,     // distributed mode");
    println!("      4,        // 4 workers");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Starting HTTP API Server ===");
    println!("  Bind Address: 0.0.0.0:8080");
    println!("  Mode: Distributed");
    println!("  Workers: 4");
    println!("  ");
    println!("  Available Endpoints:");
    println!("    [same as Example 1]");

    println!("\n");

    // ========================================================================
    // Example 4: Check server status
    // ========================================================================
    println!("Example 4: Check Server Status");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let status = ServeStatus::new(");
    println!("      config.clone(),");
    println!("      false,    // not detailed");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === HTTP Server Status ===");
    println!("  Server: ✗ Stopped");

    println!("\n");

    // ========================================================================
    // Example 5: Detailed server status
    // ========================================================================
    println!("Example 5: Detailed Server Status");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let status = ServeStatus::new(");
    println!("      config.clone(),");
    println!("      true,     // detailed");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === HTTP Server Status ===");
    println!("  Server: ✓ Running");
    println!("  Address: 127.0.0.1:8080");
    println!("  Uptime: 3600s");
    println!("  ");
    println!("  Statistics:");
    println!("    Total Requests: 10543");
    println!("    Active Connections: 12");
    println!("  ");
    println!("  Detailed Information:");
    println!("    Workers: 4");
    println!("    Distributed: No");
    println!("    Loaded Models: 1");
    println!("    Memory Usage: 512 MB");

    println!("\n");

    // ========================================================================
    // Example 6: Reload configuration
    // ========================================================================
    println!("Example 6: Reload Configuration");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let reload = ServeReload::new(");
    println!("      config.clone(),");
    println!("      None,     // use default config");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Reloading Server Configuration ===");
    println!("  Using default configuration");
    println!("  ");
    println!("  ✓ Configuration reloaded successfully");
    println!("  ");
    println!("  Changes Applied:");
    println!("    - Updated rate limits");
    println!("    - Refreshed model paths");
    println!("    - Applied new CORS settings");

    println!("\n");

    // ========================================================================
    // Example 7: Reload with custom config
    // ========================================================================
    println!("Example 7: Reload with Custom Config");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let reload = ServeReload::new(");
    println!("      config.clone(),");
    println!("      Some(PathBuf::from(\"/etc/inferno/config.toml\")),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Reloading Server Configuration ===");
    println!("  Configuration File: \"/etc/inferno/config.toml\"");
    println!("  ");
    println!("  ✓ Configuration reloaded successfully");
    println!("  ");
    println!("  Changes Applied:");
    println!("    - Updated rate limits");
    println!("    - Refreshed model paths");
    println!("    - Applied new CORS settings");

    println!("\n");

    // ========================================================================
    // Example 8: Graceful shutdown
    // ========================================================================
    println!("Example 8: Graceful Shutdown");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let stop = ServeStop::new(");
    println!("      config.clone(),");
    println!("      false,    // graceful");
    println!("      30,       // 30 second timeout");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Stopping HTTP Server ===");
    println!("  Mode: Graceful");
    println!("  Timeout: 30s");
    println!("  ");
    println!("  Waiting for active connections to complete...");
    println!("  ✓ All connections closed");
    println!("  ");
    println!("  ✓ Server stopped successfully");

    println!("\n");

    // ========================================================================
    // Example 9: Force shutdown
    // ========================================================================
    println!("Example 9: Force Shutdown");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let stop = ServeStop::new(");
    println!("      config.clone(),");
    println!("      true,     // force");
    println!("      5,        // 5 second timeout");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Stopping HTTP Server ===");
    println!("  Mode: Force");
    println!("  Timeout: 5s");
    println!("  ");
    println!("  ⚠️  Forcing immediate shutdown");
    println!("  ");
    println!("  ✓ Server stopped successfully");

    println!("\n");

    // ========================================================================
    // Example 10: Validation tests
    // ========================================================================
    println!("Example 10: Input Validation");
    println!("{}", "─".repeat(80));

    let addr: std::net::SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let too_many_workers = ServeStart::new(config.clone(), addr, None, true, 150);
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(too_many_workers), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught excessive workers:");
            println!("  {}", e);
        }
    }

    println!();

    let zero_timeout = ServeStop::new(config.clone(), false, 0);

    match pipeline
        .execute(Box::new(zero_timeout), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught zero timeout:");
            println!("  {}", e);
        }
    }

    println!();

    let excessive_timeout = ServeStop::new(config.clone(), false, 400);

    match pipeline
        .execute(Box::new(excessive_timeout), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught excessive timeout:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Serve Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ HTTP API server startup");
    println!("✓ Distributed inference mode");
    println!("✓ Preloaded model support");
    println!("✓ Server status monitoring");
    println!("✓ Configuration reload (hot reload)");
    println!("✓ Graceful shutdown");
    println!("✓ Force shutdown");
    println!("✓ OpenAI-compatible endpoints");
    println!("✓ WebSocket streaming support");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Worker count <= 100 for distributed mode");
    println!("  - Shutdown timeout: 1-300 seconds");
    println!("  - Configuration file must exist if specified");
    println!();
    println!("Use Cases:");
    println!("  - Local development API server");
    println!("  - Production inference endpoints");
    println!("  - Distributed workload management");
    println!("  - Hot configuration updates");

    Ok(())
}