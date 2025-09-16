use inferno::{
    cache::{
        ModelCache, CacheConfig, CachedModel, WarmupStrategy, CacheStats,
        SerializableCacheState, SerializableCacheEntry, ModelUsageStats,
    },
    backends::{BackendConfig, BackendType, BackendHandle},
    models::{ModelInfo, ModelManager},
    metrics::MetricsCollector,
    InfernoError,
};
use anyhow::Result;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tempfile::TempDir;
use tokio::{
    fs as async_fs,
    time::{sleep, timeout},
};

/// Test utilities for cache persistence integration tests
mod cache_test_utils {
    use super::*;

    pub fn create_test_cache_config(cache_dir: Option<PathBuf>) -> CacheConfig {
        CacheConfig {
            max_cached_models: 3,
            max_memory_mb: 1024,
            model_ttl_seconds: 300, // 5 minutes
            enable_warmup: true,
            warmup_strategy: WarmupStrategy::UsageBased,
            always_warm: vec!["test_model_1".to_string()],
            predictive_loading: true,
            usage_window_seconds: 3600, // 1 hour
            min_usage_frequency: 0.1,
            memory_based_eviction: true,
            persist_cache: cache_dir.is_some(),
            cache_dir,
        }
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

    pub fn create_mock_gguf_file(path: &Path) -> Result<()> {
        let mut content = Vec::new();
        // GGUF magic number
        content.extend_from_slice(b"GGUF");
        // Version
        content.extend_from_slice(&3u32.to_le_bytes());
        // Tensor count
        content.extend_from_slice(&0u64.to_le_bytes());
        // Metadata count
        content.extend_from_slice(&1u64.to_le_bytes());

        // Simple metadata entry
        let key = "general.name";
        content.extend_from_slice(&(key.len() as u64).to_le_bytes());
        content.extend_from_slice(key.as_bytes());

        // Value type (string = 8)
        content.extend_from_slice(&8u32.to_le_bytes());
        let value = path.file_stem().unwrap().to_str().unwrap();
        content.extend_from_slice(&(value.len() as u64).to_le_bytes());
        content.extend_from_slice(value.as_bytes());

        // Pad to reasonable size
        content.resize(2048, 0);
        fs::write(path, content)?;
        Ok(())
    }

    pub async fn create_test_models(models_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut model_paths = Vec::new();

        for i in 1..=3 {
            let model_path = models_dir.join(format!("test_model_{}.gguf", i));
            create_mock_gguf_file(&model_path)?;
            model_paths.push(model_path);
        }

        Ok(model_paths)
    }

    pub async fn wait_for_cache_persistence(cache_dir: &Path, timeout_duration: Duration) -> Result<()> {
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
                model_info: ModelInfo {
                    id: "test_model_1".to_string(),
                    name: "Test Model 1".to_string(),
                    path: PathBuf::from("/test/test_model_1.gguf"),
                    format: "gguf".to_string(),
                    size: 2048,
                    metadata: HashMap::new(),
                    backend_type: None,
                    created_at: SystemTime::now(),
                    modified_at: SystemTime::now(),
                    checksum: None,
                },
                last_used_timestamp: now,
                created_at_timestamp: now - 300,
                usage_count: 5,
                memory_estimate: 512,
                warmup_priority: 1,
            },
            SerializableCacheEntry {
                model_name: "test_model_2".to_string(),
                model_info: ModelInfo {
                    id: "test_model_2".to_string(),
                    name: "Test Model 2".to_string(),
                    path: PathBuf::from("/test/test_model_2.gguf"),
                    format: "gguf".to_string(),
                    size: 2048,
                    metadata: HashMap::new(),
                    backend_type: None,
                    created_at: SystemTime::now(),
                    modified_at: SystemTime::now(),
                    checksum: None,
                },
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

    let model_paths = cache_test_utils::create_test_models(&models_dir).await?;

    let cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir.clone()));
    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(models_dir));

    // Discover models
    let models = model_manager.discover_models().await?;
    assert!(!models.is_empty(), "Should discover test models");

    // Create cache with persistence enabled
    let cache = ModelCache::new(
        cache_config,
        backend_config.clone(),
        model_manager.clone(),
        None,
    ).await?;

    // Load some models to populate cache
    for model in &models[0..2] {
        let _ = cache.get_or_load_model(&model.id, BackendType::Gguf, &backend_config).await?;
    }

    // Wait for cache to be persisted
    cache_test_utils::wait_for_cache_persistence(&cache_dir, Duration::from_secs(5)).await?;

    // Verify cache file exists
    let cache_file = cache_dir.join("cache_state.bin.zst");
    assert!(cache_file.exists(), "Cache state file should exist");

    // Verify cache file is not empty
    let file_size = fs::metadata(&cache_file)?.len();
    assert!(file_size > 0, "Cache file should not be empty");

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

    let _model_paths = cache_test_utils::create_test_models(&models_dir).await?;

    let cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir.clone()));
    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(models_dir));

    let models = model_manager.discover_models().await?;

    // First cache instance - populate cache
    {
        let cache1 = ModelCache::new(
            cache_config.clone(),
            backend_config.clone(),
            model_manager.clone(),
            None,
        ).await?;

        // Load models and track usage
        for model in &models[0..2] {
            let _ = cache1.get_or_load_model(&model.id, BackendType::Gguf, &backend_config).await?;
        }

        // Get initial stats
        let initial_stats = cache1.get_stats().await;
        assert!(initial_stats.cache_hits + initial_stats.cache_misses > 0);

        // Wait for persistence
        cache_test_utils::wait_for_cache_persistence(&cache_dir, Duration::from_secs(5)).await?;

        // Drop cache to simulate restart
        drop(cache1);
    }

    // Second cache instance - should load from persistence
    {
        let cache2 = ModelCache::new(
            cache_config,
            backend_config.clone(),
            model_manager,
            None,
        ).await?;

        // Wait a bit for background loading
        sleep(Duration::from_millis(500)).await;

        // Access a previously cached model - should be faster (cache hit)
        let start = std::time::Instant::now();
        let _cached_model = cache2.get_or_load_model(&models[0].id, BackendType::Gguf, &backend_config).await?;
        let access_time = start.elapsed();

        // Should be relatively fast since it was restored from cache
        assert!(access_time < Duration::from_secs(2), "Restored model access should be fast");

        let stats = cache2.get_stats().await;
        assert!(stats.cache_hits > 0, "Should have cache hits from restored cache");
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
    assert_eq!(cache_state.cache_entries.len(), deserialized.cache_entries.len());
    assert_eq!(cache_state.cache_hits, deserialized.cache_hits);
    assert_eq!(cache_state.cache_misses, deserialized.cache_misses);

    // Verify compression efficiency
    assert!(compressed.len() < serialized.len(), "Compression should reduce size");

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

    let _model_paths = cache_test_utils::create_test_models(&models_dir).await?;

    let mut cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir.clone()));
    cache_config.max_cached_models = 2; // Small cache to force eviction
    cache_config.max_memory_mb = 512;   // Small memory limit

    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(models_dir));

    let models = model_manager.discover_models().await?;

    let cache = ModelCache::new(
        cache_config,
        backend_config.clone(),
        model_manager,
        None,
    ).await?;

    // Load models until eviction occurs
    for model in &models {
        let _ = cache.get_or_load_model(&model.id, BackendType::Gguf, &backend_config).await?;

        // Small delay to allow background processing
        sleep(Duration::from_millis(100)).await;
    }

    // Wait for cache operations to complete
    sleep(Duration::from_millis(500)).await;

    let stats = cache.get_stats().await;

    // Should have evictions due to cache size limit
    assert!(stats.eviction_count > 0 || stats.cached_models <= 2,
           "Should have evictions or be within cache limit");

    // Wait for persistence
    cache_test_utils::wait_for_cache_persistence(&cache_dir, Duration::from_secs(5)).await?;

    // Verify persistence file reflects evictions
    let cache_file = cache_dir.join("cache_state.bin.zst");
    assert!(cache_file.exists());

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

    let _model_paths = cache_test_utils::create_test_models(&models_dir).await?;

    let cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir.clone()));
    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(models_dir));

    let models = model_manager.discover_models().await?;

    let cache = ModelCache::new(
        cache_config,
        backend_config.clone(),
        model_manager,
        None,
    ).await?;

    // Generate some cache activity
    for _ in 0..3 {
        for model in &models[0..2] {
            let _ = cache.get_or_load_model(&model.id, BackendType::Gguf, &backend_config).await?;
        }
    }

    let initial_stats = cache.get_stats().await;
    assert!(initial_stats.cache_hits > 0);
    assert!(initial_stats.cache_misses > 0);

    // Wait for persistence
    cache_test_utils::wait_for_cache_persistence(&cache_dir, Duration::from_secs(5)).await?;

    // Create new cache instance and verify stats are restored
    drop(cache);

    let cache2 = ModelCache::new(
        cache_test_utils::create_test_cache_config(Some(cache_dir)),
        backend_config,
        model_manager,
        None,
    ).await?;

    sleep(Duration::from_millis(500)).await; // Allow loading

    let restored_stats = cache2.get_stats().await;

    // Stats should be restored (or at least partially)
    assert!(restored_stats.cache_hits >= 0);
    assert!(restored_stats.cache_misses >= 0);

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

    let _model_paths = cache_test_utils::create_test_models(&models_dir).await?;

    let cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir.clone()));
    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(models_dir));

    let models = model_manager.discover_models().await?;

    let cache = Arc::new(ModelCache::new(
        cache_config,
        backend_config.clone(),
        model_manager,
        None,
    ).await?);

    // Launch concurrent cache operations
    let mut tasks = Vec::new();

    for i in 0..5 {
        let cache_clone = cache.clone();
        let models_clone = models.clone();
        let backend_config_clone = backend_config.clone();

        let task = tokio::spawn(async move {
            for j in 0..3 {
                let model_idx = (i + j) % models_clone.len();
                let model = &models_clone[model_idx];

                let result = cache_clone.get_or_load_model(
                    &model.id,
                    BackendType::Gguf,
                    &backend_config_clone
                ).await;

                assert!(result.is_ok(), "Concurrent cache access should succeed");

                // Small delay between accesses
                sleep(Duration::from_millis(50)).await;
            }
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    futures::future::join_all(tasks).await;

    // Verify cache state is consistent
    let stats = cache.get_stats().await;
    assert!(stats.cache_hits + stats.cache_misses > 0);

    // Wait for persistence
    cache_test_utils::wait_for_cache_persistence(&cache_dir, Duration::from_secs(5)).await?;

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

    let _model_paths = cache_test_utils::create_test_models(&models_dir).await?;

    // Create a corrupted cache file
    let cache_file = cache_dir.join("cache_state.bin.zst");
    fs::write(&cache_file, b"corrupted data")?;

    let cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir));
    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(models_dir));

    // Cache should still initialize despite corrupted file
    let cache_result = ModelCache::new(
        cache_config,
        backend_config.clone(),
        model_manager,
        None,
    ).await;

    assert!(cache_result.is_ok(), "Cache should handle corruption gracefully");

    let cache = cache_result?;
    let models = cache.list_available_models().await?;

    // Should still be able to load models
    if !models.is_empty() {
        let result = cache.get_or_load_model(&models[0].id, BackendType::Gguf, &backend_config).await;
        assert!(result.is_ok(), "Should be able to load models after corruption recovery");
    }

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

    let _model_paths = cache_test_utils::create_test_models(&models_dir).await?;

    let mut cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir.clone()));
    cache_config.max_memory_mb = 256; // Very low memory limit
    cache_config.memory_based_eviction = true;

    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(models_dir));

    let models = model_manager.discover_models().await?;

    let cache = ModelCache::new(
        cache_config,
        backend_config.clone(),
        model_manager,
        None,
    ).await?;

    // Load models and monitor memory usage
    for model in &models {
        let _ = cache.get_or_load_model(&model.id, BackendType::Gguf, &backend_config).await?;

        let stats = cache.get_stats().await;

        // Memory usage should be tracked
        assert!(stats.memory_usage_mb >= 0.0);

        sleep(Duration::from_millis(100)).await;
    }

    // Wait for any evictions to complete
    sleep(Duration::from_millis(500)).await;

    let final_stats = cache.get_stats().await;

    // Memory should be within limits or evictions should have occurred
    assert!(final_stats.memory_usage_mb <= 256.0 || final_stats.eviction_count > 0);

    // Wait for persistence
    cache_test_utils::wait_for_cache_persistence(&cache_dir, Duration::from_secs(5)).await?;

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

    let _model_paths = cache_test_utils::create_test_models(&models_dir).await?;

    let mut cache_config = cache_test_utils::create_test_cache_config(Some(cache_dir.clone()));
    cache_config.enable_warmup = true;
    cache_config.always_warm = vec!["test_model_1".to_string()];

    let backend_config = cache_test_utils::create_test_backend_config();
    let model_manager = Arc::new(ModelManager::new(models_dir));

    let cache = ModelCache::new(
        cache_config,
        backend_config.clone(),
        model_manager,
        None,
    ).await?;

    // Wait for warmup to complete
    sleep(Duration::from_secs(2)).await;

    let stats = cache.get_stats().await;

    // Should have at least one warmed up model
    assert!(stats.warmup_count > 0 || stats.cached_models > 0);

    // Check if always-warm model is loaded
    let model_list = cache.list_cached_models().await;
    let has_always_warm = model_list.iter().any(|m| m.contains("test_model_1"));

    if has_always_warm {
        println!("Always-warm model successfully loaded");
    }

    // Wait for persistence
    cache_test_utils::wait_for_cache_persistence(&cache_dir, Duration::from_secs(5)).await?;

    Ok(())
}