use anyhow::Result;
use inferno::{
    backends::BackendConfig,
    cache::{
        CacheConfig, ModelCache, ModelUsageStats, SerializableCacheEntry, SerializableCacheState,
        WarmupStrategy,
    },
    models::{ModelInfo, ModelManager},
};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tempfile::TempDir;
use tokio::time::sleep;

/// Test utilities for cache persistence integration tests
mod cache_test_utils {
    use super::*;

    /// Names the cache tests expect to find in the models directory. The
    /// `always_warm` entry below refers to the first of these.
    pub const TEST_MODEL_NAMES: [&str; 3] = ["test_model_1", "test_model_2", "test_model_3"];

    pub fn create_test_cache_config(cache_dir: Option<PathBuf>) -> CacheConfig {
        CacheConfig {
            max_cached_models: 3,
            max_memory_mb: 1024,
            model_ttl_seconds: 300, // 5 minutes
            // Warmup loads models of its own accord, which would show up in
            // every test's cache contents and counts. The test that cares about
            // warmup turns it on.
            enable_warmup: false,
            warmup_strategy: WarmupStrategy::UsageBased,
            always_warm: Vec::new(),
            predictive_loading: true,
            usage_window_seconds: 3600, // 1 hour
            min_usage_frequency: 0.1,
            memory_based_eviction: true,
            persist_cache: cache_dir.is_some(),
            cache_dir,
            persist_interval_seconds: 300,
        }
    }

    /// A real GGUF file to populate the cache with, taken from
    /// `INFERNO_TEST_MODEL`.
    ///
    /// Populating the cache means loading through the genuine llama.cpp loader,
    /// which rejects the synthetic fixture below - that file carries a
    /// plausible header, not a model. So any test that caches a model needs a
    /// real one from the environment, and skips without it.
    pub fn real_model_path() -> Option<PathBuf> {
        let value = std::env::var_os("INFERNO_TEST_MODEL")?;
        let path = PathBuf::from(value);
        // A typo'd path must fail loudly rather than skip silently - a skip
        // here would look identical to "not configured".
        assert!(
            path.is_file(),
            "INFERNO_TEST_MODEL is set to {}, which is not a file",
            path.display()
        );
        Some(path)
    }

    /// Fill `models_dir` with real, loadable models under the names the cache
    /// tests use, or report that none is configured.
    pub fn require_real_models(models_dir: &Path) -> Option<Vec<PathBuf>> {
        let Some(source) = real_model_path() else {
            eprintln!("SKIP: set INFERNO_TEST_MODEL to a real GGUF file to run this test");
            return None;
        };

        let mut paths = Vec::new();
        for name in TEST_MODEL_NAMES {
            let dest = models_dir.join(format!("{}.gguf", name));
            // Hard-link so each name is a real file without copying the model
            // once per name; fall back to a copy across filesystems.
            if fs::hard_link(&source, &dest).is_err() {
                fs::copy(&source, &dest).expect("should provision test model");
            }
            paths.push(dest);
        }
        Some(paths)
    }

    pub fn create_test_backend_config() -> BackendConfig {
        BackendConfig {
            gpu_enabled: false,
            gpu_device: None,
            cpu_threads: Some(2),
            context_size: 512,
            batch_size: 8,
            memory_map: true,
        }
    }

    /// A `ModelInfo` standing in for a discovered model in serialized state.
    /// Nothing reads this file - it only has to round-trip.
    pub fn test_model_info(name: &str) -> ModelInfo {
        let path = PathBuf::from(format!("/test/{}.gguf", name));
        ModelInfo {
            name: name.to_string(),
            file_path: path.clone(),
            path,
            size: 2048,
            size_bytes: 2048,
            modified: chrono::Utc::now(),
            backend_type: "gguf".to_string(),
            format: "gguf".to_string(),
            checksum: None,
            metadata: HashMap::new(),
        }
    }

    /// Total requests the cache actually served.
    ///
    /// `CacheStats` reports hit/miss as rates, and `miss_rate` is `1.0 -
    /// hit_rate`, so an untouched cache reports a miss rate of 1.0 - asserting
    /// on it would pass without the cache doing anything. The per-model request
    /// counts are the honest source.
    pub fn total_requests(stats: &inferno::cache::CacheStats) -> u64 {
        stats.model_stats.values().map(|s| s.request_count).sum()
    }

