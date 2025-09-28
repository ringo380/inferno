//! # Application Upgrade System
//!
//! Provides seamless application upgrades through the user interface with automatic
//! backups, rollback capabilities, and platform-specific installation handling.
//!
//! ## Features
//!
//! - **Automatic Update Checking**: Background service to check for new versions
//! - **Secure Downloads**: Cryptographic verification of update packages
//! - **Platform Integration**: Native upgrade mechanisms for macOS, Linux, Windows
//! - **Zero-Downtime**: Rolling upgrades for API servers and services
//! - **Backup & Rollback**: Automatic backups with one-click rollback
//! - **Enterprise Features**: Centralized management and staged rollouts

pub mod checker;
pub mod downloader;
pub mod backup;
pub mod platform;
pub mod manager;
pub mod config;
pub mod safety;
pub mod background_service;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

pub use manager::UpgradeManager;
pub use config::UpgradeConfig;
pub use checker::UpdateChecker;
pub use config::UpdateSource;
pub use downloader::{UpdateDownloader, ProgressCallback};
pub use backup::{BackupManager, BackupMetadata, BackupType, BackupStorageStats};
pub use safety::{SafetyChecker, CompatibilityReport, ResourceReport};
pub use background_service::{BackgroundUpdateService, ServiceStatus, ServiceStatistics};

/// Current application version information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ApplicationVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
    pub build_metadata: Option<String>,
    pub build_date: Option<DateTime<Utc>>,
    pub git_commit: Option<String>,
}

impl ApplicationVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build_metadata: None,
            build_date: None,
            git_commit: None,
        }
    }

    pub fn current() -> Self {
        // This would be populated at build time using build.rs
        Self {
            major: 0,
            minor: 3,
            patch: 0,
            pre_release: None,
            build_metadata: None,
            build_date: Some(Utc::now()),
            git_commit: option_env!("GIT_COMMIT").map(String::from),
        }
    }

    pub fn to_string(&self) -> String {
        let mut version = format!("{}.{}.{}", self.major, self.minor, self.patch);

        if let Some(pre) = &self.pre_release {
            version.push_str(&format!("-{}", pre));
        }

        if let Some(build) = &self.build_metadata {
            version.push_str(&format!("+{}", build));
        }

        version
    }

    pub fn is_newer_than(&self, other: &Self) -> bool {
        self > other
    }

    pub fn is_compatible_with(&self, other: &Self) -> bool {
        // Major version must match for compatibility
        self.major == other.major
    }
}

/// Information about an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: ApplicationVersion,
    pub release_date: DateTime<Utc>,
    pub changelog: String,
    pub download_urls: HashMap<String, String>, // platform -> URL
    pub checksums: HashMap<String, String>,     // platform -> checksum
    pub signatures: HashMap<String, String>,    // platform -> signature
    pub size_bytes: HashMap<String, u64>,       // platform -> size
    pub is_critical: bool,
    pub is_security_update: bool,
    pub minimum_version: Option<ApplicationVersion>,
    pub deprecation_warnings: Vec<String>,
}

/// Current upgrade status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpgradeStatus {
    /// No updates available
    UpToDate,
    /// Update is available
    Available(UpdateInfo),
    /// Currently checking for updates
    Checking,
    /// Download in progress
    Downloading {
        progress: f32,
        bytes_downloaded: u64,
        total_bytes: u64,
        speed_bytes_per_sec: u64,
    },
    /// Installing update
    Installing {
        stage: InstallationStage,
        progress: f32,
    },
    /// Installation completed successfully
    Completed {
        old_version: ApplicationVersion,
        new_version: ApplicationVersion,
        restart_required: bool,
    },
    /// Installation failed
    Failed {
        error: String,
        recovery_available: bool,
    },
    /// Rollback in progress
    RollingBack {
        target_version: ApplicationVersion,
        progress: f32,
    },
}

/// Installation stages for progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallationStage {
    PreparingBackup,
    CreatingBackup,
    VerifyingUpdate,
    StoppingServices,
    InstallingFiles,
    UpdatingConfiguration,
    StartingServices,
    VerifyingInstallation,
    CleaningUp,
}

impl InstallationStage {
    pub fn description(&self) -> &'static str {
        match self {
            Self::PreparingBackup => "Preparing backup",
            Self::CreatingBackup => "Creating backup",
            Self::VerifyingUpdate => "Verifying update package",
            Self::StoppingServices => "Stopping services",
            Self::InstallingFiles => "Installing files",
            Self::UpdatingConfiguration => "Updating configuration",
            Self::StartingServices => "Starting services",
            Self::VerifyingInstallation => "Verifying installation",
            Self::CleaningUp => "Cleaning up",
        }
    }
}

