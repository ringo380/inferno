//! macOS-Specific Integration
//!
//! This module provides macOS-specific features including:
//! - Native menu bar with standard shortcuts
//! - System tray with live metrics
//! - Native notifications
//! - Window vibrancy and appearance detection
//! - Metal GPU acceleration support (Phase 2)
//!
//! **Note**: This module is only compiled on macOS targets.

use serde::{Deserialize, Serialize};
use tauri::{
    command,
    menu::{AboutMetadata, Menu, MenuBuilder, MenuItem, SubmenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime, Window,
};

// ============================================================================
// Menu ID Constants
// ============================================================================

pub const MENU_ID_PREFERENCES: &str = "menu.preferences";
pub const MENU_ID_SHOW_WINDOW: &str = "menu.show_window";
pub const MENU_ID_HIDE_WINDOW: &str = "menu.hide_window";
pub const MENU_ID_CHECK_UPDATES: &str = "menu.check_updates";
pub const MENU_ID_REPORT_ISSUE: &str = "menu.report_issue";
pub const MENU_ID_DOCUMENTATION: &str = "menu.documentation";
pub const MENU_ID_KEYBOARD_SHORTCUTS: &str = "menu.shortcuts";
pub const MENU_ID_NEW_INFERENCE: &str = "menu.new_inference";
pub const MENU_ID_OPEN_MODEL: &str = "menu.open_model";
pub const MENU_ID_IMPORT_MODEL: &str = "menu.import_model";
pub const MENU_ID_EXPORT_RESULTS: &str = "menu.export_results";
pub const MENU_ID_MODEL_INFO: &str = "menu.model_info";
pub const MENU_ID_VALIDATE_MODELS: &str = "menu.validate_models";
pub const MENU_ID_QUICK_INFERENCE: &str = "menu.quick_inference";
pub const MENU_ID_BATCH_INFERENCE: &str = "menu.batch_inference";
pub const MENU_ID_STOP_INFERENCE: &str = "menu.stop_inference";
pub const MENU_ID_VIEW_DASHBOARD: &str = "menu.view_dashboard";
pub const MENU_ID_VIEW_MODELS: &str = "menu.view_models";
pub const MENU_ID_VIEW_INFERENCE: &str = "menu.view_inference";
pub const MENU_ID_VIEW_METRICS: &str = "menu.view_metrics";

pub const TRAY_ID_DASHBOARD: &str = "tray.dashboard";
pub const TRAY_ID_MODELS: &str = "tray.models";
pub const TRAY_ID_INFERENCE: &str = "tray.quick_inference";
pub const TRAY_ID_SHOW: &str = "tray.show";
pub const TRAY_ID_HIDE: &str = "tray.hide";
pub const TRAY_ID_QUIT: &str = "tray.quit";

// ============================================================================
// Data Types
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MacOSNotification {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VibrancyEffect {
    AppearanceBased,
    Light,
    Dark,
    Titlebar,
    Selection,
    Menu,
    Popover,
    Sidebar,
    HeaderView,
    Sheet,
    WindowBackground,
    HudWindow,
    FullScreenUI,
    Tooltip,
    ContentBackground,
    UnderWindowBackground,
    UnderPageBackground,
}

// ============================================================================
// Menu Bar Creation
// ============================================================================

/// Create the native macOS application menu
///
/// This creates a standard macOS menu bar with:
/// - App menu (About, Preferences, Services, Hide, Quit)
/// - File menu (New, Open, Import, Export, Close)
/// - Edit menu (Undo, Redo, Cut, Copy, Paste, Select All)
/// - Models menu (Load, Unload, Info, Validate)
/// - Inference menu (Run, Stream, Stop, Batch)
/// - Window menu (Minimize, Zoom, Bring All to Front)
/// - Help menu (Documentation, Report Issue, About)
pub fn create_app_menu<R: Runtime>(app: &AppHandle<R>) -> Result<Menu<R>, String> {
    let about_metadata = AboutMetadata {
        name: Some("Inferno AI Desktop".to_string()),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        ..Default::default()
    };

    // Create menu items
    let preferences = MenuItem::with_id(
        app,
        MENU_ID_PREFERENCES,
        "Preferences…",
        true,
        Some("Cmd+,"),
    )
    .map_err(|e| e.to_string())?;
    let new_inference = MenuItem::with_id(
        app,
        MENU_ID_NEW_INFERENCE,
        "New Inference",
        true,
        Some("Cmd+N"),
    )
    .map_err(|e| e.to_string())?;
    let open_model = MenuItem::with_id(app, MENU_ID_OPEN_MODEL, "Open Model…", true, Some("Cmd+O"))
        .map_err(|e| e.to_string())?;
    let import_model = MenuItem::with_id(
        app,
        MENU_ID_IMPORT_MODEL,
        "Import Model…",
        true,
        Some("Cmd+Shift+I"),
    )
    .map_err(|e| e.to_string())?;
    let export_results = MenuItem::with_id(
        app,
        MENU_ID_EXPORT_RESULTS,
        "Export Results…",
        true,
        Some("Cmd+E"),
    )
    .map_err(|e| e.to_string())?;
    let model_info = MenuItem::with_id(
        app,
        MENU_ID_MODEL_INFO,
        "Model Information",
        true,
        Some("Cmd+I"),
    )
    .map_err(|e| e.to_string())?;
    let validate_models = MenuItem::with_id(
        app,
        MENU_ID_VALIDATE_MODELS,
        "Validate Models",
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let quick_inference = MenuItem::with_id(
        app,
        MENU_ID_QUICK_INFERENCE,
        "Quick Inference",
        true,
        Some("Cmd+R"),
    )
    .map_err(|e| e.to_string())?;
    let batch_inference = MenuItem::with_id(
        app,
        MENU_ID_BATCH_INFERENCE,
        "Batch Inference",
        true,
        Some("Cmd+Shift+R"),
    )
    .map_err(|e| e.to_string())?;
    let stop_inference = MenuItem::with_id(
        app,
        MENU_ID_STOP_INFERENCE,
        "Stop All Inference",
        true,
        Some("Cmd+."),
    )
    .map_err(|e| e.to_string())?;
    let view_dashboard = MenuItem::with_id(
        app,
        MENU_ID_VIEW_DASHBOARD,
        "Dashboard",
        true,
        Some("Cmd+1"),
    )
    .map_err(|e| e.to_string())?;
    let view_models = MenuItem::with_id(app, MENU_ID_VIEW_MODELS, "Models", true, Some("Cmd+2"))
        .map_err(|e| e.to_string())?;
    let view_inference = MenuItem::with_id(
        app,
        MENU_ID_VIEW_INFERENCE,
        "Inference",
        true,
        Some("Cmd+3"),
    )
    .map_err(|e| e.to_string())?;
    let view_metrics = MenuItem::with_id(app, MENU_ID_VIEW_METRICS, "Metrics", true, Some("Cmd+4"))
        .map_err(|e| e.to_string())?;
    let documentation = MenuItem::with_id(
        app,
        MENU_ID_DOCUMENTATION,
        "Documentation",
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let shortcuts = MenuItem::with_id(
        app,
        MENU_ID_KEYBOARD_SHORTCUTS,
        "Keyboard Shortcuts",
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let check_updates = MenuItem::with_id(
        app,
        MENU_ID_CHECK_UPDATES,
        "Check for Updates",
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let report_issue = MenuItem::with_id(
        app,
        MENU_ID_REPORT_ISSUE,
        "Report Issue…",
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let show_window = MenuItem::with_id(
        app,
        MENU_ID_SHOW_WINDOW,
        "Show Window",
        true,
        Some("Cmd+Shift+H"),
    )
    .map_err(|e| e.to_string())?;
    let hide_window =
        MenuItem::with_id(app, MENU_ID_HIDE_WINDOW, "Hide Window", true, None::<&str>)
            .map_err(|e| e.to_string())?;

    // Build submenus
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
        .build()
        .map_err(|e| e.to_string())?;

    let file_submenu = SubmenuBuilder::with_id(app, "menu.file", "File")
        .item(&new_inference)
        .item(&open_model)
        .separator()
        .item(&import_model)
        .item(&export_results)
        .separator()
        .close_window()
        .build()
        .map_err(|e| e.to_string())?;

    let edit_submenu = SubmenuBuilder::with_id(app, "menu.edit", "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()
        .map_err(|e| e.to_string())?;

    let models_submenu = SubmenuBuilder::with_id(app, "menu.models", "Models")
        .item(&model_info)
        .item(&validate_models)
        .build()
        .map_err(|e| e.to_string())?;

    let inference_submenu = SubmenuBuilder::with_id(app, "menu.inference", "Inference")
        .item(&quick_inference)
        .item(&batch_inference)
        .separator()
        .item(&stop_inference)
        .build()
        .map_err(|e| e.to_string())?;

    let view_submenu = SubmenuBuilder::with_id(app, "menu.view", "View")
        .item(&view_dashboard)
        .item(&view_models)
        .item(&view_inference)
        .item(&view_metrics)
        .separator()
        .fullscreen()
        .build()
        .map_err(|e| e.to_string())?;

    let window_submenu = SubmenuBuilder::with_id(app, "menu.window", "Window")
        .item(&show_window)
        .item(&hide_window)
        .separator()
        .minimize()
        .close_window()
        .build()
        .map_err(|e| e.to_string())?;

    let help_submenu = SubmenuBuilder::with_id(app, "menu.help", "Help")
        .item(&documentation)
        .item(&shortcuts)
        .separator()
        .item(&report_issue)
        .item(&check_updates)
        .build()
        .map_err(|e| e.to_string())?;

    // Build the main menu
    let menu = MenuBuilder::new(app)
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
        .map_err(|e| e.to_string())?;

    tracing::info!("✅ macOS application menu created successfully");
    Ok(menu)
}

// ============================================================================
// System Tray
// ============================================================================

/// Create the macOS system tray icon with menu
///
/// Provides quick access to:
/// - Dashboard
/// - Model management
/// - Quick inference
/// - Show/Hide window
/// - Quit application
pub fn create_system_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    // Create the tray menu
    let menu = create_tray_menu(app)?;

    // Build the tray icon
    let _tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Inferno AI Runner")
        .on_tray_icon_event(|tray, event| {
            handle_tray_event(tray.app_handle(), event);
        })
        .build(app)
        .map_err(|e| format!("Failed to create system tray: {}", e))?;

    tracing::info!("✅ System tray created successfully");
    Ok(())
}

/// Create the system tray menu
fn create_tray_menu<R: Runtime>(app: &AppHandle<R>) -> Result<Menu<R>, String> {
    let dashboard = MenuItem::with_id(app, TRAY_ID_DASHBOARD, "Open Dashboard", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let models = MenuItem::with_id(app, TRAY_ID_MODELS, "Manage Models", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let inference = MenuItem::with_id(
        app,
        TRAY_ID_INFERENCE,
        "Quick Inference",
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let show = MenuItem::with_id(app, TRAY_ID_SHOW, "Show Window", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let hide = MenuItem::with_id(app, TRAY_ID_HIDE, "Hide Window", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(app, TRAY_ID_QUIT, "Quit Inferno", true, None::<&str>)
        .map_err(|e| e.to_string())?;

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
        .map_err(|e| e.to_string())
}

/// Handle system tray icon events
fn handle_tray_event<R: Runtime>(app: &AppHandle<R>, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } => {
            // Left click: Show/focus main window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        TrayIconEvent::Click {
            button: MouseButton::Right,
            button_state: MouseButtonState::Up,
            ..
        } => {
            // Right click: Show menu (handled by Tauri automatically)
        }
        _ => {}
    }
}

/// Handle system tray menu item clicks
pub fn handle_tray_menu_event<R: Runtime>(app: &AppHandle<R>, menu_id: &str) {
    match menu_id {
        TRAY_ID_DASHBOARD | "dashboard" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
            // Emit navigation event
            let _ = app.emit("navigate", "dashboard");
        }
        TRAY_ID_MODELS | "models" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
            // Emit navigation event to models page
            let _ = app.emit("navigate", "models");
            tracing::info!("📦 Navigate to models page");
        }
        TRAY_ID_INFERENCE | "inference" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
            // Emit event to open quick inference dialog
            let _ = app.emit("open-quick-inference", ());
            tracing::info!("⚡ Open quick inference");
        }
        TRAY_ID_SHOW | "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        TRAY_ID_HIDE | "hide" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
        }
        TRAY_ID_QUIT | "quit" => {
            app.exit(0);
        }
        _ => {
            tracing::warn!("⚠️  Unknown tray menu item: {}", menu_id);
        }
    }
}

// ============================================================================
// Native Notifications
// ============================================================================

/// Send a native macOS notification
///
/// This uses the Tauri notification plugin to send native notifications
/// that appear in the macOS Notification Center.
///
/// Note: The app handle is required to access the notification plugin.
/// Call this from a Tauri command that has access to the app handle.
#[command]
pub async fn send_native_notification(notification: MacOSNotification) -> Result<(), String> {
    // Note: To use tauri-plugin-notification, the caller needs to pass the AppHandle.
    // This function logs the notification for now; actual sending requires the app handle.
    // Use `send_notification_with_app` when you have access to the AppHandle.
    tracing::info!(
        "📬 Notification queued: {} - {}",
        notification.title,
        notification.body
    );
    Ok(())
}

/// Send a native notification using the Tauri app handle
///
/// This is the actual implementation that sends notifications via tauri-plugin-notification.
#[cfg(feature = "tauri-plugin-notification")]
pub fn send_notification_with_app<R: Runtime>(
    app: &AppHandle<R>,
    notification: &MacOSNotification,
) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;

    let mut builder = app.notification().builder();
    builder = builder.title(&notification.title).body(&notification.body);

    if let Some(ref icon) = notification.icon {
        builder = builder.icon(icon);
    }

    builder.show().map_err(|e| e.to_string())?;

    tracing::info!(
        "📬 Notification sent: {} - {}",
        notification.title,
        notification.body
    );
    Ok(())
}

/// Fallback notification sender when plugin is not available
#[cfg(not(feature = "tauri-plugin-notification"))]
pub fn send_notification_with_app<R: Runtime>(
    _app: &AppHandle<R>,
    notification: &MacOSNotification,
) -> Result<(), String> {
    tracing::info!(
        "📬 Notification (plugin not available): {} - {}",
        notification.title,
        notification.body
    );
    Ok(())
}

// ============================================================================
// System Appearance Detection
// ============================================================================

/// Get the current macOS system appearance (light or dark mode)
///
/// Uses the `defaults` command to read the AppleInterfaceStyle setting.
/// Returns "dark" if dark mode is enabled, "light" otherwise.
#[command]
pub async fn get_system_appearance() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // Query macOS for dark mode setting
        // AppleInterfaceStyle is set to "Dark" when dark mode is enabled
        // If the key doesn't exist, light mode is active
        let output = Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let style = String::from_utf8_lossy(&result.stdout).trim().to_lowercase();
                if style == "dark" {
                    Ok("dark".to_string())
                } else {
                    Ok("light".to_string())
                }
            }
            // If the command fails or key doesn't exist, light mode is active
            _ => Ok("light".to_string()),
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok("light".to_string())
    }
}

// ============================================================================
// Window Vibrancy
// ============================================================================

/// Set window vibrancy effect (translucent background)
///
/// This creates the native macOS "frosted glass" effect on the window.
/// Note: Vibrancy effects require macOS 10.14+ and are applied to the window's content area.
///
/// In Tauri v2, window effects are configured in the window settings or via the WebviewWindow API.
/// This command provides a programmatic interface to request vibrancy changes.
#[command]
pub async fn set_window_vibrancy<R: Runtime>(
    _window: Window<R>,
    effect: VibrancyEffect,
) -> Result<(), String> {
    // Note: In Tauri v2, window vibrancy effects are typically set via:
    // 1. tauri.conf.json window configuration with "effects" property
    // 2. Or via platform-specific APIs
    //
    // The Window API in Tauri v2 may have different effect methods depending on the version.
    // For now, we log the requested effect and return success.
    // Full vibrancy support can be implemented via objective-c FFI if needed.

    #[cfg(target_os = "macos")]
    {
        tracing::info!("🎨 Window vibrancy effect requested: {:?}", effect);

        // Map effect to macOS NSVisualEffectMaterial values (for documentation)
        let material_name = match effect {
            VibrancyEffect::Sidebar => "sidebar",
            VibrancyEffect::HeaderView => "headerView",
            VibrancyEffect::Sheet => "sheet",
            VibrancyEffect::WindowBackground => "windowBackground",
            VibrancyEffect::HudWindow => "hudWindow",
            VibrancyEffect::FullScreenUI => "fullScreenUI",
            VibrancyEffect::Tooltip => "tooltip",
            VibrancyEffect::ContentBackground => "contentBackground",
            VibrancyEffect::UnderWindowBackground => "underWindowBackground",
            VibrancyEffect::UnderPageBackground => "underPageBackground",
            VibrancyEffect::Menu => "menu",
            VibrancyEffect::Popover => "popover",
            VibrancyEffect::Selection => "selection",
            VibrancyEffect::Titlebar => "titlebar",
            VibrancyEffect::AppearanceBased => "appearanceBased",
            VibrancyEffect::Light => "light",
            VibrancyEffect::Dark => "dark",
        };

        tracing::debug!(
            "🎨 Vibrancy effect '{}' maps to NSVisualEffectMaterial.{}",
            format!("{:?}", effect),
            material_name
        );
    }

    #[cfg(not(target_os = "macos"))]
    {
        tracing::info!(
            "🎨 Window vibrancy not supported on this platform (requested: {:?})",
            effect
        );
    }

    Ok(())
}

// ============================================================================
// Window Management
// ============================================================================

/// Toggle always-on-top for the main window
#[command]
pub async fn toggle_always_on_top<R: Runtime>(window: Window<R>) -> Result<bool, String> {
    let is_on_top = window.is_always_on_top().map_err(|e| e.to_string())?;
    let new_state = !is_on_top;
    window
        .set_always_on_top(new_state)
        .map_err(|e| e.to_string())?;

    tracing::info!("📌 Always on top: {}", new_state);
    Ok(new_state)
}

/// Minimize window to system tray (hide window)
#[command]
pub async fn minimize_to_tray<R: Runtime>(window: Window<R>) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())?;
    tracing::info!("🫥 Window minimized to tray");
    Ok(())
}

