//! Streaming Command - New Architecture
//!
//! This module demonstrates the migration of the streaming command to the new
//! CLI architecture. Focuses on streaming benchmarking and configuration export.
//!
//! Note: This is a focused migration covering non-interactive subcommands.
//! Interactive and long-running server modes remain available through the original module.

use crate::{
    backends::{Backend, BackendType, InferenceParams},
    config::Config,
    interfaces::cli::{Command, CommandContext, CommandOutput},
    models::ModelManager,
    streaming::{StreamingConfig, StreamingManager},
};
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use serde_json::json;
use std::{path::PathBuf, sync::Arc};
use tracing::{info, warn};

// ============================================================================
// StreamingBenchmark - Test streaming performance with concurrent streams
// ============================================================================

/// Test streaming performance with concurrent streams
pub struct StreamingBenchmark {
    config: Config,
    model: String,
    concurrent: usize,
    prompt: String,
    duration: u64,
}

impl StreamingBenchmark {
    pub fn new(
        config: Config,
        model: String,
        concurrent: usize,
        prompt: String,
        duration: u64,
    ) -> Self {
        Self {
            config,
            model,
            concurrent,
            prompt,
            duration,
        }
    }
}

#[async_trait]
impl Command for StreamingBenchmark {
    fn name(&self) -> &str {
        "streaming benchmark"
    }

    fn description(&self) -> &str {
        "Test streaming performance with concurrent streams"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.model.is_empty() {
            anyhow::bail!("Model name cannot be empty");
        }

        if self.prompt.is_empty() {
            anyhow::bail!("Prompt cannot be empty");
        }

        if self.concurrent == 0 {
            anyhow::bail!("Concurrent streams must be at least 1");
        }

        if self.concurrent > 100 {
            anyhow::bail!("Concurrent streams cannot exceed 100");
        }

        if self.duration == 0 {
            anyhow::bail!("Duration must be at least 1 second");
        }

        if self.duration > 3600 {
            anyhow::bail!("Duration cannot exceed 3600 seconds (1 hour)");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!(
            "Starting streaming benchmark with {} concurrent streams",
            self.concurrent
        );

        // Initialize streaming manager
        let streaming_config = StreamingConfig {
            max_concurrent_streams: self.concurrent * 2, // Allow some headroom
            enable_metrics: true,
            ..Default::default()
        };

        let streaming_manager = Arc::new(StreamingManager::new(streaming_config));
        streaming_manager.start().await?;

        // Load model
        let model_manager = ModelManager::new(&self.config.models_dir);
        let model_info = model_manager.resolve_model(&self.model).await?;
        let backend_type = BackendType::from_model_path(&model_info.path).ok_or_else(|| {
            anyhow::anyhow!(
                "No suitable backend found for model: {}",
                model_info.path.display()
            )
        })?;

        // Human-readable output
        if !ctx.json_output {
            println!("🚀 Starting streaming benchmark");
            println!("Model: {}", model_info.name);
            println!("Concurrent streams: {}", self.concurrent);
            println!("Duration: {}s", self.duration);
            println!("Prompt: {}\n", self.prompt);
        }

        let inference_params = InferenceParams {
            max_tokens: 100, // Shorter responses for benchmarking
            temperature: 0.7,
                    top_k: 40,
            top_p: 0.9,
            stream: true,
            stop_sequences: vec![],
            seed: None,
        };

        // Start concurrent streams
        let mut handles = Vec::new();

        for i in 0..self.concurrent {
            let streaming_manager = streaming_manager.clone();
            let model_info = model_info.clone();
            let prompt = self.prompt.clone();
            let inference_params = inference_params.clone();
            let backend_config = self.config.backend_config.clone();
            let duration = self.duration;

            let handle = tokio::spawn(async move {
                let result: Result<(usize, u64, u64)> = async move {
                    let mut backend = Backend::new(backend_type, &backend_config)?;
                    backend.load_model(&model_info).await?;

                    let start_time = std::time::Instant::now();
                    let mut total_tokens = 0u64;
                    let mut total_streams = 0u64;

                    while start_time.elapsed().as_secs() < duration {
                        match streaming_manager
                            .create_enhanced_stream(&mut backend, &prompt, &inference_params)
                            .await
                        {
                            Ok(mut stream) => {
                                total_streams += 1;

                                while let Some(token_result) = stream.next().await {
                                    match token_result {
                                        Ok(streaming_token) => {
                                            if !streaming_token.is_heartbeat() {
                                                total_tokens += 1;
                                            }
                                        }
                                        Err(_) => break,
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Stream {} failed: {}", i, e);
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            }
                        }
                    }

                    Ok((i, total_streams, total_tokens))
                }
                .await;
                result.unwrap_or((i, 0, 0))
            });

            handles.push(handle);
        }

        // Monitor progress
        let monitor_handle = if !ctx.json_output {
            let streaming_manager = streaming_manager.clone();
            let duration = self.duration;
            Some(tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

                for _ in 0..(duration / 5) {
                    interval.tick().await;
                    let metrics = streaming_manager.get_metrics();
                    println!(
                        "📊 Active: {}, Total tokens: {}, Avg tok/s: {:.1}",
                        metrics.active_streams,
                        metrics.total_tokens_streamed,
                        metrics.average_tokens_per_second
                    );
                }
            }))
        } else {
            None
        };

        // Wait for all streams to complete
        let mut total_streams = 0u64;
        let mut total_tokens = 0u64;
        let mut stream_results = Vec::new();

        for handle in handles {
            match handle.await {
                Ok((stream_id, streams, tokens)) => {
                    total_streams += streams;
                    total_tokens += tokens;

                    if !ctx.json_output && ctx.is_verbose() {
                        println!(
                            "Stream {} completed: {} streams, {} tokens",
                            stream_id, streams, tokens
                        );
                    }

                    stream_results.push(json!({
                        "stream_id": stream_id,
                        "streams_created": streams,
                        "tokens_generated": tokens,
                    }));
                }
                Err(e) => {
                    warn!("Stream failed: {}", e);
                }
            }
        }

        if let Some(handle) = monitor_handle {
            handle.abort();
        }

        // Final metrics
        let final_metrics = streaming_manager.get_metrics();

        // Human-readable summary
        if !ctx.json_output {
            println!("\n🏁 Benchmark Results:");
            println!("Total streams created: {}", total_streams);
            println!("Total tokens generated: {}", total_tokens);
            println!(
                "Average tokens/second: {:.1}",
                total_tokens as f32 / self.duration as f32
            );
            println!("Errors: {}", final_metrics.errors_count);
            println!("Timeouts: {}", final_metrics.timeouts);
            println!("Buffer overflows: {}", final_metrics.buffer_overflows);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!(
                "Benchmark completed: {} streams, {} tokens",
                total_streams, total_tokens
            ),
            json!({
                "model": self.model,
                "concurrent_streams": self.concurrent,
                "duration_seconds": self.duration,
                "total_streams_created": total_streams,
                "total_tokens_generated": total_tokens,
                "average_tokens_per_second": total_tokens as f32 / self.duration as f32,
                "per_stream_results": stream_results,
                "metrics": {
                    "total_streams": final_metrics.total_streams_created,
                    "active_streams": final_metrics.active_streams,
                    "total_tokens": final_metrics.total_tokens_streamed,
                    "errors": final_metrics.errors_count,
                    "timeouts": final_metrics.timeouts,
                    "buffer_overflows": final_metrics.buffer_overflows,
                    "average_latency_ms": final_metrics.average_latency_ms,
                },
            }),
        ))
    }
}

