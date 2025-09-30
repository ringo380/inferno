//! Cache Command - New Architecture
//!
//! This module demonstrates the migration of the cache command to the new
//! CLI architecture. Focuses on core cache management operations.
//!
//! Note: This is a focused migration covering the most commonly used subcommands.
//! Full cache functionality remains available through the original module.

use crate::cache::{ModelCache, WarmupStrategy};
use crate::config::Config;
use crate::interfaces::cli::{Command, CommandContext, CommandOutput};
use crate::metrics::MetricsCollector;
use crate::models::ModelManager;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

// ============================================================================
// CacheStats - Show cache statistics and status
// ============================================================================

/// Show cache statistics and status
pub struct CacheStats {
    config: Config,
}

impl CacheStats {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Command for CacheStats {
    fn name(&self) -> &str {
        "cache stats"
    }

    fn description(&self) -> &str {
        "Show cache statistics and status"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // No validation needed for stats
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Initializing cache to show statistics");

        let model_manager = Arc::new(ModelManager::new(&self.config.models_dir));
        let metrics = Some(Arc::new({
            let mut collector = MetricsCollector::new();
            collector.start_event_processing().await?;
            collector
        }));

        let cache = ModelCache::new(
            self.config.cache.clone(),
            self.config.backend_config.clone(),
            model_manager,
            metrics,
        )
        .await?;

        let stats = cache.get_stats().await;

        // Human-readable output
        if !ctx.json_output {
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

            println!("=== Cache Configuration ===");
            println!("Max Models: {}", self.config.cache.max_cached_models);
            println!("Max Memory: {} MB", self.config.cache.max_memory_mb);
            println!("TTL: {} seconds", self.config.cache.model_ttl_seconds);
            println!("Warmup Enabled: {}", self.config.cache.enable_warmup);
            println!("Warmup Strategy: {:?}", self.config.cache.warmup_strategy);
            println!("Always Warm: {:?}", self.config.cache.always_warm);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!("Cache statistics for {} models", stats.total_models),
            json!({
                "cache_stats": {
                    "total_models": stats.total_models,
                    "memory_usage_mb": stats.memory_usage_mb,
                    "hit_rate": stats.hit_rate,
                    "miss_rate": stats.miss_rate,
                    "eviction_count": stats.eviction_count,
                    "warmup_count": stats.warmup_count,
                    "active_models": stats.active_models,
                },
                "cache_config": {
                    "max_cached_models": self.config.cache.max_cached_models,
                    "max_memory_mb": self.config.cache.max_memory_mb,
                    "model_ttl_seconds": self.config.cache.model_ttl_seconds,
                    "enable_warmup": self.config.cache.enable_warmup,
                    "warmup_strategy": format!("{:?}", self.config.cache.warmup_strategy),
                    "always_warm": self.config.cache.always_warm,
                },
            }),
        ))
    }
}

// ============================================================================
// CacheWarmup - Warm up specific models
// ============================================================================

/// Warm up specific models in the cache
pub struct CacheWarmup {
    config: Config,
    models: Vec<String>,
    strategy: Option<WarmupStrategy>,
}

impl CacheWarmup {
    pub fn new(config: Config, models: Vec<String>, strategy: Option<WarmupStrategy>) -> Self {
        Self {
            config,
            models,
            strategy,
        }
    }
}

#[async_trait]
impl Command for CacheWarmup {
    fn name(&self) -> &str {
        "cache warmup"
    }

