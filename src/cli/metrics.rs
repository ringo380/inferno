//! Metrics Command
//!
//! This module provides metrics collection and export functionality with support
//! for JSON, Prometheus, and snapshot formats. Also includes a standalone metrics
//! HTTP server for production monitoring.

use crate::interfaces::cli::{Command, CommandContext, CommandOutput};
use crate::{config::Config, metrics::MetricsCollector};
use anyhow::Result;
use async_trait::async_trait;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use clap::{Args, Subcommand};
use serde_json::json;
use std::sync::Arc;
use tokio::signal;
use tracing::info;

#[derive(Args)]
pub struct MetricsArgs {
    #[command(subcommand)]
    pub command: MetricsCommand,
}

#[derive(Subcommand)]
pub enum MetricsCommand {
    #[command(about = "Export metrics in JSON format")]
    Json,

    #[command(about = "Export metrics in Prometheus format")]
    Prometheus,

    #[command(about = "Show detailed metrics snapshot")]
    Snapshot {
        #[arg(short, long, help = "Pretty print JSON output")]
        pretty: bool,
    },

    #[command(about = "Start standalone metrics server")]
    Server {
        #[arg(
            short,
            long,
            help = "Server bind address (format: host:port)",
            default_value = "127.0.0.1:9090"
        )]
        bind: String,
    },
}

// ============================================================================
// Command Trait Implementations
// ============================================================================

/// Export metrics in JSON format
pub struct MetricsJsonCommand {
    #[allow(dead_code)]
    config: Config,
}

impl MetricsJsonCommand {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Command for MetricsJsonCommand {
    fn name(&self) -> &str {
        "metrics json"
    }

    fn description(&self) -> &str {
        "Export metrics in JSON format"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // No validation needed for JSON export
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Exporting metrics in JSON format");

        let (collector, processor) = MetricsCollector::new();
        processor.start();

        let json_output = collector.export_metrics_json().await?;

        // Human-readable output (already JSON)
        if !ctx.json_output {
            println!("{}", json_output);
        }

        // Structured output - parse the JSON string back to Value
        let metrics_value: serde_json::Value = serde_json::from_str(&json_output)?;

        Ok(CommandOutput::success_with_data(
            "Metrics exported in JSON format",
            json!({
                "format": "json",
                "metrics": metrics_value,
            }),
        ))
    }
}

/// Export metrics in Prometheus format
pub struct MetricsPrometheusCommand {
    #[allow(dead_code)]
    config: Config,
}

impl MetricsPrometheusCommand {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Command for MetricsPrometheusCommand {
    fn name(&self) -> &str {
        "metrics prometheus"
    }

    fn description(&self) -> &str {
        "Export metrics in Prometheus format"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // No validation needed for Prometheus export
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Exporting metrics in Prometheus format");

        let (collector, processor) = MetricsCollector::new();
        processor.start();

        let prometheus_output = collector.export_prometheus_format().await?;

        // Human-readable output (Prometheus text format)
        if !ctx.json_output {
            println!("{}", prometheus_output);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Metrics exported in Prometheus format",
            json!({
                "format": "prometheus",
                "content_type": "text/plain; version=0.0.4; charset=utf-8",
                "metrics": prometheus_output,
            }),
        ))
    }
}

/// Show detailed metrics snapshot
pub struct MetricsSnapshotCommand {
    #[allow(dead_code)]
    config: Config,
    pretty: bool,
}

impl MetricsSnapshotCommand {
    pub fn new(config: Config, pretty: bool) -> Self {
        Self { config, pretty }
    }
}

#[async_trait]
impl Command for MetricsSnapshotCommand {
    fn name(&self) -> &str {
        "metrics snapshot"
    }

    fn description(&self) -> &str {
        "Show detailed metrics snapshot"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // No validation needed for snapshot
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Creating metrics snapshot");

        let (collector, processor) = MetricsCollector::new();
        processor.start();

        let snapshot = collector.get_snapshot().await?;

        // Human-readable output
        if !ctx.json_output {
            if self.pretty {
                println!("{}", serde_json::to_string_pretty(&snapshot)?);
            } else {
                println!("{}", serde_json::to_string(&snapshot)?);
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Metrics snapshot created",
            json!({
                "snapshot": snapshot,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "pretty": self.pretty,
            }),
        ))
    }
}

/// Start standalone metrics server
pub struct MetricsServerCommand {
    #[allow(dead_code)]
    config: Config,
    bind: String,
}

impl MetricsServerCommand {
    pub fn new(config: Config, bind: String) -> Self {
        Self { config, bind }
    }
}

#[async_trait]
impl Command for MetricsServerCommand {
    fn name(&self) -> &str {
        "metrics server"
    }

