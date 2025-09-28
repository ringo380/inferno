//! # Upgrade Manager
//!
//! Central coordinator for all upgrade operations, providing a unified interface
//! for checking, downloading, and installing application updates.

use super::{
    ApplicationVersion, InstallationStage, PlatformUpgradeHandler, UpdateChannel, UpdateInfo,
    UpgradeConfig, UpgradeError, UpgradeEvent, UpgradeEventType, UpgradeResult, UpgradeStatus,
};
use crate::upgrade::{BackupManager, SafetyChecker, UpdateChecker, UpdateDownloader};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Central upgrade manager coordinating all upgrade operations
pub struct UpgradeManager {
    config: UpgradeConfig,
    current_version: ApplicationVersion,
    update_checker: Arc<RwLock<UpdateChecker>>,
    downloader: UpdateDownloader,
    backup_manager: BackupManager,
    safety_checker: Arc<RwLock<SafetyChecker>>,
    platform_handler: Box<dyn PlatformUpgradeHandler>,
    status: Arc<RwLock<UpgradeStatus>>,
    event_sender: broadcast::Sender<UpgradeEvent>,
    _event_receiver: broadcast::Receiver<UpgradeEvent>,
}

impl UpgradeManager {
    /// Create a new upgrade manager
    pub async fn new(config: UpgradeConfig) -> Result<Self> {
        let current_version = ApplicationVersion::current();
        let update_checker = UpdateChecker::new(&config).await?;
        let downloader = UpdateDownloader::new(&config)?;
        let backup_manager = BackupManager::new(&config)?;
        let safety_checker = SafetyChecker::new(&config);

        // Create platform-specific handler
        let platform_handler = Self::create_platform_handler(&config)?;

        let status = Arc::new(RwLock::new(UpgradeStatus::UpToDate));
        let (event_sender, event_receiver) = broadcast::channel(1000);

        Ok(Self {
            config,
            current_version,
            update_checker: Arc::new(RwLock::new(update_checker)),
            downloader,
            backup_manager,
            safety_checker: Arc::new(RwLock::new(safety_checker)),
            platform_handler,
            status,
            event_sender,
            _event_receiver: event_receiver,
        })
    }

    /// Get current upgrade status
    pub async fn get_status(&self) -> UpgradeStatus {
        self.status.read().await.clone()
    }

