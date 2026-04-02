//! # ONNX Runtime Backend
//!
//! Provides ONNX model inference support using the ort crate with load-dynamic
//! feature for cross-platform ONNX Runtime loading.
//!
//! Requires the `onnx` feature flag and `libonnxruntime` installed or
//! `ORT_DYLIB_PATH` set at runtime.

use crate::{
    InfernoError,
    ai_features::{
        sampling::{Sampler, SamplingConfig, SamplingStrategy},
        streaming::{StreamConfig, StreamToken, create_stream_channel},
    },
    backends::{
        BackendConfig, BackendType, InferenceBackend, InferenceMetrics, InferenceParams,
        TokenStream,
    },
    models::ModelInfo,
};
use anyhow::{Result, anyhow};
use async_stream::stream;
use ort::{
    inputs,
    session::{Session, builder::GraphOptimizationLevel},
    value::Tensor,
};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokenizers::Tokenizer;
use tracing::{debug, info, warn};

pub struct OnnxBackend {
    config: BackendConfig,
    session: Option<Arc<Mutex<Session>>>,
    tokenizer: Option<Tokenizer>,
    model_info: Option<ModelInfo>,
    metrics: Arc<Mutex<Option<InferenceMetrics>>>,
    model_type: ModelType,
    input_names: InputNames,
    eos_token_id: Option<u32>,
}

#[derive(Debug, Clone)]
enum ModelType {
    TextGeneration,
    Classification,
    Embedding,
    Unknown,
}

#[derive(Debug, Clone)]
struct InputNames {
    input_ids: String,
    attention_mask: Option<String>,
}

impl Default for InputNames {
    fn default() -> Self {
        Self {
            input_ids: "input_ids".to_string(),
            attention_mask: Some("attention_mask".to_string()),
        }
    }
}

impl OnnxBackend {
    pub fn new(config: BackendConfig) -> Result<Self> {
        info!("Initializing ONNX backend (load-dynamic mode)");

        Ok(Self {
            config,
            session: None,
            tokenizer: None,
            model_info: None,
            metrics: Arc::new(Mutex::new(None)),
            model_type: ModelType::Unknown,
            input_names: InputNames::default(),
            eos_token_id: None,
        })
    }

