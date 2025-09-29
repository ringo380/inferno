//! Application State Management
//!
//! This module defines the global application state that is shared across
//! all Tauri commands and managed by Tauri's state management system.

use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use sysinfo::System;
use tauri::AppHandle;

use super::types::{AppSettings, Notification, BatchJob, MetricsSnapshot};
use super::{
    ActivityLogger, BackendManager, SecurityManager, ModelRepositoryService,
    ModelDownloadManager,
};
use super::events::EventManager;

/// Global application state for the desktop application
///
/// This state is initialized once at application startup and managed by Tauri.
/// All fields use Arc<Mutex<T>> or Arc<T> for thread-safe access across
/// async Tauri commands.
pub struct AppState {
    /// System information (CPU, memory, etc.)
    pub system: Arc<Mutex<System>>,

    /// Backend manager for model loading and inference
    pub backend_manager: Arc<BackendManager>,

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

    /// Model repository service (HuggingFace integration)
    pub model_repository: Arc<ModelRepositoryService>,

    /// Model download manager
    pub download_manager: Arc<ModelDownloadManager>,
}

impl AppState {
    /// Create a new AppState instance
    ///
    /// This should be called once during application startup, typically in the
    /// Tauri setup handler. The AppHandle is needed to initialize the EventManager
    /// which requires access to the Tauri event system.
    pub async fn new(app_handle: Option<AppHandle>) -> Result<Self, String> {
        // Initialize system info tracker
        let mut system = System::new_all();
        system.refresh_all();

        // Load settings from disk or use defaults
        let settings = Self::load_settings().await.unwrap_or_default();

        // Initialize backend manager with config from settings
        let config = inferno::core::config::Config {
            models_dir: Some(PathBuf::from(&settings.models_directory)),
            backend_config: Some(inferno::core::backends::BackendConfig {
                prefer_gpu: settings.prefer_gpu,
                context_size: Some(2048), // Default context size
                batch_size: Some(512),    // Default batch size
                ..Default::default()
            }),
            ..Default::default()
        };

        let backend_manager = BackendManager::new(config)
            .map_err(|e| format!("Failed to initialize backend manager: {}", e))?;

        // Initialize activity logger with settings path
        let activity_logger = ActivityLogger::new(
            PathBuf::from(".inferno-activity.json"),
            settings.enable_audit_log
        ).map_err(|e| format!("Failed to initialize activity logger: {}", e))?;

        // Initialize security manager
        let security_manager = SecurityManager::new(
            PathBuf::from(".inferno-keys.json"),
            settings.require_authentication
        ).map_err(|e| format!("Failed to initialize security manager: {}", e))?;

        // Initialize model repository service
        let model_repository = ModelRepositoryService::new()
            .map_err(|e| format!("Failed to initialize model repository: {}", e))?;

        // Initialize download manager with models directory
        let download_manager = ModelDownloadManager::new(
            PathBuf::from(&settings.models_directory)
        ).map_err(|e| format!("Failed to initialize download manager: {}", e))?;

        // Initialize event manager if app handle is provided
        let event_manager = if let Some(handle) = app_handle {
            Some(EventManager::new(handle))
        } else {
            None
        };

        Ok(Self {
            system: Arc::new(Mutex::new(system)),
            backend_manager: Arc::new(backend_manager),
            metrics: Arc::new(Mutex::new(MetricsSnapshot::default())),
            activity_logger: Arc::new(activity_logger),
            settings: Arc::new(Mutex::new(settings)),
            notifications: Arc::new(Mutex::new(Vec::new())),
            batch_jobs: Arc::new(Mutex::new(Vec::new())),
            security_manager: Arc::new(security_manager),
            event_manager: Arc::new(Mutex::new(event_manager)),
            model_repository: Arc::new(model_repository),
            download_manager: Arc::new(download_manager),
        })
    }

    /// Load settings from disk
    async fn load_settings() -> Result<AppSettings, String> {
        let config_path = Self::get_config_path();

        if let Ok(contents) = tokio::fs::read_to_string(&config_path).await {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&contents) {
                return Ok(settings);
            }
        }

        // Return default settings if file doesn't exist or can't be parsed
        Ok(AppSettings::default())
    }

    /// Get the configuration file path
    fn get_config_path() -> PathBuf {
        // For Tauri v2, we'll use a simple approach - store config in current directory
        // In production, you might want to use a proper config directory
        PathBuf::from(".").join("inferno-settings.json")
    }

    /// Initialize the event manager after the app is fully started
    ///
    /// This is called from the Tauri setup handler after the app handle is available.
    /// It allows the event manager to be initialized with the proper app handle.
    pub fn init_event_manager(&self, app_handle: AppHandle) {
        let mut event_mgr = self.event_manager.lock()
            .expect("Failed to lock event manager");
        *event_mgr = Some(EventManager::new(app_handle));
    }

    /// Perform cleanup when the application is shutting down
    pub async fn shutdown(&self) -> Result<(), String> {
        // Save settings to disk
        let settings = self.settings.lock()
            .map_err(|e| format!("Failed to lock settings: {}", e))?;

        let config_path = Self::get_config_path();
        let contents = serde_json::to_string_pretty(&*settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        tokio::fs::write(&config_path, contents).await
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        // Flush activity logs
        self.activity_logger.flush()
            .map_err(|e| format!("Failed to flush activity logs: {}", e))?;

        // Unload all models
        let loaded_models = self.backend_manager.get_loaded_models();
        for model_id in loaded_models {
            let _ = self.backend_manager.unload_model(model_id).await;
        }

        Ok(())
    }
}

impl Default for AppState {
    fn default() -> Self {
        // This is a fallback for situations where async initialization isn't available
        // Typically you should use AppState::new() instead
        let system = System::new_all();
        let settings = AppSettings::default();

        let config = inferno::core::config::Config {
            models_dir: Some(PathBuf::from(&settings.models_directory)),
            ..Default::default()
        };

        let backend_manager = BackendManager::new(config)
            .expect("Failed to initialize backend manager");

        let activity_logger = ActivityLogger::new(
            PathBuf::from(".inferno-activity.json"),
            true
        ).expect("Failed to initialize activity logger");

        let security_manager = SecurityManager::new(
            PathBuf::from(".inferno-keys.json"),
            false
        ).expect("Failed to initialize security manager");

        let model_repository = ModelRepositoryService::new()
            .expect("Failed to initialize model repository");

        let download_manager = ModelDownloadManager::new(
            PathBuf::from(&settings.models_directory)
        ).expect("Failed to initialize download manager");

        Self {
            system: Arc::new(Mutex::new(system)),
            backend_manager: Arc::new(backend_manager),
            metrics: Arc::new(Mutex::new(MetricsSnapshot::default())),
            activity_logger: Arc::new(activity_logger),
            settings: Arc::new(Mutex::new(settings)),
            notifications: Arc::new(Mutex::new(Vec::new())),
            batch_jobs: Arc::new(Mutex::new(Vec::new())),
            security_manager: Arc::new(security_manager),
            event_manager: Arc::new(Mutex::new(None)),
            model_repository: Arc::new(model_repository),
            download_manager: Arc::new(download_manager),
        }
    }
}
