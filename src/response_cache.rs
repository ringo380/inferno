#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    clippy::inherent_to_string
)]
use crate::metrics::MetricsCollector;
use anyhow::{Context, Result};
use blake3;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{
    sync::{Mutex, RwLock},
    time::interval,
};
use tracing::{debug, info, warn};
use zstd;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCacheConfig {
    pub enabled: bool,
    pub max_entries: usize,
    pub max_memory_mb: u64,
    pub ttl_seconds: u64,
    pub deduplication_enabled: bool,
    pub compression_enabled: bool,
    pub compression_algorithm: CompressionAlgorithm,
    pub compression_level: u32,
    pub compression_threshold_bytes: usize,
    pub hash_algorithm: HashAlgorithm,
    pub cache_strategy: CacheStrategy,
    pub eviction_policy: EvictionPolicy,
}

impl Default for ResponseCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 10000,
            max_memory_mb: 1024,
            ttl_seconds: 3600,
            deduplication_enabled: true,
            compression_enabled: true,
            compression_algorithm: CompressionAlgorithm::Gzip,
            compression_level: 6,              // Balanced compression level
            compression_threshold_bytes: 1024, // Only compress if >= 1KB
            hash_algorithm: HashAlgorithm::Sha256,
            cache_strategy: CacheStrategy::Smart,
            eviction_policy: EvictionPolicy::LeastRecentlyUsed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    Gzip,
    Zstd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
    Xxhash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheStrategy {
    None,
    Basic,
    Smart,
    Aggressive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    LeastRecentlyUsed,
    LeastFrequentlyUsed,
    TimeToLive,
    Random,
    FirstInFirstOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheKey {
    pub request_hash: String,
    pub model_id: String,
    pub parameters_hash: String,
}

impl CacheKey {
    pub fn new(
        request_text: &str,
        model_id: &str,
        parameters: &str,
        algorithm: &HashAlgorithm,
    ) -> Self {
        let request_hash = Self::compute_hash(request_text, algorithm);
        let parameters_hash = Self::compute_hash(parameters, algorithm);

        Self {
            request_hash,
            model_id: model_id.to_string(),
            parameters_hash,
        }
    }

    fn compute_hash(input: &str, algorithm: &HashAlgorithm) -> String {
        match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(input.as_bytes());
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::Blake3 => {
                let hash = blake3::hash(input.as_bytes());
                hash.to_hex().to_string()
            }
            HashAlgorithm::Xxhash => {
                let hash = xxhash_rust::xxh3::xxh3_64(input.as_bytes());
                format!("{:016x}", hash)
            }
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.model_id, self.request_hash, self.parameters_hash
        )
    }
}

#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub response_data: Vec<u8>,
    pub metadata: ResponseMetadata,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub access_count: u64,
    pub size_bytes: usize,
    pub compressed: bool,
    pub original_size_bytes: Option<usize>,
    pub compression_algorithm: Option<CompressionAlgorithm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub model_id: String,
    pub response_type: String,
    pub token_count: Option<u32>,
    pub processing_time_ms: u64,
    pub quality_score: Option<f32>,
    pub content_type: String,
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hit_rate: f32,
    pub total_entries: usize,
    pub memory_usage_bytes: usize,
    pub memory_usage_mb: f32,
    pub deduplication_savings: u64,
    pub compression_ratio: f32,
    pub evictions: u64,
    pub expired_entries: u64,
}

pub struct ResponseCache {
    config: ResponseCacheConfig,
    cache: Arc<RwLock<HashMap<String, Arc<CachedResponse>>>>,
    deduplication_map: Arc<RwLock<HashMap<String, String>>>,
    stats: Arc<Mutex<CacheStats>>,
    metrics: Option<Arc<MetricsCollector>>,
    background_cleanup_handle: Option<tokio::task::JoinHandle<()>>,
}

impl ResponseCache {
    pub async fn new(
        config: ResponseCacheConfig,
        metrics: Option<Arc<MetricsCollector>>,
    ) -> Result<Self> {
        let cache = Arc::new(RwLock::new(HashMap::new()));
        let deduplication_map = Arc::new(RwLock::new(HashMap::new()));
        let stats = Arc::new(Mutex::new(CacheStats {
            total_requests: 0,
            cache_hits: 0,
            cache_misses: 0,
            hit_rate: 0.0,
            total_entries: 0,
            memory_usage_bytes: 0,
            memory_usage_mb: 0.0,
            deduplication_savings: 0,
            compression_ratio: 1.0,
            evictions: 0,
            expired_entries: 0,
        }));

        let mut response_cache = Self {
            config,
            cache,
            deduplication_map,
            stats,
            metrics,
            background_cleanup_handle: None,
        };

        if response_cache.config.enabled {
            response_cache.start_background_cleanup().await;
        }

        Ok(response_cache)
    }

