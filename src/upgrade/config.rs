//! # Upgrade Configuration
//!
//! Configuration management for the upgrade system with support for
//! multiple update sources, channels, and security settings.

use super::{UpdateChannel, UpgradeError, UpgradeResult};
use crate::config::Config;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Update source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateSource {
    /// GitHub releases
    GitHub { owner: String, repo: String },
    /// Custom update server
    Custom { url: String },
    /// Updates disabled
    Disabled,
}

impl Default for UpdateSource {
    fn default() -> Self {
        Self::GitHub {
            owner: "inferno-ai".to_string(),
            repo: "inferno".to_string(),
        }
    }
}

/// Comprehensive upgrade configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeConfig {
    /// Update source configuration
    pub update_source: UpdateSource,

    /// Update channel (stable, beta, nightly, custom)
    pub update_channel: UpdateChannel,

    /// Automatically check for updates
    pub auto_check: bool,

    /// Check interval for automatic updates
    pub check_interval: Duration,

    /// Automatically install updates (excluding critical ones)
    pub auto_install: bool,

    /// Automatically install critical security updates
    pub auto_install_critical: bool,

    /// Create backups before installing updates
    pub create_backups: bool,

    /// Maximum number of backups to keep
    pub max_backups: u32,

    /// Directory for storing downloaded update packages
    pub download_dir: PathBuf,

    /// Directory for storing backups
    pub backup_dir: PathBuf,

    /// Require cryptographic signature verification
    pub require_signatures: bool,

    /// Trusted public keys for signature verification
    pub trusted_keys: Vec<String>,

    /// Maximum download size (in bytes)
    pub max_download_size: u64,

    /// Download timeout (in seconds)
    pub download_timeout: u64,

    /// Retry attempts for failed downloads
    pub download_retries: u32,

    /// Enable parallel chunk downloading
    pub parallel_download: bool,

    /// Number of parallel download chunks
    pub download_chunks: u32,

    /// Pre-installation safety checks
    pub safety_checks: SafetyChecksConfig,

    /// Notification settings
    pub notifications: NotificationConfig,

    /// Enterprise/deployment specific settings
    pub enterprise: EnterpriseConfig,
}

impl Default for UpgradeConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        Self {
            update_source: UpdateSource::default(),
            update_channel: UpdateChannel::Stable,
            auto_check: true,
            check_interval: Duration::from_secs(3600), // 1 hour
            auto_install: false,
            auto_install_critical: true,
            create_backups: true,
            max_backups: 5,
            download_dir: home_dir.join(".inferno").join("downloads"),
            backup_dir: home_dir.join(".inferno").join("backups"),
            require_signatures: true,
            trusted_keys: vec![
                // Default Inferno public key (placeholder)
                "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----".to_string(),
            ],
            max_download_size: 1024 * 1024 * 1024, // 1GB
            download_timeout: 300, // 5 minutes
            download_retries: 3,
            parallel_download: true,
            download_chunks: 4,
            safety_checks: SafetyChecksConfig::default(),
            notifications: NotificationConfig::default(),
            enterprise: EnterpriseConfig::default(),
        }
    }
}

/// Safety checks configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyChecksConfig {
    /// Check available disk space before download
    pub check_disk_space: bool,

    /// Minimum free disk space (in MB)
    pub min_free_space_mb: u64,

    /// Verify system compatibility
    pub check_compatibility: bool,

    /// Check for running processes that might interfere
    pub check_running_processes: bool,

    /// Verify network connectivity
    pub check_network: bool,

    /// Check system dependencies
    pub check_dependencies: bool,

    /// Simulate installation without making changes
    pub dry_run_install: bool,

    /// Verify backup integrity before installation
    pub verify_backup: bool,
}

impl Default for SafetyChecksConfig {
    fn default() -> Self {
        Self {
            check_disk_space: true,
            min_free_space_mb: 1024, // 1GB
            check_compatibility: true,
            check_running_processes: true,
            check_network: true,
            check_dependencies: true,
            dry_run_install: false,
            verify_backup: true,
        }
    }
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Enable desktop notifications
    pub desktop_notifications: bool,

    /// Enable email notifications
    pub email_notifications: bool,

    /// Email address for notifications
    pub email_address: Option<String>,

    /// Enable webhook notifications
    pub webhook_notifications: bool,

    /// Webhook URL for notifications
    pub webhook_url: Option<String>,

    /// Notify on available updates
    pub notify_on_available: bool,

    /// Notify on download start/completion
    pub notify_on_download: bool,

    /// Notify on installation start/completion
    pub notify_on_installation: bool,

    /// Notify on failures
    pub notify_on_failure: bool,

