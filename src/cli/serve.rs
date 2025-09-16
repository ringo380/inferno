use crate::{config::Config, metrics::MetricsCollector, backends::{Backend, BackendType}, models::ModelManager, api::{openai, websocket}, distributed::DistributedInference};
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use clap::Args;
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

#[derive(Args)]
pub struct ServeArgs {
    #[arg(short, long, help = "Server bind address", default_value = "127.0.0.1:8080")]
    pub bind: SocketAddr,

    #[arg(short, long, help = "Model to load on startup")]
    pub model: Option<String>,

    #[arg(long, help = "Enable distributed inference with worker pools")]
    pub distributed: bool,

    #[arg(long, help = "Number of worker processes (0 = auto)", default_value = "0")]
    pub workers: usize,
}

pub async fn execute(args: ServeArgs, config: &Config) -> Result<()> {
    info!("Starting HTTP server on {}", args.bind);

    // Initialize metrics collector
    let mut metrics_collector = MetricsCollector::new();
    metrics_collector.start_event_processing().await?;

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
        ).await {
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
            match load_model_on_startup(model_name, &*model_manager, config).await {
                Ok((backend, model_name)) => (Some(Arc::new(Mutex::new(backend))), Some(model_name)),
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

    // Create shared application state
    let state = Arc::new(ServerState {
        config: config.clone(),
        backend,
        loaded_model,
        metrics: metrics_collector,
        model_manager: (*model_manager).clone(),
        distributed,
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

        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
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
    pub backend: Option<Arc<Mutex<Backend>>>,
    pub loaded_model: Option<String>,
    pub metrics: MetricsCollector,
    pub model_manager: ModelManager,
    pub distributed: Option<Arc<DistributedInference>>,
}

// Helper functions

async fn load_model_on_startup(
    model_name: &str,
    model_manager: &ModelManager,
    config: &Config,
) -> Result<(Backend, String)> {
    let model_info = model_manager.resolve_model(model_name).await?;
    let backend_type = BackendType::from_model_path(&model_info.path);
    let mut backend = Backend::new(backend_type, &config.backend_config)?;
    backend.load_model(&model_info).await?;
    Ok((backend, model_info.name.clone()))
}

// Handler functions

async fn root_handler() -> impl IntoResponse {
    Json(json!({
        "name": "Inferno AI/ML Runner",
        "version": env!("CARGO_PKG_VERSION"),
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
            [(header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
            metrics
        ).into_response(),
        Err(e) => {
            warn!("Failed to export Prometheus metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to export metrics").into_response()
        }
    }
}

async fn metrics_json(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    match state.metrics.export_metrics_json().await {
        Ok(metrics_json) => (StatusCode::OK, metrics_json).into_response(),
        Err(e) => {
            warn!("Failed to export JSON metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to export metrics").into_response()
        }
    }
}

async fn metrics_snapshot(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    match state.metrics.get_snapshot().await {
        Ok(snapshot) => Json(snapshot).into_response(),
        Err(e) => {
            warn!("Failed to get metrics snapshot: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to get metrics snapshot"
            }))).into_response()
        }
    }
}


async fn server_status(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let snapshot = match state.metrics.get_snapshot().await {
        Ok(s) => s,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "error": "Failed to get server status"
        }))).into_response()
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
    })).into_response()
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