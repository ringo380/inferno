//! # Windows-Specific Upgrade Handler
//!
//! Handles Windows-specific upgrade operations including MSI/EXE installers,
//! service management, and Windows Store integration.

use super::{PlatformUpgradeHandler, UpgradeConfig, platform::BasePlatformHandler};
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info, warn};

/// Windows-specific upgrade handler
pub struct WindowsUpgradeHandler {
    base: BasePlatformHandler,
}

impl WindowsUpgradeHandler {
    /// Create a new Windows upgrade handler
    pub fn new(config: &UpgradeConfig) -> Result<Self> {
        let base = BasePlatformHandler::new(config)?;
        Ok(Self { base })
    }

    /// Get the current executable path
    fn get_current_executable(&self) -> Result<PathBuf> {
        std::env::current_exe().map_err(|e| anyhow::anyhow!("Failed to get executable path: {}", e))
    }

    /// Install from MSI package
    async fn install_msi(&self, package_path: &PathBuf) -> Result<()> {
        info!("Installing MSI package: {:?}", package_path);

        let output = Command::new("msiexec")
            .args([
                "/i",
                package_path.to_str().unwrap_or_default(),
                "/quiet",
                "/norestart",
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("MSI installation failed: {}", stderr);
        }

        Ok(())
    }

    /// Install from EXE installer
    async fn install_exe(&self, package_path: &PathBuf) -> Result<()> {
        info!("Running EXE installer: {:?}", package_path);

        let output = Command::new(package_path)
            .args(["/S", "/quiet"]) // Common silent install flags
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("EXE installer failed: {}", stderr);
        }

        Ok(())
    }

    /// Install binary directly to installation directory
    async fn install_binary(&self, package_path: &PathBuf) -> Result<()> {
        let install_dir = self.get_installation_directory();
        let target_binary = install_dir.join("inferno.exe");

        // Create installation directory if needed
        if !install_dir.exists() {
            std::fs::create_dir_all(&install_dir)?;
        }

        // Backup existing binary
        if target_binary.exists() {
            let backup_path = self.get_backup_directory().join("inferno.exe.backup");
            std::fs::create_dir_all(self.get_backup_directory())?;
            std::fs::copy(&target_binary, &backup_path)?;
            debug!("Backed up existing binary to {:?}", backup_path);
        }

        // Copy new binary
        std::fs::copy(package_path, &target_binary)?;

        info!("Installed binary to {:?}", target_binary);
        Ok(())
    }
}

#[async_trait::async_trait]
impl PlatformUpgradeHandler for WindowsUpgradeHandler {
    fn supports_seamless_upgrade(&self) -> bool {
        self.base.supports_seamless_upgrade()
    }

    async fn prepare_for_upgrade(&self) -> Result<()> {
        info!("Preparing Windows system for upgrade");
        self.base.stop_services().await?;
        Ok(())
    }

    async fn install_update(&self, package_path: &PathBuf) -> Result<()> {
        info!("Installing update on Windows: {:?}", package_path);

        // Determine package type by extension
        let extension = package_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "msi" => {
                self.install_msi(package_path).await?;
            }
            "exe" => {
                self.install_exe(package_path).await?;
            }
            _ => {
                // Assume it's a binary
                self.install_binary(package_path).await?;
            }
        }

        Ok(())
    }

    async fn restart_application(&self) -> Result<()> {
        info!("Restarting application on Windows");

        let current_exe = self.get_current_executable()?;

        // Check if running as a Windows service
        let is_service = std::env::var("SERVICE_NAME").is_ok();

        if is_service {
            // Restart via service controller
            if let Ok(service_name) = std::env::var("SERVICE_NAME") {
                let output = Command::new("sc").args(["stop", &service_name]).output();

                if output.is_ok() {
                    // Give service time to stop
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                    let _ = Command::new("sc").args(["start", &service_name]).output();

                    info!("Restarted Windows service: {}", service_name);
                    return Ok(());
                }
            }
            debug!("Service restart not available, spawning new process");
        }

        // Spawn new process
        info!("Spawning new process: {:?}", current_exe);
        Command::new(&current_exe)
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn new process: {}", e))?;

        info!("New process spawned, current process should exit");
        Ok(())
    }

    async fn verify_installation(&self) -> Result<bool> {
        self.base.verify_installation().await.map_err(Into::into)
    }

    async fn cleanup_after_upgrade(&self) -> Result<()> {
        self.base.cleanup_after_upgrade().await.map_err(Into::into)
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