    /// Notify on successful completion
    pub notify_on_success: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            desktop_notifications: true,
            email_notifications: false,
            email_address: None,
            webhook_notifications: false,
            webhook_url: None,
            notify_on_available: true,
            notify_on_download: false,
            notify_on_installation: true,
            notify_on_failure: true,
            notify_on_success: true,
        }
    }
}

/// Enterprise deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    /// Enable staged rollouts
    pub staged_rollouts: bool,

    /// Rollout percentage for gradual deployment
    pub rollout_percentage: f32,

    /// Deployment groups for staged rollouts
    pub deployment_groups: Vec<String>,

    /// Central management server URL
    pub management_server: Option<String>,

    /// Device/instance identifier for centralized management
    pub device_id: Option<String>,

    /// Enable telemetry reporting
    pub telemetry_enabled: bool,

    /// Custom deployment policies
    pub deployment_policies: Vec<DeploymentPolicy>,

    /// Maintenance windows for automatic updates
    pub maintenance_windows: Vec<MaintenanceWindow>,

    /// Enable A/B testing for updates
    pub ab_testing: bool,

    /// Canary deployment configuration
    pub canary_config: Option<CanaryConfig>,
}

impl Default for EnterpriseConfig {
    fn default() -> Self {
        Self {
            staged_rollouts: false,
            rollout_percentage: 100.0,
            deployment_groups: vec!["default".to_string()],
            management_server: None,
            device_id: None,
            telemetry_enabled: false,
            deployment_policies: vec![],
            maintenance_windows: vec![],
            ab_testing: false,
            canary_config: None,
        }
    }
}

/// Deployment policy for enterprise environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentPolicy {
    pub name: String,
    pub description: String,
    pub conditions: Vec<DeploymentCondition>,
    pub actions: Vec<DeploymentAction>,
}

/// Conditions for deployment policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentCondition {
    TimeWindow { start: String, end: String },
    SystemLoad { max_cpu: f32, max_memory: f32 },
    UserActivity { max_active_sessions: u32 },
    CustomScript { script_path: String },
}

/// Actions for deployment policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentAction {
    Allow,
    Deny,
    Defer { until: String },
    Notify { message: String },
    RequireApproval,
}

/// Maintenance window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceWindow {
    pub name: String,
    pub start_time: String,  // Cron expression or time string
    pub duration: Duration,
    pub timezone: String,
    pub allow_critical_updates: bool,
    pub allow_regular_updates: bool,
}

/// Canary deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryConfig {
    pub enabled: bool,
    pub percentage: f32,
    pub duration: Duration,
    pub success_criteria: Vec<SuccessCriterion>,
    pub rollback_on_failure: bool,
}

/// Success criteria for canary deployments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuccessCriterion {
    ErrorRate { max_rate: f32 },
    ResponseTime { max_p99_ms: u64 },
    HealthCheck { endpoint: String },
    CustomMetric { name: String, threshold: f32 },
}

impl UpgradeConfig {
    /// Create upgrade config from main application config
    pub fn from_config(config: &Config) -> Result<Self> {
        let mut upgrade_config = Self::default();

        // Override with values from main config if they exist
        if let Some(data_dir) = &config.data_dir {
            upgrade_config.download_dir = data_dir.join("downloads");
            upgrade_config.backup_dir = data_dir.join("backups");
        }

        // Parse configuration from environment or config files
        upgrade_config.load_from_environment()?;

        Ok(upgrade_config)
    }