    /// Read a persisted cache file, mirroring how the cache writes it
    /// (bincode, then zstd).
    pub fn read_cache_state(cache_file: &Path) -> Result<SerializableCacheState> {
        let compressed = fs::read(cache_file)?;
        let decompressed = zstd::decode_all(&compressed[..])?;
        Ok(bincode::deserialize(&decompressed)?)
    }

    pub async fn wait_for_cache_persistence(
        cache_dir: &Path,
        timeout_duration: Duration,
    ) -> Result<()> {
        let cache_file = cache_dir.join("cache_state.bin.zst");
        let start = std::time::Instant::now();

        while start.elapsed() < timeout_duration {
            if cache_file.exists() {
                return Ok(());
            }
            sleep(Duration::from_millis(100)).await;
        }

        anyhow::bail!("Cache persistence file not found within timeout")
    }

    pub fn create_mock_serializable_cache_state() -> SerializableCacheState {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let cache_entries = vec![
            SerializableCacheEntry {
                model_name: "test_model_1".to_string(),
                model_info: test_model_info("test_model_1"),
                last_used_timestamp: now,
                created_at_timestamp: now - 300,
                usage_count: 5,
                memory_estimate: 512,
                warmup_priority: 1,
            },
            SerializableCacheEntry {
                model_name: "test_model_2".to_string(),
                model_info: test_model_info("test_model_2"),
                last_used_timestamp: now - 60,
                created_at_timestamp: now - 600,
                usage_count: 3,
                memory_estimate: 256,
                warmup_priority: 2,
            },
        ];

        let mut usage_stats = HashMap::new();
        usage_stats.insert(
            "test_model_1".to_string(),
            ModelUsageStats {
                model_name: "test_model_1".to_string(),
                request_count: 5,
                last_request: SystemTime::now(),
                average_response_time: Duration::from_millis(100),
                total_response_time: Duration::from_millis(500),
                memory_usage: 512,
                usage_frequency: 5.0,
                usage_trend: 1.2,
            },
        );

        SerializableCacheState {
            version: 1,
            cache_entries,
            usage_stats,
            cache_hits: 10,
            cache_misses: 3,
            evictions: 1,
            warmups: 2,
            total_memory: 768,
            saved_at: now,
        }
    }
}

/// Test basic cache persistence functionality
#[tokio::test]
async fn test_cache_persistence_basic() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");

    fs::create_dir_all(&models_dir)?;
    fs::create_dir_all(&cache_dir)?;

    if cache_test_utils::require_real_models(&models_dir).is_none() {
        return Ok(());
    }

    let mut cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir.clone()));

    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(&models_dir));

    // Discover models
    let models = model_manager.list_models().await?;
    assert!(!models.is_empty(), "Should discover test models");

    // Create cache with persistence enabled
    let cache = ModelCache::new(
        cache_config,
        backend_config.clone(),
        model_manager.clone(),
        None,
    )
    .await?;

    // Load some models to populate cache
    for model in &models[0..2] {
        cache.get_model(&model.name).await?;
    }

    // Save explicitly rather than waiting on the background interval, so the
    // file under test is the one holding these models.
    cache.save_cache().await?;

    let cache_file = cache_dir.join("cache_state.bin.zst");
    assert!(cache_file.exists(), "Cache state file should exist");

    // The file existing is not enough: the background task writes an empty
    // state as soon as the cache starts, so assert the loaded models actually
    // reached the file.
    let state = cache_test_utils::read_cache_state(&cache_file)?;
    assert_eq!(
        state.cache_entries.len(),
        2,
        "Both loaded models should be persisted, got {:?}",
        state
            .cache_entries
            .iter()
            .map(|e| &e.model_name)
            .collect::<Vec<_>>()
    );

    Ok(())
}

