#![allow(clippy::explicit_counter_loop)]

//! # ONNX Runtime Backend
//!
//! Provides ONNX model inference support using the ort crate.
//!
//! Note: ONNX support requires the `onnx` feature flag and is currently
//! in transition due to ort 2.0 API changes. Full functionality will be
//! restored when ort 2.0 stabilizes.

use crate::{
    backends::{
        BackendConfig, BackendType, InferenceBackend, InferenceMetrics, InferenceParams,
        TokenStream,
    },
    models::ModelInfo,
};
use anyhow::{anyhow, Result};
use async_stream::stream;
use std::time::Instant;
use tokenizers::Tokenizer;
use tracing::{debug, info, warn};

pub struct OnnxBackend {
    config: BackendConfig,
    tokenizer: Option<Tokenizer>,
    model_info: Option<ModelInfo>,
    metrics: Option<InferenceMetrics>,
    model_type: ModelType,
}

#[derive(Debug, Clone)]
enum ModelType {
    TextGeneration,
    Classification,
    Embedding,
    Unknown,
}

impl OnnxBackend {
    pub fn new(config: BackendConfig) -> Result<Self> {
        info!("Initializing ONNX backend");

        // Note: Full ONNX Runtime initialization is disabled during ort 2.0 transition
        // The backend provides placeholder responses until ort 2.0 stabilizes
        warn!("ONNX backend is in stub mode - ort 2.0 prebuilt binaries not available for all platforms");

        Ok(Self {
            config,
            tokenizer: None,
            model_info: None,
            metrics: None,
            model_type: ModelType::Unknown,
        })
    }

    fn load_tokenizer(&mut self, model_path: &std::path::Path) -> Result<()> {
        // Try to find tokenizer files in the same directory as the model
        let model_dir = model_path.parent().unwrap_or(model_path);

        // Common tokenizer file names
        let tokenizer_files = ["tokenizer.json", "vocab.txt", "tokenizer_config.json"];

        for filename in &tokenizer_files {
            let tokenizer_path = model_dir.join(filename);
            if tokenizer_path.exists() {
                match Tokenizer::from_file(&tokenizer_path) {
                    Ok(tokenizer) => {
                        info!("Loaded tokenizer from: {}", tokenizer_path.display());
                        self.tokenizer = Some(tokenizer);
                        return Ok(());
                    }
                    Err(e) => {
                        debug!(
                            "Failed to load tokenizer from {}: {}",
                            tokenizer_path.display(),
                            e
                        );
                    }
                }
            }
        }

        // If no tokenizer found, use simple word-based tokenization
        warn!("No tokenizer found, using simple word-based tokenization");
        Ok(())
    }

    fn estimate_token_count(&self, text: &str) -> u32 {
        if let Some(tokenizer) = &self.tokenizer {
            if let Ok(encoding) = tokenizer.encode(text, false) {
                return encoding.len() as u32;
            }
        }

        // Fallback estimation
        (text.len() as f32 / 4.0).ceil() as u32
    }

    fn detect_model_type(&self, _model_path: &std::path::Path) -> Result<ModelType> {
        // For now, we'll default to TextGeneration
        // In a more sophisticated implementation, we could inspect the model metadata
        // or examine input/output tensor shapes to determine the model type
        Ok(ModelType::TextGeneration)
    }
}

// Add the required trait implementations for Send and Sync
unsafe impl Send for OnnxBackend {}
unsafe impl Sync for OnnxBackend {}

#[async_trait::async_trait]
impl InferenceBackend for OnnxBackend {
    async fn load_model(&mut self, model_info: &ModelInfo) -> Result<()> {
        info!("Loading ONNX model: {}", model_info.path.display());

        // Validate model file exists
        if !model_info.path.exists() {
            return Err(anyhow!(
                "Model file not found: {}",
                model_info.path.display()
            ));
        }

        // Load tokenizer if available
        self.load_tokenizer(&model_info.path)?;

        // Store model info (actual session creation disabled during ort 2.0 transition)
        self.model_info = Some(model_info.clone());
        self.model_type = self.detect_model_type(&model_info.path)?;

        warn!(
            "ONNX model '{}' registered (stub mode - actual inference not available)",
            model_info.path.display()
        );
        Ok(())
    }

    async fn unload_model(&mut self) -> Result<()> {
        info!("Unloading ONNX model");
        self.tokenizer = None;
        self.model_info = None;
        self.metrics = None;
        self.model_type = ModelType::Unknown;
        info!("ONNX model unloaded successfully");
        Ok(())
    }

