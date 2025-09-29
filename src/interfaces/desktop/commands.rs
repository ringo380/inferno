//! Tauri Command Handlers
//!
//! This module contains all 51 Tauri command handlers migrated from dashboard/src-tauri.
//! Commands are organized by functionality for maintainability.
//!
//! ## Command Categories:
//! - Core Model Operations (5 commands)
//! - Inference Operations (2 commands)
//! - System Information (4 commands)
//! - File Operations (2 commands)
//! - Settings Management (2 commands)
//! - Activity Logging (3 commands)
//! - Notifications (7 commands)
//! - Batch Jobs (9 commands)
//! - Security/API Keys (8 commands)
//! - Model Repository (10 commands)

use tauri::{command, AppHandle, State, Window, Emitter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::types::{
    SystemInfo, MetricsSnapshot, InfernoMetrics, ActiveProcessInfo,
    AppSettings, Notification, NotificationAction, BatchJob, BatchJobConfig,
};

use super::{
    AppState, ActivityLog, ActivityStats, BackendManager, ModelInfo, InferenceParams,
    GlobalMetrics, SecurityManager, ApiKey, SecurityEvent, SecurityMetrics,
    CreateApiKeyRequest, CreateApiKeyResponse, ExternalModelInfo, ModelSearchQuery,
    ModelSearchResponse, DownloadProgress,
};

// ============================================================================
// Core Model Operations (5 commands)
// ============================================================================

#[command]
pub async fn get_models(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    state.backend_manager.discover_models().await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_loaded_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state.backend_manager.get_loaded_models())
}

