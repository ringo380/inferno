#![allow(dead_code, unused_imports, unused_variables)]
use crate::{
    backends::{BackendConfig, BackendHandle, BackendType},
    metrics::MetricsCollector,
    models::{ModelInfo, ModelManager},
};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    fs as async_fs,
    sync::{RwLock, Semaphore},
    task::JoinHandle,
    time::interval,
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
    /// How often the cache is written to disk in the background (seconds)
    pub persist_interval_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_cached_models: 5,
            max_memory_mb: 8192,     // 8GB
            model_ttl_seconds: 3600, // 1 hour
            enable_warmup: true,
            warmup_strategy: WarmupStrategy::UsageBased,
            always_warm: Vec::new(),
            predictive_loading: true,
            usage_window_seconds: 86400, // 24 hours
            min_usage_frequency: 0.1,    // 10% of requests
            memory_based_eviction: true,
            persist_cache: false,
            cache_dir: None,
            persist_interval_seconds: 300, // 5 minutes
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
    pub backend: BackendHandle,
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
        // BackendHandle is now cloneable, so we can safely clone CachedModel
        Self {
            backend: self.backend.clone(),
            model_info: self.model_info.clone(),
            last_used: self.last_used,
            created_at: self.created_at,
            usage_count: AtomicU64::new(self.usage_count.load(Ordering::Relaxed)),
            memory_estimate: self.memory_estimate,
            warmup_priority: self.warmup_priority,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsageStats {
    pub model_name: String,
    pub request_count: u64,
    pub last_request: SystemTime,
    pub average_response_time: Duration,
    pub total_response_time: Duration,
    pub memory_usage: u64,
    pub usage_frequency: f64, // requests per hour
    pub usage_trend: f64,     // positive = increasing usage, negative = decreasing
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

/// Serializable cache entry for disk persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableCacheEntry {
    pub model_name: String,
    pub model_info: ModelInfo,
    pub last_used_timestamp: u64,  // Unix timestamp
    pub created_at_timestamp: u64, // Unix timestamp
    pub usage_count: u64,
    pub memory_estimate: u64,
    pub warmup_priority: u8,
}

/// Serializable cache state for disk persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableCacheState {
    pub version: u32,
    pub cache_entries: Vec<SerializableCacheEntry>,
    pub usage_stats: HashMap<String, ModelUsageStats>,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
    pub warmups: u64,
    pub total_memory: u64,
    pub saved_at: u64, // Unix timestamp
}

// Constants for cache file format
const CACHE_FORMAT_VERSION: u32 = 1;
const CACHE_FILE_NAME: &str = "cache_state.bin.zst";
const CACHE_STATS_FILE_NAME: &str = "cache_stats.bin.zst";

/// Distinguishes concurrent saves' temporary files.
static SAVE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A scratch path for one save.
///
/// The rename onto the real file is atomic, but the temporary file it renames
/// from must be unique: a shared name lets two saves in flight write the same
/// scratch file, and whichever renames second fails because the first already
/// moved it away. The periodic save and an explicit `save_cache` can overlap
/// exactly this way.
fn temp_save_path(file_path: &std::path::Path) -> PathBuf {
    file_path.with_extension(format!(
        "tmp.{}.{}",
        std::process::id(),
        SAVE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ))
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

    // Maps a caller-supplied spelling (bare name, name+extension, relative or
    // `./`-prefixed path, symlink) to the canonical cache key so repeat lookups
    // of the same spelling skip the resolve/canonicalize disk work.
    alias_map: Arc<RwLock<HashMap<String, String>>>,

    // Statistics
    //
    // These are shared with the background persistence task, which must observe
    // the live counts. Copying their values into the task would freeze the
    // persisted statistics at whatever they were when the cache was built.
    cache_hits: Arc<AtomicU64>,
    cache_misses: Arc<AtomicU64>,
    evictions: Arc<AtomicU64>,
    warmups: Arc<AtomicU64>,
    total_memory: Arc<AtomicU64>,

    // Background tasks
    cleanup_task: Option<JoinHandle<()>>,
    warmup_task: Option<JoinHandle<()>>,
    stats_task: Option<JoinHandle<()>>,

    // Concurrency control
    loading_semaphore: Arc<Semaphore>,
}

