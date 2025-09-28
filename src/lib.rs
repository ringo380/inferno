//! # Inferno - Enterprise AI/ML Model Runner
//!
//! Inferno is an enterprise-grade offline AI/ML model runner designed for production
//! deployment with comprehensive infrastructure capabilities.
//!
//! ## Features
//!
//! - **Multi-Backend Support**: GGUF (llama.cpp) and ONNX backends with pluggable architecture
//! - **Enterprise Infrastructure**: Async-first, secure, scalable, and observable
//! - **Multiple Interfaces**: CLI, TUI, HTTP API, and desktop application
//! - **Production Ready**: Comprehensive error handling, logging, monitoring, and testing
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
//! │   Interfaces    │    │   Core Engine    │    │    Backends     │
//! │                 │    │                  │    │                 │
//! │ • CLI (clap)    │────│ • Config System  │────│ • GGUF (llama)  │
//! │ • TUI (ratatui) │    │ • Error Handling │    │ • ONNX (ort)    │
//! │ • HTTP API      │    │ • Async Runtime  │    │ • Pluggable     │
//! │ • Desktop App   │    │ • Security       │    │   Architecture  │
//! └─────────────────┘    └──────────────────┘    └─────────────────┘
//! ```

use std::fmt;

// Re-export main components
pub mod backends;
pub mod config;
pub mod cli;
pub mod api;
pub mod tui;
pub mod models;
pub mod io;
pub mod metrics;
pub mod batch;
pub mod security;
pub mod model_versioning;

// Additional modules
pub mod audit;
pub mod cache;
pub mod dashboard;
pub mod distributed;
pub mod monitoring;
pub mod observability;
pub mod response_cache;
pub mod advanced_cache;
pub mod advanced_monitoring;
pub mod backup_recovery;
pub mod data_pipeline;
pub mod deployment;
pub mod federated;
pub mod gpu;
pub mod logging_audit;
pub mod marketplace;
pub mod multi_tenancy;
pub mod multimodal;
pub mod streaming;
pub mod upgrade;
pub mod versioning;
pub mod resilience;
pub mod optimization;
pub mod conversion;
pub mod api_gateway;
pub mod qa_framework;
pub mod performance_optimization;
pub mod performance_baseline;

// Conditional Tauri app support
#[cfg(feature = "tauri-app")]
pub mod tauri_app;

/// Core error types for the Inferno platform
#[derive(Debug, thiserror::Error)]
pub enum InfernoError {
    #[error("Configuration error: {0}")]
    Config(#[from] figment::Error),
    
    #[error("Backend error: {0}")]
    Backend(String),
    
    #[error("Model error: {0}")]
    Model(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Resource error: {0}")]
    Resource(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Concurrency error: {0}")]
    Concurrency(String),
    
    #[error("Cache error: {0}")]
    Cache(String),
    
    #[error("Security error: {0}")]
    Security(String),

    #[error("Security validation error: {0}")]
    SecurityValidation(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Distributed error: {0}")]
    Distributed(String),
    
    #[error("Performance error: {0}")]
    Performance(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Streaming limit exceeded: {0}")]
    StreamingLimit(String),
}

/// Result type for Inferno operations
pub type Result<T> = std::result::Result<T, InfernoError>;

/// Initialize the Inferno platform with comprehensive logging and tracing
pub fn init_platform() -> Result<()> {
    // Initialize tracing subscriber with environment filter
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| InfernoError::Unknown(format!("Failed to initialize tracing: {}", e)))?;
    
    tracing::info!("🔥 Inferno platform initialized");
    Ok(())
}

/// Platform information and capabilities
pub struct PlatformInfo {
    pub version: &'static str,
    pub backends: Vec<String>,
    pub features: Vec<String>,
    pub interfaces: Vec<String>,
}

impl PlatformInfo {
    pub fn new() -> Self {
        let mut backends = Vec::new();
        let mut features = Vec::new();
        let mut interfaces = vec!["CLI".to_string(), "TUI".to_string(), "HTTP API".to_string()];
        
        // Check available backends
        #[cfg(feature = "gguf")]
        backends.push("GGUF".to_string());
        
        #[cfg(feature = "onnx")]
        backends.push("ONNX".to_string());
        
        // Check available features
        #[cfg(feature = "gpu-metal")]
        features.push("Metal GPU".to_string());
        
        #[cfg(feature = "gpu-vulkan")]
        features.push("Vulkan GPU".to_string());
        
        #[cfg(feature = "tauri-app")]
        {
            features.push("Desktop App".to_string());
            interfaces.push("Desktop GUI".to_string());
        }
        
        #[cfg(feature = "download")]
        features.push("Model Download".to_string());
        
        Self {
            version: env!("CARGO_PKG_VERSION"),
            backends,
            features,
            interfaces,
        }
    }
}

impl fmt::Display for PlatformInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "🔥 Inferno AI/ML Platform v{}", self.version)?;
        writeln!(f, "   Backends: {}", self.backends.join(", "))?;
        writeln!(f, "   Features: {}", self.features.join(", "))?;
        writeln!(f, "   Interfaces: {}", self.interfaces.join(", "))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_platform_info() {
        let info = PlatformInfo::new();
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(!info.interfaces.is_empty());
    }
    
    #[test]
    fn test_error_types() {
        let error = InfernoError::Backend("test error".to_string());
        assert!(error.to_string().contains("Backend error"));
    }
}
