use crate::{
    config::Config,
    metrics::MetricsCollector,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};
use tokio::{
    sync::{Mutex, RwLock},
    time::interval,
};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCacheConfig {
    pub enabled: bool,
    pub max_entries: usize,
    pub max_memory_mb: u64,
    pub ttl_seconds: u64,
    pub deduplication_enabled: bool,
    pub compression_enabled: bool,
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
            hash_algorithm: HashAlgorithm::Sha256,
            cache_strategy: CacheStrategy::Smart,
            eviction_policy: EvictionPolicy::LeastRecentlyUsed,
        }
    }
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
    pub fn new(request_text: &str, model_id: &str, parameters: &str, algorithm: &HashAlgorithm) -> Self {
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
                // Placeholder - would use blake3 crate in real implementation
                let mut hasher = Sha256::new();
                hasher.update(b"blake3:");
                hasher.update(input.as_bytes());
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::Xxhash => {
                // Placeholder - would use xxhash crate in real implementation
                let mut hasher = Sha256::new();
                hasher.update(b"xxhash:");
                hasher.update(input.as_bytes());
                format!("{:x}", hasher.finalize())
            }
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.model_id, self.request_hash, self.parameters_hash)
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
            dedup_map.get(&cache_key).unwrap_or(&cache_key).clone()
        } else {
            cache_key.clone()
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
                self.decompress_data(&cached_response.response_data)
            } else {
                cached_response.response_data.clone()
            };

            debug!("Cache hit for key: {}", cache_key);
            return Some(response_data);
        }

        stats.cache_misses += 1;
        stats.hit_rate = stats.cache_hits as f32 / stats.total_requests as f32;
        debug!("Cache miss for key: {}", cache_key);

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

        // Compress data if enabled
        let (final_data, compressed) = if self.config.compression_enabled {
            (self.compress_data(&response_data), true)
        } else {
            (response_data.clone(), false)
        };

        let cached_response = Arc::new(CachedResponse {
            response_data: final_data.clone(),
            metadata,
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            access_count: 1,
            size_bytes: final_data.len(),
            compressed,
        });

        // Check memory limits before inserting
        self.ensure_memory_limits(&cached_response).await?;

        let mut cache = self.cache.write().await;
        cache.insert(actual_key, cached_response);

        self.update_stats().await;

        debug!("Cached response for key: {} (compressed: {}, size: {} bytes)",
               cache_key, compressed, final_data.len());

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

        info!("Invalidated {} cache entries matching pattern: {}", removed_count, pattern);
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
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn compress_data(&self, data: &[u8]) -> Vec<u8> {
        // Placeholder compression - in real implementation would use flate2 or similar
        let mut compressed = Vec::new();
        compressed.extend_from_slice(b"COMPRESSED:");
        compressed.extend_from_slice(data);
        compressed
    }

    fn decompress_data(&self, data: &[u8]) -> Vec<u8> {
        // Placeholder decompression
        if data.starts_with(b"COMPRESSED:") {
            data[11..].to_vec()
        } else {
            data.to_vec()
        }
    }

    fn is_expired(&self, cached_response: &CachedResponse) -> bool {
        let ttl = Duration::from_secs(self.config.ttl_seconds);
        cached_response.created_at.elapsed().unwrap_or(Duration::ZERO) > ttl
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
        if let Some(cached_response) = cache.get(key) {
            // Note: In a real implementation, we'd need to use interior mutability
            // or a different approach to update access stats
            debug!("Updated access stats for key: {}", key);
        }
    }

    async fn ensure_memory_limits(&self, new_entry: &CachedResponse) -> Result<()> {
        let cache = self.cache.read().await;
        let current_memory = self.calculate_memory_usage(&cache).await;
        let max_memory = (self.config.max_memory_mb * 1024 * 1024) as usize;

        if cache.len() >= self.config.max_entries ||
           current_memory + new_entry.size_bytes > max_memory {
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
                entries.into_iter().map(|(k, _)| k).take(cache.len() / 4).collect::<Vec<String>>()
            }
            EvictionPolicy::LeastFrequentlyUsed => {
                let mut entries: Vec<(String, u64)> = cache
                    .iter()
                    .map(|(k, v)| (k.clone(), v.access_count))
                    .collect();
                entries.sort_by_key(|(_, count)| *count);
                entries.into_iter().map(|(k, _)| k).take(cache.len() / 4).collect::<Vec<String>>()
            }
            EvictionPolicy::TimeToLive => {
                cache
                    .iter()
                    .filter(|(_, v)| self.is_expired(v))
                    .map(|(k, _)| k.clone())
                    .collect()
            }
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
                entries.into_iter().map(|(k, _)| k).take(cache.len() / 4).collect::<Vec<String>>()
            }
        };

        let evicted_count = entries_to_evict.len();
        for key in entries_to_evict {
            cache.remove(&key);
        }

        let mut stats = self.stats.lock().await;
        stats.evictions += evicted_count as u64;

        info!("Evicted {} cache entries using {:?} policy", evicted_count, self.config.eviction_policy);

        Ok(())
    }

    async fn update_stats(&self) {
        let cache = self.cache.read().await;
        let memory_usage = self.calculate_memory_usage(&cache).await;

        let mut stats = self.stats.lock().await;
        stats.total_entries = cache.len();
        stats.memory_usage_bytes = memory_usage;
        stats.memory_usage_mb = memory_usage as f32 / (1024.0 * 1024.0);

        // Calculate compression ratio
        let total_compressed_size: usize = cache
            .values()
            .filter(|entry| entry.compressed)
            .map(|entry| entry.size_bytes)
            .sum();

        let total_original_size = cache
            .values()
            .map(|entry| {
                if entry.compressed {
                    // Estimate original size (placeholder)
                    entry.size_bytes * 2
                } else {
                    entry.size_bytes
                }
            })
            .sum::<usize>();

        if total_compressed_size > 0 {
            stats.compression_ratio = total_original_size as f32 / total_compressed_size as f32;
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

                    info!("Background cleanup removed {} expired cache entries", expired_count);
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

// Smart caching strategy that adapts based on usage patterns
pub struct SmartCachingStrategy {
    request_patterns: Arc<RwLock<HashMap<String, RequestPattern>>>,
}

#[derive(Debug, Clone)]
struct RequestPattern {
    frequency: u64,
    last_seen: SystemTime,
    response_time_ms: u64,
    cache_worthiness_score: f32,
}

impl SmartCachingStrategy {
    pub fn new() -> Self {
        Self {
            request_patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn should_cache(&self, key: &CacheKey, response_time_ms: u64, response_size: usize) -> bool {
        let pattern_key = format!("{}:{}", key.model_id, key.request_hash[..8].to_string());
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

    pub async fn get_pattern_stats(&self) -> HashMap<String, RequestPattern> {
        self.request_patterns.read().await.clone()
    }
}