    pub async fn get(&self, key: &CacheKey) -> Option<Vec<u8>> {
        if !self.config.enabled {
            return None;
        }

        let cache_key = key.to_string();
        let mut stats = self.stats.lock().await;
        stats.total_requests += 1;

        // Check deduplication map first
        let actual_key = if self.config.deduplication_enabled {
            let dedup_map = self.deduplication_map.read().await;
            dedup_map.get(&cache_key).unwrap_or(&cache_key).to_owned()
        } else {
            cache_key
        };

        let cache = self.cache.read().await;
        if let Some(cached_response) = cache.get(&actual_key) {
            // Check if entry has expired
            if self.is_expired(cached_response) {
                drop(cache);
                drop(stats);
                self.remove_expired_entry(&actual_key).await;
                return None;
            }

            // Update access statistics
            stats.cache_hits += 1;
            stats.hit_rate = stats.cache_hits as f32 / stats.total_requests as f32;
            drop(stats);

            // Update last accessed time and access count
            self.update_access_stats(&actual_key).await;

            let response_data = if cached_response.compressed {
                match self.decompress_data(
                    &cached_response.response_data,
                    &cached_response.compression_algorithm,
                ) {
                    Ok(decompressed) => decompressed,
                    Err(e) => {
                        warn!(
                            "Decompression failed for key {}: {}, removing entry",
                            actual_key, e
                        );
                        drop(cache);
                        self.remove_expired_entry(&actual_key).await;
                        return None;
                    }
                }
            } else {
                cached_response.response_data.clone()
            };

            debug!("Cache hit for key: {}", actual_key);
            return Some(response_data);
        }

        stats.cache_misses += 1;
        stats.hit_rate = stats.cache_hits as f32 / stats.total_requests as f32;
        debug!("Cache miss for key: {:?}", key);

        None
    }

    pub async fn put(
        &self,
        key: &CacheKey,
        response_data: Vec<u8>,
        metadata: ResponseMetadata,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let cache_key = key.to_string();

        // Check if we should apply deduplication
        let (actual_key, is_duplicate) = if self.config.deduplication_enabled {
            self.check_deduplication(&cache_key, &response_data).await
        } else {
            (cache_key.clone(), false)
        };

        if is_duplicate {
            let mut stats = self.stats.lock().await;
            stats.deduplication_savings += response_data.len() as u64;
            debug!("Deduplication: redirecting {} to {}", cache_key, actual_key);

            // Add to deduplication map
            let mut dedup_map = self.deduplication_map.write().await;
            dedup_map.insert(cache_key, actual_key);
            return Ok(());
        }

        // Compress data if enabled and above threshold
        let (final_data, compressed, original_size, compression_algo) =
            if self.config.compression_enabled
                && response_data.len() >= self.config.compression_threshold_bytes
            {
                match self.compress_data(&response_data) {
                    Ok(compressed_data) => {
                        let compression_ratio =
                            compressed_data.len() as f32 / response_data.len() as f32;
                        // Only use compression if it actually reduces size by at least 10%
                        if compression_ratio < 0.9 {
                            (
                                compressed_data,
                                true,
                                Some(response_data.len()),
                                Some(self.config.compression_algorithm.clone()),
                            )
                        } else {
                            (response_data.clone(), false, None, None)
                        }
                    }
                    Err(e) => {
                        warn!("Compression failed: {}, storing uncompressed", e);
                        (response_data.clone(), false, None, None)
                    }
                }
            } else {
                (response_data.clone(), false, None, None)
            };

        let cached_response = Arc::new(CachedResponse {
            response_data: final_data.clone(),
            metadata,
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            access_count: 1,
            size_bytes: final_data.len(),
            compressed,
            original_size_bytes: original_size,
            compression_algorithm: compression_algo,
        });

        // Check memory limits before inserting
        self.ensure_memory_limits(&cached_response).await?;

        let mut cache = self.cache.write().await;
        cache.insert(actual_key, cached_response);

        self.update_stats().await;

        debug!(
            "Cached response for key: {} (compressed: {}, size: {} bytes)",
            cache_key,
            compressed,
            final_data.len()
        );

        Ok(())
    }

