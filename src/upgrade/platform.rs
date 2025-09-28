//! # Platform-Specific Upgrade Handlers
//!
//! Platform abstraction layer for handling upgrades across different operating systems
//! with native installation methods and system integration.

use super::{UpgradeConfig, UpgradeError, UpgradeResult};
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info, warn};

/// Platform detection utilities
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub version: String,
    pub distribution: Option<String>, // For Linux distributions
}

impl PlatformInfo {
    /// Detect current platform information
    pub fn detect() -> Result<Self> {
        let os = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();

        let version = Self::get_os_version()?;
        let distribution = Self::get_distribution();

        Ok(Self {
            os,
            arch,
            version,
            distribution,
        })
    }

    /// Get OS version string
    fn get_os_version() -> Result<String> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("sw_vers")
                .arg("-productVersion")
                .output()?;

            if output.status.success() {
                Ok(String::from_utf8(output.stdout)?.trim().to_string())
            } else {
                Ok("unknown".to_string())
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Try to get kernel version
            let output = Command::new("uname")
                .arg("-r")
                .output()?;

            if output.status.success() {
                Ok(String::from_utf8(output.stdout)?.trim().to_string())
            } else {
                Ok("unknown".to_string())
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Use PowerShell to get Windows version
            let output = Command::new("powershell")
                .args(&["-Command", "(Get-CimInstance Win32_OperatingSystem).Version"])
                .output()?;

            if output.status.success() {
                Ok(String::from_utf8(output.stdout)?.trim().to_string())
            } else {
                Ok("unknown".to_string())
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Ok("unknown".to_string())
        }
    }

    /// Get Linux distribution name
    fn get_distribution() -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            // Try to read /etc/os-release
            if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
                for line in content.lines() {
                    if line.starts_with("ID=") {
                        return Some(line.trim_start_matches("ID=").trim_matches('"').to_string());
                    }
                }
            }

            // Fallback to lsb_release
            if let Ok(output) = Command::new("lsb_release").arg("-si").output() {
                if output.status.success() {
                    return Some(String::from_utf8(output.stdout).ok()?.trim().to_string());
                }
            }
        }

        None
    }

    /// Check if the platform supports a specific installation method
    pub fn supports_installation_method(&self, method: &InstallationMethod) -> bool {
        match method {
            InstallationMethod::SelfExtractor => true, // All platforms support this
            InstallationMethod::SystemPackage => match self.os.as_str() {
                "linux" => true,
                "macos" => true,
                "windows" => true,
                _ => false,
            },
            InstallationMethod::AppBundle => self.os == "macos",
            InstallationMethod::MSI => self.os == "windows",
            InstallationMethod::DEB => {
                self.os == "linux" &&
                self.distribution.as_ref().map_or(false, |d|
                    d.contains("ubuntu") || d.contains("debian")
                )
            },
            InstallationMethod::RPM => {
                self.os == "linux" &&
                self.distribution.as_ref().map_or(false, |d|
                    d.contains("fedora") || d.contains("centos") || d.contains("rhel")
                )
            },
            InstallationMethod::Snap => self.os == "linux",
            InstallationMethod::Flatpak => self.os == "linux",
            InstallationMethod::Homebrew => self.os == "macos" || self.os == "linux",
            InstallationMethod::Winget => self.os == "windows",
        }
    }
}

/// Available installation methods
#[derive(Debug, Clone)]
pub enum InstallationMethod {
    /// Self-extracting archive (works on all platforms)
    SelfExtractor,
    /// Native system package
    SystemPackage,
    /// macOS App Bundle
    AppBundle,
    /// Windows MSI installer
    MSI,
    /// Debian/Ubuntu package
    DEB,
    /// Red Hat/Fedora package
    RPM,
    /// Snap package (Linux)
    Snap,
    /// Flatpak (Linux)
    Flatpak,
    /// Homebrew (macOS/Linux)
    Homebrew,
    /// Windows Package Manager
    Winget,
}

/// Base platform upgrade handler with common functionality
pub struct BasePlatformHandler {
    pub config: UpgradeConfig,
    pub platform_info: PlatformInfo,
    pub preferred_methods: Vec<InstallationMethod>,
}

impl BasePlatformHandler {
    /// Create a new base platform handler
    pub fn new(config: &UpgradeConfig) -> Result<Self> {
        let platform_info = PlatformInfo::detect()?;
        let preferred_methods = Self::get_preferred_installation_methods(&platform_info);

        Ok(Self {
            config: config.clone(),
            platform_info,
            preferred_methods,
        })
    }

