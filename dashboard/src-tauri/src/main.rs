// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{menu::{Menu, MenuItem}, tray::TrayIconBuilder, Manager, State, Emitter};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::PathBuf;
use sysinfo::System;
use uuid::Uuid;

mod backend_manager;
mod activity_logger;
mod security;
mod events;
mod database;
mod model_repository;

use backend_manager::{BackendManager, ModelInfo, InferenceParams, GlobalMetrics};
use activity_logger::{ActivityLogger, ActivityLog, ActivityStats};
use security::{SecurityManager, ApiKey, SecurityEvent, SecurityMetrics, CreateApiKeyRequest, CreateApiKeyResponse};
use events::{EventManager, InfernoEvent, ConnectionStatus};
use database::{DatabaseManager, DbModel, DbBatchJob, DbNotification};
use model_repository::{ModelRepositoryService, ModelDownloadManager, ExternalModelInfo, ModelSearchQuery, ModelSearchResponse, DownloadProgress};

// ModelInfo is now imported from backend_manager

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemInfo {
    pub cpu_name: String,
    pub cpu_usage: f32,
    pub cpu_cores: usize,
    pub total_memory: u64,
    pub used_memory: u64,
    pub available_memory: u64,
    pub platform: String,
    pub arch: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MetricsSnapshot {
    pub inference_count: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub average_latency: f64,
    pub models_loaded: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InfernoMetrics {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub gpu_usage: Option<f32>,
    pub active_models: u32,
    pub active_inferences: u32,
    pub inference_count: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub average_latency: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActiveProcessInfo {
    pub active_models: Vec<String>,
    pub active_inferences: u32,
    pub batch_jobs: u32,
    pub streaming_sessions: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub types: NotificationTypes,
    pub priority_filter: String,
    pub auto_dismiss_after: u32,
    pub max_notifications: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotificationTypes {
    pub system: bool,
    pub inference: bool,
    pub security: bool,
    pub batch: bool,
    pub model: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotificationAction {
    pub label: String,
    pub url: Option<String>,
    pub callback: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: String, // 'info' | 'success' | 'warning' | 'error'
    pub timestamp: String,
    pub read: bool,
    pub action: Option<NotificationAction>,
    pub source: String, // 'system' | 'inference' | 'security' | 'batch' | 'model'
    pub priority: String, // 'low' | 'medium' | 'high' | 'critical'
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BatchJobConfig {
    pub inputs: Vec<String>,
    pub output_format: String,
    pub batch_size: u32,
    pub parallel_workers: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchJobMetrics {
    pub total_time: f64,
    pub avg_time_per_task: f64,
    pub throughput: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchJobResults {
    pub outputs: Vec<String>,
    pub errors: Vec<String>,
    pub metrics: BatchJobMetrics,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchJob {
    pub id: String,
    pub name: String,
    pub status: String, // 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
    pub model_id: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub progress: f64,
    pub total_tasks: u32,
    pub completed_tasks: u32,
    pub failed_tasks: u32,
    pub schedule: Option<String>,
    pub next_run: Option<String>,
    pub config: BatchJobConfig,
    pub results: Option<BatchJobResults>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    // Model Settings
    pub models_directory: String,
    pub auto_discover_models: bool,

    // Inference Settings
    pub default_temperature: f64,
    pub default_max_tokens: u32,
    pub default_top_p: f64,
    pub default_top_k: u32,

    // System Settings
    pub max_memory_usage: u32,
    pub prefer_gpu: bool,
    pub max_concurrent_inferences: u32,

    // Cache Settings
    pub enable_cache: bool,
    pub cache_directory: String,
    pub max_cache_size: u32,

    // API Settings
    pub enable_rest_api: bool,
    pub api_port: u32,
    pub enable_cors: bool,

    // Security Settings
    pub require_authentication: bool,
    pub enable_audit_log: bool,
    pub log_level: String,

    // Notification Settings
    pub notifications: NotificationSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            models_directory: "test_models/test_models".to_string(),
            auto_discover_models: true,
            default_temperature: 0.7,
            default_max_tokens: 512,
            default_top_p: 0.9,
            default_top_k: 40,
            max_memory_usage: 80,
            prefer_gpu: true,
            max_concurrent_inferences: 3,
            enable_cache: true,
            cache_directory: ".cache".to_string(),
            max_cache_size: 1024,
            enable_rest_api: false,
            api_port: 8080,
            enable_cors: true,
            require_authentication: false,
            enable_audit_log: true,
            log_level: "info".to_string(),
            notifications: NotificationSettings {
                enabled: true,
                types: NotificationTypes {
                    system: true,
                    inference: true,
                    security: true,
                    batch: true,
                    model: true,
                },
                priority_filter: "all".to_string(),
                auto_dismiss_after: 0,
                max_notifications: 100,
            },
        }
    }
}

// InferenceParams is now imported from backend_manager

// Application state
pub struct AppState {
    pub system: Arc<Mutex<System>>,
    pub backend_manager: Arc<BackendManager>,
    pub metrics: Arc<Mutex<MetricsSnapshot>>,
    pub activity_logger: Arc<ActivityLogger>,
    pub settings: Arc<Mutex<AppSettings>>,
    pub notifications: Arc<Mutex<Vec<Notification>>>,
    pub batch_jobs: Arc<Mutex<Vec<BatchJob>>>,
    pub security_manager: Arc<SecurityManager>,
    pub event_manager: Arc<Mutex<Option<EventManager>>>,
    pub database: Arc<DatabaseManager>,
    pub model_repository: Arc<ModelRepositoryService>,
    pub download_manager: Arc<ModelDownloadManager>,
}

// Configuration management functions
fn get_config_path() -> PathBuf {
    // For Tauri v2, we'll use a simple approach - store config in current directory
    // In production, you might want to use a proper config directory
    PathBuf::from(".").join("inferno-settings.json")
}

async fn load_settings() -> AppSettings {
    let config_path = get_config_path();

    if let Ok(contents) = tokio::fs::read_to_string(&config_path).await {
        if let Ok(settings) = serde_json::from_str::<AppSettings>(&contents) {
            return settings;
        }
    }

    // Return default settings if file doesn't exist or can't be parsed
    AppSettings::default()
}

async fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let config_path = get_config_path();

    // Create parent directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            return Err(format!("Failed to create config directory: {}", e));
        }
    }

    let contents = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    tokio::fs::write(&config_path, contents).await
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    Ok(())
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_system_info(state: State<'_, AppState>) -> Result<SystemInfo, String> {
    let mut system = state.system.lock().map_err(|e| e.to_string())?;
    system.refresh_all();

    let cpu_usage = system.global_cpu_info().cpu_usage();
    let total_memory = system.total_memory(); // Already in bytes in sysinfo 0.30
    let used_memory = system.used_memory(); // Already in bytes in sysinfo 0.30
    let available_memory = total_memory - used_memory;

    Ok(SystemInfo {
        cpu_name: system.global_cpu_info().brand().to_string(),
        cpu_usage,
        cpu_cores: system.cpus().len(),
        total_memory,
        used_memory,
        available_memory,
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    })
}

#[tauri::command]
async fn get_metrics(state: State<'_, AppState>) -> Result<MetricsSnapshot, String> {
    let global_metrics = state.backend_manager.get_metrics();

    // Convert global metrics to our UI metrics format
    let ui_metrics = MetricsSnapshot {
        inference_count: global_metrics.inference_count,
        success_count: global_metrics.success_count,
        error_count: global_metrics.error_count,
        average_latency: global_metrics.average_latency,
        models_loaded: global_metrics.models_loaded,
    };

    Ok(ui_metrics)
}

#[tauri::command]
async fn get_models(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    state.backend_manager.discover_models().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_loaded_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state.backend_manager.get_loaded_models())
}

#[tauri::command]
async fn load_model(
    model_name: String,
    backend_type: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    match state.backend_manager.load_model(model_name.clone(), backend_type.clone()).await {
        Ok(backend_id) => {
            // Emit model loaded event
            if let Ok(event_mgr) = state.event_manager.lock() {
                if let Some(ref manager) = *event_mgr {
                    let _ = manager.emit_model_loaded(model_name, backend_id.clone());
                }
            }
            Ok(backend_id)
        }
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
async fn unload_model(
    backend_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    match state.backend_manager.unload_model(backend_id.clone()).await {
        Ok(()) => {
            // Emit model unloaded event
            if let Ok(event_mgr) = state.event_manager.lock() {
                if let Some(ref manager) = *event_mgr {
                    let _ = manager.emit_model_unloaded("unknown".to_string(), backend_id);
                }
            }
            Ok(())
        }
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
async fn infer(
    backend_id: String,
    prompt: String,
    params: InferenceParams,
    state: State<'_, AppState>
) -> Result<String, String> {
    let inference_id = uuid::Uuid::new_v4().to_string();

    // Emit inference started event
    if let Ok(event_mgr) = state.event_manager.lock() {
        if let Some(ref manager) = *event_mgr {
            let _ = manager.emit_inference_started(inference_id.clone(), backend_id.clone());
        }
    }

    match state.backend_manager.infer(backend_id, prompt, params).await {
        Ok(response) => {
            // Emit inference completed event
            if let Ok(event_mgr) = state.event_manager.lock() {
                if let Some(ref manager) = *event_mgr {
                    let _ = manager.emit_inference_completed(inference_id, response.clone(), 100); // Mock latency
                }
            }
            Ok(response)
        }
        Err(e) => {
            // Emit inference error event
            if let Ok(event_mgr) = state.event_manager.lock() {
                if let Some(ref manager) = *event_mgr {
                    let _ = manager.emit_inference_error(inference_id, e.to_string());
                }
            }
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn infer_stream(
    app: tauri::AppHandle,
    backend_id: String,
    prompt: String,
    params: InferenceParams,
    state: State<'_, AppState>
) -> Result<String, String> {
    use tokio::time::{sleep, Duration};

    // Generate a unique inference ID for this session
    let inference_id = uuid::Uuid::new_v4().to_string();

    // Emit the start event
    let _ = app.emit("inference_start", &inference_id);

    // Start streaming inference in the background
    let app_clone = app.clone();
    let inference_id_clone = inference_id.clone();
    let backend_manager = state.backend_manager.clone();

    tokio::spawn(async move {
        match backend_manager.infer(backend_id, prompt, params).await {
            Ok(response) => {
                // Simulate streaming by sending the response in chunks
                let words: Vec<&str> = response.split_whitespace().collect();
                let chunk_size = std::cmp::max(1, words.len() / 20); // Send in ~20 chunks

                for chunk in words.chunks(chunk_size) {
                    let text = chunk.join(" ");
                    if !text.is_empty() {
                        let _ = app_clone.emit("inference_token", serde_json::json!({
                            "inference_id": inference_id_clone,
                            "token": text + " "
                        }));

                        // Small delay to simulate real streaming
                        sleep(Duration::from_millis(50)).await;
                    }
                }

                // Emit completion event
                let _ = app_clone.emit("inference_complete", serde_json::json!({
                    "inference_id": inference_id_clone,
                    "response": response
                }));
            }
            Err(e) => {
                // Emit error event
                let _ = app_clone.emit("inference_error", serde_json::json!({
                    "inference_id": inference_id_clone,
                    "error": e.to_string()
                }));
            }
        }
    });

    Ok(inference_id)
}

#[tauri::command]
async fn validate_model(model_path: String, state: State<'_, AppState>) -> Result<bool, String> {
    state.backend_manager.validate_model(model_path).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_file_dialog(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};

    let file_path = app.dialog()
        .file()
        .add_filter("Model Files", &["gguf", "onnx", "pt", "safetensors"])
        .add_filter("GGUF Files", &["gguf"])
        .add_filter("ONNX Files", &["onnx"])
        .add_filter("All Files", &["*"])
        .set_title("Select Model File")
        .blocking_pick_file();

    Ok(file_path.map(|p| p.as_path().unwrap().to_string_lossy().to_string()))
}

#[tauri::command]
async fn upload_model(
    state: State<'_, AppState>,
    source_path: String,
    target_name: Option<String>
) -> Result<String, String> {
    use std::path::Path;
    use tokio::fs;

    let source = Path::new(&source_path);
    if !source.exists() {
        return Err("Source file does not exist".to_string());
    }

    // Get models directory from config - use default for now
    let models_dir = Path::new("test_models/test_models");

    // Create models directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&models_dir).await {
        return Err(format!("Failed to create models directory: {}", e));
    }

    // Determine target filename
    let target_filename = target_name.unwrap_or_else(|| {
        source.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "model".to_string())
    });

    let target_path = models_dir.join(&target_filename);

    // Copy the file
    if let Err(e) = fs::copy(&source, &target_path).await {
        return Err(format!("Failed to copy model file: {}", e));
    }

    // Log the upload
    state.activity_logger.log_model_operation(
        crate::activity_logger::ActivityType::ModelUpload,
        &target_filename,
        crate::activity_logger::ActivityStatus::Success,
        Some(&format!("Uploaded from {}", source_path))
    );

    Ok(target_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn get_recent_activities(state: State<'_, AppState>, limit: Option<usize>) -> Result<Vec<ActivityLog>, String> {
    let activities = state.activity_logger.get_recent_activities(limit.unwrap_or(50));
    Ok(activities)
}

#[tauri::command]
async fn get_activity_stats(state: State<'_, AppState>) -> Result<ActivityStats, String> {
    let stats = state.activity_logger.get_stats();
    Ok(stats)
}

#[tauri::command]
async fn clear_activities(state: State<'_, AppState>) -> Result<(), String> {
    state.activity_logger.clear();
    Ok(())
}

#[tauri::command]
async fn get_inferno_metrics(state: State<'_, AppState>) -> Result<InfernoMetrics, String> {
    let mut system = state.system.lock().map_err(|e| e.to_string())?;
    system.refresh_all();

    let global_metrics = state.backend_manager.get_metrics();
    let loaded_backends = state.backend_manager.get_loaded_models();

    // Calculate Inferno-specific CPU and memory usage
    // This is a simplified calculation - in production you'd track process-specific usage
    let base_cpu = system.global_cpu_info().cpu_usage();
    let inferno_cpu = if loaded_backends.is_empty() { 0.0 } else { base_cpu * 0.3 }; // Estimated usage

    let base_memory = system.used_memory();
    let inferno_memory = if loaded_backends.is_empty() {
        0
    } else {
        // Estimate memory per loaded model (simplified)
        (loaded_backends.len() as u64) * 512 * 1024 * 1024 // 512MB per model estimate
    };

    Ok(InfernoMetrics {
        cpu_usage: inferno_cpu,
        memory_usage: inferno_memory,
        gpu_usage: None, // Would need GPU-specific tracking
        active_models: loaded_backends.len() as u32,
        active_inferences: 0, // Would track from streaming sessions
        inference_count: global_metrics.inference_count,
        success_count: global_metrics.success_count,
        error_count: global_metrics.error_count,
        average_latency: global_metrics.average_latency,
    })
}

#[tauri::command]
async fn get_active_processes(state: State<'_, AppState>) -> Result<ActiveProcessInfo, String> {
    let loaded_models = state.backend_manager.get_loaded_models();

    // Get real batch job count
    let batch_jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;
    let active_batch_jobs = batch_jobs.iter()
        .filter(|j| j.status == "running" || j.status == "pending")
        .count() as u32;

    Ok(ActiveProcessInfo {
        active_models: loaded_models,
        active_inferences: 0, // Would track from streaming inference sessions
        batch_jobs: active_batch_jobs,
        streaming_sessions: 0, // Would track from active streaming sessions
    })
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
async fn set_settings(settings: AppSettings, state: State<'_, AppState>) -> Result<(), String> {
    // Save to file
    save_settings(&settings).await?;

    // Update in-memory state
    let mut state_settings = state.settings.lock().map_err(|e| e.to_string())?;
    *state_settings = settings;

    Ok(())
}

// Notification management commands
#[tauri::command]
async fn get_notifications(state: State<'_, AppState>) -> Result<Vec<Notification>, String> {
    // Try to get from database first, fallback to in-memory if needed
    match state.database.get_notifications(Some(100)).await {
        Ok(db_notifications) => {
            // Convert database notifications to UI format
            let ui_notifications = db_notifications.into_iter().map(|db_notif| {
                Notification {
                    id: db_notif.id,
                    title: db_notif.title,
                    message: db_notif.message,
                    notification_type: db_notif.notification_type,
                    timestamp: db_notif.created_at.to_rfc3339(),
                    read: db_notif.read,
                    action: None, // Could parse from metadata if needed
                    source: db_notif.source,
                    priority: db_notif.priority,
                    metadata: if let Some(metadata_str) = &db_notif.metadata { serde_json::from_str(metadata_str).unwrap_or_default() } else { std::collections::HashMap::new() },
                }
            }).collect();
            Ok(ui_notifications)
        }
        Err(_) => {
            // Fallback to in-memory storage
            let notifications = state.notifications.lock().map_err(|e| e.to_string())?;
            Ok(notifications.clone())
        }
    }
}

#[tauri::command]
async fn get_unread_notification_count(state: State<'_, AppState>) -> Result<u32, String> {
    let notifications = state.notifications.lock().map_err(|e| e.to_string())?;
    let unread_count = notifications.iter().filter(|n| !n.read).count() as u32;
    Ok(unread_count)
}

#[tauri::command]
async fn mark_notification_as_read(notification_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut notifications = state.notifications.lock().map_err(|e| e.to_string())?;

    if let Some(notification) = notifications.iter_mut().find(|n| n.id == notification_id) {
        notification.read = true;
    }

    Ok(())
}

#[tauri::command]
async fn mark_all_notifications_as_read(state: State<'_, AppState>) -> Result<(), String> {
    let mut notifications = state.notifications.lock().map_err(|e| e.to_string())?;

    for notification in notifications.iter_mut() {
        notification.read = true;
    }

    Ok(())
}

#[tauri::command]
async fn dismiss_notification(notification_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut notifications = state.notifications.lock().map_err(|e| e.to_string())?;
    notifications.retain(|n| n.id != notification_id);
    Ok(())
}

#[tauri::command]
async fn clear_all_notifications(state: State<'_, AppState>) -> Result<(), String> {
    let mut notifications = state.notifications.lock().map_err(|e| e.to_string())?;
    notifications.clear();
    Ok(())
}

#[tauri::command]
async fn create_notification(
    notification: serde_json::Value,
    state: State<'_, AppState>
) -> Result<String, String> {
    let notification_id = Uuid::new_v4().to_string();

    // Create database notification
    let db_notification = DbNotification {
        id: notification_id.clone(),
        title: notification["title"].as_str().unwrap_or("").to_string(),
        message: notification["message"].as_str().unwrap_or("").to_string(),
        notification_type: notification["type"].as_str().unwrap_or("info").to_string(),
        source: notification["source"].as_str().unwrap_or("system").to_string(),
        priority: notification["priority"].as_str().unwrap_or("medium").to_string(),
        metadata: Some(serde_json::to_string(&HashMap::<String, serde_json::Value>::new()).unwrap_or_default()),
        action_data: None,
        read: false,
        created_at: chrono::Utc::now(),
    };

    // Try to save to database first
    match state.database.create_notification(&db_notification).await {
        Ok(()) => {
            // Emit notification created event
            if let Ok(event_mgr) = state.event_manager.lock() {
                if let Some(ref manager) = *event_mgr {
                    // Create UI notification for event emission
                    let ui_notification = Notification {
                        id: db_notification.id.clone(),
                        title: db_notification.title.clone(),
                        message: db_notification.message.clone(),
                        notification_type: db_notification.notification_type.clone(),
                        timestamp: db_notification.created_at.to_rfc3339(),
                        read: db_notification.read,
                        action: None,
                        source: db_notification.source.clone(),
                        priority: db_notification.priority.clone(),
                        metadata: HashMap::new(),
                    };
                    let _ = manager.emit_notification(ui_notification);
                }
            }
            Ok(notification_id)
        }
        Err(_) => {
            // Fallback to in-memory storage
            let new_notification = Notification {
                id: notification_id.clone(),
                title: db_notification.title,
                message: db_notification.message,
                notification_type: db_notification.notification_type,
                timestamp: db_notification.created_at.to_rfc3339(),
                read: false,
                action: None,
                source: db_notification.source,
                priority: db_notification.priority,
                metadata: HashMap::new(),
            };

            let mut notifications = state.notifications.lock().map_err(|e| e.to_string())?;
            let settings = state.settings.lock().map_err(|e| e.to_string())?;

            // Respect max notifications limit
            if notifications.len() >= settings.notifications.max_notifications as usize {
                // Remove oldest notification
                notifications.remove(0);
            }

            notifications.push(new_notification);
            Ok(notification_id)
        }
    }
}

// Batch job management commands
#[tauri::command]
async fn get_batch_jobs(state: State<'_, AppState>) -> Result<Vec<BatchJob>, String> {
    // Try to get from database first, fallback to in-memory if needed
    match state.database.get_batch_jobs().await {
        Ok(db_jobs) => {
            // Convert database batch jobs to UI format
            let ui_jobs = db_jobs.into_iter().map(|db_job| {
                let config = serde_json::from_str::<BatchJobConfig>(&db_job.config).unwrap_or_default();
                let results = if let Some(results_str) = &db_job.results {
                    serde_json::from_str::<BatchJobResults>(results_str).ok()
                } else {
                    None
                };

                BatchJob {
                    id: db_job.id,
                    name: db_job.name,
                    status: db_job.status,
                    model_id: db_job.model_id,
                    created_at: db_job.created_at.to_rfc3339(),
                    started_at: db_job.started_at.map(|dt| dt.to_rfc3339()),
                    completed_at: db_job.completed_at.map(|dt| dt.to_rfc3339()),
                    progress: db_job.progress,
                    total_tasks: db_job.total_tasks as u32,
                    completed_tasks: db_job.completed_tasks as u32,
                    failed_tasks: db_job.failed_tasks as u32,
                    schedule: db_job.schedule,
                    next_run: db_job.next_run.map(|dt| dt.to_rfc3339()),
                    config,
                    results,
                }
            }).collect();
            Ok(ui_jobs)
        }
        Err(_) => {
            // Fallback to in-memory storage
            let batch_jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;
            Ok(batch_jobs.clone())
        }
    }
}

#[tauri::command]
async fn get_batch_job(job_id: String, state: State<'_, AppState>) -> Result<Option<BatchJob>, String> {
    let batch_jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;
    let job = batch_jobs.iter().find(|j| j.id == job_id).cloned();
    Ok(job)
}

#[tauri::command]
async fn create_batch_job(
    job_data: serde_json::Value,
    state: State<'_, AppState>
) -> Result<String, String> {
    let job_id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    let new_job = BatchJob {
        id: job_id.clone(),
        name: job_data["name"].as_str().unwrap_or("Unnamed Job").to_string(),
        status: "pending".to_string(),
        model_id: job_data["model_id"].as_str().unwrap_or("").to_string(),
        created_at: timestamp,
        started_at: None,
        completed_at: None,
        progress: 0.0,
        total_tasks: job_data["inputs"].as_array().map(|a| a.len() as u32).unwrap_or(0),
        completed_tasks: 0,
        failed_tasks: 0,
        schedule: job_data["schedule"].as_str().map(|s| s.to_string()),
        next_run: None,
        config: BatchJobConfig {
            inputs: job_data["inputs"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            output_format: job_data["output_format"].as_str().unwrap_or("text").to_string(),
            batch_size: job_data["batch_size"].as_u64().unwrap_or(1) as u32,
            parallel_workers: job_data["parallel_workers"].as_u64().unwrap_or(1) as u32,
        },
        results: None,
    };

    let mut batch_jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;
    batch_jobs.push(new_job);

    Ok(job_id)
}

#[tauri::command]
async fn start_batch_job(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut batch_jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;

    if let Some(job) = batch_jobs.iter_mut().find(|j| j.id == job_id) {
        job.status = "running".to_string();
        job.started_at = Some(chrono::Utc::now().to_rfc3339());

        // Simulate progress updates (in a real implementation, this would be done by a background worker)
        job.progress = 25.0;
        job.completed_tasks = job.total_tasks / 4;
    }

    Ok(())
}

#[tauri::command]
async fn pause_batch_job(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut batch_jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;

    if let Some(job) = batch_jobs.iter_mut().find(|j| j.id == job_id) {
        if job.status == "running" {
            job.status = "pending".to_string();
        }
    }

    Ok(())
}

#[tauri::command]
async fn cancel_batch_job(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut batch_jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;

    if let Some(job) = batch_jobs.iter_mut().find(|j| j.id == job_id) {
        job.status = "cancelled".to_string();
        job.completed_at = Some(chrono::Utc::now().to_rfc3339());
    }

    Ok(())
}

#[tauri::command]
async fn delete_batch_job(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut batch_jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;
    batch_jobs.retain(|j| j.id != job_id);
    Ok(())
}

#[tauri::command]
async fn get_batch_job_count(state: State<'_, AppState>) -> Result<u32, String> {
    let batch_jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;
    Ok(batch_jobs.len() as u32)
}

#[tauri::command]
async fn get_active_batch_job_count(state: State<'_, AppState>) -> Result<u32, String> {
    let batch_jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;
    let active_count = batch_jobs.iter()
        .filter(|j| j.status == "running" || j.status == "pending")
        .count() as u32;
    Ok(active_count)
}

// Security management commands
#[tauri::command]
async fn create_api_key(
    request: CreateApiKeyRequest,
    state: State<'_, AppState>
) -> Result<CreateApiKeyResponse, String> {
    match state.security_manager.generate_api_key(request.clone()) {
        Ok(response) => {
            // Emit API key created event
            if let Ok(event_mgr) = state.event_manager.lock() {
                if let Some(ref manager) = *event_mgr {
                    let _ = manager.emit_api_key_created(
                        response.api_key.id.clone(),
                        request.name,
                        request.permissions
                    );
                }
            }
            Ok(response)
        }
        Err(e) => Err(e)
    }
}

#[tauri::command]
async fn get_api_keys(state: State<'_, AppState>) -> Result<Vec<ApiKey>, String> {
    state.security_manager.get_api_keys()
}

#[tauri::command]
async fn revoke_api_key(key_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.security_manager.revoke_api_key(key_id)
}

#[tauri::command]
async fn delete_api_key(key_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.security_manager.delete_api_key(key_id)
}

#[tauri::command]
async fn validate_api_key(raw_key: String, state: State<'_, AppState>) -> Result<Option<ApiKey>, String> {
    state.security_manager.validate_api_key(raw_key)
}

#[tauri::command]
async fn get_security_events(
    limit: Option<usize>,
    state: State<'_, AppState>
) -> Result<Vec<SecurityEvent>, String> {
    state.security_manager.get_security_events(limit)
}

#[tauri::command]
async fn get_security_metrics(state: State<'_, AppState>) -> Result<SecurityMetrics, String> {
    state.security_manager.get_security_metrics()
}

#[tauri::command]
async fn clear_security_events(state: State<'_, AppState>) -> Result<(), String> {
    state.security_manager.clear_security_events()
}

// Model repository management commands
#[tauri::command]
async fn search_external_models(
    query: ModelSearchQuery,
    state: State<'_, AppState>
) -> Result<ModelSearchResponse, String> {
    state.model_repository.search_models(query).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_external_model_details(
    model_id: String,
    state: State<'_, AppState>
) -> Result<ExternalModelInfo, String> {
    state.model_repository.get_model_details(&model_id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_featured_models(state: State<'_, AppState>) -> Result<Vec<ExternalModelInfo>, String> {
    state.model_repository.get_featured_models().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_trending_models(state: State<'_, AppState>) -> Result<Vec<ExternalModelInfo>, String> {
    state.model_repository.get_trending_models().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_model_download(
    model: ExternalModelInfo,
    target_dir: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    state.download_manager.start_download(&model, &target_dir).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_download_progress(
    download_id: String,
    state: State<'_, AppState>
) -> Result<Option<DownloadProgress>, String> {
    Ok(state.download_manager.get_download_progress(&download_id))
}

#[tauri::command]
async fn get_all_downloads(state: State<'_, AppState>) -> Result<Vec<DownloadProgress>, String> {
    Ok(state.download_manager.get_all_downloads())
}

#[tauri::command]
async fn cancel_download(
    download_id: String,
    state: State<'_, AppState>
) -> Result<bool, String> {
    Ok(state.download_manager.cancel_download(&download_id))
}

// Initialize metrics with default values
fn initialize_metrics() -> MetricsSnapshot {
    MetricsSnapshot {
        inference_count: 0,
        success_count: 0,
        error_count: 0,
        average_latency: 0.0,
        models_loaded: 0,
    }
}

#[tokio::main]
async fn main() {
    let activity_logger = Arc::new(ActivityLogger::new(100));

    // Initialize the backend manager with activity logger
    let backend_manager = match BackendManager::new(activity_logger.clone()).await {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            eprintln!("Failed to initialize backend manager: {}", e);
            std::process::exit(1);
        }
    };

    // Load settings
    let settings = load_settings().await;

    // Create some sample notifications for testing
    let mut sample_notifications = Vec::new();
    sample_notifications.push(Notification {
        id: Uuid::new_v4().to_string(),
        title: "System Started".to_string(),
        message: "Inferno AI platform has started successfully".to_string(),
        notification_type: "success".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        read: false,
        action: None,
        source: "system".to_string(),
        priority: "medium".to_string(),
        metadata: HashMap::new(),
    });

    sample_notifications.push(Notification {
        id: Uuid::new_v4().to_string(),
        title: "Model Discovery".to_string(),
        message: "Found 3 available models in the models directory".to_string(),
        notification_type: "info".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        read: false,
        action: None,
        source: "model".to_string(),
        priority: "low".to_string(),
        metadata: HashMap::new(),
    });

    // Create some sample batch jobs for testing
    let mut sample_batch_jobs = Vec::new();
    sample_batch_jobs.push(BatchJob {
        id: Uuid::new_v4().to_string(),
        name: "Product Descriptions".to_string(),
        status: "running".to_string(),
        model_id: "llama-7b-q4".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        started_at: Some(chrono::Utc::now().to_rfc3339()),
        completed_at: None,
        progress: 65.0,
        total_tasks: 20,
        completed_tasks: 13,
        failed_tasks: 0,
        schedule: None,
        next_run: None,
        config: BatchJobConfig {
            inputs: vec![
                "Write a product description for a laptop".to_string(),
                "Write a product description for a phone".to_string(),
            ],
            output_format: "markdown".to_string(),
            batch_size: 5,
            parallel_workers: 2,
        },
        results: None,
    });

    sample_batch_jobs.push(BatchJob {
        id: Uuid::new_v4().to_string(),
        name: "Content Generation".to_string(),
        status: "pending".to_string(),
        model_id: "llama-7b-q4".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        started_at: None,
        completed_at: None,
        progress: 0.0,
        total_tasks: 50,
        completed_tasks: 0,
        failed_tasks: 0,
        schedule: Some("0 2 * * *".to_string()), // Daily at 2 AM
        next_run: Some("2024-01-16T02:00:00Z".to_string()),
        config: BatchJobConfig {
            inputs: vec!["Generate blog post about AI".to_string()],
            output_format: "html".to_string(),
            batch_size: 10,
            parallel_workers: 3,
        },
        results: None,
    });

    // Initialize security manager with sample data
    let security_manager = Arc::new(SecurityManager::new());
    if let Err(e) = security_manager.initialize_with_sample_data() {
        eprintln!("Failed to initialize security manager with sample data: {}", e);
    }

    // Initialize database manager
    let database_manager = match DatabaseManager::new(None).await {
        Ok(db) => Arc::new(db),
        Err(e) => {
            eprintln!("Failed to initialize database manager: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize model repository service
    let model_repository = Arc::new(ModelRepositoryService::new());

    // Initialize download manager
    let download_manager = Arc::new(ModelDownloadManager::new());

    let app_state = AppState {
        system: Arc::new(Mutex::new(System::new_all())),
        backend_manager,
        metrics: Arc::new(Mutex::new(initialize_metrics())),
        activity_logger,
        settings: Arc::new(Mutex::new(settings)),
        notifications: Arc::new(Mutex::new(sample_notifications)),
        batch_jobs: Arc::new(Mutex::new(sample_batch_jobs)),
        security_manager,
        event_manager: Arc::new(Mutex::new(None)),
        database: database_manager,
        model_repository,
        download_manager,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state)
        .setup(|app| {
            // Initialize EventManager now that we have access to AppHandle
            let state: State<AppState> = app.state();
            let event_manager = EventManager::new(app.handle().clone(), state.system.clone());

            // Start the metrics emission
            if let Err(e) = event_manager.start_metrics_emission() {
                eprintln!("Failed to start metrics emission: {}", e);
            }

            // Set the EventManager in the app state
            if let Ok(mut event_mgr) = state.event_manager.lock() {
                *event_mgr = Some(event_manager);
            }

            // Create the menu
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;

            // Create system tray
            let _tray = TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_system_info,
            get_metrics,
            get_models,
            get_loaded_models,
            load_model,
            unload_model,
            infer,
            infer_stream,
            validate_model,
            open_file_dialog,
            upload_model,
            get_recent_activities,
            get_activity_stats,
            clear_activities,
            get_inferno_metrics,
            get_active_processes,
            get_settings,
            set_settings,
            get_notifications,
            get_unread_notification_count,
            mark_notification_as_read,
            mark_all_notifications_as_read,
            dismiss_notification,
            clear_all_notifications,
            create_notification,
            get_batch_jobs,
            get_batch_job,
            create_batch_job,
            start_batch_job,
            pause_batch_job,
            cancel_batch_job,
            delete_batch_job,
            get_batch_job_count,
            get_active_batch_job_count,
            create_api_key,
            get_api_keys,
            revoke_api_key,
            delete_api_key,
            validate_api_key,
            get_security_events,
            get_security_metrics,
            clear_security_events,
            search_external_models,
            get_external_model_details,
            get_featured_models,
            get_trending_models,
            start_model_download,
            get_download_progress,
            get_all_downloads,
            cancel_download
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}