    fn load_tokenizer(&mut self, model_path: &std::path::Path) -> Result<()> {
        let model_dir = model_path.parent().unwrap_or(model_path);

        let tokenizer_files = ["tokenizer.json", "vocab.txt", "tokenizer_config.json"];

        for filename in &tokenizer_files {
            let tokenizer_path = model_dir.join(filename);
            if tokenizer_path.exists() {
                match Tokenizer::from_file(&tokenizer_path) {
                    Ok(tokenizer) => {
                        info!("Loaded tokenizer from: {}", tokenizer_path.display());
                        self.tokenizer = Some(tokenizer);
                        // Try to detect EOS token ID from tokenizer config
                        self.detect_eos_token(model_dir);
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

        warn!(
            "No tokenizer found alongside model, tokenization-dependent features will be limited"
        );
        Ok(())
    }

    fn detect_eos_token(&mut self, model_dir: &std::path::Path) {
        // Try reading tokenizer_config.json for eos_token_id
        let config_path = model_dir.join("tokenizer_config.json");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                // Simple JSON parse for eos_token_id
                if let Some(pos) = content.find("\"eos_token_id\"") {
                    let rest = &content[pos..];
                    if let Some(colon) = rest.find(':') {
                        let value_str = rest[colon + 1..].trim();
                        if let Some(end) = value_str.find(|c: char| !c.is_ascii_digit()) {
                            if let Ok(id) = value_str[..end].trim().parse::<u32>() {
                                self.eos_token_id = Some(id);
                                debug!("Detected EOS token ID: {}", id);
                                return;
                            }
                        }
                    }
                }
            }
        }

        // Try common special_tokens_map.json
        let special_path = model_dir.join("special_tokens_map.json");
        if special_path.exists() {
            debug!("Found special_tokens_map.json but EOS extraction requires full JSON parsing");
        }

        // Default EOS token ID for many HuggingFace models
        self.eos_token_id = Some(2);
        debug!("Using default EOS token ID: 2");
    }

    /// Validate ONNX file format by checking protobuf structure.
    fn validate_onnx_file(path: &std::path::Path) -> Result<()> {
        use std::io::Read;
        let mut file = std::fs::File::open(path)
            .map_err(|e| InfernoError::Backend(format!("Cannot open model file: {}", e)))?;

        let mut header = [0u8; 8];
        file.read_exact(&mut header)
            .map_err(|e| InfernoError::Backend(format!("Cannot read model file header: {}", e)))?;

        // ONNX files are Protocol Buffers. Check for valid protobuf wire types
        // in the first few bytes. The first byte should have a valid field number
        // and wire type (0-5).
        let wire_type = header[0] & 0x07;
        if wire_type > 5 {
            return Err(InfernoError::Backend(
                "File does not appear to be a valid ONNX model (invalid protobuf header)"
                    .to_string(),
            )
            .into());
        }

        // Check that the file contains ONNX-related strings in the header region
        let mut extended_header = vec![
            0u8;
            512.min(
                std::fs::metadata(path)
                    .map(|m| m.len() as usize)
                    .unwrap_or(512),
            )
        ];
        file = std::fs::File::open(path)
            .map_err(|e| InfernoError::Backend(format!("Cannot reopen model file: {}", e)))?;
        let bytes_read = file
            .read(&mut extended_header)
            .map_err(|e| InfernoError::Backend(format!("Cannot read model header: {}", e)))?;
        extended_header.truncate(bytes_read);

        let header_str = String::from_utf8_lossy(&extended_header);
        let has_onnx_markers = header_str.contains("onnx")
            || header_str.contains("ONNX")
            || header_str.contains("ir_version")
            || header_str.contains("graph")
            || header_str.contains("producer");

        if !has_onnx_markers {
            warn!("ONNX file header does not contain typical ONNX markers, proceeding anyway");
        }

        Ok(())
    }

    fn tokenize(&self, text: &str) -> Result<Vec<u32>> {
        let tokenizer = self.tokenizer.as_ref().ok_or_else(|| {
            anyhow!("No tokenizer available. Place tokenizer.json alongside the model file.")
        })?;
        let encoding = tokenizer
            .encode(text, false)
            .map_err(|e| anyhow!("Tokenization failed: {}", e))?;
        Ok(encoding.get_ids().to_vec())
    }

    fn detokenize(&self, token_ids: &[u32]) -> Result<String> {
        let tokenizer = self
            .tokenizer
            .as_ref()
            .ok_or_else(|| anyhow!("No tokenizer available"))?;
        tokenizer
            .decode(token_ids, true)
            .map_err(|e| anyhow!("Detokenization failed: {}", e))
    }

    fn estimate_token_count(&self, text: &str) -> u32 {
        if let Some(tokenizer) = &self.tokenizer {
            if let Ok(encoding) = tokenizer.encode(text, false) {
                return encoding.len() as u32;
            }
        }
        (text.len() as f32 / 4.0).ceil() as u32
    }

    fn build_execution_providers(&self) -> Vec<ort::ep::ExecutionProviderDispatch> {
        let mut providers = Vec::new();

        if self.config.gpu_enabled {
            #[cfg(target_os = "macos")]
            {
                providers.push(ort::ep::CoreML::default().with_subgraphs(true).build());
                debug!("Added CoreML execution provider");
            }

            #[cfg(feature = "cuda")]
            {
                let cuda = ort::ep::CUDA::default();
                providers.push(cuda.build());
                debug!("Added CUDA execution provider");
            }

            #[cfg(target_os = "windows")]
            {
                providers.push(ort::ep::DirectML::default().build());
                debug!("Added DirectML execution provider");
            }
        }

        providers
    }

    fn discover_input_names(session: &Session) -> InputNames {
        let inputs = session.inputs();

        let input_ids = inputs
            .iter()
            .find(|i| {
                let name = i.name().to_lowercase();
                name.contains("input_id")
            })
            .or(inputs.first())
            .map(|i| i.name().to_string())
            .unwrap_or_else(|| "input_ids".to_string());

        let attention_mask = inputs
            .iter()
            .find(|i| {
                let name = i.name().to_lowercase();
                name.contains("attention_mask") || name.contains("attn_mask")
            })
            .map(|i| i.name().to_string());

        debug!(
            "Discovered input names: input_ids='{}', attention_mask={:?}",
            input_ids, attention_mask
        );

        InputNames {
            input_ids,
            attention_mask,
        }
    }

    fn detect_model_type(session: &Session) -> ModelType {
        let outputs = session.outputs();

        if outputs.is_empty() {
            return ModelType::Unknown;
        }

        // Check output names for hints
        for output in outputs {
            let name = output.name().to_lowercase();
            if name.contains("logits") {
                debug!("Detected TextGeneration model (output name contains 'logits')");
                return ModelType::TextGeneration;
            }
            if name.contains("embed")
                || name.contains("hidden_state")
                || name.contains("last_hidden")
            {
                debug!("Detected Embedding model (output name contains embedding-related term)");
                return ModelType::Embedding;
            }
        }

        debug!("Could not determine model type from output names, defaulting to TextGeneration");
        ModelType::TextGeneration
    }

    fn build_sampling_config(params: &InferenceParams) -> SamplingConfig {
        let strategy = if params.temperature.abs() < 0.01 {
            SamplingStrategy::Greedy
        } else {
            SamplingStrategy::TopKP
        };

        SamplingConfig {
            strategy,
            temperature: params.temperature.max(0.1).min(2.0),
            top_k: params.top_k.max(1),
            top_p: params.top_p.max(0.0).min(1.0),
            repeat_penalty: 1.1,
            seed: params.seed,
        }
    }

    /// Compute softmax probabilities from logits.
    fn softmax(logits: &[f32]) -> Vec<f32> {
        if logits.is_empty() {
            return Vec::new();
        }
        let max_logit = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let exps: Vec<f32> = logits.iter().map(|&l| (l - max_logit).exp()).collect();
        let sum: f32 = exps.iter().sum();
        if sum <= 0.0 {
            return vec![0.0; logits.len()];
        }
        exps.iter().map(|&e| e / sum).collect()
    }

    /// Run a single forward pass and return raw logits for the last token position.
    /// Uses shape tuples to avoid ndarray version conflicts between ort and the project.
    fn forward_pass(
        session: &mut Session,
        token_ids: &[i64],
        input_names: &InputNames,
    ) -> Result<Vec<f32>> {
        let seq_len = token_ids.len();
        let attention_mask_data: Vec<i64> = vec![1i64; seq_len];

        let input_ids_tensor = Tensor::from_array(([1usize, seq_len], token_ids.to_vec()))
            .map_err(|e| anyhow!("Failed to create input_ids tensor: {}", e))?;

        let outputs = if let Some(ref mask_name) = input_names.attention_mask {
            let attention_mask_tensor =
                Tensor::from_array(([1usize, seq_len], attention_mask_data))
                    .map_err(|e| anyhow!("Failed to create attention_mask tensor: {}", e))?;
            session
                .run(inputs![
                    input_names.input_ids.as_str() => input_ids_tensor,
                    mask_name.as_str() => attention_mask_tensor
                ])
                .map_err(|e| InfernoError::Backend(format!("ONNX inference failed: {}", e)))?
        } else {
            session
                .run(inputs![
                    input_names.input_ids.as_str() => input_ids_tensor
                ])
                .map_err(|e| InfernoError::Backend(format!("ONNX inference failed: {}", e)))?
        };

        // Extract logits from first output using try_extract_tensor (avoids ndarray version conflict)
        let output_value = &outputs[0usize];
        let (shape, data) = output_value
            .try_extract_tensor::<f32>()
            .map_err(|e| anyhow!("Failed to extract output tensor: {}", e))?;

        // Get logits for the last position
        // Typical shapes: [batch, seq_len, vocab_size] or [batch, vocab_size]
        let last_logits = if shape.len() == 3 {
            let seq_dim = shape[1] as usize;
            let vocab_size = shape[2] as usize;
            if seq_dim == 0 || vocab_size == 0 {
                return Err(anyhow!(
                    "Model returned empty output tensor (shape: {:?})",
                    shape
                ));
            }
            let last_pos = seq_dim - 1;
            let start = last_pos * vocab_size;
            let end = start + vocab_size;
            if end > data.len() {
                return Err(anyhow!(
                    "Output tensor data too short for shape {:?} (got {} elements)",
                    shape,
                    data.len()
                ));
            }
            data[start..end].to_vec()
        } else if shape.len() == 2 {
            let vocab_size = shape[1] as usize;
            if vocab_size == 0 || vocab_size > data.len() {
                return Err(anyhow!(
                    "Output tensor data size mismatch for shape {:?} (got {} elements)",
                    shape,
                    data.len()
                ));
            }
            data[..vocab_size].to_vec()
        } else {
            data.to_vec()
        };

        Ok(last_logits)
    }

    /// Autoregressive text generation (blocking, meant for spawn_blocking)
    fn generate_text_blocking(
        session: &mut Session,
        initial_tokens: Vec<i64>,
        input_names: &InputNames,
        params: &InferenceParams,
        eos_token_id: Option<u32>,
        tokenizer: Option<&Tokenizer>,
    ) -> Result<Vec<u32>> {
        let mut all_tokens = initial_tokens.clone();
        let mut sampler = Sampler::new(Self::build_sampling_config(params));

        for _ in 0..params.max_tokens {
            let logits = Self::forward_pass(session, &all_tokens, input_names)?;

            // Compute softmax probabilities so sampling strategies (greedy, top-k, top-p) work correctly
            let probs = Self::softmax(&logits);
            let candidates: Vec<(i32, f32, f32)> = logits
                .iter()
                .zip(probs.iter())
                .enumerate()
                .map(|(id, (&logit, &p))| (id as i32, logit, p))
                .collect();

            let next_token = match sampler.sample_from_candidates(&candidates) {
                Some(token) => token,
                None => break,
            };

            // Check for EOS token
            if let Some(eos_id) = eos_token_id {
                if next_token as u32 == eos_id {
                    debug!("EOS token encountered, stopping generation");
                    break;
                }
            }

            all_tokens.push(next_token as i64);

            // Check stop sequences
            if !params.stop_sequences.is_empty() {
                if let Some(tok) = tokenizer {
                    let generated: Vec<u32> = all_tokens[initial_tokens.len()..]
                        .iter()
                        .map(|&t| t as u32)
                        .collect();
                    if let Ok(text) = tok.decode(&generated, true) {
                        if params.stop_sequences.iter().any(|stop| text.contains(stop)) {
                            debug!("Stop sequence matched, stopping generation");
                            break;
                        }
                    }
                }
            }
        }

        // Return only the generated tokens (not the prompt)
        Ok(all_tokens[initial_tokens.len()..]
            .iter()
            .map(|&t| t as u32)
            .collect())
    }
}

#[async_trait::async_trait]
impl InferenceBackend for OnnxBackend {
    async fn load_model(&mut self, model_info: &ModelInfo) -> Result<()> {
        info!("Loading ONNX model: {}", model_info.path.display());

        if !model_info.path.exists() {
            return Err(InfernoError::Backend(format!(
                "Model file not found: {}",
                model_info.path.display()
            ))
            .into());
        }

        let file_size = std::fs::metadata(&model_info.path)
            .map_err(|e| InfernoError::Backend(format!("Cannot read model file metadata: {}", e)))?
            .len();

        if file_size < 64 {
            return Err(InfernoError::Backend(
                "Model file appears too small to be a valid ONNX model".to_string(),
            )
            .into());
        }

        debug!("ONNX model file size: {} bytes", file_size);

        // Validate ONNX file format (protobuf header check)
        Self::validate_onnx_file(&model_info.path)?;

        let providers = self.build_execution_providers();
        let cpu_threads = self.config.cpu_threads;
        let model_path = model_info.path.clone();

        let session = tokio::task::spawn_blocking(move || -> Result<Session> {
            let mut builder = Session::builder()
                .map_err(|e| {
                    InfernoError::Backend(format!("Failed to create session builder: {}", e))
                })?
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .map_err(|e| {
                    InfernoError::Backend(format!("Failed to set optimization level: {}", e))
                })?;

            if let Some(threads) = cpu_threads {
                builder = builder.with_intra_threads(threads as usize).map_err(|e| {
                    InfernoError::Backend(format!("Failed to set thread count: {}", e))
                })?;
            }

            if !providers.is_empty() {
                builder = builder.with_execution_providers(providers).map_err(|e| {
                    InfernoError::Backend(format!("Failed to set execution providers: {}", e))
                })?;
            }

            builder.commit_from_file(&model_path).map_err(|e| {
                InfernoError::Backend(format!("Failed to load ONNX model: {}", e)).into()
            })
        })
        .await
        .map_err(|e| InfernoError::Backend(format!("Model loading task failed: {}", e)))??;

        // Discover input tensor names and model type
        self.input_names = Self::discover_input_names(&session);
        self.model_type = Self::detect_model_type(&session);

        for input in session.inputs() {
            info!("  Model input: {} ({:?})", input.name(), input.dtype());
        }
        for output in session.outputs() {
            info!("  Model output: {} ({:?})", output.name(), output.dtype());
        }

        self.session = Some(Arc::new(Mutex::new(session)));
        self.load_tokenizer(&model_info.path)?;
        self.model_info = Some(model_info.clone());

        info!(
            "ONNX model loaded successfully (type: {:?}, GPU: {})",
            self.model_type, self.config.gpu_enabled
        );

        // Best-effort: record usage in the local model registry
        crate::models::record_model_usage(&model_info.path).await;

        Ok(())
    }

    async fn unload_model(&mut self) -> Result<()> {
        info!("Unloading ONNX model");
        self.session = None;
        self.tokenizer = None;
        self.model_info = None;
        *self.metrics.lock().unwrap() = None;
        self.model_type = ModelType::Unknown;
        self.input_names = InputNames::default();
        self.eos_token_id = None;
        info!("ONNX model unloaded successfully");
        Ok(())
    }

    async fn is_loaded(&self) -> bool {
        self.session.is_some() && self.model_info.is_some()
    }

    async fn get_model_info(&self) -> Option<ModelInfo> {
        self.model_info.as_ref().cloned()
    }

    async fn infer(&mut self, input: &str, params: &InferenceParams) -> Result<String> {
        if !self.is_loaded().await {
            return Err(InfernoError::Backend("Model not loaded".to_string()).into());
        }

        let start_time = Instant::now();
        info!("Starting ONNX inference");

        let session = self.session.as_ref().unwrap().clone();
        let input_names = self.input_names.clone();

        let metrics = self.metrics.clone();

        match self.model_type {
            ModelType::TextGeneration | ModelType::Unknown => {
                let token_ids = self.tokenize(input)?;
                let prompt_tokens = token_ids.len() as u32;
                let prompt_time = start_time.elapsed();

                let initial_tokens: Vec<i64> = token_ids.iter().map(|&t| t as i64).collect();
                let params_clone = params.clone();
                let eos_token_id = self.eos_token_id;
                let tokenizer = self.tokenizer.clone();

                let generated_tokens = tokio::task::spawn_blocking(move || {
                    let mut session = session
                        .lock()
                        .map_err(|e| anyhow!("Session lock poisoned: {}", e))?;
                    Self::generate_text_blocking(
                        &mut session,
                        initial_tokens,
                        &input_names,
                        &params_clone,
                        eos_token_id,
                        tokenizer.as_ref(),
                    )
                })
                .await
                .map_err(|e| InfernoError::Backend(format!("Inference task failed: {}", e)))??;

                let completion_time = start_time.elapsed() - prompt_time;
                let total_time = start_time.elapsed();
                let completion_tokens = generated_tokens.len() as u32;

                let response = self.detokenize(&generated_tokens)?;

                *metrics.lock().unwrap() = Some(InferenceMetrics {
                    total_tokens: prompt_tokens + completion_tokens,
                    prompt_tokens,
                    completion_tokens,
                    total_time_ms: total_time.as_millis() as u64,
                    tokens_per_second: if completion_time.as_secs_f32() > 0.0 {
                        completion_tokens as f32 / completion_time.as_secs_f32()
                    } else {
                        0.0
                    },
                    prompt_time_ms: prompt_time.as_millis() as u64,
                    completion_time_ms: completion_time.as_millis() as u64,
                });

                info!(
                    "ONNX inference completed: {} tokens in {:.2}s ({:.1} tok/s)",
                    completion_tokens,
                    completion_time.as_secs_f32(),
                    completion_tokens as f32 / completion_time.as_secs_f32().max(0.001)
                );

                Ok(response)
            }
            ModelType::Classification => {
                let token_ids = self.tokenize(input)?;
                let prompt_tokens = token_ids.len() as u32;
                let input_i64: Vec<i64> = token_ids.iter().map(|&t| t as i64).collect();

                let scores = tokio::task::spawn_blocking(move || {
                    let mut session = session
                        .lock()
                        .map_err(|e| anyhow!("Session lock poisoned: {}", e))?;
                    Self::forward_pass(&mut session, &input_i64, &input_names)
                })
                .await
                .map_err(|e| InfernoError::Backend(format!("Inference task failed: {}", e)))??;

                let total_time = start_time.elapsed();

                let mut indexed: Vec<(usize, f32)> =
                    scores.iter().enumerate().map(|(i, &s)| (i, s)).collect();
                indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                indexed.truncate(5);

                let response = indexed
                    .iter()
                    .map(|(idx, score)| format!("class_{}: {:.4}", idx, score))
                    .collect::<Vec<_>>()
                    .join(", ");

                *metrics.lock().unwrap() = Some(InferenceMetrics {
                    total_tokens: prompt_tokens,
                    prompt_tokens,
                    completion_tokens: 0,
                    total_time_ms: total_time.as_millis() as u64,
                    tokens_per_second: 0.0,
                    prompt_time_ms: total_time.as_millis() as u64,
                    completion_time_ms: 0,
                });

                Ok(response)
            }
            ModelType::Embedding => {
                let embeddings = self.get_embeddings(input).await?;
                Ok(format!(
                    "[{} dimensional embedding vector]",
                    embeddings.len()
                ))
            }
        }
    }

    async fn infer_stream(&mut self, input: &str, params: &InferenceParams) -> Result<TokenStream> {
        if !self.is_loaded().await {
            return Err(InfernoError::Backend("Model not loaded".to_string()).into());
        }

        info!("Starting ONNX streaming inference");

        let session = self.session.as_ref().unwrap().clone();
        let input_names = self.input_names.clone();

        let token_ids = self.tokenize(input)?;
        let prompt_tokens = token_ids.len() as u32;
        let initial_tokens: Vec<i64> = token_ids.iter().map(|&t| t as i64).collect();
        let max_tokens = params.max_tokens;
        let sampling_config = Self::build_sampling_config(params);
        let stop_sequences = params.stop_sequences.clone();
        let eos_token_id = self.eos_token_id;

        let tokenizer = self
            .tokenizer
            .as_ref()
            .ok_or_else(|| anyhow!("No tokenizer available for streaming inference"))?
            .clone();

        let stream_config = StreamConfig {
            buffer_size: 64,
            include_timing: false,
            max_tokens_per_sec: 0,
        };
        let (tx, rx) = create_stream_channel(stream_config);
        let metrics = self.metrics.clone();

        tokio::task::spawn_blocking(move || {
            let start_time = Instant::now();
            let prompt_time = start_time.elapsed();
            let mut all_tokens = initial_tokens.clone();
            let mut sampler = Sampler::new(sampling_config);
            let mut generated_text = String::new();

            let mut session_guard = match session.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    let _ = tx.blocking_send(StreamToken {
                        content: format!("Error: Session lock poisoned: {}", e),
                        sequence: 0,
                        is_valid: false,
                        timestamp_ms: Some(start_time.elapsed().as_millis() as u64),
                    });
                    return;
                }
            };

            let mut completion_tokens = 0u32;

            for seq in 0..max_tokens {
                let logits = match Self::forward_pass(&mut session_guard, &all_tokens, &input_names)
                {
                    Ok(l) => l,
                    Err(e) => {
                        let _ = tx.blocking_send(StreamToken {
                            content: format!("Error: {}", e),
                            sequence: seq,
                            is_valid: false,
                            timestamp_ms: Some(start_time.elapsed().as_millis() as u64),
                        });
                        break;
                    }
                };

                let probs = Self::softmax(&logits);
                let candidates: Vec<(i32, f32, f32)> = logits
                    .iter()
                    .zip(probs.iter())
                    .enumerate()
                    .map(|(id, (&logit, &p))| (id as i32, logit, p))
                    .collect();

                let next_token = match sampler.sample_from_candidates(&candidates) {
                    Some(token) => token,
                    None => {
                        let _ = tx.blocking_send(StreamToken {
                            content: "[ERROR: No candidates available]".to_string(),
                            sequence: seq,
                            is_valid: false,
                            timestamp_ms: Some(start_time.elapsed().as_millis() as u64),
                        });
                        break;
                    }
                };

                // Check for EOS token
                if let Some(eos_id) = eos_token_id {
                    if next_token as u32 == eos_id {
                        debug!("EOS token encountered in stream, stopping generation");
                        break;
                    }
                }

                all_tokens.push(next_token as i64);
                completion_tokens += 1;

                match tokenizer.decode(&[next_token as u32], true) {
                    Ok(token_str) => {
                        generated_text.push_str(&token_str);

                        // Check stop sequences
                        if stop_sequences
                            .iter()
                            .any(|stop| generated_text.contains(stop))
                        {
                            debug!("Stop sequence matched in stream, stopping generation");
                            // Still send this last token
                            let _ = tx.blocking_send(StreamToken::new(token_str, seq));
                            break;
                        }

                        let stream_token = StreamToken {
                            content: token_str,
                            sequence: seq,
                            is_valid: true,
                            timestamp_ms: Some(start_time.elapsed().as_millis() as u64),
                        };
                        if tx.blocking_send(stream_token).is_err() {
                            debug!("Stream receiver disconnected, stopping generation");
                            break;
                        }
                    }
                    Err(_) => {
                        // Skip invalid tokens rather than sending empty strings
                        debug!("Failed to detokenize token {}, skipping", next_token);
                    }
                }
            }

            // Update metrics from the streaming task
            let completion_time = start_time.elapsed() - prompt_time;
            let total_time = start_time.elapsed();
            if let Ok(mut m) = metrics.lock() {
                *m = Some(InferenceMetrics {
                    total_tokens: prompt_tokens + completion_tokens,
                    prompt_tokens,
                    completion_tokens,
                    total_time_ms: total_time.as_millis() as u64,
                    tokens_per_second: if completion_time.as_secs_f32() > 0.0 {
                        completion_tokens as f32 / completion_time.as_secs_f32()
                    } else {
                        0.0
                    },
                    prompt_time_ms: prompt_time.as_millis() as u64,
                    completion_time_ms: completion_time.as_millis() as u64,
                });
            }

            debug!(
                "ONNX streaming complete: generated {} tokens in {:?}",
                completion_tokens,
                start_time.elapsed()
            );
        });

        let result_stream = stream! {
            let mut rx = rx;
            while let Some(stream_token) = rx.recv().await {
                if stream_token.is_valid {
                    yield Ok(stream_token.content);
                }
            }
        };

        Ok(Box::pin(result_stream))
    }

