//! # Linux-Specific Upgrade Handler
//!
//! Handles Linux-specific upgrade operations including package management,
//! systemd service integration, and distribution-specific handling.

use super::{platform::BasePlatformHandler, PlatformUpgradeHandler, UpgradeConfig};
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info, warn};

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

    /// Check if running under systemd
    fn is_systemd_available(&self) -> bool {
        std::path::Path::new("/run/systemd/system").exists()
    }

    /// Get the current executable path
    fn get_current_executable(&self) -> Result<PathBuf> {
        std::env::current_exe().map_err(|e| anyhow::anyhow!("Failed to get executable path: {}", e))
    }

    /// Install binary directly to installation directory
    async fn install_binary(&self, package_path: &PathBuf) -> Result<()> {
        let install_dir = self.get_installation_directory();
        let target_binary = install_dir.join("inferno");

        // Create installation directory if needed
        if !install_dir.exists() {
            std::fs::create_dir_all(&install_dir)?;
        }

        // Backup existing binary
        if target_binary.exists() {
            let backup_path = self.get_backup_directory().join("inferno.backup");
            std::fs::create_dir_all(self.get_backup_directory())?;
            std::fs::copy(&target_binary, &backup_path)?;
            debug!("Backed up existing binary to {:?}", backup_path);
        }

        // Copy new binary
        std::fs::copy(package_path, &target_binary)?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&target_binary)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&target_binary, perms)?;
        }

        info!("Installed binary to {:?}", target_binary);
        Ok(())
    }

    /// Install from tarball package
    async fn install_tarball(&self, package_path: &PathBuf) -> Result<()> {
        let install_dir = self.get_installation_directory();

        // Extract tarball to installation directory
        let output = Command::new("tar")
            .args(["-xzf", package_path.to_str().unwrap_or_default()])
            .arg("-C")
            .arg(install_dir.to_str().unwrap_or_default())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to extract tarball: {}", stderr);
        }

        info!("Extracted tarball to {:?}", install_dir);
        Ok(())
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

    async fn install_update(&self, package_path: &PathBuf) -> Result<()> {
        info!("Installing update on Linux: {:?}", package_path);

        // Determine package type by extension
        let extension = package_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension {
            "tar" | "gz" | "tgz" => {
                self.install_tarball(package_path).await?;
            }
            _ => {
                // Assume it's a binary
                self.install_binary(package_path).await?;
            }
        }

        Ok(())
    }

    async fn restart_application(&self) -> Result<()> {
        info!("Restarting application on Linux");

        let current_exe = self.get_current_executable()?;

        // Try systemd first if available
        if self.is_systemd_available() {
            let output = Command::new("systemctl")
                .args(["--user", "restart", "inferno"])
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    info!("Restarted via systemd");
                    return Ok(());
                }
            }
            debug!("systemd restart not available, spawning new process");
        }

        // Spawn new process and exit current
        info!("Spawning new process: {:?}", current_exe);
        Command::new(&current_exe)
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn new process: {}", e))?;

        // Signal that the current process should exit
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
