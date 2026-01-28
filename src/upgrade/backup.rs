#![allow(dead_code, unused_imports, unused_variables, clippy::ptr_arg)]
//! # Backup Manager
//!
//! Comprehensive backup and restore system for safe application upgrades
//! with versioned backups, compression, and integrity verification.

use super::{ApplicationVersion, UpgradeConfig};
use anyhow::Result;
use chrono::{DateTime, Utc};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use tracing::{debug, error, info, warn};

/// Backup metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub id: String,
    pub version: ApplicationVersion,
    pub created_at: DateTime<Utc>,
    pub backup_type: BackupType,
    pub file_path: PathBuf,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub checksum: String,
    pub compression_method: CompressionMethod,
    pub includes_config: bool,
    pub includes_data: bool,
    pub description: String,
}

/// Type of backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    /// Full application backup
    Full,
    /// Configuration only
    ConfigOnly,
    /// Data only
    DataOnly,
    /// Custom selective backup
    Custom { paths: Vec<PathBuf> },
}

/// Compression method for backups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionMethod {
    None,
    Gzip,
    Zstd,
}

/// Backup and restore manager
pub struct BackupManager {
    config: UpgradeConfig,
    backup_dir: PathBuf,
    metadata_file: PathBuf,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new(config: &UpgradeConfig) -> Result<Self> {
        let backup_dir = config.backup_dir.clone();
        fs::create_dir_all(&backup_dir)?;

        let metadata_file = backup_dir.join("backup_metadata.json");

        Ok(Self {
            config: config.clone(),
            backup_dir,
            metadata_file,
        })
    }

    /// Create a full backup before upgrade
    pub async fn create_backup(&self) -> Result<PathBuf> {
        info!("Creating full application backup");

        let backup_id = self.generate_backup_id();
        let current_version = ApplicationVersion::current();

        // Determine what to backup
        let paths_to_backup = self.get_backup_paths(BackupType::Full)?;

        // Create backup archive
        let backup_filename = format!(
            "inferno_backup_{}_{}.tar.gz",
            current_version.to_string().replace('.', "_"),
            backup_id
        );
        let backup_path = self.backup_dir.join(&backup_filename);

        // Create compressed tar archive
        let (compressed_size, uncompressed_size, checksum) = self
            .create_compressed_archive(&paths_to_backup, &backup_path)
            .await?;

        // Create metadata
        let metadata = BackupMetadata {
            id: backup_id,
            version: current_version,
            created_at: Utc::now(),
            backup_type: BackupType::Full,
            file_path: backup_path.clone(),
            compressed_size,
            uncompressed_size,
            checksum,
            compression_method: CompressionMethod::Gzip,
            includes_config: true,
            includes_data: true,
            description: "Pre-upgrade full backup".to_string(),
        };

        // Save metadata
        self.save_backup_metadata(&metadata).await?;

        // Cleanup old backups if needed
        self.cleanup_old_backups().await?;

        info!("Backup created successfully: {:?}", backup_path);
        Ok(backup_path)
    }

    /// Create a selective backup
    pub async fn create_selective_backup(
        &self,
        backup_type: BackupType,
        description: String,
    ) -> Result<PathBuf> {
        info!("Creating selective backup: {:?}", backup_type);

        let backup_id = self.generate_backup_id();
        let current_version = ApplicationVersion::current();

        let paths_to_backup = self.get_backup_paths(backup_type.clone())?;

        let backup_filename = format!(
            "inferno_selective_{}_{}.tar.gz",
            current_version.to_string().replace('.', "_"),
            backup_id
        );
        let backup_path = self.backup_dir.join(&backup_filename);

        let (compressed_size, uncompressed_size, checksum) = self
            .create_compressed_archive(&paths_to_backup, &backup_path)
            .await?;

        let (includes_config, includes_data) = match &backup_type {
            BackupType::Full => (true, true),
            BackupType::ConfigOnly => (true, false),
            BackupType::DataOnly => (false, true),
            BackupType::Custom { .. } => (true, true), // Conservative assumption
        };

        let metadata = BackupMetadata {
            id: backup_id,
            version: current_version,
            created_at: Utc::now(),
            backup_type,
            file_path: backup_path.clone(),
            compressed_size,
            uncompressed_size,
            checksum,
            compression_method: CompressionMethod::Gzip,
            includes_config,
            includes_data,
            description,
        };

        self.save_backup_metadata(&metadata).await?;
        self.cleanup_old_backups().await?;

        info!("Selective backup created successfully: {:?}", backup_path);
        Ok(backup_path)
    }