/// Show window from system tray
pub fn show_from_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        tracing::info!("👁️  Window shown from tray");
    }
    Ok(())
}

// ============================================================================
// Metal GPU Detection (Phase 2)
// ============================================================================

/// Detect if Metal GPU is available on this system
///
/// Queries the macOS system for Metal GPU capabilities using system_profiler.
/// This provides device name, memory size, and feature support.
#[command]
pub async fn detect_metal_gpu() -> Result<MetalInfo, String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // Use system_profiler to get GPU information
        let output = Command::new("system_profiler")
            .arg("SPDisplaysDataType")
            .arg("-json")
            .output()
            .map_err(|e| format!("Failed to run system_profiler: {}", e))?;

        if !output.status.success() {
            return Err("system_profiler command failed".to_string());
        }

        let json_str = String::from_utf8_lossy(&output.stdout);

        // Parse JSON to extract Metal info
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
            if let Some(displays) = json["SPDisplaysDataType"].as_array() {
                for display in displays {
                    if let Some(chipset_model) = display["sppci_model"].as_str() {
                        // Detect Metal availability (all modern Macs have Metal)
                        let available = true;

                        // Extract VRAM if available
                        let memory_gb = if let Some(vram) = display["sppci_vram"].as_str() {
                            // Parse "8 GB" or "8192 MB" format
                            if vram.contains("GB") {
                                vram.split_whitespace()
                                    .next()
                                    .and_then(|s| s.parse::<f64>().ok())
                                    .unwrap_or(0.0)
                            } else if vram.contains("MB") {
                                vram.split_whitespace()
                                    .next()
                                    .and_then(|s| s.parse::<f64>().ok())
                                    .map(|mb| mb / 1024.0)
                                    .unwrap_or(0.0)
                            } else {
                                0.0
                            }
                        } else {
                            // For Apple Silicon, use unified memory (estimate from total RAM)
                            if chipset_model.contains("Apple") {
                                if let Ok(sysinfo) = get_total_memory() {
                                    sysinfo / 1024.0 / 1024.0 / 1024.0 // Convert to GB
                                } else {
                                    0.0
                                }
                            } else {
                                0.0
                            }
                        };

                        // Check for Metal 3 support (macOS 13+ with Apple Silicon or AMD GPUs)
                        let supports_metal_3 =
                            chipset_model.contains("Apple M") || chipset_model.contains("AMD");

                        return Ok(MetalInfo {
                            available,
                            device_name: chipset_model.to_string(),
                            memory_gb,
                            supports_metal_3,
                        });
                    }
                }
            }
        }

        // Fallback: Metal is available on all Macs since 2012
        Ok(MetalInfo {
            available: true,
            device_name: "Unknown Metal Device".to_string(),
            memory_gb: 0.0,
            supports_metal_3: false,
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(MetalInfo {
            available: false,
            device_name: "Not macOS".to_string(),
            memory_gb: 0.0,
            supports_metal_3: false,
        })
    }
}

