//! # DEPRECATED: Tauri v1 Application (v0.5.0+)
//!
//! **⚠️ DEPRECATION NOTICE ⚠️**
//!
//! This module is deprecated as of v0.5.0 and will be removed in v0.6.0.
//!
//! ## Migration Path
//!
//! **Old (Tauri v1)**: `src/tauri_app.rs` with 14 commands
//! **New (Tauri v2)**: `src/interfaces/desktop/` with 51 commands
//!
//! ### Why Deprecated?
//!
//! 1. **Tauri v2 is Superior**: Modern architecture, better performance
//! 2. **More Features**: 51 commands vs 14 (3.6x more functionality)
//! 3. **Better Integration**: Unified with dashboard implementation
//! 4. **Apple Silicon Optimized**: Native M1/M2/M3/M4 support
//! 5. **Dependency Conflicts**: Tauri v1 and v2 cannot coexist
//!
//! ### How to Migrate
//!
//! Instead of using the `inferno_app` binary, use the new desktop app:
//!
//! ```bash
//! # OLD (deprecated):
//! cargo run --bin inferno_app --features tauri-app
//!
//! # NEW (recommended):
//! cd dashboard && npm run tauri dev
//! ./scripts/build-desktop.sh --release
//! ```
//!
//! ### Feature Comparison
//!
//! | Feature | Tauri v1 (Old) | Tauri v2 (New) |
//! |---------|---------------|----------------|
//! | Commands | 14 | 51 |
//! | Plugins | Limited | Full ecosystem |
//! | Performance | Good | Excellent |
//! | macOS Integration | Basic | Native + optimized |
//! | Updates | Manual | Automatic |
//!
//! See: `src/interfaces/desktop/` for the new implementation.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{command, generate_context, generate_handler, AppHandle, Manager, State, Window};
use tokio::sync::Mutex;

#[cfg(feature = "desktop")]
use crate::macos_integration::{
    create_app_menu, create_system_tray, get_system_appearance, handle_menu_event,
    handle_system_tray_event, minimize_to_tray, send_native_notification, set_window_vibrancy,
    toggle_always_on_top, MacOSNotification,
};

use crate::{
    backends::{Backend, BackendConfig, BackendType, InferenceParams},
    config::Config,
    metrics::MetricsCollector,
    models::{ModelInfo, ModelManager},
};

// App state for Tauri
#[derive(Default)]
pub struct AppState {
    backends: Mutex<HashMap<String, Backend>>,
    model_manager: Mutex<ModelManager>,
    metrics: Mutex<MetricsCollector>,
    config: Mutex<Config>,
}

// Tauri commands
#[command]
async fn get_models(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    let model_manager = state.model_manager.lock().await;
    model_manager.list_models().await.map_err(|e| e.to_string())
}

#[command]
async fn load_model(
    model_name: String,
    backend_type: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut backends = state.backends.lock().await;
    let model_manager = state.model_manager.lock().await;
    let config = state.config.lock().await;

    // Find the model
    let models = model_manager
        .list_models()
        .await
        .map_err(|e| e.to_string())?;
    let model = models
        .iter()
        .find(|m| m.name == model_name)
        .ok_or_else(|| format!("Model '{}' not found", model_name))?;

    // Create backend
    let backend_type = match backend_type.as_str() {
        "gguf" => BackendType::Gguf,
        "onnx" => BackendType::Onnx,
        _ => return Err(format!("Unsupported backend type: {}", backend_type)),
    };

    let backend_config = config.backend_config.clone();
    let mut backend = Backend::new(backend_type, &backend_config).map_err(|e| e.to_string())?;

    // Load the model
    backend.load_model(model).await.map_err(|e| e.to_string())?;

    let backend_id = format!("{}_{}", model_name, backend_type);
    backends.insert(backend_id.clone(), backend);

    Ok(backend_id)
}

