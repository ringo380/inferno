//! # Linux-Specific Upgrade Handler
//!
//! Handles Linux-specific upgrade operations including package management,
//! systemd service integration, and distribution-specific handling.

use super::{
    platform::BasePlatformHandler, PlatformUpgradeHandler, UpgradeConfig, UpgradeError,
    UpgradeResult,
};
use anyhow::Result;
use std::path::PathBuf;

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
    async fn install_upgrade(&self, _package_path: PathBuf) -> UpgradeResult<()> {
        Err(UpgradeError::UnsupportedPlatform(
            "Linux upgrade installation not yet implemented".to_string(),
        ))
    }

    fn verify_package(&self, _package_path: &PathBuf) -> UpgradeResult<bool> {
        Ok(false)
    }

    fn get_platform_info(&self) -> super::PlatformInfo {
        self.base.get_platform_info()
    }

    fn cleanup_old_versions(&self) -> UpgradeResult<()> {
        self.base.cleanup_old_versions()
    }
}
