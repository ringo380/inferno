pub mod ab_testing;
pub mod advanced_monitoring;
pub mod api;
pub mod api_gateway;
pub mod audit;
pub mod backup_recovery;
pub mod backends;
pub mod batch;
pub mod cache;
pub mod cli;
pub mod config;
pub mod conversion;
pub mod data_pipeline;
pub mod distributed;
pub mod gpu;
pub mod logging_audit;
pub mod io;
pub mod metrics;
pub mod models;
pub mod monitoring;
pub mod multimodal;
pub mod multi_tenancy;
pub mod advanced_cache;
pub mod observability;
pub mod optimization;
pub mod performance_optimization;
pub mod deployment;
pub mod marketplace;
pub mod model_versioning;
pub mod federated;
pub mod dashboard;
pub mod qa_framework;
pub mod resilience;
pub mod response_cache;
pub mod security;
pub mod streaming;
pub mod tui;
pub mod versioning;

use anyhow::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn setup_logging(level: &str, format: &str) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = tracing_subscriber::registry().with(filter);

    match format {
        "json" => {
            subscriber.with(fmt::layer().json()).init();
        }
        "compact" => {
            subscriber.with(fmt::layer().compact()).init();
        }
        _ => {
            subscriber.with(fmt::layer()).init();
        }
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum InfernoError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Invalid model format: {0}")]
    InvalidModelFormat(String),

    #[error("Model validation failed: {0}")]
    ModelValidationFailed(String),

    #[error("Model file corrupted: {0}")]
    ModelCorrupted(String),

    #[error("Model size exceeds limit: {actual} bytes (max: {limit} bytes)")]
    ModelTooLarge { actual: u64, limit: u64 },

    #[error("Unsupported model format: {0}")]
    UnsupportedFormat(String),

    #[error("Model metadata error: {0}")]
    ModelMetadata(String),

    #[error("Security validation failed: {0}")]
    SecurityValidation(String),

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Backend initialization failed: {0}")]
    BackendInit(String),

    #[error("Inference error: {0}")]
    Inference(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Resource unavailable: {0}")]
    ResourceUnavailable(String),

    #[error("Streaming error: {0}")]
    Streaming(String),

    #[error("Stream timeout: {0}")]
    Timeout(String),

    #[error("Stream limit exceeded: {0}")]
    StreamingLimit(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Optimization failed: {0}")]
    OptimizationFailed(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("I/O error: {0}")]
    IoError(String),
}