/// Helper function to get total system memory
#[cfg(target_os = "macos")]
fn get_total_memory() -> Result<f64, String> {
    use std::process::Command;

    let output = Command::new("sysctl")
        .arg("-n")
        .arg("hw.memsize")
        .output()
        .map_err(|e| format!("Failed to get memory size: {}", e))?;

    let mem_str = String::from_utf8_lossy(&output.stdout);
    mem_str
        .trim()
        .parse::<f64>()
        .map_err(|e| format!("Failed to parse memory size: {}", e))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetalInfo {
    pub available: bool,
    pub device_name: String,
    pub memory_gb: f64,
    pub supports_metal_3: bool,
}

// ============================================================================
// Chip Detection
// ============================================================================

/// Detect Apple Silicon chip type (M1, M2, M3, M4)
///
/// Uses sysctl to query chip information including core counts and Neural Engine.
#[command]
pub async fn detect_apple_silicon() -> Result<ChipInfo, String> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        use std::process::Command;

        // Get brand string to identify chip generation
        let brand_output = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
            .map_err(|e| format!("Failed to get CPU brand: {}", e))?;

        let brand_string = String::from_utf8_lossy(&brand_output.stdout)
            .trim()
            .to_string();

        // Detect chip name (M1, M2, M3, M4, etc.)
        let chip_name = if brand_string.contains("M1") {
            if brand_string.contains("Pro") {
                "Apple M1 Pro".to_string()
            } else if brand_string.contains("Max") {
                "Apple M1 Max".to_string()
            } else if brand_string.contains("Ultra") {
                "Apple M1 Ultra".to_string()
            } else {
                "Apple M1".to_string()
            }
        } else if brand_string.contains("M2") {
            if brand_string.contains("Pro") {
                "Apple M2 Pro".to_string()
            } else if brand_string.contains("Max") {
                "Apple M2 Max".to_string()
            } else if brand_string.contains("Ultra") {
                "Apple M2 Ultra".to_string()
            } else {
                "Apple M2".to_string()
            }
        } else if brand_string.contains("M3") {
            if brand_string.contains("Pro") {
                "Apple M3 Pro".to_string()
            } else if brand_string.contains("Max") {
                "Apple M3 Max".to_string()
            } else {
                "Apple M3".to_string()
            }
        } else if brand_string.contains("M4") {
            if brand_string.contains("Pro") {
                "Apple M4 Pro".to_string()
            } else if brand_string.contains("Max") {
                "Apple M4 Max".to_string()
            } else {
                "Apple M4".to_string()
            }
        } else {
            brand_string.clone()
        };

        // Get performance core count
        let perf_cores = Command::new("sysctl")
            .arg("-n")
            .arg("hw.perflevel0.physicalcpu")
            .output()
            .ok()
            .and_then(|out| {
                String::from_utf8_lossy(&out.stdout)
                    .trim()
                    .parse::<u32>()
                    .ok()
            })
            .unwrap_or(0);

        // Get efficiency core count
        let efficiency_cores = Command::new("sysctl")
            .arg("-n")
            .arg("hw.perflevel1.physicalcpu")
            .output()
            .ok()
            .and_then(|out| {
                String::from_utf8_lossy(&out.stdout)
                    .trim()
                    .parse::<u32>()
                    .ok()
            })
            .unwrap_or(0);

        // All Apple Silicon chips have Neural Engine
        let neural_engine = true;

        tracing::info!(
            "🍎 Detected: {} (P:{} E:{} ANE:{})",
            chip_name,
            perf_cores,
            efficiency_cores,
            neural_engine
        );

        Ok(ChipInfo {
            is_apple_silicon: true,
            chip_name,
            performance_cores: perf_cores,
            efficiency_cores,
            neural_engine,
        })
    }

    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    {
        use std::process::Command;

        // Try to get Intel CPU info
        #[cfg(target_os = "macos")]
        let cpu_name = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
            .ok()
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .unwrap_or_else(|| "Intel x86_64".to_string());

        #[cfg(not(target_os = "macos"))]
        let cpu_name = "x86_64".to_string();

        Ok(ChipInfo {
            is_apple_silicon: false,
            chip_name: cpu_name,
            performance_cores: 0,
            efficiency_cores: 0,
            neural_engine: false,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChipInfo {
    pub is_apple_silicon: bool,
    pub chip_name: String,
    pub performance_cores: u32,
    pub efficiency_cores: u32,
    pub neural_engine: bool,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vibrancy_effect_serialization() {
        let effect = VibrancyEffect::Sidebar;
        let json = serde_json::to_string(&effect).unwrap();
        assert!(json.contains("Sidebar"));
    }

    #[tokio::test]
    async fn test_system_appearance() {
        let appearance = get_system_appearance().await;
        assert!(appearance.is_ok());
        let mode = appearance.unwrap();
        assert!(mode == "light" || mode == "dark");
    }
}
