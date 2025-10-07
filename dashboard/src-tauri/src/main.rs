// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    menu::{AboutMetadata, Menu, MenuBuilder, MenuEvent, MenuItem, SubmenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, State, Emitter,
};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use sysinfo::System;
use uuid::Uuid;
use futures_util::StreamExt;
use tauri_plugin_notification::NotificationExt;

mod backend_manager;
mod activity_logger;
mod security;
mod events;
mod database;
mod model_repository;

use backend_manager::{BackendManager, ModelInfo, InferenceParams};
use activity_logger::{ActivityLogger, ActivityLog, ActivityStats, ActivityStatus, ActivityType};
use security::{SecurityManager, ApiKey, SecurityEvent, SecurityMetrics, CreateApiKeyRequest, CreateApiKeyResponse};
use events::{EventManager, InfernoEvent};
use database::{DatabaseManager, DbBatchJob, DbNotification};
use model_repository::{ModelRepositoryService, ModelDownloadManager, ExternalModelInfo, ModelSearchQuery, ModelSearchResponse, DownloadProgress};

const SETTINGS_FILE_NAME: &str = "inferno-settings.json";

const MENU_ID_ABOUT: &str = "menu.about";
const MENU_ID_PREFERENCES: &str = "menu.preferences";
const MENU_ID_SHOW_WINDOW: &str = "menu.show_window";
const MENU_ID_HIDE_WINDOW: &str = "menu.hide_window";
const MENU_ID_HIDE_OTHERS: &str = "menu.hide_others";
const MENU_ID_CHECK_UPDATES: &str = "menu.check_updates";
const MENU_ID_REPORT_ISSUE: &str = "menu.report_issue";
const MENU_ID_DOCUMENTATION: &str = "menu.documentation";
const MENU_ID_KEYBOARD_SHORTCUTS: &str = "menu.shortcuts";
const MENU_ID_NEW_INFERENCE: &str = "menu.new_inference";
const MENU_ID_OPEN_MODEL: &str = "menu.open_model";
const MENU_ID_IMPORT_MODEL: &str = "menu.import_model";
const MENU_ID_EXPORT_RESULTS: &str = "menu.export_results";
const MENU_ID_MODEL_INFO: &str = "menu.model_info";
const MENU_ID_VALIDATE_MODELS: &str = "menu.validate_models";
const MENU_ID_QUICK_INFERENCE: &str = "menu.quick_inference";
const MENU_ID_BATCH_INFERENCE: &str = "menu.batch_inference";
const MENU_ID_STOP_INFERENCE: &str = "menu.stop_inference";
const MENU_ID_VIEW_DASHBOARD: &str = "menu.view_dashboard";
const MENU_ID_VIEW_MODELS: &str = "menu.view_models";
const MENU_ID_VIEW_INFERENCE: &str = "menu.view_inference";
const MENU_ID_VIEW_METRICS: &str = "menu.view_metrics";
const MENU_ID_QUIT: &str = "menu.quit";

const TRAY_ID_DASHBOARD: &str = "tray.dashboard";
const TRAY_ID_MODELS: &str = "tray.models";
const TRAY_ID_INFERENCE: &str = "tray.quick_inference";
const TRAY_ID_SHOW: &str = "tray.show";
const TRAY_ID_HIDE: &str = "tray.hide";
const TRAY_ID_QUIT: &str = "tray.quit";