    fn description(&self) -> &str {
        "Start standalone metrics server"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // Validate bind address format
        validate_bind_address(&self.bind)?;
        Ok(())
    }

    async fn execute(&self, _ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Starting standalone metrics server on {}", self.bind);
        start_metrics_server(&self.bind).await?;

        Ok(CommandOutput::success_with_data(
            "Metrics server stopped",
            json!({
                "bind_address": self.bind,
                "status": "stopped",
            }),
        ))
    }
}

/// Validate a bind address format (host:port)
fn validate_bind_address(bind: &str) -> Result<()> {
    let parts: Vec<&str> = bind.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!(
            "Invalid bind address format '{}'. Expected format: host:port (e.g., 127.0.0.1:9090)",
            bind
        );
    }

    let host = parts[0];
    let port_str = parts[1];

    // Validate port is a number
    let port: u16 = port_str.parse().map_err(|_| {
        anyhow::anyhow!(
            "Invalid port '{}' in bind address. Port must be a number between 1 and 65535",
            port_str
        )
    })?;

    // Validate port is in valid range
    if port == 0 {
        anyhow::bail!("Port 0 is not allowed. Please specify a port between 1 and 65535");
    }

    // Basic host validation - allow IP addresses and hostnames
    if host.is_empty() {
        anyhow::bail!("Host cannot be empty in bind address");
    }

    Ok(())
}

pub async fn execute(args: MetricsArgs, _config: &Config) -> Result<()> {
    match args.command {
        MetricsCommand::Json => {
            let (collector, processor) = MetricsCollector::new();
            processor.start();

            let json_output = collector.export_metrics_json().await?;
            println!("{}", json_output);
        }

        MetricsCommand::Prometheus => {
            let (collector, processor) = MetricsCollector::new();
            processor.start();

            let prometheus_output = collector.export_prometheus_format().await?;
            println!("{}", prometheus_output);
        }

        MetricsCommand::Snapshot { pretty } => {
            let (collector, processor) = MetricsCollector::new();
            processor.start();

            let snapshot = collector.get_snapshot().await?;

            if pretty {
                println!("{}", serde_json::to_string_pretty(&snapshot)?);
            } else {
                println!("{}", serde_json::to_string(&snapshot)?);
            }
        }

        MetricsCommand::Server { bind } => {
            info!("Starting standalone metrics server on {}", bind);
            start_metrics_server(&bind).await?;
        }
    }

    Ok(())
}

async fn start_metrics_server(bind_addr: &str) -> Result<()> {
    use axum::{routing::get, Router};

    use std::sync::Arc;

    use tower::ServiceBuilder;
    use tower_http::{cors::CorsLayer, trace::TraceLayer};

    // Initialize metrics collector
    let (metrics_collector, processor) = MetricsCollector::new();
    processor.start();

    let state = Arc::new(MetricsServerState {
        metrics: metrics_collector,
    });

    // Build the router with metrics endpoints only
    let app = Router::new()
        .route("/", get(metrics_root))
        .route("/metrics", get(metrics_prometheus_handler))
        .route("/metrics/json", get(metrics_json_handler))
        .route("/metrics/snapshot", get(metrics_snapshot_handler))
        .route("/health", get(health_check))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state);

    info!("Metrics server endpoints:");
    info!("  GET  /             - Server information");
    info!("  GET  /health       - Health check");
    info!("  GET  /metrics      - Prometheus metrics");
    info!("  GET  /metrics/json - JSON metrics");
    info!("  GET  /metrics/snapshot - Detailed metrics snapshot");

    // Create the listener
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;

    info!("Metrics server running on http://{}", bind_addr);

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Metrics server shut down gracefully");
    Ok(())
}

struct MetricsServerState {
    metrics: MetricsCollector,
}

async fn metrics_root() -> impl IntoResponse {
    Json(json!({
        "name": "Inferno Metrics Server",
        "version": std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string()),
        "endpoints": {
            "/health": "Health check",
            "/metrics": "Prometheus metrics",
            "/metrics/json": "JSON formatted metrics",
            "/metrics/snapshot": "Detailed metrics snapshot"
        }
    }))
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn metrics_prometheus_handler(
    State(state): State<Arc<MetricsServerState>>,
) -> impl IntoResponse {
    use axum::http::header;

    match state.metrics.export_prometheus_format().await {
        Ok(metrics) => (
            StatusCode::OK,
            [(
                header::CONTENT_TYPE,
                "text/plain; version=0.0.4; charset=utf-8",
            )],
            metrics,
        )
            .into_response(),
        Err(e) => {
            tracing::warn!("Failed to export Prometheus metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to export metrics",
            )
                .into_response()
        }
    }
}

