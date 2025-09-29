//! Desktop Interface Module
//!
//! This module provides the desktop application interface using Tauri v2.
//! It consolidates all desktop-specific functionality including:
//! - Tauri command handlers
//! - Application state management
//! - macOS-specific integrations (menu bar, system tray, notifications)
//! - Event emission system

pub mod commands;
pub mod events;
pub mod macos;
pub mod state;

// Re-export key types for convenience
pub use state::AppState;

/// Initialize the desktop application
///
/// This is the main entry point for the desktop application, called from
/// the binary in `dashboard/src-tauri/src/main.rs`.
pub fn init() {
    tracing::info!("🖥️  Initializing Inferno desktop interface");
}