/// Canonicalized absolute path as a stable cache key. Falls back to the given
/// path if canonicalization fails (e.g. the file was removed), so lookups still
/// resolve deterministically. Mirrors the keying `ModelManager` uses for its
/// on-disk metadata cache, which distinguishes same-named models in different
/// directories rather than collapsing them.
fn canonical_key(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
}

impl ModelCache {
    /// Create a new model cache
    pub async fn new(
        config: CacheConfig,
        backend_config: BackendConfig,
        model_manager: Arc<ModelManager>,
        metrics: Option<Arc<MetricsCollector>>,
    ) -> Result<Self> {
        info!(
            "Initializing model cache with strategy: {:?}",
            config.warmup_strategy
        );

        let cached_models = Arc::new(RwLock::new(HashMap::new()));
        let usage_stats = Arc::new(RwLock::new(HashMap::new()));

        let mut cache = Self {
            config: config.clone(),
            backend_config,
            model_manager,
            metrics,
            cached_models: cached_models.clone(),
            usage_stats: usage_stats.clone(),
            alias_map: Arc::new(RwLock::new(HashMap::new())),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            evictions: Arc::new(AtomicU64::new(0)),
            warmups: Arc::new(AtomicU64::new(0)),
            total_memory: Arc::new(AtomicU64::new(0)),
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
        // Resolve the caller's spelling to the canonical cache key before
        // deciding hit vs. miss, so every spelling of the same file collapses
        // to one entry. Cheap on repeat spellings (served from the alias map);
        // one resolve on first sight of a new spelling. A spelling that maps to
        // no file is a miss that then fails to load - count it before returning
        // so miss statistics stay accurate.
        let key = match self.resolve_cache_key(model_name).await {
            Ok(key) => key,
            Err(e) => {
                self.cache_misses.fetch_add(1, Ordering::Relaxed);
                return Err(e);
            }
        };

        // Try to get from cache first
        {
            let cache_guard = self.cached_models.read().await;
            if let Some(cached_model) = cache_guard.get(&key) {
                // Update last used time
                // Can't modify Arc contents directly, so we'll track usage separately
                cached_model.usage_count.fetch_add(1, Ordering::Relaxed);

                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                let cached_model = cached_model.clone();
                drop(cache_guard);
                self.update_usage_stats(&key, Duration::from_millis(0))
                    .await;

                debug!("Cache hit for model: {} (key {})", model_name, key);
                return Ok(cached_model);
            }
        }

        // Cache miss - load the model
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
        info!("Cache miss for model: {}, loading...", model_name);

        // Acquire semaphore to limit concurrent loads
        let _permit = self
            .loading_semaphore
            .acquire()
            .await
            .map_err(|_| anyhow!("Failed to acquire loading permit"))?;

        let start_time = Instant::now();
        let cached_model = self.load_model(model_name).await?;
        let load_time = start_time.elapsed();

        info!("Model {} loaded in {:?}", model_name, load_time);
        self.update_usage_stats(&key, load_time).await;

        // Check if we need to evict models
        self.maybe_evict_models().await?;

        Ok(cached_model)
    }

    /// Resolve a caller-supplied model name or path to the canonical cache key:
    /// the canonicalized absolute path of the model file. All spellings of the
    /// same file (bare name, name+extension, relative path, `./` prefix,
    /// symlink) collapse to one key, so a model occupies exactly one cache
    /// entry - fixing the double-load where e.g. `llama` and `llama.gguf` were
    /// cached separately. Repeat lookups of a spelling are served from the
    /// alias map without touching disk.
    async fn resolve_cache_key(&self, model_name: &str) -> Result<String> {
        if let Some(key) = self.alias_map.read().await.get(model_name) {
            return Ok(key.clone());
        }
        let model_info = self.model_manager.resolve_model(model_name).await?;
        let key = canonical_key(&model_info.path);
        self.alias_map
            .write()
            .await
            .insert(model_name.to_string(), key.clone());
        Ok(key)
    }

    /// Canonical cache keys of the configured always-warm models. Resident
    /// models are keyed by canonical path, but `always_warm` holds caller
    /// spellings, so the spellings must be resolved to the same keys before
    /// eviction protection can match them. Unresolvable entries (missing files)
    /// are simply absent.
    async fn always_warm_keys(&self) -> std::collections::HashSet<String> {
        let mut keys = std::collections::HashSet::new();
        for name in &self.config.always_warm {
            if let Ok(key) = self.resolve_cache_key(name).await {
                keys.insert(key);
            }
        }
        keys
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

        let total_requests =
            self.cache_hits.load(Ordering::Relaxed) + self.cache_misses.load(Ordering::Relaxed);
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
            // Report the resolved model name (filename) rather than the
            // canonical-path cache key, so callers see a stable, friendly
            // identity that is the same no matter which spelling loaded it.
            active_models: cached_models
                .values()
                .map(|m| m.model_info.name.clone())
                .collect(),
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
        // Resolve to the canonical key so callers can evict by any spelling
        // (e.g. `inferno cache clear <name>`), falling back to the raw string
        // if the file no longer resolves.
        let key = self
            .resolve_cache_key(model_name)
            .await
            .unwrap_or_else(|_| model_name.to_string());
        let mut cached_models = self.cached_models.write().await;
        if let Some(model) = cached_models.remove(&key) {
            self.total_memory
                .fetch_sub(model.memory_estimate, Ordering::Relaxed);
            self.evictions.fetch_add(1, Ordering::Relaxed);
            info!("Evicted model: {}", model_name);
        }
        Ok(())
    }

    /// Clear all cached models
    pub async fn clear_cache(&self) -> Result<()> {
        let mut cached_models = self.cached_models.write().await;
        cached_models.clear();
        self.alias_map.write().await.clear();
        self.total_memory.store(0, Ordering::Relaxed);
        info!("Cleared all cached models");
        Ok(())
    }

    /// Manually trigger cache persistence to disk
    pub async fn save_cache(&self) -> Result<()> {
        self.save_to_disk().await
    }

    /// Load a model and cache it
    async fn load_model(&self, model_name: &str) -> Result<Arc<CachedModel>> {
        let model_info = self.model_manager.resolve_model(model_name).await?;
        // The canonical key is the single identity every spelling maps to, so a
        // model loads once regardless of how it was named. Record the alias so
        // this spelling skips the resolve next time.
        let key = canonical_key(&model_info.path);
        self.alias_map
            .write()
            .await
            .insert(model_name.to_string(), key.clone());
        let backend_type = BackendType::from_model_path(&model_info.path).ok_or_else(|| {
            anyhow::anyhow!(
                "No suitable backend found for model: {}",
                model_info.path.display()
            )
        })?;

        let backend_handle = BackendHandle::new_shared(backend_type, &self.backend_config)?;
        backend_handle.load_model(&model_info).await?;

        let memory_estimate = self.estimate_model_memory(&model_info);
        let cached_model = Arc::new(CachedModel {
            backend: backend_handle,
            model_info: model_info.clone(),
            last_used: Instant::now(),
            created_at: Instant::now(),
            usage_count: AtomicU64::new(0),
            memory_estimate,
            warmup_priority: self.calculate_warmup_priority(model_name).await,
        });

        // Add to cache and keep Arc reference for return
        let cached_model_ref = Arc::clone(&cached_model);
        {
            let mut cached_models = self.cached_models.write().await;
            cached_models.insert(key.clone(), cached_model);
            self.total_memory
                .fetch_add(memory_estimate, Ordering::Relaxed);
        }

        // Initialize usage stats if not exists
        {
            let mut usage_stats = self.usage_stats.write().await;
            usage_stats
                .entry(key.clone())
                .or_insert_with(|| ModelUsageStats {
                    model_name: key.clone(),
                    request_count: 0,
                    last_request: SystemTime::now(),
                    average_response_time: Duration::ZERO,
                    total_response_time: Duration::ZERO,
                    memory_usage: memory_estimate,
                    usage_frequency: 0.0,
                    usage_trend: 0.0,
                });
        }

        Ok(cached_model_ref)
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
            let window_start =
                SystemTime::now() - Duration::from_secs(self.config.usage_window_seconds);
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

        let should_evict = total_models > self.config.max_cached_models
            || current_memory > self.config.max_memory_mb * 1024 * 1024;

        if should_evict {
            self.evict_least_recently_used().await?;
        }

        Ok(())
    }

    /// Evict the least recently used model
    async fn evict_least_recently_used(&self) -> Result<()> {
        // Resolve always-warm spellings to canonical keys up front so resident
        // (canonically-keyed) models are matched and protected.
        let protected = self.always_warm_keys().await;
        let cached_models = self.cached_models.read().await;

        // Find the model with the oldest last_used time and lowest usage
        let mut oldest_time = Instant::now();
        let mut victim_model = None;
        let mut lowest_priority = u8::MAX;

        for (name, model) in cached_models.iter() {
            // Skip always-warm models
            if protected.contains(name) {
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
        let cleanup_total_memory =
            Arc::new(AtomicU64::new(self.total_memory.load(Ordering::Relaxed)));
        // Canonical keys of always-warm models, resolved once: the TTL sweep
        // keys on canonical paths but `always_warm` holds caller spellings.
        let cleanup_always_warm_keys = self.always_warm_keys().await;

        self.cleanup_task = Some(tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(300)); // 5 minutes

            loop {
                cleanup_interval.tick().await;

                let mut cached_models = cleanup_cached_models.write().await;
                let now = Instant::now();
                let ttl = Duration::from_secs(cleanup_config.model_ttl_seconds);

                let mut to_remove = Vec::new();
                for (name, model) in cached_models.iter() {
                    if now.duration_since(model.last_used) > ttl
                        && !cleanup_always_warm_keys.contains(name)
                    {
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

        // Periodic save task (if persistence is enabled)
        if self.config.persist_cache {
            let save_cache_dir = self.config.cache_dir.clone();
            let save_cached_models = self.cached_models.clone();
            let save_usage_stats = self.usage_stats.clone();
            // Share the counters rather than their current values: a periodic
            // save has to report the counts as of the save, not as of startup.
            let save_cache_hits = self.cache_hits.clone();
            let save_cache_misses = self.cache_misses.clone();
            let save_evictions = self.evictions.clone();
            let save_warmups = self.warmups.clone();
            let save_total_memory = self.total_memory.clone();
            let save_interval_seconds = self.config.persist_interval_seconds;

            self.stats_task = Some(tokio::spawn(async move {
                let mut save_interval = interval(Duration::from_secs(save_interval_seconds));

                loop {
                    save_interval.tick().await;

                    if let Some(cache_dir) = &save_cache_dir {
                        // Create a temporary cache state for saving
                        let cached_models = save_cached_models.read().await;
                        let usage_stats = save_usage_stats.read().await;

                        let mut cache_entries = Vec::new();
                        let now_timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        for (model_name, cached_model) in cached_models.iter() {
                            cache_entries.push(SerializableCacheEntry {
                                model_name: model_name.clone(),
                                model_info: cached_model.model_info.clone(),
                                last_used_timestamp: now_timestamp, // Approximation
                                created_at_timestamp: now_timestamp, // Approximation
                                usage_count: cached_model.usage_count.load(Ordering::Relaxed),
                                memory_estimate: cached_model.memory_estimate,
                                warmup_priority: cached_model.warmup_priority,
                            });
                        }

                        let cache_state = SerializableCacheState {
                            version: CACHE_FORMAT_VERSION,
                            cache_entries,
                            usage_stats: usage_stats.clone(),
                            cache_hits: save_cache_hits.load(Ordering::Relaxed),
                            cache_misses: save_cache_misses.load(Ordering::Relaxed),
                            evictions: save_evictions.load(Ordering::Relaxed),
                            warmups: save_warmups.load(Ordering::Relaxed),
                            total_memory: save_total_memory.load(Ordering::Relaxed),
                            saved_at: now_timestamp,
                        };

                        drop(cached_models);
                        drop(usage_stats);

                        // Save to disk
                        if let Err(e) = async_fs::create_dir_all(cache_dir).await {
                            warn!("Failed to create cache directory {:?}: {}", cache_dir, e);
                            continue;
                        }

                        let cache_file = cache_dir.join(CACHE_FILE_NAME);
                        match save_cache_state_to_file_static(&cache_state, &cache_file).await {
                            Ok(()) => {
                                debug!(
                                    "Periodic cache save completed with {} entries",
                                    cache_state.cache_entries.len()
                                );
                            }
                            Err(e) => {
                                warn!("Periodic cache save failed: {}", e);
                            }
                        }
                    }
                }
            }));
        }

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
        candidates.sort_by(|a, b| {
            b.usage_frequency
                .partial_cmp(&a.usage_frequency)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Warm up top candidates that aren't already cached
        for stats in candidates.iter().take(3) {
            // Warm up top 3
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

        candidates.sort_by(|a, b| {
            b.usage_trend
                .partial_cmp(&a.usage_trend)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for stats in candidates.iter().take(2) {
            if let Err(e) = self.warmup_model(&stats.model_name).await {
                warn!(
                    "Failed to predictively warm up model {}: {}",
                    stats.model_name, e
                );
            }
        }

        Ok(())
    }

    /// Warm up models optimized by size
    async fn warmup_size_optimized(&self) -> Result<()> {
        // Get all available models and sort by size
        if let Ok(models) = self.model_manager.list_models().await {
            let mut sorted_models = models;
            sorted_models.sort_by_key(|a| a.size);

            // Warm up smaller models first
            for model in sorted_models.iter().take(3) {
                if let Err(e) = self.warmup_model(&model.name).await {
                    warn!(
                        "Failed to warm up size-optimized model {}: {}",
                        model.name, e
                    );
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
            info!("Loading cache state from disk: {:?}", cache_dir);

            // Ensure cache directory exists
            if !cache_dir.exists() {
                debug!(
                    "Cache directory does not exist, skipping load: {:?}",
                    cache_dir
                );
                return Ok(());
            }

            let cache_file = cache_dir.join(CACHE_FILE_NAME);
            if !cache_file.exists() {
                debug!("Cache file does not exist, skipping load: {:?}", cache_file);
                return Ok(());
            }

            match self.load_cache_state_from_file(&cache_file).await {
                Ok(cache_state) => {
                    info!(
                        "Successfully loaded cache state with {} entries",
                        cache_state.cache_entries.len()
                    );

                    // Restore usage statistics
                    {
                        let mut usage_stats = self.usage_stats.write().await;
                        *usage_stats = cache_state.usage_stats;
                    }

                    // Restore cache statistics
                    self.cache_hits
                        .store(cache_state.cache_hits, Ordering::Relaxed);
                    self.cache_misses
                        .store(cache_state.cache_misses, Ordering::Relaxed);
                    self.evictions
                        .store(cache_state.evictions, Ordering::Relaxed);
                    self.warmups.store(cache_state.warmups, Ordering::Relaxed);
                    self.total_memory
                        .store(cache_state.total_memory, Ordering::Relaxed);

                    // Warm up models that were previously cached if they're still available
                    for entry in cache_state.cache_entries {
                        if self.should_restore_model(&entry).await {
                            if let Err(e) = self.warmup_model(&entry.model_name).await {
                                warn!(
                                    "Failed to restore model from cache: {}: {}",
                                    entry.model_name, e
                                );
                            } else {
                                debug!("Restored cached model: {}", entry.model_name);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to load cache state from disk: {}", e);
                    // Continue without cached state
                }
            }
        }
        Ok(())
    }

    /// Save cache state to disk
    async fn save_to_disk(&self) -> Result<()> {
        if let Some(cache_dir) = &self.config.cache_dir {
            info!("Saving cache state to disk: {:?}", cache_dir);

            // Ensure cache directory exists
            if let Err(e) = async_fs::create_dir_all(cache_dir).await {
                return Err(anyhow!(
                    "Failed to create cache directory {:?}: {}",
                    cache_dir,
                    e
                ));
            }

            // Collect current cache state
            let cache_state = self.collect_cache_state().await;

            let cache_file = cache_dir.join(CACHE_FILE_NAME);
            match self
                .save_cache_state_to_file(&cache_state, &cache_file)
                .await
            {
                Ok(()) => {
                    info!(
                        "Successfully saved cache state with {} entries to {:?}",
                        cache_state.cache_entries.len(),
                        cache_file
                    );
                }
                Err(e) => {
                    error!("Failed to save cache state to disk: {}", e);
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// Load cache state from a specific file
    async fn load_cache_state_from_file(
        &self,
        file_path: &PathBuf,
    ) -> Result<SerializableCacheState> {
        let compressed_data = async_fs::read(file_path)
            .await
            .map_err(|e| anyhow!("Failed to read cache file {:?}: {}", file_path, e))?;

        // Decompress the data
        let decompressed_data = zstd::decode_all(&compressed_data[..])
            .map_err(|e| anyhow!("Failed to decompress cache data: {}", e))?;

        // Deserialize the data
        let cache_state: SerializableCacheState = bincode::deserialize(&decompressed_data)
            .map_err(|e| anyhow!("Failed to deserialize cache data: {}", e))?;

        // Validate version compatibility
        if cache_state.version != CACHE_FORMAT_VERSION {
            return Err(anyhow!(
                "Incompatible cache format version: {} (expected {})",
                cache_state.version,
                CACHE_FORMAT_VERSION
            ));
        }

        Ok(cache_state)
    }

    /// Save cache state to a specific file
    async fn save_cache_state_to_file(
        &self,
        cache_state: &SerializableCacheState,
        file_path: &PathBuf,
    ) -> Result<()> {
        // Serialize the data
        let serialized_data = bincode::serialize(cache_state)
            .map_err(|e| anyhow!("Failed to serialize cache data: {}", e))?;

        // Compress the data
        let compressed_data =
            zstd::encode_all(&serialized_data[..], 3) // Compression level 3 for good balance
                .map_err(|e| anyhow!("Failed to compress cache data: {}", e))?;

        // Write to temporary file first, then atomically rename
        let temp_file = temp_save_path(file_path);
        async_fs::write(&temp_file, &compressed_data)
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to write temporary cache file {:?}: {}",
                    temp_file,
                    e
                )
            })?;

        async_fs::rename(&temp_file, file_path).await.map_err(|e| {
            anyhow!(
                "Failed to rename cache file {:?} to {:?}: {}",
                temp_file,
                file_path,
                e
            )
        })?;

        Ok(())
    }

    /// Collect current cache state for serialization
    async fn collect_cache_state(&self) -> SerializableCacheState {
        let cached_models = self.cached_models.read().await;
        let usage_stats = self.usage_stats.read().await;

        let mut cache_entries = Vec::new();

        for (model_name, cached_model) in cached_models.iter() {
            // Convert Instant to Unix timestamp for serialization
            let last_used_timestamp = self.instant_to_unix_timestamp(cached_model.last_used);
            let created_at_timestamp = self.instant_to_unix_timestamp(cached_model.created_at);

            cache_entries.push(SerializableCacheEntry {
                model_name: model_name.clone(),
                model_info: cached_model.model_info.clone(),
                last_used_timestamp,
                created_at_timestamp,
                usage_count: cached_model.usage_count.load(Ordering::Relaxed),
                memory_estimate: cached_model.memory_estimate,
                warmup_priority: cached_model.warmup_priority,
            });
        }

        SerializableCacheState {
            version: CACHE_FORMAT_VERSION,
            cache_entries,
            usage_stats: usage_stats.clone(),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            warmups: self.warmups.load(Ordering::Relaxed),
            total_memory: self.total_memory.load(Ordering::Relaxed),
            saved_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Determine if a model should be restored from cache
    async fn should_restore_model(&self, entry: &SerializableCacheEntry) -> bool {
        // Check if the model file still exists and hasn't changed
        if !entry.model_info.path.exists() {
            debug!(
                "Model file no longer exists, skipping restore: {:?}",
                entry.model_info.path
            );
            return false;
        }

        // Check if model is still valid
        if let Ok(current_model_info) = self.model_manager.resolve_model(&entry.model_name).await {
            // Compare file size as a simple integrity check
            if current_model_info.size != entry.model_info.size {
                debug!(
                    "Model file size changed, skipping restore: {}",
                    entry.model_name
                );
                return false;
            }
        } else {
            debug!(
                "Failed to get current model info, skipping restore: {}",
                entry.model_name
            );
            return false;
        }

        // Check if model was recently used (within 24 hours)
        let last_used_time =
            SystemTime::UNIX_EPOCH + Duration::from_secs(entry.last_used_timestamp);
        let time_since_last_use = SystemTime::now()
            .duration_since(last_used_time)
            .unwrap_or(Duration::from_secs(u64::MAX));

        if time_since_last_use > Duration::from_secs(86400) {
            // 24 hours
            debug!(
                "Model not used recently, skipping restore: {}",
                entry.model_name
            );
            return false;
        }

        true
    }

    /// Convert Instant to Unix timestamp (best effort)
    fn instant_to_unix_timestamp(&self, _instant: Instant) -> u64 {
        // This is an approximation since Instant is relative to program start
        // We use SystemTime for the actual timestamp
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Static function to save cache state (used by background task)
async fn save_cache_state_to_file_static(
    cache_state: &SerializableCacheState,
    file_path: &PathBuf,
) -> Result<()> {
    // Serialize the data
    let serialized_data = bincode::serialize(cache_state)
        .map_err(|e| anyhow!("Failed to serialize cache data: {}", e))?;

    // Compress the data
    let compressed_data = zstd::encode_all(&serialized_data[..], 3) // Compression level 3 for good balance
        .map_err(|e| anyhow!("Failed to compress cache data: {}", e))?;

    // Write to temporary file first, then atomically rename
    let temp_file = temp_save_path(file_path);
    async_fs::write(&temp_file, &compressed_data)
        .await
        .map_err(|e| {
            anyhow!(
                "Failed to write temporary cache file {:?}: {}",
                temp_file,
                e
            )
        })?;

    async_fs::rename(&temp_file, file_path).await.map_err(|e| {
        anyhow!(
            "Failed to rename cache file {:?} to {:?}: {}",
            temp_file,
            file_path,
            e
        )
    })?;

    Ok(())
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

        // Save cache state to disk on shutdown if persistence is enabled.
        let Some(cache_dir) = self.config.cache_dir.clone() else {
            return;
        };
        if !self.config.persist_cache {
            return;
        }

        // Drop is synchronous, but blocking on a new runtime panics when this
        // thread is already driving one - and a panic in Drop aborts the
        // process. Building the runtime succeeds inside an async context, so
        // it cannot be used to detect one; ask the runtime directly.
        //
        // The state is behind shared handles, so a save can run without `self`
        // either way.
        let cache_state = SaveHandles {
            cached_models: self.cached_models.clone(),
            usage_stats: self.usage_stats.clone(),
            cache_hits: self.cache_hits.clone(),
            cache_misses: self.cache_misses.clone(),
            evictions: self.evictions.clone(),
            warmups: self.warmups.clone(),
            total_memory: self.total_memory.clone(),
        };

        match tokio::runtime::Handle::try_current() {
            // Already inside a runtime: hand the save to it. Callers that need
            // the save to be durable should use `save_cache` before dropping.
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(e) = cache_state.save(&cache_dir).await {
                        error!("Failed to save cache state on shutdown: {}", e);
                    }
                });
            }
            // No runtime driving this thread, so blocking is safe.
            Err(_) => match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    if let Err(e) = rt.block_on(cache_state.save(&cache_dir)) {
                        error!("Failed to save cache state on shutdown: {}", e);
                    }
                }
                Err(e) => error!("Failed to save cache state on shutdown: {}", e),
            },
        }
    }
}

/// The shared state a save needs, detached from the cache itself so it can
/// outlive `Drop`.
struct SaveHandles {
    cached_models: Arc<RwLock<HashMap<String, Arc<CachedModel>>>>,
    usage_stats: Arc<RwLock<HashMap<String, ModelUsageStats>>>,
    cache_hits: Arc<AtomicU64>,
    cache_misses: Arc<AtomicU64>,
    evictions: Arc<AtomicU64>,
    warmups: Arc<AtomicU64>,
    total_memory: Arc<AtomicU64>,
}

impl SaveHandles {
    async fn save(&self, cache_dir: &std::path::Path) -> Result<()> {
        async_fs::create_dir_all(cache_dir)
            .await
            .map_err(|e| anyhow!("Failed to create cache directory {:?}: {}", cache_dir, e))?;

        let now_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let cache_entries = {
            let cached_models = self.cached_models.read().await;
            cached_models
                .iter()
                .map(|(model_name, cached_model)| SerializableCacheEntry {
                    model_name: model_name.clone(),
                    model_info: cached_model.model_info.clone(),
                    last_used_timestamp: now_timestamp,
                    created_at_timestamp: now_timestamp,
                    usage_count: cached_model.usage_count.load(Ordering::Relaxed),
                    memory_estimate: cached_model.memory_estimate,
                    warmup_priority: cached_model.warmup_priority,
                })
                .collect()
        };

        let cache_state = SerializableCacheState {
            version: CACHE_FORMAT_VERSION,
            cache_entries,
            usage_stats: self.usage_stats.read().await.clone(),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            warmups: self.warmups.load(Ordering::Relaxed),
            total_memory: self.total_memory.load(Ordering::Relaxed),
            saved_at: now_timestamp,
        };

        save_cache_state_to_file_static(&cache_state, &cache_dir.join(CACHE_FILE_NAME)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    async fn cache_over(models_dir: &Path) -> ModelCache {
        let config = CacheConfig {
            persist_cache: false,
            enable_warmup: false,
            ..CacheConfig::default()
        };
        let model_manager = Arc::new(ModelManager::new(models_dir));
        ModelCache::new(config, BackendConfig::default(), model_manager, None)
            .await
            .expect("cache construction")
    }

    /// `canonical_key` must collapse `./`-prefixed and relative spellings of a
    /// file to the same absolute string - the class of aliasing CLAUDE.md flags
    /// for cache keys. If canonicalization were dropped, the raw strings differ
    /// and this fails.
    #[test]
    fn canonical_key_collapses_relative_and_dotslash() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("m.gguf");
        fs::write(&file, b"gguf-stub").unwrap();
        // `canonicalize` requires every path component to exist, so the `..`
        // spelling needs a real intermediate directory to traverse.
        fs::create_dir(dir.path().join("sub")).unwrap();

        let abs = canonical_key(&file);
        let dotslash = canonical_key(&dir.path().join("./m.gguf"));
        let indirect = canonical_key(&dir.path().join("sub").join("..").join("m.gguf"));

        assert_eq!(abs, dotslash, "`./` prefix must resolve to the same key");
        assert_eq!(abs, indirect, "`..` segment must resolve to the same key");
    }

    /// The headline bug: `model` and `model.gguf` (and a path spelling) all name
    /// one file but were cached under separate keys, loading it twice. Every
    /// spelling must now resolve to a single canonical cache key.
    #[tokio::test]
    async fn spellings_of_one_model_share_a_cache_key() {
        let dir = TempDir::new().unwrap();
        let models_dir = dir.path().join("models");
        fs::create_dir_all(&models_dir).unwrap();
        let file = models_dir.join("dedup_model.gguf");
        fs::write(&file, b"gguf-stub").unwrap();

        let cache = cache_over(&models_dir).await;

        let by_name = cache.resolve_cache_key("dedup_model").await.unwrap();
        let by_ext = cache.resolve_cache_key("dedup_model.gguf").await.unwrap();
        let by_abs = cache
            .resolve_cache_key(&file.to_string_lossy())
            .await
            .unwrap();

        assert_eq!(by_name, by_ext, "bare name and name+ext must share a key");
        assert_eq!(by_name, by_abs, "name and absolute path must share a key");
        assert_eq!(by_abs, canonical_key(&file), "key is the canonical path");
    }
}