    /// Restore from a backup
    pub async fn restore_backup(&self, backup_path: &PathBuf) -> Result<()> {
        info!("Restoring from backup: {:?}", backup_path);

        // Verify backup exists and is valid
        if !backup_path.exists() {
            return Err(anyhow::anyhow!("Backup file not found: {:?}", backup_path));
        }

        // Load backup metadata
        let metadata = self.get_backup_metadata_by_path(backup_path).await?;

        // Verify backup integrity
        self.verify_backup_integrity(&metadata).await?;

        // Create restore point before restoring
        let restore_point = self.create_restore_point().await?;

        match self.perform_restore(&metadata).await {
            Ok(_) => {
                info!("Backup restored successfully");
                // Cleanup the temporary restore point since restore succeeded
                if let Err(e) = fs::remove_file(&restore_point) {
                    warn!("Failed to cleanup restore point: {}", e);
                }
                Ok(())
            }
            Err(e) => {
                error!("Restore failed: {}", e);
                // Keep the restore point for manual recovery
                warn!("Restore point preserved at: {:?}", restore_point);
                Err(e)
            }
        }
    }

    /// List all available backups
    pub async fn list_backups(&self) -> Result<Vec<BackupMetadata>> {
        let all_metadata = self.load_all_backup_metadata().await?;

        // Sort by creation date (newest first)
        let mut backups = all_metadata;
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    /// Get backup by ID
    pub async fn get_backup_by_id(&self, backup_id: &str) -> Result<Option<BackupMetadata>> {
        let all_backups = self.list_backups().await?;
        Ok(all_backups.into_iter().find(|b| b.id == backup_id))
    }

    /// Delete a specific backup
    pub async fn delete_backup(&self, backup_id: &str) -> Result<()> {
        info!("Deleting backup: {}", backup_id);

        if let Some(metadata) = self.get_backup_by_id(backup_id).await? {
            // Remove backup file
            if metadata.file_path.exists() {
                fs::remove_file(&metadata.file_path)?;
                debug!("Removed backup file: {:?}", metadata.file_path);
            }

            // Remove from metadata
            self.remove_backup_metadata(backup_id).await?;

            info!("Backup deleted successfully: {}", backup_id);
        } else {
            warn!("Backup not found: {}", backup_id);
        }

        Ok(())
    }

    /// Verify backup integrity
    pub async fn verify_backup_integrity(&self, metadata: &BackupMetadata) -> Result<()> {
        debug!("Verifying backup integrity: {}", metadata.id);

        if !metadata.file_path.exists() {
            return Err(anyhow::anyhow!(
                "Backup file missing: {:?}",
                metadata.file_path
            ));
        }

        // Verify file size
        let file_size = fs::metadata(&metadata.file_path)?.len();
        if file_size != metadata.compressed_size {
            return Err(anyhow::anyhow!(
                "Backup file size mismatch: expected {}, got {}",
                metadata.compressed_size,
                file_size
            ));
        }

        // Verify checksum
        let calculated_checksum = self.calculate_file_checksum(&metadata.file_path).await?;
        if calculated_checksum != metadata.checksum {
            return Err(anyhow::anyhow!(
                "Backup checksum mismatch: expected {}, got {}",
                metadata.checksum,
                calculated_checksum
            ));
        }

        // Verify archive can be opened
        match metadata.compression_method {
            CompressionMethod::Gzip => {
                let file = File::open(&metadata.file_path)?;
                let decoder = GzDecoder::new(file);
                let mut archive = Archive::new(decoder);

                // Try to list entries without extracting
                for entry in archive.entries()? {
                    let entry = entry?;
                    debug!("Archive entry: {:?}", entry.path()?);
                }
            }
            CompressionMethod::None => {
                let file = File::open(&metadata.file_path)?;
                let mut archive = Archive::new(file);

                for entry in archive.entries()? {
                    let entry = entry?;
                    debug!("Archive entry: {:?}", entry.path()?);
                }
            }
            CompressionMethod::Zstd => {
                // Zstd verification would go here
                warn!("Zstd verification not yet implemented");
            }
        }

        debug!("Backup integrity verification passed");
        Ok(())
    }

    /// Get storage usage statistics
    pub async fn get_storage_stats(&self) -> Result<BackupStorageStats> {
        let backups = self.list_backups().await?;

        let total_backups = backups.len();
        let total_size = backups.iter().map(|b| b.compressed_size).sum();
        let oldest_backup = backups.iter().map(|b| b.created_at).min();
        let newest_backup = backups.iter().map(|b| b.created_at).max();

        Ok(BackupStorageStats {
            total_backups,
            total_size_bytes: total_size,
            oldest_backup,
            newest_backup,
            backup_dir: self.backup_dir.clone(),
        })
    }

    /// Cleanup old backups based on retention policy
    async fn cleanup_old_backups(&self) -> Result<()> {
        let backups = self.list_backups().await?;

        if backups.len() <= self.config.max_backups as usize {
            return Ok(());
        }

        // Sort by creation date (oldest first)
        let mut sorted_backups = backups;
        sorted_backups.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        // Keep only the most recent max_backups
        let total_backups = sorted_backups.len();
        if total_backups > self.config.max_backups as usize {
            let to_delete: Vec<_> = sorted_backups
                .into_iter()
                .take(total_backups - self.config.max_backups as usize)
                .collect();

            for backup in to_delete {
                info!(
                    "Cleaning up old backup: {} ({})",
                    backup.id, backup.created_at
                );
                if let Err(e) = self.delete_backup(&backup.id).await {
                    warn!("Failed to delete old backup {}: {}", backup.id, e);
                }
            }
        }

        Ok(())
    }

    /// Generate unique backup ID
    fn generate_backup_id(&self) -> String {
        use uuid::Uuid;
        Uuid::new_v4().to_string()[..8].to_string()
    }

    /// Get paths to backup based on backup type
    fn get_backup_paths(&self, backup_type: BackupType) -> Result<Vec<PathBuf>> {
        let current_exe = std::env::current_exe()?;
        let app_dir = current_exe
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine application directory"))?;

        match backup_type {
            BackupType::Full => {
                let mut paths = vec![current_exe.clone()];

                // Add configuration files
                if let Some(home) = dirs::home_dir() {
                    let config_paths = vec![
                        home.join(".inferno.toml"),
                        home.join(".config/inferno/config.toml"),
                    ];

                    for path in config_paths {
                        if path.exists() {
                            paths.push(path);
                        }
                    }
                }

                // Add data directory if it exists
                if let Some(data_dir) = &self.config.download_dir.parent() {
                    if data_dir.exists() {
                        paths.push(data_dir.to_path_buf());
                    }
                }

                Ok(paths)
            }
            BackupType::ConfigOnly => {
                let mut paths = vec![];

                if let Some(home) = dirs::home_dir() {
                    let config_paths = vec![
                        home.join(".inferno.toml"),
                        home.join(".config/inferno/config.toml"),
                    ];

                    for path in config_paths {
                        if path.exists() {
                            paths.push(path);
                        }
                    }
                }

                Ok(paths)
            }
            BackupType::DataOnly => {
                let mut paths = vec![];

                if let Some(data_dir) = &self.config.download_dir.parent() {
                    if data_dir.exists() {
                        paths.push(data_dir.to_path_buf());
                    }
                }

                Ok(paths)
            }
            BackupType::Custom { paths } => Ok(paths),
        }
    }

    /// Create compressed archive
    async fn create_compressed_archive(
        &self,
        paths: &[PathBuf],
        output_path: &PathBuf,
    ) -> Result<(u64, u64, String)> {
        let file = File::create(output_path)?;
        let encoder = GzEncoder::new(file, Compression::default());
        let mut archive = Builder::new(encoder);

        let mut uncompressed_size = 0u64;

        for path in paths {
            if path.is_file() {
                debug!("Adding file to backup: {:?}", path);

                let file_size = fs::metadata(path)?.len();
                uncompressed_size += file_size;

                let relative_path = path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("Invalid file path: {:?}", path))?;

                archive.append_file(relative_path, &mut File::open(path)?)?;
            } else if path.is_dir() {
                debug!("Adding directory to backup: {:?}", path);
                self.add_directory_to_archive(&mut archive, path, &mut uncompressed_size)?;
            }
        }

        archive.finish()?;

        let compressed_size = fs::metadata(output_path)?.len();
        let checksum = self.calculate_file_checksum(output_path).await?;

        Ok((compressed_size, uncompressed_size, checksum))
    }