#[command]
pub async fn load_model(
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

#[command]
pub async fn unload_model(
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

#[command]
pub async fn get_model_info(
    backend_id: String,
    state: State<'_, AppState>
) -> Result<Option<ModelInfo>, String> {
    Ok(state.backend_manager.get_model_info(&backend_id))
}

// ============================================================================
// Inference Operations (2 commands)
// ============================================================================

#[command]
pub async fn infer(
    backend_id: String,
    prompt: String,
    params: InferenceParams,
    state: State<'_, AppState>
) -> Result<String, String> {
    let inference_id = Uuid::new_v4().to_string();

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

#[command]
pub async fn infer_stream(
    app: AppHandle,
    backend_id: String,
    prompt: String,
    params: InferenceParams,
    state: State<'_, AppState>
) -> Result<String, String> {
    use tokio::time::{sleep, Duration};

    // Generate a unique inference ID for this session
    let inference_id = Uuid::new_v4().to_string();

    // Emit the start event
    let _ = app.emit("inference_start", &inference_id);

    // Start streaming inference in the background
    let app_clone = app.clone();
    let inference_id_clone = inference_id.clone();
    let backend_id_clone = backend_id.clone();

    tokio::spawn(async move {
        // Get the stream from backend manager
        match state.backend_manager.infer_stream(backend_id_clone.clone(), prompt, params).await {
            Ok(mut stream) => {
                use futures::StreamExt;

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(token) => {
                            let _ = app_clone.emit("inference_token", serde_json::json!({
                                "inference_id": inference_id_clone,
                                "token": token
                            }));

                            // Small delay to prevent overwhelming the frontend
                            sleep(Duration::from_millis(10)).await;
                        }
                        Err(e) => {
                            let _ = app_clone.emit("inference_error", serde_json::json!({
                                "inference_id": inference_id_clone,
                                "error": e.to_string()
                            }));
                            break;
                        }
                    }
                }

                // Emit completion event
                let _ = app_clone.emit("inference_complete", &inference_id_clone);
            }
            Err(e) => {
                let _ = app_clone.emit("inference_error", serde_json::json!({
                    "inference_id": inference_id_clone,
                    "error": e.to_string()
                }));
            }
        }
    });

    Ok(inference_id)
}

// ============================================================================
// System Information (4 commands)
// ============================================================================

#[command]
pub async fn get_system_info(state: State<'_, AppState>) -> Result<SystemInfo, String> {
    let mut system = state.system.lock().map_err(|e| e.to_string())?;
    system.refresh_all();

    let cpu_usage = system.global_cpu_info().cpu_usage();
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
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

#[command]
pub async fn get_metrics(state: State<'_, AppState>) -> Result<MetricsSnapshot, String> {
    let global_metrics = state.backend_manager.get_metrics();

    Ok(MetricsSnapshot {
        inference_count: global_metrics.inference_count,
        success_count: global_metrics.success_count,
        error_count: global_metrics.error_count,
        average_latency: global_metrics.average_latency,
        models_loaded: global_metrics.models_loaded,
    })
}

#[command]
pub async fn get_inferno_metrics(state: State<'_, AppState>) -> Result<InfernoMetrics, String> {
    let mut system = state.system.lock().map_err(|e| e.to_string())?;
    system.refresh_all();

    let cpu_usage = system.global_cpu_info().cpu_usage();
    let memory_usage = system.used_memory();

    let global_metrics = state.backend_manager.get_metrics();

    Ok(InfernoMetrics {
        cpu_usage,
        memory_usage,
        gpu_usage: None, // TODO: Implement GPU usage detection
        active_models: global_metrics.models_loaded,
        active_inferences: 0, // TODO: Track active inferences
        inference_count: global_metrics.inference_count,
        success_count: global_metrics.success_count,
        error_count: global_metrics.error_count,
        average_latency: global_metrics.average_latency,
    })
}

#[command]
pub async fn get_active_processes(state: State<'_, AppState>) -> Result<ActiveProcessInfo, String> {
    let loaded_models = state.backend_manager.get_loaded_models();
    let batch_jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;
    let running_jobs = batch_jobs.iter().filter(|j| j.status == "running").count() as u32;

    Ok(ActiveProcessInfo {
        active_models: loaded_models,
        active_inferences: 0, // TODO: Track active inferences
        batch_jobs: running_jobs,
        streaming_sessions: 0, // TODO: Track streaming sessions
    })
}

// ============================================================================
// File Operations (2 commands)
// ============================================================================

#[command]
pub async fn read_file(path: String) -> Result<String, String> {
    tokio::fs::read_to_string(&path).await
        .map_err(|e| format!("Failed to read file {}: {}", path, e))
}

#[command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(&path).parent() {
        tokio::fs::create_dir_all(parent).await
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    tokio::fs::write(&path, content).await
        .map_err(|e| format!("Failed to write file {}: {}", path, e))
}

// ============================================================================
// Settings Management (2 commands)
// ============================================================================

#[command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[command]
pub async fn update_settings(
    settings: AppSettings,
    state: State<'_, AppState>
) -> Result<(), String> {
    // Update in-memory settings
    let mut current_settings = state.settings.lock().map_err(|e| e.to_string())?;
    *current_settings = settings.clone();
    drop(current_settings);

    // Save to disk
    let config_path = std::path::PathBuf::from(".").join("inferno-settings.json");
    let contents = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    tokio::fs::write(&config_path, contents).await
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    Ok(())
}

// ============================================================================
// Activity Logging (3 commands)
// ============================================================================

#[command]
pub async fn get_activity_logs(
    filter: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>
) -> Result<Vec<ActivityLog>, String> {
    state.activity_logger.get_logs(filter, limit)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_activity_stats(state: State<'_, AppState>) -> Result<ActivityStats, String> {
    state.activity_logger.get_stats()
        .map_err(|e| e.to_string())
}

#[command]
pub async fn clear_activity_logs(state: State<'_, AppState>) -> Result<(), String> {
    state.activity_logger.clear_logs()
        .map_err(|e| e.to_string())
}

// ============================================================================
// Notifications (7 commands)
// ============================================================================

#[command]
pub async fn get_notifications(state: State<'_, AppState>) -> Result<Vec<Notification>, String> {
    let notifications = state.notifications.lock().map_err(|e| e.to_string())?;
    Ok(notifications.clone())
}

#[command]
pub async fn get_unread_notifications(state: State<'_, AppState>) -> Result<Vec<Notification>, String> {
    let notifications = state.notifications.lock().map_err(|e| e.to_string())?;
    Ok(notifications.iter().filter(|n| !n.read).cloned().collect())
}

#[command]
pub async fn mark_notification_read(
    notification_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let mut notifications = state.notifications.lock().map_err(|e| e.to_string())?;
    if let Some(notification) = notifications.iter_mut().find(|n| n.id == notification_id) {
        notification.read = true;
    }
    Ok(())
}

#[command]
pub async fn mark_all_notifications_read(state: State<'_, AppState>) -> Result<(), String> {
    let mut notifications = state.notifications.lock().map_err(|e| e.to_string())?;
    for notification in notifications.iter_mut() {
        notification.read = true;
    }
    Ok(())
}

#[command]
pub async fn delete_notification(
    notification_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let mut notifications = state.notifications.lock().map_err(|e| e.to_string())?;
    notifications.retain(|n| n.id != notification_id);
    Ok(())
}

#[command]
pub async fn clear_notifications(state: State<'_, AppState>) -> Result<(), String> {
    let mut notifications = state.notifications.lock().map_err(|e| e.to_string())?;
    notifications.clear();
    Ok(())
}

#[command]
pub async fn create_notification(
    notification: Notification,
    state: State<'_, AppState>
) -> Result<(), String> {
    let mut notifications = state.notifications.lock().map_err(|e| e.to_string())?;

    // Check max notifications limit
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    let max_notifications = settings.notifications.max_notifications as usize;

    if notifications.len() >= max_notifications {
        // Remove oldest notification
        notifications.remove(0);
    }

    notifications.push(notification);
    Ok(())
}

// ============================================================================
// Batch Jobs (9 commands)
// ============================================================================

#[command]
pub async fn get_batch_jobs(state: State<'_, AppState>) -> Result<Vec<BatchJob>, String> {
    let jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;
    Ok(jobs.clone())
}

#[command]
pub async fn get_batch_job(
    job_id: String,
    state: State<'_, AppState>
) -> Result<Option<BatchJob>, String> {
    let jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;
    Ok(jobs.iter().find(|j| j.id == job_id).cloned())
}

#[command]
pub async fn create_batch_job(
    name: String,
    model_id: String,
    config: BatchJobConfig,
    state: State<'_, AppState>
) -> Result<String, String> {
    let job_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let job = BatchJob {
        id: job_id.clone(),
        name,
        status: "pending".to_string(),
        model_id,
        created_at: now,
        started_at: None,
        completed_at: None,
        progress: 0.0,
        total_tasks: config.inputs.len() as u32,
        completed_tasks: 0,
        failed_tasks: 0,
        schedule: None,
        next_run: None,
        config,
        results: None,
    };

    let mut jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;
    jobs.push(job);

    Ok(job_id)
}

#[command]
pub async fn start_batch_job(
    job_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let mut jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;

    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        job.status = "running".to_string();
        job.started_at = Some(chrono::Utc::now().to_rfc3339());

        // TODO: Implement actual batch job execution
        // For now, this is a stub that would need backend integration
    }

    Ok(())
}

#[command]
pub async fn cancel_batch_job(
    job_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let mut jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;

    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        job.status = "cancelled".to_string();
        job.completed_at = Some(chrono::Utc::now().to_rfc3339());
    }

    Ok(())
}