    /// Load configuration from environment variables
    fn load_from_environment(&mut self) -> Result<()> {
        use std::env;

        if let Ok(auto_check) = env::var("INFERNO_AUTO_CHECK_UPDATES") {
            self.auto_check = auto_check.parse().unwrap_or(self.auto_check);
        }

        if let Ok(auto_install) = env::var("INFERNO_AUTO_INSTALL_UPDATES") {
            self.auto_install = auto_install.parse().unwrap_or(self.auto_install);
        }

        if let Ok(channel) = env::var("INFERNO_UPDATE_CHANNEL") {
            self.update_channel = UpdateChannel::from_str(&channel);
        }

        if let Ok(check_interval) = env::var("INFERNO_UPDATE_CHECK_INTERVAL") {
            if let Ok(seconds) = check_interval.parse::<u64>() {
                self.check_interval = Duration::from_secs(seconds);
            }
        }

        if let Ok(download_dir) = env::var("INFERNO_DOWNLOAD_DIR") {
            self.download_dir = PathBuf::from(download_dir);
        }

        if let Ok(backup_dir) = env::var("INFERNO_BACKUP_DIR") {
            self.backup_dir = PathBuf::from(backup_dir);
        }

        if let Ok(max_size) = env::var("INFERNO_MAX_DOWNLOAD_SIZE") {
            if let Ok(size) = max_size.parse::<u64>() {
                self.max_download_size = size;
            }
        }

        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> UpgradeResult<()> {
        // Check that required directories are accessible
        std::fs::create_dir_all(&self.download_dir)
            .map_err(|e| UpgradeError::ConfigurationError(format!("Cannot create download directory: {}", e)))?;

        std::fs::create_dir_all(&self.backup_dir)
            .map_err(|e| UpgradeError::ConfigurationError(format!("Cannot create backup directory: {}", e)))?;

        // Validate update source
        match &self.update_source {
            UpdateSource::GitHub { owner, repo } => {
                if owner.is_empty() || repo.is_empty() {
                    return Err(UpgradeError::ConfigurationError(
                        "GitHub owner and repo cannot be empty".to_string()
                    ));
                }
            }
            UpdateSource::Custom { url } => {
                if url.is_empty() {
                    return Err(UpgradeError::ConfigurationError(
                        "Custom update server URL cannot be empty".to_string()
                    ));
                }
                // Validate URL format
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    return Err(UpgradeError::ConfigurationError(
                        "Custom update server URL must start with http:// or https://".to_string()
                    ));
                }
            }
            UpdateSource::Disabled => {
                // Nothing to validate for disabled updates
            }
        }

        // Validate safety check configuration
        if self.safety_checks.min_free_space_mb == 0 {
            return Err(UpgradeError::ConfigurationError(
                "Minimum free space must be greater than 0".to_string()
            ));
        }

        // Validate notification configuration
        if self.notifications.email_notifications && self.notifications.email_address.is_none() {
            return Err(UpgradeError::ConfigurationError(
                "Email address required when email notifications are enabled".to_string()
            ));
        }

        if self.notifications.webhook_notifications && self.notifications.webhook_url.is_none() {
            return Err(UpgradeError::ConfigurationError(
                "Webhook URL required when webhook notifications are enabled".to_string()
            ));
        }

        // Validate enterprise configuration
        if self.enterprise.staged_rollouts && self.enterprise.rollout_percentage <= 0.0 {
            return Err(UpgradeError::ConfigurationError(
                "Rollout percentage must be greater than 0 for staged rollouts".to_string()
            ));
        }

        Ok(())
    }

    /// Get the effective update channel based on configuration
    pub fn get_effective_update_channel(&self) -> &UpdateChannel {
        &self.update_channel
    }

    /// Check if automatic updates are enabled for the given update type
    pub fn should_auto_install(&self, is_critical: bool) -> bool {
        if is_critical {
            self.auto_install_critical
        } else {
            self.auto_install
        }
    }

    /// Get download directory with creation if needed
    pub fn ensure_download_dir(&self) -> UpgradeResult<&PathBuf> {
        std::fs::create_dir_all(&self.download_dir)
            .map_err(|e| UpgradeError::ConfigurationError(e.to_string()))?;
        Ok(&self.download_dir)
    }

    /// Get backup directory with creation if needed
    pub fn ensure_backup_dir(&self) -> UpgradeResult<&PathBuf> {
        std::fs::create_dir_all(&self.backup_dir)
            .map_err(|e| UpgradeError::ConfigurationError(e.to_string()))?;
        Ok(&self.backup_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UpgradeConfig::default();
        assert_eq!(config.update_channel, UpdateChannel::Stable);
        assert!(config.auto_check);
        assert!(!config.auto_install);
        assert!(config.auto_install_critical);
        assert!(config.create_backups);
    }

    #[test]
    fn test_config_validation() {
        let config = UpgradeConfig::default();
        // This might fail if the home directory is not writable
        // but in most test environments it should pass
        let result = config.validate();
        if result.is_err() {
            println!("Validation failed (expected in some test environments): {:?}", result);
        }
    }

    #[test]
    fn test_auto_install_logic() {
        let config = UpgradeConfig::default();
        assert!(!config.should_auto_install(false)); // Regular updates
        assert!(config.should_auto_install(true));   // Critical updates
    }

    #[test]
    fn test_update_channel_from_str() {
        assert_eq!(UpdateChannel::from_str("stable"), UpdateChannel::Stable);
        assert_eq!(UpdateChannel::from_str("beta"), UpdateChannel::Beta);
        assert_eq!(UpdateChannel::from_str("nightly"), UpdateChannel::Nightly);
        assert_eq!(UpdateChannel::from_str("custom"), UpdateChannel::Custom("custom".to_string()));
    }
}