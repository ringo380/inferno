#![allow(dead_code, unused_imports, unused_variables)]
use crate::{
    api::{openai, websocket},
    backends::{BackendHandle, BackendType},
    config::Config,
    distributed::DistributedInference,
    metrics::MetricsCollector,
    models::ModelManager,
    upgrade::UpgradeManager,
};
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Args;
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

#[derive(Args)]
pub struct ServeArgs {
    #[arg(
        short,
        long,
        help = "Server bind address",
        default_value = "127.0.0.1:8080"
    )]
    pub bind: SocketAddr,

    #[arg(short, long, help = "Model to load on startup")]
    pub model: Option<String>,

    #[arg(long, help = "Enable distributed inference with worker pools")]
    pub distributed: bool,

    #[arg(
        long,
        help = "Number of worker processes (0 = auto)",
        default_value = "0"
    )]
    pub workers: usize,
}

/// Maximum allowed worker count for distributed mode
const MAX_DISTRIBUTED_WORKERS: usize = 100;

/// Validate server arguments before execution
pub fn validate_args(args: &ServeArgs) -> Result<()> {
    // Validate port range (port is part of SocketAddr, already validated by clap)
    let port = args.bind.port();
    if port == 0 {
        anyhow::bail!(
            "Invalid port 0: Port must be between 1 and 65535. \
            Use a specific port like 8080 or 3000."
        );
    }

    // Validate worker count for distributed mode
    if args.distributed && args.workers > MAX_DISTRIBUTED_WORKERS {
        anyhow::bail!(
            "Worker count {} exceeds maximum of {} for distributed mode. \
            Reduce the worker count or use auto-detection with --workers 0.",
            args.workers,
            MAX_DISTRIBUTED_WORKERS
        );
    }

    // Warn about privileged ports on Unix systems
    #[cfg(unix)]
    if port < 1024 && !is_running_as_root() {
        warn!(
            "Port {} is a privileged port (< 1024). \
            You may need root/sudo privileges to bind to this port.",
            port
        );
    }

    Ok(())
}

#[cfg(unix)]
fn is_running_as_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