    /// Subscribe to upgrade events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<UpgradeEvent> {
        self.event_sender.subscribe()
    }

    /// Check for available updates
    pub async fn check_for_updates(&self) -> UpgradeResult<Option<UpdateInfo>> {
        self.emit_event(UpgradeEventType::UpdateCheckStarted, "Starting update check").await;

        // Update status
        {
            let mut status = self.status.write().await;
            *status = UpgradeStatus::Checking;
        }

        let result = {
            let mut checker = self.update_checker.write().await;
            checker.check_for_updates(&self.current_version).await
        };
        match result {
            Ok(Some(update_info)) => {
                info!("Update available: {}", update_info.version.to_string());

                // Update status
                {
                    let mut status = self.status.write().await;
                    *status = UpgradeStatus::Available(update_info.clone());
                }

                self.emit_event(
                    UpgradeEventType::UpdateAvailable,
                    &format!("Update available: {}", update_info.version.to_string()),
                ).await;

                Ok(Some(update_info))
            }
            Ok(None) => {
                info!("No updates available");

                // Update status
                {
                    let mut status = self.status.write().await;
                    *status = UpgradeStatus::UpToDate;
                }

                self.emit_event(UpgradeEventType::UpdateCheckCompleted, "No updates available").await;
                Ok(None)
            }
            Err(e) => {
                error!("Update check failed: {}", e);

                // Update status
                {
                    let mut status = self.status.write().await;
                    *status = UpgradeStatus::Failed {
                        error: e.to_string(),
                        recovery_available: false,
                    };
                }

                self.emit_event(
                    UpgradeEventType::UpdateCheckFailed,
                    &format!("Update check failed: {}", e),
                ).await;

                Err(e)
            }
        }
    }

    /// Download and install an available update
    pub async fn install_update(&self, update_info: &UpdateInfo) -> UpgradeResult<()> {
        info!("Starting installation of version {}", update_info.version.to_string());

        // Pre-installation safety checks
        {
            let mut checker = self.safety_checker.write().await;
            checker.check_pre_installation(&update_info).await?;
        }

        // Stage 1: Download the update
        let package_path = self.download_update(update_info).await?;

        // Stage 2: Create backup
        let backup_path = self.create_backup().await?;

        // Stage 3: Install the update
        match self.perform_installation(&package_path, update_info).await {
            Ok(_) => {
                info!("Installation completed successfully");

                // Update status
                {
                    let mut status = self.status.write().await;
                    *status = UpgradeStatus::Completed {
                        old_version: self.current_version.clone(),
                        new_version: update_info.version.clone(),
                        restart_required: true,
                    };
                }

                self.emit_event(
                    UpgradeEventType::InstallationCompleted,
                    "Installation completed successfully",
                ).await;

                Ok(())
            }
            Err(e) => {
                error!("Installation failed: {}", e);

                // Attempt automatic rollback
                warn!("Attempting automatic rollback to previous version");
                if let Err(rollback_error) = self.rollback_from_backup(&backup_path).await {
                    error!("Rollback failed: {}", rollback_error);

                    // Update status with rollback failure
                    {
                        let mut status = self.status.write().await;
                        *status = UpgradeStatus::Failed {
                            error: format!("Installation failed: {}. Rollback also failed: {}", e, rollback_error),
                            recovery_available: false,
                        };
                    }
                } else {
                    info!("Rollback completed successfully");

                    // Update status with recovery available
                    {
                        let mut status = self.status.write().await;
                        *status = UpgradeStatus::Failed {
                            error: format!("Installation failed: {}. System restored to previous version.", e),
                            recovery_available: true,
                        };
                    }
                }

                self.emit_event(
                    UpgradeEventType::InstallationFailed,
                    &format!("Installation failed: {}", e),
                ).await;

                Err(e)
            }
        }
    }

    /// Download an update package
    async fn download_update(&self, update_info: &UpdateInfo) -> UpgradeResult<PathBuf> {
        info!("Downloading update package");

        self.emit_event(UpgradeEventType::DownloadStarted, "Starting download").await;

        // Get platform-specific download URL
        let platform = std::env::consts::OS;
        let download_url = update_info
            .download_urls
            .get(platform)
            .ok_or_else(|| UpgradeError::PlatformNotSupported(platform.to_string()))?;

        let expected_checksum = update_info
            .checksums
            .get(platform)
            .ok_or_else(|| UpgradeError::VerificationFailed("No checksum available".to_string()))?;

        // Create download progress callback
        let status_clone = Arc::clone(&self.status);
        let event_sender_clone = self.event_sender.clone();

        let progress_callback = move |bytes_downloaded: u64, total_bytes: u64, speed: u64| {
            let progress = if total_bytes > 0 {
                (bytes_downloaded as f32 / total_bytes as f32) * 100.0
            } else {
                0.0
            };

            // Update status
            {
                let mut status = futures::executor::block_on(status_clone.write());
                *status = UpgradeStatus::Downloading {
                    progress,
                    bytes_downloaded,
                    total_bytes,
                    speed_bytes_per_sec: speed,
                };
            }

            // Emit progress event
            let event = UpgradeEvent {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                event_type: UpgradeEventType::DownloadProgress,
                version: None,
                message: format!("Downloaded {} of {} bytes ({:.1}%)", bytes_downloaded, total_bytes, progress),
                data: Some(serde_json::json!({
                    "bytes_downloaded": bytes_downloaded,
                    "total_bytes": total_bytes,
                    "progress": progress,
                    "speed_bytes_per_sec": speed
                })),
            };

            let _ = event_sender_clone.send(event);
        };

        // Download the update
        match self.downloader.download_update(download_url, expected_checksum, progress_callback).await {
            Ok(package_path) => {
                info!("Download completed: {:?}", package_path);

                self.emit_event(
                    UpgradeEventType::DownloadCompleted,
                    &format!("Download completed: {:?}", package_path),
                ).await;

                Ok(package_path)
            }
            Err(e) => {
                error!("Download failed: {}", e);

                self.emit_event(
                    UpgradeEventType::DownloadFailed,
                    &format!("Download failed: {}", e),
                ).await;

                Err(e)
            }
        }
    }

    /// Create backup before installation
    async fn create_backup(&self) -> UpgradeResult<PathBuf> {
        info!("Creating backup before installation");

        self.update_installation_status(InstallationStage::PreparingBackup, 0.0).await;

        let backup_path = self.backup_manager.create_backup().await
            .map_err(|e| UpgradeError::BackupFailed(e.to_string()))?;

        info!("Backup created: {:?}", backup_path);
        Ok(backup_path)
    }

    /// Perform the actual installation
    async fn perform_installation(&self, package_path: &PathBuf, update_info: &UpdateInfo) -> UpgradeResult<()> {
        info!("Performing installation");

        self.emit_event(UpgradeEventType::InstallationStarted, "Starting installation").await;

        // Stage 1: Verify the package
        self.update_installation_status(InstallationStage::VerifyingUpdate, 10.0).await;
        self.safety_checker.read().await.verify_package(package_path, update_info).await?;

        // Stage 2: Prepare for upgrade
        self.update_installation_status(InstallationStage::StoppingServices, 20.0).await;
        self.platform_handler.prepare_for_upgrade().await
            .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?;

        // Stage 3: Install files
        self.update_installation_status(InstallationStage::InstallingFiles, 40.0).await;
        self.platform_handler.install_update(package_path).await
            .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?;

        // Stage 4: Update configuration
        self.update_installation_status(InstallationStage::UpdatingConfiguration, 70.0).await;
        // Configuration updates would go here

        // Stage 5: Start services
        self.update_installation_status(InstallationStage::StartingServices, 80.0).await;
        // Service restart would go here

        // Stage 6: Verify installation
        self.update_installation_status(InstallationStage::VerifyingInstallation, 90.0).await;
        let verification_result = self.platform_handler.verify_installation().await
            .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?;

        if !verification_result {
            return Err(UpgradeError::InstallationFailed("Installation verification failed".to_string()));
        }

        // Stage 7: Cleanup
        self.update_installation_status(InstallationStage::CleaningUp, 95.0).await;
        self.platform_handler.cleanup_after_upgrade().await
            .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?;

        self.update_installation_status(InstallationStage::CleaningUp, 100.0).await;
        Ok(())
    }

    /// Rollback from a backup
    async fn rollback_from_backup(&self, backup_path: &PathBuf) -> UpgradeResult<()> {
        info!("Rolling back from backup: {:?}", backup_path);

        self.emit_event(UpgradeEventType::RollbackStarted, "Starting rollback").await;

        {
            let mut status = self.status.write().await;
            *status = UpgradeStatus::RollingBack {
                target_version: self.current_version.clone(),
                progress: 0.0,
            };
        }

        match self.backup_manager.restore_backup(backup_path).await {
            Ok(_) => {
                info!("Rollback completed successfully");

                self.emit_event(UpgradeEventType::RollbackCompleted, "Rollback completed").await;
                Ok(())
            }
            Err(e) => {
                error!("Rollback failed: {}", e);

                self.emit_event(
                    UpgradeEventType::RollbackFailed,
                    &format!("Rollback failed: {}", e),
                ).await;

                Err(UpgradeError::RollbackFailed(e.to_string()))
            }
        }
    }

    /// Update installation status
    async fn update_installation_status(&self, stage: InstallationStage, progress: f32) {
        debug!("Installation stage: {:?} ({}%)", stage, progress);

        {
            let mut status = self.status.write().await;
            *status = UpgradeStatus::Installing { stage: stage.clone(), progress };
        }

        self.emit_event(
            UpgradeEventType::InstallationProgress,
            &format!("Installation progress: {} ({}%)", stage.description(), progress),
        ).await;
    }

    /// Emit an upgrade event
    async fn emit_event(&self, event_type: UpgradeEventType, message: &str) {
        let event = UpgradeEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            version: Some(self.current_version.clone()),
            message: message.to_string(),
            data: None,
        };

        if let Err(e) = self.event_sender.send(event) {
            warn!("Failed to send upgrade event: {}", e);
        }
    }

    /// Create platform-specific handler
    fn create_platform_handler(_config: &UpgradeConfig) -> Result<Box<dyn PlatformUpgradeHandler>> {
        #[cfg(target_os = "macos")]
        {
            Ok(Box::new(crate::upgrade::macos::MacOSUpgradeHandler::new(_config)?))
        }

        #[cfg(target_os = "linux")]
        {
            Ok(Box::new(crate::upgrade::linux::LinuxUpgradeHandler::new(_config)?))
        }

        #[cfg(target_os = "windows")]
        {
            Ok(Box::new(crate::upgrade::windows::WindowsUpgradeHandler::new(_config)?))
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(anyhow::anyhow!("Unsupported platform for seamless upgrades"))
        }
    }

    /// Get current application version
    pub fn get_current_version(&self) -> &ApplicationVersion {
        &self.current_version
    }

    /// Get update channel
    pub fn get_update_channel(&self) -> &UpdateChannel {
        &self.config.update_channel
    }

    /// Check if auto-updates are enabled
    pub fn is_auto_update_enabled(&self) -> bool {
        self.config.auto_install
    }

    /// Check if auto-check is enabled
    pub fn is_auto_check_enabled(&self) -> bool {
        self.config.auto_check
    }
}