    pub async fn invalidate(&self, pattern: &str) -> Result<usize> {
        let mut cache = self.cache.write().await;
        let mut dedup_map = self.deduplication_map.write().await;

        let keys_to_remove: Vec<String> = cache
            .keys()
            .filter(|key| key.contains(pattern))
            .cloned()
            .collect();

        let removed_count = keys_to_remove.len();

        for key in &keys_to_remove {
            cache.remove(key);
        }

        // Also remove from deduplication map
        let dedup_keys_to_remove: Vec<String> = dedup_map
            .iter()
            .filter(|(k, v)| k.contains(pattern) || v.contains(pattern))
            .map(|(k, _)| k.clone())
            .collect();

        for key in &dedup_keys_to_remove {
            dedup_map.remove(key);
        }

        self.update_stats().await;

        info!(
            "Invalidated {} cache entries matching pattern: {}",
            removed_count, pattern
        );
        Ok(removed_count)
    }

    pub async fn clear(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        let mut dedup_map = self.deduplication_map.write().await;

        cache.clear();
        dedup_map.clear();

        let mut stats = self.stats.lock().await;
        stats.total_entries = 0;
        stats.memory_usage_bytes = 0;
        stats.memory_usage_mb = 0.0;

        info!("Cache cleared");
        Ok(())
    }

    pub async fn get_stats(&self) -> CacheStats {
        self.stats.lock().await.clone()
    }

    async fn check_deduplication(&self, key: &str, data: &[u8]) -> (String, bool) {
        let content_hash = self.compute_content_hash(data);
        let cache = self.cache.read().await;

        // Look for existing entry with same content
        for (existing_key, cached_response) in cache.iter() {
            let existing_content_hash = self.compute_content_hash(&cached_response.response_data);
            if existing_content_hash == content_hash {
                return (existing_key.clone(), true);
            }
        }

        (key.to_string(), false)
    }

