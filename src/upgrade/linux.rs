//! # Linux-Specific Upgrade Handler
//!
//! Handles Linux-specific upgrade operations including package management,
//! systemd service integration, and distribution-specific handling.

use super::{platform::BasePlatformHandler, PlatformUpgradeHandler, UpgradeConfig};
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// Linux-specific upgrade handler
pub struct LinuxUpgradeHandler {
    base: BasePlatformHandler,
}

impl LinuxUpgradeHandler {
    /// Create a new Linux upgrade handler
    pub fn new(config: &UpgradeConfig) -> Result<Self> {
        let base = BasePlatformHandler::new(config)?;
        Ok(Self { base })
    }
}

#[async_trait::async_trait]
impl PlatformUpgradeHandler for LinuxUpgradeHandler {
    fn supports_seamless_upgrade(&self) -> bool {
        self.base.supports_seamless_upgrade()
    }

    async fn prepare_for_upgrade(&self) -> Result<()> {
        info!("Preparing Linux system for upgrade");
        self.base.stop_services().await?;
        Ok(())
    }

    async fn install_update(&self, _package_path: &PathBuf) -> Result<()> {
        anyhow::bail!("Linux upgrade installation not yet implemented")
    }

    async fn restart_application(&self) -> Result<()> {
        info!("Restarting application on Linux");
        // TODO: Implement Linux-specific restart
        Ok(())
    }

    async fn verify_installation(&self) -> Result<bool> {
        self.base.verify_installation().await.map_err(Into::into)
    }

    async fn cleanup_after_upgrade(&self) -> Result<()> {
        self.base.cleanup_temporary_files().await.map_err(Into::into)
    }

    fn requires_elevated_privileges(&self) -> bool {
        self.base.requires_elevated_privileges()
    }

    fn get_installation_directory(&self) -> PathBuf {
        self.base.get_installation_directory()
    }

    fn get_backup_directory(&self) -> PathBuf {
        self.base.get_backup_directory()
    }
}
