use crate::{
    backends::{Backend, BackendConfig, BackendType, InferenceParams},
    models::{ModelInfo, ModelManager},
    metrics::MetricsCollector,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{RwLock, Semaphore},
    task::JoinHandle,
    time::{interval, timeout},
};
use tracing::{debug, error, info, warn};

/// Configuration for model caching and warm-up strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of models to keep in memory
    pub max_cached_models: usize,
    /// Maximum memory usage for cached models in MB
    pub max_memory_mb: u64,
    /// Time before unused models are evicted (seconds)
    pub model_ttl_seconds: u64,
    /// Enable automatic model warm-up based on usage patterns
    pub enable_warmup: bool,
    /// Warm-up strategy to use
    pub warmup_strategy: WarmupStrategy,
    /// Models to always keep warm (preload on startup)
    pub always_warm: Vec<String>,
    /// Enable predictive loading based on usage patterns
    pub predictive_loading: bool,
    /// Time window for usage pattern analysis (seconds)
    pub usage_window_seconds: u64,
    /// Minimum usage frequency to trigger predictive loading
    pub min_usage_frequency: f64,
    /// Enable memory-based eviction
    pub memory_based_eviction: bool,
    /// Cache persistence to disk
    pub persist_cache: bool,
    /// Cache directory for persistence
    pub cache_dir: Option<PathBuf>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_cached_models: 5,
            max_memory_mb: 8192, // 8GB
            model_ttl_seconds: 3600, // 1 hour
            enable_warmup: true,
            warmup_strategy: WarmupStrategy::UsageBased,
            always_warm: Vec::new(),
            predictive_loading: true,
            usage_window_seconds: 86400, // 24 hours
            min_usage_frequency: 0.1, // 10% of requests
            memory_based_eviction: true,
            persist_cache: false,
            cache_dir: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarmupStrategy {
    /// Load models based on recent usage patterns
    UsageBased,
    /// Load models based on predicted future usage
    Predictive,
    /// Load models in order of file size (smallest first)
    SizeOptimized,
    /// Load models based on priority configuration
    Priority,
    /// Hybrid approach combining multiple strategies
    Hybrid,
}

pub struct CachedModel {
    pub backend: Backend,
    pub model_info: ModelInfo,
    pub last_used: Instant,
    pub created_at: Instant,
    pub usage_count: AtomicU64,
    pub memory_estimate: u64,
    pub warmup_priority: u8,
}

impl std::fmt::Debug for CachedModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedModel")
            .field("model_info", &self.model_info)
            .field("last_used", &self.last_used)
            .field("created_at", &self.created_at)
            .field("usage_count", &self.usage_count.load(Ordering::Relaxed))
            .field("memory_estimate", &self.memory_estimate)
            .field("warmup_priority", &self.warmup_priority)
            .finish()
    }
}

impl Clone for CachedModel {
    fn clone(&self) -> Self {
        // Note: This creates a new backend instance, which might not be ideal
        // In practice, you might want to use Arc<Backend> instead
        panic!("CachedModel cannot be cloned due to Backend limitations")
    }
}

#[derive(Debug, Clone)]
pub struct ModelUsageStats {
    pub model_name: String,
    pub request_count: u64,
    pub last_request: SystemTime,
    pub average_response_time: Duration,
    pub total_response_time: Duration,
    pub memory_usage: u64,
    pub usage_frequency: f64, // requests per hour
    pub usage_trend: f64, // positive = increasing usage, negative = decreasing
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_models: usize,
    pub memory_usage_mb: f64,
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub eviction_count: u64,
    pub warmup_count: u64,
    pub active_models: Vec<String>,
    pub model_stats: HashMap<String, ModelUsageStats>,
}

/// Advanced model cache with intelligent warm-up strategies
pub struct ModelCache {
    pub config: CacheConfig,
    backend_config: BackendConfig,
    model_manager: Arc<ModelManager>,
    metrics: Option<Arc<MetricsCollector>>,

