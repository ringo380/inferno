use crate::{
    cache::{ModelCache, WarmupStrategy},
    config::Config,
    metrics::MetricsCollector,
    models::ModelManager,
};
use anyhow::{Result, bail};
use clap::{Args, Subcommand, ValueEnum};
use serde_json;
use std::{sync::Arc, time::Instant};
use tracing::{info, warn};

// ============================================================================
// Validation Constants
// ============================================================================

/// Maximum number of models that can be cached
const MAX_CACHED_MODELS_LIMIT: usize = 100;

/// Maximum memory limit in MB (1TB)
const MAX_MEMORY_MB_LIMIT: u64 = 1_000_000;

/// Maximum TTL in seconds (1 year)
const MAX_TTL_SECONDS_LIMIT: u64 = 31_536_000;

#[derive(Args)]
pub struct CacheArgs {
    #[command(subcommand)]
    pub command: CacheCommand,
}

#[derive(Subcommand)]
pub enum CacheCommand {
    #[command(about = "Show cache statistics and status")]
    Stats,

    #[command(about = "Warm up specific models")]
    Warmup {
        #[arg(help = "Models to warm up (space-separated)")]
        models: Vec<String>,

        #[arg(long, help = "Warmup strategy to use")]
        strategy: Option<WarmupStrategyArg>,

        #[arg(long, help = "Maximum concurrent loads", default_value = "2")]
        concurrent: usize,
    },

    #[command(about = "Clear the model cache")]
    Clear {
        #[arg(long, help = "Clear specific model")]
        model: Option<String>,

        #[arg(long, help = "Force clear even always-warm models")]
        force: bool,
    },

    #[command(about = "Configure cache settings")]
    Configure {
        #[arg(long, help = "Maximum cached models")]
        max_models: Option<usize>,

        #[arg(long, help = "Maximum memory in MB")]
        max_memory_mb: Option<u64>,

        #[arg(long, help = "Model TTL in seconds")]
        ttl_seconds: Option<u64>,

        #[arg(long, help = "Enable/disable warmup")]
        warmup: Option<bool>,

        #[arg(long, help = "Warmup strategy")]
        strategy: Option<WarmupStrategyArg>,

        #[arg(long, help = "Always warm models (comma-separated)")]
        always_warm: Option<String>,
    },

