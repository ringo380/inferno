//! Run Command - New Architecture
//!
//! This module demonstrates the migration of the run command to the new
//! CLI architecture with Command trait, pipeline, and middleware support.
//!
//! Supports both single inference and batch processing modes.

use crate::backends::{Backend, BackendType, InferenceParams};
use crate::batch::{BatchConfig, BatchOutputFormat, BatchProcessor};
use crate::config::Config;
use crate::interfaces::cli::{Command, CommandContext, CommandOutput};
use crate::io::{InputFormat, OutputFormat};
use crate::models::ModelManager;
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use serde_json::json;
use std::path::PathBuf;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tracing::{info, warn};

// ============================================================================
// RunCommand - Main inference execution
// ============================================================================

/// Execute inference with a loaded model
pub struct RunCommand {
    config: Config,
    model: String,
    prompt: Option<String>,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    input_type: InputFormat,
    output_format: OutputFormat,
    max_tokens: u32,
    temperature: f32,
    top_p: f32,
    stream: bool,
    batch: bool,
    backend: Option<BackendType>,
}

impl RunCommand {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: Config,
        model: String,
        prompt: Option<String>,
        input: Option<PathBuf>,
        output: Option<PathBuf>,
        input_type: InputFormat,
        output_format: OutputFormat,
        max_tokens: u32,
        temperature: f32,
        top_p: f32,
        stream: bool,
        batch: bool,
        backend: Option<BackendType>,
    ) -> Self {
        Self {
            config,
            model,
            prompt,
            input,
            output,
            input_type,
            output_format,
            max_tokens,
            temperature,
            top_p,
            stream,
            batch,
            backend,
        }
    }
}

#[async_trait]
impl Command for RunCommand {
    fn name(&self) -> &str {
        "run"
    }

    fn description(&self) -> &str {
        "Execute inference with a loaded model"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // Validate model name
        if self.model.is_empty() {
            anyhow::bail!("Model name cannot be empty");
        }

        // Validate input requirements
        if self.prompt.is_none() && self.input.is_none() {
            anyhow::bail!("Either --prompt or --input must be provided");
        }

        // Validate batch mode requirements
        if self.batch && self.input.is_none() {
            anyhow::bail!("Batch mode requires --input file");
        }

        // Validate input file exists
        if let Some(ref input_path) = self.input {
            if !input_path.exists() {
                anyhow::bail!("Input file does not exist: {}", input_path.display());
            }
        }

        // Validate inference parameters
        if self.max_tokens == 0 {
            anyhow::bail!("max_tokens must be greater than 0");
        }

        if !(0.0..=2.0).contains(&self.temperature) {
            anyhow::bail!("temperature must be between 0.0 and 2.0");
        }

        if !(0.0..=1.0).contains(&self.top_p) {
            anyhow::bail!("top_p must be between 0.0 and 1.0");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Running inference with model: {}", self.model);

        // Resolve model
        let model_manager = ModelManager::new(&self.config.models_dir);
        let model_info = model_manager.resolve_model(&self.model).await?;

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

        // Load model
        let mut backend = Backend::new(backend_type, &self.config.backend_config)?;
        backend.load_model(&model_info).await?;

        // Execute based on mode
        if self.batch {
            self.execute_batch(&mut backend, ctx).await
        } else {
            self.execute_single(&mut backend, ctx).await
        }
    }
}

impl RunCommand {
    /// Execute batch processing
    async fn execute_batch(
        &self,
        backend: &mut Backend,
        ctx: &mut CommandContext,
    ) -> Result<CommandOutput> {
        let input_path = self
            .input
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Batch mode requires input file"))?;

        info!("Starting batch processing: {}", input_path.display());

        // Create batch configuration
        let batch_config = BatchConfig {
            concurrency: 1, // Single-threaded for compatibility
            timeout_seconds: 300,
            retry_attempts: 3,
            checkpoint_interval: 50,
            output_format: BatchOutputFormat::JsonLines,
            continue_on_error: true,
            shuffle_inputs: false,
        };

        // Estimate batch size
        let total_items = estimate_batch_size(input_path).await?;
        let processor = BatchProcessor::new(batch_config, total_items);

        // Create inference parameters
        let inference_params = InferenceParams {
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            top_p: self.top_p,
            stream: false,
            stop_sequences: vec![],
            seed: None,
        };

        // Process batch
        let progress = processor
            .process_file(
                backend,
                input_path,
                self.output.as_deref(),
                &inference_params,
            )
            .await?;

        let result_json = json!({
            "mode": "batch",
            "model": self.model,
            "backend": backend.get_backend_type().to_string(),
            "total_items": progress.total_items,
            "completed_items": progress.completed_items,
            "failed_items": progress.failed_items,
            "success_rate": if progress.total_items > 0 {
                (progress.completed_items as f64 / progress.total_items as f64) * 100.0
            } else {
                0.0
            },
            "output_file": self.output.as_ref().map(|p| p.display().to_string()),
        });

        if !ctx.json_output {
            println!("✓ Batch processing complete:");
            println!(
                "  Processed: {}/{}",
                progress.completed_items, progress.total_items
            );
            println!("  Failed: {}", progress.failed_items);
            if let Some(output) = &self.output {
                println!("  Output: {}", output.display());
            }
        }

        Ok(CommandOutput::success_with_data(
            format!(
                "Batch processing completed: {}/{} items",
                progress.completed_items, progress.total_items
            ),
            result_json,
        ))
    }

