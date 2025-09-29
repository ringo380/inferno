//! Application State Management
//!
//! This module defines the global application state that is shared across
//! all Tauri commands and managed by Tauri's state management system.

use std::sync::{Arc, Mutex};
use sysinfo::System;

/// Global application state for the desktop application
///
/// This state is initialized once at application startup and managed by Tauri.
/// All fields use Arc<Mutex<T>> or Arc<T> for thread-safe access across
/// async Tauri commands.
pub struct AppState {
    /// System information (CPU, memory, etc.)
    pub system: Arc<Mutex<System>>,

    /// Backend manager for model loading and inference
    /// Located in: dashboard/src-tauri/src/backend_manager.rs
    pub backend_manager: Arc<crate::backends::Backend>,

    /// Metrics collection and reporting
    pub metrics: Arc<Mutex<MetricsSnapshot>>,

    /// Activity logging system
    pub activity_logger: Arc<ActivityLogger>,

    /// Application settings
    pub settings: Arc<Mutex<AppSettings>>,

    /// In-memory notification store
    pub notifications: Arc<Mutex<Vec<Notification>>>,

    /// Batch job management
    pub batch_jobs: Arc<Mutex<Vec<BatchJob>>>,

    /// Security and API key management
    pub security_manager: Arc<SecurityManager>,

    /// Event emission system
    pub event_manager: Arc<Mutex<Option<EventManager>>>,

    /// SQLite database manager
    pub database: Arc<DatabaseManager>,

    /// Model repository service (HuggingFace integration)
    pub model_repository: Arc<ModelRepositoryService>,

    /// Model download manager
    pub download_manager: Arc<ModelDownloadManager>,
}

// Type aliases for external types
// These will be properly imported once we integrate with the dashboard codebase
pub type MetricsSnapshot = (); // TODO: Import from dashboard
pub type ActivityLogger = (); // TODO: Import from dashboard
pub type AppSettings = (); // TODO: Import from dashboard
pub type Notification = (); // TODO: Import from dashboard
pub type BatchJob = (); // TODO: Import from dashboard
pub type SecurityManager = (); // TODO: Import from dashboard
pub type EventManager = (); // TODO: Import from dashboard
pub type DatabaseManager = (); // TODO: Import from dashboard
pub type ModelRepositoryService = (); // TODO: Import from dashboard
pub type ModelDownloadManager = (); // TODO: Import from dashboard

impl AppState {
    /// Create a new AppState instance
    pub fn new() -> Self {
        unimplemented!("AppState initialization will be implemented during integration")
    }
}
