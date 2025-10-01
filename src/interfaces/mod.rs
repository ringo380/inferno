//! # User Interfaces
//!
//! This module contains all user-facing interfaces:
//! - Command-line interface (CLI) - both old and new architectures
//! - HTTP API (OpenAI-compatible)
//! - Terminal user interface (TUI)
//! - Web dashboard
//! - Desktop application (Tauri v2) - **PRIMARY INTERFACE for macOS**
//!
//! Interface modules provide different ways to interact with the platform.

// New CLI command architecture (v0.4.0+)
pub mod cli;

// Desktop interface (Tauri v2) - NEW in v0.5.0
// Only compiled when desktop feature is enabled to avoid dependency conflicts
#[cfg(feature = "desktop")]
pub mod desktop;

// Legacy CLI commands (re-export from old locations for backward compatibility)
pub use crate::cli as legacy_cli;

// Re-export from existing locations for now
pub use crate::api;
pub use crate::dashboard;
pub use crate::tui;

// REMOVED: Tauri v1 implementation has been removed
// Desktop app is now in dashboard/src-tauri/ (Tauri v2)
