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
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Central upgrade manager coordinating all upgrade operations
pub struct UpgradeManager {
    config: UpgradeConfig,
    current_version: ApplicationVersion,
    update_checker: UpdateChecker,
    downloader: UpdateDownloader,
    backup_manager: BackupManager,
    safety_checker: SafetyChecker,
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
            update_checker,
            downloader,
            backup_manager,
            safety_checker,
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

        match self.update_checker.check_for_updates(&self.current_version).await {
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
        self.safety_checker.check_pre_installation(&update_info).await?;

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
        self.safety_checker.verify_package(package_path, update_info).await?;

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
            *status = UpgradeStatus::Installing { stage, progress };
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