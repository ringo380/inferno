#![allow(clippy::manual_map)]

//! Application State Management
//!
//! This module defines the global application state that is shared across
//! all Tauri commands and managed by Tauri's state management system.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use sysinfo::{System, SystemExt};
use tauri::AppHandle;
use tokio::runtime::Runtime;

use super::events::EventManager;
use super::types::{AppSettings, BatchJob, MetricsSnapshot, Notification};
use super::{
    ActivityLogger, BackendManager, ModelDownloadManager, ModelRepositoryService, SecurityManager,
};
use crate::gpu::{GpuConfiguration, GpuManager};

/// Global application state for the desktop application
///
/// This state is initialized once at application startup and managed by Tauri.
/// All fields use Arc<Mutex<T>> or Arc<T> for thread-safe access across
/// async Tauri commands.
pub struct AppState {
    /// System information (CPU, memory, etc.)
    pub system: Arc<Mutex<System>>,

    /// GPU detection and telemetry manager
    pub gpu_manager: Arc<GpuManager>,

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

        const MAX_ACTIVITY_LOGS: usize = 500;

        // Initialize subsystems shared across the desktop interface
        let activity_logger = Arc::new(ActivityLogger::new(MAX_ACTIVITY_LOGS));

        let models_dir = PathBuf::from(&settings.models_directory);
        let backend_manager = Arc::new(
            BackendManager::with_models_dir(Arc::clone(&activity_logger), models_dir)
                .await
                .map_err(|e| format!("Failed to initialize backend manager: {}", e))?,
        );

        let gpu_manager = Arc::new(GpuManager::new(GpuConfiguration {
            enabled: settings.prefer_gpu,
            preferred_vendor: None,
            ..Default::default()
        }));
        if let Err(err) = gpu_manager.initialize().await {
            tracing::warn!(
                error = ?err,
                "Failed to initialize GPU manager; GPU metrics will be unavailable"
            );
        }

        let security_manager = Arc::new(SecurityManager::new(()));
        let model_repository = Arc::new(ModelRepositoryService::new());
        let download_manager = Arc::new(ModelDownloadManager::new());

        // Initialize event manager if app handle is provided
        let event_manager = if let Some(handle) = app_handle {
            Some(EventManager::new(handle))
        } else {
            None
        };

        Ok(Self {
            system: Arc::new(Mutex::new(system)),
            gpu_manager,
            backend_manager,
            metrics: Arc::new(Mutex::new(MetricsSnapshot::default())),
            activity_logger,
            settings: Arc::new(Mutex::new(settings)),
            notifications: Arc::new(Mutex::new(Vec::new())),
            batch_jobs: Arc::new(Mutex::new(Vec::new())),
            security_manager,
            event_manager: Arc::new(Mutex::new(event_manager)),
            model_repository,
            download_manager,
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
        let mut event_mgr = self
            .event_manager
            .lock()
            .expect("Failed to lock event manager");
        *event_mgr = Some(EventManager::new(app_handle));
    }

    /// Perform cleanup when the application is shutting down
    pub async fn shutdown(&self) -> Result<(), String> {
        // Save settings to disk
        let settings = self
            .settings
            .lock()
            .map_err(|e| format!("Failed to lock settings: {}", e))?;

        let config_path = Self::get_config_path();
        let contents = serde_json::to_string_pretty(&*settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        tokio::fs::write(&config_path, contents)
            .await
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        // Unload all models
        let loaded_models = self.backend_manager.get_loaded_models();
        for model_id in loaded_models {
            let _ = self.backend_manager.unload_model(model_id).await;
        }

        // Stop GPU monitoring tasks (if any)
        self.gpu_manager.shutdown().await;

        Ok(())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Runtime::new()
            .expect("Failed to create Tokio runtime for AppState::default")
            .block_on(Self::new(None))
            .expect("Failed to initialize AppState")
    }
}
