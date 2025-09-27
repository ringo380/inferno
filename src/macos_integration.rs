use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::{
    api::notification::Notification, command, AppHandle, CustomMenuItem, Manager, Menu, MenuItem,
    Submenu, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, Window,
};

#[derive(Serialize, Deserialize)]
pub struct MacOSNotification {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
}

// Create native macOS menu bar
pub fn create_system_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit Inferno");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide Window");
    let show = CustomMenuItem::new("show".to_string(), "Show Window");
    let dashboard = CustomMenuItem::new("dashboard".to_string(), "Open Dashboard");
    let models = CustomMenuItem::new("models".to_string(), "Manage Models");
    let inference = CustomMenuItem::new("inference".to_string(), "Quick Inference");

    let tray_menu = SystemTrayMenu::new()
        .add_item(dashboard)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(models)
        .add_item(inference)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(show)
        .add_item(hide)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    SystemTray::new().with_menu(tray_menu)
}

// Create native macOS application menu
pub fn create_app_menu() -> Menu {
    let app_submenu = Submenu::new(
        "Inferno",
        Menu::new()
            .add_native_item(MenuItem::About(
                "Inferno AI Runner".to_string(),
                tauri::AboutMetadata::default(),
            ))
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::Services)
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::Hide)
            .add_native_item(MenuItem::HideOthers)
            .add_native_item(MenuItem::ShowAll)
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::Quit),
    );

    let file_submenu = Submenu::new(
        "File",
        Menu::new()
            .add_item(CustomMenuItem::new("new_inference", "New Inference").accelerator("cmd+n"))
            .add_item(CustomMenuItem::new("open_model", "Open Model...").accelerator("cmd+o"))
            .add_native_item(MenuItem::Separator)
            .add_item(CustomMenuItem::new("import_model", "Import Model...").accelerator("cmd+i"))
            .add_item(
                CustomMenuItem::new("export_results", "Export Results...").accelerator("cmd+e"),
            )
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::CloseWindow),
    );

    let edit_submenu = Submenu::new(
        "Edit",
        Menu::new()
            .add_native_item(MenuItem::Undo)
            .add_native_item(MenuItem::Redo)
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::Cut)
            .add_native_item(MenuItem::Copy)
            .add_native_item(MenuItem::Paste)
            .add_native_item(MenuItem::SelectAll),
    );

    let models_submenu = Submenu::new(
        "Models",
        Menu::new()
            .add_item(CustomMenuItem::new("load_model", "Load Model").accelerator("cmd+l"))
            .add_item(
                CustomMenuItem::new("unload_all", "Unload All Models").accelerator("cmd+shift+u"),
            )
            .add_native_item(MenuItem::Separator)
            .add_item(
                CustomMenuItem::new("model_info", "Model Information").accelerator("cmd+shift+i"),
            )
            .add_item(CustomMenuItem::new("validate_models", "Validate Models")),
    );

    let inference_submenu = Submenu::new(
        "Inference",
        Menu::new()
            .add_item(
                CustomMenuItem::new("quick_inference", "Quick Inference").accelerator("cmd+r"),
            )
            .add_item(
                CustomMenuItem::new("batch_inference", "Batch Inference")
                    .accelerator("cmd+shift+r"),
            )
            .add_native_item(MenuItem::Separator)
            .add_item(
                CustomMenuItem::new("stop_inference", "Stop All Inference").accelerator("cmd+."),
            ),
    );

    let view_submenu = Submenu::new(
        "View",
        Menu::new()
            .add_item(CustomMenuItem::new("dashboard", "Dashboard").accelerator("cmd+1"))
            .add_item(CustomMenuItem::new("models_view", "Models").accelerator("cmd+2"))
            .add_item(CustomMenuItem::new("inference_view", "Inference").accelerator("cmd+3"))
            .add_item(CustomMenuItem::new("metrics_view", "Metrics").accelerator("cmd+4"))
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::EnterFullScreen),
    );

    let window_submenu = Submenu::new(
        "Window",
        Menu::new()
            .add_native_item(MenuItem::Minimize)
            .add_native_item(MenuItem::Zoom)
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::CloseWindow),
    );

    let help_submenu = Submenu::new(
        "Help",
        Menu::new()
            .add_item(CustomMenuItem::new("documentation", "Documentation"))
            .add_item(CustomMenuItem::new("shortcuts", "Keyboard Shortcuts"))
            .add_native_item(MenuItem::Separator)
            .add_item(CustomMenuItem::new("report_issue", "Report Issue"))
            .add_item(CustomMenuItem::new("check_updates", "Check for Updates")),
    );

    Menu::new()
        .add_submenu(app_submenu)
        .add_submenu(file_submenu)
        .add_submenu(edit_submenu)
        .add_submenu(models_submenu)
        .add_submenu(inference_submenu)
        .add_submenu(view_submenu)
        .add_submenu(window_submenu)
        .add_submenu(help_submenu)
}