#[command]
async fn unload_model(backend_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut backends = state.backends.lock().await;
    if let Some(mut backend) = backends.remove(&backend_id) {
        backend.unload_model().await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[command]
async fn infer(
    backend_id: String,
    prompt: String,
    params: InferenceParams,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut backends = state.backends.lock().await;
    let backend = backends
        .get_mut(&backend_id)
        .ok_or_else(|| format!("Backend '{}' not found", backend_id))?;

    backend
        .infer(&prompt, &params)
        .await
        .map_err(|e| e.to_string())
}

#[command]
async fn get_metrics(state: State<'_, AppState>) -> Result<MetricsSnapshot, String> {
    let metrics = state.metrics.lock().await;
    metrics.get_snapshot().await.map_err(|e| e.to_string())
}

#[command]
async fn get_loaded_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let backends = state.backends.lock().await;
    let loaded_models: Vec<String> = backends.keys().cloned().collect();
    Ok(loaded_models)
}

#[command]
async fn validate_model(model_path: String) -> Result<bool, String> {
    // Basic validation - check if file exists and has correct extension
    let path = std::path::Path::new(&model_path);
    if !path.exists() {
        return Ok(false);
    }

    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    match extension {
        "gguf" | "onnx" | "pt" | "safetensors" => Ok(true),
        _ => Ok(false),
    }
}

#[command]
async fn open_file_dialog(window: Window) -> Result<Option<String>, String> {
    use tauri::api::dialog::FileDialogBuilder;

    let file_path = FileDialogBuilder::new()
        .add_filter("Model Files", &["gguf", "onnx", "pt", "safetensors"])
        .pick_file()
        .await;

    Ok(file_path.map(|p| p.to_string_lossy().to_string()))
}

#[command]
async fn get_system_info() -> Result<SystemInfo, String> {
    use sysinfo::{CpuExt, System, SystemExt};

    let mut system = System::new_all();
    system.refresh_all();

    let cpu_info = system.global_cpu_info();
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();

    Ok(SystemInfo {
        cpu_name: cpu_info.brand().to_string(),
        cpu_usage: cpu_info.cpu_usage(),
        cpu_cores: num_cpus::get(),
        total_memory,
        used_memory,
        available_memory: total_memory - used_memory,
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    })
}

#[derive(Serialize, Deserialize)]
struct SystemInfo {
    cpu_name: String,
    cpu_usage: f32,
    cpu_cores: usize,
    total_memory: u64,
    used_memory: u64,
    available_memory: u64,
    platform: String,
    arch: String,
}

#[derive(Serialize, Deserialize)]
struct MetricsSnapshot {
    inference_count: u64,
    success_count: u64,
    error_count: u64,
    average_latency: f64,
    models_loaded: usize,
}

impl From<crate::metrics::MetricsSnapshot> for MetricsSnapshot {
    fn from(snapshot: crate::metrics::MetricsSnapshot) -> Self {
        Self {
            inference_count: snapshot.inference_count,
            success_count: snapshot.success_count,
            error_count: snapshot.error_count,
            average_latency: snapshot.average_latency,
            models_loaded: snapshot.models_loaded,
        }
    }
}

// Main Tauri app runner
pub fn run_tauri_app() -> Result<()> {
    let context = generate_context!();

    tauri::Builder::default()
        .menu(create_app_menu())
        .system_tray(create_system_tray())
        .on_system_tray_event(handle_system_tray_event)
        .on_menu_event(|event| {
            handle_menu_event(&event.window(), &event.menu_item_id());
        })
        .manage(AppState::default())
        .invoke_handler(generate_handler![
            get_models,
            load_model,
            unload_model,
            infer,
            get_metrics,
            get_loaded_models,
            validate_model,
            open_file_dialog,
            get_system_info,
            send_native_notification,
            get_system_appearance,
            set_window_vibrancy,
            toggle_always_on_top,
            minimize_to_tray
        ])
        .setup(|app| {
            // Initialize app state
            let app_handle = app.handle();
            let state = app_handle.state::<AppState>();

            tauri::async_runtime::spawn(async move {
                let config = Config::load().unwrap_or_default();
                let model_manager = ModelManager::new(&config.models_dir);
                let metrics = MetricsCollector::new();

                // Start metrics collection
                if let Err(e) = metrics.start_event_processing().await {
                    eprintln!("Failed to start metrics collection: {}", e);
                }

                // Initialize state
                *state.config.lock().await = config;
                *state.model_manager.lock().await = model_manager;
                *state.metrics.lock().await = metrics;
            });

            Ok(())
        })
        .run(context)
        .map_err(|e| anyhow::anyhow!("Tauri error: {}", e))?;

    Ok(())
}
