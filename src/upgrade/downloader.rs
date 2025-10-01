//! # Update Downloader
//!
//! Secure download system for application updates with cryptographic verification,
//! progress tracking, and resume capabilities.

use super::{UpgradeConfig, UpgradeError, UpgradeResult};
use anyhow::Result;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Progress callback function type
pub type ProgressCallback = dyn Fn(u64, u64, u64) + Send + Sync;

/// Update downloader with secure verification and progress tracking
pub struct UpdateDownloader {
    config: UpgradeConfig,
    http_client: Client,
    download_dir: PathBuf,
    resume_enabled: bool,
}

/// Download session state for resume capability
#[derive(Debug)]
struct DownloadSession {
    url: String,
    file_path: PathBuf,
    expected_checksum: String,
    total_size: Option<u64>,
    downloaded_size: u64,
    start_time: Instant,
    should_cancel: Arc<AtomicBool>,
}

impl UpdateDownloader {
    /// Create a new update downloader
    pub fn new(config: &UpgradeConfig) -> Result<Self> {
        let download_dir = config.download_dir.clone();
        std::fs::create_dir_all(&download_dir)?;

        let http_client = Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minutes for large files
            .user_agent(format!(
                "Inferno/{} UpdateDownloader ({})",
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS
            ))
            .build()?;

        Ok(Self {
            config: config.clone(),
            http_client,
            download_dir,
            resume_enabled: true,
        })
    }

    /// Download an update package with verification
    pub async fn download_update<F>(
        &self,
        url: &str,
        expected_checksum: &str,
        progress_callback: F,
    ) -> UpgradeResult<PathBuf>
    where
        F: Fn(u64, u64, u64) + Send + Sync,
    {
        info!("Starting download: {}", url);

        // Create download session
        let filename = self.extract_filename_from_url(url)?;
        let file_path = self.download_dir.join(&filename);
        let should_cancel = Arc::new(AtomicBool::new(false));

        let mut session = DownloadSession {
            url: url.to_string(),
            file_path: file_path.clone(),
            expected_checksum: expected_checksum.to_string(),
            total_size: None,
            downloaded_size: 0,
            start_time: Instant::now(),
            should_cancel: Arc::clone(&should_cancel),
        };

        // Check for existing partial download
        if self.resume_enabled && file_path.exists() {
            session.downloaded_size = self.get_file_size(&file_path)?;
            info!("Resuming download from {} bytes", session.downloaded_size);
        }

        // Perform the download with retry logic
        let final_path = self
            .download_with_retry(&mut session, progress_callback)
            .await?;

        // Verify the downloaded file
        self.verify_download(&final_path, expected_checksum).await?;

        info!("Download completed and verified: {:?}", final_path);
        Ok(final_path)
    }

