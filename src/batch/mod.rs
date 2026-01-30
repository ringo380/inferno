#![allow(dead_code, unused_imports, unused_variables)]
pub mod queue;
pub mod scheduler;

use crate::{
    backends::{Backend, InferenceParams},
    metrics::{InferenceEvent, MetricsCollector},
};
use anyhow::Result;
// Futures support for parallel processing (if needed in future)
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicUsize},
    time::{Duration, Instant},
};
// use tokio::sync::Semaphore; // Reserved for future concurrent processing
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    pub concurrency: usize,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub checkpoint_interval: u32,
    pub output_format: BatchOutputFormat,
    pub continue_on_error: bool,
    pub shuffle_inputs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchOutputFormat {
    JsonLines,
    Json,
    Csv,
    Tsv,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInput {
    pub id: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub id: String,
    pub input: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub tokens_generated: Option<u32>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    pub total_items: usize,
    pub completed_items: usize,
    pub failed_items: usize,
    pub skipped_items: usize,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
    pub current_rate: f64, // items per second
}

#[derive(Debug)]
pub struct BatchProcessor {
    config: BatchConfig,
    metrics: Option<Arc<MetricsCollector>>,
    progress: Arc<AtomicUsize>,
    total: usize,
    start_time: Instant,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            concurrency: 4,
            timeout_seconds: 300,
            retry_attempts: 3,
            checkpoint_interval: 100,
            output_format: BatchOutputFormat::JsonLines,
            continue_on_error: true,
            shuffle_inputs: false,
        }
    }
}

impl BatchProcessor {
    pub fn new(config: BatchConfig, total_items: usize) -> Self {
        Self {
            config,
            metrics: None,
            progress: Arc::new(AtomicUsize::new(0)),
            total: total_items,
            start_time: Instant::now(),
        }
    }

    pub fn with_metrics(mut self, metrics: Arc<MetricsCollector>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub async fn process_file(
        &self,
        backend: &mut Backend,
        input_path: &Path,
        output_path: Option<&Path>,
        inference_params: &InferenceParams,
    ) -> Result<BatchProgress> {
        let inputs = self.load_inputs(input_path).await?;
        self.process_inputs(backend, inputs, output_path, inference_params)
            .await
    }

    pub async fn process_inputs(
        &self,
        backend: &mut Backend,
        mut inputs: Vec<BatchInput>,
        output_path: Option<&Path>,
        inference_params: &InferenceParams,
    ) -> Result<BatchProgress> {
        if self.config.shuffle_inputs {
            use rand::seq::SliceRandom;
            inputs.shuffle(&mut rand::thread_rng());
        }

        let total_items = inputs.len();
        info!(
            "Starting batch processing of {} items (sequential mode)",
            total_items
        );

        let mut results = Vec::new();
        let start_time = chrono::Utc::now();
        let mut completed = 0;
        let mut failed = 0;

        for (i, input) in inputs.into_iter().enumerate() {
            if (i + 1) % 10 == 0 || i == 0 {
                info!("Processing item {}/{}", i + 1, total_items);
            }

            let result = Self::process_single_input_simple(
                backend,
                input,
                inference_params,
                self.metrics.clone(),
                "batch_model".to_string(),
                self.config.timeout_seconds,
                self.config.retry_attempts,
            )
            .await;

            if result.error.is_none() {
                completed += 1;
            } else {
                failed += 1;
                if !self.config.continue_on_error {
                    warn!("Stopping batch processing due to error (continue_on_error=false)");
                    break;
                }
            }

            results.push(result);

            // Checkpoint save
            if results.len() % self.config.checkpoint_interval as usize == 0 {
                if let Some(output_path) = output_path {
                    self.save_checkpoint(output_path, &results).await?;
                }
            }
        }

        // Final save
        if let Some(output_path) = output_path {
            self.save_results(output_path, &results).await?;
        }

        let elapsed = chrono::Utc::now() - start_time;
        let elapsed_seconds = elapsed.num_seconds().max(1);

        info!(
            "Batch processing completed: {}/{} items processed ({} failed) in {}",
            completed,
            total_items,
            failed,
            humantime::format_duration(elapsed.to_std().unwrap_or(Duration::ZERO))
        );

        Ok(BatchProgress {
            total_items,
            completed_items: completed,
            failed_items: failed,
            skipped_items: 0,
            start_time,
            estimated_completion: Some(chrono::Utc::now()),
            current_rate: completed as f64 / elapsed_seconds as f64,
        })
    }

    async fn process_single_input_simple(
        backend: &mut Backend,
        input: BatchInput,
        params: &InferenceParams,
        metrics: Option<Arc<MetricsCollector>>,
        model_name: String,
        timeout_seconds: u64,
        retry_attempts: u32,
    ) -> BatchResult {
        let start_time = Instant::now();
        let timestamp = chrono::Utc::now();

        for attempt in 0..=retry_attempts {
            match tokio::time::timeout(
                Duration::from_secs(timeout_seconds),
                backend.infer(&input.content, params),
            )
            .await
            {
                Ok(Ok(output)) => {
                    let duration = start_time.elapsed();

                    // Record metrics
                    if let Some(metrics) = &metrics {
                        let event = InferenceEvent {
                            model_name: model_name.clone(),
                            input_length: input.content.len() as u32,
                            output_length: output.len() as u32, // Rough estimate
                            duration,
                            success: true,
                        };
                        metrics.record_inference(event);
                    }

                    return BatchResult {
                        id: input.id,
                        input: input.content,
                        output: Some(output.clone()),
                        error: None,
                        duration_ms: duration.as_millis() as u64,
                        tokens_generated: Some((output.len() / 4) as u32), // Rough token estimate
                        timestamp,
                        metadata: input.metadata,
                    };
                }
                Ok(Err(e)) => {
                    warn!(
                        "Inference failed for item {}: {} (attempt {}/{})",
                        input.id,
                        e,
                        attempt + 1,
                        retry_attempts + 1
                    );
                    if attempt == retry_attempts {
                        // Record failed metrics
                        if let Some(metrics) = &metrics {
                            let event = InferenceEvent {
                                model_name: model_name.clone(),
                                input_length: input.content.len() as u32,
                                output_length: 0,
                                duration: start_time.elapsed(),
                                success: false,
                            };
                            metrics.record_inference(event);
                        }

                        return BatchResult {
                            id: input.id,
                            input: input.content,
                            output: None,
                            error: Some(e.to_string()),
                            duration_ms: start_time.elapsed().as_millis() as u64,
                            tokens_generated: None,
                            timestamp,
                            metadata: input.metadata,
                        };
                    }
                    tokio::time::sleep(Duration::from_millis(1000 * (attempt + 1) as u64)).await;
                }
                Err(_) => {
                    warn!(
                        "Timeout for item {} (attempt {}/{})",
                        input.id,
                        attempt + 1,
                        retry_attempts + 1
                    );
                    if attempt == retry_attempts {
                        return BatchResult {
                            id: input.id,
                            input: input.content,
                            output: None,
                            error: Some("Timeout".to_string()),
                            duration_ms: start_time.elapsed().as_millis() as u64,
                            tokens_generated: None,
                            timestamp,
                            metadata: input.metadata,
                        };
                    }
                }
            }
        }

        unreachable!()
    }

    pub async fn load_inputs(&self, input_path: &Path) -> Result<Vec<BatchInput>> {
        let content = tokio::fs::read_to_string(input_path).await?;
        let extension = input_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "json" => self.load_json_inputs(&content),
            "jsonl" | "ndjson" => self.load_jsonl_inputs(&content),
            "csv" => self.load_csv_inputs(&content).await,
            "tsv" => self.load_tsv_inputs(&content).await,
            _ => self.load_text_inputs(&content),
        }
    }