    async fn get_embeddings(&mut self, input: &str) -> Result<Vec<f32>> {
        if !self.is_loaded().await {
            return Err(InfernoError::Backend("Model not loaded".to_string()).into());
        }

        info!("Computing ONNX embeddings");

        let session = self.session.as_ref().unwrap().clone();
        let input_names = self.input_names.clone();

        let token_ids = self.tokenize(input)?;
        let seq_len = token_ids.len();
        let input_i64: Vec<i64> = token_ids.iter().map(|&t| t as i64).collect();

        let embeddings = tokio::task::spawn_blocking(move || -> Result<Vec<f32>> {
            let mut session_guard = session
                .lock()
                .map_err(|e| anyhow!("Session lock poisoned: {}", e))?;
            let attention_mask_data: Vec<i64> = vec![1i64; seq_len];

            let input_ids_tensor = Tensor::from_array(([1usize, seq_len], input_i64))
                .map_err(|e| anyhow!("Failed to create input_ids tensor: {}", e))?;

            let outputs = if let Some(ref mask_name) = input_names.attention_mask {
                let attention_mask_tensor =
                    Tensor::from_array(([1usize, seq_len], attention_mask_data))
                        .map_err(|e| anyhow!("Failed to create attention_mask tensor: {}", e))?;
                session_guard
                    .run(inputs![
                        input_names.input_ids.as_str() => input_ids_tensor,
                        mask_name.as_str() => attention_mask_tensor
                    ])
                    .map_err(|e| {
                        InfernoError::Backend(format!("ONNX embedding inference failed: {}", e))
                    })?
            } else {
                session_guard
                    .run(inputs![
                        input_names.input_ids.as_str() => input_ids_tensor
                    ])
                    .map_err(|e| {
                        InfernoError::Backend(format!("ONNX embedding inference failed: {}", e))
                    })?
            };

            let output_value = &outputs[0usize];
            let (shape, data) = output_value
                .try_extract_tensor::<f32>()
                .map_err(|e| anyhow!("Failed to extract embedding tensor: {}", e))?;

            // Mean-pool over the sequence dimension
            let pooled = if shape.len() == 3 {
                // Shape: [batch, seq_len, hidden_dim]
                let seq_dim = shape[1] as usize;
                let hidden_dim = shape[2] as usize;
                let mut result = vec![0.0f32; hidden_dim];
                for s in 0..seq_dim {
                    for h in 0..hidden_dim {
                        result[h] += data[s * hidden_dim + h];
                    }
                }
                for val in &mut result {
                    *val /= seq_dim as f32;
                }
                result
            } else if shape.len() == 2 {
                // Shape: [batch, hidden_dim] — already pooled
                let hidden_dim = shape[1] as usize;
                data[..hidden_dim].to_vec()
            } else {
                data.to_vec()
            };

            Ok(pooled)
        })
        .await
        .map_err(|e| InfernoError::Backend(format!("Embedding task failed: {}", e)))??;

        info!("Generated {} dimensional embeddings", embeddings.len());
        Ok(embeddings)
    }