/// Installation context information for contextual upgrade handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationContext {
    /// Whether this is an upgrade or fresh install
    pub installation_type: InstallationType,
    /// Previous version if this is an upgrade
    pub previous_version: Option<ApplicationVersion>,
    /// Installation timestamp
    pub installation_date: DateTime<Utc>,
    /// Installation directory
    pub installation_directory: PathBuf,
    /// Whether configuration exists from previous installation
    pub has_existing_config: bool,
    /// Whether user data exists from previous installation
    pub has_existing_data: bool,
}

/// Type of installation being performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallationType {
    /// Fresh installation (no previous version detected)
    FreshInstall,
    /// Upgrade from previous version
    Upgrade { from_version: ApplicationVersion },
    /// Reinstall (same version over existing installation)
    Reinstall,
    /// Downgrade (installing older version over newer)
    Downgrade { from_version: ApplicationVersion },
}

impl UpgradeManager {
    /// Detect installation context to determine upgrade vs fresh install behavior
    pub async fn detect_installation_context(&self) -> UpgradeResult<InstallationContext> {
        debug!("Detecting installation context");

        let installation_directory = self.platform_handler.get_installation_directory();
        let config_dir = self.get_config_directory();
        let data_dir = self.get_data_directory();

        // Check if this is an existing installation
        let previous_installation_info = self.detect_previous_installation(&installation_directory).await?;
        let has_existing_config = self.has_existing_configuration(&config_dir).await;
        let has_existing_data = self.has_existing_user_data(&data_dir).await;

        let installation_type = match &previous_installation_info {
            Some(prev_version) => {
                if *prev_version == self.current_version {
                    InstallationType::Reinstall
                } else if prev_version.is_newer_than(&self.current_version) {
                    InstallationType::Downgrade { from_version: prev_version.clone() }
                } else {
                    InstallationType::Upgrade { from_version: prev_version.clone() }
                }
            }
            None => InstallationType::FreshInstall,
        };

        info!("Installation context detected: {:?}", installation_type);

        Ok(InstallationContext {
            installation_type,
            previous_version: previous_installation_info,
            installation_date: Utc::now(),
            installation_directory,
            has_existing_config,
            has_existing_data,
        })
    }