#[command]
pub async fn delete_batch_job(
    job_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let mut jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;
    jobs.retain(|j| j.id != job_id);
    Ok(())
}

#[command]
pub async fn update_batch_job_progress(
    job_id: String,
    progress: f64,
    completed_tasks: u32,
    failed_tasks: u32,
    state: State<'_, AppState>
) -> Result<(), String> {
    let mut jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;

    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        job.progress = progress;
        job.completed_tasks = completed_tasks;
        job.failed_tasks = failed_tasks;

        if progress >= 100.0 {
            job.status = "completed".to_string();
            job.completed_at = Some(chrono::Utc::now().to_rfc3339());
        }
    }

    Ok(())
}

#[command]
pub async fn schedule_batch_job(
    job_id: String,
    schedule: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let mut jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;

    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        job.schedule = Some(schedule);
        // TODO: Implement cron-style scheduling
    }

    Ok(())
}

#[command]
pub async fn clear_batch_jobs(state: State<'_, AppState>) -> Result<(), String> {
    let mut jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;
    jobs.retain(|j| j.status == "running");
    Ok(())
}

// ============================================================================
// Security / API Keys (8 commands)
// ============================================================================

#[command]
pub async fn list_api_keys(state: State<'_, AppState>) -> Result<Vec<ApiKey>, String> {
    state.security_manager.list_keys()
        .map_err(|e| e.to_string())
}

