#![allow(dead_code, unused_imports, unused_variables)]
use crate::InfernoError;
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Cursor, Read as StdRead};
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub path: PathBuf,
    pub file_path: PathBuf,
    pub size: u64,
    pub size_bytes: u64,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub backend_type: String,
    pub format: String,
    pub checksum: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GgufMetadata {
    pub architecture: String,
    pub parameter_count: u64,
    pub quantization: String,
    pub context_length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnnxMetadata {
    pub version: String,
    pub producer: String,
    pub input_count: u32,
    pub output_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
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

/// Estimated compatibility of a model with the current system
#[derive(Debug, Clone)]
pub struct CompatibilityInfo {
    pub estimated_ram_gb: f64,
    pub available_ram_gb: f64,
    pub is_compatible: bool,
    pub warning: Option<String>,
}

/// Persistent registry tracking metadata, tags, and usage for locally installed models.
/// Stored at `{models_dir}/.inferno_registry.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelRegistry {
    /// Key = canonical absolute path string
    pub entries: HashMap<String, RegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub name: String,
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub use_count: u64,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub added_at: chrono::DateTime<chrono::Utc>,
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

    // ── Discovery ────────────────────────────────────────────────────────────

    /// Recursively scan `models_dir` for GGUF and ONNX model files.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        if !self.models_dir.exists() {
            warn!(
                "Models directory does not exist: {}",
                self.models_dir.display()
            );
            return Ok(Vec::new());
        }

        let mut models = Vec::new();
        let mut dirs_to_scan = vec![self.models_dir.clone()];

        while let Some(dir) = dirs_to_scan.pop() {
            let mut entries = match async_fs::read_dir(&dir).await {
                Ok(e) => e,
                Err(e) => {
                    warn!("Cannot read directory {}: {}", dir.display(), e);
                    continue;
                }
            };

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_dir() {
                    // Skip hidden directories (e.g. .inferno_cache)
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !name.starts_with('.') {
                        dirs_to_scan.push(path);
                    }
                } else if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_lower = ext.to_string_lossy().to_lowercase();
                        if matches!(ext_lower.as_str(), "gguf" | "onnx") {
                            match self.create_model_info(&path).await {
                                Ok(info) => models.push(info),
                                Err(e) => {
                                    error!("Failed to process model {}: {}", path.display(), e)
                                }
                            }
                        }
                    }
                }
            }
        }

        models.sort_by(|a, b| b.modified.cmp(&a.modified));
        info!(
            "Found {} models under {}",
            models.len(),
            self.models_dir.display()
        );
        Ok(models)
    }

    pub async fn resolve_model(&self, model_name_or_path: &str) -> Result<ModelInfo> {
        let path = if model_name_or_path.contains('/') || model_name_or_path.contains('\\') {
            PathBuf::from(model_name_or_path)
        } else {
            self.find_model_by_name(model_name_or_path).await?
        };

        if !path.exists() {
            return Err(anyhow::anyhow!("Model not found: {}", path.display()));
        }

        self.create_model_info(&path).await
    }

    async fn find_model_by_name(&self, name: &str) -> Result<PathBuf> {
        let models = self.list_models().await?;
        for model in &models {
            if model.name == name || model.name.starts_with(name) {
                return Ok(model.path.clone());
            }
        }
        for ext in ["gguf", "onnx"] {
            let p = self.models_dir.join(format!("{}.{}", name, ext));
            if p.exists() {
                return Ok(p);
            }
        }
        Err(anyhow::anyhow!(
            "Model '{}' not found in models directory",
            name
        ))
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
            file_path: path.to_path_buf(),
            size: metadata.len(),
            size_bytes: metadata.len(),
            modified,
            backend_type: backend_type.clone(),
            format: backend_type,
            checksum: None,
            metadata: HashMap::new(),
        })
    }

    fn determine_backend_type(&self, path: &Path) -> String {
        match path.extension().and_then(|e| e.to_str()) {
            Some("gguf") => "gguf".to_string(),
            Some("onnx") => "onnx".to_string(),
            _ => "unknown".to_string(),
        }
    }

    // ── Local search ─────────────────────────────────────────────────────────

    /// Filter locally discovered models by name substring or tag (case-insensitive).
    pub async fn search_local(&self, query: &str) -> Result<Vec<ModelInfo>> {
        let query_lower = query.to_lowercase();
        let all = self.list_models().await?;
        let registry = self.load_registry().await.unwrap_or_default();

        Ok(all
            .into_iter()
            .filter(|m| {
                if m.name.to_lowercase().contains(&query_lower) {
                    return true;
                }
                if let Some(entry) = registry.entries.get(&m.path.to_string_lossy().to_string()) {
                    return entry
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower));
                }
                false
            })
            .collect())
    }

    // ── Metadata ─────────────────────────────────────────────────────────────

    /// Read real GGUF metadata from the file header, falling back to filename heuristics.
    pub async fn get_gguf_metadata(&self, path: &Path) -> Result<GgufMetadata> {
        self.get_or_cache_gguf_metadata(path).await
    }

    /// Read from metadata cache if still fresh; otherwise parse and write cache.
    pub async fn get_or_cache_gguf_metadata(&self, path: &Path) -> Result<GgufMetadata> {
        let cache_path = self.metadata_cache_path(path);

        // Use cache if it exists and is newer than the model file
        if cache_path.exists() {
            let model_mtime = async_fs::metadata(path).await?.modified()?;
            let cache_mtime = async_fs::metadata(&cache_path).await?.modified()?;
            if cache_mtime >= model_mtime {
                if let Ok(data) = async_fs::read_to_string(&cache_path).await {
                    if let Ok(meta) = serde_json::from_str::<GgufMetadata>(&data) {
                        return Ok(meta);
                    }
                }
            }
        }

        // Parse the file header
        let meta = self.parse_gguf_from_file(path).await?;

        // Write to cache
        if let Err(e) = self.write_metadata_cache(&cache_path, &meta).await {
            warn!(
                "Could not write metadata cache for {}: {}",
                path.display(),
                e
            );
        }

        Ok(meta)
    }

    async fn parse_gguf_from_file(&self, path: &Path) -> Result<GgufMetadata> {
        // Read up to 128 KB for the header — enough to cover all KV metadata
        let mut file = async_fs::File::open(path).await?;
        let mut buffer = vec![0u8; 131_072];
        use tokio::io::AsyncReadExt;
        let bytes_read = file.read(&mut buffer).await?;
        buffer.truncate(bytes_read);

        match parse_gguf_kv_metadata(&buffer) {
            Ok(meta) => Ok(meta),
            Err(e) => {
                warn!(
                    "GGUF header parse failed for {}: {}. Using filename heuristics.",
                    path.display(),
                    e
                );
                Ok(infer_gguf_metadata_from_filename(
                    path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                ))
            }
        }
    }

    fn metadata_cache_path(&self, model_path: &Path) -> PathBuf {
        // Use a hash of the full absolute path so same-named models in different
        // subdirectories each get a distinct cache entry.
        let abs = model_path
            .canonicalize()
            .unwrap_or_else(|_| model_path.to_path_buf());
        let hash = {
            let mut h = Sha256::new();
            h.update(abs.to_string_lossy().as_bytes());
            format!("{:x}", h.finalize())
        };
        // Prefix with the bare filename for human readability when browsing the cache.
        let filename = model_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        self.cache_dir()
            .join(format!("{}-{}.json", filename, &hash[..12]))
    }

    fn cache_dir(&self) -> PathBuf {
        self.models_dir.join(".inferno_cache")
    }

    async fn write_metadata_cache<T: Serialize>(&self, path: &PathBuf, value: &T) -> Result<()> {
        let dir = path.parent().unwrap_or(path);
        if !dir.exists() {
            async_fs::create_dir_all(dir).await?;
        }
        let json = serde_json::to_string_pretty(value)?;
        async_fs::write(path, json).await?;
        Ok(())
    }

    pub async fn get_onnx_metadata(&self, path: &Path) -> Result<OnnxMetadata> {
        info!("Reading ONNX metadata from: {}", path.display());
        // ONNX metadata requires full protobuf parsing; return basic stub for now
        Ok(OnnxMetadata {
            version: "1.13.0".to_string(),
            producer: "unknown".to_string(),
            input_count: 1,
            output_count: 1,
        })
    }

    // ── Registry ─────────────────────────────────────────────────────────────

    fn registry_path(&self) -> PathBuf {
        self.models_dir.join(".inferno_registry.json")
    }

    pub async fn load_registry(&self) -> Result<ModelRegistry> {
        let path = self.registry_path();
        if !path.exists() {
            return Ok(ModelRegistry::default());
        }
        let data = async_fs::read_to_string(&path).await?;
        Ok(serde_json::from_str(&data)?)
    }

    pub async fn save_registry(&self, registry: &ModelRegistry) -> Result<()> {
        let path = self.registry_path();
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                async_fs::create_dir_all(parent).await?;
            }
        }
        let json = serde_json::to_string_pretty(registry)?;
        async_fs::write(&path, json).await?;
        Ok(())
    }

    /// Increment the use count and update `last_used` for a model.
    pub async fn record_usage(&self, path: &Path) -> Result<()> {
        let mut registry = self.load_registry().await.unwrap_or_default();
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let key = canonical.to_string_lossy().to_string();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let entry = registry
            .entries
            .entry(key)
            .or_insert_with(|| RegistryEntry {
                name,
                path: path.to_path_buf(),
                tags: Vec::new(),
                use_count: 0,
                last_used: None,
                added_at: chrono::Utc::now(),
            });
        entry.use_count += 1;
        entry.last_used = Some(chrono::Utc::now());

        if let Err(e) = self.save_registry(&registry).await {
            warn!("Could not save registry: {}", e);
        }
        Ok(())
    }

    /// Add (or replace) the tags for a model.
    pub async fn tag_model(&self, path: &Path, tags: &[String]) -> Result<()> {
        let mut registry = self.load_registry().await.unwrap_or_default();
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let key = canonical.to_string_lossy().to_string();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let entry = registry
            .entries
            .entry(key)
            .or_insert_with(|| RegistryEntry {
                name,
                path: path.to_path_buf(),
                tags: Vec::new(),
                use_count: 0,
                last_used: None,
                added_at: chrono::Utc::now(),
            });
        for tag in tags {
            if !entry.tags.contains(tag) {
                entry.tags.push(tag.clone());
            }
        }
        self.save_registry(&registry).await?;
        Ok(())
    }

    /// Register a newly installed model in the registry.
    pub async fn register_model(&self, path: &Path) -> Result<()> {
        let mut registry = self.load_registry().await.unwrap_or_default();
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let key = canonical.to_string_lossy().to_string();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        registry
            .entries
            .entry(key)
            .or_insert_with(|| RegistryEntry {
                name,
                path: path.to_path_buf(),
                tags: Vec::new(),
                use_count: 0,
                last_used: None,
                added_at: chrono::Utc::now(),
            });
        self.save_registry(&registry).await?;
        Ok(())
    }

    // ── Compatibility ─────────────────────────────────────────────────────────

    /// Estimate whether the current system can run this model.
    pub fn check_compatibility(&self, model_info: &ModelInfo) -> CompatibilityInfo {
        // Rough estimate: model file size ≈ RAM needed, plus 20% KV-cache overhead
        let estimated_ram_gb = model_info.size_bytes as f64 / 1_073_741_824.0 * 1.2;

        let available_ram_gb = get_available_ram_gb();

        let is_compatible = available_ram_gb >= estimated_ram_gb;
        let warning = if !is_compatible {
            Some(format!(
                "Model requires ~{:.1} GB RAM but only {:.1} GB available",
                estimated_ram_gb, available_ram_gb
            ))
        } else if estimated_ram_gb > available_ram_gb * 0.8 {
            Some(format!(
                "Model needs ~{:.1} GB RAM; only {:.1} GB free — may be slow",
                estimated_ram_gb, available_ram_gb
            ))
        } else {
            None
        };

        CompatibilityInfo {
            estimated_ram_gb,
            available_ram_gb,
            is_compatible,
            warning,
        }
    }

    // ── Validation ───────────────────────────────────────────────────────────

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

        match async_fs::File::open(path).await {
            Ok(_) => result.file_readable = true,
            Err(e) => {
                result.add_error(format!("Cannot read file: {}", e));
                result.finalize();
                return Ok(result);
            }
        }

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

        if let Some(config) = config {
            let max_size_bytes = if let Some(ref sec) = config.model_security {
                (sec.max_model_size_gb * 1024.0 * 1024.0 * 1024.0) as u64
            } else {
                5 * 1024 * 1024 * 1024
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

        if let Err(e) = self.security_validate(path, &metadata).await {
            result.add_error(format!("Security validation failed: {}", e));
        } else {
            result.security_valid = true;
        }

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

    async fn security_validate(
        &self,
        path: &Path,
        metadata: &std::fs::Metadata,
    ) -> Result<(), InfernoError> {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        let suspicious_patterns = [
            "..", "~", "$", "`", ";", "|", "&", "<", ">", "\\", "script", "exec", "eval", "system",
        ];

        for pattern in &suspicious_patterns {
            if file_name.contains(pattern) {
                return Err(InfernoError::SecurityValidation(format!(
                    "Suspicious filename pattern detected: {}",
                    pattern
                )));
            }
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = metadata.permissions();
            if perms.mode() & 0o111 != 0 {
                return Err(InfernoError::SecurityValidation(
                    "Model file should not be executable".to_string(),
                ));
            }
        }

        let mut file = async_fs::File::open(path).await?;
        let mut buffer = vec![0u8; 4096];
        use tokio::io::AsyncReadExt;
        let bytes_read = file.read(&mut buffer).await?;

        if bytes_read > 0 {
            let content = &buffer[..bytes_read];
            let script_patterns: &[&[u8]] = &[
                b"#!/bin/",
                b"#!/usr/",
                b"<script",
                b"javascript:",
                b"python",
                b"exec(",
            ];
            for pattern in script_patterns {
                if content.windows(pattern.len()).any(|w| w == *pattern) {
                    return Err(InfernoError::SecurityValidation(
                        "Suspicious script content detected in model file".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    async fn validate_format_specific(
        &self,
        path: &Path,
        extension: &str,
    ) -> Result<(bool, String)> {
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
        if &buffer[0..4] != b"GGUF" {
            return Ok((
                false,
                format!(
                    "Invalid GGUF magic bytes. Expected 'GGUF', found {:?}",
                    String::from_utf8_lossy(&buffer[0..4])
                ),
            ));
        }
        let version = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
        if version == 0 || version > 10 {
            return Ok((false, format!("Invalid GGUF version: {}", version)));
        }
        Ok((true, format!("Valid GGUF file, version {}", version)))
    }

    fn validate_onnx_format_detailed(&self, buffer: &[u8]) -> Result<(bool, String)> {
        if buffer.len() < 16 {
            return Ok((false, "File too small to be a valid ONNX file".to_string()));
        }
        let mut has_valid_protobuf = false;
        for i in 0..buffer.len().min(100) {
            if buffer[i] & 0x07 <= 5 {
                has_valid_protobuf = true;
                break;
            }
        }
        if !has_valid_protobuf {
            return Ok((false, "No valid protobuf structure found".to_string()));
        }
        let header_str = String::from_utf8_lossy(&buffer[..buffer.len().min(512)]);
        let has_markers = header_str.contains("onnx")
            || header_str.contains("model_proto")
            || header_str.contains("GraphProto")
            || buffer.windows(4).any(|w| w == b"onnx");
        if !has_markers {
            return Ok((false, "No ONNX markers found in header".to_string()));
        }
        Ok((true, "Valid ONNX file detected".to_string()))
    }

    // ── Checksum ─────────────────────────────────────────────────────────────

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
        Ok(format!("{:x}", hasher.finalize()))
    }
}

// ── GGUF binary parsing ───────────────────────────────────────────────────────

/// Parse the GGUF KV metadata section from the raw file bytes.
/// Returns a populated `GgufMetadata` or an error if the buffer is too short/corrupt.
fn parse_gguf_kv_metadata(data: &[u8]) -> Result<GgufMetadata> {
    let mut cursor = Cursor::new(data);

    // Verify GGUF magic bytes
    if data.len() < 4 || &data[..4] != b"GGUF" {
        return Err(anyhow::anyhow!("Not a GGUF file: invalid magic bytes"));
    }
    cursor.set_position(4);
    let _version = cursor.read_u32::<LittleEndian>()?;
    let _n_tensors = cursor.read_u64::<LittleEndian>()?;
    let n_kv = cursor.read_u64::<LittleEndian>()?;

    if n_kv > 2048 {
        return Err(anyhow::anyhow!("Unreasonable n_kv count: {}", n_kv));
    }

    let mut architecture = String::new();
    let mut parameter_count = 0u64;
    let mut context_length = 0u32;
    let mut quantization = String::new();

    for _ in 0..n_kv {
        // Read key
        let key_len = cursor.read_u64::<LittleEndian>()? as usize;
        if key_len == 0 || key_len > 512 {
            break; // sanity guard
        }
        let mut key_bytes = vec![0u8; key_len];
        cursor.read_exact(&mut key_bytes)?;
        let key = String::from_utf8_lossy(&key_bytes).to_string();

        // Read value type and value
        let value_type = cursor.read_u32::<LittleEndian>()?;
        let value_str = read_gguf_value(&mut cursor, value_type)?;

        match key.as_str() {
            "general.architecture" => architecture = value_str,
            "general.parameter_count" => {
                parameter_count = value_str.parse().unwrap_or(0);
            }
            "general.file_type" => {
                quantization = gguf_file_type_to_str(value_str.parse().unwrap_or(1));
            }
            k if k.ends_with(".context_length") => {
                if context_length == 0 {
                    context_length = value_str.parse().unwrap_or(0);
                }
            }
            _ => {}
        }

        // Early exit once we have all four fields
        if !architecture.is_empty()
            && parameter_count > 0
            && context_length > 0
            && !quantization.is_empty()
        {
            break;
        }
    }

    Ok(GgufMetadata {
        architecture: if architecture.is_empty() {
            "unknown".to_string()
        } else {
            architecture
        },
        parameter_count,
        quantization: if quantization.is_empty() {
            "F16".to_string()
        } else {
            quantization
        },
        context_length: if context_length == 0 {
            2048
        } else {
            context_length
        },
    })
}

/// Read a single GGUF value and return it as a string.
/// Advances the cursor past the value regardless of whether we care about it.
fn read_gguf_value(cursor: &mut Cursor<&[u8]>, value_type: u32) -> Result<String> {
    match value_type {
        0 => Ok(cursor.read_u8()?.to_string()),
        1 => Ok(cursor.read_i8()?.to_string()),
        2 => Ok(cursor.read_u16::<LittleEndian>()?.to_string()),
        3 => Ok(cursor.read_i16::<LittleEndian>()?.to_string()),
        4 => Ok(cursor.read_u32::<LittleEndian>()?.to_string()),
        5 => Ok(cursor.read_i32::<LittleEndian>()?.to_string()),
        6 => Ok(cursor.read_f32::<LittleEndian>()?.to_string()),
        7 => Ok((cursor.read_u8()? != 0).to_string()),
        8 => {
            let len = cursor.read_u64::<LittleEndian>()? as usize;
            if len > 65536 {
                let pos = cursor.position() + len as u64;
                cursor.set_position(pos);
                return Ok(String::new());
            }
            let mut bytes = vec![0u8; len];
            cursor.read_exact(&mut bytes)?;
            Ok(String::from_utf8_lossy(&bytes).to_string())
        }
        9 => {
            let elem_type = cursor.read_u32::<LittleEndian>()?;
            let count = cursor.read_u64::<LittleEndian>()?;
            if count > 10_000_000 {
                return Err(anyhow::anyhow!("GGUF array count {} exceeds sanity limit", count));
            }
            for _ in 0..count {
                skip_gguf_value(cursor, elem_type)?;
            }
            Ok(String::new())
        }
        10 => Ok(cursor.read_u64::<LittleEndian>()?.to_string()),
        11 => Ok(cursor.read_i64::<LittleEndian>()?.to_string()),
        12 => Ok(cursor.read_f64::<LittleEndian>()?.to_string()),
        _ => Err(anyhow::anyhow!("Unknown GGUF value type: {}", value_type)),
    }
}

/// Skip a GGUF value without converting it to a string.
fn skip_gguf_value(cursor: &mut Cursor<&[u8]>, value_type: u32) -> Result<()> {
    match value_type {
        0 | 1 | 7 => {
            cursor.read_u8()?;
        }
        2 | 3 => {
            cursor.read_u16::<LittleEndian>()?;
        }
        4..=6 => {
            cursor.read_u32::<LittleEndian>()?;
        }
        8 => {
            let len = cursor.read_u64::<LittleEndian>()? as usize;
            let pos = cursor.position() + len as u64;
            cursor.set_position(pos);
        }
        9 => {
            let elem_type = cursor.read_u32::<LittleEndian>()?;
            let count = cursor.read_u64::<LittleEndian>()?;
            if count > 10_000_000 {
                return Err(anyhow::anyhow!("GGUF array count {} exceeds sanity limit", count));
            }
            for _ in 0..count {
                skip_gguf_value(cursor, elem_type)?;
            }
        }
        10..=12 => {
            cursor.read_u64::<LittleEndian>()?;
        }
        _ => return Err(anyhow::anyhow!("Unknown GGUF value type: {}", value_type)),
    }
    Ok(())
}

/// Map GGUF `general.file_type` integer to a human-readable quantization string.
fn gguf_file_type_to_str(file_type: u32) -> String {
    match file_type {
        0 => "F32",
        1 => "F16",
        2 => "Q4_0",
        3 => "Q4_1",
        7 => "Q8_0",
        8 => "Q5_0",
        9 => "Q5_1",
        10 => "Q2_K",
        11 => "Q3_K_S",
        12 => "Q3_K_M",
        13 => "Q3_K_L",
        14 => "Q4_K_S",
        15 => "Q4_K_M",
        16 => "Q5_K_S",
        17 => "Q5_K_M",
        18 => "Q6_K",
        _ => "unknown",
    }
    .to_string()
}

/// Fallback: derive rough metadata from filename patterns.
fn infer_gguf_metadata_from_filename(name: &str) -> GgufMetadata {
    let lower = name.to_lowercase();

    let parameter_count = if lower.contains("70b") {
        70_000_000_000
    } else if lower.contains("34b") {
        34_000_000_000
    } else if lower.contains("30b") {
        30_000_000_000
    } else if lower.contains("13b") {
        13_000_000_000
    } else if lower.contains("8b") {
        8_000_000_000
    } else if lower.contains("7b") {
        7_000_000_000
    } else if lower.contains("3b") {
        3_000_000_000
    } else if lower.contains("1b") {
        1_000_000_000
    } else {
        7_000_000_000
    };

    let quantization = if lower.contains("q4_k_m") {
        "Q4_K_M"
    } else if lower.contains("q4_k_s") {
        "Q4_K_S"
    } else if lower.contains("q5_k_m") {
        "Q5_K_M"
    } else if lower.contains("q5_k_s") {
        "Q5_K_S"
    } else if lower.contains("q6_k") {
        "Q6_K"
    } else if lower.contains("q4_0") {
        "Q4_0"
    } else if lower.contains("q4_1") {
        "Q4_1"
    } else if lower.contains("q5_0") {
        "Q5_0"
    } else if lower.contains("q5_1") {
        "Q5_1"
    } else if lower.contains("q8_0") {
        "Q8_0"
    } else if lower.contains("q2_k") {
        "Q2_K"
    } else if lower.contains("f16") {
        "F16"
    } else {
        "unknown"
    }
    .to_string();

    let architecture = if lower.contains("llama") || lower.contains("meta-llama") {
        "llama"
    } else if lower.contains("mistral") {
        "mistral"
    } else if lower.contains("falcon") {
        "falcon"
    } else if lower.contains("gpt") {
        "gpt2"
    } else if lower.contains("phi") {
        "phi"
    } else if lower.contains("gemma") {
        "gemma"
    } else {
        "unknown"
    }
    .to_string();

    GgufMetadata {
        architecture,
        parameter_count,
        quantization,
        context_length: 4096,
    }
}

// ── Convenience top-level functions ──────────────────────────────────────────

/// Record that a model at `model_path` was used, without needing a pre-built `ModelManager`.
/// Finds the `models_dir` by walking up from the model path to locate the registry.
/// This is a best-effort operation; errors are logged but not propagated.
pub async fn record_model_usage(model_path: &Path) {
    let models_dir = infer_models_dir(model_path);
    let manager = ModelManager::new(&models_dir);
    if let Err(e) = manager.record_usage(model_path).await {
        warn!(
            "Failed to record model usage for {}: {}",
            model_path.display(),
            e
        );
    }
}

/// Walk up from `model_path` to find the directory containing `.inferno_registry.json`
/// or an `.inferno_cache` subdirectory. Falls back to the immediate parent directory.
fn infer_models_dir(model_path: &Path) -> PathBuf {
    let mut candidate = model_path.parent().unwrap_or(model_path);
    for _ in 0..5 {
        if candidate.join(".inferno_registry.json").exists()
            || candidate.join(".inferno_cache").is_dir()
        {
            return candidate.to_path_buf();
        }
        match candidate.parent() {
            Some(p) => candidate = p,
            None => break,
        }
    }
    // Fallback: immediate parent
    model_path.parent().unwrap_or(model_path).to_path_buf()
}

// ── System helpers ────────────────────────────────────────────────────────────

fn get_available_ram_gb() -> f64 {
    use sysinfo::{System, SystemExt};
    let mut sys = System::new();
    sys.refresh_memory();
    // sysinfo 0.29+ returns memory in bytes
    sys.available_memory() as f64 / 1_073_741_824.0
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_model_manager() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir).await.unwrap();

        let manager = ModelManager::new(&models_dir);
        let models = manager.list_models().await.unwrap();
        assert!(models.is_empty());

        let model_path = models_dir.join("test_model.gguf");
        fs::write(&model_path, b"GGUF\x03\x00\x00\x00mock data")
            .await
            .unwrap();

        let models = manager.list_models().await.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "test_model.gguf");
        assert_eq!(models[0].backend_type, "gguf");
    }

    #[tokio::test]
    async fn test_recursive_discovery() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let models_dir = temp_dir.path().join("models");
        let subdir = models_dir.join("subfolder");
        fs::create_dir_all(&subdir).await.unwrap();

        let manager = ModelManager::new(&models_dir);
        fs::write(models_dir.join("top.gguf"), b"GGUF\x03\x00\x00\x00data")
            .await
            .unwrap();
        fs::write(subdir.join("nested.gguf"), b"GGUF\x03\x00\x00\x00data")
            .await
            .unwrap();

        let models = manager.list_models().await.unwrap();
        assert_eq!(models.len(), 2);
    }

    #[tokio::test]
    async fn test_model_validation() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir).await.unwrap();

        let manager = ModelManager::new(&models_dir);

        let gguf_path = models_dir.join("valid.gguf");
        fs::write(&gguf_path, b"GGUF\x03\x00\x00\x00mock data")
            .await
            .unwrap();
        assert!(manager.validate_model(&gguf_path).await.unwrap());

        let invalid_path = models_dir.join("invalid.gguf");
        fs::write(&invalid_path, b"INVALID_DATA").await.unwrap();
        assert!(!manager.validate_model(&invalid_path).await.unwrap());

        let empty_path = models_dir.join("empty.gguf");
        fs::write(&empty_path, b"").await.unwrap();
        assert!(!manager.validate_model(&empty_path).await.unwrap());
    }

    #[tokio::test]
    async fn test_checksum_computation() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir).await.unwrap();

        let manager = ModelManager::new(&models_dir);
        let model_path = models_dir.join("test.gguf");
        fs::write(&model_path, b"test model data for checksum")
            .await
            .unwrap();

        let checksum = manager.compute_checksum(&model_path).await.unwrap();
        assert_eq!(checksum.len(), 64);
        let checksum2 = manager.compute_checksum(&model_path).await.unwrap();
        assert_eq!(checksum, checksum2);
    }

    #[test]
    fn test_gguf_file_type_to_str() {
        assert_eq!(gguf_file_type_to_str(1), "F16");
        assert_eq!(gguf_file_type_to_str(2), "Q4_0");
        assert_eq!(gguf_file_type_to_str(15), "Q4_K_M");
        assert_eq!(gguf_file_type_to_str(99), "unknown");
    }

    #[test]
    fn test_infer_gguf_metadata_from_filename() {
        let meta = infer_gguf_metadata_from_filename("llama-2-7b-chat.Q4_K_M.gguf");
        assert_eq!(meta.architecture, "llama");
        assert_eq!(meta.parameter_count, 7_000_000_000);
        assert_eq!(meta.quantization, "Q4_K_M");

        let meta = infer_gguf_metadata_from_filename("mistral-7b-instruct-v0.2.Q5_K_M.gguf");
        assert_eq!(meta.architecture, "mistral");
        assert_eq!(meta.quantization, "Q5_K_M");
    }

    #[tokio::test]
    async fn test_registry_tags_and_usage() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir).await.unwrap();
        let manager = ModelManager::new(&models_dir);

        let model_path = models_dir.join("test.gguf");
        fs::write(&model_path, b"GGUF\x03\x00\x00\x00data")
            .await
            .unwrap();

        // Tag a model
        manager
            .tag_model(&model_path, &["chat".to_string(), "assistant".to_string()])
            .await
            .unwrap();

        let canonical_key = model_path
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let registry = manager.load_registry().await.unwrap();
        let entry = registry.entries.get(&canonical_key).unwrap();
        assert!(entry.tags.contains(&"chat".to_string()));
        assert_eq!(entry.use_count, 0);

        // Record usage
        manager.record_usage(&model_path).await.unwrap();
        manager.record_usage(&model_path).await.unwrap();

        let registry = manager.load_registry().await.unwrap();
        let entry = registry.entries.get(&canonical_key).unwrap();
        assert_eq!(entry.use_count, 2);
        assert!(entry.last_used.is_some());
    }

    #[tokio::test]
    async fn test_search_local() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir).await.unwrap();
        let manager = ModelManager::new(&models_dir);

        fs::write(
            models_dir.join("llama-7b.gguf"),
            b"GGUF\x03\x00\x00\x00data",
        )
        .await
        .unwrap();
        fs::write(
            models_dir.join("mistral-7b.gguf"),
            b"GGUF\x03\x00\x00\x00data",
        )
        .await
        .unwrap();

        let results = manager.search_local("llama").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "llama-7b.gguf");
    }
}