    fn load_json_inputs(&self, content: &str) -> Result<Vec<BatchInput>> {
        let value: serde_json::Value = serde_json::from_str(content)?;
        match value {
            serde_json::Value::Array(items) => {
                let mut inputs = Vec::new();
                for (i, item) in items.into_iter().enumerate() {
                    match item {
                        serde_json::Value::String(text) => {
                            inputs.push(BatchInput {
                                id: format!("item_{}", i),
                                content: text,
                                metadata: None,
                            });
                        }
                        serde_json::Value::Object(obj) => {
                            let content = obj
                                .get("content")
                                .or_else(|| obj.get("text"))
                                .or_else(|| obj.get("input"))
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| {
                                    anyhow::anyhow!("No content field found in JSON object")
                                })?
                                .to_string();

                            let id = obj
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&format!("item_{}", i))
                                .to_string();

                            inputs.push(BatchInput {
                                id,
                                content,
                                metadata: Some(serde_json::Value::Object(obj)),
                            });
                        }
                        _ => return Err(anyhow::anyhow!("Invalid JSON array item format")),
                    }
                }
                Ok(inputs)
            }
            _ => Err(anyhow::anyhow!("JSON must be an array")),
        }
    }

    fn load_jsonl_inputs(&self, content: &str) -> Result<Vec<BatchInput>> {
        let mut inputs = Vec::new();
        for (i, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let value: serde_json::Value = serde_json::from_str(line)?;
            match value {
                serde_json::Value::String(text) => {
                    inputs.push(BatchInput {
                        id: format!("line_{}", i + 1),
                        content: text,
                        metadata: None,
                    });
                }
                serde_json::Value::Object(obj) => {
                    let content = obj
                        .get("content")
                        .or_else(|| obj.get("text"))
                        .or_else(|| obj.get("input"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "No content field found in JSONL object at line {}",
                                i + 1
                            )
                        })?
                        .to_string();

                    let id = obj
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&format!("line_{}", i + 1))
                        .to_string();

                    inputs.push(BatchInput {
                        id,
                        content,
                        metadata: Some(serde_json::Value::Object(obj)),
                    });
                }
                _ => return Err(anyhow::anyhow!("Invalid JSONL format at line {}", i + 1)),
            }
        }
        Ok(inputs)
    }

    async fn load_csv_inputs(&self, content: &str) -> Result<Vec<BatchInput>> {
        self.load_delimited_inputs(content, ',').await
    }

    async fn load_tsv_inputs(&self, content: &str) -> Result<Vec<BatchInput>> {
        self.load_delimited_inputs(content, '\t').await
    }

    async fn load_delimited_inputs(
        &self,
        content: &str,
        delimiter: char,
    ) -> Result<Vec<BatchInput>> {
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(delimiter as u8)
            .from_reader(content.as_bytes());

        let headers = rdr.headers()?.clone();
        let mut inputs = Vec::new();

        for (i, result) in rdr.records().enumerate() {
            let record = result?;

            // Look for content in common column names
            let content = record
                .get(0)
                .or_else(|| {
                    headers
                        .iter()
                        .enumerate()
                        .find(|(_, h)| {
                            matches!(
                                h.to_lowercase().as_str(),
                                "content" | "text" | "input" | "prompt"
                            )
                        })
                        .and_then(|(idx, _)| record.get(idx))
                })
                .ok_or_else(|| anyhow::anyhow!("No content column found in CSV row {}", i + 1))?
                .to_string();

            // Create metadata from all columns
            let mut metadata = serde_json::Map::new();
            for (idx, value) in record.iter().enumerate() {
                if let Some(header) = headers.get(idx) {
                    metadata.insert(
                        header.to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
                }
            }

            let id = metadata
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or(&format!("row_{}", i + 1))
                .to_string();

            inputs.push(BatchInput {
                id,
                content,
                metadata: Some(serde_json::Value::Object(metadata)),
            });
        }

        Ok(inputs)
    }

    fn load_text_inputs(&self, content: &str) -> Result<Vec<BatchInput>> {
        let inputs = content
            .lines()
            .enumerate()
            .filter_map(|(i, line)| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(BatchInput {
                        id: format!("line_{}", i + 1),
                        content: trimmed.to_string(),
                        metadata: None,
                    })
                }
            })
            .collect();
        Ok(inputs)
    }

    async fn save_checkpoint(&self, output_path: &Path, results: &[BatchResult]) -> Result<()> {
        let checkpoint_path = output_path.with_extension(format!(
            "checkpoint.{}",
            output_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("json")
        ));
        self.save_results(&checkpoint_path, results).await
    }

    async fn save_results(&self, output_path: &Path, results: &[BatchResult]) -> Result<()> {
        let content = match self.config.output_format {
            BatchOutputFormat::Json => serde_json::to_string_pretty(results)?,
            BatchOutputFormat::JsonLines => results
                .iter()
                .map(serde_json::to_string)
                .collect::<Result<Vec<_>, _>>()?
                .join("\n"),
            BatchOutputFormat::Csv => self.results_to_csv(results)?,
            BatchOutputFormat::Tsv => self.results_to_tsv(results)?,
        };

        tokio::fs::write(output_path, content).await?;
        Ok(())
    }

    fn results_to_csv(&self, results: &[BatchResult]) -> Result<String> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        // Write header
        wtr.write_record([
            "id",
            "input",
            "output",
            "error",
            "duration_ms",
            "tokens_generated",
            "timestamp",
        ])?;

        // Write data
        for result in results {
            wtr.write_record([
                &result.id,
                &result.input,
                result.output.as_deref().unwrap_or(""),
                result.error.as_deref().unwrap_or(""),
                &result.duration_ms.to_string(),
                &result
                    .tokens_generated
                    .map(|t| t.to_string())
                    .unwrap_or_default(),
                &result.timestamp.to_rfc3339(),
            ])?;
        }

        let data = String::from_utf8(wtr.into_inner()?)?;
        Ok(data)
    }

    fn results_to_tsv(&self, results: &[BatchResult]) -> Result<String> {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(b'\t')
            .from_writer(vec![]);

        // Write header
        wtr.write_record([
            "id",
            "input",
            "output",
            "error",
            "duration_ms",
            "tokens_generated",
            "timestamp",
        ])?;

        // Write data
        for result in results {
            wtr.write_record([
                &result.id,
                &result.input,
                result.output.as_deref().unwrap_or(""),
                result.error.as_deref().unwrap_or(""),
                &result.duration_ms.to_string(),
                &result
                    .tokens_generated
                    .map(|t| t.to_string())
                    .unwrap_or_default(),
                &result.timestamp.to_rfc3339(),
            ])?;
        }

        let data = String::from_utf8(wtr.into_inner()?)?;
        Ok(data)
    }
}