#[command]
pub async fn create_api_key(
    request: CreateApiKeyRequest,
    state: State<'_, AppState>
) -> Result<CreateApiKeyResponse, String> {
    state.security_manager.create_key(request)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn revoke_api_key(
    key_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    state.security_manager.revoke_key(&key_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn validate_api_key(
    key: String,
    state: State<'_, AppState>
) -> Result<bool, String> {
    Ok(state.security_manager.validate_key(&key))
}

#[command]
pub async fn get_security_events(
    limit: Option<usize>,
    state: State<'_, AppState>
) -> Result<Vec<SecurityEvent>, String> {
    state.security_manager.get_events(limit)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_security_metrics(state: State<'_, AppState>) -> Result<SecurityMetrics, String> {
    state.security_manager.get_metrics()
        .map_err(|e| e.to_string())
}

#[command]
pub async fn clear_security_events(state: State<'_, AppState>) -> Result<(), String> {
    state.security_manager.clear_events()
        .map_err(|e| e.to_string())
}

#[command]
pub async fn export_security_log(
    path: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let events = state.security_manager.get_events(None)
        .map_err(|e| e.to_string())?;

    let json = serde_json::to_string_pretty(&events)
        .map_err(|e| format!("Failed to serialize events: {}", e))?;

    tokio::fs::write(&path, json).await
        .map_err(|e| format!("Failed to write log file {}: {}", path, e))
}

// ============================================================================
// Model Repository (10 commands)
// ============================================================================

#[command]
pub async fn search_models(
    query: ModelSearchQuery,
    state: State<'_, AppState>
) -> Result<ModelSearchResponse, String> {
    state.model_repository.search_models(query).await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_model_details(
    model_id: String,
    state: State<'_, AppState>
) -> Result<ExternalModelInfo, String> {
    state.model_repository.get_model_info(&model_id).await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn download_model(
    app: AppHandle,
    model_id: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let download_id = Uuid::new_v4().to_string();

    // Start download in background with progress updates
    let app_clone = app.clone();
    let download_id_clone = download_id.clone();
    let model_id_clone = model_id.clone();

    tokio::spawn(async move {
        let mut progress_rx = state.download_manager.download_model(model_id_clone).await
            .expect("Failed to start download");

        while let Some(progress) = progress_rx.recv().await {
            let _ = app_clone.emit("download_progress", serde_json::json!({
                "download_id": download_id_clone,
                "progress": progress
            }));
        }

        let _ = app_clone.emit("download_complete", &download_id_clone);
    });

    Ok(download_id)
}

#[command]
pub async fn cancel_download(
    download_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    state.download_manager.cancel_download(&download_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_download_progress(
    download_id: String,
    state: State<'_, AppState>
) -> Result<Option<DownloadProgress>, String> {
    Ok(state.download_manager.get_progress(&download_id))
}

#[command]
pub async fn list_downloads(state: State<'_, AppState>) -> Result<Vec<DownloadProgress>, String> {
    Ok(state.download_manager.list_downloads())
}

#[command]
pub async fn clear_completed_downloads(state: State<'_, AppState>) -> Result<(), String> {
    state.download_manager.clear_completed()
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_popular_models(
    limit: Option<usize>,
    state: State<'_, AppState>
) -> Result<Vec<ExternalModelInfo>, String> {
    state.model_repository.get_popular_models(limit.unwrap_or(10)).await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_recommended_models(state: State<'_, AppState>) -> Result<Vec<ExternalModelInfo>, String> {
    state.model_repository.get_recommended_models().await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn check_model_updates(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.model_repository.check_for_updates().await
        .map_err(|e| e.to_string())
}

// ============================================================================
// Utility Command (1 command)
// ============================================================================

#[command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Inferno.", name)
}