    /// Add directory recursively to archive
    fn add_directory_to_archive(
        &self,
        archive: &mut Builder<GzEncoder<File>>,
        dir_path: &Path,
        uncompressed_size: &mut u64,
    ) -> Result<()> {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let file_size = fs::metadata(&path)?.len();
                *uncompressed_size += file_size;

                let relative_path = path.strip_prefix(dir_path.parent().unwrap_or(dir_path))?;
                archive.append_file(relative_path, &mut File::open(&path)?)?;
            } else if path.is_dir() {
                self.add_directory_to_archive(archive, &path, uncompressed_size)?;
            }
        }

        Ok(())
    }

    /// Perform the actual restore operation
    async fn perform_restore(&self, metadata: &BackupMetadata) -> Result<()> {
        info!("Performing restore from backup: {}", metadata.id);

        let file = File::open(&metadata.file_path)?;

        match metadata.compression_method {
            CompressionMethod::Gzip => {
                let decoder = GzDecoder::new(file);
                let mut archive = Archive::new(decoder);

                // Extract to a temporary directory first
                let temp_dir = tempfile::TempDir::new()?;
                archive.unpack(temp_dir.path())?;

                // Move files to their final locations
                self.move_restored_files(temp_dir.path()).await?;
            }
            CompressionMethod::None => {
                let mut archive = Archive::new(file);
                let temp_dir = tempfile::TempDir::new()?;
                archive.unpack(temp_dir.path())?;
                self.move_restored_files(temp_dir.path()).await?;
            }
            CompressionMethod::Zstd => {
                return Err(anyhow::anyhow!("Zstd decompression not yet implemented"));
            }
        }

        Ok(())
    }

    /// Move restored files to their final locations
    async fn move_restored_files(&self, temp_dir: &Path) -> Result<()> {
        // This is a simplified implementation
        // In a real implementation, you would carefully map files back to their original locations

        for entry in fs::read_dir(temp_dir)? {
            let entry = entry?;
            let source_path = entry.path();

            if let Some(filename) = source_path.file_name() {
                if filename == "inferno" || filename == "inferno.exe" {
                    // Restore executable
                    let current_exe = std::env::current_exe()?;

                    // On Windows, we might need to rename the current executable first
                    #[cfg(target_os = "windows")]
                    {
                        let backup_exe = current_exe.with_extension("exe.old");
                        fs::rename(&current_exe, &backup_exe)?;
                    }

                    fs::copy(&source_path, &current_exe)?;

                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let mut perms = fs::metadata(&current_exe)?.permissions();
                        perms.set_mode(0o755);
                        fs::set_permissions(&current_exe, perms)?;
                    }
                } else if filename.to_string_lossy().contains("config") {
                    // Restore configuration files
                    if let Some(home) = dirs::home_dir() {
                        let config_path = home.join(".inferno.toml");
                        fs::copy(&source_path, &config_path)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Create a restore point before performing restore
    async fn create_restore_point(&self) -> Result<PathBuf> {
        let restore_point_path = self.backup_dir.join(format!(
            "restore_point_{}.tar.gz",
            chrono::Utc::now().timestamp()
        ));

        let current_exe = std::env::current_exe()?;
        let paths = vec![current_exe];

        let (_, _, _) = self
            .create_compressed_archive(&paths, &restore_point_path)
            .await?;

        Ok(restore_point_path)
    }

    /// Calculate SHA256 checksum of a file
    async fn calculate_file_checksum(&self, file_path: &PathBuf) -> Result<String> {
        let file_path = file_path.clone();

        tokio::task::spawn_blocking(move || {
            let mut file = File::open(&file_path)?;
            let mut hasher = Sha256::new();
            let mut buffer = [0; 8192];

            loop {
                let bytes_read = file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }

            let hash = hasher.finalize();
            Ok(format!("{:x}", hash))
        })
        .await?
    }

    /// Save backup metadata
    async fn save_backup_metadata(&self, metadata: &BackupMetadata) -> Result<()> {
        let mut all_metadata = self.load_all_backup_metadata().await.unwrap_or_default();
        all_metadata.push(metadata.clone());

        let json_data = serde_json::to_string_pretty(&all_metadata)?;
        fs::write(&self.metadata_file, json_data)?;

        Ok(())
    }

    /// Load all backup metadata
    async fn load_all_backup_metadata(&self) -> Result<Vec<BackupMetadata>> {
        if !self.metadata_file.exists() {
            return Ok(vec![]);
        }

        let json_data = fs::read_to_string(&self.metadata_file)?;
        let metadata: Vec<BackupMetadata> = serde_json::from_str(&json_data)?;

        Ok(metadata)
    }

    /// Get backup metadata by file path
    async fn get_backup_metadata_by_path(&self, backup_path: &PathBuf) -> Result<BackupMetadata> {
        let all_metadata = self.load_all_backup_metadata().await?;

        all_metadata
            .into_iter()
            .find(|m| m.file_path == *backup_path)
            .ok_or_else(|| anyhow::anyhow!("Backup metadata not found for path: {:?}", backup_path))
    }

    /// Remove backup metadata by ID
    async fn remove_backup_metadata(&self, backup_id: &str) -> Result<()> {
        let mut all_metadata = self.load_all_backup_metadata().await?;
        all_metadata.retain(|m| m.id != backup_id);

        let json_data = serde_json::to_string_pretty(&all_metadata)?;
        fs::write(&self.metadata_file, json_data)?;

        Ok(())
    }
}

/// Backup storage statistics
#[derive(Debug, Clone)]
pub struct BackupStorageStats {
    pub total_backups: usize,
    pub total_size_bytes: u64,
    pub oldest_backup: Option<DateTime<Utc>>,
    pub newest_backup: Option<DateTime<Utc>>,
    pub backup_dir: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> UpgradeConfig {
        let temp_dir = TempDir::new().unwrap();
        UpgradeConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            max_backups: 3,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_backup_manager_creation() {
        let config = create_test_config();
        let manager = BackupManager::new(&config);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_backup_id_generation() {
        let config = create_test_config();
        let manager = BackupManager::new(&config).unwrap();

        let id1 = manager.generate_backup_id();
        let id2 = manager.generate_backup_id();

        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 8);
    }

    #[tokio::test]
    async fn test_backup_paths() {
        let config = create_test_config();
        let manager = BackupManager::new(&config).unwrap();

        let full_paths = manager.get_backup_paths(BackupType::Full).unwrap();
        assert!(!full_paths.is_empty());

        let config_paths = manager.get_backup_paths(BackupType::ConfigOnly).unwrap();
        // Config paths might be empty in test environment
        println!("Config paths: {:?}", config_paths);
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let config = create_test_config();
        let manager = BackupManager::new(&config).unwrap();

        let stats = manager.get_storage_stats().await.unwrap();
        assert_eq!(stats.total_backups, 0);
        assert_eq!(stats.total_size_bytes, 0);
    }
}
