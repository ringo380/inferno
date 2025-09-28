//! # Background Update Service
//!
//! Persistent background service that automatically checks for updates and notifies
//! all active interfaces about available upgrades in real-time.

use super::{
    ApplicationVersion, UpdateInfo, UpgradeConfig, UpgradeError, UpgradeEvent, UpgradeEventType,
    UpgradeManager, UpgradeResult, UpgradeStatus,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};

/// Background service for automatic update checking and notifications
#[derive(Clone)]
pub struct BackgroundUpdateService {
    upgrade_manager: Arc<UpgradeManager>,
    config: UpgradeConfig,
    event_sender: broadcast::Sender<UpgradeEvent>,
    status: Arc<RwLock<ServiceStatus>>,
    should_stop: Arc<RwLock<bool>>,
}

/// Status of the background update service
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub running: bool,
    pub last_check: Option<DateTime<Utc>>,
    pub next_check: Option<DateTime<Utc>>,
    pub check_count: u64,
    pub last_error: Option<String>,
    pub available_update: Option<UpdateInfo>,
}

impl Default for ServiceStatus {
    fn default() -> Self {
        Self {
            running: false,
            last_check: None,
            next_check: None,
            check_count: 0,
            last_error: None,
            available_update: None,
        }
    }
}

impl BackgroundUpdateService {
    /// Create a new background update service
    pub fn new(
        upgrade_manager: Arc<UpgradeManager>,
        config: UpgradeConfig,
        event_sender: broadcast::Sender<UpgradeEvent>,
    ) -> Self {
        Self {
            upgrade_manager,
            config,
            event_sender,
            status: Arc::new(RwLock::new(ServiceStatus::default())),
            should_stop: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the background service
    pub async fn start(&self) -> Result<()> {
        info!("Starting background update service");

        // Mark service as running
        {
            let mut status = self.status.write().await;
            status.running = true;
            status.next_check = Some(Utc::now() + chrono::Duration::from_std(self.config.check_interval)?);
        }

        // Emit service started event
        self.emit_service_event(UpgradeEventType::UpdateCheckStarted, "Background update service started").await;

        // Start the checking loop
        self.run_check_loop().await
    }

    /// Stop the background service
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping background update service");

        {
            let mut should_stop = self.should_stop.write().await;
            *should_stop = true;
        }

        {
            let mut status = self.status.write().await;
            status.running = false;
            status.next_check = None;
        }

        self.emit_service_event(UpgradeEventType::UpdateCheckCompleted, "Background update service stopped").await;

        Ok(())
    }

    /// Get current service status
    pub async fn get_status(&self) -> ServiceStatus {
        self.status.read().await.clone()
    }

    /// Force an immediate update check
    pub async fn check_now(&self) -> UpgradeResult<Option<UpdateInfo>> {
        info!("Performing immediate update check");
        self.perform_update_check().await
    }

    /// Main checking loop
    async fn run_check_loop(&self) -> Result<()> {
        let mut check_interval = interval(self.config.check_interval);

        loop {
            // Check if we should stop
            {
                let should_stop = self.should_stop.read().await;
                if *should_stop {
                    break;
                }
            }

            // Wait for next check interval
            check_interval.tick().await;

            // Check if auto-check is still enabled
            if !self.config.auto_check {
                debug!("Auto-check disabled, skipping update check");
                continue;
            }

            // Perform the update check
            match self.perform_update_check().await {
                Ok(Some(update_info)) => {
                    self.handle_update_available(update_info).await;
                }
                Ok(None) => {
                    self.handle_no_update_available().await;
                }
                Err(e) => {
                    self.handle_check_error(e).await;
                }
            }
        }

        Ok(())
    }

    /// Perform actual update check
    async fn perform_update_check(&self) -> UpgradeResult<Option<UpdateInfo>> {
        debug!("Checking for updates");

        // Update status
        {
            let mut status = self.status.write().await;
            status.last_check = Some(Utc::now());
            status.check_count += 1;
            status.last_error = None;
            status.next_check = Some(Utc::now() + chrono::Duration::from_std(self.config.check_interval).unwrap_or(chrono::Duration::hours(1)));
        }

        // Delegate to upgrade manager
        self.upgrade_manager.check_for_updates().await
    }

    /// Handle when an update is available
    async fn handle_update_available(&self, update_info: UpdateInfo) {
        info!("Update available: {} -> {}",
              ApplicationVersion::current().to_string(),
              update_info.version.to_string());

        // Update status
        {
            let mut status = self.status.write().await;
            status.available_update = Some(update_info.clone());
        }

        // Emit notification event
        self.emit_update_event(
            UpgradeEventType::UpdateAvailable,
            &format!("New version {} is available", update_info.version.to_string()),
            Some(update_info.clone()),
        ).await;

        // Send notification to all interfaces
        self.notify_interfaces_of_update(&update_info).await;

        // Auto-install if configured and not critical
        if self.config.should_auto_install(update_info.is_critical) {
            info!("Auto-installing update");
            self.initiate_auto_update(update_info).await;
        }
    }

    /// Handle when no update is available
    async fn handle_no_update_available(&self) {
        debug!("No updates available");

        // Clear available update from status
        {
            let mut status = self.status.write().await;
            status.available_update = None;
        }
    }

    /// Handle check errors
    async fn handle_check_error(&self, error: UpgradeError) {
        warn!("Update check failed: {}", error);

        // Update status with error
        {
            let mut status = self.status.write().await;
            status.last_error = Some(error.to_string());
        }

        // Emit error event
        self.emit_service_event(
            UpgradeEventType::UpdateCheckFailed,
            &format!("Update check failed: {}", error),
        ).await;

        // Implement exponential backoff on repeated failures
        if self.should_apply_backoff().await {
            warn!("Applying exponential backoff due to repeated failures");
            sleep(Duration::from_secs(300)).await; // 5 minutes extra delay
        }
    }

    /// Notify all active interfaces about available update
    async fn notify_interfaces_of_update(&self, update_info: &UpdateInfo) {
        // Create comprehensive notification data
        let notification_data = serde_json::json!({
            "update_available": true,
            "current_version": ApplicationVersion::current().to_string(),
            "new_version": update_info.version.to_string(),
            "release_date": update_info.release_date,
            "is_critical": update_info.is_critical,
            "is_security_update": update_info.is_security_update,
            "changelog_preview": self.get_changelog_preview(&update_info.changelog),
            "download_size": self.get_download_size_for_platform(update_info),
            "can_auto_install": self.config.should_auto_install(update_info.is_critical),
        });

        // Emit interface notification event
        self.emit_update_event(
            UpgradeEventType::UpdateAvailable,
            "Update notification sent to interfaces",
            Some(update_info.clone()),
        ).await;

        // The event will be received by:
        // - TUI interface (if running)
        // - Web dashboard (via WebSocket)
        // - CLI commands (via status check)
    }

    /// Initiate automatic update installation
    async fn initiate_auto_update(&self, update_info: UpdateInfo) {
        info!("Initiating automatic update installation");

        // Emit auto-install start event
        self.emit_update_event(
            UpgradeEventType::InstallationStarted,
            "Automatic update installation started",
            Some(update_info.clone()),
        ).await;

        // Delegate to upgrade manager for actual installation
        match self.upgrade_manager.install_update(&update_info).await {
            Ok(_) => {
                info!("Automatic update installation completed successfully");
                self.emit_update_event(
                    UpgradeEventType::InstallationCompleted,
                    "Automatic update installation completed",
                    Some(update_info),
                ).await;
            }
            Err(e) => {
                error!("Automatic update installation failed: {}", e);
                self.emit_service_event(
                    UpgradeEventType::InstallationFailed,
                    &format!("Automatic update installation failed: {}", e),
                ).await;
            }
        }
    }

    /// Check if exponential backoff should be applied
    async fn should_apply_backoff(&self) -> bool {
        let status = self.status.read().await;

        // Apply backoff if we've had recent errors
        status.last_error.is_some() &&
        status.last_check.map_or(false, |last| {
            Utc::now().signed_duration_since(last) < chrono::Duration::hours(1)
        })
    }

    /// Get abbreviated changelog preview
    fn get_changelog_preview(&self, changelog: &str) -> String {
        let lines: Vec<&str> = changelog.lines().take(3).collect();
        let preview = lines.join("\n");

        if changelog.lines().count() > 3 {
            format!("{}...", preview)
        } else {
            preview
        }
    }

    /// Get download size for current platform
    fn get_download_size_for_platform(&self, update_info: &UpdateInfo) -> Option<u64> {
        let platform = std::env::consts::OS;
        update_info.size_bytes.get(platform).copied()
    }

    /// Emit service-related event
    async fn emit_service_event(&self, event_type: UpgradeEventType, message: &str) {
        let event = UpgradeEvent {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            version: Some(ApplicationVersion::current()),
            message: message.to_string(),
            data: None,
        };

        if let Err(e) = self.event_sender.send(event) {
            debug!("Failed to send service event: {}", e);
        }
    }

    /// Emit update-related event with data
    async fn emit_update_event(&self, event_type: UpgradeEventType, message: &str, update_info: Option<UpdateInfo>) {
        let data = update_info.as_ref().map(|info| {
            serde_json::json!({
                "version": info.version.to_string(),
                "release_date": info.release_date,
                "is_critical": info.is_critical,
                "is_security_update": info.is_security_update,
                "changelog": info.changelog,
            })
        });

        let event = UpgradeEvent {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            version: update_info.map(|info| info.version),
            message: message.to_string(),
            data,
        };

        if let Err(e) = self.event_sender.send(event) {
            debug!("Failed to send update event: {}", e);
        }
    }

    /// Subscribe to upgrade events (for testing and monitoring)
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<UpgradeEvent> {
        self.event_sender.subscribe()
    }

    /// Get statistics about the service
    pub async fn get_statistics(&self) -> ServiceStatistics {
        let status = self.status.read().await;

        ServiceStatistics {
            total_checks: status.check_count,
            last_check: status.last_check,
            next_check: status.next_check,
            uptime: status.last_check.map(|start| Utc::now().signed_duration_since(start)),
            has_available_update: status.available_update.is_some(),
            last_error: status.last_error.clone(),
        }
    }
}

/// Statistics about the background service
#[derive(Debug, Clone)]
pub struct ServiceStatistics {
    pub total_checks: u64,
    pub last_check: Option<DateTime<Utc>>,
    pub next_check: Option<DateTime<Utc>>,
    pub uptime: Option<chrono::Duration>,
    pub has_available_update: bool,
    pub last_error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upgrade::UpgradeConfig;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_background_service_creation() {
        let config = UpgradeConfig::default();
        let upgrade_manager = Arc::new(UpgradeManager::new(config.clone()).await.unwrap());
        let (event_sender, _) = broadcast::channel(100);

        let service = BackgroundUpdateService::new(upgrade_manager, config, event_sender);
        let status = service.get_status().await;

        assert!(!status.running);
        assert_eq!(status.check_count, 0);
    }