async fn metrics_json_handler(State(state): State<Arc<MetricsServerState>>) -> impl IntoResponse {
    match state.metrics.export_metrics_json().await {
        Ok(metrics_json) => (StatusCode::OK, metrics_json).into_response(),
        Err(e) => {
            tracing::warn!("Failed to export JSON metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to export metrics",
            )
                .into_response()
        }
    }
}

async fn metrics_snapshot_handler(
    State(state): State<Arc<MetricsServerState>>,
) -> impl IntoResponse {
    match state.metrics.get_snapshot().await {
        Ok(snapshot) => Json(snapshot).into_response(),
        Err(e) => {
            tracing::warn!("Failed to get metrics snapshot: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to get metrics snapshot"
                })),
            )
                .into_response()
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_bind_address_valid() {
        assert!(validate_bind_address("127.0.0.1:9090").is_ok());
        assert!(validate_bind_address("0.0.0.0:8080").is_ok());
        assert!(validate_bind_address("localhost:3000").is_ok());
        assert!(validate_bind_address("192.168.1.100:65535").is_ok());
    }

    #[test]
    fn test_validate_bind_address_invalid_format() {
        // Missing port
        let result = validate_bind_address("127.0.0.1");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid bind address format"));

        // Too many colons
        let result = validate_bind_address("127.0.0.1:9090:extra");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_bind_address_invalid_port() {
        // Non-numeric port
        let result = validate_bind_address("127.0.0.1:abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid port"));

        // Port 0
        let result = validate_bind_address("127.0.0.1:0");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Port 0 is not allowed"));

        // Port too large
        let result = validate_bind_address("127.0.0.1:70000");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_bind_address_empty_host() {
        let result = validate_bind_address(":9090");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Host cannot be empty"));
    }

    #[tokio::test]
    async fn test_metrics_json_command_validation() {
        let config = Config::default();
        let cmd = MetricsJsonCommand::new(config.clone());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_prometheus_command_validation() {
        let config = Config::default();
        let cmd = MetricsPrometheusCommand::new(config.clone());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_snapshot_command_validation() {
        let config = Config::default();
        let cmd = MetricsSnapshotCommand::new(config.clone(), false);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_server_command_validation_valid() {
        let config = Config::default();
        let cmd = MetricsServerCommand::new(config.clone(), "127.0.0.1:9090".to_string());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_server_command_validation_invalid() {
        let config = Config::default();
        let cmd = MetricsServerCommand::new(config.clone(), "invalid".to_string());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_metrics_snapshot_command_pretty() {
        let config = Config::default();
        let cmd = MetricsSnapshotCommand::new(config.clone(), true);
        let mut ctx = CommandContext::new(config);

        // Should execute without errors
        let result = cmd.execute(&mut ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.data.is_some());

        let data = output.data.unwrap();
        assert_eq!(data["pretty"], true);
    }

    #[tokio::test]
    async fn test_metrics_json_command_execution() {
        let config = Config::default();
        let cmd = MetricsJsonCommand::new(config.clone());
        let mut ctx = CommandContext::new(config);
        ctx.json_output = true; // Suppress stdout

        let result = cmd.execute(&mut ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.data.is_some());

        let data = output.data.unwrap();
        assert_eq!(data["format"], "json");
    }

    #[tokio::test]
    async fn test_metrics_prometheus_command_execution() {
        let config = Config::default();
        let cmd = MetricsPrometheusCommand::new(config.clone());
        let mut ctx = CommandContext::new(config);
        ctx.json_output = true; // Suppress stdout

        let result = cmd.execute(&mut ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.data.is_some());

        let data = output.data.unwrap();
        assert_eq!(data["format"], "prometheus");
        assert_eq!(
            data["content_type"],
            "text/plain; version=0.0.4; charset=utf-8"
        );
    }

    #[test]
    fn test_command_names() {
        let config = Config::default();

        let json_cmd = MetricsJsonCommand::new(config.clone());
        assert_eq!(json_cmd.name(), "metrics json");
        assert_eq!(json_cmd.description(), "Export metrics in JSON format");

        let prom_cmd = MetricsPrometheusCommand::new(config.clone());
        assert_eq!(prom_cmd.name(), "metrics prometheus");
        assert_eq!(
            prom_cmd.description(),
            "Export metrics in Prometheus format"
        );

        let snap_cmd = MetricsSnapshotCommand::new(config.clone(), false);
        assert_eq!(snap_cmd.name(), "metrics snapshot");
        assert_eq!(snap_cmd.description(), "Show detailed metrics snapshot");

        let server_cmd = MetricsServerCommand::new(config, "127.0.0.1:9090".to_string());
        assert_eq!(server_cmd.name(), "metrics server");
        assert_eq!(server_cmd.description(), "Start standalone metrics server");
    }
}