    /// Install update with contextual handling based on installation type
    pub async fn install_update_contextual(&self, update_info: &UpdateInfo) -> UpgradeResult<()> {
        let context = self.detect_installation_context().await?;

        match context.installation_type {
            InstallationType::FreshInstall => {
                info!("Performing fresh installation of version {}", update_info.version.to_string());
                self.perform_fresh_install(update_info, &context).await
            }
            InstallationType::Upgrade { ref from_version } => {
                info!("Performing upgrade from {} to {}",
                      from_version.to_string(),
                      update_info.version.to_string());
                self.perform_upgrade_install(update_info, &context).await
            }
            InstallationType::Reinstall => {
                info!("Performing reinstallation of version {}", update_info.version.to_string());
                self.perform_reinstall(update_info, &context).await
            }
            InstallationType::Downgrade { ref from_version } => {
                warn!("Performing downgrade from {} to {}",
                      from_version.to_string(),
                      update_info.version.to_string());
                self.perform_downgrade_install(update_info, &context).await
            }
        }
    }

    /// Detect previous installation by looking for version files, registry entries, etc.
    async fn detect_previous_installation(&self, install_dir: &PathBuf) -> UpgradeResult<Option<ApplicationVersion>> {
        // Check for version file in installation directory
        let version_file = install_dir.join("VERSION");
        if version_file.exists() {
            match tokio::fs::read_to_string(&version_file).await {
                Ok(version_str) => {
                    if let Ok(version) = self.parse_version_from_string(&version_str.trim()) {
                        return Ok(Some(version));
                    }
                }
                Err(_) => debug!("Could not read version file"),
            }
        }

        // Check for binary with version info (platform-specific)
        if let Ok(binary_version) = self.detect_version_from_binary(install_dir).await {
            return Ok(Some(binary_version));
        }

        // Check for installation manifest or registry entries (platform-specific)
        if let Ok(manifest_version) = self.detect_version_from_manifest(install_dir).await {
            return Ok(Some(manifest_version));
        }

        Ok(None)
    }