    #[command(about = "Benchmark cache performance")]
    Benchmark {
        #[arg(
            short,
            long,
            help = "Number of test requests per model",
            default_value = "10"
        )]
        requests: usize,

        #[arg(short, long, help = "Test models (space-separated)")]
        models: Vec<String>,

        #[arg(long, help = "Enable concurrent requests")]
        concurrent: bool,
    },

    #[command(about = "Monitor cache usage in real-time")]
    Monitor {
        #[arg(short, long, help = "Update interval in seconds", default_value = "5")]
        interval: u64,

        #[arg(long, help = "Show detailed model statistics")]
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

#[derive(Clone, Debug, ValueEnum)]
pub enum WarmupStrategyArg {
    UsageBased,
    Predictive,
    SizeOptimized,
    Priority,
    Hybrid,
}

impl From<WarmupStrategyArg> for WarmupStrategy {
    fn from(arg: WarmupStrategyArg) -> Self {
        match arg {
            WarmupStrategyArg::UsageBased => WarmupStrategy::UsageBased,
            WarmupStrategyArg::Predictive => WarmupStrategy::Predictive,
            WarmupStrategyArg::SizeOptimized => WarmupStrategy::SizeOptimized,
            WarmupStrategyArg::Priority => WarmupStrategy::Priority,
            WarmupStrategyArg::Hybrid => WarmupStrategy::Hybrid,
        }
    }
}

#[derive(Clone, ValueEnum)]
pub enum ExportFormat {
    Json,
    Yaml,
    Toml,
}

pub async fn execute(args: CacheArgs, config: &Config) -> Result<()> {
    match args.command {
        CacheCommand::Stats => show_cache_stats(config).await,
        CacheCommand::Warmup {
            models,
            strategy,
            concurrent,
        } => warmup_models(config, models, strategy, concurrent).await,
        CacheCommand::Clear { model, force } => clear_cache(config, model, force).await,
        CacheCommand::Configure {
            max_models,
            max_memory_mb,
            ttl_seconds,
            warmup,
            strategy,
            always_warm,
        } => {
            configure_cache(
                config,
                max_models,
                max_memory_mb,
                ttl_seconds,
                warmup,
                strategy,
                always_warm,
            )
            .await
        }
        CacheCommand::Benchmark {
            requests,
            models,
            concurrent,
        } => benchmark_cache(config, requests, models, concurrent).await,
        CacheCommand::Monitor { interval, detailed } => {
            monitor_cache(config, interval, detailed).await
        }
        CacheCommand::Export { output, format } => {
            export_cache_config(config, output, format).await
        }
    }
}

async fn show_cache_stats(config: &Config) -> Result<()> {
    info!("Initializing cache to show statistics...");

    let model_manager = Arc::new(ModelManager::new(&config.models_dir));
    let metrics = Some(Arc::new({
        let (collector, processor) = MetricsCollector::new();
        processor.start();
        collector
    }));

    let cache = ModelCache::new(
        config.cache.clone(),
        config.backend_config.clone(),
        model_manager,
        metrics,
    )
    .await?;

    let stats = cache.get_stats().await;

    println!("=== Model Cache Statistics ===");
    println!("Total Models: {}", stats.total_models);
    println!("Memory Usage: {:.2} MB", stats.memory_usage_mb);
    println!("Hit Rate: {:.2}%", stats.hit_rate * 100.0);
    println!("Miss Rate: {:.2}%", stats.miss_rate * 100.0);
    println!("Evictions: {}", stats.eviction_count);
    println!("Warmups: {}", stats.warmup_count);

    if !stats.active_models.is_empty() {
        println!("\nActive Models:");
        for model in &stats.active_models {
            println!("  - {}", model);
        }
    }

    if !stats.model_stats.is_empty() {
        println!("\n=== Model Usage Statistics ===");
        for (name, model_stats) in &stats.model_stats {
            println!("Model: {}", name);
            println!("  Requests: {}", model_stats.request_count);
            println!(
                "  Avg Response Time: {:?}",
                model_stats.average_response_time
            );
            println!(
                "  Memory Usage: {:.2} MB",
                model_stats.memory_usage as f64 / (1024.0 * 1024.0)
            );
            println!(
                "  Usage Frequency: {:.2} req/hour",
                model_stats.usage_frequency
            );
            println!("  Usage Trend: {:.2}", model_stats.usage_trend);
            println!();
        }
    }

    println!("\n=== Cache Configuration ===");
    println!("Max Models: {}", config.cache.max_cached_models);
    println!("Max Memory: {} MB", config.cache.max_memory_mb);
    println!("TTL: {} seconds", config.cache.model_ttl_seconds);
    println!("Warmup Enabled: {}", config.cache.enable_warmup);
    println!("Warmup Strategy: {:?}", config.cache.warmup_strategy);
    println!("Always Warm: {:?}", config.cache.always_warm);

    Ok(())
}

async fn warmup_models(
    config: &Config,
    models: Vec<String>,
    strategy: Option<WarmupStrategyArg>,
    concurrent: usize,
) -> Result<()> {
    info!("Starting model warmup process...");

    let model_manager = Arc::new(ModelManager::new(&config.models_dir));
    let metrics = Some(Arc::new({
        let (collector, processor) = MetricsCollector::new();
        processor.start();
        collector
    }));

    let mut cache_config = config.cache.clone();
    if let Some(strat) = strategy {
        cache_config.warmup_strategy = strat.into();
    }

    let cache = ModelCache::new(
        cache_config,
        config.backend_config.clone(),
        model_manager,
        metrics,
    )
    .await?;

    if models.is_empty() {
        // Use configured warmup strategy
        info!(
            "Running automatic warmup based on strategy: {:?}",
            cache.config.warmup_strategy
        );
        let start_time = Instant::now();
        cache.warmup_models().await?;
        let duration = start_time.elapsed();
        println!("Automatic warmup completed in {:?}", duration);
    } else {
        // Warm up specific models
        println!("Warming up {} models...", models.len());
        let start_time = Instant::now();

        if concurrent > 1 {
            println!("  Concurrent warmup not supported due to cache limitations");
            println!("  Running sequential warmup instead...");
        }

        {
            // Sequential warmup
            for model in models {
                match cache.warmup_model(&model).await {
                    Ok(_) => println!("✓ Warmed up model: {}", model),
                    Err(e) => warn!("Failed to warm up model {}: {}", model, e),
                }
            }
        }

        let duration = start_time.elapsed();
        println!("Warmup completed in {:?}", duration);
    }

    let final_stats = cache.get_stats().await;
    println!("\nFinal cache status:");
    println!("Active models: {}", final_stats.total_models);
    println!("Memory usage: {:.2} MB", final_stats.memory_usage_mb);

    Ok(())
}

async fn clear_cache(config: &Config, model: Option<String>, force: bool) -> Result<()> {
    // Validate always-warm configuration upfront (fail fast instead of warn)
    if !force {
        if let Some(ref model_name) = model {
            if config.cache.always_warm.contains(model_name) {
                bail!(
                    "Model '{}' is configured as always-warm. Use --force to clear.",
                    model_name
                );
            }
        } else if !config.cache.always_warm.is_empty() {
            bail!(
                "Some models are configured as always-warm: {:?}. Use --force to clear all.",
                config.cache.always_warm
            );
        }
    }

    info!("Clearing model cache...");

    let model_manager = Arc::new(ModelManager::new(&config.models_dir));
    let metrics = Some(Arc::new({
        let (collector, processor) = MetricsCollector::new();
        processor.start();
        collector
    }));

    let cache = ModelCache::new(
        config.cache.clone(),
        config.backend_config.clone(),
        model_manager,
        metrics,
    )
    .await?;

    if let Some(model_name) = model {
        cache.evict_model(&model_name).await?;
        println!("✓ Cleared model: {}", model_name);
    } else {
        cache.clear_cache().await?;
        println!("✓ Cleared all cached models");
    }

    let stats = cache.get_stats().await;
    println!("Remaining models: {}", stats.total_models);
    println!("Memory usage: {:.2} MB", stats.memory_usage_mb);

    Ok(())
}

/// Validate configuration parameters
fn validate_cache_config(
    max_models: Option<usize>,
    max_memory_mb: Option<u64>,
    ttl_seconds: Option<u64>,
) -> Result<()> {
    // Validate max_models
    if let Some(max) = max_models {
        if max == 0 {
            bail!("Max models cannot be 0");
        }
        if max > MAX_CACHED_MODELS_LIMIT {
            bail!(
                "Max models cannot exceed {} (got {})",
                MAX_CACHED_MODELS_LIMIT,
                max
            );
        }
    }

    // Validate max_memory_mb
    if let Some(mem) = max_memory_mb {
        if mem == 0 {
            bail!("Max memory cannot be 0");
        }
        if mem > MAX_MEMORY_MB_LIMIT {
            bail!(
                "Max memory cannot exceed {} MB (1TB) (got {} MB)",
                MAX_MEMORY_MB_LIMIT,
                mem
            );
        }
    }

    // Validate ttl_seconds
    if let Some(ttl) = ttl_seconds {
        if ttl == 0 {
            bail!("TTL cannot be 0");
        }
        if ttl > MAX_TTL_SECONDS_LIMIT {
            bail!(
                "TTL cannot exceed {} seconds (1 year) (got {} seconds)",
                MAX_TTL_SECONDS_LIMIT,
                ttl
            );
        }
    }

    Ok(())
}

async fn configure_cache(
    _config: &Config,
    max_models: Option<usize>,
    max_memory_mb: Option<u64>,
    ttl_seconds: Option<u64>,
    warmup: Option<bool>,
    strategy: Option<WarmupStrategyArg>,
    always_warm: Option<String>,
) -> Result<()> {
    // Validate inputs first
    validate_cache_config(max_models, max_memory_mb, ttl_seconds)?;

    println!("=== Cache Configuration Update ===");

    if let Some(max) = max_models {
        println!("Max models: {}", max);
    }
    if let Some(mem) = max_memory_mb {
        println!("Max memory: {} MB", mem);
    }
    if let Some(ttl) = ttl_seconds {
        println!("TTL: {} seconds", ttl);
    }
    if let Some(enable) = warmup {
        println!("Warmup enabled: {}", enable);
    }
    if let Some(strat) = strategy {
        println!("Warmup strategy: {:?}", strat);
    }
    if let Some(models) = always_warm {
        let model_list: Vec<&str> = models.split(',').collect();
        println!("Always warm: {:?}", model_list);
    }

    println!("\nNote: Configuration changes require restart to take effect.");
    println!("Update your config.toml file with these values.");

    Ok(())
}

async fn benchmark_cache(
    config: &Config,
    requests: usize,
    models: Vec<String>,
    concurrent: bool,
) -> Result<()> {
    // Validate inputs
    if models.is_empty() {
        bail!("No models specified for benchmark. Use --models to specify models to test.");
    }
    if requests == 0 {
        bail!("Number of requests must be greater than 0");
    }

    info!("Starting cache benchmark...");
    println!("Benchmark parameters:");
    println!("  Requests per model: {}", requests);
    println!("  Models: {:?}", models);
    println!("  Concurrent: {}", concurrent);

    let model_manager = Arc::new(ModelManager::new(&config.models_dir));
    let metrics = Some(Arc::new({
        let (collector, processor) = MetricsCollector::new();
        processor.start();
        collector
    }));

    let cache = ModelCache::new(
        config.cache.clone(),
        config.backend_config.clone(),
        model_manager,
        metrics,
    )
    .await?;

    println!("\n=== Benchmark Results ===");

    for model in &models {
        println!("\nBenchmarking model: {}", model);
        let start_time = Instant::now();

        if concurrent {
            println!("  Concurrent benchmarking not supported due to cache limitations");
            println!("  Running sequential benchmark instead...");
        }

        {
            // Sequential requests
            let mut successful = 0;
            for i in 0..requests {
                let request_start = Instant::now();
                if cache.get_model(model).await.is_ok() {
                    successful += 1;
                }
                let request_duration = request_start.elapsed();
                if i == 0 {
                    println!("    First request (cold): {:?}", request_duration);
                } else if i == 1 {
                    println!("    Second request (warm): {:?}", request_duration);
                }
            }

            let total_duration = start_time.elapsed();
            println!("  Sequential results:");
            println!("    Successful requests: {}/{}", successful, requests);
            println!("    Total time: {:?}", total_duration);
            println!(
                "    Average per request: {:?}",
                total_duration / requests as u32
            );
            println!(
                "    Requests per second: {:.2}",
                requests as f64 / total_duration.as_secs_f64()
            );
        }
    }

    let final_stats = cache.get_stats().await;
    println!("\n=== Final Cache Statistics ===");
    println!("Hit rate: {:.2}%", final_stats.hit_rate * 100.0);
    println!("Miss rate: {:.2}%", final_stats.miss_rate * 100.0);
    println!("Active models: {}", final_stats.total_models);
    println!("Memory usage: {:.2} MB", final_stats.memory_usage_mb);

    Ok(())
}

async fn monitor_cache(config: &Config, interval: u64, detailed: bool) -> Result<()> {
    info!("Starting cache monitor...");

    let model_manager = Arc::new(ModelManager::new(&config.models_dir));
    let metrics = Some(Arc::new({
        let (collector, processor) = MetricsCollector::new();
        processor.start();
        collector
    }));

    let cache = ModelCache::new(
        config.cache.clone(),
        config.backend_config.clone(),
        model_manager,
        metrics,
    )
    .await?;

    println!("Press Ctrl+C to stop monitoring");
    println!("Update interval: {} seconds", interval);

    let mut counter = 0;
    loop {
        if counter % 20 == 0 {
            // Print header every 20 iterations
            if detailed {
                println!(
                    "\n{:<8} {:<6} {:<10} {:<8} {:<6} {:<6}",
                    "Time", "Models", "Memory(MB)", "Hit%", "Evict", "Warmup"
                );
            } else {
                println!(
                    "\n{:<8} {:<6} {:<10} {:<8}",
                    "Time", "Models", "Memory(MB)", "Hit%"
                );
            }
        }

        let stats = cache.get_stats().await;
        let now = chrono::Utc::now().format("%H:%M:%S");

        if detailed {
            println!(
                "{:<8} {:<6} {:<10.2} {:<8.1} {:<6} {:<6}",
                now,
                stats.total_models,
                stats.memory_usage_mb,
                stats.hit_rate * 100.0,
                stats.eviction_count,
                stats.warmup_count
            );
        } else {
            println!(
                "{:<8} {:<6} {:<10.2} {:<8.1}",
                now,
                stats.total_models,
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
    let cache_config = &config.cache;

    let output_str = match format {
        ExportFormat::Json => serde_json::to_string_pretty(cache_config)?,
        ExportFormat::Yaml => serde_yaml::to_string(cache_config)
            .map_err(|e| anyhow::anyhow!("YAML serialization failed: {}", e))?,
        ExportFormat::Toml => toml::to_string_pretty(cache_config)?,
    };

    if let Some(path) = output {
        tokio::fs::write(&path, output_str).await?;
        println!("Cache configuration exported to: {:?}", path);
    } else {
        println!("{}", output_str);
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cache_config_valid() {
        // Valid configuration
        let result = validate_cache_config(Some(10), Some(1024), Some(3600));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_cache_config_none_values() {
        // All None values should be valid
        let result = validate_cache_config(None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_cache_config_max_models_zero() {
        let result = validate_cache_config(Some(0), None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be 0"));
    }

    #[test]
    fn test_validate_cache_config_max_models_excessive() {
        let result = validate_cache_config(Some(200), None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot exceed"));
    }

    #[test]
    fn test_validate_cache_config_max_memory_zero() {
        let result = validate_cache_config(None, Some(0), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be 0"));
    }

    #[test]
    fn test_validate_cache_config_max_memory_excessive() {
        let result = validate_cache_config(None, Some(2_000_000), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot exceed"));
    }

    #[test]
    fn test_validate_cache_config_ttl_zero() {
        let result = validate_cache_config(None, None, Some(0));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be 0"));
    }

    #[test]
    fn test_validate_cache_config_ttl_excessive() {
        let result = validate_cache_config(None, None, Some(50_000_000));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot exceed"));
    }

    #[test]
    fn test_validate_cache_config_boundary_values() {
        // Test boundary values (should be valid)
        let result = validate_cache_config(
            Some(MAX_CACHED_MODELS_LIMIT),
            Some(MAX_MEMORY_MB_LIMIT),
            Some(MAX_TTL_SECONDS_LIMIT),
        );
        assert!(result.is_ok());

        // Test just over boundary (should fail)
        let result = validate_cache_config(Some(MAX_CACHED_MODELS_LIMIT + 1), None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_warmup_strategy_conversion() {
        assert!(matches!(
            WarmupStrategy::from(WarmupStrategyArg::UsageBased),
            WarmupStrategy::UsageBased
        ));
        assert!(matches!(
            WarmupStrategy::from(WarmupStrategyArg::Predictive),
            WarmupStrategy::Predictive
        ));
        assert!(matches!(
            WarmupStrategy::from(WarmupStrategyArg::SizeOptimized),
            WarmupStrategy::SizeOptimized
        ));
        assert!(matches!(
            WarmupStrategy::from(WarmupStrategyArg::Priority),
            WarmupStrategy::Priority
        ));
        assert!(matches!(
            WarmupStrategy::from(WarmupStrategyArg::Hybrid),
            WarmupStrategy::Hybrid
        ));
    }
}