/// The background save must report the statistics as of the save, not as of
/// startup. It runs on its own task, so it can only see the counters if it
/// shares them - holding copies of their startup values would pin every
/// persisted count to zero for the life of the process.
///
/// No model is needed: a cache miss is counted before the load is attempted,
/// so asking for a model that does not exist moves the counter on its own.
#[tokio::test]
async fn test_periodic_save_persists_live_statistics() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");

    fs::create_dir_all(&models_dir)?;
    fs::create_dir_all(&cache_dir)?;

    let mut cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir.clone()));
    // Tick often enough to observe a save.
    cache_config.persist_interval_seconds = 1;

    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(&models_dir));
    let cache = ModelCache::new(cache_config, backend_config, model_manager, None).await?;

    // Counted as a miss before the load fails, which is all this needs.
    let missed = cache.get_model("no-such-model").await;
    assert!(missed.is_err(), "a model that does not exist cannot load");

    let cache_file = cache_dir.join("cache_state.bin.zst");
    // Long enough for a tick after the miss, short enough to fail fast.
    cache_test_utils::wait_for_cache_persistence(&cache_dir, Duration::from_secs(5)).await?;

    // The first tick fires immediately at startup, before the miss above, so
    // poll until a save reflects it rather than trusting the first file.
    let mut persisted_misses = 0;
    for _ in 0..30 {
        persisted_misses = cache_test_utils::read_cache_state(&cache_file)?.cache_misses;
        if persisted_misses > 0 {
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }

    assert_eq!(
        persisted_misses, 1,
        "the periodic save should report the miss that happened after startup"
    );

    Ok(())
}

/// Test cache persistence across restarts
#[tokio::test]
async fn test_cache_persistence_across_restarts() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");

    fs::create_dir_all(&models_dir)?;
    fs::create_dir_all(&cache_dir)?;

    if cache_test_utils::require_real_models(&models_dir).is_none() {
        return Ok(());
    }

    let cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir.clone()));
    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(&models_dir));

    let models = model_manager.list_models().await?;
    let first_model = models[0].name.clone();

    // First cache instance - populate cache
    {
        let cache1 = ModelCache::new(
            cache_config.clone(),
            backend_config.clone(),
            model_manager.clone(),
            None,
        )
        .await?;

        for model in &models[0..2] {
            cache1.get_model(&model.name).await?;
        }

        // Every load was a miss, so the cache did real work.
        assert!(
            cache_test_utils::total_requests(&cache1.get_stats().await) > 0,
            "Loading models should register cache activity"
        );

        cache1.save_cache().await?;
        drop(cache1);
    }

    // Second cache instance - should restore the persisted entries
    {
        let cache2 =
            ModelCache::new(cache_config, backend_config.clone(), model_manager, None).await?;

        // Asking for a restored model must be served from the cache rather
        // than loaded again, which is what persistence is for.
        cache2.get_model(&first_model).await?;

        let stats = cache2.get_stats().await;
        assert!(
            stats.hit_rate > 0.0,
            "A restored model should be served as a cache hit, got hit_rate {}",
            stats.hit_rate
        );
    }

    Ok(())
}

/// Test cache compression and decompression
#[tokio::test]
async fn test_cache_compression() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&cache_dir)?;

    // Create a mock cache state
    let cache_state = cache_test_utils::create_mock_serializable_cache_state();

    // Serialize and compress
    let serialized = bincode::serialize(&cache_state)?;
    let compressed = zstd::bulk::compress(&serialized, 1)?;

    // Write to file
    let cache_file = cache_dir.join("cache_state.bin.zst");
    fs::write(&cache_file, &compressed)?;

    // Read and decompress
    let read_compressed = fs::read(&cache_file)?;
    let decompressed = zstd::bulk::decompress(&read_compressed, serialized.len() * 2)?;
    let deserialized: SerializableCacheState = bincode::deserialize(&decompressed)?;

    // Verify data integrity
    assert_eq!(cache_state.version, deserialized.version);
    assert_eq!(
        cache_state.cache_entries.len(),
        deserialized.cache_entries.len()
    );
    assert_eq!(cache_state.cache_hits, deserialized.cache_hits);
    assert_eq!(cache_state.cache_misses, deserialized.cache_misses);

    // Verify compression efficiency
    assert!(
        compressed.len() < serialized.len(),
        "Compression should reduce size"
    );

    Ok(())
}