// ============================================================================
// StreamingConfigExport - Export streaming configuration
// ============================================================================

/// Export streaming configuration
pub struct StreamingConfigExport {
    format: ConfigFormat,
    output: Option<PathBuf>,
}

impl StreamingConfigExport {
    pub fn new(format: ConfigFormat, output: Option<PathBuf>) -> Self {
        Self { format, output }
    }
}

#[derive(Clone, Debug)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
}

#[async_trait]
impl Command for StreamingConfigExport {
    fn name(&self) -> &str {
        "streaming config"
    }

    fn description(&self) -> &str {
        "Export streaming configuration"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // Validate output path if specified
        if let Some(ref output) = self.output {
            if let Some(parent) = output.parent() {
                if !parent.exists() {
                    anyhow::bail!("Output directory does not exist: {}", parent.display());
                }
            }
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Exporting streaming configuration");

        let config = StreamingConfig::default();

        let content = match self.format {
            ConfigFormat::Json => serde_json::to_string_pretty(&config)?,
            ConfigFormat::Yaml => serde_yaml::to_string(&config)?,
            ConfigFormat::Toml => toml::to_string_pretty(&config)?,
        };

        // Output handling
        match &self.output {
            Some(path) => {
                tokio::fs::write(path, &content).await?;
                if !ctx.json_output {
                    println!("✓ Configuration exported to: {}", path.display());
                }
            }
            None => {
                if !ctx.json_output {
                    println!("{}", content);
                }
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Streaming configuration exported",
            json!({
                "format": format!("{:?}", self.format).to_lowercase(),
                "output_file": self.output.as_ref().map(|p| p.display().to_string()),
                "config": config,
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_streaming_benchmark_validation_empty_model() {
        let config = Config::default();
        let cmd = StreamingBenchmark::new(
            config.clone(),
            String::new(),
            5,
            "test prompt".to_string(),
            30,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Model name cannot be empty"));
    }

    #[tokio::test]
    async fn test_streaming_benchmark_validation_empty_prompt() {
        let config = Config::default();
        let cmd = StreamingBenchmark::new(
            config.clone(),
            "model.gguf".to_string(),
            5,
            String::new(),
            30,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Prompt cannot be empty"));
    }

    #[tokio::test]
    async fn test_streaming_benchmark_validation_zero_concurrent() {
        let config = Config::default();
        let cmd = StreamingBenchmark::new(
            config.clone(),
            "model.gguf".to_string(),
            0,
            "test".to_string(),
            30,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Concurrent streams must be at least 1"));
    }

    #[tokio::test]
    async fn test_streaming_benchmark_validation_excessive_concurrent() {
        let config = Config::default();
        let cmd = StreamingBenchmark::new(
            config.clone(),
            "model.gguf".to_string(),
            200,
            "test".to_string(),
            30,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Concurrent streams cannot exceed 100"));
    }

    #[tokio::test]
    async fn test_streaming_benchmark_validation_excessive_duration() {
        let config = Config::default();
        let cmd = StreamingBenchmark::new(
            config.clone(),
            "model.gguf".to_string(),
            5,
            "test".to_string(),
            5000,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Duration cannot exceed 3600 seconds"));
    }

    #[tokio::test]
    async fn test_config_export_validation() {
        let config = Config::default();
        let cmd = StreamingConfigExport::new(ConfigFormat::Json, None);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }
}
