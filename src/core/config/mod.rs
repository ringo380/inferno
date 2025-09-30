//! # Configuration Module
//!
//! Modern configuration system for Inferno with builder pattern, presets, and validation.
//!
//! ## Quick Start
//!
//! ```no_run
//! use inferno::core::config::{ConfigBuilder, Preset};
//!
//! // Simple configuration
//! let config = ConfigBuilder::new()
//!     .models_dir("./models")
//!     .build()?;
//!
//! // Use a preset
//! let config = ConfigBuilder::new()
//!     .preset(Preset::Production)
//!     .build()?;
//!
//! // Customize a preset
//! let config = ConfigBuilder::new()
//!     .preset(Preset::Development)
//!     .log_level(LogLevel::Trace)
//!     .models_dir("./my-models")
//!     .build()?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Architecture
//!
//! The new configuration system is organized into logical groups:
//!
//! - **Core**: Essential settings (models_dir, logging, etc.)
//! - **Builder**: Fluent API for constructing configs
//! - **Presets**: Predefined configurations (Dev, Prod, Test, Benchmark)
//! - **Types**: Type-safe enums (LogLevel, LogFormat)
//!
//! ## Migration from Old System
//!
//! The old `Config` struct is still available for backward compatibility:
//!
//! ```no_run
//! use inferno::config::Config;
//!
//! // Old way (still works)
//! let config = Config::load()?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! New code should use `ConfigBuilder`:
//!
//! ```no_run
//! use inferno::core::config::ConfigBuilder;
//!
//! // New way (recommended)
//! let config = ConfigBuilder::new()
//!     .preset(Preset::Development)
//!     .build()?;
//! # Ok::<(), anyhow::Error>(())
//! ```

pub mod builder;
pub mod core;
pub mod presets;
pub mod types;

// Re-export commonly used types
pub use builder::ConfigBuilder;
pub use core::CoreConfig;
pub use presets::Preset;
pub use types::{LogFormat, LogLevel};

// For backward compatibility, also re-export from crate::config
// This will be handled in the main config.rs file
