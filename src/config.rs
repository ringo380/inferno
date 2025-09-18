use crate::{backends::BackendConfig, distributed::DistributedConfig, cache::CacheConfig, response_cache::ResponseCacheConfig, monitoring::MonitoringConfig, /* ab_testing_config::ABTestingConfig, */ observability::ObservabilityConfig, marketplace::MarketplaceConfig, deployment::DeploymentConfig, federated::FederatedConfig, dashboard::DashboardConfig, advanced_monitoring::AdvancedMonitoringConfig, api_gateway::ApiGatewayConfig, model_versioning::ModelVersioningConfig, data_pipeline::DataPipelineConfig, backup_recovery::BackupRecoveryConfig, logging_audit::LoggingAuditConfig, performance_optimization::PerformanceOptimizationConfig, multi_tenancy::MultiTenancyConfig, advanced_cache::AdvancedCacheConfig};
use anyhow::Result;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub models_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_level: String,
    pub log_format: String,
    pub backend_config: BackendConfig,
    pub server: ServerConfig,
    pub model_security: Option<ModelSecurityConfig>,
    pub auth_security: Option<crate::security::SecurityConfig>,
    pub metrics: MetricsConfig,
    pub distributed: DistributedConfig,
    pub cache: CacheConfig,
    pub response_cache: ResponseCacheConfig,
    pub monitoring: MonitoringConfig,
    // pub ab_testing: ABTestingConfig,
    pub observability: ObservabilityConfig,
    pub marketplace: MarketplaceConfig,
    pub deployment: DeploymentConfig,
    pub federated: FederatedConfig,
    pub dashboard: DashboardConfig,
    pub advanced_monitoring: AdvancedMonitoringConfig,
    pub api_gateway: ApiGatewayConfig,
    pub model_versioning: ModelVersioningConfig,
    pub data_pipeline: DataPipelineConfig,
    pub backup_recovery: BackupRecoveryConfig,
    pub logging_audit: LoggingAuditConfig,
    pub performance_optimization: PerformanceOptimizationConfig,
    pub multi_tenancy: MultiTenancyConfig,
    pub advanced_cache: AdvancedCacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub max_concurrent_requests: u32,
    pub request_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSecurityConfig {
    pub verify_checksums: bool,
    pub allowed_model_extensions: Vec<String>,
    pub max_model_size_gb: f64,
    pub sandbox_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub bind_address: String,
    pub port: u16,
    pub path: String,
    pub collection_interval_seconds: u64,
    pub retention_hours: u64,
    pub export_system_metrics: bool,
    pub export_model_metrics: bool,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("inferno");

        Self {
            models_dir: data_dir.join("models"),
            cache_dir: data_dir.join("cache"),
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            backend_config: BackendConfig::default(),
            server: ServerConfig::default(),
            model_security: Some(ModelSecurityConfig::default()),
            auth_security: None,
            metrics: MetricsConfig::default(),
            distributed: DistributedConfig::default(),
            cache: CacheConfig::default(),
            response_cache: ResponseCacheConfig::default(),
            monitoring: MonitoringConfig::default(),
            // ab_testing: ABTestingConfig::default(),
            observability: ObservabilityConfig::default(),
            marketplace: MarketplaceConfig::default(),
            deployment: DeploymentConfig::default(),
            federated: FederatedConfig::default(),
            dashboard: DashboardConfig::default(),
            advanced_monitoring: AdvancedMonitoringConfig::default(),
            api_gateway: ApiGatewayConfig::default(),
            model_versioning: ModelVersioningConfig::default(),
            data_pipeline: DataPipelineConfig::default(),
            backup_recovery: BackupRecoveryConfig::default(),
            logging_audit: LoggingAuditConfig::default(),
            performance_optimization: PerformanceOptimizationConfig::default(),
            multi_tenancy: MultiTenancyConfig::default(),
            advanced_cache: AdvancedCacheConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            max_concurrent_requests: 10,
            request_timeout_seconds: 300,
        }
    }
}

impl Default for ModelSecurityConfig {
    fn default() -> Self {
        Self {
            verify_checksums: true,
            allowed_model_extensions: vec!["gguf".to_string(), "onnx".to_string()],
            max_model_size_gb: 50.0,
            sandbox_enabled: true,
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind_address: "127.0.0.1".to_string(),
            port: 9090,
            path: "/metrics".to_string(),
            collection_interval_seconds: 10,
            retention_hours: 24,
            export_system_metrics: true,
            export_model_metrics: true,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_paths = Self::get_config_paths();

        // Start with default configuration
        let default_config = Self::default();
        let mut figment = Figment::from(figment::providers::Serialized::defaults(default_config));

        // Load configuration files in order of precedence (lowest to highest)
        for config_path in &config_paths {
            if config_path.exists() {
                info!("Loading config from: {}", config_path.display());
                figment = figment.merge(Toml::file(config_path));
            }
        }

        // Environment variables override config files
        figment = figment.merge(Env::prefixed("INFERNO_"));

        let config: Config = figment.extract()?;

        // Ensure directories exist
        config.ensure_directories()?;

        Ok(config)
    }

    pub fn save(&self, path: Option<&Path>) -> Result<()> {
        let config_path = if let Some(p) = path {
            p.to_path_buf()
        } else {
            Self::get_default_config_path()
        };

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let toml_string = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, toml_string)?;

        info!("Configuration saved to: {}", config_path.display());
        Ok(())
    }