fn inferno_config_dir() -> PathBuf {
    dirs::config_dir()
        .map(|dir| dir.join("inferno"))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn inferno_data_dir() -> PathBuf {
    dirs::data_dir()
        .map(|dir| dir.join("inferno"))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn default_models_directory() -> PathBuf {
    inferno_data_dir().join("models")
}

fn default_cache_directory() -> PathBuf {
    inferno_data_dir().join("cache")
}

fn get_config_path() -> PathBuf {
    inferno_config_dir().join(SETTINGS_FILE_NAME)
}

fn legacy_config_path() -> PathBuf {
    PathBuf::from(SETTINGS_FILE_NAME)
}

fn build_app_menu<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<Menu<R>> {
    let about_metadata = AboutMetadata {
        name: Some("Inferno AI Desktop".to_string()),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        ..Default::default()
    };

    let preferences = MenuItem::with_id(
        app,
        MENU_ID_PREFERENCES,
        "Preferences…",
        true,
        Some("Cmd+,"),
    )?;

    let new_inference = MenuItem::with_id(
        app,
        MENU_ID_NEW_INFERENCE,
        "New Inference",
        true,
        Some("Cmd+N"),
    )?;
    let open_model = MenuItem::with_id(
        app,
        MENU_ID_OPEN_MODEL,
        "Open Model…",
        true,
        Some("Cmd+O"),
    )?;
    let import_model = MenuItem::with_id(
        app,
        MENU_ID_IMPORT_MODEL,
        "Import Model…",
        true,
        Some("Cmd+Shift+I"),
    )?;
    let export_results = MenuItem::with_id(
        app,
        MENU_ID_EXPORT_RESULTS,
        "Export Results…",
        true,
        Some("Cmd+E"),
    )?;
    let model_info = MenuItem::with_id(
        app,
        MENU_ID_MODEL_INFO,
        "Model Information",
        true,
        Some("Cmd+Shift+I"),
    )?;
    let validate_models = MenuItem::with_id(
        app,
        MENU_ID_VALIDATE_MODELS,
        "Validate Models",
        true,
        None::<&str>,
    )?;
    let quick_inference = MenuItem::with_id(
        app,
        MENU_ID_QUICK_INFERENCE,
        "Quick Inference",
        true,
        Some("Cmd+R"),
    )?;
    let batch_inference = MenuItem::with_id(
        app,
        MENU_ID_BATCH_INFERENCE,
        "Batch Inference",
        true,
        Some("Cmd+Shift+R"),
    )?;
    let stop_inference = MenuItem::with_id(
        app,
        MENU_ID_STOP_INFERENCE,
        "Stop All Inference",
        true,
        Some("Cmd+."),
    )?;
    let view_dashboard = MenuItem::with_id(
        app,
        MENU_ID_VIEW_DASHBOARD,
        "Dashboard",
        true,
        Some("Cmd+1"),
    )?;
    let view_models = MenuItem::with_id(
        app,
        MENU_ID_VIEW_MODELS,
        "Models",
        true,
        Some("Cmd+2"),
    )?;
    let view_inference = MenuItem::with_id(
        app,
        MENU_ID_VIEW_INFERENCE,
        "Inference",
        true,
        Some("Cmd+3"),
    )?;
    let view_metrics = MenuItem::with_id(
        app,
        MENU_ID_VIEW_METRICS,
        "Metrics",
        true,
        Some("Cmd+4"),
    )?;
    let documentation = MenuItem::with_id(
        app,
        MENU_ID_DOCUMENTATION,
        "Documentation",
        true,
        None::<&str>,
    )?;
    let shortcuts = MenuItem::with_id(
        app,
        MENU_ID_KEYBOARD_SHORTCUTS,
        "Keyboard Shortcuts",
        true,
        None::<&str>,
    )?;
    let check_updates = MenuItem::with_id(
        app,
        MENU_ID_CHECK_UPDATES,
        "Check for Updates",
        true,
        None::<&str>,
    )?;
    let report_issue = MenuItem::with_id(
        app,
        MENU_ID_REPORT_ISSUE,
        "Report Issue…",
        true,
        None::<&str>,
    )?;
    let show_window = MenuItem::with_id(
        app,
        MENU_ID_SHOW_WINDOW,
        "Show Window",
        true,
        Some("Cmd+Shift+H"),
    )?;
    let hide_window = MenuItem::with_id(
        app,
        MENU_ID_HIDE_WINDOW,
        "Hide Window",
        true,
        None::<&str>,
    )?;

    let inferno_submenu = SubmenuBuilder::with_id(app, "menu.inferno", "Inferno")
        .about(Some(about_metadata))
        .separator()
        .item(&preferences)
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit_with_text("Quit Inferno")
        .build()?;

    let file_submenu = SubmenuBuilder::with_id(app, "menu.file", "File")
        .item(&new_inference)
        .item(&open_model)
        .separator()
        .item(&import_model)
        .item(&export_results)
        .separator()
        .close_window()
        .build()?;

    let edit_submenu = SubmenuBuilder::with_id(app, "menu.edit", "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let models_submenu = SubmenuBuilder::with_id(app, "menu.models", "Models")
        .item(&model_info)
        .item(&validate_models)
        .build()?;

    let inference_submenu = SubmenuBuilder::with_id(app, "menu.inference", "Inference")
        .item(&quick_inference)
        .item(&batch_inference)
        .separator()
        .item(&stop_inference)
        .build()?;

    let view_submenu = SubmenuBuilder::with_id(app, "menu.view", "View")
        .item(&view_dashboard)
        .item(&view_models)
        .item(&view_inference)
        .item(&view_metrics)
        .separator()
        .fullscreen()
        .build()?;

    let window_submenu = SubmenuBuilder::with_id(app, "menu.window", "Window")
        .item(&show_window)
        .item(&hide_window)
        .separator()
        .minimize()
        .close_window()
        .build()?;

    let help_submenu = SubmenuBuilder::with_id(app, "menu.help", "Help")
        .item(&documentation)
        .item(&shortcuts)
        .separator()
        .item(&report_issue)
        .item(&check_updates)
        .build()?;

    MenuBuilder::new(app)
        .items(&[
            &inferno_submenu,
            &file_submenu,
            &edit_submenu,
            &models_submenu,
            &inference_submenu,
            &view_submenu,
            &window_submenu,
            &help_submenu,
        ])
        .build()
}

fn build_tray_menu<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<Menu<R>> {
    let dashboard = MenuItem::with_id(app, TRAY_ID_DASHBOARD, "Open Dashboard", true, None::<&str>)?;
    let models = MenuItem::with_id(app, TRAY_ID_MODELS, "Manage Models", true, None::<&str>)?;
    let inference = MenuItem::with_id(app, TRAY_ID_INFERENCE, "Quick Inference", true, None::<&str>)?;
    let show = MenuItem::with_id(app, TRAY_ID_SHOW, "Show Window", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, TRAY_ID_HIDE, "Hide Window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, TRAY_ID_QUIT, "Quit Inferno", true, None::<&str>)?;

    MenuBuilder::new(app)
        .item(&dashboard)
        .separator()
        .item(&models)
        .item(&inference)
        .separator()
        .item(&show)
        .item(&hide)
        .separator()
        .item(&quit)
        .build()
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn handle_menu_event(app: &tauri::AppHandle, menu_id: &str) {
    match menu_id {
        MENU_ID_PREFERENCES => {
            let _ = app.emit("menu://open-preferences", ());
            show_main_window(app);
        }
        MENU_ID_NEW_INFERENCE => {
            let _ = app.emit("menu://new-inference", ());
            show_main_window(app);
        }
        MENU_ID_OPEN_MODEL => {
            let _ = app.emit("menu://open-model", ());
            show_main_window(app);
        }
        MENU_ID_IMPORT_MODEL => {
            let _ = app.emit("menu://import-model", ());
            show_main_window(app);
        }
        MENU_ID_EXPORT_RESULTS => {
            let _ = app.emit("menu://export-results", ());
        }
        MENU_ID_MODEL_INFO => {
            let _ = app.emit("menu://model-info", ());
            show_main_window(app);
        }
        MENU_ID_VALIDATE_MODELS => {
            let _ = app.emit("menu://validate-models", ());
        }
        MENU_ID_QUICK_INFERENCE => {
            let _ = app.emit("menu://quick-inference", ());
            show_main_window(app);
        }
        MENU_ID_BATCH_INFERENCE => {
            let _ = app.emit("menu://batch-inference", ());
            show_main_window(app);
        }
        MENU_ID_STOP_INFERENCE => {
            let _ = app.emit("menu://stop-inference", ());
        }
        MENU_ID_VIEW_DASHBOARD => {
            let _ = app.emit("menu://navigate", serde_json::json!({ "target": "dashboard" }));
            show_main_window(app);
        }
        MENU_ID_VIEW_MODELS => {
            let _ = app.emit("menu://navigate", serde_json::json!({ "target": "models" }));
            show_main_window(app);
        }
        MENU_ID_VIEW_INFERENCE => {
            let _ = app.emit("menu://navigate", serde_json::json!({ "target": "inference" }));
            show_main_window(app);
        }
        MENU_ID_VIEW_METRICS => {
            let _ = app.emit("menu://navigate", serde_json::json!({ "target": "metrics" }));
            show_main_window(app);
        }
        MENU_ID_DOCUMENTATION => {
            let _ = app.emit("menu://open-docs", ());
        }
        MENU_ID_KEYBOARD_SHORTCUTS => {
            let _ = app.emit("menu://show-shortcuts", ());
        }
        MENU_ID_REPORT_ISSUE => {
            let _ = app.emit("menu://report-issue", ());
        }
        MENU_ID_CHECK_UPDATES => {
            let _ = app.emit("menu://check-updates", ());
        }
        MENU_ID_SHOW_WINDOW => show_main_window(app),
        MENU_ID_HIDE_WINDOW => hide_main_window(app),
        MENU_ID_QUIT => app.exit(0),
        MENU_ID_ABOUT => {
            let _ = app.emit("menu://about", ());
        }
        MENU_ID_HIDE_OTHERS => {
            let _ = app.emit("menu://hide-others", ());
        }
        _ => {}
    }
}

fn handle_tray_menu_event(app: &tauri::AppHandle, menu_id: &str) {
    match menu_id {
        TRAY_ID_DASHBOARD => {
            let _ = app.emit("tray://open-dashboard", ());
            show_main_window(app);
        }
        TRAY_ID_MODELS => {
            let _ = app.emit("tray://open-models", ());
            show_main_window(app);
        }
        TRAY_ID_INFERENCE => {
            let _ = app.emit("tray://quick-inference", ());
            show_main_window(app);
        }
        TRAY_ID_SHOW => show_main_window(app),
        TRAY_ID_HIDE => hide_main_window(app),
        TRAY_ID_QUIT => app.exit(0),
        _ => {}
    }
}

struct StreamingGuard {
    counter: Arc<AtomicU32>,
}

impl StreamingGuard {
    fn new(counter: Arc<AtomicU32>) -> Self {
        Self { counter }
    }
}

impl Drop for StreamingGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::SeqCst);
    }
}

fn current_gpu_usage() -> Option<f32> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        if let Ok(output) = Command::new("ioreg")
            .args(["-c", "IOAccelerator", "-r", "-d", "1"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(idx) = stdout.find("GPURendererUtilization") {
                    let rest = &stdout[idx..];
                    if let Some(eq_idx) = rest.find('=') {
                        let mut value = String::new();
                        for ch in rest[eq_idx + 1..].chars() {
                            if ch.is_ascii_digit() || ch == '.' {
                                value.push(ch);
                            } else if !value.is_empty() {
                                break;
                            }
                        }

                        if let Ok(raw) = value.parse::<f32>() {
                            return Some((raw * 100.0).min(100.0));
                        }
                    }
                }
            }
        }
    }

    None
}

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
    pub active_streaming_sessions: u32,
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