    // Cache storage
    cached_models: Arc<RwLock<HashMap<String, Arc<CachedModel>>>>,
    usage_stats: Arc<RwLock<HashMap<String, ModelUsageStats>>>,

    // Statistics
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    evictions: AtomicU64,
    warmups: AtomicU64,
    total_memory: AtomicU64,

    // Background tasks
    cleanup_task: Option<JoinHandle<()>>,
    warmup_task: Option<JoinHandle<()>>,
    stats_task: Option<JoinHandle<()>>,

    // Concurrency control
    loading_semaphore: Arc<Semaphore>,
}

impl ModelCache {
    /// Create a new model cache
    pub async fn new(
        config: CacheConfig,
        backend_config: BackendConfig,
        model_manager: Arc<ModelManager>,
        metrics: Option<Arc<MetricsCollector>>,
    ) -> Result<Self> {
        info!("Initializing model cache with strategy: {:?}", config.warmup_strategy);

        let cached_models = Arc::new(RwLock::new(HashMap::new()));
        let usage_stats = Arc::new(RwLock::new(HashMap::new()));

        let mut cache = Self {
            config: config.clone(),
            backend_config,
            model_manager,
            metrics,
            cached_models: cached_models.clone(),
            usage_stats: usage_stats.clone(),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            warmups: AtomicU64::new(0),
            total_memory: AtomicU64::new(0),
            cleanup_task: None,
            warmup_task: None,
            stats_task: None,
            loading_semaphore: Arc::new(Semaphore::new(2)), // Allow 2 concurrent model loads
        };

        // Start background tasks
        cache.start_background_tasks().await?;

        // Load always-warm models
        cache.warmup_always_warm_models().await?;

        // Load cached models from disk if persistence is enabled
        if config.persist_cache {
            cache.load_from_disk().await?;
        }

        Ok(cache)
    }

    /// Get a model from cache, loading it if necessary
    pub async fn get_model(&self, model_name: &str) -> Result<Arc<CachedModel>> {
        // Try to get from cache first
        {
            let cache_guard = self.cached_models.read().await;
            if let Some(cached_model) = cache_guard.get(model_name) {
                // Update last used time
                let mut model = Arc::try_unwrap(cached_model.clone()).unwrap_or_else(|arc| (*arc).clone());
                // Can't modify Arc contents directly, so we'll track usage separately

                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                self.update_usage_stats(model_name, Duration::from_millis(0)).await;

                debug!("Cache hit for model: {}", model_name);
                return Ok(cached_model.clone());
            }
        }

        // Cache miss - load the model
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
        info!("Cache miss for model: {}, loading...", model_name);

        // Acquire semaphore to limit concurrent loads
        let _permit = self.loading_semaphore.acquire().await
            .map_err(|_| anyhow!("Failed to acquire loading permit"))?;

        let start_time = Instant::now();
        let cached_model = self.load_model(model_name).await?;
        let load_time = start_time.elapsed();

        info!("Model {} loaded in {:?}", model_name, load_time);
        self.update_usage_stats(model_name, load_time).await;

        // Check if we need to evict models
        self.maybe_evict_models().await?;

        Ok(cached_model)
    }

    /// Warm up models based on the configured strategy
    pub async fn warmup_models(&self) -> Result<()> {
        match self.config.warmup_strategy {
            WarmupStrategy::UsageBased => self.warmup_usage_based().await,
            WarmupStrategy::Predictive => self.warmup_predictive().await,
            WarmupStrategy::SizeOptimized => self.warmup_size_optimized().await,
            WarmupStrategy::Priority => self.warmup_priority_based().await,
            WarmupStrategy::Hybrid => self.warmup_hybrid().await,
        }
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let cached_models = self.cached_models.read().await;
        let usage_stats = self.usage_stats.read().await;

        let total_requests = self.cache_hits.load(Ordering::Relaxed) + self.cache_misses.load(Ordering::Relaxed);
        let hit_rate = if total_requests > 0 {
            self.cache_hits.load(Ordering::Relaxed) as f64 / total_requests as f64
        } else {
            0.0
        };

        CacheStats {
            total_models: cached_models.len(),
            memory_usage_mb: self.total_memory.load(Ordering::Relaxed) as f64 / (1024.0 * 1024.0),
            hit_rate,
            miss_rate: 1.0 - hit_rate,
            eviction_count: self.evictions.load(Ordering::Relaxed),
            warmup_count: self.warmups.load(Ordering::Relaxed),
            active_models: cached_models.keys().cloned().collect(),
            model_stats: usage_stats.clone(),
        }
    }

