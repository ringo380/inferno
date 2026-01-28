//! Desktop Interface Module
//!
//! This module provides the desktop application interface using Tauri v2.
//! It consolidates all desktop-specific functionality including:
//! - Tauri command handlers
//! - Application state management
//! - macOS-specific integrations (menu bar, system tray, notifications)
//! - Event emission system
//! - Backend management for model loading and inference
//! - Security and API key management
//! - Activity logging and auditing
//! - Model repository (HuggingFace) integration

pub mod activity_logger;
pub mod backend_manager;
pub mod commands;
pub mod events;
pub mod macos;
pub mod model_repository;
pub mod security;
pub mod state;
pub mod types;

// Re-export key types for convenience
pub use activity_logger::{
    ActivityLog, ActivityLogger, ActivityStats, ActivityStatus, ActivityType,
};
pub use backend_manager::{BackendManager, GlobalMetrics, InferenceParams, ModelInfo};
pub use model_repository::{
    DownloadProgress, ExternalModelInfo, ModelDownloadManager, ModelRepositoryService,
    ModelSearchQuery, ModelSearchResponse,
};
pub use security::{
    ApiKey, CreateApiKeyRequest, CreateApiKeyResponse, SecurityEvent, SecurityManager,
    SecurityMetrics, SecurityScanResult,
};
pub use state::AppState;

/// Initialize the desktop application
///
/// This is the main entry point for the desktop application, called from
/// the binary in `dashboard/src-tauri/src/main.rs`.
pub fn init() {
    tracing::info!("🖥️  Initializing Inferno desktop interface");
}
