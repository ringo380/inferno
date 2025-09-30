//! Batch Command - New Architecture
//!
//! This module demonstrates the migration of the batch processing command
//! to the new CLI architecture. Handles bulk inference processing with
//! checkpointing, metrics, and error handling.

use crate::{
    backends::{Backend, BackendType, InferenceParams},
    batch::{BatchConfig, BatchOutputFormat, BatchProcessor},
    config::Config,
    interfaces::cli::{Command, CommandContext, CommandOutput},
    metrics::MetricsCollector,
    models::ModelManager,
};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::{path::PathBuf, sync::Arc};
use tracing::{info, warn};

// ============================================================================
// BatchProcess - Process inputs in batch mode
// ============================================================================

/// Process inputs in batch mode
pub struct BatchProcess {
    config: Config,
    model: String,
    input: PathBuf,
    output: Option<PathBuf>,
    output_format: BatchOutputFormat,
    max_tokens: u32,
    temperature: f32,
    top_p: f32,
    concurrency: usize,
    timeout: u64,
    retries: u32,
    checkpoint: u32,
    continue_on_error: bool,
    shuffle: bool,
    enable_metrics: bool,
    resume: Option<PathBuf>,
    dry_run: bool,
    backend: Option<BackendType>,
}

impl BatchProcess {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: Config,
        model: String,
        input: PathBuf,
        output: Option<PathBuf>,
        output_format: BatchOutputFormat,
        max_tokens: u32,
        temperature: f32,
        top_p: f32,
        concurrency: usize,
        timeout: u64,
        retries: u32,
        checkpoint: u32,
        continue_on_error: bool,
        shuffle: bool,
        enable_metrics: bool,
        resume: Option<PathBuf>,
        dry_run: bool,
        backend: Option<BackendType>,
    ) -> Self {
        Self {
            config,
            model,
            input,
            output,
            output_format,
            max_tokens,
            temperature,
            top_p,
            concurrency,
            timeout,
            retries,
            checkpoint,
            continue_on_error,
            shuffle,
            enable_metrics,
            resume,
            dry_run,
            backend,
        }
    }
}

#[async_trait]
impl Command for BatchProcess {
    fn name(&self) -> &str {
        "batch"
    }

