#![allow(clippy::assign_op_pattern)]

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

use sysinfo::{CpuExt, SystemExt};
use tauri::{command, AppHandle, Emitter, State};
use uuid::Uuid;

use super::types::{
    ActiveProcessInfo,
    AppSettings,
    BatchJob,
    BatchJobConfig,
    ChipInfo, // Phase 2: GPU and chip detection types
    GpuInfo,
    InfernoMetrics,
    MetricsSnapshot,
    Notification,
    SystemInfo,
};

use super::{
    ActivityLog, ActivityStats, ActivityType, ApiKey, AppState, CreateApiKeyRequest,
    CreateApiKeyResponse, DownloadProgress, ExternalModelInfo, InferenceParams, ModelInfo,
    ModelSearchQuery, ModelSearchResponse, SecurityEvent, SecurityMetrics, SecurityScanResult,
};

// ============================================================================
// Core Model Operations (5 commands)
// ============================================================================

#[command]
pub async fn get_models(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    state
        .backend_manager
        .discover_models()
        .await
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
    state: State<'_, AppState>,
) -> Result<String, String> {
    match state
        .backend_manager
        .load_model(model_name.clone(), backend_type.clone())
        .await
    {
        Ok(backend_id) => {
            // Emit model loaded event
            if let Ok(event_mgr) = state.event_manager.lock() {
                if let Some(ref manager) = *event_mgr {
                    let _ = manager.emit_model_loaded(model_name, backend_id.clone());
                }
            }
            Ok(backend_id)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[command]
pub async fn unload_model(backend_id: String, state: State<'_, AppState>) -> Result<(), String> {
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
        Err(e) => Err(e.to_string()),
    }
}

#[command]
pub async fn get_model_info(
    backend_id: String,
    state: State<'_, AppState>,
) -> Result<Option<ModelInfo>, String> {
    Ok(state.backend_manager.get_model_info(&backend_id).await)
}

// ============================================================================
// Inference Operations (2 commands)
// ============================================================================

#[command]
pub async fn infer(
    backend_id: String,
    prompt: String,
    params: InferenceParams,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let inference_id = Uuid::new_v4().to_string();

    // Emit inference started event
    if let Ok(event_mgr) = state.event_manager.lock() {
        if let Some(ref manager) = *event_mgr {
            let _ = manager.emit_inference_started(inference_id.clone(), backend_id.clone());
        }
    }

    match state
        .backend_manager
        .infer(backend_id, prompt, params)
        .await
    {
        Ok(response) => {
            // Emit inference completed event
            if let Ok(event_mgr) = state.event_manager.lock() {
                if let Some(ref manager) = *event_mgr {
                    let _ = manager.emit_inference_completed(inference_id, response.clone(), 100);
                    // Mock latency
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
    state: State<'_, AppState>,
) -> Result<String, String> {
    use tokio::time::{sleep, Duration};

    // Generate a unique inference ID for this session
    let inference_id = Uuid::new_v4().to_string();

    // Emit the start event
    let _ = app.emit("inference_start", &inference_id);

    let backend_manager = state.backend_manager.clone();
    let app_clone = app.clone();
    let inference_id_clone = inference_id.clone();
    let backend_id_clone = backend_id.clone();
    let streaming_guard = backend_manager.begin_streaming_session();
    let prompt_for_stream = prompt;
    let params_for_stream = params;

    tokio::spawn(async move {
        let _session_guard = streaming_guard;

        // Get the stream from backend manager
        match backend_manager
            .infer_stream(&backend_id_clone, &prompt_for_stream, &params_for_stream)
            .await
        {
            Ok(mut stream) => {
                use futures::StreamExt;

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(token) => {
                            let _ = app_clone.emit(
                                "inference_token",
                                serde_json::json!({
                                    "inference_id": inference_id_clone,
                                    "token": token
                                }),
                            );

                            // Small delay to prevent overwhelming the frontend
                            sleep(Duration::from_millis(10)).await;
                        }
                        Err(e) => {
                            let _ = app_clone.emit(
                                "inference_error",
                                serde_json::json!({
                                    "inference_id": inference_id_clone,
                                    "error": e.to_string()
                                }),
                            );
                            break;
                        }
                    }
                }

                // Emit completion event
                let _ = app_clone.emit("inference_complete", &inference_id_clone);
            }
            Err(e) => {
                let _ = app_clone.emit(
                    "inference_error",
                    serde_json::json!({
                        "inference_id": inference_id_clone,
                        "error": e.to_string()
                    }),
                );
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

    // Get GPU information (Phase 2)
    let gpu_info = if cfg!(target_os = "macos") {
        #[cfg(feature = "desktop")]
        {
            use crate::interfaces::desktop::macos::detect_metal_gpu;
            match detect_metal_gpu().await {
                Ok(metal_info) => Some(GpuInfo {
                    available: metal_info.available,
                    device_name: metal_info.device_name.clone(),
                    memory_gb: metal_info.memory_gb,
                    supports_metal_3: metal_info.supports_metal_3,
                    vendor: if metal_info.device_name.contains("Apple") {
                        "Apple".to_string()
                    } else if metal_info.device_name.contains("AMD") {
                        "AMD".to_string()
                    } else if metal_info.device_name.contains("NVIDIA") {
                        "NVIDIA".to_string()
                    } else {
                        "Unknown".to_string()
                    },
                }),
                Err(_) => None,
            }
        }
        #[cfg(not(feature = "desktop"))]
        {
            None
        }
    } else {
        None
    };

    // Get chip information (Phase 2)
    let chip_info = if cfg!(target_os = "macos") {
        #[cfg(feature = "desktop")]
        {
            use crate::interfaces::desktop::macos::detect_apple_silicon;
            match detect_apple_silicon().await {
                Ok(chip) => Some(ChipInfo {
                    is_apple_silicon: chip.is_apple_silicon,
                    chip_name: chip.chip_name.clone(),
                    performance_cores: chip.performance_cores,
                    efficiency_cores: chip.efficiency_cores,
                    neural_engine: chip.neural_engine,
                    total_cores: chip.performance_cores + chip.efficiency_cores,
                }),
                Err(_) => None,
            }
        }
        #[cfg(not(feature = "desktop"))]
        {
            None
        }
    } else {
        None
    };

    Ok(SystemInfo {
        cpu_name: system.global_cpu_info().brand().to_string(),
        cpu_usage,
        cpu_cores: system.cpus().len(),
        total_memory,
        used_memory,
        available_memory,
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        gpu_info,
        chip_info,
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
        active_streaming_sessions: global_metrics.active_streaming_sessions,
    })
}

#[command]
pub async fn get_inferno_metrics(state: State<'_, AppState>) -> Result<InfernoMetrics, String> {
    let mut system = state.system.lock().map_err(|e| e.to_string())?;
    system.refresh_all();

    let cpu_usage = system.global_cpu_info().cpu_usage();
    let memory_usage = system.used_memory();

    let global_metrics = state.backend_manager.get_metrics();

    // Refresh GPU info and calculate aggregate utilization
    if let Err(err) = state.gpu_manager.refresh_gpu_info().await {
        tracing::debug!(?err, "Failed to refresh GPU info for metrics");
    }
    let available_gpus = state.gpu_manager.get_available_gpus().await;
    let gpu_usage = if available_gpus.is_empty() {
        None
    } else {
        let total_util: f32 = available_gpus
            .iter()
            .map(|gpu| gpu.utilization_percent.max(0.0).min(100.0))
            .sum();
        let average = total_util / available_gpus.len() as f32;
        Some(average)
    };

    Ok(InfernoMetrics {
        cpu_usage,
        memory_usage,
        gpu_usage,
        active_models: global_metrics.models_loaded,
        active_inferences: global_metrics.active_inferences,
        active_streaming_sessions: global_metrics.active_streaming_sessions,
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
    let global_metrics = state.backend_manager.get_metrics();

    Ok(ActiveProcessInfo {
        active_models: loaded_models,
        active_inferences: global_metrics.active_inferences,
        batch_jobs: running_jobs,
        streaming_sessions: global_metrics.active_streaming_sessions,
    })
}

// ============================================================================
// File Operations (2 commands)
// ============================================================================

#[command]
pub async fn read_file(path: String) -> Result<String, String> {
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read file {}: {}", path, e))
}

#[command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(&path).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    tokio::fs::write(&path, content)
        .await
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
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Update in-memory settings
    let mut current_settings = state.settings.lock().map_err(|e| e.to_string())?;
    *current_settings = settings.clone();
    drop(current_settings);

    // Save to disk
    let config_path = std::path::PathBuf::from(".").join("inferno-settings.json");
    let contents = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    tokio::fs::write(&config_path, contents)
        .await
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
    state: State<'_, AppState>,
) -> Result<Vec<ActivityLog>, String> {
    let limit = limit.unwrap_or(100);

    let logs = match filter {
        Some(filter) => {
            let activity_type = match filter.to_lowercase().as_str() {
                "inference" => Some(ActivityType::Inference),
                "model_load" => Some(ActivityType::ModelLoad),
                "model_unload" => Some(ActivityType::ModelUnload),
                "model_validation" => Some(ActivityType::ModelValidation),
                "model_upload" => Some(ActivityType::ModelUpload),
                "configuration" => Some(ActivityType::Configuration),
                "system" => Some(ActivityType::System),
                "error" => Some(ActivityType::Error),
                _ => None,
            };

            if let Some(activity_type) = activity_type {
                state
                    .activity_logger
                    .get_activities_by_type(activity_type, limit)
            } else {
                state.activity_logger.get_recent_activities(limit)
            }
        }
        None => state.activity_logger.get_recent_activities(limit),
    };

    Ok(logs)
}

#[command]
pub async fn get_activity_stats(state: State<'_, AppState>) -> Result<ActivityStats, String> {
    Ok(state.activity_logger.get_stats())
}

#[command]
pub async fn clear_activity_logs(state: State<'_, AppState>) -> Result<(), String> {
    state.activity_logger.clear();
    Ok(())
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
pub async fn get_unread_notifications(
    state: State<'_, AppState>,
) -> Result<Vec<Notification>, String> {
    let notifications = state.notifications.lock().map_err(|e| e.to_string())?;
    Ok(notifications.iter().filter(|n| !n.read).cloned().collect())
}

#[command]
pub async fn mark_notification_read(
    notification_id: String,
    state: State<'_, AppState>,
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
    state: State<'_, AppState>,
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
    state: State<'_, AppState>,
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
    state: State<'_, AppState>,
) -> Result<Option<BatchJob>, String> {
    let jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;
    Ok(jobs.iter().find(|j| j.id == job_id).cloned())
}

#[command]
pub async fn create_batch_job(
    name: String,
    model_id: String,
    config: BatchJobConfig,
    state: State<'_, AppState>,
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
pub async fn start_batch_job(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    // Get job details
    let (model_id, inputs) = {
        let mut jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;

        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            if job.status != "pending" && job.status != "scheduled" {
                return Err(format!(
                    "Job {} is not pending (status: {})",
                    job_id, job.status
                ));
            }
            job.status = "running".to_string();
            job.started_at = Some(chrono::Utc::now().to_rfc3339());
            (job.model_id.clone(), job.config.inputs.clone())
        } else {
            return Err(format!("Job {} not found", job_id));
        }
    };

    // Clone Arc references for the spawned task
    let backend_manager = state.backend_manager.clone();
    let batch_jobs = state.batch_jobs.clone();
    let activity_logger = state.activity_logger.clone();
    let job_id_clone = job_id.clone();

    // Spawn async task to run the batch job
    tokio::spawn(async move {
        let total_inputs = inputs.len();
        let mut completed = 0;
        let mut failed = 0;
        let mut outputs = Vec::new();
        let mut errors = Vec::new();
        let start_time = std::time::Instant::now();

        // Check if model is loaded
        let loaded_models = backend_manager.get_loaded_models();
        if !loaded_models.contains(&model_id) {
            // Try to load the model (default to GGUF backend)
            if let Err(e) = backend_manager
                .load_model(model_id.clone(), "gguf".to_string())
                .await
            {
                let mut jobs = batch_jobs.lock().unwrap();
                if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id_clone) {
                    job.status = "failed".to_string();
                    job.completed_at = Some(chrono::Utc::now().to_rfc3339());
                }
                tracing::error!("Failed to load model for batch job: {}", e);
                return;
            }
        }

        // Process each input
        for (i, input) in inputs.iter().enumerate() {
            let params = super::InferenceParams::default();

            match backend_manager
                .infer(model_id.clone(), input.clone(), params)
                .await
            {
                Ok(output) => {
                    outputs.push(output);
                    completed += 1;
                }
                Err(e) => {
                    errors.push(format!("Input {}: {}", i, e));
                    failed += 1;
                }
            }

            // Update progress
            let progress = ((i + 1) as f64 / total_inputs as f64) * 100.0;
            let mut jobs = batch_jobs.lock().unwrap();
            if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id_clone) {
                job.progress = progress;
                job.completed_tasks = completed;
                job.failed_tasks = failed;
            }
        }

        // Finalize job
        let elapsed = start_time.elapsed().as_secs_f64();
        let avg_time = if total_inputs > 0 {
            elapsed / total_inputs as f64
        } else {
            0.0
        };
        let throughput = if elapsed > 0.0 {
            total_inputs as f64 / elapsed
        } else {
            0.0
        };

        let mut jobs = batch_jobs.lock().unwrap();
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id_clone) {
            job.status = if failed == 0 {
                "completed"
            } else {
                "completed_with_errors"
            }
            .to_string();
            job.completed_at = Some(chrono::Utc::now().to_rfc3339());
            job.progress = 100.0;
            job.results = Some(super::types::BatchJobResults {
                outputs,
                errors,
                metrics: super::types::BatchJobMetrics {
                    total_time: elapsed,
                    avg_time_per_task: avg_time,
                    throughput,
                },
            });
        }

        // Log activity
        activity_logger.log_simple(
            super::ActivityType::System,
            "Batch Job Completed".to_string(),
            format!(
                "Batch job {} completed: {} succeeded, {} failed",
                job_id_clone, completed, failed
            ),
            if failed == 0 {
                super::ActivityStatus::Success
            } else {
                super::ActivityStatus::Warning
            },
        );

        tracing::info!(
            "Batch job {} completed: {}/{} tasks succeeded in {:.2}s",
            job_id_clone,
            completed,
            total_inputs,
            elapsed
        );
    });

    Ok(())
}

#[command]
pub async fn cancel_batch_job(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;

    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        job.status = "cancelled".to_string();
        job.completed_at = Some(chrono::Utc::now().to_rfc3339());
    }

    Ok(())
}

#[command]
pub async fn delete_batch_job(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
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
    state: State<'_, AppState>,
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
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Validate and parse the cron expression to calculate next run
    let next_run = parse_cron_schedule(&schedule)?;

    let mut jobs = state.batch_jobs.lock().map_err(|e| e.to_string())?;

    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        job.schedule = Some(schedule.clone());
        job.next_run = Some(next_run.to_rfc3339());
        job.status = "scheduled".to_string();

        tracing::info!(
            "Batch job {} scheduled with '{}', next run: {}",
            job_id,
            schedule,
            next_run
        );
    } else {
        return Err(format!("Job {} not found", job_id));
    }

    Ok(())
}

/// Parse a cron schedule and calculate the next run time
fn parse_cron_schedule(schedule: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
    use chrono::{Datelike, Timelike, Utc};

    let now = Utc::now();

    // Handle special keywords
    let next = match schedule {
        "@hourly" => {
            // Next hour at :00
            let next_hour = now + chrono::Duration::hours(1);
            next_hour
                .with_minute(0)
                .and_then(|t| t.with_second(0))
                .unwrap_or(next_hour)
        }
        "@daily" | "@midnight" => {
            // Tomorrow at 00:00
            let tomorrow = now + chrono::Duration::days(1);
            tomorrow
                .with_hour(0)
                .and_then(|t| t.with_minute(0))
                .and_then(|t| t.with_second(0))
                .unwrap_or(tomorrow)
        }
        "@weekly" => {
            // Next Sunday at 00:00
            let days_until_sunday = (7 - now.weekday().num_days_from_sunday()) % 7;
            let days_until_sunday = if days_until_sunday == 0 {
                7
            } else {
                days_until_sunday
            };
            let next_sunday = now + chrono::Duration::days(days_until_sunday as i64);
            next_sunday
                .with_hour(0)
                .and_then(|t| t.with_minute(0))
                .and_then(|t| t.with_second(0))
                .unwrap_or(next_sunday)
        }
        "@monthly" => {
            // First day of next month at 00:00
            let next_month = if now.month() == 12 {
                now.with_year(now.year() + 1)
                    .and_then(|t| t.with_month(1))
                    .unwrap_or(now)
            } else {
                now.with_month(now.month() + 1).unwrap_or(now)
            };
            next_month
                .with_day(1)
                .and_then(|t| t.with_hour(0))
                .and_then(|t| t.with_minute(0))
                .and_then(|t| t.with_second(0))
                .unwrap_or(next_month)
        }
        _ => {
            // Parse standard cron expression: "minute hour day month weekday"
            let parts: Vec<&str> = schedule.split_whitespace().collect();
            if parts.len() != 5 {
                return Err(format!(
                    "Invalid cron expression: '{}'. Expected 5 fields (minute hour day month weekday) or keyword (@hourly, @daily, @weekly, @monthly)",
                    schedule
                ));
            }

            // Simple parsing for common patterns
            // For full cron parsing, we'd use a cron crate, but this covers basic cases
            let minute: u32 = if parts[0] == "*" {
                0
            } else {
                parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid minute: {}", parts[0]))?
            };
            let hour: u32 = if parts[1] == "*" {
                (now.hour() + 1) % 24
            } else {
                parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid hour: {}", parts[1]))?
            };

            if minute > 59 || hour > 23 {
                return Err(format!("Invalid time: {}:{}", hour, minute));
            }

            // Calculate next occurrence
            let mut next = now
                .with_hour(hour)
                .and_then(|t| t.with_minute(minute))
                .and_then(|t| t.with_second(0))
                .unwrap_or(now);

            // If the time is in the past today, move to tomorrow
            if next <= now {
                next = next + chrono::Duration::days(1);
            }

            next
        }
    };

    Ok(next)
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
    state
        .security_manager
        .get_api_keys()
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn create_api_key(
    request: CreateApiKeyRequest,
    state: State<'_, AppState>,
) -> Result<CreateApiKeyResponse, String> {
    state
        .security_manager
        .generate_api_key(request)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn revoke_api_key(key_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .security_manager
        .revoke_api_key(key_id)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn validate_api_key(key: String, state: State<'_, AppState>) -> Result<bool, String> {
    state
        .security_manager
        .validate_api_key(key)
        .await
        .map(|result| result.is_some())
}

#[command]
pub async fn get_security_events(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SecurityEvent>, String> {
    state
        .security_manager
        .get_security_events(limit)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_security_metrics(state: State<'_, AppState>) -> Result<SecurityMetrics, String> {
    state
        .security_manager
        .get_security_metrics()
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn run_security_scan(state: State<'_, AppState>) -> Result<SecurityScanResult, String> {
    state.security_manager.run_security_scan().await
}

#[command]
pub async fn clear_security_events(state: State<'_, AppState>) -> Result<(), String> {
    state
        .security_manager
        .clear_security_events()
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn export_security_log(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let events = state
        .security_manager
        .get_security_events(None)
        .await
        .map_err(|e| e.to_string())?;

    let json = serde_json::to_string_pretty(&events)
        .map_err(|e| format!("Failed to serialize events: {}", e))?;

    tokio::fs::write(&path, json)
        .await
        .map_err(|e| format!("Failed to write log file {}: {}", path, e))
}

// ============================================================================
// Model Repository (10 commands)
// ============================================================================

#[command]
pub async fn search_models(
    query: ModelSearchQuery,
    state: State<'_, AppState>,
) -> Result<ModelSearchResponse, String> {
    state
        .model_repository
        .search_models(query)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_model_details(
    model_id: String,
    state: State<'_, AppState>,
) -> Result<ExternalModelInfo, String> {
    state
        .model_repository
        .get_model_details(&model_id)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn download_model(
    app: AppHandle,
    model_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let model = state
        .model_repository
        .get_model_details(&model_id)
        .await
        .map_err(|e| e.to_string())?;

    let target_dir = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        settings.models_directory.clone()
    };

    let download_id = state
        .download_manager
        .start_download(&model, &target_dir)
        .await
        .map_err(|e| e.to_string())?;

    let _ = app.emit(
        "download_started",
        serde_json::json!({
            "download_id": download_id,
            "model_id": model_id,
        }),
    );

    Ok(download_id)
}

#[command]
pub async fn cancel_download(
    download_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .download_manager
        .cancel_download(&download_id)
        .then_some(())
        .ok_or_else(|| "Download not found".to_string())
}

#[command]
pub async fn get_download_progress(
    download_id: String,
    state: State<'_, AppState>,
) -> Result<Option<DownloadProgress>, String> {
    Ok(state.download_manager.get_download_progress(&download_id))
}

#[command]
pub async fn list_downloads(state: State<'_, AppState>) -> Result<Vec<DownloadProgress>, String> {
    Ok(state.download_manager.get_all_downloads())
}

#[command]
pub async fn clear_completed_downloads(state: State<'_, AppState>) -> Result<(), String> {
    state.download_manager.clear_completed_downloads();
    Ok(())
}

#[command]
pub async fn get_popular_models(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<ExternalModelInfo>, String> {
    state
        .model_repository
        .get_trending_models()
        .await
        .map(|mut models| {
            let limit = limit.unwrap_or(10);
            models.truncate(limit);
            models
        })
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_recommended_models(
    state: State<'_, AppState>,
) -> Result<Vec<ExternalModelInfo>, String> {
    state
        .model_repository
        .get_featured_models()
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn check_model_updates(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let _ = state;
    Ok(Vec::new())
}

// ============================================================================
// Utility Command (1 command)
// ============================================================================

#[command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Inferno.", name)
}
