//! # Configuration Builder
//!
//! Fluent API for constructing configuration with type safety and validation.

use super::core::CoreConfig;
use super::presets::Preset;
use super::types::{LogFormat, LogLevel};
use anyhow::Result;
use std::path::PathBuf;

/// Builder for constructing Inferno configuration
///
/// This builder provides a fluent API for creating configuration with compile-time
/// type safety and runtime validation. It supports starting from presets and
/// customizing individual settings.
///
/// # Examples
///
/// ```no_run
/// use inferno::core::config::{ConfigBuilder, Preset};
///
/// // Simple development config
/// let config = ConfigBuilder::new()
///     .models_dir("./models")
///     .build()?;
///
/// // Production config with customization
/// let config = ConfigBuilder::new()
///     .preset(Preset::Production)
///     .models_dir("./production-models")
///     .log_level(LogLevel::Warn)
///     .build()?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub struct ConfigBuilder {
    preset: Option<Preset>,
    core: CoreConfigBuilder,
}

/// Builder for core configuration
struct CoreConfigBuilder {
    models_dir: Option<PathBuf>,
    cache_dir: Option<PathBuf>,
    log_level: Option<LogLevel>,
    log_format: Option<LogFormat>,
}

impl ConfigBuilder {
    /// Create a new configuration builder with defaults
    pub fn new() -> Self {
        Self {
            preset: None,
            core: CoreConfigBuilder {
                models_dir: None,
                cache_dir: None,
                log_level: None,
                log_format: None,
            },
        }
    }

    /// Start from a preset configuration
    ///
    /// Presets provide sensible defaults for common scenarios:
    /// - `Development`: Fast startup, verbose logging
    /// - `Production`: Optimized for deployment
    /// - `Testing`: Minimal, deterministic behavior
    /// - `Benchmark`: Maximum performance
    ///
    /// Settings can be further customized after applying a preset.
    pub fn preset(mut self, preset: Preset) -> Self {
        self.preset = Some(preset);
        self
    }

    /// Set the models directory
    pub fn models_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.core.models_dir = Some(dir.into());
        self
    }

    /// Set the cache directory
    pub fn cache_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.core.cache_dir = Some(dir.into());
        self
    }

    /// Set the log level
    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.core.log_level = Some(level);
        self
    }

    /// Set the log format
    pub fn log_format(mut self, format: LogFormat) -> Self {
        self.core.log_format = Some(format);
        self
    }

    /// Build the configuration, applying validation
    ///
    /// This method:
    /// 1. Applies the preset (if specified)
    /// 2. Applies custom settings
    /// 3. Validates the configuration
    /// 4. Ensures required directories exist
    ///
    /// Returns an error if validation fails.
    pub fn build(self) -> Result<CoreConfig> {
        // Start with default or preset
        let mut config = if let Some(preset) = self.preset {
            preset.apply_to_core(CoreConfig::default())
        } else {
            CoreConfig::default()
        };

        // Apply custom settings (these override preset)
        if let Some(models_dir) = self.core.models_dir {
            config.models_dir = models_dir;
        }
        if let Some(cache_dir) = self.core.cache_dir {
            config.cache_dir = cache_dir;
        }
        if let Some(log_level) = self.core.log_level {
            config.log_level = log_level;
        }
        if let Some(log_format) = self.core.log_format {
            config.log_format = log_format;
        }

        // Validate
        config.validate()?;

        // Ensure directories exist
        config.ensure_directories()?;

        Ok(config)
    }

    /// Build without validation (useful for testing)
    ///
    /// **Warning**: This skips validation and directory creation.
    /// Only use this for testing or when you're sure the configuration is valid.
    pub fn build_unchecked(self) -> CoreConfig {
        let mut config = if let Some(preset) = self.preset {
            preset.apply_to_core(CoreConfig::default())
        } else {
            CoreConfig::default()
        };

        if let Some(models_dir) = self.core.models_dir {
            config.models_dir = models_dir;
        }
        if let Some(cache_dir) = self.core.cache_dir {
            config.cache_dir = cache_dir;
        }
        if let Some(log_level) = self.core.log_level {
            config.log_level = log_level;
        }
        if let Some(log_format) = self.core.log_format {
            config.log_format = log_format;
        }

        config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_builder() {
        let config = ConfigBuilder::new().build_unchecked();

        assert_eq!(config.log_level, LogLevel::Info);
        assert_eq!(config.log_format, LogFormat::Pretty);
    }

    #[test]
    fn test_builder_with_preset() {
        let config = ConfigBuilder::new()
            .preset(Preset::Production)
            .build_unchecked();

        assert_eq!(config.log_level, LogLevel::Info);
        assert_eq!(config.log_format, LogFormat::Json);
    }

    #[test]
    fn test_builder_override_preset() {
        let config = ConfigBuilder::new()
            .preset(Preset::Production)
            .log_level(LogLevel::Debug)
            .build_unchecked();

        // Custom setting overrides preset
        assert_eq!(config.log_level, LogLevel::Debug);
        // But other preset settings remain
        assert_eq!(config.log_format, LogFormat::Json);
    }

    #[test]
    fn test_builder_custom_paths() {
        let temp_dir = tempdir().unwrap();
        let models_path = temp_dir.path().join("models");
        let cache_path = temp_dir.path().join("cache");

        let config = ConfigBuilder::new()
            .models_dir(&models_path)
            .cache_dir(&cache_path)
            .build()
            .unwrap();

        assert_eq!(config.models_dir, models_path);
        assert_eq!(config.cache_dir, cache_path);
        assert!(models_path.exists());
        assert!(cache_path.exists());
    }

    #[test]
    fn test_builder_fluent_api() {
        let config = ConfigBuilder::new()
            .preset(Preset::Development)
            .models_dir("./test-models")
            .log_level(LogLevel::Trace)
            .log_format(LogFormat::Compact)
            .build_unchecked();

        assert_eq!(config.log_level, LogLevel::Trace);
        assert_eq!(config.log_format, LogFormat::Compact);
    }
}