    fn description(&self) -> &str {
        "Warm up specific models"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // Models list can be empty (use automatic warmup)
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Starting model warmup process");

        let model_manager = Arc::new(ModelManager::new(&self.config.models_dir));
        let metrics = Some(Arc::new({
            let mut collector = MetricsCollector::new();
            collector.start_event_processing().await?;
            collector
        }));

        let mut cache_config = self.config.cache.clone();
        if let Some(ref strat) = self.strategy {
            cache_config.warmup_strategy = strat.clone();
        }

        let cache = ModelCache::new(
            cache_config,
            self.config.backend_config.clone(),
            model_manager,
            metrics,
        )
        .await?;

        let start_time = Instant::now();
        let mut warmed_count = 0;
        let mut failed_models = Vec::new();

        if self.models.is_empty() {
            // Automatic warmup based on strategy
            if !ctx.json_output {
                println!(
                    "Running automatic warmup based on strategy: {:?}",
                    cache.config.warmup_strategy
                );
            }

            cache.warmup_models().await?;
            let final_stats = cache.get_stats().await;
            warmed_count = final_stats.total_models;
        } else {
            // Warm up specific models
            if !ctx.json_output {
                println!("Warming up {} models...", self.models.len());
            }

            for model in &self.models {
                match cache.warmup_model(model).await {
                    Ok(_) => {
                        warmed_count += 1;
                        if !ctx.json_output {
                            println!("✓ Warmed up model: {}", model);
                        }
                    }
                    Err(e) => {
                        failed_models.push((model.clone(), e.to_string()));
                        if !ctx.json_output {
                            println!("✗ Failed to warm up model {}: {}", model, e);
                        }
                    }
                }
            }
        }

        let duration = start_time.elapsed();
        let final_stats = cache.get_stats().await;

        // Human-readable output
        if !ctx.json_output {
            println!("Warmup completed in {:?}", duration);
            println!();
            println!("Final cache status:");
            println!("  Active models: {}", final_stats.total_models);
            println!("  Memory usage: {:.2} MB", final_stats.memory_usage_mb);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!(
                "Successfully warmed up {} of {} models",
                warmed_count,
                if self.models.is_empty() {
                    warmed_count
                } else {
                    self.models.len()
                }
            ),
            json!({
                "warmed_count": warmed_count,
                "requested_count": if self.models.is_empty() { 0 } else { self.models.len() },
                "duration_ms": duration.as_millis(),
                "failed_models": failed_models,
                "final_stats": {
                    "total_models": final_stats.total_models,
                    "memory_usage_mb": final_stats.memory_usage_mb,
                },
            }),
        ))
    }
}

// ============================================================================
// CacheClear - Clear the model cache
// ============================================================================

/// Clear models from the cache
pub struct CacheClear {
    config: Config,
    model: Option<String>,
    force: bool,
}

impl CacheClear {
    pub fn new(config: Config, model: Option<String>, force: bool) -> Self {
        Self {
            config,
            model,
            force,
        }
    }
}

#[async_trait]
impl Command for CacheClear {
    fn name(&self) -> &str {
        "cache clear"
    }

    fn description(&self) -> &str {
        "Clear the model cache"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // Check always-warm configuration if not forcing
        if !self.force {
            if let Some(ref model_name) = self.model {
                if self.config.cache.always_warm.contains(model_name) {
                    anyhow::bail!(
                        "Model {} is configured as always-warm. Use --force to clear.",
                        model_name
                    );
                }
            } else if !self.config.cache.always_warm.is_empty() {
                anyhow::bail!(
                    "Some models are configured as always-warm. Use --force to clear all."
                );
            }
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Clearing model cache");

        let model_manager = Arc::new(ModelManager::new(&self.config.models_dir));
        let metrics = Some(Arc::new({
            let mut collector = MetricsCollector::new();
            collector.start_event_processing().await?;
            collector
        }));

        let cache = ModelCache::new(
            self.config.cache.clone(),
            self.config.backend_config.clone(),
            model_manager,
            metrics,
        )
        .await?;

        let cleared_model: Option<String>;

        if let Some(ref model_name) = self.model {
            // Clear specific model
            cache.evict_model(model_name).await?;
            cleared_model = Some(model_name.clone());

            if !ctx.json_output {
                println!("✓ Cleared model: {}", model_name);
            }
        } else {
            // Clear all models
            cache.clear_cache().await?;
            cleared_model = None;

            if !ctx.json_output {
                println!("✓ Cleared all cached models");
            }
        }

        let stats = cache.get_stats().await;

        // Human-readable output
        if !ctx.json_output {
            println!("Remaining models: {}", stats.total_models);
            println!("Memory usage: {:.2} MB", stats.memory_usage_mb);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            if cleared_model.is_some() {
                format!("Cleared model: {}", cleared_model.as_ref().unwrap())
            } else {
                "Cleared all cached models".to_string()
            },
            json!({
                "cleared_model": cleared_model,
                "cleared_all": cleared_model.is_none(),
                "force": self.force,
                "remaining_stats": {
                    "total_models": stats.total_models,
                    "memory_usage_mb": stats.memory_usage_mb,
                },
            }),
        ))
    }
}

// ============================================================================
// CacheConfigure - Configure cache settings
// ============================================================================