pub async fn execute(args: ServeArgs, config: &Config) -> Result<()> {
    // Validate arguments before proceeding
    validate_args(&args)?;

    info!("Starting HTTP server on {}", args.bind);

    // Initialize metrics collector
    let (metrics_collector, processor) = MetricsCollector::new();
    processor.start();

    // Initialize model manager
    let model_manager = Arc::new(ModelManager::new(&config.models_dir));

    // Optionally initialize distributed inference
    let distributed = if args.distributed {
        info!("Initializing distributed inference with worker pools");

        let mut distributed_config = config.distributed.clone();
        if args.workers > 0 {
            distributed_config.worker_count = args.workers;
        }

        match DistributedInference::new(
            distributed_config,
            config.backend_config.clone(),
            model_manager.clone(),
            Some(Arc::new(metrics_collector.clone())),
        )
        .await
        {
            Ok(dist) => {
                info!("Distributed inference initialized successfully");
                Some(Arc::new(dist))
            }
            Err(e) => {
                warn!("Failed to initialize distributed inference: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Optionally load a model on startup (only if not using distributed)
    let (backend, loaded_model) = if !args.distributed {
        if let Some(model_name) = &args.model {
            info!("Loading model on startup: {}", model_name);
            match load_model_on_startup(model_name, &model_manager, config).await {
                Ok((backend_handle, model_name)) => (Some(backend_handle), Some(model_name)),
                Err(e) => {
                    warn!("Failed to load startup model: {}", e);
                    (None, None)
                }
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // Initialize upgrade manager
    let upgrade_manager = match crate::upgrade::UpgradeConfig::from_config(config) {
        Ok(upgrade_config) => match UpgradeManager::new(upgrade_config).await {
            Ok(manager) => {
                info!("Upgrade system initialized for HTTP server");
                Some(Arc::new(manager))
            }
            Err(e) => {
                warn!("Failed to initialize upgrade system: {}", e);
                None
            }
        },
        Err(e) => {
            warn!("Failed to load upgrade config: {}", e);
            None
        }
    };

    // Create shared application state
    let state = Arc::new(ServerState {
        config: config.clone(),
        backend,
        loaded_model,
        metrics: metrics_collector,
        model_manager: (*model_manager).clone(),
        distributed,
        upgrade_manager,
    });

    // Build the router with all endpoints
    let app = Router::new()
        // Health and status endpoints
        .route("/health", get(health_check))
        .route("/", get(root_handler))
        // Metrics endpoints
        .route("/metrics", get(metrics_prometheus))
        .route("/metrics/json", get(metrics_json))
        .route("/metrics/snapshot", get(metrics_snapshot))
        // OpenAI-compatible API endpoints
        .route("/v1/models", get(openai::list_models))
        .route("/v1/chat/completions", post(openai::chat_completions))
        .route("/v1/completions", post(openai::completions))
        .route("/v1/embeddings", post(openai::embeddings))
        // WebSocket streaming endpoints
        .route("/ws/stream", get(websocket::websocket_handler))
        // API v1 endpoints
        .route("/v1/status", get(server_status))
        // Upgrade API endpoints
        .route("/v1/upgrade/status", get(upgrade_status))
        .route("/v1/upgrade/check", post(upgrade_check))
        .route("/v1/upgrade/install", post(upgrade_install))
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state);

    info!("HTTP API server is running on http://{}", args.bind);
    info!("Available endpoints:");
    info!("  GET  /             - Server information");
    info!("  GET  /health       - Health check");
    info!("  GET  /metrics      - Prometheus metrics");
    info!("  GET  /metrics/json - JSON metrics");
    info!("  GET  /v1/models           - List available models (OpenAI-compatible)");
    info!("  POST /v1/chat/completions - Chat completions (OpenAI-compatible)");
    info!("  POST /v1/completions      - Text completions (OpenAI-compatible)");
    info!("  POST /v1/embeddings       - Generate embeddings (OpenAI-compatible)");
    info!("  GET  /v1/status           - Server status");
    info!("  WS   /ws/stream           - WebSocket streaming inference");

    // Create the listener
    let listener = tokio::net::TcpListener::bind(&args.bind).await?;

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shut down gracefully");
    Ok(())
}

pub struct ServerState {
    pub config: Config,
    pub backend: Option<BackendHandle>,
    pub loaded_model: Option<String>,
    pub metrics: MetricsCollector,
    pub model_manager: ModelManager,
    pub distributed: Option<Arc<DistributedInference>>,
    pub upgrade_manager: Option<Arc<UpgradeManager>>,
}

// Helper functions

async fn load_model_on_startup(
    model_name: &str,
    model_manager: &ModelManager,
    config: &Config,
) -> Result<(BackendHandle, String)> {
    let model_info = model_manager.resolve_model(model_name).await?;
    let backend_type = BackendType::from_model_path(&model_info.path).ok_or_else(|| {
        anyhow::anyhow!(
            "No suitable backend found for model: {}",
            model_info.path.display()
        )
    })?;
    let backend_handle = BackendHandle::new_shared(backend_type, &config.backend_config)?;
    backend_handle.load_model(&model_info).await?;
    Ok((backend_handle, model_info.name.clone()))
}

// Handler functions

async fn root_handler() -> impl IntoResponse {
    Json(json!({
        "name": "Inferno AI/ML Runner",
        "version": std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string()),
        "description": "Offline AI/ML model runner for GGUF and ONNX models",
        "endpoints": {
            "/health": "Health check",
            "/metrics": "Prometheus metrics",
            "/metrics/json": "JSON formatted metrics",
            "/metrics/snapshot": "Detailed metrics snapshot",
            "/v1/models": "List available models (OpenAI-compatible)",
            "/v1/chat/completions": "Chat completions (OpenAI-compatible)",
            "/v1/completions": "Text completions (OpenAI-compatible)",
            "/v1/embeddings": "Generate embeddings (OpenAI-compatible)",
            "/v1/status": "Server status",
            "/ws/stream": "WebSocket streaming inference"
        }
    }))
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "uptime": "unknown" // Could track actual uptime
    }))
}

async fn metrics_prometheus(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
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
            warn!("Failed to export Prometheus metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to export metrics",
            )
                .into_response()
        }
    }
}

async fn metrics_json(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    match state.metrics.export_metrics_json().await {
        Ok(metrics_json) => (StatusCode::OK, metrics_json).into_response(),
        Err(e) => {
            warn!("Failed to export JSON metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to export metrics",
            )
                .into_response()
        }
    }
}

async fn metrics_snapshot(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    match state.metrics.get_snapshot().await {
        Ok(snapshot) => Json(snapshot).into_response(),
        Err(e) => {
            warn!("Failed to get metrics snapshot: {}", e);
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

async fn server_status(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let snapshot = match state.metrics.get_snapshot().await {
        Ok(s) => s,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to get server status"
                })),
            )
                .into_response()
        }
    };

    Json(json!({
        "status": "running",
        "loaded_model": state.loaded_model,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "metrics": {
            "total_requests": snapshot.inference_metrics.total_requests,
            "successful_requests": snapshot.inference_metrics.successful_requests,
            "memory_usage": snapshot.system_metrics.memory_usage_bytes,
            "cpu_usage": snapshot.system_metrics.cpu_usage_percent,
            "loaded_models": snapshot.model_metrics.loaded_models.len()
        }
    }))
    .into_response()
}