    #[tokio::test]
    async fn test_service_status_tracking() {
        let config = UpgradeConfig::default();
        let upgrade_manager = Arc::new(UpgradeManager::new(config.clone()).await.unwrap());
        let (event_sender, _) = broadcast::channel(100);

        let service = BackgroundUpdateService::new(upgrade_manager, config, event_sender);

        // Initial status
        let status = service.get_status().await;
        assert!(!status.running);

        // After starting (we won't actually start the loop in test)
        {
            let mut status_guard = service.status.write().await;
            status_guard.running = true;
            status_guard.check_count = 1;
        }

        let status = service.get_status().await;
        assert!(status.running);
        assert_eq!(status.check_count, 1);
    }

    #[tokio::test]
    async fn test_event_emission() {
        let config = UpgradeConfig::default();
        let upgrade_manager = Arc::new(UpgradeManager::new(config.clone()).await.unwrap());
        let (event_sender, mut event_receiver) = broadcast::channel(100);

        let service = BackgroundUpdateService::new(upgrade_manager, config, event_sender);

        // Emit a test event
        service.emit_service_event(UpgradeEventType::UpdateCheckStarted, "Test message").await;

        // Verify event was received
        let received_event = tokio::time::timeout(Duration::from_millis(100), event_receiver.recv()).await;
        assert!(received_event.is_ok());

        let event = received_event.unwrap().unwrap();
        assert!(matches!(event.event_type, UpgradeEventType::UpdateCheckStarted));
        assert_eq!(event.message, "Test message");
    }
}