#[derive(Deserialize, Clone, Debug)]
pub struct NativeNotificationRequest {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub sound: Option<String>,
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

fn db_batch_job_to_ui(db_job: DbBatchJob) -> BatchJob {
    let config = serde_json::from_str::<BatchJobConfig>(&db_job.config).unwrap_or_default();
    let results = db_job
        .results
        .as_ref()
        .and_then(|raw| serde_json::from_str::<BatchJobResults>(raw).ok());

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
        let models_dir = default_models_directory();
        let cache_dir = default_cache_directory();
        Self {
            models_directory: models_dir.to_string_lossy().to_string(),
            auto_discover_models: true,
            default_temperature: 0.7,
            default_max_tokens: 512,
            default_top_p: 0.9,
            default_top_k: 40,
            max_memory_usage: 80,
            prefer_gpu: true,
            max_concurrent_inferences: 3,
            enable_cache: true,
            cache_directory: cache_dir.to_string_lossy().to_string(),
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
    pub security_manager: Arc<SecurityManager>,
    pub event_manager: Arc<Mutex<Option<EventManager>>>,
    pub database: Arc<DatabaseManager>,
    pub model_repository: Arc<ModelRepositoryService>,
    pub download_manager: Arc<ModelDownloadManager>,
    pub streaming_sessions: Arc<AtomicU32>,
}

// Configuration management functions
async fn load_settings() -> AppSettings {
    let config_path = get_config_path();

    if let Ok(contents) = tokio::fs::read_to_string(&config_path).await {
        if let Ok(mut settings) = serde_json::from_str::<AppSettings>(&contents) {
            match normalize_settings(&mut settings).await {
                Ok(changed) => {
                    if changed {
                        if let Err(e) = save_settings(&settings).await {
                            eprintln!("Failed to persist migrated settings: {}", e);
                        }
                    }
                }
                Err(e) => eprintln!("Failed to normalize settings: {}", e),
            }
            return settings;
        }
    }

    let legacy_path = legacy_config_path();
    if legacy_path != config_path {
        if let Ok(contents) = tokio::fs::read_to_string(&legacy_path).await {
            if let Ok(mut settings) = serde_json::from_str::<AppSettings>(&contents) {
                match normalize_settings(&mut settings).await {
                    Ok(_) => {
                        if let Err(e) = save_settings(&settings).await {
                            eprintln!("Failed to migrate legacy settings: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to normalize legacy settings: {}", e),
                }
                return settings;
            }
        }
    }

    let mut settings = AppSettings::default();
    match normalize_settings(&mut settings).await {
        Ok(_) => {
            if let Err(e) = save_settings(&settings).await {
                eprintln!("Failed to write default settings: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to prepare default settings: {}", e),
    }
    settings
}

async fn ensure_directory(path_str: &str, label: &str) -> Result<(), String> {
    if path_str.trim().is_empty() {
        return Err(format!("{} cannot be empty", label));
    }

    let path = PathBuf::from(path_str);
    tokio::fs::create_dir_all(&path)
        .await
        .map_err(|e| format!("Failed to create {} '{}': {}", label, path_str, e))
}

async fn ensure_settings_directories(settings: &AppSettings) -> Result<(), String> {
    ensure_directory(&settings.models_directory, "models directory").await?;
    ensure_directory(&settings.cache_directory, "cache directory").await?;
    Ok(())
}

async fn normalize_settings(settings: &mut AppSettings) -> Result<bool, String> {
    let mut changed = false;

    if settings.models_directory.trim().is_empty()
        || settings.models_directory == "test_models/test_models"
    {
        settings.models_directory = default_models_directory().to_string_lossy().to_string();
        changed = true;
    }

    if settings.cache_directory.trim().is_empty() || settings.cache_directory == ".cache" {
        settings.cache_directory = default_cache_directory().to_string_lossy().to_string();
        changed = true;
    }

    ensure_settings_directories(settings).await?;
    Ok(changed)
}

async fn save_settings(settings: &AppSettings) -> Result<(), String> {
    ensure_settings_directories(settings).await?;

    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let contents = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    tokio::fs::write(&config_path, contents)
        .await
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
    let start_time = Instant::now();

    // Emit inference started event
    if let Ok(event_mgr) = state.event_manager.lock() {
        if let Some(ref manager) = *event_mgr {
            let _ = manager.emit_inference_started(inference_id.clone(), backend_id.clone());
        }
    }

    match state.backend_manager.infer(backend_id, prompt, params).await {
        Ok(response) => {
            let latency_ms = start_time.elapsed().as_millis() as u64;
            // Emit inference completed event
            if let Ok(event_mgr) = state.event_manager.lock() {
                if let Some(ref manager) = *event_mgr {
                    let _ = manager.emit_inference_completed(inference_id, response.clone(), latency_ms);
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
    let inference_id = uuid::Uuid::new_v4().to_string();
    let start_time = Instant::now();

    state.activity_logger.log_inference(
        &backend_id,
        prompt.split_whitespace().count() as u32,
        0,
        0,
        ActivityStatus::InProgress,
    );

    let _ = app.emit("inference_start", &inference_id);

    if let Ok(event_mgr) = state.event_manager.lock() {
        if let Some(ref manager) = *event_mgr {
            let _ = manager.emit_inference_started(inference_id.clone(), backend_id.clone());
        }
    }

    let backend_manager = state.backend_manager.clone();
    let event_manager = state.event_manager.clone();
    let streaming_counter = state.streaming_sessions.clone();
    streaming_counter.fetch_add(1, Ordering::SeqCst);

    let prompt_tokens = prompt.split_whitespace().count() as u32;
    let guard = backend_manager.begin_inference();
    let max_tokens = params.max_tokens.unwrap_or(0);

    let stream = match backend_manager.infer_stream(&backend_id, &prompt, &params).await {
        Ok(stream) => stream,
        Err(e) => {
            streaming_counter.fetch_sub(1, Ordering::SeqCst);
            backend_manager.record_inference_result(
                &backend_id,
                prompt_tokens,
                0,
                start_time.elapsed().as_millis() as u64,
                ActivityStatus::Error,
            );

            let error_message = e.to_string();
            let _ = app.emit(
                "inference_error",
                serde_json::json!({ "inference_id": inference_id, "error": error_message.clone() }),
            );

            if let Ok(event_mgr) = event_manager.lock() {
                if let Some(ref manager) = *event_mgr {
                    let _ = manager.emit_inference_error(inference_id.clone(), error_message.clone());
                }
            }

            return Err(error_message);
        }
    };

    let app_clone = app.clone();
    let inference_id_clone = inference_id.clone();
    let backend_manager_clone = backend_manager.clone();
    let event_manager_clone = event_manager.clone();
    let backend_id_clone = backend_id.clone();
    let streaming_counter_clone = streaming_counter.clone();

    tokio::spawn(async move {
        let mut stream = stream;
        let _guard = guard;
        let _stream_guard = StreamingGuard::new(streaming_counter_clone);
        let start_time = Instant::now();
        let mut response = String::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(token) => {
                    response.push_str(&token);
                    let _ = app_clone.emit(
                        "inference_token",
                        serde_json::json!({
                            "inference_id": inference_id_clone.clone(),
                            "token": token
                        }),
                    );

                    let progress = if max_tokens > 0 {
                        (response.split_whitespace().count() as f32 / max_tokens as f32).min(1.0)
                    } else {
                        0.0
                    };

                    let _ = app_clone.emit(
                        "inference_progress",
                        serde_json::json!({
                            "inference_id": inference_id_clone.clone(),
                            "progress": progress,
                            "partial_response": response.clone()
                        }),
                    );

                    if let Ok(event_mgr) = event_manager_clone.lock() {
                        if let Some(ref manager) = *event_mgr {
                            let _ = manager.emit_inference_progress(
                                inference_id_clone.clone(),
                                progress,
                                Some(response.clone()),
                            );
                        }
                    }
                }
                Err(err) => {
                    let error_message = err.to_string();
                    backend_manager_clone.record_inference_result(
                        &backend_id_clone,
                        prompt_tokens,
                        response.split_whitespace().count() as u32,
                        start_time.elapsed().as_millis() as u64,
                        ActivityStatus::Error,
                    );

                    let _ = app_clone.emit(
                        "inference_error",
                        serde_json::json!({
                            "inference_id": inference_id_clone.clone(),
                            "error": error_message.clone()
                        }),
                    );

                    if let Ok(event_mgr) = event_manager_clone.lock() {
                        if let Some(ref manager) = *event_mgr {
                            let _ = manager.emit_inference_error(
                                inference_id_clone.clone(),
                                error_message,
                            );
                        }
                    }

                    return;
                }
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let completion_tokens = response.split_whitespace().count() as u32;

        backend_manager_clone.record_inference_result(
            &backend_id_clone,
            prompt_tokens,
            completion_tokens,
            duration_ms,
            ActivityStatus::Success,
        );

        let _ = app_clone.emit(
            "inference_progress",
            serde_json::json!({
                "inference_id": inference_id_clone.clone(),
                "progress": 1.0,
                "partial_response": response.clone()
            }),
        );

        let _ = app_clone.emit(
            "inference_complete",
            serde_json::json!({
                "inference_id": inference_id_clone.clone(),
                "response": response.clone()
            }),
        );

        if let Ok(event_mgr) = event_manager_clone.lock() {
            if let Some(ref manager) = *event_mgr {
                let _ = manager.emit_inference_completed(
                    inference_id_clone,
                    response,
                    duration_ms,
                );
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
use tauri_plugin_dialog::DialogExt;

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

    let models_dir_path: PathBuf = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        PathBuf::from(settings.models_directory.clone())
    };

    if let Err(e) = fs::create_dir_all(&models_dir_path).await {
        return Err(format!(
            "Failed to create models directory '{}': {}",
            models_dir_path.to_string_lossy(),
            e
        ));
    }

    // Determine target filename
    let target_filename = target_name.unwrap_or_else(|| {
        source.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "model".to_string())
    });

    let target_path = models_dir_path.join(&target_filename);

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

    let cpu_usage = system.global_cpu_info().cpu_usage();
    let memory_usage = system.used_memory();

    let global_metrics = state.backend_manager.get_metrics();
    let loaded_backends = state.backend_manager.get_loaded_models();
    let active_inferences = state.backend_manager.get_active_inference_count();
    let gpu_usage = current_gpu_usage();
    let streaming_sessions = state.streaming_sessions.load(Ordering::SeqCst);

    Ok(InfernoMetrics {
        cpu_usage,
        memory_usage,
        gpu_usage,
        active_models: loaded_backends.len() as u32,
        active_inferences,
        active_streaming_sessions: streaming_sessions,
        inference_count: global_metrics.inference_count,
        success_count: global_metrics.success_count,
        error_count: global_metrics.error_count,
        average_latency: global_metrics.average_latency,
    })
}

#[tauri::command]
async fn get_active_processes(state: State<'_, AppState>) -> Result<ActiveProcessInfo, String> {
    let loaded_models = state.backend_manager.get_loaded_models();

    let batch_jobs = state
        .database
        .get_batch_jobs()
        .await
        .map_err(|e| e.to_string())?;
    let active_batch_jobs = batch_jobs
        .iter()
        .filter(|j| j.status == "running" || j.status == "pending")
        .count() as u32;

    Ok(ActiveProcessInfo {
        active_models: loaded_models,
        active_inferences: state.backend_manager.get_active_inference_count(),
        batch_jobs: active_batch_jobs,
        streaming_sessions: state.streaming_sessions.load(Ordering::SeqCst),
    })
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
async fn set_settings(mut settings: AppSettings, state: State<'_, AppState>) -> Result<(), String> {
    normalize_settings(&mut settings).await?;
    save_settings(&settings).await?;

    let mut state_settings = state.settings.lock().map_err(|e| e.to_string())?;
    *state_settings = settings;

    Ok(())
}

// Notification management commands
#[tauri::command]
async fn get_notifications(state: State<'_, AppState>) -> Result<Vec<Notification>, String> {
    let db_notifications = state
        .database
        .get_notifications(Some(100))
        .await
        .map_err(|e| e.to_string())?;

    let notifications = db_notifications
        .into_iter()
        .map(|db_notif| Notification {
            id: db_notif.id,
            title: db_notif.title,
            message: db_notif.message,
            notification_type: db_notif.notification_type,
            timestamp: db_notif.created_at.to_rfc3339(),
            read: db_notif.read,
            action: None,
            source: db_notif.source,
            priority: db_notif.priority,
            metadata: db_notif
                .metadata
                .as_ref()
                .and_then(|raw| serde_json::from_str(raw).ok())
                .unwrap_or_default(),
        })
        .collect();

    Ok(notifications)
}

#[tauri::command]
async fn get_unread_notification_count(state: State<'_, AppState>) -> Result<u32, String> {
    let count = state
        .database
        .get_unread_notification_count()
        .await
        .map_err(|e| e.to_string())?;

    Ok(u32::try_from(count).unwrap_or(0))
}

#[tauri::command]
async fn mark_notification_as_read(notification_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .mark_notification_as_read(&notification_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mark_all_notifications_as_read(state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .mark_all_notifications_as_read()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn dismiss_notification(notification_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .delete_notification(&notification_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_all_notifications(state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .clear_all_notifications()
        .await
        .map_err(|e| e.to_string())
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

    state
        .database
        .create_notification(&db_notification)
        .await
        .map_err(|e| e.to_string())?;

    if let Ok(event_mgr) = state.event_manager.lock() {
        if let Some(ref manager) = *event_mgr {
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

#[tauri::command]
async fn send_native_notification(
    app: tauri::AppHandle,
    payload: NativeNotificationRequest,
) -> Result<(), String> {
    let mut builder = app
        .notification()
        .builder()
        .title(payload.title)
        .body(payload.body);

    if let Some(icon) = payload.icon {
        builder = builder.icon(icon);
    }

    if let Some(sound) = payload.sound {
        builder = builder.sound(sound);
    }

    builder.show().map_err(|e| e.to_string())
}

// Batch job management commands
#[tauri::command]
async fn get_batch_jobs(state: State<'_, AppState>) -> Result<Vec<BatchJob>, String> {
    let db_jobs = state
        .database
        .get_batch_jobs()
        .await
        .map_err(|e| e.to_string())?;

    Ok(db_jobs.into_iter().map(db_batch_job_to_ui).collect())
}

#[tauri::command]
async fn get_batch_job(job_id: String, state: State<'_, AppState>) -> Result<Option<BatchJob>, String> {
    let job = state
        .database
        .get_batch_job_by_id(&job_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.map(db_batch_job_to_ui))
}

#[tauri::command]
async fn create_batch_job(
    job_data: serde_json::Value,
    state: State<'_, AppState>
) -> Result<String, String> {
    let job_id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now();
    let inputs: Vec<String> = job_data
        .get("inputs")
        .and_then(|arr| arr.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let config = BatchJobConfig {
        inputs,
        output_format: job_data["output_format"].as_str().unwrap_or("text").to_string(),
        batch_size: job_data["batch_size"].as_u64().unwrap_or(1) as u32,
        parallel_workers: job_data["parallel_workers"].as_u64().unwrap_or(1) as u32,
    };

    let db_job = DbBatchJob {
        id: job_id.clone(),
        name: job_data["name"].as_str().unwrap_or("Unnamed Job").to_string(),
        model_id: job_data["model_id"].as_str().unwrap_or("").to_string(),
        status: "pending".to_string(),
        progress: 0.0,
        total_tasks: config.inputs.len() as i32,
        completed_tasks: 0,
        failed_tasks: 0,
        config: serde_json::to_string(&config).map_err(|e| e.to_string())?,
        results: None,
        schedule: job_data["schedule"].as_str().map(|s| s.to_string()),
        next_run: None,
        created_at,
        started_at: None,
        completed_at: None,
    };

    state
        .database
        .create_batch_job(&db_job)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job_id)
}

#[tauri::command]
async fn start_batch_job(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    // Ensure the job exists before updating status
    let job = state
        .database
        .get_batch_job_by_id(&job_id)
        .await
        .map_err(|e| e.to_string())?;

    if job.is_none() {
        return Err("Batch job not found".to_string());
    }

    state
        .database
        .update_batch_job_status(&job_id, "running")
        .await
        .map_err(|e| e.to_string())?;

    let backend_manager = state.backend_manager.clone();
    let database = state.database.clone();
    let activity_logger = state.activity_logger.clone();
    let event_manager = state.event_manager.clone();
    let job_id_clone = job_id.clone();

    if let Ok(event_mgr) = event_manager.lock() {
        if let Some(ref manager) = *event_mgr {
            let _ = manager.emit_event(InfernoEvent::BatchJobStarted {
                job_id: job_id.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    tokio::spawn(async move {
        if let Err(err) = run_batch_job(
            job_id_clone,
            backend_manager,
            database,
            activity_logger,
            event_manager,
        )
        .await
        {
            eprintln!("Batch job execution failed: {}", err);
        }
    });

    Ok(())
}

#[tauri::command]
async fn pause_batch_job(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .update_batch_job_status(&job_id, "pending")
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn cancel_batch_job(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .update_batch_job_status(&job_id, "cancelled")
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn delete_batch_job(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .database
        .delete_batch_job(&job_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

async fn run_batch_job(
    job_id: String,
    backend_manager: Arc<BackendManager>,
    database: Arc<DatabaseManager>,
    activity_logger: Arc<ActivityLogger>,
    event_manager: Arc<Mutex<Option<EventManager>>>,
) -> Result<(), String> {
    let maybe_job = database
        .get_batch_job_by_id(&job_id)
        .await
        .map_err(|e| e.to_string())?;

    let job = match maybe_job {
        Some(job) => job,
        None => return Err("Batch job not found".to_string()),
    };

    let config: BatchJobConfig = serde_json::from_str(&job.config).map_err(|e| e.to_string())?;

    if config.inputs.is_empty() {
        database
            .update_batch_job_status(&job_id, "failed")
            .await
            .map_err(|e| e.to_string())?;
        return Err("Batch job does not contain any inputs".to_string());
    }

    let backend_id = match backend_manager
        .load_model(job.model_id.clone(), "gguf".to_string())
        .await
    {
        Ok(id) => id,
        Err(e) => {
            database
                .update_batch_job_status(&job_id, "failed")
                .await
                .map_err(|err| err.to_string())?;
            return Err(e.to_string());
        }
    };

    let total_tasks = config.inputs.len() as u32;
    let mut completed_tasks = 0u32;
    let mut failed_tasks = 0u32;
    let mut outputs = Vec::with_capacity(config.inputs.len());
    let mut errors = Vec::new();
    let mut total_duration_ms = 0.0;

    let inference_params = InferenceParams {
        temperature: None,
        top_k: None,
        top_p: None,
        max_tokens: None,
        stream: Some(false),
        stop_sequences: None,
        seed: None,
    };

    let job_start = Instant::now();

    for input in config.inputs.iter() {
        let step_start = Instant::now();
        match backend_manager
            .infer(backend_id.clone(), input.clone(), inference_params.clone())
            .await
        {
            Ok(output) => {
                completed_tasks += 1;
                outputs.push(output);
            }
            Err(err) => {
                failed_tasks += 1;
                errors.push(err.to_string());
            }
        }

        total_duration_ms += step_start.elapsed().as_millis() as f64;

        let progress = if total_tasks == 0 {
            100.0
        } else {
            ((completed_tasks + failed_tasks) as f64 / total_tasks as f64 * 100.0).min(100.0)
        };

        if let Err(e) = database
            .update_batch_job_progress(
                &job_id,
                progress,
                completed_tasks as i32,
                failed_tasks as i32,
            )
            .await
        {
            eprintln!("Failed to update batch job progress: {}", e);
        }

        if let Ok(event_mgr) = event_manager.lock() {
            if let Some(ref manager) = *event_mgr {
                let _ = manager.emit_batch_job_progress(
                    job_id.clone(),
                    progress,
                    completed_tasks,
                    failed_tasks,
                );
            }
        }
    }

    let total_seconds = total_duration_ms / 1000.0;
    let metrics = BatchJobMetrics {
        total_time: total_seconds,
        avg_time_per_task: if completed_tasks + failed_tasks > 0 {
            total_seconds / (completed_tasks + failed_tasks) as f64
        } else {
            0.0
        },
        throughput: if total_seconds > 0.0 {
            completed_tasks as f64 / total_seconds
        } else {
            0.0
        },
    };

    let results = BatchJobResults {
        outputs,
        errors: errors.clone(),
        metrics,
    };

    if let Err(e) = database
        .update_batch_job_results(&job_id, &results)
        .await
    {
        eprintln!("Failed to store batch job results: {}", e);
    }

    let status = if errors.is_empty() { "completed" } else { "failed" };

    if let Err(e) = database
        .update_batch_job_status(&job_id, status)
        .await
    {
        eprintln!("Failed to set batch job status: {}", e);
    }

    if let Ok(event_mgr) = event_manager.lock() {
        if let Some(ref manager) = *event_mgr {
            let event = if status == "completed" {
                InfernoEvent::BatchJobCompleted {
                    job_id: job_id.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }
            } else {
                InfernoEvent::BatchJobFailed {
                    job_id: job_id.clone(),
                    error: if errors.is_empty() {
                        "Unknown failure".to_string()
                    } else {
                        errors.join("; ")
                    },
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }
            };
            let _ = manager.emit_event(event);
        }
    }

    activity_logger.log_simple(
        ActivityType::System,
        format!("Batch job {} {}", job_id, status),
        format!(
            "Processed {} task(s) with {} failure(s) in {:.2}s",
            completed_tasks,
            failed_tasks,
            job_start.elapsed().as_secs_f64()
        ),
        if status == "completed" {
            ActivityStatus::Success
        } else {
            ActivityStatus::Error
        },
    );

    if let Err(e) = backend_manager.unload_model(backend_id.clone()).await {
        eprintln!(
            "Failed to unload backend {} after batch job {}: {}",
            backend_id, job_id, e
        );
    }

    Ok(())
}

#[tauri::command]
async fn get_batch_job_count(state: State<'_, AppState>) -> Result<u32, String> {
    let count = state
        .database
        .get_batch_jobs()
        .await
        .map_err(|e| e.to_string())?
        .len();

    Ok(count as u32)
}

#[tauri::command]
async fn get_active_batch_job_count(state: State<'_, AppState>) -> Result<u32, String> {
    let active_count = state
        .database
        .get_active_batch_job_count()
        .await
        .map_err(|e| e.to_string())?;

    Ok(u32::try_from(active_count).unwrap_or(0))
}

// Security management commands
#[tauri::command]
async fn create_api_key(
    request: CreateApiKeyRequest,
    state: State<'_, AppState>
) -> Result<CreateApiKeyResponse, String> {
    match state.security_manager.generate_api_key(request.clone()).await {
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
    state.security_manager.get_api_keys().await
}

#[tauri::command]
async fn revoke_api_key(key_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.security_manager.revoke_api_key(key_id).await
}

#[tauri::command]
async fn delete_api_key(key_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.security_manager.delete_api_key(key_id).await
}

#[tauri::command]
async fn validate_api_key(raw_key: String, state: State<'_, AppState>) -> Result<Option<ApiKey>, String> {
    state.security_manager.validate_api_key(raw_key).await
}

#[tauri::command]
async fn get_security_events(
    limit: Option<usize>,
    state: State<'_, AppState>
) -> Result<Vec<SecurityEvent>, String> {
    state.security_manager.get_security_events(limit).await
}

#[tauri::command]
async fn get_security_metrics(state: State<'_, AppState>) -> Result<SecurityMetrics, String> {
    state.security_manager.get_security_metrics().await
}

#[tauri::command]
async fn clear_security_events(state: State<'_, AppState>) -> Result<(), String> {
    state.security_manager.clear_security_events().await
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
    use tokio::fs;

    let resolved_dir: PathBuf = if target_dir.trim().is_empty() {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        PathBuf::from(settings.models_directory.clone())
    } else {
        PathBuf::from(target_dir)
    };

    fs::create_dir_all(&resolved_dir)
        .await
        .map_err(|e| format!(
            "Failed to prepare download directory '{}': {}",
            resolved_dir.to_string_lossy(),
            e
        ))?;

    let target_dir_string = resolved_dir.to_string_lossy().to_string();

    state
        .download_manager
        .start_download(&model, &target_dir_string)
        .await
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

    // Initialize database manager
    let database_manager = match DatabaseManager::new(None).await {
        Ok(db) => Arc::new(db),
        Err(e) => {
            eprintln!("Failed to initialize database manager: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize security manager
    let security_manager = Arc::new(SecurityManager::new(database_manager.clone()));

    // Initialize model repository service
    let model_repository = Arc::new(ModelRepositoryService::new());

    // Initialize download manager
    let default_models_directory = settings.models_directory.clone();
    let download_manager = Arc::new(
        ModelDownloadManager::new().with_default_target(default_models_directory),
    );

    let app_state = AppState {
        system: Arc::new(Mutex::new(System::new_all())),
        backend_manager,
        metrics: Arc::new(Mutex::new(initialize_metrics())),
        activity_logger,
        settings: Arc::new(Mutex::new(settings)),
        security_manager,
        event_manager: Arc::new(Mutex::new(None)),
        database: database_manager,
        model_repository,
        download_manager,
        streaming_sessions: Arc::new(AtomicU32::new(0)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state)
        .plugin(tauri_plugin_notification::init())
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

            let app_handle = app.handle();
            let app_menu = build_app_menu(&app_handle)?;
            app.set_menu(app_menu.clone())?;
            app_handle.on_menu_event(|app_handle, event: MenuEvent| {
                handle_menu_event(app_handle, event.id().as_ref());
            });

            let tray_menu = build_tray_menu(&app_handle)?;

            // Create system tray
            let _tray = TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app_handle, event| {
                    handle_tray_menu_event(app_handle, event.id.as_ref());
                })
                .on_tray_icon_event(|tray, event| {
                    match event {
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } => {
                            show_main_window(&tray.app_handle());
                        }
                        TrayIconEvent::Click {
                            button: MouseButton::Right,
                            button_state: MouseButtonState::Up,
                            ..
                        } => {
                            // native menu handling
                        }
                        _ => {}
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
            send_native_notification,
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