    fn get_backend_type(&self) -> BackendType {
        BackendType::Onnx
    }

    fn get_metrics(&self) -> Option<InferenceMetrics> {
        self.metrics.lock().ok().and_then(|m| m.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onnx_backend_creation() {
        let config = BackendConfig::default();
        let backend = OnnxBackend::new(config).unwrap();
        assert_eq!(backend.get_backend_type(), BackendType::Onnx);
        assert!(backend.session.is_none());
        assert!(backend.tokenizer.is_none());
    }

    #[tokio::test]
    async fn test_onnx_model_loading_missing_file() {
        let config = BackendConfig::default();
        let mut backend = OnnxBackend::new(config).unwrap();
        let model_info = ModelInfo {
            name: "nonexistent".to_string(),
            path: std::path::PathBuf::from("/tmp/nonexistent_model.onnx"),
            file_path: std::path::PathBuf::from("/tmp/nonexistent_model.onnx"),
            size: 0,
            size_bytes: 0,
            modified: chrono::Utc::now(),
            backend_type: "onnx".to_string(),
            format: "onnx".to_string(),
            checksum: None,
            metadata: std::collections::HashMap::new(),
        };
        let result = backend.load_model(&model_info).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"), "Error was: {}", err_msg);
    }

    #[tokio::test]
    async fn test_onnx_inference_without_model() {
        let config = BackendConfig::default();
        let mut backend = OnnxBackend::new(config).unwrap();
        let params = InferenceParams::default();
        let result = backend.infer("test", &params).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not loaded"));
    }

    #[tokio::test]
    async fn test_onnx_stream_without_model() {
        let config = BackendConfig::default();
        let mut backend = OnnxBackend::new(config).unwrap();
        let params = InferenceParams::default();
        let result = backend.infer_stream("test", &params).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("not loaded"));
    }

    #[tokio::test]
    async fn test_onnx_embeddings_without_model() {
        let config = BackendConfig::default();
        let mut backend = OnnxBackend::new(config).unwrap();
        let result = backend.get_embeddings("test").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not loaded"));
    }

    #[tokio::test]
    async fn test_onnx_unload_model() {
        let config = BackendConfig::default();
        let mut backend = OnnxBackend::new(config).unwrap();
        let result = backend.unload_model().await;
        assert!(result.is_ok());
        assert!(!backend.is_loaded().await);
    }

    #[test]
    fn test_onnx_tokenize_without_tokenizer() {
        let config = BackendConfig::default();
        let backend = OnnxBackend::new(config).unwrap();
        let result = backend.tokenize("hello world");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("tokenizer"));
    }