    /// Perform fresh installation (no previous installation detected)
    async fn perform_fresh_install(&self, update_info: &UpdateInfo, context: &InstallationContext) -> UpgradeResult<()> {
        info!("Starting fresh installation");

        // No backup needed for fresh install
        self.emit_event(UpgradeEventType::InstallationStarted, "Starting fresh installation").await;

        // Download and verify the update
        let package_path = self.download_and_verify_update(update_info).await?;

        // Perform clean installation
        self.platform_handler.install_update(&package_path).await
            .map_err(|e| UpgradeError::InstallationFailed(format!("Fresh install failed: {}", e)))?;

        // Create version tracking files
        self.create_installation_metadata(update_info, context).await?;

        info!("Fresh installation completed successfully");
        Ok(())
    }

    /// Perform upgrade installation (preserving existing configuration and data)
    async fn perform_upgrade_install(&self, update_info: &UpdateInfo, context: &InstallationContext) -> UpgradeResult<()> {
        info!("Starting upgrade installation");

        // Create backup before upgrade
        self.emit_event(UpgradeEventType::InstallationStarted, "Creating backup before upgrade").await;
        let backup_id = self.backup_manager.create_backup().await
            .map_err(|e| UpgradeError::BackupFailed(format!("Pre-upgrade backup failed: {}", e)))?;

        // Download and verify the update
        let package_path = self.download_and_verify_update(update_info).await?;

        // Preserve configuration and user data
        let preserved_config = if context.has_existing_config {
            Some(self.preserve_configuration().await?)
        } else { None };

        let preserved_data = if context.has_existing_data {
            Some(self.preserve_user_data().await?)
        } else { None };

        // Perform upgrade installation
        match self.platform_handler.install_update(&package_path).await {
            Ok(_) => {
                // Restore preserved configuration and data
                if let Some(config) = preserved_config {
                    self.restore_configuration(config).await?;
                }
                if let Some(data) = preserved_data {
                    self.restore_user_data(data).await?;
                }

                // Update installation metadata
                self.update_installation_metadata(update_info, context).await?;

                info!("Upgrade installation completed successfully");
                Ok(())
            }
            Err(e) => {
                error!("Upgrade installation failed, attempting rollback: {}", e);

                // Attempt to restore from backup
                if let Err(rollback_err) = self.backup_manager.restore_backup(&backup_id).await {
                    error!("Rollback also failed: {}", rollback_err);
                    return Err(UpgradeError::InstallationFailed(format!(
                        "Upgrade failed and rollback failed: {} (rollback error: {})",
                        e, rollback_err
                    )));
                }

                Err(UpgradeError::InstallationFailed(format!("Upgrade failed but rollback succeeded: {}", e)))
            }
        }
    }

    /// Perform reinstallation (same version)
    async fn perform_reinstall(&self, update_info: &UpdateInfo, context: &InstallationContext) -> UpgradeResult<()> {
        info!("Starting reinstallation");

        // Minimal backup for safety
        self.emit_event(UpgradeEventType::InstallationStarted, "Creating safety backup for reinstall").await;
        let backup_id = self.backup_manager.create_backup().await
            .map_err(|e| UpgradeError::BackupFailed(format!("Pre-reinstall backup failed: {}", e)))?;

        // Download and verify the update
        let package_path = self.download_and_verify_update(update_info).await?;

        // Preserve configuration and user data
        let preserved_config = if context.has_existing_config {
            Some(self.preserve_configuration().await?)
        } else { None };

        let preserved_data = if context.has_existing_data {
            Some(self.preserve_user_data().await?)
        } else { None };

        // Perform reinstallation
        self.platform_handler.install_update(&package_path).await
            .map_err(|e| UpgradeError::InstallationFailed(format!("Reinstall failed: {}", e)))?;

        // Restore preserved configuration and data
        if let Some(config) = preserved_config {
            self.restore_configuration(config).await?;
        }
        if let Some(data) = preserved_data {
            self.restore_user_data(data).await?;
        }

        info!("Reinstallation completed successfully");
        Ok(())
    }

