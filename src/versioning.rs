#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    clippy::inherent_to_string
)]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};
use tokio::{fs, sync::RwLock};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    pub id: String,
    pub model_name: String,
    pub version: String,
    pub semantic_version: SemanticVersion,
    pub file_path: PathBuf,
    pub checksum: String,
    pub size_bytes: u64,
    pub metadata: ModelMetadata,
    pub created_at: SystemTime,
    pub created_by: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub status: VersionStatus,
    pub parent_version: Option<String>,
    pub deployment_info: Option<DeploymentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
    pub build_metadata: Option<String>,
}

impl SemanticVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build_metadata: None,
        }
    }

    pub fn to_string(&self) -> String {
        let mut version = format!("{}.{}.{}", self.major, self.minor, self.patch);

        if let Some(ref pre) = self.pre_release {
            version.push_str(&format!("-{}", pre));
        }

        if let Some(ref build) = self.build_metadata {
            version.push_str(&format!("+{}", build));
        }

        version
    }

    pub fn from_string(version_str: &str) -> Result<Self> {
        // Parse semantic version string like "1.2.3-alpha+build123"
        let (version_part, build_metadata) = if let Some(plus_pos) = version_str.find('+') {
            (
                &version_str[..plus_pos],
                Some(version_str[plus_pos + 1..].to_string()),
            )
        } else {
            (version_str, None)
        };

        let (version_part, pre_release) = if let Some(dash_pos) = version_part.find('-') {
            (
                &version_part[..dash_pos],
                Some(version_part[dash_pos + 1..].to_string()),
            )
        } else {
            (version_part, None)
        };

        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid version format: {}", version_str));
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Invalid major version: {}", parts[0]))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Invalid minor version: {}", parts[1]))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Invalid patch version: {}", parts[2]))?;

        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
            build_metadata,
        })
    }

    pub fn compare(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }

        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }

        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Pre-release versions have lower precedence
        match (&self.pre_release, &other.pre_release) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }

    pub fn is_compatible(&self, other: &Self) -> bool {
        // Compatible if major version is the same
        self.major == other.major
    }

    pub fn next_patch(&self) -> Self {
        Self {
            major: self.major,
            minor: self.minor,
            patch: self.patch + 1,
            pre_release: None,
            build_metadata: None,
        }
    }

    pub fn next_minor(&self) -> Self {
        Self {
            major: self.major,
            minor: self.minor + 1,
            patch: 0,
            pre_release: None,
            build_metadata: None,
        }
    }

    pub fn next_major(&self) -> Self {
        Self {
            major: self.major + 1,
            minor: 0,
            patch: 0,
            pre_release: None,
            build_metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_type: String,
    pub architecture: String,
    pub framework: String,
    pub framework_version: String,
    pub parameters_count: Option<u64>,
    pub file_format: String,
    pub training_info: Option<TrainingInfo>,
    pub performance_metrics: HashMap<String, f64>,
    pub custom_metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingInfo {
    pub dataset_name: Option<String>,
    pub dataset_size: Option<u64>,
    pub training_duration_hours: Option<f64>,
    pub epochs: Option<u32>,
    pub learning_rate: Option<f64>,
    pub batch_size: Option<u32>,
    pub optimizer: Option<String>,
    pub loss_function: Option<String>,
    pub validation_accuracy: Option<f64>,
    pub final_loss: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionStatus {
    Draft,
    Testing,
    Staging,
    Production,
    Deprecated,
    Archived,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub deployed_at: SystemTime,
    pub deployed_by: String,
    pub deployment_id: String,
    pub environment: String,
    pub deployment_config: HashMap<String, serde_json::Value>,
    pub health_check_passed: bool,
    pub performance_baseline: Option<PerformanceBaseline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub avg_response_time_ms: f64,
    pub throughput_rps: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub accuracy_score: Option<f64>,
    pub measured_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    pub models: HashMap<String, ModelVersionHistory>,
    pub active_deployments: HashMap<String, ActiveDeployment>,
    pub rollback_history: Vec<RollbackRecord>,
    pub registry_metadata: RegistryMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersionHistory {
    pub model_name: String,
    pub versions: Vec<ModelVersion>,
    pub current_production_version: Option<String>,
    pub current_staging_version: Option<String>,
    pub latest_version: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveDeployment {
    pub model_name: String,
    pub version_id: String,
    pub environment: String,
    pub deployed_at: SystemTime,
    pub deployment_config: HashMap<String, serde_json::Value>,
    pub health_status: DeploymentHealth,
    pub performance_metrics: Option<PerformanceBaseline>,
    pub auto_rollback_enabled: bool,
    pub rollback_triggers: Vec<RollbackTrigger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackTrigger {
    pub trigger_type: TriggerType,
    pub threshold: f64,
    pub measurement_window_minutes: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    ErrorRate,
    ResponseTime,
    Throughput,
    Accuracy,
    CustomMetric(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRecord {
    pub id: String,
    pub model_name: String,
    pub from_version: String,
    pub to_version: String,
    pub environment: String,
    pub reason: RollbackReason,
    pub triggered_by: String,
    pub triggered_at: SystemTime,
    pub completed_at: Option<SystemTime>,
    pub status: RollbackStatus,
    pub rollback_metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackReason {
    Manual,
    AutoTriggered(TriggerType),
    HealthCheck,
    Emergency,
    Scheduled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackStatus {
    Initiated,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetadata {
    pub created_at: SystemTime,
    pub last_updated: SystemTime,
    pub version: String,
    pub total_models: usize,
    pub total_versions: usize,
    pub storage_path: PathBuf,
}

/// Configuration for creating a new model version
/// Reduces create_version() signature from 8 parameters to 4
pub struct CreateVersionConfig {
    pub version: Option<SemanticVersion>,
    pub metadata: ModelMetadata,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub created_by: String,
}

pub struct ModelVersionManager {
    registry: Arc<RwLock<ModelRegistry>>,
    storage_path: PathBuf,
    config: VersioningConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersioningConfig {
    pub storage_path: PathBuf,
    pub auto_backup_enabled: bool,
    pub backup_retention_days: u32,
    pub checksum_algorithm: ChecksumAlgorithm,
    pub compression_enabled: bool,
    pub auto_cleanup_enabled: bool,
    pub max_versions_per_model: Option<u32>,
    pub rollback_timeout_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    Md5,
    Sha256,
    Sha512,
}

impl Default for VersioningConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from("./models/versions"),
            auto_backup_enabled: true,
            backup_retention_days: 30,
            checksum_algorithm: ChecksumAlgorithm::Sha256,
            compression_enabled: false,
            auto_cleanup_enabled: true,
            max_versions_per_model: Some(50),
            rollback_timeout_minutes: 30,
        }
    }
}

impl ModelVersionManager {
    pub async fn new(config: VersioningConfig) -> Result<Self> {
        // Ensure storage directory exists
        fs::create_dir_all(&config.storage_path).await?;

        let registry_path = config.storage_path.join("registry.json");
        let registry = if registry_path.exists() {
            let content = fs::read_to_string(&registry_path).await?;
            serde_json::from_str::<ModelRegistry>(&content)?
        } else {
            ModelRegistry {
                models: HashMap::new(),
                active_deployments: HashMap::new(),
                rollback_history: Vec::new(),
                registry_metadata: RegistryMetadata {
                    created_at: SystemTime::now(),
                    last_updated: SystemTime::now(),
                    version: "1.0.0".to_string(),
                    total_models: 0,
                    total_versions: 0,
                    storage_path: config.storage_path.clone(),
                },
            }
        };

        Ok(Self {
            registry: Arc::new(RwLock::new(registry)),
            storage_path: config.storage_path.clone(),
            config,
        })
    }

    pub async fn create_version(
        &self,
        model_name: &str,
        model_file: &Path,
        config: CreateVersionConfig,
    ) -> Result<String> {
        // Calculate checksum
        let checksum = self.calculate_checksum(model_file).await?;

        // Get file size
        let size_bytes = fs::metadata(model_file).await?.len();

        // Determine version number
        let semantic_version = if let Some(v) = config.version {
            v
        } else {
            self.auto_increment_version(model_name).await?
        };

        // Create version ID
        let version_id = Uuid::new_v4().to_string();

        // Copy model file to versioned storage
        let version_dir = self.storage_path.join(model_name).join(&version_id);
        fs::create_dir_all(&version_dir).await?;

        let stored_file_path = version_dir.join(
            model_file
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?,
        );
        fs::copy(model_file, &stored_file_path).await?;

        // Create model version
        let model_version = ModelVersion {
            id: version_id.clone(),
            model_name: model_name.to_string(),
            version: semantic_version.to_string(),
            semantic_version,
            file_path: stored_file_path,
            checksum,
            size_bytes,
            metadata: config.metadata,
            created_at: SystemTime::now(),
            created_by: config.created_by,
            description: config.description,
            tags: config.tags,
            status: VersionStatus::Draft,
            parent_version: self.get_latest_version(model_name).await.ok(),
            deployment_info: None,
        };

        // Add to registry
        {
            let mut registry = self.registry.write().await;

            if let Some(history) = registry.models.get_mut(model_name) {
                history.versions.push(model_version.clone());
                history.latest_version = version_id.clone();
                history.updated_at = SystemTime::now();
            } else {
                let history = ModelVersionHistory {
                    model_name: model_name.to_string(),
                    versions: vec![model_version.clone()],
                    current_production_version: None,
                    current_staging_version: None,
                    latest_version: version_id.clone(),
                    created_at: SystemTime::now(),
                    updated_at: SystemTime::now(),
                };
                registry.models.insert(model_name.to_string(), history);
            }

            registry.registry_metadata.total_models = registry.models.len();
            registry.registry_metadata.total_versions =
                registry.models.values().map(|h| h.versions.len()).sum();
            registry.registry_metadata.last_updated = SystemTime::now();
        }

        // Save registry
        self.save_registry().await?;

        info!("Created model version {} for {}", version_id, model_name);
        Ok(version_id)
    }

    async fn auto_increment_version(&self, model_name: &str) -> Result<SemanticVersion> {
        let registry = self.registry.read().await;

        if let Some(history) = registry.models.get(model_name) {
            if let Some(latest) = history.versions.last() {
                return Ok(latest.semantic_version.next_patch());
            }
        }

        Ok(SemanticVersion::new(1, 0, 0))
    }

    async fn calculate_checksum(&self, file_path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};

        let content = fs::read(file_path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    pub async fn promote_version(
        &self,
        model_name: &str,
        version_id: &str,
        target_status: VersionStatus,
        _promoted_by: String,
    ) -> Result<()> {
        let mut registry = self.registry.write().await;

        let history = registry
            .models
            .get_mut(model_name)
            .ok_or_else(|| anyhow::anyhow!("Model '{}' not found", model_name))?;

        let version = history
            .versions
            .iter_mut()
            .find(|v| v.id == version_id)
            .ok_or_else(|| anyhow::anyhow!("Version '{}' not found", version_id))?;

        let old_status = version.status.clone();
        version.status = target_status.clone();

        // Update current version pointers
        match target_status {
            VersionStatus::Production => {
                history.current_production_version = Some(version_id.to_string());
            }
            VersionStatus::Staging => {
                history.current_staging_version = Some(version_id.to_string());
            }
            _ => {}
        }

        history.updated_at = SystemTime::now();
        registry.registry_metadata.last_updated = SystemTime::now();

        drop(registry);
        self.save_registry().await?;

        info!(
            "Promoted version {} of {} from {:?} to {:?}",
            version_id, model_name, old_status, target_status
        );
        Ok(())
    }

    pub async fn rollback_model(
        &self,
        model_name: &str,
        target_version_id: &str,
        environment: &str,
        reason: RollbackReason,
        triggered_by: String,
    ) -> Result<String> {
        let rollback_id = Uuid::new_v4().to_string();

        // Get current deployment
        let current_version = {
            let registry = self.registry.read().await;
            registry
                .active_deployments
                .get(&format!("{}:{}", model_name, environment))
                .map(|d| d.version_id.clone())
        };

        let current_version = current_version.ok_or_else(|| {
            anyhow::anyhow!(
                "No active deployment found for {} in {}",
                model_name,
                environment
            )
        })?;

        // Validate target version exists
        self.get_version(model_name, target_version_id).await?;

        // Create rollback record
        let rollback_record = RollbackRecord {
            id: rollback_id.clone(),
            model_name: model_name.to_string(),
            from_version: current_version,
            to_version: target_version_id.to_string(),
            environment: environment.to_string(),
            reason,
            triggered_by,
            triggered_at: SystemTime::now(),
            completed_at: None,
            status: RollbackStatus::Initiated,
            rollback_metadata: HashMap::new(),
        };

        // Add to registry
        {
            let mut registry = self.registry.write().await;
            registry.rollback_history.push(rollback_record);
        }

        // Perform the rollback (update active deployment)
        self.deploy_version(model_name, target_version_id, environment, HashMap::new())
            .await?;

        // Update rollback status
        {
            let mut registry = self.registry.write().await;
            if let Some(record) = registry
                .rollback_history
                .iter_mut()
                .find(|r| r.id == rollback_id)
            {
                record.status = RollbackStatus::Completed;
                record.completed_at = Some(SystemTime::now());
            }
        }

        self.save_registry().await?;

        info!(
            "Completed rollback {} for {} in {} to version {}",
            rollback_id, model_name, environment, target_version_id
        );
        Ok(rollback_id)
    }

    pub async fn deploy_version(
        &self,
        model_name: &str,
        version_id: &str,
        environment: &str,
        deployment_config: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        // Validate version exists and is deployable
        let version = self.get_version(model_name, version_id).await?;

        match version.status {
            VersionStatus::Draft => return Err(anyhow::anyhow!("Cannot deploy draft version")),
            VersionStatus::Failed => return Err(anyhow::anyhow!("Cannot deploy failed version")),
            VersionStatus::Archived => {
                return Err(anyhow::anyhow!("Cannot deploy archived version"))
            }
            _ => {}
        }

        let deployment_key = format!("{}:{}", model_name, environment);
        let deployment = ActiveDeployment {
            model_name: model_name.to_string(),
            version_id: version_id.to_string(),
            environment: environment.to_string(),
            deployed_at: SystemTime::now(),
            deployment_config,
            health_status: DeploymentHealth::Unknown,
            performance_metrics: None,
            auto_rollback_enabled: false,
            rollback_triggers: Vec::new(),
        };

        {
            let mut registry = self.registry.write().await;
            registry
                .active_deployments
                .insert(deployment_key, deployment);
            registry.registry_metadata.last_updated = SystemTime::now();
        }

        self.save_registry().await?;

        info!(
            "Deployed version {} of {} to {}",
            version_id, model_name, environment
        );
        Ok(())
    }

    pub async fn get_version(&self, model_name: &str, version_id: &str) -> Result<ModelVersion> {
        let registry = self.registry.read().await;

        let history = registry
            .models
            .get(model_name)
            .ok_or_else(|| anyhow::anyhow!("Model '{}' not found", model_name))?;

        history
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Version '{}' not found", version_id))
    }

    pub async fn get_latest_version(&self, model_name: &str) -> Result<String> {
        let registry = self.registry.read().await;

        registry
            .models
            .get(model_name)
            .map(|h| h.latest_version.clone())
            .ok_or_else(|| anyhow::anyhow!("Model '{}' not found", model_name))
    }

    pub async fn list_versions(&self, model_name: &str) -> Result<Vec<ModelVersion>> {
        let registry = self.registry.read().await;

        registry
            .models
            .get(model_name)
            .map(|h| h.versions.clone())
            .ok_or_else(|| anyhow::anyhow!("Model '{}' not found", model_name))
    }

    pub async fn list_models(&self) -> Vec<String> {
        let registry = self.registry.read().await;
        registry.models.keys().cloned().collect()
    }

    pub async fn delete_version(&self, model_name: &str, version_id: &str) -> Result<()> {
        // Check if version is in use
        {
            let registry = self.registry.read().await;
            for deployment in registry.active_deployments.values() {
                if deployment.model_name == model_name && deployment.version_id == version_id {
                    return Err(anyhow::anyhow!(
                        "Cannot delete version {} - it's currently deployed",
                        version_id
                    ));
                }
            }
        }

        // Get version info before deletion
        let version = self.get_version(model_name, version_id).await?;

        // Remove from registry
        {
            let mut registry = self.registry.write().await;
            if let Some(history) = registry.models.get_mut(model_name) {
                history.versions.retain(|v| v.id != version_id);

                // Update latest version if this was it
                if history.latest_version == version_id {
                    history.latest_version = history
                        .versions
                        .last()
                        .map(|v| v.id.clone())
                        .unwrap_or_default();
                }

                // Clear current version pointers if they point to deleted version
                if history.current_production_version.as_ref() == Some(&version_id.to_string()) {
                    history.current_production_version = None;
                }
                if history.current_staging_version.as_ref() == Some(&version_id.to_string()) {
                    history.current_staging_version = None;
                }

                history.updated_at = SystemTime::now();
            }
        }

        // Delete files
        if let Some(parent) = version.file_path.parent() {
            if parent.file_name().and_then(|n| n.to_str()) == Some(version_id) {
                fs::remove_dir_all(parent).await?;
            }
        }

        self.save_registry().await?;

        info!("Deleted version {} of {}", version_id, model_name);
        Ok(())
    }

    async fn save_registry(&self) -> Result<()> {
        let registry = self.registry.read().await;
        let registry_path = self.storage_path.join("registry.json");
        let content = serde_json::to_string_pretty(&*registry)?;
        fs::write(registry_path, content).await?;
        Ok(())
    }

    pub async fn get_registry_info(&self) -> RegistryMetadata {
        let registry = self.registry.read().await;
        registry.registry_metadata.clone()
    }

    pub async fn get_rollback_history(&self, model_name: Option<&str>) -> Vec<RollbackRecord> {
        let registry = self.registry.read().await;

        if let Some(model_name) = model_name {
            registry
                .rollback_history
                .iter()
                .filter(|r| r.model_name == model_name)
                .cloned()
                .collect()
        } else {
            registry.rollback_history.clone()
        }
    }

    pub async fn get_active_deployments(&self) -> HashMap<String, ActiveDeployment> {
        let registry = self.registry.read().await;
        registry.active_deployments.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_semantic_version_parsing() {
        let version = SemanticVersion::from_string("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, None);
        assert_eq!(version.build_metadata, None);

        let version = SemanticVersion::from_string("2.0.0-alpha+build123").unwrap();
        assert_eq!(version.major, 2);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
        assert_eq!(version.pre_release, Some("alpha".to_string()));
        assert_eq!(version.build_metadata, Some("build123".to_string()));
    }

    #[test]
    fn test_version_comparison() {
        let v1 = SemanticVersion::new(1, 0, 0);
        let v2 = SemanticVersion::new(1, 0, 1);
        let v3 = SemanticVersion::new(1, 1, 0);
        let v4 = SemanticVersion::new(2, 0, 0);

        assert_eq!(v1.compare(&v2), std::cmp::Ordering::Less);
        assert_eq!(v2.compare(&v3), std::cmp::Ordering::Less);
        assert_eq!(v3.compare(&v4), std::cmp::Ordering::Less);
        assert_eq!(v1.compare(&v1), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_version_compatibility() {
        let v1 = SemanticVersion::new(1, 0, 0);
        let v2 = SemanticVersion::new(1, 1, 0);
        let v3 = SemanticVersion::new(2, 0, 0);

        assert!(v1.is_compatible(&v2));
        assert!(!v1.is_compatible(&v3));
    }

    #[tokio::test]
    async fn test_version_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let config = VersioningConfig {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = ModelVersionManager::new(config).await.unwrap();
        let models = manager.list_models().await;
        assert!(models.is_empty());
    }
}