    /// Download with automatic retry on failure
    async fn download_with_retry<F>(
        &self,
        session: &mut DownloadSession,
        progress_callback: F,
    ) -> UpgradeResult<PathBuf>
    where
        F: Fn(u64, u64, u64) + Send + Sync,
    {
        let max_retries = 3;
        let mut retry_count = 0;

        loop {
            match self.perform_download(session, &progress_callback).await {
                Ok(path) => return Ok(path),
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        return Err(e);
                    }

                    warn!(
                        "Download failed (attempt {}/{}): {}",
                        retry_count, max_retries, e
                    );

                    // Exponential backoff
                    let delay = Duration::from_secs(2_u64.pow(retry_count as u32));
                    sleep(delay).await;

                    // Reset session for retry
                    session.downloaded_size = if session.file_path.exists() {
                        self.get_file_size(&session.file_path)?
                    } else {
                        0
                    };
                }
            }
        }
    }

    /// Perform the actual download
    async fn perform_download<F>(
        &self,
        session: &mut DownloadSession,
        progress_callback: &F,
    ) -> UpgradeResult<PathBuf>
    where
        F: Fn(u64, u64, u64) + Send + Sync,
    {
        // Build request with range header for resume
        let mut request = self.http_client.get(&session.url);
        if session.downloaded_size > 0 {
            request = request.header("Range", format!("bytes={}-", session.downloaded_size));
        }

        let response = request
            .send()
            .await
            .map_err(|e| UpgradeError::NetworkError(e.to_string()))?;

        // Handle response status
        if !response.status().is_success() && response.status().as_u16() != 206 {
            return Err(UpgradeError::NetworkError(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        // Get content length
        let content_length = response.content_length().unwrap_or(0);
        if session.total_size.is_none() {
            session.total_size = Some(session.downloaded_size + content_length);
        }

        let total_size = session.total_size.unwrap_or(0);

        // Open file for writing (append if resuming)
        let mut file = if session.downloaded_size > 0 {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&session.file_path)
                .map_err(|e| UpgradeError::InvalidPackage(e.to_string()))?
        } else {
            File::create(&session.file_path)
                .map_err(|e| UpgradeError::InvalidPackage(e.to_string()))?
        };

        // Download with progress tracking
        let mut bytes_stream = response.bytes_stream();
        let mut last_progress_update = Instant::now();
        let progress_update_interval = Duration::from_millis(100);

        use futures_util::StreamExt;
        while let Some(chunk) = bytes_stream.next().await {
            // Check for cancellation
            if session.should_cancel.load(Ordering::Relaxed) {
                return Err(UpgradeError::Cancelled);
            }

            let chunk = chunk.map_err(|e| UpgradeError::NetworkError(e.to_string()))?;

            // Write chunk to file
            file.write_all(&chunk)
                .map_err(|e| UpgradeError::InvalidPackage(e.to_string()))?;

            session.downloaded_size += chunk.len() as u64;

            // Update progress (throttled to avoid excessive callbacks)
            if last_progress_update.elapsed() >= progress_update_interval {
                let speed = self.calculate_download_speed(session);
                progress_callback(session.downloaded_size, total_size, speed);
                last_progress_update = Instant::now();
            }
        }

        // Final progress update
        let speed = self.calculate_download_speed(session);
        progress_callback(session.downloaded_size, total_size, speed);

        // Ensure file is flushed
        file.sync_all()
            .map_err(|e| UpgradeError::InvalidPackage(e.to_string()))?;

        Ok(session.file_path.clone())
    }

    /// Verify downloaded file integrity
    async fn verify_download(
        &self,
        file_path: &Path,
        expected_checksum: &str,
    ) -> UpgradeResult<()> {
        info!("Verifying download integrity");

        if expected_checksum.is_empty() {
            warn!("No checksum provided, skipping verification");
            return Ok(());
        }

        let calculated_checksum = self.calculate_file_checksum(file_path).await?;

        if calculated_checksum.to_lowercase() != expected_checksum.to_lowercase() {
            // Remove corrupted file
            if let Err(e) = std::fs::remove_file(file_path) {
                warn!("Failed to remove corrupted file: {}", e);
            }

            return Err(UpgradeError::VerificationFailed(format!(
                "Checksum mismatch: expected {}, got {}",
                expected_checksum, calculated_checksum
            )));
        }

        info!("Download verification successful");
        Ok(())
    }

    /// Calculate SHA256 checksum of a file
    async fn calculate_file_checksum(&self, file_path: &Path) -> UpgradeResult<String> {
        let file_path = file_path.to_path_buf();

        // Use blocking task for file I/O
        tokio::task::spawn_blocking(move || {
            let mut file =
                File::open(&file_path).map_err(|e| UpgradeError::InvalidPackage(e.to_string()))?;

            let mut hasher = Sha256::new();
            let mut buffer = [0; 8192];

            loop {
                let bytes_read = file
                    .read(&mut buffer)
                    .map_err(|e| UpgradeError::InvalidPackage(e.to_string()))?;

                if bytes_read == 0 {
                    break;
                }

                hasher.update(&buffer[..bytes_read]);
            }

            let hash = hasher.finalize();
            Ok(format!("{:x}", hash))
        })
        .await
        .map_err(|e| UpgradeError::Internal(e.to_string()))?
    }

    /// Extract filename from URL
    fn extract_filename_from_url(&self, url: &str) -> UpgradeResult<String> {
        url.split('/')
            .next_back()
            .ok_or_else(|| UpgradeError::InvalidPackage("Invalid download URL".to_string()))
            .map(|s| s.to_string())
    }

    /// Get file size safely
    fn get_file_size(&self, file_path: &Path) -> UpgradeResult<u64> {
        let metadata = std::fs::metadata(file_path)
            .map_err(|e| UpgradeError::InvalidPackage(e.to_string()))?;
        Ok(metadata.len())
    }

    /// Calculate download speed in bytes per second
    fn calculate_download_speed(&self, session: &DownloadSession) -> u64 {
        let elapsed = session.start_time.elapsed();
        if elapsed.as_secs() == 0 {
            return 0;
        }

        session.downloaded_size / elapsed.as_secs()
    }

    /// Cancel an ongoing download
    pub fn cancel_download(&self, session_id: &str) {
        // In a real implementation, you'd track sessions by ID
        // For now, this is a placeholder for the cancellation mechanism
        debug!(
            "Download cancellation requested for session: {}",
            session_id
        );
    }

    /// Clean up temporary download files
    pub async fn cleanup_downloads(&self) -> Result<()> {
        info!("Cleaning up temporary download files");

        let mut cleaned_files = 0;
        let entries = std::fs::read_dir(&self.download_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                // Remove files older than 24 hours
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified.elapsed().unwrap_or(Duration::ZERO) > Duration::from_secs(86400)
                        {
                            if let Err(e) = std::fs::remove_file(&path) {
                                warn!("Failed to remove old download file {:?}: {}", path, e);
                            } else {
                                cleaned_files += 1;
                                debug!("Removed old download file: {:?}", path);
                            }
                        }
                    }
                }
            }
        }

        if cleaned_files > 0 {
            info!("Cleaned up {} old download files", cleaned_files);
        }

        Ok(())
    }

    /// Check available disk space
    pub fn check_disk_space(&self, required_bytes: u64) -> UpgradeResult<()> {
        // Platform-specific disk space checking would go here
        // For now, we'll use a simplified check

        #[cfg(unix)]
        {
            use std::ffi::CString;
            use std::mem;
            use std::os::raw::{c_char, c_ulong};

            #[repr(C)]
            struct Statvfs {
                f_bsize: c_ulong,
                f_frsize: c_ulong,
                f_blocks: c_ulong,
                f_bfree: c_ulong,
                f_bavail: c_ulong,
                f_files: c_ulong,
                f_ffree: c_ulong,
                f_favail: c_ulong,
                f_fsid: c_ulong,
                f_flag: c_ulong,
                f_namemax: c_ulong,
            }

            extern "C" {
                fn statvfs(path: *const c_char, buf: *mut Statvfs) -> i32;
            }

            let path = CString::new(self.download_dir.to_string_lossy().as_ref()).unwrap();
            let mut stat: Statvfs = unsafe { mem::zeroed() };

            if unsafe { statvfs(path.as_ptr(), &mut stat) } == 0 {
                let available_bytes = stat.f_bavail * stat.f_frsize;
                if available_bytes < required_bytes {
                    return Err(UpgradeError::InsufficientDiskSpace {
                        required: required_bytes / 1024 / 1024,
                        available: available_bytes / 1024 / 1024,
                    });
                }
            }
        }

        Ok(())
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
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_downloader_creation() {
        let config = create_test_config();
        let downloader = UpdateDownloader::new(&config);
        assert!(downloader.is_ok());
    }

    #[tokio::test]
    async fn test_checksum_calculation() {
        let config = create_test_config();
        let downloader = UpdateDownloader::new(&config).unwrap();

        // Create a test file
        let test_content = b"Hello, world!";
        let test_file = config.download_dir.join("test.txt");
        std::fs::write(&test_file, test_content).unwrap();

        let checksum = downloader
            .calculate_file_checksum(&test_file)
            .await
            .unwrap();

        // Expected SHA256 of "Hello, world!"
        let expected = "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3";
        assert_eq!(checksum, expected);
    }

    #[test]
    fn test_filename_extraction() {
        let config = create_test_config();
        let downloader = UpdateDownloader::new(&config).unwrap();

        let url = "https://example.com/files/app-v1.2.3.tar.gz";
        let filename = downloader.extract_filename_from_url(url).unwrap();
        assert_eq!(filename, "app-v1.2.3.tar.gz");
    }

    #[test]
    fn test_disk_space_check() {
        let config = create_test_config();
        let downloader = UpdateDownloader::new(&config).unwrap();

        // Check for a reasonable amount of space (1MB)
        let result = downloader.check_disk_space(1024 * 1024);
        // This should generally pass on development machines
        assert!(result.is_ok());
    }
}