    fn description(&self) -> &str {
        "Process multiple inputs in batch mode"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // Validate model name
        if self.model.is_empty() {
            anyhow::bail!("Model name cannot be empty");
        }

        // Validate input file
        if !self.input.exists() {
            anyhow::bail!("Input file does not exist: {}", self.input.display());
        }

        // Validate output directory if specified
        if let Some(ref output) = self.output {
            if let Some(parent) = output.parent() {
                if !parent.exists() {
                    anyhow::bail!("Output directory does not exist: {}", parent.display());
                }
            }
        }

        // Validate resume checkpoint if specified
        if let Some(ref resume) = self.resume {
            if !resume.exists() {
                anyhow::bail!("Checkpoint file does not exist: {}", resume.display());
            }
        }

        // Validate parameter ranges
        if self.max_tokens == 0 {
            anyhow::bail!("Max tokens must be greater than 0");
        }

        if self.max_tokens > 32768 {
            anyhow::bail!("Max tokens cannot exceed 32768");
        }

        if self.temperature < 0.0 || self.temperature > 2.0 {
            anyhow::bail!("Temperature must be between 0.0 and 2.0");
        }

        if self.top_p < 0.0 || self.top_p > 1.0 {
            anyhow::bail!("Top-p must be between 0.0 and 1.0");
        }

        if self.concurrency == 0 {
            anyhow::bail!("Concurrency must be at least 1");
        }

        if self.concurrency > 128 {
            anyhow::bail!("Concurrency cannot exceed 128");
        }

        if self.timeout == 0 {
            anyhow::bail!("Timeout must be at least 1 second");
        }

        if self.checkpoint == 0 {
            anyhow::bail!("Checkpoint interval must be at least 1");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Starting batch processing with model: {}", self.model);

        // Dry run mode - validate inputs only
        if self.dry_run {
            return self.validate_batch_inputs(ctx).await;
        }

        // Set up metrics if requested
        let metrics = if self.enable_metrics {
            let mut collector = MetricsCollector::new();
            collector.start_event_processing().await?;
            Some(Arc::new(collector))
        } else {
            None
        };

        // Create batch configuration
        let batch_config = BatchConfig {
            concurrency: self.concurrency,
            timeout_seconds: self.timeout,
            retry_attempts: self.retries,
            checkpoint_interval: self.checkpoint,
            output_format: self.output_format.clone(),
            continue_on_error: self.continue_on_error,
            shuffle_inputs: self.shuffle,
        };

        // Load and validate model
        let model_manager = ModelManager::new(&self.config.models_dir);
        let model_info = model_manager.resolve_model(&self.model).await?;

        if !ctx.json_output && ctx.is_verbose() {
            info!("Validating model: {}", model_info.name);
        }

        let validation_result = model_manager
            .validate_model_comprehensive(&model_info.path, Some(&self.config))
            .await?;

        if !validation_result.is_valid {
            warn!("Model validation issues:");
            for error in &validation_result.errors {
                warn!("  - {}", error);
            }
            if !self.continue_on_error {
                anyhow::bail!("Model validation failed");
            }
        }

        // Determine backend
        let backend_type = self
            .backend
            .or_else(|| BackendType::from_model_path(&model_info.path))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No suitable backend found for model: {}",
                    model_info.path.display()
                )
            })?;

        let mut backend = Backend::new(backend_type, &self.config.backend_config)?;

        // Load model
        if !ctx.json_output {
            info!("Loading model...");
        }

        let load_start = std::time::Instant::now();
        backend.load_model(&model_info).await?;
        let load_duration = load_start.elapsed();

        if !ctx.json_output && ctx.is_verbose() {
            info!("Model loaded in {:?}", load_duration);
        }

        // Record model load metrics
        if let Some(ref metrics) = metrics {
            metrics.record_model_loaded(
                model_info.name.clone(),
                model_info.size,
                load_duration,
                backend_type.to_string(),
            );
        }

        // Create inference parameters
        let inference_params = InferenceParams {
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            top_p: self.top_p,
            stream: false, // Batch processing uses non-streaming
            stop_sequences: vec![],
            seed: None,
        };

        // Estimate total items for progress tracking
        let total_items = self.estimate_batch_size().await?;

        if !ctx.json_output {
            info!("Estimated {} items to process", total_items);
        }

        // Create batch processor
        let mut processor = BatchProcessor::new(batch_config, total_items);
        if let Some(metrics) = metrics {
            processor = processor.with_metrics(metrics);
        }

        // Determine output path
        let default_output = self.input.with_extension("batch.jsonl");
        let output_path = self.output.as_deref().unwrap_or(default_output.as_path());

        if !ctx.json_output {
            info!("Output will be saved to: {}", output_path.display());
        }

        // Process the batch
        let progress = processor
            .process_file(
                &mut backend,
                &self.input,
                Some(output_path),
                &inference_params,
            )
            .await?;

        // Human-readable summary
        if !ctx.json_output {
            self.print_batch_summary(&progress);
        }

        // Structured output
        let success_rate = if progress.total_items > 0 {
            (progress.completed_items as f64 / progress.total_items as f64) * 100.0
        } else {
            0.0
        };

        let duration_secs = progress
            .estimated_completion
            .and_then(|comp| {
                comp.signed_duration_since(progress.start_time)
                    .to_std()
                    .ok()
            })
            .map(|d| d.as_secs_f64());

        Ok(CommandOutput::success_with_data(
            format!(
                "Processed {} items with {:.1}% success rate",
                progress.total_items, success_rate
            ),
            json!({
                "input_file": self.input.display().to_string(),
                "output_file": output_path.display().to_string(),
                "model": self.model,
                "total_items": progress.total_items,
                "completed_items": progress.completed_items,
                "failed_items": progress.failed_items,
                "skipped_items": progress.skipped_items,
                "success_rate": success_rate,
                "average_rate": progress.current_rate,
                "duration_seconds": duration_secs,
                "config": {
                    "concurrency": self.concurrency,
                    "max_tokens": self.max_tokens,
                    "temperature": self.temperature,
                    "top_p": self.top_p,
                    "continue_on_error": self.continue_on_error,
                },
            }),
        ))
    }
}

impl BatchProcess {
    async fn validate_batch_inputs(&self, ctx: &CommandContext) -> Result<CommandOutput> {
        info!("Validating batch inputs (dry run mode)");

        let batch_config = BatchConfig::default();
        let processor = BatchProcessor::new(batch_config, 0);

        let inputs = processor.load_inputs(&self.input).await?;

        if !ctx.json_output {
            info!(
                "✓ Successfully parsed {} inputs from {}",
                inputs.len(),
                self.input.display()
            );

            if ctx.is_verbose() {
                info!("Sample inputs:");
                for input in inputs.iter().take(3) {
                    info!(
                        "  {}: {} ({})",
                        input.id,
                        input.content.chars().take(50).collect::<String>(),
                        if input.content.len() > 50 { "..." } else { "" }
                    );
                }
                if inputs.len() > 3 {
                    info!("  ... and {} more", inputs.len() - 3);
                }
            }

            // Validate output path
            if let Some(ref output_path) = self.output {
                if let Some(parent) = output_path.parent() {
                    if !parent.exists() {
                        warn!("Output directory does not exist: {}", parent.display());
                    }
                }
            }

            info!("✓ Batch validation complete - ready for processing");
        }

        Ok(CommandOutput::success_with_data(
            format!("Validated {} inputs", inputs.len()),
            json!({
                "valid": true,
                "input_count": inputs.len(),
                "input_file": self.input.display().to_string(),
                "sample_inputs": inputs.iter().take(3).map(|i| json!({
                    "id": i.id,
                    "content": i.content.chars().take(100).collect::<String>(),
                })).collect::<Vec<_>>(),
            }),
        ))
    }