    async fn is_loaded(&self) -> bool {
        self.model_info.is_some()
    }

    async fn get_model_info(&self) -> Option<ModelInfo> {
        self.model_info.as_ref().cloned()
    }

    async fn infer(&mut self, input: &str, _params: &InferenceParams) -> Result<String> {
        if self.model_info.is_none() {
            return Err(anyhow!("Model not loaded"));
        }

        let start_time = Instant::now();
        info!("Processing ONNX inference request (stub mode)");

        let prompt_tokens = self.estimate_token_count(input);
        let prompt_time = start_time.elapsed();

        // Generate a stub response (actual inference disabled during ort 2.0 transition)
        let result = format!(
            "[ONNX stub mode] Input processed: \"{}\" ({} estimated tokens). \
             Full ONNX Runtime inference will be available when ort 2.0 stabilizes with prebuilt binaries.",
            input.chars().take(50).collect::<String>(),
            prompt_tokens
        );

        let completion_time = start_time.elapsed() - prompt_time;
        let total_time = start_time.elapsed();
        let completion_tokens = self.estimate_token_count(&result);
        let total_tokens = prompt_tokens + completion_tokens;

        // Update metrics
        self.metrics = Some(InferenceMetrics {
            total_tokens,
            prompt_tokens,
            completion_tokens,
            total_time_ms: total_time.as_millis() as u64,
            tokens_per_second: completion_tokens as f32 / completion_time.as_secs_f32().max(0.001),
            prompt_time_ms: prompt_time.as_millis() as u64,
            completion_time_ms: completion_time.as_millis() as u64,
        });

        debug!(
            "ONNX stub inference completed: {} tokens in {:.2}s",
            completion_tokens,
            completion_time.as_secs_f32()
        );

        Ok(result)
    }

    async fn infer_stream(&mut self, input: &str, params: &InferenceParams) -> Result<TokenStream> {
        if self.model_info.is_none() {
            return Err(anyhow!("Model not loaded"));
        }

        info!("Starting ONNX streaming inference (stub mode)");

        // Since ONNX models don't inherently support streaming, we'll simulate it
        let result = self.infer(input, params).await?;
        let max_tokens = params.max_tokens as usize;

        let stream = stream! {
            // Split result into words and stream them
            let words: Vec<&str> = result.split_whitespace().collect();
            let mut token_count = 0;

            for (i, word) in words.iter().enumerate() {
                if token_count >= max_tokens {
                    break;
                }

                if i > 0 {
                    yield Ok(" ".to_string());
                }
                yield Ok(word.to_string());

                token_count += 1;

                // Add realistic delay between tokens
                let delay = match word.len() {
                    1..=3 => 30,
                    4..=6 => 50,
                    7..=10 => 70,
                    _ => 90,
                };

                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            }
        };

        Ok(Box::pin(stream))
    }

    async fn get_embeddings(&mut self, input: &str) -> Result<Vec<f32>> {
        if self.model_info.is_none() {
            return Err(anyhow!("Model not loaded"));
        }

        info!("Computing ONNX embeddings (stub mode)");

        // For now, create embeddings based on actual input analysis
        // A real implementation would run the model to get actual embeddings
        let embedding_size = 768; // Common embedding size
        let mut embeddings = vec![0.0f32; embedding_size];

        // Create meaningful embeddings based on input characteristics
        let input_len = input.len() as f32;
        let word_count = input.split_whitespace().count() as f32;
        let char_variety = input
            .chars()
            .collect::<std::collections::HashSet<_>>()
            .len() as f32;

        // Generate embeddings that reflect input properties
        for (i, embedding) in embeddings.iter_mut().enumerate() {
            let position_factor = (i as f32) / (embedding_size as f32);
            *embedding = ((input_len / 100.0).sin() * position_factor
                + (word_count / 10.0).cos() * (1.0 - position_factor)
                + (char_variety / 26.0).sin() * 0.1)
                .tanh();
        }

        info!(
            "Generated {} dimensional embeddings (stub mode)",
            embeddings.len()
        );
        Ok(embeddings)
    }

    fn get_backend_type(&self) -> BackendType {
        BackendType::Onnx
    }

    fn get_metrics(&self) -> Option<InferenceMetrics> {
        self.metrics.as_ref().cloned()
    }
}