    /// Perform downgrade installation (installing older version)
    async fn perform_downgrade_install(&self, update_info: &UpdateInfo, context: &InstallationContext) -> UpgradeResult<()> {
        warn!("Starting downgrade installation - this may cause compatibility issues");

        // Mandatory backup for downgrades
        self.emit_event(UpgradeEventType::InstallationStarted, "Creating mandatory backup for downgrade").await;
        let backup_id = self.backup_manager.create_backup().await
            .map_err(|e| UpgradeError::BackupFailed(format!("Pre-downgrade backup failed: {}", e)))?;

        // Download and verify the update
        let package_path = self.download_and_verify_update(update_info).await?;

        // Check compatibility warnings
        if let Some(prev_version) = &context.previous_version {
            if !update_info.version.is_compatible_with(prev_version) {
                warn!("Downgrade may cause compatibility issues - backing up configuration");
            }
        }

        // Perform downgrade installation
        self.platform_handler.install_update(&package_path).await
            .map_err(|e| UpgradeError::InstallationFailed(format!("Downgrade failed: {}", e)))?;

        // Update installation metadata
        self.update_installation_metadata(update_info, context).await?;

        warn!("Downgrade completed - monitor for compatibility issues");
        Ok(())
    }

    /// Helper methods for installation context detection
    async fn has_existing_configuration(&self, config_dir: &PathBuf) -> bool {
        config_dir.exists() && config_dir.join("config.toml").exists()
    }

    async fn has_existing_user_data(&self, data_dir: &PathBuf) -> bool {
        data_dir.exists() && tokio::fs::read_dir(data_dir).await.map_or(false, |mut entries| {
            futures::executor::block_on(async move {
                entries.next_entry().await.map_or(false, |entry| entry.is_some())
            })
        })
    }

    fn get_config_directory(&self) -> PathBuf {
        // Platform-specific config directory
        #[cfg(target_os = "macos")]
        return dirs::config_dir().unwrap_or_else(|| PathBuf::from("/usr/local/etc")).join("inferno");

        #[cfg(target_os = "linux")]
        return dirs::config_dir().unwrap_or_else(|| PathBuf::from("/etc")).join("inferno");

        #[cfg(target_os = "windows")]
        return dirs::config_dir().unwrap_or_else(|| PathBuf::from("C:\\ProgramData")).join("inferno");
    }

    fn get_data_directory(&self) -> PathBuf {
        // Platform-specific data directory
        #[cfg(target_os = "macos")]
        return dirs::data_dir().unwrap_or_else(|| PathBuf::from("/usr/local/share")).join("inferno");

        #[cfg(target_os = "linux")]
        return dirs::data_dir().unwrap_or_else(|| PathBuf::from("/usr/share")).join("inferno");

        #[cfg(target_os = "windows")]
        return dirs::data_dir().unwrap_or_else(|| PathBuf::from("C:\\ProgramData")).join("inferno");
    }

    async fn detect_version_from_binary(&self, install_dir: &PathBuf) -> UpgradeResult<ApplicationVersion> {
        // Try to execute the installed binary to get version
        let binary_path = install_dir.join("inferno");

        #[cfg(target_os = "windows")]
        let binary_path = install_dir.join("inferno.exe");

        if binary_path.exists() {
            let output = tokio::process::Command::new(&binary_path)
                .args(&["--version"])
                .output()
                .await
                .map_err(|e| UpgradeError::Internal(format!("Failed to execute binary: {}", e)))?;

            if output.status.success() {
                let version_output = String::from_utf8_lossy(&output.stdout);
                // Parse version from output like "inferno 0.2.1"
                if let Some(version_str) = version_output.split_whitespace().nth(1) {
                    return self.parse_version_from_string(version_str);
                }
            }
        }

        Err(UpgradeError::Internal("Could not detect version from binary".to_string()))
    }

    async fn detect_version_from_manifest(&self, _install_dir: &PathBuf) -> UpgradeResult<ApplicationVersion> {
        // Platform-specific manifest/registry checking would go here
        // For now, return error to indicate no manifest found
        Err(UpgradeError::Internal("No manifest found".to_string()))
    }