/// Configure cache settings
pub struct CacheConfigure {
    max_models: Option<usize>,
    max_memory_mb: Option<u64>,
    ttl_seconds: Option<u64>,
    warmup: Option<bool>,
    strategy: Option<WarmupStrategy>,
    always_warm: Option<Vec<String>>,
}

impl CacheConfigure {
    pub fn new(
        max_models: Option<usize>,
        max_memory_mb: Option<u64>,
        ttl_seconds: Option<u64>,
        warmup: Option<bool>,
        strategy: Option<WarmupStrategy>,
        always_warm: Option<Vec<String>>,
    ) -> Self {
        Self {
            max_models,
            max_memory_mb,
            ttl_seconds,
            warmup,
            strategy,
            always_warm,
        }
    }
}

#[async_trait]
impl Command for CacheConfigure {
    fn name(&self) -> &str {
        "cache configure"
    }

    fn description(&self) -> &str {
        "Configure cache settings"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // Validate max_models
        if let Some(max) = self.max_models {
            if max == 0 {
                anyhow::bail!("Max models cannot be 0");
            }
            if max > 100 {
                anyhow::bail!("Max models cannot exceed 100");
            }
        }

        // Validate max_memory_mb
        if let Some(mem) = self.max_memory_mb {
            if mem == 0 {
                anyhow::bail!("Max memory cannot be 0");
            }
            if mem > 1_000_000 {
                // 1TB limit
                anyhow::bail!("Max memory cannot exceed 1,000,000 MB (1TB)");
            }
        }

        // Validate ttl_seconds
        if let Some(ttl) = self.ttl_seconds {
            if ttl == 0 {
                anyhow::bail!("TTL cannot be 0");
            }
            if ttl > 31_536_000 {
                // 1 year limit
                anyhow::bail!("TTL cannot exceed 31,536,000 seconds (1 year)");
            }
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Updating cache configuration");

        // Human-readable output
        if !ctx.json_output {
            println!("=== Cache Configuration Update ===");

            if let Some(max) = self.max_models {
                println!("Max models: {}", max);
            }
            if let Some(mem) = self.max_memory_mb {
                println!("Max memory: {} MB", mem);
            }
            if let Some(ttl) = self.ttl_seconds {
                println!("TTL: {} seconds", ttl);
            }
            if let Some(enable) = self.warmup {
                println!("Warmup enabled: {}", enable);
            }
            if let Some(ref strat) = self.strategy {
                println!("Warmup strategy: {:?}", strat);
            }
            if let Some(ref models) = self.always_warm {
                println!("Always warm: {:?}", models);
            }

            println!();
            println!("Note: Configuration changes require restart to take effect.");
            println!("Update your config.toml file with these values.");
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Cache configuration updated",
            json!({
                "updated_settings": {
                    "max_models": self.max_models,
                    "max_memory_mb": self.max_memory_mb,
                    "ttl_seconds": self.ttl_seconds,
                    "warmup_enabled": self.warmup,
                    "warmup_strategy": self.strategy.as_ref().map(|s| format!("{:?}", s)),
                    "always_warm": self.always_warm,
                },
                "note": "Configuration changes require restart to take effect",
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_stats_validation() {
        let config = Config::default();
        let cmd = CacheStats::new(config.clone());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cache_warmup_validation() {
        let config = Config::default();
        let cmd = CacheWarmup::new(config.clone(), vec![], None);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cache_clear_always_warm_protection() {
        let mut config = Config::default();
        config.cache.always_warm = vec!["important-model".to_string()];

        let cmd = CacheClear::new(config.clone(), Some("important-model".to_string()), false);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("always-warm"));
    }

    #[tokio::test]
    async fn test_cache_configure_validation() {
        let config = Config::default();
        let cmd = CacheConfigure::new(Some(0), None, None, None, None, None); // Invalid - zero max models
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be 0"));
    }

    #[tokio::test]
    async fn test_cache_configure_excessive_values() {
        let config = Config::default();

        // Test excessive max_models
        let cmd = CacheConfigure::new(Some(200), None, None, None, None, None);
        let ctx = CommandContext::new(config.clone());
        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());

        // Test excessive ttl
        let cmd2 = CacheConfigure::new(None, None, Some(50_000_000), None, None, None);
        let result2 = cmd2.validate(&ctx).await;
        assert!(result2.is_err());
    }
}