    fn compute_content_hash(&self, data: &[u8]) -> String {
        // Use Blake3 for content hashing as it's fast and cryptographically secure
        let hash = blake3::hash(data);
        hash.to_hex().to_string()
    }

    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        match self.config.compression_algorithm {
            CompressionAlgorithm::Gzip => {
                let mut encoder =
                    GzEncoder::new(Vec::new(), Compression::new(self.config.compression_level));
                encoder
                    .write_all(data)
                    .context("Failed to write data to gzip encoder")?;
                encoder
                    .finish()
                    .context("Failed to finalize gzip compression")
            }
            CompressionAlgorithm::Zstd => {
                zstd::encode_all(data, self.config.compression_level as i32)
                    .context("Failed to compress data with zstd")
            }
        }
    }

    fn decompress_data(
        &self,
        data: &[u8],
        algorithm: &Option<CompressionAlgorithm>,
    ) -> Result<Vec<u8>> {
        let algo = algorithm
            .as_ref()
            .unwrap_or(&self.config.compression_algorithm);

        match algo {
            CompressionAlgorithm::Gzip => {
                let mut decoder = GzDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder
                    .read_to_end(&mut decompressed)
                    .context("Failed to decompress gzip data")?;
                Ok(decompressed)
            }
            CompressionAlgorithm::Zstd => {
                zstd::decode_all(data).context("Failed to decompress zstd data")
            }
        }
    }

    fn is_expired(&self, cached_response: &CachedResponse) -> bool {
        let ttl = Duration::from_secs(self.config.ttl_seconds);
        cached_response
            .created_at
            .elapsed()
            .unwrap_or(Duration::ZERO)
            > ttl
    }

    async fn remove_expired_entry(&self, key: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(key);

        let mut stats = self.stats.lock().await;
        stats.expired_entries += 1;

        debug!("Removed expired cache entry: {}", key);
    }

    async fn update_access_stats(&self, key: &str) {
        let cache = self.cache.read().await;
        if let Some(_cached_response) = cache.get(key) {
            // Note: In a real implementation, we'd need to use interior mutability
            // or a different approach to update access stats
            debug!("Updated access stats for key: {}", key);
        }
    }

    async fn ensure_memory_limits(&self, new_entry: &CachedResponse) -> Result<()> {
        let cache = self.cache.read().await;
        let current_memory = self.calculate_memory_usage(&cache).await;
        let max_memory = (self.config.max_memory_mb * 1024 * 1024) as usize;

        if cache.len() >= self.config.max_entries
            || current_memory + new_entry.size_bytes > max_memory
        {
            drop(cache);
            self.evict_entries().await?;
        }

        Ok(())
    }

    async fn calculate_memory_usage(&self, cache: &HashMap<String, Arc<CachedResponse>>) -> usize {
        cache.values().map(|entry| entry.size_bytes).sum()
    }

    async fn evict_entries(&self) -> Result<()> {
        let mut cache = self.cache.write().await;

        let entries_to_evict = match self.config.eviction_policy {
            EvictionPolicy::LeastRecentlyUsed => {
                let mut entries: Vec<(String, SystemTime)> = cache
                    .iter()
                    .map(|(k, v)| (k.clone(), v.last_accessed))
                    .collect();
                entries.sort_by_key(|(_, time)| *time);
                entries
                    .into_iter()
                    .map(|(k, _)| k)
                    .take(cache.len() / 4)
                    .collect::<Vec<String>>()
            }
            EvictionPolicy::LeastFrequentlyUsed => {
                let mut entries: Vec<(String, u64)> = cache
                    .iter()
                    .map(|(k, v)| (k.clone(), v.access_count))
                    .collect();
                entries.sort_by_key(|(_, count)| *count);
                entries
                    .into_iter()
                    .map(|(k, _)| k)
                    .take(cache.len() / 4)
                    .collect::<Vec<String>>()
            }
            EvictionPolicy::TimeToLive => cache
                .iter()
                .filter(|(_, v)| self.is_expired(v))
                .map(|(k, _)| k.clone())
                .collect(),
            EvictionPolicy::Random => {
                use std::collections::hash_map::RandomState;
                use std::hash::{BuildHasher, Hasher};
                let hasher = RandomState::new();
                let mut entries: Vec<String> = cache.keys().cloned().collect();
                entries.sort_by_key(|k| {
                    let mut h = hasher.build_hasher();
                    h.write(k.as_bytes());
                    h.finish()
                });
                entries.into_iter().take(cache.len() / 4).collect()
            }
            EvictionPolicy::FirstInFirstOut => {
                let mut entries: Vec<(String, SystemTime)> = cache
                    .iter()
                    .map(|(k, v)| (k.clone(), v.created_at))
                    .collect();
                entries.sort_by_key(|(_, time)| *time);
                entries
                    .into_iter()
                    .map(|(k, _)| k)
                    .take(cache.len() / 4)
                    .collect::<Vec<String>>()
            }
        };

        let evicted_count = entries_to_evict.len();
        for key in entries_to_evict {
            cache.remove(&key);
        }

        let mut stats = self.stats.lock().await;
        stats.evictions += evicted_count as u64;

        info!(
            "Evicted {} cache entries using {:?} policy",
            evicted_count, self.config.eviction_policy
        );

        Ok(())
    }

    async fn update_stats(&self) {
        let cache = self.cache.read().await;
        let memory_usage = self.calculate_memory_usage(&cache).await;

        let mut stats = self.stats.lock().await;
        stats.total_entries = cache.len();
        stats.memory_usage_bytes = memory_usage;
        stats.memory_usage_mb = memory_usage as f32 / (1024.0 * 1024.0);

        // Calculate compression ratio using actual original sizes
        let total_compressed_size: usize = cache
            .values()
            .filter(|entry| entry.compressed)
            .map(|entry| entry.size_bytes)
            .sum();

        let total_original_size = cache
            .values()
            .map(|entry| {
                if entry.compressed {
                    entry.original_size_bytes.unwrap_or(entry.size_bytes)
                } else {
                    entry.size_bytes
                }
            })
            .sum::<usize>();

        if total_compressed_size > 0 && total_original_size > 0 {
            stats.compression_ratio = total_original_size as f32 / total_compressed_size as f32;
        } else {
            stats.compression_ratio = 1.0;
        }
    }

    async fn start_background_cleanup(&mut self) {
        let cache = Arc::clone(&self.cache);
        let dedup_map = Arc::clone(&self.deduplication_map);
        let stats = Arc::clone(&self.stats);
        let ttl_seconds = self.config.ttl_seconds;

        let handle = tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(300)); // Clean up every 5 minutes

            loop {
                cleanup_interval.tick().await;

                let mut cache_guard = cache.write().await;
                let mut dedup_guard = dedup_map.write().await;

                let expired_keys: Vec<String> = cache_guard
                    .iter()
                    .filter(|(_, entry)| {
                        let ttl = Duration::from_secs(ttl_seconds);
                        entry.created_at.elapsed().unwrap_or(Duration::ZERO) > ttl
                    })
                    .map(|(k, _)| k.clone())
                    .collect();

                let expired_count = expired_keys.len();
                for key in &expired_keys {
                    cache_guard.remove(key);
                }

                // Clean up deduplication map entries that point to expired cache entries
                let dedup_keys_to_remove: Vec<String> = dedup_guard
                    .iter()
                    .filter(|(_, v)| !cache_guard.contains_key(*v))
                    .map(|(k, _)| k.clone())
                    .collect();

                for key in &dedup_keys_to_remove {
                    dedup_guard.remove(key);
                }

                if expired_count > 0 {
                    let mut stats_guard = stats.lock().await;
                    stats_guard.expired_entries += expired_count as u64;
                    drop(stats_guard);

                    info!(
                        "Background cleanup removed {} expired cache entries",
                        expired_count
                    );
                }
            }
        });

        self.background_cleanup_handle = Some(handle);
    }

    pub async fn shutdown(&mut self) {
        if let Some(handle) = self.background_cleanup_handle.take() {
            handle.abort();
            info!("Response cache background cleanup task stopped");
        }
    }
}