    fn get_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Global config
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("inferno").join("config.toml"));
        }

        // User config in home directory
        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join(".inferno.toml"));
        }

        // Local project config
        paths.push(PathBuf::from(".inferno.toml"));

        paths
    }

    pub fn get_default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("inferno")
            .join("config.toml")
    }

    fn ensure_directories(&self) -> Result<()> {
        std::fs::create_dir_all(&self.models_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;

        // Create logs directory if using file logging
        if let Some(cache_dir) = self.cache_dir.parent() {
            std::fs::create_dir_all(cache_dir.join("logs"))?;
        }

        Ok(())
    }

    pub fn get_model_path(&self, model_name: &str) -> PathBuf {
        self.models_dir.join(model_name)
    }

    pub fn get_cache_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.cache", key))
    }

    pub fn is_model_extension_allowed(&self, extension: &str) -> bool {
        if let Some(ref sec_config) = self.model_security {
            sec_config.allowed_model_extensions
                .iter()
                .any(|ext| ext.eq_ignore_ascii_case(extension))
        } else {
            // Default to allowing common extensions if security not configured
            matches!(extension.to_lowercase().as_str(), "gguf" | "onnx")
        }
    }

    pub fn is_model_size_allowed(&self, size_bytes: u64) -> bool {
        if let Some(ref sec_config) = self.model_security {
            let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            size_gb <= sec_config.max_model_size_gb
        } else {
            // Default to 5GB if security not configured
            let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            size_gb <= 5.0
        }
    }

    pub fn validate(&self) -> Result<()> {
        // Validate models directory
        if !self.models_dir.exists() {
            return Err(anyhow::anyhow!(
                "Models directory does not exist: {}",
                self.models_dir.display()
            ));
        }

        // Validate cache directory
        if !self.cache_dir.exists() {
            return Err(anyhow::anyhow!(
                "Cache directory does not exist: {}",
                self.cache_dir.display()
            ));
        }

        // Validate log level
        match self.log_level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid log level: {}. Must be one of: trace, debug, info, warn, error",
                    self.log_level
                ));
            }
        }

        // Validate log format
        match self.log_format.to_lowercase().as_str() {
            "pretty" | "compact" | "json" => {}
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid log format: {}. Must be one of: pretty, compact, json",
                    self.log_format
                ));
            }
        }

        // Validate server config
        if self.server.port == 0 {
            return Err(anyhow::anyhow!("Server port cannot be 0"));
        }

        if self.server.max_concurrent_requests == 0 {
            return Err(anyhow::anyhow!("Max concurrent requests must be greater than 0"));
        }

        // Validate model security config if present
        if let Some(ref sec_config) = self.model_security {
            if sec_config.max_model_size_gb == 0.0 {
                return Err(anyhow::anyhow!("Max model size must be greater than 0"));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.log_level, "info");
        assert_eq!(config.log_format, "pretty");
        assert!(config.model_security.is_some());
    }

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        assert!(config.validate().is_err()); // Directories don't exist

        // Create a config with valid directories
        let temp_dir = tempdir().unwrap();
        let mut config = Config::default();
        config.models_dir = temp_dir.path().join("models");
        config.cache_dir = temp_dir.path().join("cache");
        std::fs::create_dir_all(&config.models_dir).unwrap();
        std::fs::create_dir_all(&config.cache_dir).unwrap();

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_model_extension_validation() {
        let config = Config::default();
        assert!(config.is_model_extension_allowed("gguf"));
        assert!(config.is_model_extension_allowed("ONNX")); // Case insensitive
        assert!(!config.is_model_extension_allowed("bin"));
    }

    #[test]
    fn test_model_size_validation() {
        let mut config = Config::default();
        if let Some(ref mut security) = config.model_security {
            security.max_model_size_gb = 1.0; // 1 GB limit
        }

        let one_mb = 1024 * 1024;
        assert!(config.is_model_size_allowed(one_mb)); // 1 MB - OK
        assert!(config.is_model_size_allowed(one_mb * 500)); // 500 MB - OK
        assert!(!config.is_model_size_allowed(one_mb * 2000)); // 2 GB - Too large
    }
}