    #[test]
    fn test_onnx_detokenize_without_tokenizer() {
        let config = BackendConfig::default();
        let backend = OnnxBackend::new(config).unwrap();
        let result = backend.detokenize(&[1, 2, 3]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("tokenizer"));
    }

    #[test]
    fn test_onnx_estimate_token_count_fallback() {
        let config = BackendConfig::default();
        let backend = OnnxBackend::new(config).unwrap();
        let count = backend.estimate_token_count("hello world");
        assert!(count > 0);
    }

    #[test]
    fn test_onnx_execution_provider_config_gpu_disabled() {
        let config = BackendConfig {
            gpu_enabled: false,
            ..BackendConfig::default()
        };
        let backend = OnnxBackend::new(config).unwrap();
        let providers = backend.build_execution_providers();
        assert!(providers.is_empty());
    }

    #[test]
    fn test_onnx_sampling_config_greedy() {
        let params = InferenceParams {
            temperature: 0.0,
            ..InferenceParams::default()
        };
        let config = OnnxBackend::build_sampling_config(&params);
        assert!(matches!(config.strategy, SamplingStrategy::Greedy));
    }

    #[test]
    fn test_onnx_sampling_config_topkp() {
        let params = InferenceParams {
            temperature: 0.8,
            top_k: 50,
            top_p: 0.95,
            ..InferenceParams::default()
        };
        let config = OnnxBackend::build_sampling_config(&params);
        assert!(matches!(config.strategy, SamplingStrategy::TopKP));
        assert!((config.temperature - 0.8).abs() < 0.01);
        assert_eq!(config.top_k, 50);
    }

    #[test]
    fn test_onnx_input_names_default() {
        let names = InputNames::default();
        assert_eq!(names.input_ids, "input_ids");
        assert_eq!(names.attention_mask, Some("attention_mask".to_string()));
    }
}