// Upgrade API handlers

async fn upgrade_status(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let upgrade_manager = match &state.upgrade_manager {
        Some(manager) => manager,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "Upgrade system not available"
                })),
            )
                .into_response()
        }
    };

    let status = upgrade_manager.get_status().await;
    let current_version = crate::upgrade::ApplicationVersion::current();

    Json(json!({
        "current_version": current_version.to_string(),
        "status": status,
        "upgrade_available": matches!(status, crate::upgrade::UpgradeStatus::Available(_)),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
    .into_response()
}

async fn upgrade_check(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let upgrade_manager = match &state.upgrade_manager {
        Some(manager) => manager,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "Upgrade system not available"
                })),
            )
                .into_response()
        }
    };

    match upgrade_manager.check_for_updates().await {
        Ok(Some(update_info)) => Json(json!({
            "update_available": true,
            "current_version": crate::upgrade::ApplicationVersion::current().to_string(),
            "new_version": update_info.version.to_string(),
            "release_date": update_info.release_date.to_rfc3339(),
            "changelog": update_info.changelog,
            "is_critical": update_info.is_critical,
            "is_security_update": update_info.is_security_update,
            "download_urls": update_info.download_urls,
            "checksums": update_info.checksums
        }))
        .into_response(),
        Ok(None) => Json(json!({
            "update_available": false,
            "current_version": crate::upgrade::ApplicationVersion::current().to_string(),
            "message": "Application is up to date"
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Update check failed: {}", e)
            })),
        )
            .into_response(),
    }
}

#[derive(serde::Deserialize)]
struct UpgradeInstallRequest {
    version: Option<String>,
    auto_backup: Option<bool>,
}

async fn upgrade_install(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<UpgradeInstallRequest>,
) -> impl IntoResponse {
    let upgrade_manager = match &state.upgrade_manager {
        Some(manager) => manager,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "Upgrade system not available"
                })),
            )
                .into_response()
        }
    };

    // First check for available updates
    match upgrade_manager.check_for_updates().await {
        Ok(Some(update_info)) => {
            // Verify version if specified
            if let Some(requested_version) = &payload.version {
                if &update_info.version.to_string() != requested_version {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "error": format!("Requested version {} not available", requested_version)
                        })),
                    )
                        .into_response();
                }
            }

            // Start installation
            match upgrade_manager.install_update(&update_info).await {
                Ok(_) => Json(json!({
                    "success": true,
                    "message": "Update installation completed successfully",
                    "old_version": crate::upgrade::ApplicationVersion::current().to_string(),
                    "new_version": update_info.version.to_string(),
                    "restart_required": true
                }))
                .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": format!("Installation failed: {}", e)
                    })),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "No updates available"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Update check failed: {}", e)
            })),
        )
            .into_response(),
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

    fn create_test_args(bind: &str, distributed: bool, workers: usize) -> ServeArgs {
        ServeArgs {
            bind: bind.parse().unwrap(),
            model: None,
            distributed,
            workers,
        }
    }

    #[test]
    fn test_validate_args_valid_defaults() {
        let args = create_test_args("127.0.0.1:8080", false, 0);
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_valid_distributed() {
        let args = create_test_args("127.0.0.1:8080", true, 4);
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_max_workers_allowed() {
        let args = create_test_args("127.0.0.1:8080", true, MAX_DISTRIBUTED_WORKERS);
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_too_many_workers() {
        let args = create_test_args("127.0.0.1:8080", true, 150);
        let result = validate_args(&args);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Worker count"));
        assert!(err_msg.contains("150"));
        assert!(err_msg.contains("100"));
    }

    #[test]
    fn test_validate_args_workers_ignored_non_distributed() {
        // Workers > MAX is allowed when not in distributed mode
        let args = create_test_args("127.0.0.1:8080", false, 200);
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_various_ports() {
        // Test various valid ports
        let args = create_test_args("127.0.0.1:3000", false, 0);
        assert!(validate_args(&args).is_ok());

        let args = create_test_args("0.0.0.0:9090", false, 0);
        assert!(validate_args(&args).is_ok());

        let args = create_test_args("127.0.0.1:65535", false, 0);
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_ipv6_address() {
        let args = ServeArgs {
            bind: "[::1]:8080".parse().unwrap(),
            model: None,
            distributed: false,
            workers: 0,
        };
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_with_model() {
        let args = ServeArgs {
            bind: "127.0.0.1:8080".parse().unwrap(),
            model: Some("test-model".to_string()),
            distributed: false,
            workers: 0,
        };
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_distributed_auto_workers() {
        // Auto worker detection (workers = 0) should be valid
        let args = create_test_args("127.0.0.1:8080", true, 0);
        assert!(validate_args(&args).is_ok());
    }
}