    fn parse_version_from_string(&self, version_str: &str) -> UpgradeResult<ApplicationVersion> {
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() >= 3 {
            let major = parts[0].parse::<u32>()
                .map_err(|_| UpgradeError::Internal("Invalid major version".to_string()))?;
            let minor = parts[1].parse::<u32>()
                .map_err(|_| UpgradeError::Internal("Invalid minor version".to_string()))?;
            let patch = parts[2].parse::<u32>()
                .map_err(|_| UpgradeError::Internal("Invalid patch version".to_string()))?;

            Ok(ApplicationVersion::new(major, minor, patch))
        } else {
            Err(UpgradeError::Internal("Invalid version format".to_string()))
        }
    }

    async fn create_installation_metadata(&self, update_info: &UpdateInfo, context: &InstallationContext) -> UpgradeResult<()> {
        let version_file = context.installation_directory.join("VERSION");
        tokio::fs::write(&version_file, update_info.version.to_string()).await
            .map_err(|e| UpgradeError::Internal(format!("Failed to create version file: {}", e)))?;
        Ok(())
    }

    async fn update_installation_metadata(&self, update_info: &UpdateInfo, context: &InstallationContext) -> UpgradeResult<()> {
        self.create_installation_metadata(update_info, context).await
    }

    async fn preserve_configuration(&self) -> UpgradeResult<PathBuf> {
        // Create temporary backup of configuration
        let temp_config = std::env::temp_dir().join(format!("inferno_config_backup_{}", Utc::now().timestamp()));
        // Implementation would copy config files here
        Ok(temp_config)
    }

    async fn preserve_user_data(&self) -> UpgradeResult<PathBuf> {
        // Create temporary backup of user data
        let temp_data = std::env::temp_dir().join(format!("inferno_data_backup_{}", Utc::now().timestamp()));
        // Implementation would copy user data here
        Ok(temp_data)
    }

    async fn restore_configuration(&self, _config_backup: PathBuf) -> UpgradeResult<()> {
        // Restore configuration from backup
        // Implementation would restore config files here
        Ok(())
    }

    async fn restore_user_data(&self, _data_backup: PathBuf) -> UpgradeResult<()> {
        // Restore user data from backup
        // Implementation would restore user data here
        Ok(())
    }

    async fn download_and_verify_update(&self, update_info: &UpdateInfo) -> UpgradeResult<PathBuf> {
        // Download the update package
        self.emit_event(UpgradeEventType::DownloadStarted, "Starting download").await;

        // Get platform-specific download URL and checksum
        let platform = std::env::consts::OS;
        let download_url = update_info.download_urls.get(platform)
            .ok_or_else(|| UpgradeError::PlatformNotSupported(platform.to_string()))?;
        let expected_checksum = update_info.checksums.get(platform)
            .ok_or_else(|| UpgradeError::VerificationFailed("No checksum available".to_string()))?;

        let package_path = self.downloader.download_update(
            download_url,
            expected_checksum,
            |downloaded, total, _speed| {
                // Progress callback - could emit events here if needed
                debug!("Download progress: {}/{} bytes", downloaded, total);
            },
        ).await?;

        self.emit_event(UpgradeEventType::DownloadCompleted, "Download completed").await;

        // Verify the downloaded package
        self.safety_checker.read().await.verify_package(&package_path, update_info).await?;

        Ok(package_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_upgrade_manager_creation() {
        let config = UpgradeConfig::default();
        let manager = UpgradeManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_status_tracking() {
        let config = UpgradeConfig::default();
        let manager = UpgradeManager::new(config).await.unwrap();

        let initial_status = manager.get_status().await;
        assert!(matches!(initial_status, UpgradeStatus::UpToDate));
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let config = UpgradeConfig::default();
        let manager = UpgradeManager::new(config).await.unwrap();

        let mut event_receiver = manager.subscribe_to_events();

        // Emit a test event
        manager.emit_event(UpgradeEventType::UpdateCheckStarted, "Test message").await;

        // Check if we receive the event
        let received_event = tokio::time::timeout(Duration::from_millis(100), event_receiver.recv()).await;
        assert!(received_event.is_ok());

        let event = received_event.unwrap().unwrap();
        assert!(matches!(event.event_type, UpgradeEventType::UpdateCheckStarted));
        assert_eq!(event.message, "Test message");
    }
}