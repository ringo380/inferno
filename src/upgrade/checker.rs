#![allow(dead_code, unused_imports, unused_variables)]
//! # Update Checker Service
//!
//! Background service that periodically checks for application updates from
//! various sources (GitHub releases, custom update servers, etc.).

use super::config::UpdateSource;
use super::{
    ApplicationVersion, UpdateChannel, UpdateInfo, UpgradeConfig, UpgradeError, UpgradeResult,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{debug, info, warn};

/// Update checker service for background update checking
pub struct UpdateChecker {
    config: UpgradeConfig,
    http_client: Client,
    last_check: Option<DateTime<Utc>>,
}

/// GitHub release information
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    body: String,
    published_at: String,
    prerelease: bool,
    draft: bool,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    content_type: String,
}

/// Custom update server response format
#[derive(Debug, Serialize, Deserialize)]
struct UpdateServerResponse {
    latest_version: String,
    releases: Vec<ReleaseInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReleaseInfo {
    version: String,
    release_date: String,
    changelog: String,
    downloads: HashMap<String, DownloadInfo>,
    is_critical: bool,
    is_security_update: bool,
    minimum_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DownloadInfo {
    url: String,
    checksum: String,
    signature: Option<String>,
    size: u64,
}

impl UpdateChecker {
    /// Create a new update checker
    pub async fn new(config: &UpgradeConfig) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(format!(
                "Inferno/{} ({})",
                ApplicationVersion::current().to_string(),
                std::env::consts::OS
            ))
            .build()?;

        Ok(Self {
            config: config.clone(),
            http_client,
            last_check: None,
        })
    }

    /// Check for available updates
    pub async fn check_for_updates(
        &mut self,
        current_version: &ApplicationVersion,
    ) -> UpgradeResult<Option<UpdateInfo>> {
        info!(
            "Checking for updates (current version: {})",
            current_version.to_string()
        );

        self.last_check = Some(Utc::now());

        // Check based on configured update source
        match &self.config.update_source {
            UpdateSource::GitHub { owner, repo } => {
                self.check_github_releases(owner, repo, current_version)
                    .await
            }
            UpdateSource::Custom { url } => self.check_custom_server(url, current_version).await,
            UpdateSource::Disabled => {
                debug!("Update checking is disabled");
                Ok(None)
            }
        }
    }

    /// Start periodic update checking in the background
    pub async fn start_periodic_checking(
        &mut self,
        current_version: ApplicationVersion,
    ) -> Result<()> {
        if !self.config.auto_check {
            info!("Automatic update checking is disabled");
            return Ok(());
        }

        let check_interval = self.config.check_interval;
        info!(
            "Starting periodic update checking every {:?}",
            check_interval
        );

        let mut interval_timer = interval(check_interval);

        loop {
            interval_timer.tick().await;

            match self.check_for_updates(&current_version).await {
                Ok(Some(update_info)) => {
                    info!(
                        "Update available: {} -> {}",
                        current_version.to_string(),
                        update_info.version.to_string()
                    );

                    // If auto-install is enabled and it's not a critical update requiring confirmation
                    if self.config.auto_install && !update_info.is_critical {
                        info!("Auto-installing update");
                        // Note: Auto-installation would be handled by the UpgradeManager
                    }
                }
                Ok(None) => {
                    debug!("No updates available");
                }
                Err(e) => {
                    warn!("Update check failed: {}", e);

                    // Exponential backoff on failures
                    sleep(Duration::from_secs(300)).await; // 5 minutes
                }
            }
        }
    }

    /// Check GitHub releases for updates
    async fn check_github_releases(
        &self,
        owner: &str,
        repo: &str,
        current_version: &ApplicationVersion,
    ) -> UpgradeResult<Option<UpdateInfo>> {
        let url = format!("https://api.github.com/repos/{}/{}/releases", owner, repo);

        debug!("Checking GitHub releases: {}", url);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| UpgradeError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(UpgradeError::NetworkError(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let releases: Vec<GitHubRelease> = response
            .json()
            .await
            .map_err(|e| UpgradeError::NetworkError(e.to_string()))?;

        // Filter releases based on channel
        let filtered_releases = self.filter_releases_by_channel(&releases);

        // Find the latest applicable release
        for release in filtered_releases {
            if let Ok(release_version) = self.parse_github_version(&release.tag_name) {
                if release_version.is_newer_than(current_version) {
                    return Ok(Some(
                        self.create_update_info_from_github(release, release_version)?,
                    ));
                }
            }
        }

        Ok(None)
    }

    /// Check custom update server for updates
    async fn check_custom_server(
        &self,
        server_url: &str,
        current_version: &ApplicationVersion,
    ) -> UpgradeResult<Option<UpdateInfo>> {
        let url = format!("{}/api/v1/updates", server_url);

        debug!("Checking custom update server: {}", url);

        let response = self
            .http_client
            .get(&url)
            .query(&[
                ("current_version", current_version.to_string()),
                ("channel", self.config.update_channel.as_str().to_string()),
                ("platform", std::env::consts::OS.to_string()),
            ])
            .send()
            .await
            .map_err(|e| UpgradeError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(UpgradeError::NetworkError(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let update_response: UpdateServerResponse = response
            .json()
            .await
            .map_err(|e| UpgradeError::NetworkError(e.to_string()))?;

        // Find the latest version newer than current
        for release in update_response.releases {
            if let Ok(release_version) = self.parse_version_string(&release.version) {
                if release_version.is_newer_than(current_version) {
                    return Ok(Some(
                        self.create_update_info_from_custom(&release, release_version)?,
                    ));
                }
            }
        }

        Ok(None)
    }

    /// Filter GitHub releases based on update channel
    fn filter_releases_by_channel<'a>(
        &self,
        releases: &'a [GitHubRelease],
    ) -> Vec<&'a GitHubRelease> {
        releases
            .iter()
            .filter(|release| {
                // Skip drafts
                if release.draft {
                    return false;
                }

                match &self.config.update_channel {
                    UpdateChannel::Stable => !release.prerelease,
                    UpdateChannel::Beta => true, // Include both stable and pre-release
                    UpdateChannel::Nightly => true, // Include all releases
                    UpdateChannel::Custom(_) => true, // Include all, let custom logic handle filtering
                }
            })
            .collect()
    }

    /// Parse GitHub version tag (e.g., "v1.2.3" or "1.2.3")
    fn parse_github_version(&self, tag: &str) -> Result<ApplicationVersion> {
        let version_str = tag.strip_prefix('v').unwrap_or(tag);
        self.parse_version_string(version_str)
    }

    /// Parse version string into ApplicationVersion
    fn parse_version_string(&self, version_str: &str) -> Result<ApplicationVersion> {
        // Split on '-' first to separate version from pre-release (e.g., "1.2.3-beta.1")
        let (version_part, pre_release) = if let Some(dash_pos) = version_str.find('-') {
            (
                &version_str[..dash_pos],
                Some(version_str[dash_pos + 1..].to_string()),
            )
        } else {
            (version_str, None)
        };

        let parts: Vec<&str> = version_part.split('.').collect();

        if parts.len() < 3 {
            return Err(anyhow::anyhow!("Invalid version format: {}", version_str));
        }

        let major = parts[0].parse::<u32>()?;
        let minor = parts[1].parse::<u32>()?;
        let patch = parts[2].parse::<u32>()?;

        Ok(ApplicationVersion {
            major,
            minor,
            patch,
            pre_release,
            build_metadata: None,
            build_date: None,
            git_commit: None,
        })
    }

    /// Create UpdateInfo from GitHub release
    fn create_update_info_from_github(
        &self,
        release: &GitHubRelease,
        version: ApplicationVersion,
    ) -> UpgradeResult<UpdateInfo> {
        let release_date = DateTime::parse_from_rfc3339(&release.published_at)
            .map_err(|e| UpgradeError::InvalidPackage(format!("Invalid release date: {}", e)))?
            .with_timezone(&Utc);

        let mut download_urls = HashMap::new();
        let mut checksums = HashMap::new();
        let mut size_bytes = HashMap::new();

        // Process GitHub assets to find platform-specific downloads
        for asset in &release.assets {
            if let Some(platform) = self.detect_platform_from_filename(&asset.name) {
                download_urls.insert(platform.clone(), asset.browser_download_url.clone());
                size_bytes.insert(platform.clone(), asset.size);

                // GitHub doesn't provide checksums directly, would need to be in a separate file
                // For now, we'll mark as empty and rely on HTTPS integrity
                checksums.insert(platform, String::new());
            }
        }

        Ok(UpdateInfo {
            version,
            release_date,
            changelog: release.body.clone(),
            download_urls,
            checksums,
            signatures: HashMap::new(), // Would need separate signature files
            size_bytes,
            is_critical: false, // GitHub doesn't provide this info
            is_security_update: release.body.to_lowercase().contains("security"),
            minimum_version: None,
            deprecation_warnings: Vec::new(),
        })
    }

    /// Create UpdateInfo from custom server response
    fn create_update_info_from_custom(
        &self,
        release: &ReleaseInfo,
        version: ApplicationVersion,
    ) -> UpgradeResult<UpdateInfo> {
        let release_date = DateTime::parse_from_rfc3339(&release.release_date)
            .map_err(|e| UpgradeError::InvalidPackage(format!("Invalid release date: {}", e)))?
            .with_timezone(&Utc);

        let mut download_urls = HashMap::new();
        let mut checksums = HashMap::new();
        let mut signatures = HashMap::new();
        let mut size_bytes = HashMap::new();

        for (platform, download_info) in &release.downloads {
            download_urls.insert(platform.clone(), download_info.url.clone());
            checksums.insert(platform.clone(), download_info.checksum.clone());
            size_bytes.insert(platform.clone(), download_info.size);

            if let Some(sig) = &download_info.signature {
                signatures.insert(platform.clone(), sig.clone());
            }
        }

        let minimum_version = if let Some(min_ver_str) = &release.minimum_version {
            Some(self.parse_version_string(min_ver_str).map_err(|e| {
                UpgradeError::InvalidPackage(format!("Invalid minimum version: {}", e))
            })?)
        } else {
            None
        };

        Ok(UpdateInfo {
            version,
            release_date,
            changelog: release.changelog.clone(),
            download_urls,
            checksums,
            signatures,
            size_bytes,
            is_critical: release.is_critical,
            is_security_update: release.is_security_update,
            minimum_version,
            deprecation_warnings: Vec::new(),
        })
    }

    /// Detect platform from filename
    fn detect_platform_from_filename(&self, filename: &str) -> Option<String> {
        let filename_lower = filename.to_lowercase();

        if filename_lower.contains("macos") || filename_lower.contains("darwin") {
            Some("macos".to_string())
        } else if filename_lower.contains("linux") {
            Some("linux".to_string())
        } else if filename_lower.contains("windows") || filename_lower.contains("win") {
            Some("windows".to_string())
        } else {
            None
        }
    }

    /// Get time since last check
    pub fn time_since_last_check(&self) -> Option<chrono::Duration> {
        self.last_check.map(|last| Utc::now() - last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_version_parsing() {
        let checker = UpdateChecker::new(&UpgradeConfig::default()).await.unwrap();

        let version = checker.parse_version_string("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, None);

        let pre_release = checker.parse_version_string("1.2.3-beta.1").unwrap();
        assert_eq!(pre_release.pre_release, Some("beta.1".to_string()));
    }

    #[tokio::test]
    async fn test_github_version_parsing() {
        let checker = UpdateChecker::new(&UpgradeConfig::default()).await.unwrap();

        let version = checker.parse_github_version("v1.2.3").unwrap();
        assert_eq!(version.major, 1);

        let without_v = checker.parse_github_version("1.2.3").unwrap();
        assert_eq!(without_v.major, 1);
    }

    #[test]
    fn test_platform_detection() {
        let checker =
            futures::executor::block_on(UpdateChecker::new(&UpgradeConfig::default())).unwrap();

        assert_eq!(
            checker.detect_platform_from_filename("inferno-macos.tar.gz"),
            Some("macos".to_string())
        );
        assert_eq!(
            checker.detect_platform_from_filename("inferno-linux.tar.gz"),
            Some("linux".to_string())
        );
        assert_eq!(
            checker.detect_platform_from_filename("inferno-windows.exe"),
            Some("windows".to_string())
        );
        assert_eq!(checker.detect_platform_from_filename("inferno.txt"), None);
    }

    #[test]
    fn test_release_filtering() {
        let checker =
            futures::executor::block_on(UpdateChecker::new(&UpgradeConfig::default())).unwrap();

        let releases = vec![
            GitHubRelease {
                tag_name: "v1.0.0".to_string(),
                name: "Release 1.0.0".to_string(),
                body: "Stable release".to_string(),
                published_at: "2023-01-01T00:00:00Z".to_string(),
                prerelease: false,
                draft: false,
                assets: vec![],
            },
            GitHubRelease {
                tag_name: "v1.1.0-beta.1".to_string(),
                name: "Beta 1.1.0".to_string(),
                body: "Beta release".to_string(),
                published_at: "2023-02-01T00:00:00Z".to_string(),
                prerelease: true,
                draft: false,
                assets: vec![],
            },
            GitHubRelease {
                tag_name: "v1.2.0".to_string(),
                name: "Draft 1.2.0".to_string(),
                body: "Draft release".to_string(),
                published_at: "2023-03-01T00:00:00Z".to_string(),
                prerelease: false,
                draft: true,
                assets: vec![],
            },
        ];

        let stable_filtered = checker.filter_releases_by_channel(&releases);
        assert_eq!(stable_filtered.len(), 1); // Only stable, non-draft release
        assert_eq!(stable_filtered[0].tag_name, "v1.0.0");
    }
}
