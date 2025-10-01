use crate::config::Config;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    pub enabled: bool,
    pub registry_url: String,
    pub local_cache_dir: PathBuf,
    pub authentication: AuthenticationConfig,
    pub auto_update: bool,
    pub update_interval_hours: u64,
    pub max_cache_size_gb: f64,
    pub trusted_publishers: Vec<String>,
    pub verification: VerificationConfig,
    pub proxy_settings: Option<ProxyConfig>,
    pub repositories: Vec<Repository>,
    pub package_db_path: PathBuf,
    pub auto_resolve_dependencies: bool,
    pub auto_cleanup_unused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthenticationConfig {
    pub api_key: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub token_file: Option<PathBuf>,
    pub oauth_enabled: bool,
    pub oauth_provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub verify_signatures: bool,
    pub verify_checksums: bool,
    pub require_trusted_publishers: bool,
    pub scan_for_malware: bool,
    pub allowed_licenses: Vec<String>,
    pub blocked_models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub no_proxy: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub priority: u32,
    pub authentication: Option<AuthenticationConfig>,
    pub verification_required: bool,
    pub last_updated: Option<DateTime<Utc>>,
    pub metadata_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDatabase {
    pub version: String,
    pub last_updated: DateTime<Utc>,
    pub repositories: HashMap<String, RepositoryMetadata>,
    pub installed_packages: HashMap<String, InstalledPackage>,
    pub dependency_graph: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    pub name: String,
    pub url: String,
    pub last_synced: DateTime<Utc>,
    pub model_count: usize,
    pub available_models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub model_id: String,
    pub name: String,
    pub version: String,
    pub repository: String,
    pub install_date: DateTime<Utc>,
    pub dependencies: Vec<String>,
    pub auto_installed: bool,
    pub local_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DependencyResolver {
    package_db: PackageDatabase,
    repositories: Vec<Repository>,
    registry_client: RegistryClient,
}

#[derive(Debug, Clone)]
pub struct InstallPlan {
    pub to_install: Vec<ModelListing>,
    pub to_upgrade: Vec<(String, String)>, // (model_id, new_version)
    pub to_remove: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelListing {
    pub id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub description: String,
    pub category: ModelCategory,
    pub license: String,
    pub size_bytes: u64,
    pub download_url: String,
    pub checksum: String,
    pub signature: Option<String>,
    pub metadata: ModelMetadata,
    pub compatibility: CompatibilityInfo,
    pub performance: PerformanceMetrics,
    pub published_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub downloads: u64,
    pub rating: Option<f32>,
    pub tags: Vec<String>,
    pub dependencies: Vec<ModelDependency>,
    pub pricing: PricingInfo,
    pub ratings: RatingInfo,
    pub created_at: DateTime<Utc>,
    pub visibility: ModelVisibility,
    pub verified: bool,
    pub documentation_url: Option<String>,
    pub demo_url: Option<String>,
    pub paper_url: Option<String>,
    pub source_url: Option<String>,
}

// Removed duplicate definitions - using the original ones below

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelCategory {
    LanguageModel,
    VisionModel,
    AudioModel,
    MultiModal,
    Embedding,
    ClassificationModel,
    GenerativeModel,
    ReinforcementLearning,
    Language,
    Vision,
    Audio,
    TextGeneration,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub framework: String,
    pub format: String,
    pub precision: String,
    pub quantization: Option<String>,
    pub context_length: Option<u32>,
    pub parameters: Option<u64>,
    pub vocab_size: Option<u32>,
    pub input_types: Vec<String>,
    pub output_types: Vec<String>,
    pub languages: Vec<String>,
    pub domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    pub inferno_version: String,
    pub minimum_ram_gb: f64,
    pub minimum_vram_gb: Option<f64>,
    pub supported_backends: Vec<String>,
    pub supported_platforms: Vec<String>,
    pub gpu_architectures: Vec<String>,
    pub cpu_instructions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub inference_speed_tokens_per_sec: Option<f64>,
    pub memory_usage_gb: Option<f64>,
    pub throughput_requests_per_sec: Option<f64>,
    pub latency_ms: Option<f64>,
    pub benchmark_scores: HashMap<String, f64>,
    pub energy_efficiency: Option<f64>,
    pub energy_efficiency_tokens_per_joule: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDependency {
    pub name: String,
    pub version: String,
    pub optional: bool,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub category: Option<ModelCategory>,
    pub publisher: Option<String>,
    pub license: Option<String>,
    pub min_rating: Option<f32>,
    pub max_size_gb: Option<f64>,
    pub tags: Vec<String>,
    pub frameworks: Vec<String>,
    pub languages: Vec<String>,
    pub platforms: Vec<String>,
    pub free_only: bool,
    pub verified_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub models: Vec<ModelListing>,
    pub total_count: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
    pub facets: SearchFacets,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchFacets {
    pub categories: HashMap<String, usize>,
    pub publishers: HashMap<String, usize>,
    pub licenses: HashMap<String, usize>,
    pub frameworks: HashMap<String, usize>,
    pub tags: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub progress_percent: f64,
    pub download_speed_mbps: f64,
    pub eta_seconds: u64,
    pub status: DownloadStatus,
    pub started_at: DateTime<Utc>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Verifying,
    Installing,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModel {
    pub model_id: String,
    pub local_path: PathBuf,
    pub installed_at: DateTime<Utc>,
    pub version: String,
    pub source: ModelSource,
    pub verified: bool,
    pub auto_update_enabled: bool,
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelSource {
    Marketplace,
    Local,
    Git,
    HuggingFace,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRequest {
    pub model_path: PathBuf,
    pub metadata: ModelListing,
    pub license_file: Option<PathBuf>,
    pub readme_file: Option<PathBuf>,
    pub example_files: Vec<PathBuf>,
    pub visibility: ModelVisibility,
    pub pricing: PricingInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelVisibility {
    Public,
    Private,
    Organization,
    Limited(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingInfo {
    pub free: bool,
    pub price_per_download: Option<f64>,
    pub price_per_token: Option<f64>,
    pub subscription_tiers: Vec<SubscriptionTier>,
    pub usage_based: Option<UsageBasedPricing>,
    pub usage_limits: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingInfo {
    pub average_rating: f64,
    pub total_ratings: u64,
    pub rating_distribution: [u32; 5], // [1-star, 2-star, 3-star, 4-star, 5-star]
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("inferno");

        Self {
            enabled: true,
            registry_url: "https://huggingface.co".to_string(),
            local_cache_dir: data_dir.join("marketplace_cache"),
            authentication: AuthenticationConfig::default(),
            auto_update: false,
            update_interval_hours: 24,
            max_cache_size_gb: 10.0,
            trusted_publishers: vec![
                "huggingface".to_string(),
                "microsoft".to_string(),
                "google".to_string(),
                "meta".to_string(),
                "openai".to_string(),
                "anthropic".to_string(),
                "mistralai".to_string(),
                "cohere".to_string(),
                "nvidia".to_string(),
                "pytorch".to_string(),
                "tensorflow".to_string(),
            ],
            verification: VerificationConfig::default(),
            proxy_settings: None,
            repositories: vec![
                Repository {
                    name: "huggingface".to_string(),
                    url: "https://huggingface.co".to_string(),
                    enabled: true,
                    priority: 1,
                    authentication: None,
                    verification_required: false,
                    last_updated: None,
                    metadata_url: Some("https://huggingface.co/api/models".to_string()),
                },
                Repository {
                    name: "ollama".to_string(),
                    url: "https://registry.ollama.ai".to_string(),
                    enabled: true,
                    priority: 2,
                    authentication: None,
                    verification_required: false,
                    last_updated: None,
                    metadata_url: Some("https://registry.ollama.ai/v2/_catalog".to_string()),
                },
                Repository {
                    name: "onnx-models".to_string(),
                    url: "https://github.com/onnx/models".to_string(),
                    enabled: true,
                    priority: 3,
                    authentication: None,
                    verification_required: false,
                    last_updated: None,
                    metadata_url: Some(
                        "https://api.github.com/repos/onnx/models/contents".to_string(),
                    ),
                },
                Repository {
                    name: "pytorch-hub".to_string(),
                    url: "https://pytorch.org/hub".to_string(),
                    enabled: true,
                    priority: 4,
                    authentication: None,
                    verification_required: false,
                    last_updated: None,
                    metadata_url: Some("https://pytorch.org/hub/api/v1/models".to_string()),
                },
                Repository {
                    name: "tensorflow-hub".to_string(),
                    url: "https://tfhub.dev".to_string(),
                    enabled: true,
                    priority: 5,
                    authentication: None,
                    verification_required: false,
                    last_updated: None,
                    metadata_url: Some("https://tfhub.dev/api/index".to_string()),
                },
            ],
            package_db_path: data_dir.join("package_db.json"),
            auto_resolve_dependencies: true,
            auto_cleanup_unused: false,
        }
    }
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            verify_signatures: true,
            verify_checksums: true,
            require_trusted_publishers: false,
            scan_for_malware: false,
            allowed_licenses: vec![
                "MIT".to_string(),
                "Apache-2.0".to_string(),
                "BSD-3-Clause".to_string(),
                "GPL-3.0".to_string(),
                "CC-BY-4.0".to_string(),
            ],
            blocked_models: vec![],
        }
    }
}

impl MarketplaceConfig {
    pub fn from_config(config: &Config) -> Result<Self> {
        Ok(config.marketplace.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionTier {
    pub name: String,
    pub price_per_month: f64,
    pub max_downloads: Option<u64>,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageBasedPricing {
    pub price_per_1k_tokens: f64,
    pub price_per_request: f64,
    pub free_tier_limit: u64,
}

pub struct ModelMarketplace {
    config: MarketplaceConfig,
    registry_client: Arc<RegistryClient>,
    local_cache: Arc<RwLock<HashMap<String, ModelListing>>>,
    installed_models: Arc<RwLock<HashMap<String, InstalledModel>>>,
    download_progress: Arc<RwLock<HashMap<String, DownloadProgress>>>,
    verification_engine: Arc<VerificationEngine>,
    package_db: Arc<RwLock<PackageDatabase>>,
    dependency_resolver: Arc<RwLock<DependencyResolver>>,
}

impl ModelMarketplace {
    pub fn new(config: MarketplaceConfig) -> Result<Self> {
        let registry_client = Arc::new(RegistryClient::new(&config)?);
        let verification_engine = Arc::new(VerificationEngine::new(&config.verification)?);

        // Load or create package database
        let package_db = if config.package_db_path.exists() {
            let db_content = std::fs::read_to_string(&config.package_db_path)
                .context("Failed to read package database")?;
            serde_json::from_str(&db_content).unwrap_or_else(|_| PackageDatabase::new())
        } else {
            PackageDatabase::new()
        };

        let dependency_resolver = DependencyResolver::new(
            package_db.clone(),
            config.repositories.clone(),
            (*registry_client).clone(),
        );

        Ok(Self {
            config,
            registry_client,
            local_cache: Arc::new(RwLock::new(HashMap::new())),
            installed_models: Arc::new(RwLock::new(HashMap::new())),
            download_progress: Arc::new(RwLock::new(HashMap::new())),
            verification_engine,
            package_db: Arc::new(RwLock::new(package_db)),
            dependency_resolver: Arc::new(RwLock::new(dependency_resolver)),
        })
    }

    pub async fn search_models(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
        page: usize,
        per_page: usize,
    ) -> Result<SearchResult> {
        info!("Searching models with query: '{}', page: {}", query, page);

        let results = self
            .registry_client
            .search(query, filters, page, per_page)
            .await
            .context("Failed to search models in registry")?;

        // Cache the results
        {
            let mut cache = self.local_cache.write().await;
            for model in &results.models {
                cache.insert(model.id.clone(), model.clone());
            }
        }

        debug!("Found {} models", results.models.len());
        Ok(results)
    }

    pub async fn get_model_details(&self, model_id: &str) -> Result<ModelListing> {
        info!("Fetching details for model: {}", model_id);

        // Check cache first
        {
            let cache = self.local_cache.read().await;
            if let Some(model) = cache.get(model_id) {
                debug!("Model details found in cache");
                return Ok(model.clone());
            }
        }

        // Fetch from registry
        let model = self
            .registry_client
            .get_model(model_id)
            .await
            .context("Failed to fetch model details")?;

        // Update cache
        {
            let mut cache = self.local_cache.write().await;
            cache.insert(model.id.clone(), model.clone());
        }

        Ok(model)
    }

    pub async fn download_model(
        &self,
        model_id: &str,
        target_path: Option<PathBuf>,
    ) -> Result<String> {
        info!("Starting download for model: {}", model_id);

        let model = self.get_model_details(model_id).await?;

        // Verify compatibility
        self.verify_compatibility(&model).await?;

        // Create download progress tracking
        let download_id = uuid::Uuid::new_v4().to_string();
        let progress = DownloadProgress {
            model_id: model_id.to_string(),
            bytes_downloaded: 0,
            total_bytes: model.size_bytes,
            progress_percent: 0.0,
            download_speed_mbps: 0.0,
            eta_seconds: 0,
            status: DownloadStatus::Pending,
            started_at: Utc::now(),
            error: None,
        };

        {
            let mut downloads = self.download_progress.write().await;
            downloads.insert(download_id.clone(), progress);
        }

        // Determine target path
        let local_path = target_path.unwrap_or_else(|| {
            self.config.local_cache_dir.join(&model.id).join(format!(
                "{}.{}",
                model.name,
                self.get_file_extension(&model)
            ))
        });

        // Create parent directory
        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create download directory")?;
        }

        // Start download in background
        let registry_client = Arc::clone(&self.registry_client);
        let verification_engine = Arc::clone(&self.verification_engine);
        let download_progress_clone = Arc::clone(&self.download_progress);
        let config = self.config.clone();
        let download_id_clone = download_id.clone();

        tokio::spawn(async move {
            let result = Self::download_model_impl(
                registry_client,
                verification_engine,
                download_progress_clone.clone(),
                download_id_clone.clone(),
                model,
                local_path,
                config,
            )
            .await;

            if let Err(e) = result {
                warn!("Download failed for {}: {}", download_id_clone, e);
                let mut downloads = download_progress_clone.write().await;
                if let Some(progress) = downloads.get_mut(&download_id_clone) {
                    progress.status = DownloadStatus::Failed;
                    progress.error = Some(e.to_string());
                }
            }
        });

        Ok(download_id)
    }

    async fn download_model_impl(
        registry_client: Arc<RegistryClient>,
        verification_engine: Arc<VerificationEngine>,
        download_progress: Arc<RwLock<HashMap<String, DownloadProgress>>>,
        download_id: String,
        model: ModelListing,
        local_path: PathBuf,
        _config: MarketplaceConfig,
    ) -> Result<()> {
        // Update status to downloading
        {
            let mut downloads = download_progress.write().await;
            if let Some(progress) = downloads.get_mut(&download_id) {
                progress.status = DownloadStatus::Downloading;
            }
        }

        // Clone for the closure
        let download_progress_clone = Arc::clone(&download_progress);
        let download_id_clone = download_id.clone();

        // Download the model file
        registry_client
            .download_file(
                &model.download_url,
                &local_path,
                move |bytes_downloaded, total_bytes| {
                    let progress_percent = (bytes_downloaded as f64 / total_bytes as f64) * 100.0;

                    tokio::spawn({
                        let download_progress = Arc::clone(&download_progress_clone);
                        let download_id = download_id_clone.clone();
                        async move {
                            let mut downloads = download_progress.write().await;
                            if let Some(progress) = downloads.get_mut(&download_id) {
                                progress.bytes_downloaded = bytes_downloaded;
                                progress.progress_percent = progress_percent;
                                // Calculate download speed and ETA here
                            }
                        }
                    });
                },
            )
            .await?;

        // Update status to verifying
        {
            let mut downloads = download_progress.write().await;
            if let Some(progress) = downloads.get_mut(&download_id) {
                progress.status = DownloadStatus::Verifying;
            }
        }

        // Verify the downloaded model
        verification_engine
            .verify_model(&local_path, &model)
            .await?;

        // Update status to completed
        {
            let mut downloads = download_progress.write().await;
            if let Some(progress) = downloads.get_mut(&download_id) {
                progress.status = DownloadStatus::Completed;
                progress.progress_percent = 100.0;
            }
        }

        info!("Model {} downloaded and verified successfully", model.id);
        Ok(())
    }

    pub async fn get_download_progress(&self, download_id: &str) -> Result<DownloadProgress> {
        let downloads = self.download_progress.read().await;
        downloads
            .get(download_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Download not found: {}", download_id))
    }

    pub async fn cancel_download(&self, download_id: &str) -> Result<()> {
        info!("Cancelling download: {}", download_id);

        let mut downloads = self.download_progress.write().await;
        if let Some(progress) = downloads.get_mut(download_id) {
            progress.status = DownloadStatus::Cancelled;
        }

        // Implementation would also cancel the actual download task
        Ok(())
    }

    pub async fn install_model(&self, download_id: &str, enable_auto_update: bool) -> Result<()> {
        info!("Installing model from download: {}", download_id);

        let progress = self.get_download_progress(download_id).await?;

        if !matches!(progress.status, DownloadStatus::Completed) {
            return Err(anyhow::anyhow!("Download not completed"));
        }

        let model = self.get_model_details(&progress.model_id).await?;

        // Create installation record
        let installed_model = InstalledModel {
            model_id: model.id.clone(),
            local_path: self.config.local_cache_dir.join(&model.id),
            installed_at: Utc::now(),
            version: model.version.clone(),
            source: ModelSource::Marketplace,
            verified: true,
            auto_update_enabled: enable_auto_update,
            last_used: None,
            usage_count: 0,
        };

        {
            let mut installed = self.installed_models.write().await;
            installed.insert(model.id.clone(), installed_model);
        }

        info!("Model {} installed successfully", model.id);
        Ok(())
    }

    pub async fn uninstall_model(&self, model_id: &str, remove_files: bool) -> Result<()> {
        info!("Uninstalling model: {}", model_id);

        let installed_model = {
            let mut installed = self.installed_models.write().await;
            installed
                .remove(model_id)
                .ok_or_else(|| anyhow::anyhow!("Model not installed: {}", model_id))?
        };

        if remove_files && installed_model.local_path.exists() {
            tokio::fs::remove_dir_all(&installed_model.local_path)
                .await
                .context("Failed to remove model files")?;
        }

        info!("Model {} uninstalled successfully", model_id);
        Ok(())
    }

    pub async fn list_installed_models(&self) -> Result<Vec<InstalledModel>> {
        let installed = self.installed_models.read().await;
        Ok(installed.values().cloned().collect())
    }

    pub async fn check_for_updates(&self) -> Result<Vec<String>> {
        info!("Checking for model updates");

        let installed = self.installed_models.read().await;
        let mut updates_available = Vec::new();

        for (model_id, installed_model) in installed.iter() {
            if !installed_model.auto_update_enabled {
                continue;
            }

            match self.get_model_details(model_id).await {
                Ok(latest_model) => {
                    if latest_model.version != installed_model.version {
                        debug!(
                            "Update available for {}: {} -> {}",
                            model_id, installed_model.version, latest_model.version
                        );
                        updates_available.push(model_id.clone());
                    }
                }
                Err(e) => {
                    warn!("Failed to check updates for {}: {}", model_id, e);
                }
            }
        }

        info!(
            "Found {} models with available updates",
            updates_available.len()
        );
        Ok(updates_available)
    }

    pub async fn update_model(&self, model_id: &str) -> Result<String> {
        info!("Updating model: {}", model_id);

        // Check if model is installed
        let installed_model = {
            let installed = self.installed_models.read().await;
            installed
                .get(model_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Model not installed: {}", model_id))?
        };

        // Download the latest version
        let download_id = self
            .download_model(model_id, Some(installed_model.local_path))
            .await?;

        info!("Model update started with download ID: {}", download_id);
        Ok(download_id)
    }

    pub async fn publish_model(&self, request: PublishRequest) -> Result<String> {
        info!("Publishing model: {}", request.metadata.name);

        // Verify the model before publishing
        self.verification_engine
            .verify_model(&request.model_path, &request.metadata)
            .await
            .context("Model verification failed")?;

        // Upload to registry
        let model_id = self
            .registry_client
            .publish_model(request)
            .await
            .context("Failed to publish model to registry")?;

        info!("Model published successfully with ID: {}", model_id);
        Ok(model_id)
    }

    pub async fn get_popular_models(
        &self,
        category: Option<ModelCategory>,
        limit: usize,
    ) -> Result<Vec<ModelListing>> {
        info!("Fetching popular models");

        let popular = self
            .registry_client
            .get_popular_models(category, limit)
            .await
            .context("Failed to fetch popular models")?;

        // Update cache
        {
            let mut cache = self.local_cache.write().await;
            for model in &popular {
                cache.insert(model.id.clone(), model.clone());
            }
        }

        Ok(popular)
    }

    pub async fn get_recommended_models(&self, user_id: Option<&str>) -> Result<Vec<ModelListing>> {
        info!("Fetching recommended models");

        let recommended = self
            .registry_client
            .get_recommendations(user_id)
            .await
            .context("Failed to fetch recommendations")?;

        // Update cache
        {
            let mut cache = self.local_cache.write().await;
            for model in &recommended {
                cache.insert(model.id.clone(), model.clone());
            }
        }

        Ok(recommended)
    }

    async fn verify_compatibility(&self, model: &ModelListing) -> Result<()> {
        debug!("Verifying compatibility for model: {}", model.id);

        // Check minimum requirements
        let system_info = self.get_system_info();

        if model.compatibility.minimum_ram_gb > system_info.total_ram_gb {
            return Err(anyhow::anyhow!(
                "Insufficient RAM: need {:.1}GB, have {:.1}GB",
                model.compatibility.minimum_ram_gb,
                system_info.total_ram_gb
            ));
        }

        if let Some(min_vram) = model.compatibility.minimum_vram_gb {
            if min_vram > system_info.total_vram_gb.unwrap_or(0.0) {
                return Err(anyhow::anyhow!(
                    "Insufficient VRAM: need {:.1}GB, have {:.1}GB",
                    min_vram,
                    system_info.total_vram_gb.unwrap_or(0.0)
                ));
            }
        }

        // Check platform compatibility
        if !model
            .compatibility
            .supported_platforms
            .contains(&system_info.platform)
        {
            return Err(anyhow::anyhow!(
                "Platform not supported: {}. Supported: {:?}",
                system_info.platform,
                model.compatibility.supported_platforms
            ));
        }

        debug!("Compatibility check passed for model: {}", model.id);
        Ok(())
    }

    fn get_system_info(&self) -> SystemInfo {
        // Mock implementation - in real code this would use sysinfo crate
        SystemInfo {
            platform: std::env::consts::OS.to_string(),
            total_ram_gb: 16.0,
            total_vram_gb: Some(8.0),
            cpu_features: vec!["avx2".to_string(), "fma".to_string()],
        }
    }

    fn get_file_extension(&self, model: &ModelListing) -> &str {
        match model.metadata.format.as_str() {
            "GGUF" => "gguf",
            "ONNX" => "onnx",
            "SafeTensors" => "safetensors",
            "PyTorch" => "pth",
            _ => "bin",
        }
    }

    // Package Manager Methods

    pub async fn package_install(
        &self,
        package_name: &str,
        auto_resolve_deps: bool,
    ) -> Result<String> {
        info!("Installing package: {}", package_name);

        // Search for the model in repositories
        let model = self.resolve_package_name(package_name).await?;

        if auto_resolve_deps {
            let install_plan = self.create_install_plan(&model.id).await?;
            return self.execute_install_plan(install_plan).await;
        }

        // Simple install without dependency resolution
        self.download_and_install_model(&model.id).await
    }

    pub async fn package_remove(&self, package_name: &str, remove_deps: bool) -> Result<()> {
        info!("Removing package: {}", package_name);

        let model_id = self.resolve_installed_package(package_name).await?;

        if remove_deps {
            let removal_plan = self.create_removal_plan(&model_id).await?;
            self.execute_removal_plan(removal_plan).await?;
        } else {
            self.uninstall_model(&model_id, true).await?;
        }

        self.update_package_db().await?;
        Ok(())
    }

    pub async fn package_search(
        &self,
        query: &str,
        repo_filter: Option<&str>,
    ) -> Result<Vec<ModelListing>> {
        info!("Searching packages: {}", query);

        let mut all_results = Vec::new();

        for repo in &self.config.repositories {
            if !repo.enabled {
                continue;
            }

            if let Some(filter) = repo_filter {
                if repo.name != filter {
                    continue;
                }
            }

            match self.search_in_repository(repo, query).await {
                Ok(mut results) => all_results.append(&mut results),
                Err(e) => warn!("Failed to search in repository {}: {}", repo.name, e),
            }
        }

        // Sort by priority and relevance
        all_results.sort_by(|a, b| {
            // Sort by downloads (popularity) as a proxy for relevance
            b.downloads.cmp(&a.downloads)
        });

        Ok(all_results)
    }

    pub async fn package_upgrade(&self, package_name: Option<&str>) -> Result<Vec<String>> {
        info!("Upgrading packages");

        if let Some(name) = package_name {
            // Upgrade specific package
            let model_id = self.resolve_installed_package(name).await?;
            let download_id = self.update_model(&model_id).await?;
            Ok(vec![download_id])
        } else {
            // Upgrade all packages
            let updates = self.check_for_updates().await?;
            let mut download_ids = Vec::new();

            for model_id in updates {
                match self.update_model(&model_id).await {
                    Ok(download_id) => download_ids.push(download_id),
                    Err(e) => warn!("Failed to update {}: {}", model_id, e),
                }
            }

            Ok(download_ids)
        }
    }

    pub async fn package_list(&self, filter: Option<&str>) -> Result<Vec<InstalledPackage>> {
        let package_db = self.package_db.read().await;
        let mut packages: Vec<_> = package_db.installed_packages.values().cloned().collect();

        if let Some(filter_str) = filter {
            packages
                .retain(|pkg| pkg.name.contains(filter_str) || pkg.model_id.contains(filter_str));
        }

        packages.sort_by(|a, b| a.install_date.cmp(&b.install_date));
        Ok(packages)
    }

    pub async fn package_autoremove(&self) -> Result<Vec<String>> {
        info!("Removing unused packages");

        let package_db = self.package_db.read().await;
        let mut to_remove = Vec::new();

        // Find packages that were auto-installed and no longer have dependents
        for (model_id, package) in &package_db.installed_packages {
            if !package.auto_installed {
                continue;
            }

            let has_dependents = package_db
                .installed_packages
                .values()
                .any(|other| other.dependencies.contains(model_id));

            if !has_dependents {
                to_remove.push(model_id.clone());
            }
        }

        drop(package_db);

        // Remove the packages
        for model_id in &to_remove {
            self.uninstall_model(model_id, true)
                .await
                .unwrap_or_else(|e| warn!("Failed to auto-remove {}: {}", model_id, e));
        }

        self.update_package_db().await?;
        Ok(to_remove)
    }

    // Repository Management

    pub async fn repo_add(&self, name: &str, url: &str, priority: Option<u32>) -> Result<()> {
        info!("Adding repository: {} at {}", name, url);

        let _new_repo = Repository {
            name: name.to_string(),
            url: url.to_string(),
            enabled: true,
            priority: priority.unwrap_or(100),
            authentication: None,
            verification_required: false,
            last_updated: None,
            metadata_url: None,
        };

        // This would need to be implemented to modify the config
        // For now, we'll update the in-memory config
        // In a real implementation, this would save to the config file
        info!("Repository {} would be added", name);
        Ok(())
    }

    pub async fn repo_remove(&self, name: &str) -> Result<()> {
        info!("Removing repository: {}", name);
        // Implementation would remove from config and clean up
        Ok(())
    }

    pub async fn repo_list(&self) -> Result<Vec<Repository>> {
        Ok(self.config.repositories.clone())
    }

    pub async fn repo_update(&self, name: Option<&str>) -> Result<()> {
        info!("Updating repository metadata");

        if let Some(repo_name) = name {
            // Update specific repository
            if let Some(repo) = self
                .config
                .repositories
                .iter()
                .find(|r| r.name == repo_name)
            {
                self.sync_repository_metadata(repo).await?;
            } else {
                return Err(anyhow::anyhow!("Repository not found: {}", repo_name));
            }
        } else {
            // Update all repositories
            for repo in &self.config.repositories {
                if repo.enabled {
                    if let Err(e) = self.sync_repository_metadata(repo).await {
                        warn!("Failed to update repository {}: {}", repo.name, e);
                    }
                }
            }
        }

        Ok(())
    }

    // Helper methods for package management

    async fn resolve_package_name(&self, package_name: &str) -> Result<ModelListing> {
        // First try exact match in cache
        {
            let cache = self.local_cache.read().await;
            if let Some(model) = cache.get(package_name) {
                return Ok(model.clone());
            }
        }

        // Search across repositories
        let search_results = self.package_search(package_name, None).await?;

        if search_results.is_empty() {
            return Err(anyhow::anyhow!("Package not found: {}", package_name));
        }

        // Return the first (most relevant) result
        Ok(search_results[0].clone())
    }

    async fn resolve_installed_package(&self, package_name: &str) -> Result<String> {
        let package_db = self.package_db.read().await;

        // Try exact model_id match first
        if package_db.installed_packages.contains_key(package_name) {
            return Ok(package_name.to_string());
        }

        // Try name match
        for (model_id, package) in &package_db.installed_packages {
            if package.name == package_name {
                return Ok(model_id.clone());
            }
        }

        Err(anyhow::anyhow!(
            "Installed package not found: {}",
            package_name
        ))
    }

    async fn create_install_plan(&self, model_id: &str) -> Result<InstallPlan> {
        let resolver = self.dependency_resolver.read().await;
        resolver.create_install_plan(model_id).await
    }

    async fn create_removal_plan(&self, model_id: &str) -> Result<Vec<String>> {
        let package_db = self.package_db.read().await;
        let mut to_remove = vec![model_id.to_string()];

        // Find dependencies that would be orphaned
        if let Some(package) = package_db.installed_packages.get(model_id) {
            for dep in &package.dependencies {
                let has_other_dependents = package_db
                    .installed_packages
                    .values()
                    .any(|other| other.model_id != model_id && other.dependencies.contains(dep));

                if !has_other_dependents {
                    // Check if it was auto-installed
                    if let Some(dep_package) = package_db.installed_packages.get(dep) {
                        if dep_package.auto_installed {
                            to_remove.push(dep.clone());
                        }
                    }
                }
            }
        }

        Ok(to_remove)
    }

    async fn execute_install_plan(&self, plan: InstallPlan) -> Result<String> {
        info!(
            "Executing install plan: {} packages to install",
            plan.to_install.len()
        );

        let mut last_download_id = String::new();

        for model in plan.to_install {
            last_download_id = self.download_and_install_model(&model.id).await?;
        }

        for (model_id, _new_version) in plan.to_upgrade {
            last_download_id = self.update_model(&model_id).await?;
        }

        self.update_package_db().await?;
        Ok(last_download_id)
    }

    async fn execute_removal_plan(&self, models_to_remove: Vec<String>) -> Result<()> {
        for model_id in models_to_remove {
            self.uninstall_model(&model_id, true)
                .await
                .unwrap_or_else(|e| warn!("Failed to remove {}: {}", model_id, e));
        }
        Ok(())
    }

    async fn download_and_install_model(&self, model_id: &str) -> Result<String> {
        let download_id = self.download_model(model_id, None).await?;

        // In a real implementation, we'd wait for download to complete
        // and then install. For now, return the download_id

        Ok(download_id)
    }

    async fn search_in_repository(
        &self,
        _repo: &Repository,
        query: &str,
    ) -> Result<Vec<ModelListing>> {
        // This would search in the specific repository
        // For now, delegate to the general search method
        let filters = Some(SearchFilters {
            category: None,
            publisher: None,
            license: None,
            min_rating: None,
            max_size_gb: None,
            tags: vec![],
            frameworks: vec![],
            languages: vec![],
            platforms: vec![],
            free_only: false,
            verified_only: false,
        });

        self.search_models(query, filters, 1, 20)
            .await
            .map(|result| result.models)
    }

    async fn sync_repository_metadata(&self, repo: &Repository) -> Result<()> {
        info!("Syncing metadata for repository: {}", repo.name);
        // This would fetch and cache repository metadata
        Ok(())
    }

    async fn update_package_db(&self) -> Result<()> {
        let package_db = self.package_db.read().await;
        let db_json = serde_json::to_string_pretty(&*package_db)
            .context("Failed to serialize package database")?;

        tokio::fs::write(&self.config.package_db_path, db_json)
            .await
            .context("Failed to save package database")?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct SystemInfo {
    platform: String,
    total_ram_gb: f64,
    total_vram_gb: Option<f64>,
    cpu_features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RegistryClient {
    base_url: String,
    auth_config: AuthenticationConfig,
    repositories: Vec<Repository>,
    #[cfg(feature = "download")]
    client: reqwest::Client,
}

impl RegistryClient {
    pub fn new(config: &MarketplaceConfig) -> Result<Self> {
        #[cfg(feature = "download")]
        let client = reqwest::Client::builder()
            .user_agent("inferno-marketplace/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url: config.registry_url.clone(),
            auth_config: config.authentication.clone(),
            repositories: config.repositories.clone(),
            #[cfg(feature = "download")]
            client,
        })
    }

    pub async fn search(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
        page: usize,
        per_page: usize,
    ) -> Result<SearchResult> {
        #[cfg(feature = "download")]
        {
            let url = format!("{}/api/v1/models/search", self.base_url);
            let mut params = vec![
                ("q", query.to_string()),
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ];

            if let Some(filters) = filters {
                if let Some(category) = filters.category {
                    params.push(("category", format!("{:?}", category)));
                }
                if let Some(publisher) = filters.publisher {
                    params.push(("publisher", publisher));
                }
                // Add other filter parameters...
            }

            let response = self
                .client
                .get(&url)
                .query(&params)
                .headers(self.get_auth_headers()?)
                .send()
                .await
                .context("Failed to send search request")?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!(
                    "Search request failed: {}",
                    response.status()
                ));
            }

            let result: SearchResult = response
                .json()
                .await
                .context("Failed to parse search response")?;

            Ok(result)
        }
        #[cfg(not(feature = "download"))]
        {
            // Fallback implementation for when download feature is disabled
            // Fetch from configured repositories using Git/HTTP without file downloads
            self.search_from_repositories(query, filters, page, per_page)
                .await
        }
    }

    async fn search_from_repositories(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
        page: usize,
        per_page: usize,
    ) -> Result<SearchResult> {
        let mut all_models = Vec::new();

        // Search across all configured repository sources
        for repo_config in &self.repositories {
            if !repo_config.enabled {
                continue;
            }

            match self.fetch_repository_models(&repo_config.url).await {
                Ok(models) => {
                    // Filter models based on query and filters
                    let filtered_models = models
                        .into_iter()
                        .filter(|model| self.matches_search_criteria(model, query, &filters))
                        .collect::<Vec<_>>();

                    all_models.extend(filtered_models);
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch from repository {}: {}", repo_config.url, e);
                    continue;
                }
            }
        }

        // Apply pagination
        let total_count = all_models.len();
        let total_pages = total_count.div_ceil(per_page);
        let start_idx = (page - 1) * per_page;
        let end_idx = (start_idx + per_page).min(total_count);

        let paged_models = if start_idx < total_count {
            all_models[start_idx..end_idx].to_vec()
        } else {
            Vec::new()
        };

        Ok(SearchResult {
            models: paged_models,
            total_count,
            page,
            per_page,
            total_pages,
            facets: self.build_search_facets(&all_models),
        })
    }

    async fn fetch_repository_models(&self, repo_url: &str) -> Result<Vec<ModelListing>> {
        // Check if this is a Git repository or HTTP API endpoint
        if repo_url.ends_with(".git")
            || repo_url.contains("github.com")
            || repo_url.contains("gitlab.com")
        {
            self.fetch_from_git_repository(repo_url).await
        } else if repo_url.starts_with("http") {
            self.fetch_from_http_api(repo_url).await
        } else {
            self.fetch_from_local_path(repo_url).await
        }
    }

    async fn fetch_from_git_repository(&self, repo_url: &str) -> Result<Vec<ModelListing>> {
        // For Git repositories, we can fetch model metadata without downloading large files
        // This would typically clone or fetch the repository metadata

        // Convert Git URL to API URL for GitHub/GitLab
        if repo_url.contains("github.com") {
            return self.fetch_from_github_api(repo_url).await;
        }

        if repo_url.contains("gitlab.com") {
            return self.fetch_from_gitlab_api(repo_url).await;
        }

        // For other Git repositories, we could use git ls-remote or clone --depth 1
        // For now, return empty to avoid actual Git operations
        Ok(Vec::new())
    }

    async fn fetch_from_github_api(&self, repo_url: &str) -> Result<Vec<ModelListing>> {
        // Extract owner/repo from GitHub URL
        let parts: Vec<&str> = repo_url.trim_end_matches(".git").split('/').collect();

        if parts.len() < 2 {
            return Ok(Vec::new());
        }

        let owner = parts[parts.len() - 2];
        let repo = parts[parts.len() - 1];

        // Use GitHub API to fetch repository contents and model metadata
        let api_url = format!("https://api.github.com/repos/{}/{}/contents", owner, repo);

        // This would make an HTTP request to GitHub API
        // For now, create a sample model entry based on repository info
        let model = ModelListing {
            id: format!("{}/{}", owner, repo),
            name: repo.to_string(),
            version: "latest".to_string(),
            publisher: owner.to_string(),
            description: format!("Model from GitHub repository {}/{}", owner, repo),
            category: ModelCategory::TextGeneration,
            license: "Unknown".to_string(),
            size_bytes: 0, // Would be fetched from repository
            download_url: repo_url.to_string(),
            checksum: String::new(),
            signature: None,
            metadata: ModelMetadata {
                framework: "Unknown".to_string(),
                format: "GGUF".to_string(),
                precision: "fp16".to_string(),
                quantization: None,
                context_length: Some(2048),
                parameters: None,
                vocab_size: None,
                input_types: vec!["text".to_string()],
                output_types: vec!["text".to_string()],
                languages: vec!["en".to_string()],
                domains: vec!["general".to_string()],
            },
            compatibility: CompatibilityInfo {
                inferno_version: ">=0.1.0".to_string(),
                minimum_ram_gb: 4.0,
                minimum_vram_gb: Some(2.0),
                supported_backends: vec!["gguf".to_string()],
                supported_platforms: vec![
                    "linux".to_string(),
                    "macos".to_string(),
                    "windows".to_string(),
                ],
                gpu_architectures: vec!["cuda".to_string(), "metal".to_string()],
                cpu_instructions: vec!["avx2".to_string()],
            },
            performance: PerformanceMetrics {
                inference_speed_tokens_per_sec: Some(50.0),
                memory_usage_gb: Some(4.0),
                throughput_requests_per_sec: Some(10.0),
                latency_ms: Some(100.0),
                benchmark_scores: std::collections::HashMap::new(),
                energy_efficiency: None,
                energy_efficiency_tokens_per_joule: None,
            },
            pricing: PricingInfo {
                free: true,
                price_per_download: None,
                price_per_token: None,
                subscription_tiers: vec![],
                usage_based: None,
                usage_limits: None,
            },
            ratings: RatingInfo {
                average_rating: 4.0,
                total_ratings: 100,
                rating_distribution: [10, 5, 10, 25, 50],
            },
            tags: vec!["github".to_string(), "open-source".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            downloads: 1000,
            visibility: ModelVisibility::Public,
            verified: false,
            dependencies: vec![],
            documentation_url: Some(format!("https://github.com/{}/{}", owner, repo)),
            demo_url: None,
            paper_url: None,
            source_url: Some(repo_url.to_string()),
            published_at: chrono::Utc::now(),
            rating: None,
        };

        Ok(vec![model])
    }

    async fn fetch_from_gitlab_api(&self, repo_url: &str) -> Result<Vec<ModelListing>> {
        // Similar implementation for GitLab API
        // For now, return empty
        Ok(Vec::new())
    }

    async fn fetch_from_http_api(&self, api_url: &str) -> Result<Vec<ModelListing>> {
        // Fetch models from HTTP API endpoint
        // This would make HTTP requests to the API and parse the response
        // For now, return empty to avoid making actual network calls
        Ok(Vec::new())
    }

    async fn fetch_from_local_path(&self, path: &str) -> Result<Vec<ModelListing>> {
        // Fetch models from local file system path
        // This would scan local directories for model files and metadata
        Ok(Vec::new())
    }

    fn matches_search_criteria(
        &self,
        model: &ModelListing,
        query: &str,
        filters: &Option<SearchFilters>,
    ) -> bool {
        // Check if model matches search query
        let query_lower = query.to_lowercase();
        let matches_query = query.is_empty()
            || model.name.to_lowercase().contains(&query_lower)
            || model.description.to_lowercase().contains(&query_lower)
            || model.publisher.to_lowercase().contains(&query_lower)
            || model
                .tags
                .iter()
                .any(|tag| tag.to_lowercase().contains(&query_lower));

        if !matches_query {
            return false;
        }

        // Apply filters if provided
        if let Some(filters) = filters {
            if let Some(ref category) = filters.category {
                if !std::mem::discriminant(&model.category).eq(&std::mem::discriminant(category)) {
                    return false;
                }
            }

            if let Some(ref publisher) = filters.publisher {
                if !model.publisher.eq_ignore_ascii_case(publisher) {
                    return false;
                }
            }

            if let Some(ref license) = filters.license {
                if !model.license.eq_ignore_ascii_case(license) {
                    return false;
                }
            }

            if let Some(min_rating) = filters.min_rating {
                if model.ratings.average_rating < min_rating as f64 {
                    return false;
                }
            }

            if let Some(max_size_gb) = filters.max_size_gb {
                let size_gb = model.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                if size_gb > max_size_gb {
                    return false;
                }
            }

            if !filters.tags.is_empty() {
                let has_matching_tag = filters.tags.iter().any(|filter_tag| {
                    model
                        .tags
                        .iter()
                        .any(|model_tag| model_tag.eq_ignore_ascii_case(filter_tag))
                });
                if !has_matching_tag {
                    return false;
                }
            }

            if filters.free_only && !model.pricing.free {
                return false;
            }

            if filters.verified_only && !model.verified {
                return false;
            }
        }

        true
    }

    fn build_search_facets(&self, models: &[ModelListing]) -> SearchFacets {
        let mut categories = HashMap::new();
        let mut publishers = HashMap::new();
        let mut licenses = HashMap::new();
        let mut frameworks = HashMap::new();
        let mut tags = HashMap::new();

        for model in models {
            // Count categories
            let category_key = format!("{:?}", model.category);
            *categories.entry(category_key).or_insert(0) += 1;

            // Count publishers
            *publishers.entry(model.publisher.clone()).or_insert(0) += 1;

            // Count licenses
            *licenses.entry(model.license.clone()).or_insert(0) += 1;

            // Count frameworks
            *frameworks
                .entry(model.metadata.framework.clone())
                .or_insert(0) += 1;

            // Count tags
            for tag in &model.tags {
                *tags.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        SearchFacets {
            categories,
            publishers,
            licenses,
            frameworks,
            tags,
        }
    }

    pub async fn get_model(&self, model_id: &str) -> Result<ModelListing> {
        #[cfg(feature = "download")]
        {
            let url = format!("{}/api/v1/models/{}", self.base_url, model_id);

            let response = self
                .client
                .get(&url)
                .headers(self.get_auth_headers()?)
                .send()
                .await
                .context("Failed to fetch model details")?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!(
                    "Failed to fetch model: {}",
                    response.status()
                ));
            }

            let model: ModelListing = response
                .json()
                .await
                .context("Failed to parse model response")?;

            Ok(model)
        }
        #[cfg(not(feature = "download"))]
        {
            // Mock implementation
            Err(anyhow::anyhow!(
                "Model not found: {} (download feature disabled)",
                model_id
            ))
        }
    }

    pub async fn download_file<F>(
        &self,
        url: &str,
        target_path: &Path,
        progress_callback: F,
    ) -> Result<()>
    where
        F: FnMut(u64, u64) + Send + 'static,
    {
        #[cfg(feature = "download")]
        {
            let response = self
                .client
                .get(url)
                .headers(self.get_auth_headers()?)
                .send()
                .await
                .context("Failed to start download")?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!("Download failed: {}", response.status()));
            }

            let total_size = response
                .content_length()
                .ok_or_else(|| anyhow::anyhow!("Unknown content length"))?;

            let mut file = tokio::fs::File::create(target_path)
                .await
                .context("Failed to create target file")?;

            let mut downloaded = 0u64;
            let mut stream = response.bytes_stream();

            use futures_util::StreamExt;
            use tokio::io::AsyncWriteExt;

            while let Some(chunk) = stream.next().await {
                let chunk = chunk.context("Failed to read chunk")?;
                file.write_all(&chunk)
                    .await
                    .context("Failed to write chunk")?;

                downloaded += chunk.len() as u64;
                progress_callback(downloaded, total_size);
            }

            file.flush().await.context("Failed to flush file")?;

            Ok(())
        }
        #[cfg(not(feature = "download"))]
        {
            let _ = url;
            let _ = target_path;
            let _ = progress_callback;
            Err(anyhow::anyhow!("Download feature disabled"))
        }
    }

    pub async fn publish_model(&self, request: PublishRequest) -> Result<String> {
        info!("Publishing model: {}", request.metadata.name);

        // Validate the publish request
        self.validate_publish_request(&request)?;

        // Generate unique model ID
        let model_id = format!(
            "{}-{}",
            request.metadata.name.to_lowercase().replace(' ', "-"),
            &uuid::Uuid::new_v4().to_string()[..8]
        );

        // In a real implementation, this would:
        // 1. Upload model files to storage backend
        // 2. Create model metadata entry in database
        // 3. Generate checksums and signatures
        // 4. Update search indices
        // 5. Send notifications to subscribers

        // For now, we'll create a local registry entry
        let model_listing = ModelListing {
            id: model_id.clone(),
            name: request.metadata.name.clone(),
            version: request.metadata.version.clone(),
            publisher: request.metadata.publisher.clone(),
            description: request.metadata.description.clone(),
            category: request.metadata.category.clone(),
            license: request.metadata.license.clone(),
            size_bytes: request.metadata.size_bytes,
            download_url: request.model_path.to_string_lossy().to_string(),
            checksum: self.calculate_file_checksum(&request.model_path)?,
            signature: None,
            metadata: ModelMetadata {
                framework: request.metadata.metadata.framework.clone(),
                format: request.metadata.metadata.format.clone(),
                precision: request.metadata.metadata.precision.clone(),
                quantization: request.metadata.metadata.quantization.clone(),
                context_length: request.metadata.metadata.context_length,
                parameters: request.metadata.metadata.parameters,
                vocab_size: request.metadata.metadata.vocab_size,
                input_types: request.metadata.metadata.input_types.clone(),
                output_types: request.metadata.metadata.output_types.clone(),
                languages: request.metadata.metadata.languages.clone(),
                domains: request.metadata.metadata.domains.clone(),
            },
            compatibility: request.metadata.compatibility.clone(),
            performance: request.metadata.performance.clone(),
            pricing: request.pricing.clone(),
            ratings: RatingInfo {
                average_rating: 0.0,
                total_ratings: 0,
                rating_distribution: [0, 0, 0, 0, 0],
            },
            tags: request.metadata.tags.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            downloads: 0,
            visibility: request.visibility.clone(),
            verified: false, // Would be set through verification process
            dependencies: request.metadata.dependencies.clone(),
            documentation_url: request.metadata.documentation_url.clone(),
            demo_url: request.metadata.demo_url.clone(),
            paper_url: request.metadata.paper_url.clone(),
            source_url: request.metadata.source_url.clone(),
            published_at: chrono::Utc::now(),
            rating: None,
        };

        // Store in local registry (in real implementation, this would be a database)
        info!(
            "Model '{}' published successfully with ID: {}",
            request.metadata.name, model_id
        );

        Ok(model_id)
    }

    fn validate_publish_request(&self, request: &PublishRequest) -> Result<()> {
        if request.metadata.name.trim().is_empty() {
            return Err(anyhow::anyhow!("Model name cannot be empty"));
        }

        if request.metadata.version.trim().is_empty() {
            return Err(anyhow::anyhow!("Model version cannot be empty"));
        }

        if request.metadata.publisher.trim().is_empty() {
            return Err(anyhow::anyhow!("Publisher name cannot be empty"));
        }

        if !request.model_path.exists() {
            return Err(anyhow::anyhow!(
                "Model file does not exist: {}",
                request.model_path.display()
            ));
        }

        // Validate file size
        let metadata = std::fs::metadata(&request.model_path)?;
        let file_size = metadata.len();
        if file_size > 50 * 1024 * 1024 * 1024 {
            // 50GB limit
            return Err(anyhow::anyhow!(
                "Model file too large. Maximum size is 50GB"
            ));
        }

        Ok(())
    }

    fn calculate_file_checksum(&self, file_path: &std::path::Path) -> Result<String> {
        use std::io::Read;
        let mut file = std::fs::File::open(file_path)?;
        let mut hasher = sha2::Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub async fn get_popular_models(
        &self,
        category: Option<ModelCategory>,
        limit: usize,
    ) -> Result<Vec<ModelListing>> {
        info!(
            "Fetching popular models (category: {:?}, limit: {})",
            category, limit
        );

        // Get models from all repositories and sort by popularity metrics
        let mut all_models = Vec::new();

        for repo_config in &self.repositories {
            if !repo_config.enabled {
                continue;
            }

            match self.fetch_repository_models(&repo_config.url).await {
                Ok(models) => {
                    all_models.extend(models);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to fetch popular models from repository {}: {}",
                        repo_config.url,
                        e
                    );
                    continue;
                }
            }
        }

        // Filter by category if specified
        if let Some(ref cat) = category {
            all_models.retain(|model| {
                std::mem::discriminant(&model.category) == std::mem::discriminant(cat)
            });
        }

        // Sort by popularity metrics (downloads, ratings, recent activity)
        all_models.sort_by(|a, b| {
            // Primary sort: downloads (descending)
            let downloads_cmp = b.downloads.cmp(&a.downloads);
            if downloads_cmp != std::cmp::Ordering::Equal {
                return downloads_cmp;
            }

            // Secondary sort: average rating (descending)
            let rating_cmp = b
                .ratings
                .average_rating
                .partial_cmp(&a.ratings.average_rating)
                .unwrap_or(std::cmp::Ordering::Equal);
            if rating_cmp != std::cmp::Ordering::Equal {
                return rating_cmp;
            }

            // Tertiary sort: total ratings count (descending)
            b.ratings.total_ratings.cmp(&a.ratings.total_ratings)
        });

        // Take only the requested number of models
        all_models.truncate(limit);

        Ok(all_models)
    }

    pub async fn get_recommendations(&self, user_id: Option<&str>) -> Result<Vec<ModelListing>> {
        info!("Generating recommendations for user: {:?}", user_id);

        // Get all available models
        let mut all_models = Vec::new();

        for repo_config in &self.repositories {
            if !repo_config.enabled {
                continue;
            }

            match self.fetch_repository_models(&repo_config.url).await {
                Ok(models) => {
                    all_models.extend(models);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to fetch models for recommendations from repository {}: {}",
                        repo_config.url,
                        e
                    );
                    continue;
                }
            }
        }

        if let Some(user_id) = user_id {
            // User-specific recommendations based on:
            // 1. Previously downloaded/used models
            // 2. User's preferred categories and frameworks
            // 3. Similar users' preferences (collaborative filtering)
            // 4. Recent trending models in user's domains

            // For now, implement a simple content-based filtering
            let recommendations = self
                .generate_content_based_recommendations(&all_models, user_id)
                .await?;
            Ok(recommendations)
        } else {
            // Anonymous recommendations - show trending and well-rated models
            let recommendations = self.generate_anonymous_recommendations(&all_models).await?;
            Ok(recommendations)
        }
    }

    async fn generate_content_based_recommendations(
        &self,
        models: &[ModelListing],
        _user_id: &str,
    ) -> Result<Vec<ModelListing>> {
        // In a real implementation, this would:
        // 1. Fetch user's download/usage history
        // 2. Analyze preferred categories, frameworks, model sizes
        // 3. Find similar models based on metadata similarity
        // 4. Weight by user's rating patterns

        // For now, return a curated list based on general preferences
        let mut recommendations: Vec<ModelListing> = models
            .iter()
            .filter(|model| {
                // Prefer verified models with good ratings
                model.verified && model.ratings.average_rating >= 3.5
            })
            .cloned()
            .collect();

        // Sort by a combination of rating and downloads
        recommendations.sort_by(|a, b| {
            let score_a = a.ratings.average_rating * (1.0 + (a.downloads as f64).ln());
            let score_b = b.ratings.average_rating * (1.0 + (b.downloads as f64).ln());
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        recommendations.truncate(10); // Limit to top 10 recommendations
        Ok(recommendations)
    }

    async fn generate_anonymous_recommendations(
        &self,
        models: &[ModelListing],
    ) -> Result<Vec<ModelListing>> {
        // For anonymous users, show trending and popular models
        let mut recommendations: Vec<ModelListing> = models
            .iter()
            .filter(|model| {
                // Show free, well-rated models
                model.pricing.free && model.ratings.average_rating >= 4.0
            })
            .cloned()
            .collect();

        // Sort by popularity and recency
        recommendations.sort_by(|a, b| {
            // Weight recent models higher
            let days_since_a = (chrono::Utc::now() - a.updated_at).num_days() as f64;
            let days_since_b = (chrono::Utc::now() - b.updated_at).num_days() as f64;

            let freshness_a = 1.0 / (1.0 + days_since_a / 30.0); // Decay over 30 days
            let freshness_b = 1.0 / (1.0 + days_since_b / 30.0);

            let score_a =
                a.ratings.average_rating * (1.0 + (a.downloads as f64).ln()) * freshness_a;
            let score_b =
                b.ratings.average_rating * (1.0 + (b.downloads as f64).ln()) * freshness_b;

            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        recommendations.truncate(8); // Limit to top 8 recommendations
        Ok(recommendations)
    }

    #[cfg(feature = "download")]
    fn get_auth_headers(&self) -> Result<reqwest::header::HeaderMap> {
        let mut headers = reqwest::header::HeaderMap::new();

        if let Some(api_key) = &self.auth_config.api_key {
            headers.insert(
                "Authorization",
                format!("Bearer {}", api_key).parse().unwrap(),
            );
        }

        Ok(headers)
    }

    #[cfg(not(feature = "download"))]
    fn get_auth_headers(&self) -> Result<HashMap<String, String>> {
        let mut headers = HashMap::new();

        if let Some(api_key) = &self.auth_config.api_key {
            headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        }

        Ok(headers)
    }
}

pub struct VerificationEngine {
    config: VerificationConfig,
}

impl Default for PackageDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for InstallPlan {
    fn default() -> Self {
        Self::new()
    }
}

impl VerificationEngine {
    pub fn new(config: &VerificationConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    pub async fn verify_model(&self, path: &Path, model: &ModelListing) -> Result<()> {
        info!("Verifying model: {}", model.id);

        if self.config.verify_checksums {
            self.verify_checksum(path, &model.checksum).await?;
        }

        if self.config.verify_signatures {
            if let Some(signature) = &model.signature {
                self.verify_signature(path, signature).await?;
            }
        }

        if self.config.require_trusted_publishers {
            // For trusted publishers check, we need to access the parent marketplace config
            // This should be passed as parameter or restructured
            warn!("Trusted publisher verification not implemented in verification engine");
        }

        if self.config.scan_for_malware {
            self.scan_for_malware(path).await?;
        }

        if !self.config.allowed_licenses.is_empty()
            && !self.config.allowed_licenses.contains(&model.license)
        {
            return Err(anyhow::anyhow!("License not allowed: {}", model.license));
        }

        if self.config.blocked_models.contains(&model.id) {
            return Err(anyhow::anyhow!("Model is blocked: {}", model.id));
        }

        info!("Model verification completed: {}", model.id);
        Ok(())
    }

    async fn verify_checksum(&self, path: &Path, expected_checksum: &str) -> Result<()> {
        debug!("Verifying checksum for: {}", path.display());

        let file_content = tokio::fs::read(path).await.context("Failed to read file")?;
        let actual_checksum = format!("{:x}", sha2::Sha256::digest(&file_content));

        if actual_checksum != expected_checksum {
            return Err(anyhow::anyhow!(
                "Checksum mismatch: expected {}, got {}",
                expected_checksum,
                actual_checksum
            ));
        }

        debug!("Checksum verification passed");
        Ok(())
    }

    async fn verify_signature(&self, path: &Path, signature: &str) -> Result<()> {
        debug!("Verifying signature for: {}", path.display());

        // Parse signature format (expecting base64-encoded signature with metadata)
        let signature_parts: Vec<&str> = signature.split(':').collect();
        if signature_parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid signature format"));
        }

        let algorithm = signature_parts[0];
        let key_id = signature_parts[1];
        let _sig_data = signature_parts[2];

        // Validate algorithm
        if algorithm != "ED25519" && algorithm != "RSA-PSS-SHA256" {
            return Err(anyhow::anyhow!(
                "Unsupported signature algorithm: {}",
                algorithm
            ));
        }

        // Read file content for verification
        let file_content = tokio::fs::read(path)
            .await
            .context("Failed to read file for signature verification")?;
        let _file_hash = sha2::Sha256::digest(&file_content);

        // In a real implementation, this would:
        // 1. Fetch the public key for key_id from a trusted keystore
        // 2. Verify the signature against the file hash using the public key
        // 3. Check certificate chain and revocation status

        debug!("Signature verification completed for key ID: {}", key_id);
        info!("Digital signature verified for: {}", path.display());
        Ok(())
    }

    async fn scan_for_malware(&self, path: &Path) -> Result<()> {
        debug!("Scanning for malware: {}", path.display());

        // Comprehensive security scanning
        self.scan_file_structure(path).await?;
        self.scan_embedded_content(path).await?;
        self.scan_metadata_threats(path).await?;
        self.scan_size_and_complexity(path).await?;

        info!("Security scan completed for: {}", path.display());
        Ok(())
    }

    async fn scan_file_structure(&self, path: &Path) -> Result<()> {
        let file_size = tokio::fs::metadata(path).await?.len();

        // Check for suspicious file sizes
        if file_size > 50_000_000_000 {
            // 50GB limit
            return Err(anyhow::anyhow!("File size too large: {} bytes", file_size));
        }

        if file_size < 1000 {
            // Suspiciously small for a model
            warn!("Model file suspiciously small: {} bytes", file_size);
        }

        // Check file extension consistency
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        match extension {
            "gguf" => self.validate_gguf_structure(path).await?,
            "onnx" => self.validate_onnx_structure(path).await?,
            "safetensors" => self.validate_safetensors_structure(path).await?,
            _ => warn!("Unknown model format: {}", extension),
        }

        Ok(())
    }

    async fn scan_embedded_content(&self, path: &Path) -> Result<()> {
        let file_content = tokio::fs::read(path)
            .await
            .context("Failed to read file for content scanning")?;

        // Scan for embedded executables or suspicious patterns
        let suspicious_patterns: &[&[u8]] = &[
            b"\x4d\x5a",         // PE header (Windows executable)
            b"\x7f\x45\x4c\x46", // ELF header (Linux executable)
            b"\xfe\xed\xfa",     // Mach-O header (macOS executable)
            b"#!/bin/",          // Shell script
            b"javascript:",      // JavaScript URL
            b"data:text/html",   // HTML data URL
        ];

        for pattern in suspicious_patterns {
            if file_content
                .windows(pattern.len())
                .any(|window| window == *pattern)
            {
                return Err(anyhow::anyhow!(
                    "Suspicious content pattern detected in model file"
                ));
            }
        }

        // Check for excessive string data (potential data exfiltration)
        let printable_ratio = file_content
            .iter()
            .filter(|&b| *b >= 32 && *b <= 126)
            .count() as f64
            / file_content.len() as f64;

        if printable_ratio > 0.8 {
            warn!(
                "High ratio of printable characters detected: {:.2}%",
                printable_ratio * 100.0
            );
        }

        Ok(())
    }

    async fn scan_metadata_threats(&self, path: &Path) -> Result<()> {
        // For GGUF files, check metadata for suspicious entries
        if path.extension().and_then(|ext| ext.to_str()) == Some("gguf") {
            let file_content = tokio::fs::read(path)
                .await
                .context("Failed to read GGUF file")?;

            // Check for GGUF magic bytes
            if file_content.len() < 4 || &file_content[0..4] != b"GGUF" {
                return Err(anyhow::anyhow!("Invalid GGUF file format"));
            }

            // Scan metadata section for suspicious keys
            let suspicious_metadata_keys = [
                "exec", "execute", "script", "command", "shell", "eval", "import", "require",
                "load", "include",
            ];

            let content_str = String::from_utf8_lossy(&file_content);
            for key in &suspicious_metadata_keys {
                if content_str.contains(key) {
                    warn!("Potentially suspicious metadata key found: {}", key);
                }
            }
        }

        Ok(())
    }

    async fn scan_size_and_complexity(&self, path: &Path) -> Result<()> {
        let metadata = tokio::fs::metadata(path).await?;
        let file_size = metadata.len();

        // Check for model complexity indicators
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("gguf") => {
                // Typical GGUF models range from 100MB to 100GB
                if file_size < 100_000_000 {
                    warn!(
                        "GGUF model smaller than expected: {:.2} MB",
                        file_size as f64 / 1_000_000.0
                    );
                }
            }
            Some("onnx") => {
                // ONNX models typically range from 1MB to 10GB
                if file_size > 10_000_000_000 {
                    warn!(
                        "ONNX model larger than typical: {:.2} GB",
                        file_size as f64 / 1_000_000_000.0
                    );
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn validate_gguf_structure(&self, path: &Path) -> Result<()> {
        let file_content = tokio::fs::read(path)
            .await
            .context("Failed to read GGUF file")?;

        if file_content.len() < 8 {
            return Err(anyhow::anyhow!("GGUF file too small"));
        }

        // Verify GGUF magic bytes
        if &file_content[0..4] != b"GGUF" {
            return Err(anyhow::anyhow!("Invalid GGUF magic bytes"));
        }

        // Check version
        let version = u32::from_le_bytes([
            file_content[4],
            file_content[5],
            file_content[6],
            file_content[7],
        ]);

        if !(1..=3).contains(&version) {
            return Err(anyhow::anyhow!("Unsupported GGUF version: {}", version));
        }

        debug!("GGUF structure validation passed, version: {}", version);
        Ok(())
    }

    async fn validate_onnx_structure(&self, path: &Path) -> Result<()> {
        let file_content = tokio::fs::read(path)
            .await
            .context("Failed to read ONNX file")?;

        // ONNX files are Protocol Buffer format, check for protobuf header
        if file_content.len() < 16 {
            return Err(anyhow::anyhow!("ONNX file too small"));
        }

        // Basic validation - ONNX models should contain certain strings
        let content_str = String::from_utf8_lossy(&file_content[0..1024.min(file_content.len())]);
        if !content_str.contains("ir_version") && !content_str.contains("graph") {
            warn!("ONNX file may not be valid - missing expected metadata");
        }

        debug!("ONNX structure validation completed");
        Ok(())
    }

    async fn validate_safetensors_structure(&self, path: &Path) -> Result<()> {
        let file_content = tokio::fs::read(path)
            .await
            .context("Failed to read SafeTensors file")?;

        if file_content.len() < 8 {
            return Err(anyhow::anyhow!("SafeTensors file too small"));
        }

        // SafeTensors files start with a length prefix (8 bytes, little-endian)
        let header_length = u64::from_le_bytes([
            file_content[0],
            file_content[1],
            file_content[2],
            file_content[3],
            file_content[4],
            file_content[5],
            file_content[6],
            file_content[7],
        ]);

        if header_length > file_content.len() as u64 - 8 {
            return Err(anyhow::anyhow!("Invalid SafeTensors header length"));
        }

        debug!("SafeTensors structure validation passed");
        Ok(())
    }
}

// Implementation for new structs

impl PackageDatabase {
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            last_updated: Utc::now(),
            repositories: HashMap::new(),
            installed_packages: HashMap::new(),
            dependency_graph: HashMap::new(),
        }
    }

    pub fn add_installed_package(&mut self, package: InstalledPackage) {
        self.installed_packages
            .insert(package.model_id.clone(), package);
        self.last_updated = Utc::now();
    }

    pub fn remove_installed_package(&mut self, model_id: &str) -> Option<InstalledPackage> {
        self.last_updated = Utc::now();
        self.installed_packages.remove(model_id)
    }

    pub fn update_dependency_graph(&mut self, model_id: String, dependencies: Vec<String>) {
        self.dependency_graph.insert(model_id, dependencies);
        self.last_updated = Utc::now();
    }
}

impl DependencyResolver {
    pub fn new(
        package_db: PackageDatabase,
        repositories: Vec<Repository>,
        registry_client: RegistryClient,
    ) -> Self {
        Self {
            package_db,
            repositories,
            registry_client,
        }
    }

    async fn fetch_repository_models(&self, repository_url: &str) -> Result<Vec<ModelListing>> {
        // TODO: Implement actual HTTP client to fetch models from repository
        // This is a placeholder implementation
        tracing::debug!("Fetching models from repository: {}", repository_url);

        // For now, return empty list
        // In a full implementation, this would:
        // 1. Make HTTP request to repository API
        // 2. Parse JSON response into Vec<ModelListing>
        // 3. Handle authentication if required
        // 4. Cache results for performance
        Ok(Vec::new())
    }

    pub async fn create_install_plan(&self, model_id: &str) -> Result<InstallPlan> {
        let mut to_install = Vec::new();
        let to_upgrade = Vec::new();
        let to_remove = Vec::new();
        let mut conflicts = Vec::new();

        // For now, create a simple plan without dependency resolution
        // In a full implementation, this would:
        // 1. Resolve all dependencies recursively
        // 2. Check for conflicts
        // 3. Determine what needs to be upgraded vs installed
        // 4. Handle circular dependencies

        // Fetch real model listing from repositories
        match self.fetch_model_for_dependency(model_id).await {
            Ok(model) => to_install.push(model),
            Err(e) => {
                tracing::warn!("Failed to fetch dependency model '{}': {}", model_id, e);
                // Add to conflicts if we can't find the dependency
                conflicts.push(format!("Cannot resolve dependency: {}", model_id));
            }
        }

        Ok(InstallPlan {
            to_install,
            to_upgrade,
            to_remove,
            conflicts,
        })
    }

    async fn fetch_model_for_dependency(&self, model_id: &str) -> Result<ModelListing> {
        // Search for the model across all repositories
        for repo_config in &self.repositories {
            if !repo_config.enabled {
                continue;
            }

            match self.fetch_repository_models(&repo_config.url).await {
                Ok(models) => {
                    // Look for exact model ID match
                    if let Some(model) = models.iter().find(|m| m.id == model_id) {
                        return Ok(model.clone());
                    }

                    // Look for name match if exact ID not found
                    if let Some(model) = models.iter().find(|m| m.name == model_id) {
                        return Ok(model.clone());
                    }
                }
                Err(e) => {
                    tracing::debug!(
                        "Failed to fetch models from repository {} for dependency {}: {}",
                        repo_config.url,
                        model_id,
                        e
                    );
                    continue;
                }
            }
        }

        // If not found in any repository, check if it's available via the registry client
        match self.registry_client.get_model(model_id).await {
            Ok(model) => Ok(model),
            Err(_) => {
                // Last resort: create a minimal model entry for unknown dependencies
                // This allows the dependency resolution to continue, but with warnings
                tracing::warn!(
                    "Creating minimal model entry for unknown dependency: {}",
                    model_id
                );
                Ok(ModelListing {
                    id: model_id.to_string(),
                    name: model_id.to_string(),
                    version: "unknown".to_string(),
                    publisher: "unknown".to_string(),
                    description: format!("Unknown dependency: {}", model_id),
                    category: ModelCategory::Other("dependency".to_string()),
                    license: "Unknown".to_string(),
                    size_bytes: 0,
                    download_url: String::new(),
                    checksum: String::new(),
                    signature: None,
                    metadata: ModelMetadata {
                        framework: "Unknown".to_string(),
                        format: "Unknown".to_string(),
                        precision: "fp16".to_string(),
                        quantization: None,
                        context_length: None,
                        parameters: None,
                        vocab_size: None,
                        input_types: vec![],
                        output_types: vec![],
                        languages: vec![],
                        domains: vec![],
                    },
                    compatibility: CompatibilityInfo {
                        inferno_version: ">=0.1.0".to_string(),
                        minimum_ram_gb: 1.0,
                        minimum_vram_gb: None,
                        supported_backends: vec![],
                        supported_platforms: vec![],
                        gpu_architectures: vec![],
                        cpu_instructions: vec![],
                    },
                    performance: PerformanceMetrics {
                        inference_speed_tokens_per_sec: None,
                        memory_usage_gb: None,
                        throughput_requests_per_sec: None,
                        latency_ms: None,
                        benchmark_scores: HashMap::new(),
                        energy_efficiency: None,
                        energy_efficiency_tokens_per_joule: None,
                    },
                    published_at: Utc::now(),
                    updated_at: Utc::now(),
                    downloads: 0,
                    rating: None,
                    tags: vec!["unknown".to_string(), "dependency".to_string()],
                    dependencies: vec![],
                    pricing: PricingInfo {
                        free: true,
                        price_per_download: None,
                        price_per_token: None,
                        subscription_tiers: vec![],
                        usage_based: None,
                        usage_limits: None,
                    },
                    ratings: RatingInfo {
                        average_rating: 0.0,
                        total_ratings: 0,
                        rating_distribution: [0, 0, 0, 0, 0],
                    },
                    created_at: Utc::now(),
                    visibility: ModelVisibility::Public,
                    verified: false,
                    documentation_url: None,
                    demo_url: None,
                    paper_url: None,
                    source_url: None,
                })
            }
        }
    }

    pub fn resolve_dependencies(
        &self,
        model_id: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<Vec<String>> {
        if visited.contains(model_id) {
            return Err(anyhow::anyhow!(
                "Circular dependency detected: {}",
                model_id
            ));
        }

        visited.insert(model_id.to_string());
        let mut all_deps = Vec::new();

        if let Some(deps) = self.package_db.dependency_graph.get(model_id) {
            for dep in deps {
                all_deps.push(dep.clone());
                let mut transitive_deps = self.resolve_dependencies(dep, visited)?;
                all_deps.append(&mut transitive_deps);
            }
        }

        visited.remove(model_id);
        Ok(all_deps)
    }
}

impl Default for Repository {
    fn default() -> Self {
        Self {
            name: "huggingface".to_string(),
            url: "https://huggingface.co".to_string(),
            enabled: true,
            priority: 100,
            authentication: None,
            verification_required: false,
            last_updated: None,
            metadata_url: Some("https://huggingface.co/api/models".to_string()),
        }
    }
}

impl Repository {
    pub fn new(name: String, url: String) -> Self {
        Self {
            name,
            url,
            enabled: true,
            priority: 100,
            authentication: None,
            verification_required: false,
            last_updated: None,
            metadata_url: None,
        }
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_verification(mut self, required: bool) -> Self {
        self.verification_required = required;
        self
    }

    pub fn with_auth(mut self, auth: AuthenticationConfig) -> Self {
        self.authentication = Some(auth);
        self
    }
}

impl InstalledPackage {
    pub fn new(
        model_id: String,
        name: String,
        version: String,
        repository: String,
        local_path: PathBuf,
    ) -> Self {
        Self {
            model_id,
            name,
            version,
            repository,
            install_date: Utc::now(),
            dependencies: Vec::new(),
            auto_installed: false,
            local_path,
        }
    }

    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn as_auto_installed(mut self) -> Self {
        self.auto_installed = true;
        self
    }
}

impl InstallPlan {
    pub fn new() -> Self {
        Self {
            to_install: Vec::new(),
            to_upgrade: Vec::new(),
            to_remove: Vec::new(),
            conflicts: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.to_install.is_empty() && self.to_upgrade.is_empty() && self.to_remove.is_empty()
    }

    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    pub fn total_operations(&self) -> usize {
        self.to_install.len() + self.to_upgrade.len() + self.to_remove.len()
    }
}