    /// Get preferred installation methods for the current platform
    fn get_preferred_installation_methods(platform_info: &PlatformInfo) -> Vec<InstallationMethod> {
        match platform_info.os.as_str() {
            "macos" => vec![
                InstallationMethod::AppBundle,
                InstallationMethod::Homebrew,
                InstallationMethod::SelfExtractor,
            ],
            "linux" => {
                let mut methods = vec![];

                // Add distribution-specific methods first
                if let Some(distro) = &platform_info.distribution {
                    if distro.contains("ubuntu") || distro.contains("debian") {
                        methods.push(InstallationMethod::DEB);
                    } else if distro.contains("fedora") || distro.contains("centos") || distro.contains("rhel") {
                        methods.push(InstallationMethod::RPM);
                    }
                }

                // Add universal Linux methods
                methods.extend([
                    InstallationMethod::Snap,
                    InstallationMethod::Flatpak,
                    InstallationMethod::Homebrew,
                    InstallationMethod::SelfExtractor,
                ]);

                methods
            }
            "windows" => vec![
                InstallationMethod::MSI,
                InstallationMethod::Winget,
                InstallationMethod::SelfExtractor,
            ],
            _ => vec![InstallationMethod::SelfExtractor],
        }
    }

    /// Check if the platform supports seamless upgrades
    pub fn supports_seamless_upgrade(&self) -> bool {
        // Most platforms support some form of seamless upgrade
        match self.platform_info.os.as_str() {
            "macos" | "linux" | "windows" => true,
            _ => false,
        }
    }

    /// Get installation directory for the current platform
    pub fn get_installation_directory(&self) -> PathBuf {
        match self.platform_info.os.as_str() {
            "macos" => PathBuf::from("/Applications/Inferno.app"),
            "linux" => PathBuf::from("/usr/local/bin"),
            "windows" => PathBuf::from("C:\\Program Files\\Inferno"),
            _ => std::env::current_exe()
                .map(|exe| exe.parent().unwrap_or(&exe).to_path_buf())
                .unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    /// Get backup directory for the current platform
    pub fn get_backup_directory(&self) -> PathBuf {
        match self.platform_info.os.as_str() {
            "macos" => dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("Library/Application Support/Inferno/Backups"),
            "linux" => dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".local/share/inferno/backups"),
            "windows" => dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
                .join("Inferno\\Backups"),
            _ => PathBuf::from("./backups"),
        }
    }

    /// Check if administrator/root privileges are required
    pub fn requires_elevated_privileges(&self) -> bool {
        let installation_dir = self.get_installation_directory();

        // Check if we can write to the installation directory
        match std::fs::metadata(&installation_dir) {
            Ok(_) => {
                // Try to create a test file
                let test_file = installation_dir.join(".write_test");
                std::fs::write(&test_file, "test").is_err()
            }
            Err(_) => {
                // Directory doesn't exist, check parent
                if let Some(parent) = installation_dir.parent() {
                    let test_file = parent.join(".write_test");
                    std::fs::write(&test_file, "test").is_err()
                } else {
                    true // Conservative assumption
                }
            }
        }
    }

    /// Stop application services before upgrade
    pub async fn stop_services(&self) -> UpgradeResult<Vec<String>> {
        info!("Stopping application services");

        let mut stopped_services = vec![];

        // Stop any running instances of the application
        if let Err(e) = self.stop_running_instances().await {
            warn!("Failed to stop some running instances: {}", e);
        } else {
            stopped_services.push("inferno-instances".to_string());
        }

        // Platform-specific service stopping
        match self.platform_info.os.as_str() {
            "macos" => {
                if let Ok(services) = self.stop_macos_services().await {
                    stopped_services.extend(services);
                }
            }
            "linux" => {
                if let Ok(services) = self.stop_linux_services().await {
                    stopped_services.extend(services);
                }
            }
            "windows" => {
                if let Ok(services) = self.stop_windows_services().await {
                    stopped_services.extend(services);
                }
            }
            _ => {}
        }

        Ok(stopped_services)
    }

    /// Start application services after upgrade
    pub async fn start_services(&self, stopped_services: &[String]) -> UpgradeResult<()> {
        info!("Starting application services");

        // Platform-specific service starting
        match self.platform_info.os.as_str() {
            "macos" => self.start_macos_services(stopped_services).await?,
            "linux" => self.start_linux_services(stopped_services).await?,
            "windows" => self.start_windows_services(stopped_services).await?,
            _ => {}
        }

        Ok(())
    }

    /// Stop running application instances
    async fn stop_running_instances(&self) -> Result<()> {
        use sysinfo::{ProcessExt, System, SystemExt};

        let mut system = System::new_all();
        system.refresh_all();

        let current_pid = sysinfo::get_current_pid().unwrap();

        for (pid, process) in system.processes() {
            if *pid != current_pid && process.name().to_lowercase().contains("inferno") {
                info!("Stopping process: {} (PID: {})", process.name(), pid);

                #[cfg(unix)]
                {
                    let _ = Command::new("kill")
                        .arg("-TERM")
                        .arg(pid.to_string())
                        .output();
                }

                #[cfg(windows)]
                {
                    let _ = Command::new("taskkill")
                        .args(&["/PID", &pid.to_string(), "/F"])
                        .output();
                }
            }
        }

        // Wait a moment for processes to stop gracefully
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        Ok(())
    }

    // Platform-specific service management methods
    #[cfg(target_os = "macos")]
    async fn stop_macos_services(&self) -> Result<Vec<String>> {
        let mut stopped = vec![];

        // Check for launchd services
        if let Ok(output) = Command::new("launchctl")
            .args(&["list"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("inferno") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let service_name = parts[2];
                        debug!("Stopping macOS service: {}", service_name);

                        let _ = Command::new("launchctl")
                            .args(&["unload", service_name])
                            .output();

                        stopped.push(service_name.to_string());
                    }
                }
            }
        }

