use crate::{
    config::Config,
    metrics::MetricsCollector,
    response_cache::{CacheKey, HashAlgorithm, ResponseCache, ResponseMetadata},
};
use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
use std::sync::Arc;
use tracing::info;

#[derive(Args)]
pub struct ResponseCacheArgs {
    #[command(subcommand)]
    pub command: ResponseCacheCommand,
}

#[derive(Subcommand)]
pub enum ResponseCacheCommand {
    #[command(about = "Show response cache statistics")]
    Stats,

    #[command(about = "Test response cache functionality")]
    Test {
        #[arg(help = "Number of test requests", default_value = "100")]
        requests: usize,

        #[arg(long, help = "Test deduplication")]
        test_dedup: bool,

        #[arg(long, help = "Test compression")]
        test_compression: bool,
    },

    #[command(about = "Clear response cache")]
    Clear {
        #[arg(long, help = "Clear entries matching pattern")]
        pattern: Option<String>,
    },

    #[command(about = "Invalidate cache entries")]
    Invalidate {
        #[arg(help = "Pattern to match for invalidation")]
        pattern: String,
    },

    #[command(about = "Configure response cache settings")]
    Configure {
        #[arg(long, help = "Enable/disable cache")]
        enabled: Option<bool>,

        #[arg(long, help = "Maximum cache entries")]
        max_entries: Option<usize>,

        #[arg(long, help = "Maximum memory in MB")]
        max_memory_mb: Option<u64>,

        #[arg(long, help = "TTL in seconds")]
        ttl_seconds: Option<u64>,

        #[arg(long, help = "Enable deduplication")]
        deduplication: Option<bool>,

        #[arg(long, help = "Enable compression")]
        compression: Option<bool>,

        #[arg(long, help = "Hash algorithm", value_enum)]
        hash_algorithm: Option<HashAlgorithmArg>,
    },

    #[command(about = "Benchmark cache performance")]
    Benchmark {
        #[arg(
            short,
            long,
            help = "Number of benchmark iterations",
            default_value = "1000"
        )]
        iterations: usize,

        #[arg(long, help = "Response data size in bytes", default_value = "1024")]
        data_size: usize,

        #[arg(long, help = "Hit rate percentage (0-100)", default_value = "30")]
        hit_rate: u8,
    },

    #[command(about = "Monitor cache usage in real-time")]
    Monitor {
        #[arg(short, long, help = "Update interval in seconds", default_value = "5")]
        interval: u64,

        #[arg(long, help = "Show detailed statistics")]
        detailed: bool,
    },

    #[command(about = "Export cache configuration")]
    Export {
        #[arg(short, long, help = "Output file path")]
        output: Option<std::path::PathBuf>,

        #[arg(long, help = "Export format", value_enum, default_value = "json")]
        format: ExportFormat,
    },
}

