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
    menu::{Menu, MenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime, Window,
};

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
    // TODO: Implement Tauri v2 menu API
    // This will replace the old Tauri v1 Menu API
    //
    // Tauri v2 uses a new menu builder pattern:
    // let menu = Menu::new(app)?;
    // let app_submenu = Submenu::new(app, "Inferno", ...)?;
    // menu.append(&app_submenu)?;

    tracing::warn!("📋 Menu creation not yet implemented (Tauri v2 API)");
    Err("Menu creation pending Tauri v2 migration".to_string())
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
    // TODO: Implement Tauri v2 tray menu
    //
    // Example Tauri v2 API:
    // let menu = Menu::new(app)?;
    // let dashboard = MenuItem::with_id(app, "dashboard", "Open Dashboard", true, None)?;
    // menu.append(&dashboard)?;

    tracing::warn!("📋 Tray menu creation not yet implemented (Tauri v2 API)");
    Err("Tray menu pending Tauri v2 migration".to_string())
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
        "dashboard" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "models" => {
            // TODO: Emit event to navigate to models page
            tracing::info!("📦 Navigate to models page");
        }
        "inference" => {
            // TODO: Emit event to open quick inference dialog
            tracing::info!("⚡ Open quick inference");
        }
        "quit" => {
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
#[command]
pub async fn send_native_notification(notification: MacOSNotification) -> Result<(), String> {
    // TODO: Implement using tauri-plugin-notification
    //
    // Example:
    // use tauri_plugin_notification::NotificationExt;
    // app.notification()
    //     .builder()
    //     .title(&notification.title)
    //     .body(&notification.body)
    //     .show()
    //     .map_err(|e| e.to_string())?;

    tracing::info!(
        "📬 Notification: {} - {}",
        notification.title,
        notification.body
    );
    Ok(())
}

// ============================================================================
// System Appearance Detection
// ============================================================================

/// Get the current macOS system appearance (light or dark mode)
#[command]
pub async fn get_system_appearance() -> Result<String, String> {
    // TODO: Implement using macOS system APIs
    //
    // This requires:
    // 1. Objective-C FFI to query NSAppearance
    // 2. Or use a crate like `dark-light` for cross-platform detection
    //
    // For now, return a placeholder

    #[cfg(target_os = "macos")]
    {
        // Check if dark mode is enabled
        // This is a simplified check - proper implementation would use NSAppearance
        match std::env::var("DARKMODE") {
            Ok(val) if val == "1" => Ok("dark".to_string()),
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
#[command]
pub async fn set_window_vibrancy<R: Runtime>(
    window: Window<R>,
    effect: VibrancyEffect,
) -> Result<(), String> {
    // TODO: Implement using Tauri window effects API
    //
    // Tauri v2 has built-in support for window effects:
    // window.set_effects(WindowEffects {
    //     effects: vec![Effect::Acrylic],
    //     state: Some(EffectState::Active),
    //     ..Default::default()
    // }).map_err(|e| e.to_string())?;

    tracing::info!("🎨 Set window vibrancy: {:?}", effect);
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
                                    sysinfo / 1024.0 / 1024.0 / 1024.0  // Convert to GB
                                } else {
                                    0.0
                                }
                            } else {
                                0.0
                            }
                        };

                        // Check for Metal 3 support (macOS 13+ with Apple Silicon or AMD GPUs)
                        let supports_metal_3 = chipset_model.contains("Apple M")
                            || chipset_model.contains("AMD");

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
    mem_str.trim()
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
            .map(|out| {
                String::from_utf8_lossy(&out.stdout)
                    .trim()
                    .to_string()
            })
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
