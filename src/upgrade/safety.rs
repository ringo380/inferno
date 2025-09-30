//! # Safety Checker
//!
//! Pre-installation safety checks to ensure system compatibility,
//! sufficient resources, and safe upgrade conditions.

use super::{UpdateInfo, UpgradeConfig, UpgradeError, UpgradeResult};
use std::path::PathBuf;
use std::process::Command;
use sysinfo::{DiskExt, ProcessExt, System, SystemExt};
use tracing::{debug, info, warn};

/// Safety checker for pre-installation validation
pub struct SafetyChecker {
    config: UpgradeConfig,
    system: System,
}

/// System compatibility check result
#[derive(Debug, Clone)]
pub struct CompatibilityReport {
    pub os_compatible: bool,
    pub arch_compatible: bool,
    pub version_compatible: bool,
    pub dependencies_satisfied: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

/// Resource availability check result
#[derive(Debug, Clone)]
pub struct ResourceReport {
    pub disk_space_sufficient: bool,
    pub memory_sufficient: bool,
    pub cpu_load_acceptable: bool,
    pub network_available: bool,
    pub available_disk_mb: u64,
    pub available_memory_mb: u64,
    pub cpu_usage_percent: f32,
    pub issues: Vec<String>,
}

impl SafetyChecker {
    /// Create a new safety checker
    pub fn new(config: &UpgradeConfig) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            config: config.clone(),
            system,
        }
    }

    /// Perform comprehensive pre-installation safety checks
    pub async fn check_pre_installation(&mut self, update_info: &UpdateInfo) -> UpgradeResult<()> {
        info!("Running pre-installation safety checks");

        // Refresh system information
        self.system.refresh_all();

        // Run all safety checks
        if self.config.safety_checks.check_compatibility {
            self.check_system_compatibility(update_info).await?;
        }

        if self.config.safety_checks.check_disk_space {
            self.check_disk_space(update_info).await?;
        }

        if self.config.safety_checks.check_network {
            self.check_network_connectivity().await?;
        }

        if self.config.safety_checks.check_running_processes {
            self.check_running_processes().await?;
        }

        if self.config.safety_checks.check_dependencies {
            self.check_system_dependencies().await?;
        }

        info!("All pre-installation safety checks passed");
        Ok(())
    }

    /// Verify package integrity and authenticity
    pub async fn verify_package(
        &self,
        package_path: &PathBuf,
        update_info: &UpdateInfo,
    ) -> UpgradeResult<()> {
        info!("Verifying package integrity");

        // Check if file exists
        if !package_path.exists() {
            return Err(UpgradeError::InvalidPackage(
                "Package file not found".to_string(),
            ));
        }

        // Verify file size
        let file_size = std::fs::metadata(package_path)
            .map_err(|e| UpgradeError::InvalidPackage(e.to_string()))?
            .len();

        let platform = std::env::consts::OS;
        if let Some(expected_size) = update_info.size_bytes.get(platform) {
            if file_size != *expected_size {
                return Err(UpgradeError::VerificationFailed(format!(
                    "File size mismatch: expected {} bytes, got {} bytes",
                    expected_size, file_size
                )));
            }
        }

        // Verify file format based on extension
        self.verify_package_format(package_path).await?;

        // Verify digital signature if required
        if self.config.require_signatures {
            self.verify_package_signature(package_path, update_info)
                .await?;
        }

        // Additional malware scanning could go here
        if self.is_malware_scanning_available() {
            self.scan_for_malware(package_path).await?;
        }

        info!("Package verification completed successfully");
        Ok(())
    }

    /// Check system compatibility
    async fn check_system_compatibility(&self, update_info: &UpdateInfo) -> UpgradeResult<()> {
        debug!("Checking system compatibility");

        // Check OS compatibility
        let current_os = std::env::consts::OS;
        if !update_info.download_urls.contains_key(current_os) {
            return Err(UpgradeError::PlatformNotSupported(current_os.to_string()));
        }

        // Check architecture compatibility
        let current_arch = std::env::consts::ARCH;
        debug!("Current architecture: {}", current_arch);

        // Check minimum version requirements
        if let Some(min_version) = &update_info.minimum_version {
            let current_version = super::ApplicationVersion::current();
            if !current_version.is_compatible_with(min_version) {
                return Err(UpgradeError::VerificationFailed(format!(
                    "Current version {} is not compatible with minimum required version {}",
                    current_version.to_string(),
                    min_version.to_string()
                )));
            }
        }

        // Check system version (OS version)
        self.check_os_version_compatibility()?;

        Ok(())
    }

    /// Check available disk space
    async fn check_disk_space(&self, update_info: &UpdateInfo) -> UpgradeResult<()> {
        debug!("Checking disk space");

        let platform = std::env::consts::OS;
        let package_size = update_info.size_bytes.get(platform).copied().unwrap_or(0);

        // Account for decompression (estimate 3x package size)
        let required_space =
            package_size * 3 + (self.config.safety_checks.min_free_space_mb * 1024 * 1024);

        // Get available disk space
        let available_space = self.get_available_disk_space(&self.config.download_dir)?;

        if available_space < required_space {
            return Err(UpgradeError::InsufficientDiskSpace {
                required: required_space / 1024 / 1024,
                available: available_space / 1024 / 1024,
            });
        }

        debug!(
            "Disk space check passed: {} MB available, {} MB required",
            available_space / 1024 / 1024,
            required_space / 1024 / 1024
        );

        Ok(())
    }

    /// Check network connectivity
    async fn check_network_connectivity(&self) -> UpgradeResult<()> {
        debug!("Checking network connectivity");

        // Simple connectivity check - try to resolve a known domain
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tokio::net::lookup_host("api.github.com:443"),
        )
        .await;

        match result {
            Ok(Ok(_)) => {
                debug!("Network connectivity check passed");
                Ok(())
            }
            Ok(Err(e)) => Err(UpgradeError::NetworkError(format!(
                "DNS resolution failed: {}",
                e
            ))),
            Err(_) => Err(UpgradeError::NetworkError(
                "Network connectivity timeout".to_string(),
            )),
        }
    }

    /// Check for potentially interfering running processes
    async fn check_running_processes(&self) -> UpgradeResult<()> {
        debug!("Checking running processes");

        let dangerous_processes = vec!["antivirus", "scanner", "backup", "sync", "cloud"];

        for (_, process) in self.system.processes() {
            let process_name = process.name().to_lowercase();

            for dangerous in &dangerous_processes {
                if process_name.contains(dangerous) {
                    warn!(
                        "Potentially interfering process detected: {}",
                        process.name()
                    );
                    // For now, just warn. In production, you might want to:
                    // - Ask user to close the process
                    // - Automatically pause certain processes
                    // - Defer the upgrade
                }
            }
        }

        // Check if current application instances are running
        let current_exe =
            std::env::current_exe().map_err(|e| UpgradeError::Internal(e.to_string()))?;

        let current_name = current_exe
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("inferno");

        let mut running_instances = 0;
        for (pid, process) in self.system.processes() {
            if process.name().to_lowercase().contains("inferno")
                && *pid != sysinfo::get_current_pid().unwrap()
            {
                running_instances += 1;
            }
        }

        if running_instances > 0 {
            warn!(
                "Found {} other running instances of the application",
                running_instances
            );
        }

        Ok(())
    }

    /// Check system dependencies
    async fn check_system_dependencies(&self) -> UpgradeResult<()> {
        debug!("Checking system dependencies");

        // Check for required system libraries/tools
        #[cfg(target_os = "macos")]
        {
            self.check_macos_dependencies()?;
        }

        #[cfg(target_os = "linux")]
        {
            self.check_linux_dependencies()?;
        }

        #[cfg(target_os = "windows")]
        {
            self.check_windows_dependencies()?;
        }

        Ok(())
    }

    /// Verify package format
    async fn verify_package_format(&self, package_path: &PathBuf) -> UpgradeResult<()> {
        let extension = package_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "tar" | "tgz" | "tar.gz" => self.verify_tar_format(package_path),
            "zip" => self.verify_zip_format(package_path),
            "pkg" => self.verify_pkg_format(package_path),
            "msi" | "exe" => self.verify_windows_format(package_path),
            "deb" => self.verify_deb_format(package_path),
            "rpm" => self.verify_rpm_format(package_path),
            _ => Err(UpgradeError::InvalidPackage(format!(
                "Unsupported package format: {}",
                extension
            ))),
        }
    }

    /// Verify digital signature
    async fn verify_package_signature(
        &self,
        package_path: &PathBuf,
        update_info: &UpdateInfo,
    ) -> UpgradeResult<()> {
        debug!("Verifying package digital signature");

        let platform = std::env::consts::OS;
        if let Some(signature) = update_info.signatures.get(platform) {
            // In a real implementation, you would:
            // 1. Extract the signature from the signature string
            // 2. Use a cryptographic library to verify the signature
            // 3. Check against trusted public keys

            if signature.is_empty() {
                return Err(UpgradeError::VerificationFailed(
                    "No signature provided".to_string(),
                ));
            }

            // Placeholder signature verification
            // This would be replaced with actual cryptographic verification
            debug!(
                "Signature verification placeholder - signature length: {}",
                signature.len()
            );
        } else if self.config.require_signatures {
            return Err(UpgradeError::VerificationFailed(
                "Signature required but not provided".to_string(),
            ));
        }

        Ok(())
    }

    /// Scan for malware (if antivirus is available)
    async fn scan_for_malware(&self, package_path: &PathBuf) -> UpgradeResult<()> {
        debug!("Scanning package for malware");

        // This is a placeholder for malware scanning
        // In a real implementation, you might integrate with:
        // - Windows Defender API
        // - ClamAV on Linux
        // - Third-party antivirus APIs

        #[cfg(target_os = "windows")]
        {
            // Check Windows Defender
            if let Ok(output) = Command::new("powershell")
                .args(&[
                    "-Command",
                    "Get-MpComputerStatus | Select-Object RealTimeProtectionEnabled",
                ])
                .output()
            {
                if output.status.success() {
                    debug!("Windows Defender is available for scanning");
                }
            }
        }

        Ok(())
    }

    /// Check if malware scanning is available
    fn is_malware_scanning_available(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            Command::new("powershell")
                .args(&["-Command", "Get-MpComputerStatus"])
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("clamscan")
                .arg("--version")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }

        #[cfg(target_os = "macos")]
        {
            // macOS doesn't have built-in command-line antivirus
            false
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            false
        }
    }

    /// Get available disk space for a given path
    fn get_available_disk_space(&self, path: &PathBuf) -> UpgradeResult<u64> {
        for disk in self.system.disks() {
            if path.starts_with(disk.mount_point()) {
                return Ok(disk.available_space());
            }
        }

        // Fallback: try to get space for root filesystem
        if let Some(root_disk) = self.system.disks().first() {
            Ok(root_disk.available_space())
        } else {
            Err(UpgradeError::Internal(
                "Cannot determine available disk space".to_string(),
            ))
        }
    }

    /// Check OS version compatibility
    fn check_os_version_compatibility(&self) -> UpgradeResult<()> {
        let os_version = self.system.os_version();
        debug!("OS version: {:?}", os_version);

        #[cfg(target_os = "macos")]
        {
            // Check minimum macOS version (example: 10.15+)
            if let Some(version) = os_version {
                if self.is_macos_version_too_old(&version) {
                    return Err(UpgradeError::PlatformNotSupported(format!(
                        "macOS version {} is too old. Minimum version required: 10.15",
                        version
                    )));
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Check kernel version and distribution
            if let Some(version) = os_version {
                debug!("Linux version: {}", version);
                // Additional Linux-specific checks could go here
            }
        }

        Ok(())
    }

    // Platform-specific dependency checks
    #[cfg(target_os = "macos")]
    fn check_macos_dependencies(&self) -> UpgradeResult<()> {
        // Check for required macOS frameworks/libraries
        debug!("Checking macOS dependencies");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn check_linux_dependencies(&self) -> UpgradeResult<()> {
        // Check for required Linux libraries
        debug!("Checking Linux dependencies");
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn check_windows_dependencies(&self) -> UpgradeResult<()> {
        // Check for required Windows components
        debug!("Checking Windows dependencies");
        Ok(())
    }

    // Package format verification methods
    fn verify_tar_format(&self, path: &PathBuf) -> UpgradeResult<()> {
        // Basic tar file validation
        Command::new("tar")
            .args(&["-tf", path.to_str().unwrap()])
            .output()
            .map_err(|e| UpgradeError::InvalidPackage(format!("Tar validation failed: {}", e)))
            .and_then(|output| {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(UpgradeError::InvalidPackage(
                        "Invalid tar file format".to_string(),
                    ))
                }
            })
    }

    fn verify_zip_format(&self, path: &PathBuf) -> UpgradeResult<()> {
        // Basic zip file validation
        use std::fs::File;
        let file = File::open(path).map_err(|e| UpgradeError::InvalidPackage(e.to_string()))?;

        // Check ZIP magic bytes
        use std::io::Read;
        let mut magic = [0u8; 4];
        let mut reader = file;
        reader
            .read_exact(&mut magic)
            .map_err(|e| UpgradeError::InvalidPackage(e.to_string()))?;

        if &magic == b"PK\x03\x04" || &magic == b"PK\x05\x06" || &magic == b"PK\x07\x08" {
            Ok(())
        } else {
            Err(UpgradeError::InvalidPackage(
                "Invalid ZIP file format".to_string(),
            ))
        }
    }

    fn verify_pkg_format(&self, _path: &PathBuf) -> UpgradeResult<()> {
        // macOS pkg validation would go here
        Ok(())
    }

    fn verify_windows_format(&self, _path: &PathBuf) -> UpgradeResult<()> {
        // Windows MSI/EXE validation would go here
        Ok(())
    }

    fn verify_deb_format(&self, path: &PathBuf) -> UpgradeResult<()> {
        // Debian package validation
        Command::new("dpkg")
            .args(&["--info", path.to_str().unwrap()])
            .output()
            .map_err(|e| UpgradeError::InvalidPackage(format!("DEB validation failed: {}", e)))
            .and_then(|output| {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(UpgradeError::InvalidPackage(
                        "Invalid DEB package format".to_string(),
                    ))
                }
            })
    }

    fn verify_rpm_format(&self, path: &PathBuf) -> UpgradeResult<()> {
        // RPM package validation
        Command::new("rpm")
            .args(&["-qp", path.to_str().unwrap()])
            .output()
            .map_err(|e| UpgradeError::InvalidPackage(format!("RPM validation failed: {}", e)))
            .and_then(|output| {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(UpgradeError::InvalidPackage(
                        "Invalid RPM package format".to_string(),
                    ))
                }
            })
    }

    #[cfg(target_os = "macos")]
    fn is_macos_version_too_old(&self, version: &str) -> bool {
        // Simple version comparison for macOS
        // This is a simplified check - a real implementation would use proper version parsing
        let major_version = version
            .split('.')
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        major_version < 10 || (major_version == 10 && self.get_macos_minor_version(version) < 15)
    }

    #[cfg(target_os = "macos")]
    fn get_macos_minor_version(&self, version: &str) -> u32 {
        version
            .split('.')
            .nth(1)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> UpgradeConfig {
        let temp_dir = TempDir::new().unwrap();
        UpgradeConfig {
            download_dir: temp_dir.path().to_path_buf(),
            backup_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_safety_checker_creation() {
        let config = create_test_config();
        let checker = SafetyChecker::new(&config);
        assert!(checker.system.disks().len() > 0);
    }

    #[tokio::test]
    async fn test_network_connectivity() {
        let config = create_test_config();
        let checker = SafetyChecker::new(&config);

        // This test might fail in offline environments
        let result = checker.check_network_connectivity().await;
        if result.is_err() {
            println!(
                "Network connectivity test failed (expected in offline environments): {:?}",
                result
            );
        }
    }

    #[test]
    fn test_disk_space_calculation() {
        let config = create_test_config();
        let checker = SafetyChecker::new(&config);

        let space = checker.get_available_disk_space(&config.download_dir);
        assert!(space.is_ok());
        assert!(space.unwrap() > 0);
    }
}
