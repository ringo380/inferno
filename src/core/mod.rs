//! # Core Platform Functionality
//!
//! This module contains the core functionality of the Inferno platform including:
//! - Configuration system (new builder pattern + presets)
//! - Model execution backends
//! - Model discovery and metadata
//! - I/O format handling
//! - Security and sandboxing
//!
//! The core module provides the foundational capabilities that other modules build upon.

// New configuration system (v0.4.0+)
pub mod config;

// Re-export from existing locations for now, will move files later
pub use crate::backends;
pub use crate::io;
pub use crate::models;
pub use crate::security;

// Keep old config available for backward compatibility
pub use crate::config as legacy_config;

/// Core error types for the Inferno platform
pub use crate::{InfernoError, Result};