/// Test cache eviction with persistence
#[tokio::test]
async fn test_cache_eviction_with_persistence() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");

    fs::create_dir_all(&models_dir)?;
    fs::create_dir_all(&cache_dir)?;

    if cache_test_utils::require_real_models(&models_dir).is_none() {
        return Ok(());
    }

    let mut cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir.clone()));
    cache_config.max_cached_models = 2; // Small cache to force eviction

    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(&models_dir));

    let models = model_manager.list_models().await?;
    assert!(
        models.len() > 2,
        "This test needs more models than the cache can hold"
    );

    let cache = ModelCache::new(cache_config, backend_config.clone(), model_manager, None).await?;

    // Load more models than the cache holds, forcing eviction.
    for model in &models {
        cache.get_model(&model.name).await?;
    }

    let stats = cache.get_stats().await;

    // Assert the limit is enforced, not just that it happens to hold. Testing
    // `total_models <= 2` alone would pass even if nothing were ever cached.
    assert!(
        stats.total_models <= 2,
        "Cache should hold at most max_cached_models, got {}",
        stats.total_models
    );
    assert!(
        stats.total_models > 0,
        "Cache should still hold the most recent models"
    );
    assert!(
        stats.eviction_count > 0,
        "Loading {} models into a cache of 2 must evict",
        models.len()
    );

    // The persisted state must reflect the eviction, not the pre-eviction set.
    cache.save_cache().await?;
    let state = cache_test_utils::read_cache_state(&cache_dir.join("cache_state.bin.zst"))?;
    assert!(
        state.cache_entries.len() <= 2,
        "Persisted state should reflect evictions, got {} entries",
        state.cache_entries.len()
    );

    Ok(())
}

/// Test cache statistics persistence
#[tokio::test]
async fn test_cache_statistics_persistence() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");

    fs::create_dir_all(&models_dir)?;
    fs::create_dir_all(&cache_dir)?;

    if cache_test_utils::require_real_models(&models_dir).is_none() {
        return Ok(());
    }

    let cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir.clone()));
    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(&models_dir));

    let models = model_manager.list_models().await?;

    let cache = ModelCache::new(
        cache_config,
        backend_config.clone(),
        model_manager.clone(),
        None,
    )
    .await?;

    // Generate cache activity: the first pass over each model misses, the
    // later passes hit.
    for _ in 0..3 {
        for model in &models[0..2] {
            cache.get_model(&model.name).await?;
        }
    }

    let initial_stats = cache.get_stats().await;
    assert!(
        initial_stats.hit_rate > 0.0,
        "Repeated access should produce cache hits"
    );
    let requests_before = cache_test_utils::total_requests(&initial_stats);
    assert_eq!(
        requests_before, 6,
        "Six accesses should be counted, got {}",
        requests_before
    );

    cache.save_cache().await?;
    drop(cache);

    // A restarted cache must recover the counts, not start from zero.
    let cache2 = ModelCache::new(
        cache_test_utils::create_test_cache_config(Some(cache_dir)),
        backend_config,
        model_manager,
        None,
    )
    .await?;

    let restored_stats = cache2.get_stats().await;
    assert_eq!(
        cache_test_utils::total_requests(&restored_stats),
        requests_before,
        "Restored cache should recover the persisted request counts"
    );

    Ok(())
}

