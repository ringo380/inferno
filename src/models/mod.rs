use crate::InfernoError;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub backend_type: String,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GgufMetadata {
    pub architecture: String,
    pub parameter_count: u64,
    pub quantization: String,
    pub context_length: u32,
}

#[derive(Debug, Clone)]
pub struct OnnxMetadata {
    pub version: String,
    pub producer: String,
    pub input_count: u32,
    pub output_count: u32,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub file_readable: bool,
    pub format_valid: bool,
    pub size_valid: bool,
    pub checksum_valid: Option<bool>,
    pub security_valid: bool,
    pub metadata_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            file_readable: false,
            format_valid: false,
            size_valid: false,
            checksum_valid: None,
            security_valid: false,
            metadata_valid: false,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn finalize(&mut self) {
        self.is_valid = self.file_readable
            && self.format_valid
            && self.size_valid
            && self.security_valid
            && self.metadata_valid
            && self.checksum_valid.unwrap_or(true);
    }
}

#[derive(Clone)]
pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new(models_dir: &Path) -> Self {
        Self {
            models_dir: models_dir.to_path_buf(),
        }
    }

    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        if !self.models_dir.exists() {
            warn!("Models directory does not exist: {}", self.models_dir.display());
            return Ok(Vec::new());
        }

        let mut models = Vec::new();
        let mut entries = async_fs::read_dir(&self.models_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext_str = extension.to_string_lossy().to_lowercase();
                    if matches!(ext_str.as_str(), "gguf" | "onnx") {
                        match self.create_model_info(&path).await {
                            Ok(model_info) => models.push(model_info),
                            Err(e) => {
                                error!("Failed to process model {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        // Sort models by modification time (newest first)
        models.sort_by(|a, b| b.modified.cmp(&a.modified));

        info!("Found {} models in {}", models.len(), self.models_dir.display());
        Ok(models)
    }

    pub async fn resolve_model(&self, model_name_or_path: &str) -> Result<ModelInfo> {
        let path = if model_name_or_path.contains('/') || model_name_or_path.contains('\\') {
            // Treat as path
            PathBuf::from(model_name_or_path)
        } else {
            // Treat as model name, look in models directory
            self.find_model_by_name(model_name_or_path).await?
        };

        if !path.exists() {
            return Err(anyhow::anyhow!("Model not found: {}", path.display()));
        }

        self.create_model_info(&path).await
    }

    async fn find_model_by_name(&self, name: &str) -> Result<PathBuf> {
        let models = self.list_models().await?;

        for model in models {
            if model.name == name || model.name.starts_with(name) {
                return Ok(model.path);
            }
        }

        // Try adding common extensions
        for ext in ["gguf", "onnx"] {
            let path_with_ext = self.models_dir.join(format!("{}.{}", name, ext));
            if path_with_ext.exists() {
                return Ok(path_with_ext);
            }
        }

        Err(anyhow::anyhow!("Model '{}' not found in models directory", name))
    }

    async fn create_model_info(&self, path: &Path) -> Result<ModelInfo> {
        let metadata = async_fs::metadata(path).await?;
        let modified = chrono::DateTime::from(metadata.modified()?);

        let name = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?
            .to_string_lossy()
            .to_string();

        let backend_type = self.determine_backend_type(path);

        Ok(ModelInfo {
            name,
            path: path.to_path_buf(),
            size: metadata.len(),
            modified,
            backend_type,
            checksum: None, // Computed on demand
        })
    }

    fn determine_backend_type(&self, path: &Path) -> String {
        if let Some(extension) = path.extension() {
            match extension.to_string_lossy().to_lowercase().as_str() {
                "gguf" => "gguf".to_string(),
                "onnx" => "onnx".to_string(),
                _ => "unknown".to_string(),
            }
        } else {
            "unknown".to_string()
        }
    }

    pub async fn validate_model(&self, path: &Path) -> Result<bool> {
        let result = self.validate_model_comprehensive(path, None).await?;
        Ok(result.is_valid)
    }

    pub async fn validate_model_comprehensive(
        &self,
        path: &Path,
        config: Option<&crate::config::Config>,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Check if file exists
        if !path.exists() {
            result.add_error(format!("File does not exist: {}", path.display()));
            result.finalize();
            return Ok(result);
        }

        if !path.is_file() {
            result.add_error(format!("Path is not a file: {}", path.display()));
            result.finalize();
            return Ok(result);
        }

        // File readability check
        match async_fs::File::open(path).await {
            Ok(_) => result.file_readable = true,
            Err(e) => {
                result.add_error(format!("Cannot read file: {}", e));
                result.finalize();
                return Ok(result);
            }
        }

        // File size and metadata check
        let metadata = match async_fs::metadata(path).await {
            Ok(meta) => meta,
            Err(e) => {
                result.add_error(format!("Cannot read file metadata: {}", e));
                result.finalize();
                return Ok(result);
            }
        };

        if metadata.len() == 0 {
            result.add_error("File is empty".to_string());
            result.finalize();
            return Ok(result);
        }

        // Size validation against config limits
        if let Some(config) = config {
            let max_size_bytes = if let Some(ref sec) = config.model_security {
                (sec.max_model_size_gb * 1024.0 * 1024.0 * 1024.0) as u64
            } else {
                5 * 1024 * 1024 * 1024 // 5GB default
            };
            if metadata.len() > max_size_bytes {
                result.add_error(format!(
                    "Model size {} bytes exceeds limit of {} bytes",
                    metadata.len(),
                    max_size_bytes
                ));
                result.finalize();
                return Ok(result);
            }
        }
        result.size_valid = true;

        // Extension and format validation
        let extension = match path.extension() {
            Some(ext) => ext.to_string_lossy().to_lowercase(),
            None => {
                result.add_error("File has no extension".to_string());
                result.finalize();
                return Ok(result);
            }
        };

        if let Some(config) = config {
            if !config.is_model_extension_allowed(&extension) {
                result.add_error(format!("File extension '{}' is not allowed", extension));
                result.finalize();
                return Ok(result);
            }
        } else if !matches!(extension.as_str(), "gguf" | "onnx") {
            result.add_error(format!("Unsupported file extension: {}", extension));
            result.finalize();
            return Ok(result);
        }

        // Security validation
        if let Err(e) = self.security_validate(path, &metadata).await {
            result.add_error(format!("Security validation failed: {}", e));
        } else {
            result.security_valid = true;
        }

        // Format-specific validation
        match self.validate_format_specific(path, &extension).await {
            Ok(format_result) => {
                result.format_valid = format_result.0;
                if !format_result.0 {
                    result.add_error(format_result.1);
                } else {
                    result.metadata_valid = true;
                }
            }
            Err(e) => {
                result.add_error(format!("Format validation failed: {}", e));
            }
        }

        result.finalize();
        Ok(result)
    }

    async fn security_validate(&self, path: &Path, metadata: &std::fs::Metadata) -> Result<(), InfernoError> {
        // Check for suspicious file patterns
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        // Block files with suspicious names
        let suspicious_patterns = [
            "..", "~", "$", "`", ";", "|", "&", "<", ">", "\\",
            "script", "exec", "eval", "system"
        ];

        for pattern in &suspicious_patterns {
            if file_name.contains(pattern) {
                return Err(InfernoError::SecurityValidation(
                    format!("Suspicious filename pattern detected: {}", pattern)
                ));
            }
        }

        // Check file permissions (Unix-specific)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = metadata.permissions();
            let mode = perms.mode();

            // Check if file is executable (suspicious for model files)
            if mode & 0o111 != 0 {
                return Err(InfernoError::SecurityValidation(
                    "Model file should not be executable".to_string()
                ));
            }
        }

        // Read first chunk to check for suspicious content
        let mut file = async_fs::File::open(path).await.map_err(InfernoError::Io)?;
        let mut buffer = vec![0u8; 4096];
        use tokio::io::AsyncReadExt;
        let bytes_read = file.read(&mut buffer).await.map_err(InfernoError::Io)?;

        if bytes_read > 0 {
            let content = &buffer[..bytes_read];

            // Check for script patterns in binary files
            let script_patterns: &[&[u8]] = &[
                b"#!/bin/", b"#!/usr/", b"<script", b"javascript:", b"python", b"exec("
            ];

            for pattern in script_patterns {
                if content.windows(pattern.len()).any(|window| window == *pattern) {
                    return Err(InfernoError::SecurityValidation(
                        "Suspicious script content detected in model file".to_string()
                    ));
                }
            }
        }

        Ok(())
    }

    async fn validate_format_specific(&self, path: &Path, extension: &str) -> Result<(bool, String)> {
        let mut file = async_fs::File::open(path).await?;
        let mut buffer = vec![0u8; 8192];
        use tokio::io::AsyncReadExt;
        let bytes_read = file.read(&mut buffer).await?;

        if bytes_read == 0 {
            return Ok((false, "Cannot read file content".to_string()));
        }

        match extension {
            "gguf" => self.validate_gguf_format_detailed(&buffer),
            "onnx" => self.validate_onnx_format_detailed(&buffer),
            _ => Ok((false, format!("Unknown format: {}", extension))),
        }
    }

    fn validate_gguf_format(&self, buffer: &[u8]) -> Result<bool> {
        let (valid, _) = self.validate_gguf_format_detailed(buffer)?;
        Ok(valid)
    }

    fn validate_onnx_format(&self, buffer: &[u8]) -> Result<bool> {
        let (valid, _) = self.validate_onnx_format_detailed(buffer)?;
        Ok(valid)
    }

    fn validate_gguf_format_detailed(&self, buffer: &[u8]) -> Result<(bool, String)> {
        if buffer.len() < 8 {
            return Ok((false, "File too small to be a valid GGUF file".to_string()));
        }

        // Check magic bytes
        let magic = &buffer[0..4];
        if magic != b"GGUF" {
            return Ok((false, format!(
                "Invalid GGUF magic bytes. Expected 'GGUF', found {:?}",
                String::from_utf8_lossy(magic)
            )));
        }

        // Check version (next 4 bytes, little-endian)
        let version = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
        if version == 0 || version > 10 {
            return Ok((false, format!("Invalid GGUF version: {}", version)));
        }

        // Additional validation could include:
        // - Tensor count validation
        // - Metadata key-value pairs validation
        // - Architecture validation

        Ok((true, format!("Valid GGUF file, version {}", version)))
    }

    fn validate_onnx_format_detailed(&self, buffer: &[u8]) -> Result<(bool, String)> {
        if buffer.len() < 16 {
            return Ok((false, "File too small to be a valid ONNX file".to_string()));
        }

        // ONNX files are Protocol Buffers, so we check for protobuf structure
        // Protocol Buffers start with field numbers and wire types

        // Check for protobuf varint encoding (common in ONNX)
        let mut has_valid_protobuf_structure = false;
        let mut i = 0;
        while i < buffer.len().min(100) {
            let byte = buffer[i];
            // Check for valid protobuf wire types (0-5)
            let wire_type = byte & 0x07;
            if wire_type <= 5 {
                has_valid_protobuf_structure = true;
                break;
            }
            i += 1;
        }

        if !has_valid_protobuf_structure {
            return Ok((false, "Invalid ONNX file: No valid protobuf structure found".to_string()));
        }

        // Check for common ONNX strings in the header
        let header_str = String::from_utf8_lossy(&buffer[0..buffer.len().min(512)]);
        let has_onnx_markers = header_str.contains("onnx")
            || header_str.contains("model_proto")
            || header_str.contains("GraphProto")
            || buffer.windows(4).any(|w| w == b"onnx");

        if !has_onnx_markers {
            return Ok((false, "Invalid ONNX file: No ONNX markers found in header".to_string()));
        }

        Ok((true, "Valid ONNX file detected".to_string()))
    }

    pub async fn compute_checksum(&self, path: &Path) -> Result<String> {
        let mut file = async_fs::File::open(path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 8192];

        use tokio::io::AsyncReadExt;
        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    pub async fn get_gguf_metadata(&self, path: &Path) -> Result<GgufMetadata> {
        // This is a placeholder implementation
        // Real implementation would parse GGUF file headers

        info!("Reading GGUF metadata from: {}", path.display());

        // For demonstration, return mock metadata based on filename patterns
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        let parameter_count = if filename.contains("7b") {
            7_000_000_000
        } else if filename.contains("13b") {
            13_000_000_000
        } else if filename.contains("30b") {
            30_000_000_000
        } else if filename.contains("65b") {
            65_000_000_000
        } else {
            7_000_000_000 // Default
        };

        let quantization = if filename.contains("q4_0") {
            "Q4_0".to_string()
        } else if filename.contains("q4_1") {
            "Q4_1".to_string()
        } else if filename.contains("q5_0") {
            "Q5_0".to_string()
        } else if filename.contains("q5_1") {
            "Q5_1".to_string()
        } else if filename.contains("q8_0") {
            "Q8_0".to_string()
        } else {
            "F16".to_string()
        };

        let architecture = if filename.contains("llama") {
            "llama".to_string()
        } else if filename.contains("gpt") {
            "gpt".to_string()
        } else if filename.contains("falcon") {
            "falcon".to_string()
        } else {
            "unknown".to_string()
        };

        Ok(GgufMetadata {
            architecture,
            parameter_count,
            quantization,
            context_length: 2048,
        })
    }

    pub async fn get_onnx_metadata(&self, path: &Path) -> Result<OnnxMetadata> {
        // This is a placeholder implementation
        // Real implementation would parse ONNX model metadata

        info!("Reading ONNX metadata from: {}", path.display());

        // For demonstration, return mock metadata
        Ok(OnnxMetadata {
            version: "1.13.0".to_string(),
            producer: "pytorch".to_string(),
            input_count: 1,
            output_count: 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_model_manager() {
        let temp_dir = tempdir().expect("Failed to create temporary directory for test");
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir).await.expect("Failed to create models directory for test");

        let manager = ModelManager::new(&models_dir);

        // Initially no models
        let models = manager.list_models().await.expect("Failed to list models in test");
        assert!(models.is_empty());

        // Create a mock model file
        let model_path = models_dir.join("test_model.gguf");
        fs::write(&model_path, b"GGUF\x00\x00\x00\x01mock data").await.expect("Failed to write test model file");

        let models = manager.list_models().await.expect("Failed to list models in test");
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "test_model.gguf");
        assert_eq!(models[0].backend_type, "gguf");
    }

    #[tokio::test]
    async fn test_model_validation() {
        let temp_dir = tempdir().expect("Failed to create temporary directory for test");
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir).await.expect("Failed to create models directory for test");

        let manager = ModelManager::new(&models_dir);

        // Valid GGUF file
        let gguf_path = models_dir.join("valid.gguf");
        fs::write(&gguf_path, b"GGUF\x00\x00\x00\x01mock data").await.expect("Failed to write valid GGUF file");
        assert!(manager.validate_model(&gguf_path).await.expect("Failed to validate valid GGUF model"));

        // Invalid GGUF file (wrong magic bytes)
        let invalid_path = models_dir.join("invalid.gguf");
        fs::write(&invalid_path, b"INVALID_DATA").await.expect("Failed to write invalid file");
        assert!(!manager.validate_model(&invalid_path).await.expect("Failed to validate invalid model"));

        // Empty file
        let empty_path = models_dir.join("empty.gguf");
        fs::write(&empty_path, b"").await.expect("Failed to write empty file");
        assert!(!manager.validate_model(&empty_path).await.expect("Failed to validate empty model"));
    }

    #[tokio::test]
    async fn test_checksum_computation() {
        let temp_dir = tempdir().expect("Failed to create temporary directory for test");
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir).await.expect("Failed to create models directory for test");

        let manager = ModelManager::new(&models_dir);

        let model_path = models_dir.join("test.gguf");
        let test_data = b"test model data for checksum";
        fs::write(&model_path, test_data).await.expect("Failed to write test data for checksum test");

        let checksum = manager.compute_checksum(&model_path).await.expect("Failed to compute checksum");
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // SHA256 hex string length

        // Same data should produce same checksum
        let checksum2 = manager.compute_checksum(&model_path).await.expect("Failed to compute second checksum");
        assert_eq!(checksum, checksum2);
    }
}