        Ok(stopped)
    }

    #[cfg(not(target_os = "macos"))]
    async fn stop_macos_services(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    #[cfg(target_os = "linux")]
    async fn stop_linux_services(&self) -> Result<Vec<String>> {
        let mut stopped = vec![];

        // Check for systemd services
        if let Ok(output) = Command::new("systemctl")
            .args(&["list-units", "--type=service", "--state=active"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("inferno") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if !parts.is_empty() {
                        let service_name = parts[0];
                        debug!("Stopping Linux service: {}", service_name);

                        let _ = Command::new("systemctl")
                            .args(&["stop", service_name])
                            .output();

                        stopped.push(service_name.to_string());
                    }
                }
            }
        }

        Ok(stopped)
    }

    #[cfg(not(target_os = "linux"))]
    async fn stop_linux_services(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    #[cfg(target_os = "windows")]
    async fn stop_windows_services(&self) -> Result<Vec<String>> {
        let mut stopped = vec![];

        // Check for Windows services
        if let Ok(output) = Command::new("sc")
            .args(&["query", "state=", "all"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("inferno") && line.contains("SERVICE_NAME:") {
                    if let Some(service_name) = line.split(':').nth(1) {
                        let service_name = service_name.trim();
                        debug!("Stopping Windows service: {}", service_name);

                        let _ = Command::new("sc")
                            .args(&["stop", service_name])
                            .output();

                        stopped.push(service_name.to_string());
                    }
                }
            }
        }

        Ok(stopped)
    }

    #[cfg(not(target_os = "windows"))]
    async fn stop_windows_services(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    #[cfg(target_os = "macos")]
    async fn start_macos_services(&self, stopped_services: &[String]) -> Result<()> {
        for service in stopped_services {
            if service != "inferno-instances" {
                debug!("Starting macOS service: {}", service);
                let _ = Command::new("launchctl")
                    .args(&["load", service])
                    .output();
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    async fn start_macos_services(&self, _stopped_services: &[String]) -> Result<()> {
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn start_linux_services(&self, stopped_services: &[String]) -> Result<()> {
        for service in stopped_services {
            if service != "inferno-instances" {
                debug!("Starting Linux service: {}", service);
                let _ = Command::new("systemctl")
                    .args(&["start", service])
                    .output();
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    async fn start_linux_services(&self, _stopped_services: &[String]) -> Result<()> {
        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn start_windows_services(&self, stopped_services: &[String]) -> Result<()> {
        for service in stopped_services {
            if service != "inferno-instances" {
                debug!("Starting Windows service: {}", service);
                let _ = Command::new("sc")
                    .args(&["start", service])
                    .output();
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    async fn start_windows_services(&self, _stopped_services: &[String]) -> Result<()> {
        Ok(())
    }

    /// Extract and install from a self-extracting archive
    pub async fn install_self_extractor(&self, package_path: &PathBuf) -> UpgradeResult<()> {
        info!("Installing from self-extracting archive: {:?}", package_path);

        // Extract to temporary directory
        let temp_dir = tempfile::TempDir::new()
            .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?;

        // Extract the archive
        self.extract_archive(package_path, temp_dir.path()).await?;

        // Find the main executable in the extracted content
        let executable = self.find_main_executable(temp_dir.path())?;

        // Install the executable
        self.install_executable(&executable).await?;

        Ok(())
    }

    /// Extract archive to destination
    async fn extract_archive(&self, archive_path: &PathBuf, dest_dir: &std::path::Path) -> UpgradeResult<()> {
        use flate2::read::GzDecoder;
        use std::fs::File;
        use tar::Archive;

        let file = File::open(archive_path)
            .map_err(|e| UpgradeError::InvalidPackage(e.to_string()))?;

        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        archive.unpack(dest_dir)
            .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?;

        Ok(())
    }

    /// Find the main executable in extracted content
    fn find_main_executable(&self, dir: &std::path::Path) -> UpgradeResult<PathBuf> {
        for entry in std::fs::read_dir(dir)
            .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?
        {
            let entry = entry.map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?;
            let path = entry.path();

            if path.is_file() {
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if filename.starts_with("inferno") {
                    return Ok(path);
                }
            }
        }

        Err(UpgradeError::InstallationFailed("Main executable not found in package".to_string()))
    }

    /// Install executable to the appropriate location
    async fn install_executable(&self, source_exe: &PathBuf) -> UpgradeResult<()> {
        let current_exe = std::env::current_exe()
            .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?;

        // Create backup of current executable
        let backup_exe = current_exe.with_extension("exe.backup");
        std::fs::copy(&current_exe, &backup_exe)
            .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?;

        // Replace current executable
        std::fs::copy(source_exe, &current_exe)
            .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&current_exe)
                .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&current_exe, perms)
                .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?;
        }

        Ok(())
    }

    /// Verify installation by checking executable
    pub async fn verify_installation(&self) -> UpgradeResult<bool> {
        let current_exe = std::env::current_exe()
            .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?;

        // Try to run the executable with --version
        let output = Command::new(&current_exe)
            .arg("--version")
            .output()
            .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?;

        Ok(output.status.success())
    }

    /// Clean up temporary files after upgrade
    pub async fn cleanup_after_upgrade(&self) -> UpgradeResult<()> {
        info!("Cleaning up after upgrade");

        // Remove backup executable if it exists
        let current_exe = std::env::current_exe()
            .map_err(|e| UpgradeError::InstallationFailed(e.to_string()))?;

        let backup_exe = current_exe.with_extension("exe.backup");
        if backup_exe.exists() {
            if let Err(e) = std::fs::remove_file(&backup_exe) {
                warn!("Failed to remove backup executable: {}", e);
            }
        }

        // Platform-specific cleanup
        match self.platform_info.os.as_str() {
            "macos" => self.cleanup_macos().await?,
            "linux" => self.cleanup_linux().await?,
            "windows" => self.cleanup_windows().await?,
            _ => {}
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn cleanup_macos(&self) -> UpgradeResult<()> {
        // Clean up any macOS-specific temporary files
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    async fn cleanup_macos(&self) -> UpgradeResult<()> {
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn cleanup_linux(&self) -> UpgradeResult<()> {
        // Clean up any Linux-specific temporary files
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    async fn cleanup_linux(&self) -> UpgradeResult<()> {
        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn cleanup_windows(&self) -> UpgradeResult<()> {
        // Clean up any Windows-specific temporary files
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    async fn cleanup_windows(&self) -> UpgradeResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = PlatformInfo::detect().unwrap();
        println!("Detected platform: {:?}", platform.os);
        println!("Architecture: {}", platform.arch);
        println!("Version: {}", platform.version);
        if let Some(distro) = &platform.distribution {
            println!("Distribution: {}", distro);
        }

        assert!(!platform.os.is_empty());
        assert!(!platform.arch.is_empty());
    }

    #[test]
    fn test_installation_methods() {
        let platform = PlatformInfo::detect().unwrap();
        let methods = BasePlatformHandler::get_preferred_installation_methods(&platform);

        assert!(!methods.is_empty());
        println!("Preferred installation methods: {:?}", methods);
    }

    #[tokio::test]
    async fn test_base_handler_creation() {
        let config = UpgradeConfig::default();
        let handler = BasePlatformHandler::new(&config);
        assert!(handler.is_ok());

        let handler = handler.unwrap();
        assert!(handler.supports_seamless_upgrade());
    }
}