// Handle system tray events
pub fn handle_system_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    let window = app.get_window("main").unwrap();

    match event {
        SystemTrayEvent::LeftClick {
            position: _,
            size: _,
            ..
        } => {
            if window.is_visible().unwrap() {
                let _ = window.hide();
            } else {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => {
                std::process::exit(0);
            }
            "hide" => {
                let _ = window.hide();
            }
            "show" => {
                let _ = window.show();
                let _ = window.set_focus();
            }
            "dashboard" => {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.emit("navigate", "/");
            }
            "models" => {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.emit("navigate", "/models");
            }
            "inference" => {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.emit("navigate", "/inference");
            }
            _ => {}
        },
        _ => {}
    }
}

// Handle menu events
pub fn handle_menu_event(window: &Window, event_id: &str) {
    match event_id {
        "new_inference" => {
            let _ = window.emit("menu_action", "new_inference");
        }
        "open_model" => {
            let _ = window.emit("menu_action", "open_model");
        }
        "import_model" => {
            let _ = window.emit("menu_action", "import_model");
        }
        "export_results" => {
            let _ = window.emit("menu_action", "export_results");
        }
        "load_model" => {
            let _ = window.emit("menu_action", "load_model");
        }
        "unload_all" => {
            let _ = window.emit("menu_action", "unload_all");
        }
        "model_info" => {
            let _ = window.emit("menu_action", "model_info");
        }
        "validate_models" => {
            let _ = window.emit("menu_action", "validate_models");
        }
        "quick_inference" => {
            let _ = window.emit("menu_action", "quick_inference");
        }
        "batch_inference" => {
            let _ = window.emit("menu_action", "batch_inference");
        }
        "stop_inference" => {
            let _ = window.emit("menu_action", "stop_inference");
        }
        "dashboard" => {
            let _ = window.emit("navigate", "/");
        }
        "models_view" => {
            let _ = window.emit("navigate", "/models");
        }
        "inference_view" => {
            let _ = window.emit("navigate", "/inference");
        }
        "metrics_view" => {
            let _ = window.emit("navigate", "/metrics");
        }
        "documentation" => {
            let _ = window.emit(
                "external_link",
                "https://github.com/inferno-ai/inferno/docs",
            );
        }
        "shortcuts" => {
            let _ = window.emit("menu_action", "show_shortcuts");
        }
        "report_issue" => {
            let _ = window.emit(
                "external_link",
                "https://github.com/inferno-ai/inferno/issues/new",
            );
        }
        "check_updates" => {
            let _ = window.emit("menu_action", "check_updates");
        }
        _ => {}
    }
}

// Tauri commands for native macOS features
#[command]
pub async fn send_native_notification(notification: MacOSNotification) -> Result<(), String> {
    Notification::new("com.inferno.ai.runner")
        .title(&notification.title)
        .body(&notification.body)
        .icon("icon.png")
        .show()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub async fn get_system_appearance() -> Result<String, String> {
    // On macOS, we can detect dark mode
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let output = Command::new("defaults")
            .args(&["read", "-g", "AppleInterfaceStyle"])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            let style = String::from_utf8_lossy(&output.stdout);
            if style.trim() == "Dark" {
                return Ok("dark".to_string());
            }
        }
        Ok("light".to_string())
    }

    #[cfg(not(target_os = "macos"))]
    Ok("dark".to_string()) // Default to dark mode
}

#[command]
pub async fn set_window_vibrancy(window: Window, vibrancy: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use tauri::Manager;
        // Enable window vibrancy on macOS
        let _ = window.set_transparent(true);
        // Additional vibrancy settings would need custom implementation
    }
    Ok(())
}

#[command]
pub async fn toggle_always_on_top(window: Window) -> Result<bool, String> {
    let is_on_top = window.is_always_on_top().map_err(|e| e.to_string())?;
    window
        .set_always_on_top(!is_on_top)
        .map_err(|e| e.to_string())?;
    Ok(!is_on_top)
}

#[command]
pub async fn minimize_to_tray(window: Window) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())?;
    Ok(())
}