/// Upgrade event for notifications and logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: UpgradeEventType,
    pub version: Option<ApplicationVersion>,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpgradeEventType {
    UpdateCheckStarted,
    UpdateCheckCompleted,
    UpdateCheckFailed,
    UpdateAvailable,
    DownloadStarted,
    DownloadProgress,
    DownloadCompleted,
    DownloadFailed,
    InstallationStarted,
    InstallationProgress,
    InstallationCompleted,
    InstallationFailed,
    RollbackStarted,
    RollbackCompleted,
    RollbackFailed,
    ConfigurationUpdated,
}

/// Update channel for receiving different types of releases
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateChannel {
    Stable,
    Beta,
    Nightly,
    Custom(String),
}

impl Default for UpdateChannel {
    fn default() -> Self {
        Self::Stable
    }
}

impl UpdateChannel {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
            Self::Nightly => "nightly",
            Self::Custom(name) => name,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "stable" => Self::Stable,
            "beta" => Self::Beta,
            "nightly" => Self::Nightly,
            custom => Self::Custom(custom.to_string()),
        }
    }
}

/// Platform-specific upgrade handler trait
#[async_trait::async_trait]
pub trait PlatformUpgradeHandler: Send + Sync {
    /// Check if the current platform supports seamless upgrades
    fn supports_seamless_upgrade(&self) -> bool;

    /// Prepare the system for upgrade (stop services, etc.)
    async fn prepare_for_upgrade(&self) -> Result<()>;

    /// Install the downloaded update package
    async fn install_update(&self, package_path: &PathBuf) -> Result<()>;

    /// Restart the application after upgrade
    async fn restart_application(&self) -> Result<()>;

    /// Verify the installation was successful
    async fn verify_installation(&self) -> Result<bool>;

    /// Clean up temporary files and old versions
    async fn cleanup_after_upgrade(&self) -> Result<()>;

    /// Check if administrator/root privileges are required
    fn requires_elevated_privileges(&self) -> bool;

    /// Get platform-specific installation directory
    fn get_installation_directory(&self) -> PathBuf;

    /// Get platform-specific backup directory
    fn get_backup_directory(&self) -> PathBuf;
}

/// Error types for upgrade operations
#[derive(Debug, thiserror::Error)]
pub enum UpgradeError {
    #[error("Network error during update check: {0}")]
    NetworkError(String),

    #[error("Invalid update package: {0}")]
    InvalidPackage(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Insufficient disk space: required {required} MB, available {available} MB")]
    InsufficientDiskSpace { required: u64, available: u64 },

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Backup failed: {0}")]
    BackupFailed(String),

    #[error("Installation failed: {0}")]
    InstallationFailed(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Upgrade cancelled by user")]
    Cancelled,

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type UpgradeResult<T> = std::result::Result<T, UpgradeError>;

/// Initialize the upgrade system
pub async fn init_upgrade_system(config: &crate::config::Config) -> Result<UpgradeManager> {
    info!("Initializing upgrade system");

    let upgrade_config = UpgradeConfig::from_config(config)?;
    let manager = UpgradeManager::new(upgrade_config).await?;

    info!("Upgrade system initialized successfully");
    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_version_comparison() {
        let v1 = ApplicationVersion::new(1, 0, 0);
        let v2 = ApplicationVersion::new(1, 0, 1);
        let v3 = ApplicationVersion::new(2, 0, 0);

        assert!(v2.is_newer_than(&v1));
        assert!(v3.is_newer_than(&v2));
        assert!(!v1.is_newer_than(&v2));

        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
    }

    #[test]
    fn test_version_string_formatting() {
        let mut version = ApplicationVersion::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");

        version.pre_release = Some("beta.1".to_string());
        assert_eq!(version.to_string(), "1.2.3-beta.1");

        version.build_metadata = Some("20231201".to_string());
        assert_eq!(version.to_string(), "1.2.3-beta.1+20231201");
    }

    #[test]
    fn test_update_channel_conversion() {
        assert_eq!(UpdateChannel::Stable.as_str(), "stable");
        assert_eq!(UpdateChannel::from_str("beta"), UpdateChannel::Beta);
        assert_eq!(UpdateChannel::from_str("custom"), UpdateChannel::Custom("custom".to_string()));
    }
}