impl Drop for ResponseCache {
    fn drop(&mut self) {
        if let Some(handle) = &self.background_cleanup_handle {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_hash_algorithms_basic() {
        let test_input = "Hello, World!";

        // Test SHA256
        let sha256_hash = CacheKey::compute_hash(test_input, &HashAlgorithm::Sha256);
        assert_eq!(sha256_hash.len(), 64); // SHA256 produces 64 hex characters
        assert!(!sha256_hash.is_empty());

        // Test Blake3
        let blake3_hash = CacheKey::compute_hash(test_input, &HashAlgorithm::Blake3);
        assert_eq!(blake3_hash.len(), 64); // Blake3 produces 64 hex characters
        assert!(!blake3_hash.is_empty());

        // Test xxHash
        let xxhash_hash = CacheKey::compute_hash(test_input, &HashAlgorithm::Xxhash);
        assert_eq!(xxhash_hash.len(), 16); // xxHash produces 16 hex characters (64-bit)
        assert!(!xxhash_hash.is_empty());

        // Ensure different algorithms produce different hashes
        assert_ne!(sha256_hash, blake3_hash);
        assert_ne!(sha256_hash, xxhash_hash);
        assert_ne!(blake3_hash, xxhash_hash);
    }

    #[test]
    fn test_hash_algorithms_reproducibility() {
        let test_input = "Test reproducibility";

        // Test that each algorithm produces consistent results
        for algorithm in &[
            HashAlgorithm::Sha256,
            HashAlgorithm::Blake3,
            HashAlgorithm::Xxhash,
        ] {
            let hash1 = CacheKey::compute_hash(test_input, algorithm);
            let hash2 = CacheKey::compute_hash(test_input, algorithm);
            assert_eq!(
                hash1, hash2,
                "Hash algorithm {:?} should be reproducible",
                algorithm
            );
        }
    }

    #[test]
    fn test_hash_algorithms_different_inputs() {
        let inputs = vec![
            "",
            "a",
            "Hello, World!",
            "A very long string that contains multiple words and should test the hash function with more data",
            "🦀 Rust with unicode characters 中文 🎉",
        ];

        for algorithm in &[
            HashAlgorithm::Sha256,
            HashAlgorithm::Blake3,
            HashAlgorithm::Xxhash,
        ] {
            let mut hashes = std::collections::HashSet::new();

            for input in &inputs {
                let hash = CacheKey::compute_hash(input, algorithm);

                // Ensure hash is not empty
                assert!(
                    !hash.is_empty(),
                    "Hash should not be empty for algorithm {:?}",
                    algorithm
                );

                // Ensure hash is unique for different inputs
                assert!(
                    hashes.insert(hash.clone()),
                    "Hash collision detected for algorithm {:?} with input: {}",
                    algorithm,
                    input
                );
            }
        }
    }

    #[test]
    fn test_cache_key_creation() {
        let request_text = "Translate 'hello' to French";
        let model_id = "gpt-3.5-turbo";
        let parameters = "temperature=0.7,max_tokens=100";

        for algorithm in &[
            HashAlgorithm::Sha256,
            HashAlgorithm::Blake3,
            HashAlgorithm::Xxhash,
        ] {
            let key = CacheKey::new(request_text, model_id, parameters, algorithm);

            assert_eq!(key.model_id, model_id);
            assert!(!key.request_hash.is_empty());
            assert!(!key.parameters_hash.is_empty());

            // Test key string representation
            let key_string = key.to_string();
            assert!(key_string.contains(model_id));
            assert!(key_string.contains(&key.request_hash));
            assert!(key_string.contains(&key.parameters_hash));
        }
    }

    #[test]
    fn test_content_hash_function() {
        // Mock a response cache instance to test content hashing
        let data1 = b"Hello, World!";
        let data2 = b"Hello, World!"; // Same content
        let data3 = b"Different content";

        // Create a temporary cache config for testing
        let config = ResponseCacheConfig::default();

        // We can't easily create a ResponseCache instance due to async requirements,
        // so we'll test the Blake3 hashing directly since that's what compute_content_hash uses
        let hash1 = blake3::hash(data1).to_hex().to_string();
        let hash2 = blake3::hash(data2).to_hex().to_string();
        let hash3 = blake3::hash(data3).to_hex().to_string();

        // Same content should produce same hash
        assert_eq!(hash1, hash2);
        // Different content should produce different hash
        assert_ne!(hash1, hash3);
        assert_ne!(hash2, hash3);
    }

    #[test]
    fn test_hash_performance_characteristics() {
        let test_data = "x".repeat(10000); // 10KB of data

        // Test that all hash functions can handle larger data
        for algorithm in &[
            HashAlgorithm::Sha256,
            HashAlgorithm::Blake3,
            HashAlgorithm::Xxhash,
        ] {
            let start = std::time::Instant::now();
            let hash = CacheKey::compute_hash(&test_data, algorithm);
            let duration = start.elapsed();

            assert!(!hash.is_empty());
            // Hash should complete in reasonable time (less than 1ms for 10KB)
            assert!(
                duration < std::time::Duration::from_millis(1),
                "Hash algorithm {:?} took too long: {:?}",
                algorithm,
                duration
            );
        }
    }

    #[test]
    fn test_hash_security_properties() {
        // Test that small changes in input produce significantly different hashes
        let base_input = "The quick brown fox jumps over the lazy dog";
        let modified_input = "The quick brown fox jumps over the lazy cat"; // Changed 'dog' to 'cat'

        for algorithm in &[
            HashAlgorithm::Sha256,
            HashAlgorithm::Blake3,
            HashAlgorithm::Xxhash,
        ] {
            let hash1 = CacheKey::compute_hash(base_input, algorithm);
            let hash2 = CacheKey::compute_hash(modified_input, algorithm);

            assert_ne!(
                hash1, hash2,
                "Small input changes should produce different hashes for {:?}",
                algorithm
            );

            // For cryptographic hashes, check avalanche effect (at least 25% of bits should change)
            if matches!(algorithm, HashAlgorithm::Sha256 | HashAlgorithm::Blake3) {
                let different_chars = hash1
                    .chars()
                    .zip(hash2.chars())
                    .filter(|(a, b)| a != b)
                    .count();
                let total_chars = hash1.len();
                let change_ratio = different_chars as f64 / total_chars as f64;

                assert!(change_ratio >= 0.25,
                    "Cryptographic hash {:?} should have good avalanche effect. Change ratio: {:.3}",
                    algorithm, change_ratio);
            }
        }
    }

    #[test]
    fn test_gzip_compression_decompression() {
        let config = ResponseCacheConfig {
            compression_algorithm: CompressionAlgorithm::Gzip,
            compression_level: 6,
            ..Default::default()
        };

        // Create a mock ResponseCache for testing compression
        let test_data =
            b"Hello, World! This is a test string that should compress well when repeated. "
                .repeat(50);

        // Test compression
        let compressed =
            compress_test_data(&test_data, &config).expect("Compression should succeed");
        assert!(
            compressed.len() < test_data.len(),
            "Compressed data should be smaller than original"
        );

        // Test decompression
        let decompressed = decompress_test_data(&compressed, &Some(CompressionAlgorithm::Gzip))
            .expect("Decompression should succeed");
        assert_eq!(
            decompressed, test_data,
            "Decompressed data should match original"
        );
    }

    #[test]
    fn test_zstd_compression_decompression() {
        let config = ResponseCacheConfig {
            compression_algorithm: CompressionAlgorithm::Zstd,
            compression_level: 3,
            ..Default::default()
        };

        let test_data = b"ZSTD compression test data. ".repeat(100);

        // Test compression
        let compressed =
            compress_test_data(&test_data, &config).expect("ZSTD compression should succeed");
        assert!(
            compressed.len() < test_data.len(),
            "ZSTD compressed data should be smaller than original"
        );

        // Test decompression
        let decompressed = decompress_test_data(&compressed, &Some(CompressionAlgorithm::Zstd))
            .expect("ZSTD decompression should succeed");
        assert_eq!(
            decompressed, test_data,
            "ZSTD decompressed data should match original"
        );
    }

    #[test]
    fn test_compression_algorithms_comparison() {
        let test_data = b"This is a test string for comparing compression algorithms. ".repeat(200);

        let gzip_config = ResponseCacheConfig {
            compression_algorithm: CompressionAlgorithm::Gzip,
            compression_level: 6,
            ..Default::default()
        };

        let zstd_config = ResponseCacheConfig {
            compression_algorithm: CompressionAlgorithm::Zstd,
            compression_level: 3,
            ..Default::default()
        };

        let gzip_compressed =
            compress_test_data(&test_data, &gzip_config).expect("Gzip compression should work");
        let zstd_compressed =
            compress_test_data(&test_data, &zstd_config).expect("Zstd compression should work");

        // Both should compress the data
        assert!(gzip_compressed.len() < test_data.len());
        assert!(zstd_compressed.len() < test_data.len());

        // Both should decompress correctly
        let gzip_decompressed =
            decompress_test_data(&gzip_compressed, &Some(CompressionAlgorithm::Gzip))
                .expect("Gzip decompression should work");
        let zstd_decompressed =
            decompress_test_data(&zstd_compressed, &Some(CompressionAlgorithm::Zstd))
                .expect("Zstd decompression should work");

        assert_eq!(gzip_decompressed, test_data);
        assert_eq!(zstd_decompressed, test_data);
    }

    #[test]
    fn test_compression_levels() {
        let test_data = b"Compression level test data. ".repeat(100);

        // Test different compression levels for Gzip
        for level in [1, 6, 9] {
            let config = ResponseCacheConfig {
                compression_algorithm: CompressionAlgorithm::Gzip,
                compression_level: level,
                ..Default::default()
            };

            let compressed = compress_test_data(&test_data, &config)
                .expect(&format!("Gzip compression level {} should work", level));
            let decompressed = decompress_test_data(&compressed, &Some(CompressionAlgorithm::Gzip))
                .expect(&format!("Gzip decompression level {} should work", level));

            assert_eq!(decompressed, test_data);
        }

        // Test different compression levels for Zstd
        for level in [1, 3, 19] {
            let config = ResponseCacheConfig {
                compression_algorithm: CompressionAlgorithm::Zstd,
                compression_level: level,
                ..Default::default()
            };

            let compressed = compress_test_data(&test_data, &config)
                .expect(&format!("Zstd compression level {} should work", level));
            let decompressed = decompress_test_data(&compressed, &Some(CompressionAlgorithm::Zstd))
                .expect(&format!("Zstd decompression level {} should work", level));

            assert_eq!(decompressed, test_data);
        }
    }

    #[test]
    fn test_compression_edge_cases() {
        let config = ResponseCacheConfig::default();

        // Test empty data
        let empty_data = b"";
        let compressed = compress_test_data(empty_data, &config).expect("Should handle empty data");
        let decompressed =
            decompress_test_data(&compressed, &Some(config.compression_algorithm.clone()))
                .expect("Should decompress empty data");
        assert_eq!(decompressed, empty_data);

        // Test single byte
        let single_byte = b"A";
        let compressed =
            compress_test_data(single_byte, &config).expect("Should handle single byte");
        let decompressed =
            decompress_test_data(&compressed, &Some(config.compression_algorithm.clone()))
                .expect("Should decompress single byte");
        assert_eq!(decompressed, single_byte);

        // Test binary data
        let binary_data: Vec<u8> = (0..=255).collect();
        let compressed =
            compress_test_data(&binary_data, &config).expect("Should handle binary data");
        let decompressed =
            decompress_test_data(&compressed, &Some(config.compression_algorithm.clone()))
                .expect("Should decompress binary data");
        assert_eq!(decompressed, binary_data);
    }

    #[test]
    fn test_compression_performance_characteristics() {
        let large_data =
            b"Performance test data with repeated patterns for compression efficiency testing. "
                .repeat(1000);

        let config = ResponseCacheConfig::default();

        let start = std::time::Instant::now();
        let compressed =
            compress_test_data(&large_data, &config).expect("Compression should complete");
        let compression_time = start.elapsed();

        let start = std::time::Instant::now();
        let decompressed =
            decompress_test_data(&compressed, &Some(config.compression_algorithm.clone()))
                .expect("Decompression should complete");
        let decompression_time = start.elapsed();

        assert_eq!(decompressed, large_data);

        // Compression and decompression should complete in reasonable time
        assert!(
            compression_time < std::time::Duration::from_millis(100),
            "Compression took too long: {:?}",
            compression_time
        );
        assert!(
            decompression_time < std::time::Duration::from_millis(50),
            "Decompression took too long: {:?}",
            decompression_time
        );

        // Should achieve some compression
        let compression_ratio = large_data.len() as f32 / compressed.len() as f32;
        assert!(
            compression_ratio > 1.5,
            "Should achieve reasonable compression ratio: {:.2}",
            compression_ratio
        );
    }

    // Helper functions for testing compression without needing a full ResponseCache instance
    fn compress_test_data(data: &[u8], config: &ResponseCacheConfig) -> Result<Vec<u8>> {
        match config.compression_algorithm {
            CompressionAlgorithm::Gzip => {
                use std::io::Write;
                let mut encoder = flate2::write::GzEncoder::new(
                    Vec::new(),
                    flate2::Compression::new(config.compression_level),
                );
                encoder.write_all(data)?;
                encoder.finish().map_err(Into::into)
            }
            CompressionAlgorithm::Zstd => {
                zstd::encode_all(data, config.compression_level as i32).map_err(Into::into)
            }
        }
    }

    fn decompress_test_data(
        data: &[u8],
        algorithm: &Option<CompressionAlgorithm>,
    ) -> Result<Vec<u8>> {
        match algorithm.as_ref().unwrap() {
            CompressionAlgorithm::Gzip => {
                use std::io::Read;
                let mut decoder = flate2::read::GzDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }
            CompressionAlgorithm::Zstd => zstd::decode_all(data).map_err(Into::into),
        }
    }
}

// Smart caching strategy that adapts based on usage patterns
pub struct SmartCachingStrategy {
    request_patterns: Arc<RwLock<HashMap<String, RequestPattern>>>,
}

#[derive(Debug, Clone)]
pub(crate) struct RequestPattern {
    frequency: u64,
    last_seen: SystemTime,
    response_time_ms: u64,
    cache_worthiness_score: f32,
}

impl Default for SmartCachingStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl SmartCachingStrategy {
    pub fn new() -> Self {
        Self {
            request_patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn should_cache(
        &self,
        key: &CacheKey,
        response_time_ms: u64,
        response_size: usize,
    ) -> bool {
        let pattern_key = format!("{}:{}", key.model_id, &key.request_hash[..8]);
        let mut patterns = self.request_patterns.write().await;

        let pattern = patterns.entry(pattern_key).or_insert(RequestPattern {
            frequency: 0,
            last_seen: SystemTime::now(),
            response_time_ms: 0,
            cache_worthiness_score: 0.0,
        });

        pattern.frequency += 1;
        pattern.last_seen = SystemTime::now();
        pattern.response_time_ms = (pattern.response_time_ms + response_time_ms) / 2;

        // Calculate cache worthiness score based on multiple factors
        let frequency_score = (pattern.frequency as f32).ln().max(0.0) / 10.0;
        let response_time_score = (response_time_ms as f32 / 1000.0).min(10.0) / 10.0;
        let size_score = (response_size as f32 / (1024.0 * 1024.0)).min(1.0); // Up to 1MB gets full score

        pattern.cache_worthiness_score = (frequency_score + response_time_score + size_score) / 3.0;

        // Cache if score is above threshold and response time is significant
        pattern.cache_worthiness_score > 0.3 && response_time_ms > 100
    }

    pub(crate) async fn get_pattern_stats(&self) -> HashMap<String, RequestPattern> {
        self.request_patterns.read().await.clone()
    }
}
