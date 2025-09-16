use crate::{config::Config, metrics::MetricsCollector};
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use clap::{Args, Subcommand};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
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
        #[arg(short, long, help = "Server bind address", default_value = "127.0.0.1:9090")]
        bind: String,
    },
}

pub async fn execute(args: MetricsArgs, _config: &Config) -> Result<()> {
    match args.command {
        MetricsCommand::Json => {
            let mut collector = MetricsCollector::new();
            collector.start_event_processing().await?;

            let json_output = collector.export_metrics_json().await?;
            println!("{}", json_output);
        }

        MetricsCommand::Prometheus => {
            let mut collector = MetricsCollector::new();
            collector.start_event_processing().await?;

            let prometheus_output = collector.export_prometheus_format().await?;
            println!("{}", prometheus_output);
        }

        MetricsCommand::Snapshot { pretty } => {
            let mut collector = MetricsCollector::new();
            collector.start_event_processing().await?;

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
    use axum::{
        extract::State,
        http::StatusCode,
        response::IntoResponse,
        routing::get,
        Json, Router,
    };
    use serde_json::json;
    use std::sync::Arc;
    use tokio::signal;
    use tower::ServiceBuilder;
    use tower_http::{cors::CorsLayer, trace::TraceLayer};

    // Initialize metrics collector
    let mut metrics_collector = MetricsCollector::new();
    metrics_collector.start_event_processing().await?;

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
                .layer(CorsLayer::permissive())
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
        "version": env!("CARGO_PKG_VERSION"),
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

async fn metrics_prometheus_handler(State(state): State<Arc<MetricsServerState>>) -> impl IntoResponse {
    use axum::http::header;

    match state.metrics.export_prometheus_format().await {
        Ok(metrics) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
            metrics
        ).into_response(),
        Err(e) => {
            tracing::warn!("Failed to export Prometheus metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to export metrics").into_response()
        }
    }
}

async fn metrics_json_handler(State(state): State<Arc<MetricsServerState>>) -> impl IntoResponse {
    match state.metrics.export_metrics_json().await {
        Ok(metrics_json) => (StatusCode::OK, metrics_json).into_response(),
        Err(e) => {
            tracing::warn!("Failed to export JSON metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to export metrics").into_response()
        }
    }
}

async fn metrics_snapshot_handler(State(state): State<Arc<MetricsServerState>>) -> impl IntoResponse {
    match state.metrics.get_snapshot().await {
        Ok(snapshot) => Json(snapshot).into_response(),
        Err(e) => {
            tracing::warn!("Failed to get metrics snapshot: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to get metrics snapshot"
            }))).into_response()
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