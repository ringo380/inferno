//! # Configuration Presets
//!
//! Predefined configuration profiles for common use cases.

use super::core::CoreConfig;
use super::types::{LogFormat, LogLevel};

/// Configuration presets for different deployment scenarios
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Preset {
    /// Development preset: verbose logging, minimal features, fast startup
    ///
    /// - Debug logging
    /// - Pretty log format
    /// - Cache disabled
    /// - Monitoring disabled
    /// - Faster compilation and startup
    #[default]
    Development,

    /// Production preset: optimized for deployment
    ///
    /// - Info logging
    /// - JSON log format (for log aggregation)
    /// - Cache enabled with production limits
    /// - Monitoring and metrics enabled
    /// - Security features enabled
    Production,

    /// Testing preset: deterministic behavior, minimal output
    ///
    /// - Error-only logging
    /// - Compact format
    /// - All optional features disabled
    /// - Deterministic random seeds
    /// - Fast execution
    Testing,

    /// Benchmark preset: optimized for performance measurement
    ///
    /// - Minimal logging (warn only)
    /// - All monitoring disabled
    /// - Maximum performance settings
    /// - Consistent behavior across runs
    Benchmark,
}

impl Preset {
    /// Get a human-readable description of this preset
    pub fn description(&self) -> &'static str {
        match self {
            Self::Development => "Development: Verbose logging, minimal features, fast startup",
            Self::Production => "Production: Optimized settings, full monitoring, security enabled",
            Self::Testing => {
                "Testing: Minimal logging, deterministic behavior, all features disabled"
            }
            Self::Benchmark => {
                "Benchmark: Maximum performance, monitoring disabled, consistent behavior"
            }
        }
    }

    /// Apply this preset to a CoreConfig
    pub fn apply_to_core(&self, mut core: CoreConfig) -> CoreConfig {
        match self {
            Self::Development => {
                core.log_level = LogLevel::Debug;
                core.log_format = LogFormat::Pretty;
                core
            }
            Self::Production => {
                core.log_level = LogLevel::Info;
                core.log_format = LogFormat::Json;
                core
            }
            Self::Testing => {
                core.log_level = LogLevel::Error;
                core.log_format = LogFormat::Compact;
                core
            }
            Self::Benchmark => {
                core.log_level = LogLevel::Warn;
                core.log_format = LogFormat::Compact;
                core
            }
        }
    }

    /// Check if caching should be enabled for this preset
    pub fn cache_enabled(&self) -> bool {
        matches!(self, Self::Production)
    }

    /// Check if monitoring should be enabled for this preset
    pub fn monitoring_enabled(&self) -> bool {
        matches!(self, Self::Production)
    }

    /// Check if metrics collection should be enabled for this preset
    pub fn metrics_enabled(&self) -> bool {
        matches!(self, Self::Production)
    }

    /// Check if audit logging should be enabled for this preset
    pub fn audit_enabled(&self) -> bool {
        matches!(self, Self::Production)
    }

    /// Get recommended max concurrent requests for this preset
    pub fn max_concurrent_requests(&self) -> u32 {
        match self {
            Self::Development => 5,
            Self::Production => 100,
            Self::Testing => 1,
            Self::Benchmark => 1,
        }
    }

    /// Get recommended request timeout for this preset
    pub fn request_timeout_seconds(&self) -> u64 {
        match self {
            Self::Development => 600, // 10 minutes for debugging
            Self::Production => 300,  // 5 minutes
            Self::Testing => 10,      // 10 seconds
            Self::Benchmark => 3600,  // 1 hour for long benchmarks
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_descriptions() {
        assert!(!Preset::Development.description().is_empty());
        assert!(!Preset::Production.description().is_empty());
        assert!(!Preset::Testing.description().is_empty());
        assert!(!Preset::Benchmark.description().is_empty());
    }

    #[test]
    fn test_apply_to_core() {
        let core = CoreConfig::default();

        let dev = Preset::Development.apply_to_core(core.clone());
        assert_eq!(dev.log_level, LogLevel::Debug);
        assert_eq!(dev.log_format, LogFormat::Pretty);

        let prod = Preset::Production.apply_to_core(core.clone());
        assert_eq!(prod.log_level, LogLevel::Info);
        assert_eq!(prod.log_format, LogFormat::Json);
    }

    #[test]
    fn test_feature_flags() {
        assert!(!Preset::Development.cache_enabled());
        assert!(Preset::Production.cache_enabled());
        assert!(!Preset::Testing.cache_enabled());
        assert!(!Preset::Benchmark.cache_enabled());

        assert!(!Preset::Development.monitoring_enabled());
        assert!(Preset::Production.monitoring_enabled());
        assert!(!Preset::Testing.monitoring_enabled());
        assert!(!Preset::Benchmark.monitoring_enabled());
    }

    #[test]
    fn test_resource_limits() {
        assert_eq!(Preset::Development.max_concurrent_requests(), 5);
        assert_eq!(Preset::Production.max_concurrent_requests(), 100);
        assert_eq!(Preset::Testing.max_concurrent_requests(), 1);

        assert_eq!(Preset::Development.request_timeout_seconds(), 600);
        assert_eq!(Preset::Production.request_timeout_seconds(), 300);
    }

    #[test]
    fn test_default_preset() {
        assert_eq!(Preset::default(), Preset::Development);
    }
}
