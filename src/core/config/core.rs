//! # Core Configuration
//!
//! Fundamental configuration settings required by all parts of the platform.

use super::types::{LogFormat, LogLevel};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Core configuration settings for the Inferno platform
///
/// This contains only the essential configuration needed for basic operation.
/// Feature-specific configuration is organized into separate modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// Directory where model files are stored
    pub models_dir: PathBuf,

    /// Directory for cache files and temporary data
    pub cache_dir: PathBuf,

    /// Logging level (trace, debug, info, warn, error)
    pub log_level: LogLevel,

    /// Log output format (pretty, compact, json)
    pub log_format: LogFormat,
}

impl Default for CoreConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("inferno");

        Self {
            models_dir: data_dir.join("models"),
            cache_dir: data_dir.join("cache"),
            log_level: LogLevel::default(),
            log_format: LogFormat::default(),
        }
    }
}

impl CoreConfig {
    /// Create a new CoreConfig with specified models directory
    pub fn with_models_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.models_dir = dir.into();
        self
    }

    /// Set the cache directory
    pub fn with_cache_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cache_dir = dir.into();
        self
    }

    /// Set the log level
    pub fn with_log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }

    /// Set the log format
    pub fn with_log_format(mut self, format: LogFormat) -> Self {
        self.log_format = format;
        self
    }

    /// Validate the core configuration
    pub fn validate(&self) -> Result<()> {
        // Models directory validation
        if !self.models_dir.is_absolute() {
            // Allow relative paths, but warn
            tracing::warn!(
                "Models directory is not absolute: {}",
                self.models_dir.display()
            );
        }

        // Cache directory validation
        if !self.cache_dir.is_absolute() {
            tracing::warn!(
                "Cache directory is not absolute: {}",
                self.cache_dir.display()
            );
        }

        Ok(())
    }

    /// Ensure required directories exist
    pub fn ensure_directories(&self) -> Result<()> {
        std::fs::create_dir_all(&self.models_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;

        // Create logs directory
        if let Some(parent) = self.cache_dir.parent() {
            std::fs::create_dir_all(parent.join("logs"))?;
        }

        Ok(())
    }

    /// Get the full path for a model file
    pub fn model_path(&self, model_name: &str) -> PathBuf {
        self.models_dir.join(model_name)
    }

    /// Get the full path for a cache file
    pub fn cache_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.cache", key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_core_config() {
        let config = CoreConfig::default();
        assert_eq!(config.log_level, LogLevel::Info);
        assert_eq!(config.log_format, LogFormat::Pretty);
        assert!(config.models_dir.to_string_lossy().contains("inferno"));
    }

    #[test]
    fn test_with_methods() {
        let config = CoreConfig::default()
            .with_models_dir("/custom/models")
            .with_log_level(LogLevel::Debug)
            .with_log_format(LogFormat::Json);

        assert_eq!(config.models_dir, PathBuf::from("/custom/models"));
        assert_eq!(config.log_level, LogLevel::Debug);
        assert_eq!(config.log_format, LogFormat::Json);
    }

    #[test]
    fn test_ensure_directories() {
        let temp_dir = tempdir().unwrap();
        let config = CoreConfig::default()
            .with_models_dir(temp_dir.path().join("models"))
            .with_cache_dir(temp_dir.path().join("cache"));

        assert!(config.ensure_directories().is_ok());
        assert!(config.models_dir.exists());
        assert!(config.cache_dir.exists());
    }

    #[test]
    fn test_path_helpers() {
        let config = CoreConfig::default();
        let model_path = config.model_path("test-model.gguf");
        let cache_path = config.cache_path("test-key");

        assert!(model_path.to_string_lossy().ends_with("test-model.gguf"));
        assert!(cache_path.to_string_lossy().ends_with("test-key.cache"));
    }
}