/// Test concurrent cache operations with persistence
#[tokio::test]
async fn test_concurrent_cache_with_persistence() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");

    fs::create_dir_all(&models_dir)?;
    fs::create_dir_all(&cache_dir)?;

    if cache_test_utils::require_real_models(&models_dir).is_none() {
        return Ok(());
    }

    let cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir.clone()));
    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(&models_dir));

    let models = model_manager.list_models().await?;

    let cache = Arc::new(ModelCache::new(cache_config, backend_config, model_manager, None).await?);

    // Launch concurrent cache operations
    let mut tasks = Vec::new();

    for i in 0..5 {
        let cache_clone = cache.clone();
        let names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();

        let task = tokio::spawn(async move {
            for j in 0..3 {
                let name = &names[(i + j) % names.len()];
                cache_clone
                    .get_model(name)
                    .await
                    .expect("concurrent cache access should succeed");
            }
        });

        tasks.push(task);
    }

    // A panic inside a task must fail the test rather than be swallowed.
    for task in tasks {
        task.await?;
    }

    // All 15 accesses must be accounted for, and the cache must not have
    // exceeded its limit under concurrency.
    let stats = cache.get_stats().await;
    assert_eq!(
        cache_test_utils::total_requests(&stats),
        15,
        "Every concurrent access should be counted exactly once"
    );
    assert!(
        stats.total_models <= 3,
        "Concurrent loads should still respect max_cached_models, got {}",
        stats.total_models
    );

    Ok(())
}

/// Test cache corruption recovery
#[tokio::test]
async fn test_cache_corruption_recovery() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");

    fs::create_dir_all(&models_dir)?;
    fs::create_dir_all(&cache_dir)?;

    // A corrupt cache file must not stop the cache from starting. This needs
    // no model: starting up is the whole behaviour under test.
    let cache_file = cache_dir.join("cache_state.bin.zst");
    fs::write(&cache_file, b"corrupted data")?;

    let mut cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir));

    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(&models_dir));

    let cache = ModelCache::new(cache_config, backend_config, model_manager, None)
        .await
        .expect("cache should start despite a corrupt state file");

    // Recovery means starting empty, not carrying over garbage.
    let stats = cache.get_stats().await;
    assert_eq!(
        stats.total_models, 0,
        "A corrupt state file should yield an empty cache"
    );

    Ok(())
}

/// Test cache memory management with persistence
#[tokio::test]
async fn test_cache_memory_management() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");

    fs::create_dir_all(&models_dir)?;
    fs::create_dir_all(&cache_dir)?;

    if cache_test_utils::require_real_models(&models_dir).is_none() {
        return Ok(());
    }

    let mut cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir));
    cache_config.max_memory_mb = 256; // Very low memory limit
    cache_config.memory_based_eviction = true;

    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(&models_dir));

    let models = model_manager.list_models().await?;

    let cache = ModelCache::new(cache_config, backend_config, model_manager, None).await?;

    for model in &models {
        cache.get_model(&model.name).await?;
    }

    let final_stats = cache.get_stats().await;

    // Caching a model has to account for its memory. Asserting `>= 0.0` on an
    // unsigned-derived figure would hold even if nothing were tracked at all.
    assert!(
        final_stats.memory_usage_mb > 0.0,
        "Cached models should contribute tracked memory"
    );

    // The limit is the point: either usage stays under it, or eviction brought
    // it back under.
    assert!(
        final_stats.memory_usage_mb <= 256.0,
        "Memory-based eviction should keep usage within max_memory_mb, got {} MB",
        final_stats.memory_usage_mb
    );

    Ok(())
}

/// Test cache warmup with persistence
#[tokio::test]
async fn test_cache_warmup_with_persistence() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");

    fs::create_dir_all(&models_dir)?;
    fs::create_dir_all(&cache_dir)?;

    if cache_test_utils::require_real_models(&models_dir).is_none() {
        return Ok(());
    }

    let mut cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir));
    cache_config.enable_warmup = true;
    cache_config.always_warm = vec!["test_model_1".to_string()];

    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(&models_dir));

    // Always-warm models are loaded during construction.
    let cache = ModelCache::new(cache_config, backend_config, model_manager, None).await?;

    let stats = cache.get_stats().await;

    // The always-warm model must actually be resident - the original test only
    // printed when it was, so it passed whether warmup worked or not.
    // `active_models` now reports the resolved model name (the filename), which
    // is the same regardless of the spelling that loaded it, so the bare
    // `always_warm` string "test_model_1" surfaces as "test_model_1.gguf".
    assert!(
        stats.active_models.iter().any(|m| m == "test_model_1.gguf"),
        "always_warm model should be cached after startup, got {:?}",
        stats.active_models
    );
    assert!(
        stats.warmup_count > 0,
        "Warming a model should be counted as a warmup"
    );

    Ok(())
}