    async fn estimate_batch_size(&self) -> Result<usize> {
        let content = tokio::fs::read_to_string(&self.input).await?;
        let extension = self
            .input
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let count = match extension.to_lowercase().as_str() {
            "json" => {
                let value: serde_json::Value = serde_json::from_str(&content)?;
                match value {
                    serde_json::Value::Array(ref items) => items.len(),
                    _ => 1,
                }
            }
            "jsonl" | "ndjson" => content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count(),
            "csv" | "tsv" => {
                let delimiter = if extension == "tsv" { b'\t' } else { b',' };
                let mut rdr = csv::ReaderBuilder::new()
                    .delimiter(delimiter)
                    .from_reader(content.as_bytes());
                rdr.records().count()
            }
            _ => content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count(),
        };

        Ok(count)
    }

    fn print_batch_summary(&self, progress: &crate::batch::BatchProgress) {
        println!("\n=== Batch Processing Summary ===");
        println!("Input file: {}", self.input.display());
        println!("Model: {}", self.model);
        println!("Total items: {}", progress.total_items);
        println!("Completed: {}", progress.completed_items);
        println!("Failed: {}", progress.failed_items);
        println!("Skipped: {}", progress.skipped_items);

        let success_rate = if progress.total_items > 0 {
            (progress.completed_items as f64 / progress.total_items as f64) * 100.0
        } else {
            0.0
        };
        println!("Success rate: {:.1}%", success_rate);

        if let Some(completion_time) = progress.estimated_completion {
            let duration = completion_time - progress.start_time;
            println!(
                "Processing time: {}",
                humantime::format_duration(duration.to_std().unwrap_or(std::time::Duration::ZERO))
            );
        }

        println!("Average rate: {:.2} items/second", progress.current_rate);

        if let Some(ref output) = self.output {
            println!("Output saved to: {}", output.display());
        }

        if progress.failed_items > 0 {
            println!("\n⚠️  {} items failed processing", progress.failed_items);
            if self.continue_on_error {
                println!("Failed items are included in output with error details");
            }
        }

        if progress.completed_items > 0 {
            println!("\n✅ Batch processing completed successfully!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_validation_empty_model() {
        let config = Config::default();
        let cmd = BatchProcess::new(
            config.clone(),
            String::new(),
            PathBuf::from("test.json"),
            None,
            BatchOutputFormat::JsonLines,
            512,
            0.7,
            0.9,
            4,
            300,
            3,
            100,
            false,
            false,
            false,
            None,
            false,
            None,
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
    async fn test_batch_validation_invalid_temperature() {
        let config = Config::default();
        let cmd = BatchProcess::new(
            config.clone(),
            "test-model".to_string(),
            PathBuf::from("test.json"),
            None,
            BatchOutputFormat::JsonLines,
            512,
            3.0, // Invalid temperature
            0.9,
            4,
            300,
            3,
            100,
            false,
            false,
            false,
            None,
            false,
            None,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Temperature must be between"));
    }

    #[tokio::test]
    async fn test_batch_validation_invalid_concurrency() {
        let config = Config::default();
        let cmd = BatchProcess::new(
            config.clone(),
            "test-model".to_string(),
            PathBuf::from("test.json"),
            None,
            BatchOutputFormat::JsonLines,
            512,
            0.7,
            0.9,
            0, // Invalid concurrency
            300,
            3,
            100,
            false,
            false,
            false,
            None,
            false,
            None,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Concurrency must be at least 1"));
    }

    #[tokio::test]
    async fn test_batch_validation_excessive_concurrency() {
        let config = Config::default();
        let cmd = BatchProcess::new(
            config.clone(),
            "test-model".to_string(),
            PathBuf::from("test.json"),
            None,
            BatchOutputFormat::JsonLines,
            512,
            0.7,
            0.9,
            200, // Excessive concurrency
            300,
            3,
            100,
            false,
            false,
            false,
            None,
            false,
            None,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Concurrency cannot exceed 128"));
    }

    #[tokio::test]
    async fn test_batch_validation_invalid_max_tokens() {
        let config = Config::default();
        let cmd = BatchProcess::new(
            config.clone(),
            "test-model".to_string(),
            PathBuf::from("test.json"),
            None,
            BatchOutputFormat::JsonLines,
            0, // Invalid max_tokens
            0.7,
            0.9,
            4,
            300,
            3,
            100,
            false,
            false,
            false,
            None,
            false,
            None,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Max tokens must be greater than 0"));
    }
}