    /// Explicitly warm up a specific model
    pub async fn warmup_model(&self, model_name: &str) -> Result<()> {
        info!("Warming up model: {}", model_name);
        let _cached_model = self.load_model(model_name).await?;
        self.warmups.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Evict a specific model from cache
    pub async fn evict_model(&self, model_name: &str) -> Result<()> {
        let mut cached_models = self.cached_models.write().await;
        if let Some(model) = cached_models.remove(model_name) {
            self.total_memory.fetch_sub(model.memory_estimate, Ordering::Relaxed);
            self.evictions.fetch_add(1, Ordering::Relaxed);
            info!("Evicted model: {}", model_name);
        }
        Ok(())
    }

    /// Clear all cached models
    pub async fn clear_cache(&self) -> Result<()> {
        let mut cached_models = self.cached_models.write().await;
        cached_models.clear();
        self.total_memory.store(0, Ordering::Relaxed);
        info!("Cleared all cached models");
        Ok(())
    }

    /// Load a model and cache it
    async fn load_model(&self, model_name: &str) -> Result<Arc<CachedModel>> {
        let model_info = self.model_manager.resolve_model(model_name).await?;
        let backend_type = BackendType::from_model_path(&model_info.path);

        let mut backend = Backend::new(backend_type, &self.backend_config)?;
        backend.load_model(&model_info).await?;

        let memory_estimate = self.estimate_model_memory(&model_info);
        let cached_model = Arc::new(CachedModel {
            backend,
            model_info: model_info.clone(),
            last_used: Instant::now(),
            created_at: Instant::now(),
            usage_count: AtomicU64::new(0),
            memory_estimate,
            warmup_priority: self.calculate_warmup_priority(model_name).await,
        });

        // Add to cache
        {
            let mut cached_models = self.cached_models.write().await;
            cached_models.insert(model_name.to_string(), cached_model.clone());
            self.total_memory.fetch_add(memory_estimate, Ordering::Relaxed);
        }

        // Initialize usage stats if not exists
        {
            let mut usage_stats = self.usage_stats.write().await;
            usage_stats.entry(model_name.to_string()).or_insert_with(|| ModelUsageStats {
                model_name: model_name.to_string(),
                request_count: 0,
                last_request: SystemTime::now(),
                average_response_time: Duration::ZERO,
                total_response_time: Duration::ZERO,
                memory_usage: memory_estimate,
                usage_frequency: 0.0,
                usage_trend: 0.0,
            });
        }

        Ok(cached_model)
    }

    /// Update usage statistics for a model
    async fn update_usage_stats(&self, model_name: &str, response_time: Duration) {
        let mut usage_stats = self.usage_stats.write().await;
        if let Some(stats) = usage_stats.get_mut(model_name) {
            stats.request_count += 1;
            stats.last_request = SystemTime::now();
            stats.total_response_time += response_time;
            stats.average_response_time = stats.total_response_time / stats.request_count as u32;

            // Calculate usage frequency (requests per hour in the last window)
            let window_start = SystemTime::now() - Duration::from_secs(self.config.usage_window_seconds);
            if stats.last_request >= window_start {
                let hours = self.config.usage_window_seconds as f64 / 3600.0;
                stats.usage_frequency = stats.request_count as f64 / hours;
            }
        }
    }

    /// Maybe evict models based on cache limits
    async fn maybe_evict_models(&self) -> Result<()> {
        let cached_models = self.cached_models.read().await;
        let total_models = cached_models.len();
        let current_memory = self.total_memory.load(Ordering::Relaxed);
        drop(cached_models);

        let should_evict = total_models > self.config.max_cached_models ||
                          current_memory > self.config.max_memory_mb * 1024 * 1024;

        if should_evict {
            self.evict_least_recently_used().await?;
        }

        Ok(())
    }

    /// Evict the least recently used model
    async fn evict_least_recently_used(&self) -> Result<()> {
        let cached_models = self.cached_models.read().await;

        // Find the model with the oldest last_used time and lowest usage
        let mut oldest_time = Instant::now();
        let mut victim_model = None;
        let mut lowest_priority = u8::MAX;

        for (name, model) in cached_models.iter() {
            // Skip always-warm models
            if self.config.always_warm.contains(name) {
                continue;
            }

            let is_older = model.last_used < oldest_time;
            let lower_priority = model.warmup_priority < lowest_priority;

            if is_older || (model.last_used == oldest_time && lower_priority) {
                oldest_time = model.last_used;
                lowest_priority = model.warmup_priority;
                victim_model = Some(name.clone());
            }
        }
        drop(cached_models);

        if let Some(model_name) = victim_model {
            info!("Evicting least recently used model: {}", model_name);
            self.evict_model(&model_name).await?;
        }

        Ok(())
    }

    /// Start background tasks for maintenance
    async fn start_background_tasks(&mut self) -> Result<()> {
        // Cleanup task
        let cleanup_cached_models = self.cached_models.clone();
        let cleanup_config = self.config.clone();
        let cleanup_evictions = Arc::new(AtomicU64::new(self.evictions.load(Ordering::Relaxed)));
        let cleanup_total_memory = Arc::new(AtomicU64::new(self.total_memory.load(Ordering::Relaxed)));

        self.cleanup_task = Some(tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(300)); // 5 minutes

            loop {
                cleanup_interval.tick().await;

                let mut cached_models = cleanup_cached_models.write().await;
                let now = Instant::now();
                let ttl = Duration::from_secs(cleanup_config.model_ttl_seconds);

                let mut to_remove = Vec::new();
                for (name, model) in cached_models.iter() {
                    if now.duration_since(model.last_used) > ttl &&
                       !cleanup_config.always_warm.contains(name) {
                        to_remove.push((name.clone(), model.memory_estimate));
                    }
                }

                for (name, memory) in to_remove {
                    cached_models.remove(&name);
                    cleanup_total_memory.fetch_sub(memory, Ordering::Relaxed);
                    cleanup_evictions.fetch_add(1, Ordering::Relaxed);
                    debug!("TTL expired, evicted model: {}", name);
                }
            }
        }));

        // Warmup task disabled due to self-reference complexity
        // In a production implementation, this would use Arc<Self> or other patterns

        Ok(())
    }

    /// Warm up always-warm models
    async fn warmup_always_warm_models(&self) -> Result<()> {
        for model_name in &self.config.always_warm {
            if let Err(e) = self.warmup_model(model_name).await {
                warn!("Failed to warm up always-warm model {}: {}", model_name, e);
            }
        }
        Ok(())
    }

    /// Warm up models based on usage patterns
    async fn warmup_usage_based(&self) -> Result<()> {
        let usage_stats = self.usage_stats.read().await;
        let mut candidates: Vec<_> = usage_stats
            .values()
            .filter(|stats| stats.usage_frequency >= self.config.min_usage_frequency)
            .collect();

        // Sort by usage frequency
        candidates.sort_by(|a, b| b.usage_frequency.partial_cmp(&a.usage_frequency).unwrap_or(std::cmp::Ordering::Equal));

        // Warm up top candidates that aren't already cached
        for stats in candidates.iter().take(3) { // Warm up top 3
            let cached_models = self.cached_models.read().await;
            let should_warmup = !cached_models.contains_key(&stats.model_name);
            drop(cached_models);

            if should_warmup {
                if let Err(e) = self.warmup_model(&stats.model_name).await {
                    warn!("Failed to warm up model {}: {}", stats.model_name, e);
                }
            }
        }

        Ok(())
    }

    /// Warm up models based on predictive analysis
    async fn warmup_predictive(&self) -> Result<()> {
        // Simple predictive logic based on usage trends
        let usage_stats = self.usage_stats.read().await;
        let mut candidates: Vec<_> = usage_stats
            .values()
            .filter(|stats| stats.usage_trend > 0.1) // Positive trend
            .collect();

        candidates.sort_by(|a, b| b.usage_trend.partial_cmp(&a.usage_trend).unwrap_or(std::cmp::Ordering::Equal));

        for stats in candidates.iter().take(2) {
            if let Err(e) = self.warmup_model(&stats.model_name).await {
                warn!("Failed to predictively warm up model {}: {}", stats.model_name, e);
            }
        }

        Ok(())
    }

    /// Warm up models optimized by size
    async fn warmup_size_optimized(&self) -> Result<()> {
        // Get all available models and sort by size
        if let Ok(models) = self.model_manager.list_models().await {
            let mut sorted_models = models;
            sorted_models.sort_by(|a, b| a.size.cmp(&b.size));

            // Warm up smaller models first
            for model in sorted_models.iter().take(3) {
                if let Err(e) = self.warmup_model(&model.name).await {
                    warn!("Failed to warm up size-optimized model {}: {}", model.name, e);
                }
            }
        }

        Ok(())
    }

    /// Warm up models based on priority
    async fn warmup_priority_based(&self) -> Result<()> {
        // Use always_warm list as priority
        for model_name in &self.config.always_warm {
            if let Err(e) = self.warmup_model(model_name).await {
                warn!("Failed to warm up priority model {}: {}", model_name, e);
            }
        }
        Ok(())
    }

    /// Hybrid warmup strategy
    async fn warmup_hybrid(&self) -> Result<()> {
        // Combine multiple strategies
        self.warmup_always_warm_models().await?;
        self.warmup_usage_based().await?;
        self.warmup_predictive().await?;
        Ok(())
    }

    /// Estimate memory usage for a model
    fn estimate_model_memory(&self, model_info: &ModelInfo) -> u64 {
        // Simple estimation based on file size
        // In practice, this could be more sophisticated
        (model_info.size as f64 * 1.2) as u64 // 20% overhead estimate
    }

    /// Calculate warmup priority for a model
    async fn calculate_warmup_priority(&self, model_name: &str) -> u8 {
        // Priority based on always_warm list and usage stats
        if self.config.always_warm.contains(&model_name.to_string()) {
            return 255; // Highest priority
        }

        let usage_stats = self.usage_stats.read().await;
        if let Some(stats) = usage_stats.get(model_name) {
            // Priority based on usage frequency
            (stats.usage_frequency.min(10.0) * 25.0) as u8
        } else {
            1 // Lowest priority for unknown models
        }
    }

    /// Load cache state from disk
    async fn load_from_disk(&self) -> Result<()> {
        if let Some(cache_dir) = &self.config.cache_dir {
            // Implementation for loading cached model metadata from disk
            // This would restore usage statistics and warm frequently used models
            info!("Loading cache state from disk: {:?}", cache_dir);
            // TODO: Implement actual disk loading logic
        }
        Ok(())
    }

    /// Save cache state to disk
    async fn save_to_disk(&self) -> Result<()> {
        if let Some(cache_dir) = &self.config.cache_dir {
            // Implementation for saving cached model metadata to disk
            info!("Saving cache state to disk: {:?}", cache_dir);
            // TODO: Implement actual disk saving logic
        }
        Ok(())
    }
}

impl Drop for ModelCache {
    fn drop(&mut self) {
        // Cancel background tasks
        if let Some(task) = self.cleanup_task.take() {
            task.abort();
        }
        if let Some(task) = self.warmup_task.take() {
            task.abort();
        }
        if let Some(task) = self.stats_task.take() {
            task.abort();
        }
    }
}