/// Configuration for cache settings
/// Reduces function signature from 8 parameters to 2
pub struct CacheSettingsConfig {
    pub enabled: Option<bool>,
    pub max_entries: Option<usize>,
    pub max_memory_mb: Option<u64>,
    pub ttl_seconds: Option<u64>,
    pub deduplication: Option<bool>,
    pub compression: Option<bool>,
    pub hash_algorithm: Option<HashAlgorithmArg>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum HashAlgorithmArg {
    Sha256,
    Blake3,
    Xxhash,
}

impl From<HashAlgorithmArg> for HashAlgorithm {
    fn from(arg: HashAlgorithmArg) -> Self {
        match arg {
            HashAlgorithmArg::Sha256 => HashAlgorithm::Sha256,
            HashAlgorithmArg::Blake3 => HashAlgorithm::Blake3,
            HashAlgorithmArg::Xxhash => HashAlgorithm::Xxhash,
        }
    }
}

#[derive(Clone, ValueEnum)]
pub enum ExportFormat {
    Json,
    Yaml,
    Toml,
}

pub async fn execute(args: ResponseCacheArgs, config: &Config) -> Result<()> {
    match args.command {
        ResponseCacheCommand::Stats => show_cache_stats(config).await,
        ResponseCacheCommand::Test {
            requests,
            test_dedup,
            test_compression,
        } => test_cache(config, requests, test_dedup, test_compression).await,
        ResponseCacheCommand::Clear { pattern } => clear_cache(config, pattern).await,
        ResponseCacheCommand::Invalidate { pattern } => invalidate_cache(config, pattern).await,
        ResponseCacheCommand::Configure {
            enabled,
            max_entries,
            max_memory_mb,
            ttl_seconds,
            deduplication,
            compression,
            hash_algorithm,
        } => {
            let settings = CacheSettingsConfig {
                enabled,
                max_entries,
                max_memory_mb,
                ttl_seconds,
                deduplication,
                compression,
                hash_algorithm,
            };
            configure_cache(config, settings).await
        }
        ResponseCacheCommand::Benchmark {
            iterations,
            data_size,
            hit_rate,
        } => benchmark_cache(config, iterations, data_size, hit_rate).await,
        ResponseCacheCommand::Monitor { interval, detailed } => {
            monitor_cache(config, interval, detailed).await
        }
        ResponseCacheCommand::Export { output, format } => {
            export_cache_config(config, output, format).await
        }
    }
}

async fn show_cache_stats(config: &Config) -> Result<()> {
    info!("Initializing response cache to show statistics...");

    let metrics = Some(Arc::new({
        let (collector, processor) = MetricsCollector::new();
        processor.start();
        collector
    }));

    let cache = ResponseCache::new(config.response_cache.clone(), metrics).await?;
    let stats = cache.get_stats().await;

    println!("=== Response Cache Statistics ===");
    println!("Total Requests: {}", stats.total_requests);
    println!("Cache Hits: {}", stats.cache_hits);
    println!("Cache Misses: {}", stats.cache_misses);
    println!("Hit Rate: {:.2}%", stats.hit_rate * 100.0);
    println!("Total Entries: {}", stats.total_entries);
    println!("Memory Usage: {:.2} MB", stats.memory_usage_mb);
    println!(
        "Deduplication Savings: {} bytes",
        stats.deduplication_savings
    );
    println!("Compression Ratio: {:.2}x", stats.compression_ratio);
    println!("Evictions: {}", stats.evictions);
    println!("Expired Entries: {}", stats.expired_entries);

    println!("\n=== Configuration ===");
    println!("Enabled: {}", config.response_cache.enabled);
    println!("Max Entries: {}", config.response_cache.max_entries);
    println!("Max Memory: {} MB", config.response_cache.max_memory_mb);
    println!("TTL: {} seconds", config.response_cache.ttl_seconds);
    println!(
        "Deduplication: {}",
        config.response_cache.deduplication_enabled
    );
    println!("Compression: {}", config.response_cache.compression_enabled);
    println!("Hash Algorithm: {:?}", config.response_cache.hash_algorithm);
    println!("Cache Strategy: {:?}", config.response_cache.cache_strategy);
    println!(
        "Eviction Policy: {:?}",
        config.response_cache.eviction_policy
    );

    Ok(())
}

async fn test_cache(
    config: &Config,
    requests: usize,
    test_dedup: bool,
    test_compression: bool,
) -> Result<()> {
    // Validate request count
    if requests == 0 {
        return Err(anyhow::anyhow!("Request count must be at least 1"));
    }
    if requests > 10000 {
        return Err(anyhow::anyhow!("Request count cannot exceed 10000"));
    }

    println!("Testing response cache with {} requests...", requests);

    let mut cache_config = config.response_cache.clone();
    cache_config.deduplication_enabled = test_dedup;
    cache_config.compression_enabled = test_compression;

    let cache = ResponseCache::new(cache_config, None).await?;

    println!("Running cache test...");
    let start_time = std::time::Instant::now();

    // Phase 1: Fill cache with unique requests
    let unique_requests = requests / 2;
    for i in 0..unique_requests {
        let key = CacheKey::new(
            &format!("test request {}", i),
            "test-model",
            "temperature=0.7",
            &config.response_cache.hash_algorithm,
        );

        let response_data = format!("Response for request {}", i).repeat(10);
        let metadata = ResponseMetadata {
            model_id: "test-model".to_string(),
            response_type: "text".to_string(),
            token_count: Some(response_data.len() as u32 / 4),
            processing_time_ms: 100 + i as u64,
            quality_score: Some(0.9),
            content_type: "text/plain".to_string(),
        };

        cache
            .put(&key, response_data.into_bytes(), metadata)
            .await?;
    }

    // Phase 2: Test cache hits with existing requests
    let mut hits = 0;
    for i in 0..unique_requests {
        let key = CacheKey::new(
            &format!("test request {}", i),
            "test-model",
            "temperature=0.7",
            &config.response_cache.hash_algorithm,
        );

        if cache.get(&key).await.is_some() {
            hits += 1;
        }
    }

    // Phase 3: Test deduplication if enabled
    if test_dedup {
        println!("Testing deduplication...");
        let duplicate_data = "This is duplicate content".repeat(50);

        for i in 0..10 {
            let key = CacheKey::new(
                &format!("duplicate request {}", i),
                "test-model",
                "temperature=0.7",
                &config.response_cache.hash_algorithm,
            );

            let metadata = ResponseMetadata {
                model_id: "test-model".to_string(),
                response_type: "text".to_string(),
                token_count: Some(duplicate_data.len() as u32 / 4),
                processing_time_ms: 150,
                quality_score: Some(0.8),
                content_type: "text/plain".to_string(),
            };

            cache
                .put(&key, duplicate_data.clone().into_bytes(), metadata)
                .await?;
        }
    }

    let duration = start_time.elapsed();
    let stats = cache.get_stats().await;

    println!("\n=== Test Results ===");
    println!("Test Duration: {:?}", duration);
    println!("Total Operations: {}", requests);
    println!(
        "Cache Hits: {}/{} ({:.2}%)",
        hits,
        unique_requests,
        hits as f32 / unique_requests as f32 * 100.0
    );
    println!("Final Stats:");
    println!("  Total Entries: {}", stats.total_entries);
    println!("  Memory Usage: {:.2} MB", stats.memory_usage_mb);

    if test_dedup {
        println!(
            "  Deduplication Savings: {} bytes",
            stats.deduplication_savings
        );
    }

    if test_compression {
        println!("  Compression Ratio: {:.2}x", stats.compression_ratio);
    }

    Ok(())
}

async fn clear_cache(config: &Config, pattern: Option<String>) -> Result<()> {
    let cache = ResponseCache::new(config.response_cache.clone(), None).await?;

    match pattern {
        Some(p) => {
            let removed = cache.invalidate(&p).await?;
            println!("Cleared {} cache entries matching pattern: {}", removed, p);
        }
        None => {
            cache.clear().await?;
            println!("Cleared all cache entries");
        }
    }

    Ok(())
}

async fn invalidate_cache(config: &Config, pattern: String) -> Result<()> {
    // Validate pattern is not empty
    if pattern.is_empty() {
        return Err(anyhow::anyhow!("Pattern cannot be empty"));
    }

    let cache = ResponseCache::new(config.response_cache.clone(), None).await?;
    let removed = cache.invalidate(&pattern).await?;

    println!(
        "Invalidated {} cache entries matching pattern: {}",
        removed, pattern
    );

    Ok(())
}

async fn configure_cache(_config: &Config, settings: CacheSettingsConfig) -> Result<()> {
    // Validate max_entries if provided
    if let Some(entries) = settings.max_entries {
        if entries == 0 {
            return Err(anyhow::anyhow!("Max entries must be at least 1"));
        }
    }

    println!("=== Response Cache Configuration Update ===");

    if let Some(e) = settings.enabled {
        println!("Enabled: {}", e);
    }
    if let Some(max) = settings.max_entries {
        println!("Max entries: {}", max);
    }
    if let Some(mem) = settings.max_memory_mb {
        println!("Max memory: {} MB", mem);
    }
    if let Some(ttl) = settings.ttl_seconds {
        println!("TTL: {} seconds", ttl);
    }
    if let Some(dedup) = settings.deduplication {
        println!("Deduplication: {}", dedup);
    }
    if let Some(comp) = settings.compression {
        println!("Compression: {}", comp);
    }
    if let Some(hash) = settings.hash_algorithm {
        println!("Hash algorithm: {:?}", hash);
    }

    println!("\nNote: Configuration changes require restart to take effect.");
    println!("Update your config.toml file with these values.");

    Ok(())
}

async fn benchmark_cache(
    config: &Config,
    iterations: usize,
    data_size: usize,
    hit_rate: u8,
) -> Result<()> {
    if hit_rate > 100 {
        return Err(anyhow::anyhow!("Hit rate cannot exceed 100%"));
    }

    println!("Benchmarking response cache performance...");
    println!("Iterations: {}", iterations);
    println!("Data size: {} bytes", data_size);
    println!("Target hit rate: {}%", hit_rate);

    let cache = ResponseCache::new(config.response_cache.clone(), None).await?;

    // Generate test data
    let test_data = "x".repeat(data_size);

    // Pre-populate cache to achieve target hit rate
    let cache_entries = (iterations * hit_rate as usize) / 100;
    println!("Pre-populating cache with {} entries...", cache_entries);

    for i in 0..cache_entries {
        let key = CacheKey::new(
            &format!("benchmark request {}", i),
            "benchmark-model",
            "temperature=0.7",
            &config.response_cache.hash_algorithm,
        );

        let metadata = ResponseMetadata {
            model_id: "benchmark-model".to_string(),
            response_type: "text".to_string(),
            token_count: Some(data_size as u32 / 4),
            processing_time_ms: 100,
            quality_score: Some(0.9),
            content_type: "text/plain".to_string(),
        };

        cache
            .put(&key, test_data.clone().into_bytes(), metadata)
            .await?;
    }

    println!("Running benchmark...");
    let start_time = std::time::Instant::now();

    let mut hits = 0;
    let mut misses = 0;

    for i in 0..iterations {
        let request_id = if i < cache_entries {
            i // This should be a cache hit
        } else {
            cache_entries + i // This should be a cache miss
        };

        let key = CacheKey::new(
            &format!("benchmark request {}", request_id),
            "benchmark-model",
            "temperature=0.7",
            &config.response_cache.hash_algorithm,
        );

        if cache.get(&key).await.is_some() {
            hits += 1;
        } else {
            misses += 1;

            // Add new entry for cache miss
            let metadata = ResponseMetadata {
                model_id: "benchmark-model".to_string(),
                response_type: "text".to_string(),
                token_count: Some(data_size as u32 / 4),
                processing_time_ms: 100,
                quality_score: Some(0.9),
                content_type: "text/plain".to_string(),
            };

            cache
                .put(&key, test_data.clone().into_bytes(), metadata)
                .await?;
        }
    }

    let duration = start_time.elapsed();
    let ops_per_second = iterations as f64 / duration.as_secs_f64();

    println!("\n=== Benchmark Results ===");
    println!("Total operations: {}", iterations);
    println!("Cache hits: {}", hits);
    println!("Cache misses: {}", misses);
    println!(
        "Actual hit rate: {:.2}%",
        hits as f32 / iterations as f32 * 100.0
    );
    println!("Total time: {:?}", duration);
    println!("Operations per second: {:.2}", ops_per_second);
    println!("Average operation time: {:?}", duration / iterations as u32);

    let final_stats = cache.get_stats().await;
    println!("\n=== Final Cache Statistics ===");
    println!("Total entries: {}", final_stats.total_entries);
    println!("Memory usage: {:.2} MB", final_stats.memory_usage_mb);
    println!("Hit rate: {:.2}%", final_stats.hit_rate * 100.0);

    Ok(())
}

async fn monitor_cache(config: &Config, interval: u64, detailed: bool) -> Result<()> {
    println!("Starting response cache monitor...");
    println!("Press Ctrl+C to stop monitoring");
    println!("Update interval: {} seconds", interval);

    let cache = ResponseCache::new(config.response_cache.clone(), None).await?;

    let mut counter = 0;
    loop {
        if counter % 20 == 0 {
            // Print header every 20 iterations
            if detailed {
                println!(
                    "\n{:<8} {:<8} {:<8} {:<8} {:<10} {:<8} {:<8}",
                    "Time", "Entries", "Hits", "Misses", "Memory(MB)", "Hit%", "Evict"
                );
            } else {
                println!(
                    "\n{:<8} {:<8} {:<8} {:<10} {:<8}",
                    "Time", "Entries", "Hits", "Memory(MB)", "Hit%"
                );
            }
        }

        let stats = cache.get_stats().await;
        let now = chrono::Utc::now().format("%H:%M:%S");

        if detailed {
            println!(
                "{:<8} {:<8} {:<8} {:<8} {:<10.2} {:<8.1} {:<8}",
                now,
                stats.total_entries,
                stats.cache_hits,
                stats.cache_misses,
                stats.memory_usage_mb,
                stats.hit_rate * 100.0,
                stats.evictions
            );
        } else {
            println!(
                "{:<8} {:<8} {:<8} {:<10.2} {:<8.1}",
                now,
                stats.total_entries,
                stats.cache_hits,
                stats.memory_usage_mb,
                stats.hit_rate * 100.0
            );
        }

        counter += 1;
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }
}

async fn export_cache_config(
    config: &Config,
    output: Option<std::path::PathBuf>,
    format: ExportFormat,
) -> Result<()> {
    let cache_config = &config.response_cache;

    let output_str = match format {
        ExportFormat::Json => serde_json::to_string_pretty(cache_config)?,
        ExportFormat::Yaml => serde_yaml::to_string(cache_config)
            .map_err(|e| anyhow::anyhow!("YAML serialization failed: {}", e))?,
        ExportFormat::Toml => toml::to_string_pretty(cache_config)?,
    };

    if let Some(path) = output {
        tokio::fs::write(&path, output_str).await?;
        println!("Response cache configuration exported to: {:?}", path);
    } else {
        println!("{}", output_str);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_test_validation_zero_requests() {
        let config = Config::default();
        let result = test_cache(&config, 0, false, false).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Request count must be at least 1")
        );
    }

    #[tokio::test]
    async fn test_cache_test_validation_excessive_requests() {
        let config = Config::default();
        let result = test_cache(&config, 20000, false, false).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Request count cannot exceed 10000")
        );
    }

    #[tokio::test]
    async fn test_cache_invalidate_validation_empty_pattern() {
        let config = Config::default();
        let result = invalidate_cache(&config, String::new()).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Pattern cannot be empty")
        );
    }

    #[tokio::test]
    async fn test_cache_configure_validation_zero_entries() {
        let config = Config::default();
        let settings = CacheSettingsConfig {
            enabled: None,
            max_entries: Some(0),
            max_memory_mb: None,
            ttl_seconds: None,
            deduplication: None,
            compression: None,
            hash_algorithm: None,
        };
        let result = configure_cache(&config, settings).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Max entries must be at least 1")
        );
    }

    #[tokio::test]
    async fn test_benchmark_validation_hit_rate_exceeds_100() {
        let config = Config::default();
        let result = benchmark_cache(&config, 100, 1024, 150).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Hit rate cannot exceed 100%")
        );
    }
}