    /// Execute single inference
    async fn execute_single(
        &self,
        backend: &mut Backend,
        ctx: &mut CommandContext,
    ) -> Result<CommandOutput> {
        // Get input text
        let input_text = if let Some(ref prompt) = self.prompt {
            prompt.clone()
        } else if let Some(ref input_path) = self.input {
            tokio::fs::read_to_string(input_path).await?
        } else {
            // Read from stdin
            let stdin = io::stdin();
            let reader = BufReader::new(stdin);
            let mut lines = reader.lines();
            let mut input = String::new();

            while let Some(line) = lines.next_line().await? {
                input.push_str(&line);
                input.push('\n');
            }

            input
        };

        if input_text.is_empty() {
            anyhow::bail!("Input text is empty");
        }

        // Create inference parameters
        let params = InferenceParams {
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            top_p: self.top_p,
            stream: self.stream,
            stop_sequences: vec![],
            seed: None,
        };

        // Execute inference
        let start = std::time::Instant::now();
        let response = if self.stream {
            self.execute_streaming(backend, &input_text, &params, ctx)
                .await?
        } else {
            backend.infer(&input_text, &params).await?
        };
        let elapsed = start.elapsed();

        // Get metrics
        let metrics = backend.get_metrics();

        // Write output
        if let Some(output_path) = &self.output {
            tokio::fs::write(output_path, &response).await?;
            info!("Output written to: {}", output_path.display());
        } else if !ctx.json_output && !self.stream {
            println!("{}", response);
        }

        // Build result
        let result_json = json!({
            "mode": "single",
            "model": self.model,
            "backend": backend.get_backend_type().to_string(),
            "input_length": input_text.len(),
            "output_length": response.len(),
            "elapsed_ms": elapsed.as_millis(),
            "streaming": self.stream,
            "metrics": metrics.map(|m| json!({
                "total_tokens": m.total_tokens,
                "prompt_tokens": m.prompt_tokens,
                "completion_tokens": m.completion_tokens,
                "tokens_per_second": m.tokens_per_second,
            })),
            "output_file": self.output.as_ref().map(|p| p.display().to_string()),
        });

        Ok(CommandOutput::success_with_data(
            format!("Inference completed in {:.2}s", elapsed.as_secs_f64()),
            result_json,
        ))
    }

    /// Execute streaming inference
    async fn execute_streaming(
        &self,
        backend: &mut Backend,
        input: &str,
        params: &InferenceParams,
        ctx: &CommandContext,
    ) -> Result<String> {
        let mut stream = backend.infer_stream(input, params).await?;
        let mut full_response = String::new();

        while let Some(token_result) = stream.next().await {
            match token_result {
                Ok(token) => {
                    if !ctx.json_output {
                        print!("{}", token);
                        use std::io::Write;
                        std::io::stdout().flush()?;
                    }
                    full_response.push_str(&token);
                }
                Err(e) => {
                    warn!("Error during streaming: {}", e);
                    break;
                }
            }
        }

        if !ctx.json_output {
            println!(); // Newline after streaming
        }

        Ok(full_response)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

async fn estimate_batch_size(input_path: &std::path::Path) -> Result<usize> {
    let content = tokio::fs::read_to_string(input_path).await?;
    let extension = input_path
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_estimate_batch_size_jsonl() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let content = r#"{"text": "line 1"}
{"text": "line 2"}
{"text": "line 3"}"#;
        tokio::fs::write(path, content).await.unwrap();

        // Rename to .jsonl
        let jsonl_path = path.with_extension("jsonl");
        tokio::fs::rename(path, &jsonl_path).await.unwrap();

        let size = estimate_batch_size(&jsonl_path).await.unwrap();
        assert_eq!(size, 3);

        tokio::fs::remove_file(&jsonl_path).await.ok();
    }

    #[tokio::test]
    async fn test_estimate_batch_size_json_array() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let content = r#"[
            {"text": "item 1"},
            {"text": "item 2"}
        ]"#;
        tokio::fs::write(path, content).await.unwrap();

        let json_path = path.with_extension("json");
        tokio::fs::rename(path, &json_path).await.unwrap();

        let size = estimate_batch_size(&json_path).await.unwrap();
        assert_eq!(size, 2);

        tokio::fs::remove_file(&json_path).await.ok();
    }
}
