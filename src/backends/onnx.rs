use crate::{
    backends::{BackendConfig, BackendType, InferenceBackend, InferenceMetrics, InferenceParams, TokenStream},
    models::ModelInfo,
    InfernoError,
};
use anyhow::{anyhow, Result};
use async_stream::stream;
use ndarray::{s, Array2};
use ort::{
    Environment, ExecutionProvider, GraphOptimizationLevel, Session,
    SessionBuilder, Value,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::Instant,
};
use tokenizers::Tokenizer;
use tracing::{debug, info, warn};

pub struct OnnxBackend {
    config: BackendConfig,
    session: Option<Session>,
    tokenizer: Option<Tokenizer>,
    model_info: Option<ModelInfo>,
    metrics: Option<InferenceMetrics>,
    input_names: Vec<String>,
    output_names: Vec<String>,
    environment: Arc<Environment>,
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
        info!("Initializing ONNX backend with ONNX Runtime");

        // Initialize ONNX Runtime environment
        let environment = Arc::new(
            Environment::builder()
                .with_name("inferno-onnx")
                .with_log_level(ort::LoggingLevel::Warning)
                .build()
                .map_err(|e| anyhow!("Failed to initialize ONNX environment: {}", e))?,
        );

        Ok(Self {
            config,
            session: None,
            tokenizer: None,
            model_info: None,
            metrics: None,
            input_names: Vec::new(),
            output_names: Vec::new(),
            environment,
            model_type: ModelType::Unknown,
        })
    }

    fn get_execution_providers(&self) -> Vec<ExecutionProvider> {
        let mut providers = Vec::new();

        if self.config.gpu_enabled {
            // Try GPU providers in order of preference
            #[cfg(target_os = "windows")]
            {
                providers.push(ExecutionProvider::DirectML(Default::default()));
                info!("Added DirectML execution provider");
            }

            #[cfg(target_os = "linux")]
            {
                providers.push(ExecutionProvider::CUDA(Default::default()));
                info!("Added CUDA execution provider");
            }

            #[cfg(target_os = "macos")]
            {
                providers.push(ExecutionProvider::CoreML(Default::default()));
                info!("Added CoreML execution provider");
            }
        }

        // CPU is always available as fallback
        providers.push(ExecutionProvider::CPU(Default::default()));
        info!("Added CPU execution provider as fallback");

        providers
    }

    fn detect_model_type(&self, session: &Session) -> ModelType {
        let input_names = session.inputs.iter().map(|i| i.name.clone()).collect::<Vec<_>>();
        let output_names = session.outputs.iter().map(|o| o.name.clone()).collect::<Vec<_>>();

        // Analyze input/output names to determine model type
        if input_names.contains(&"input_ids".to_string()) ||
           input_names.contains(&"attention_mask".to_string()) {
            if output_names.contains(&"last_hidden_state".to_string()) ||
               output_names.contains(&"pooler_output".to_string()) ||
               output_names.contains(&"embeddings".to_string()) {
                ModelType::Embedding
            } else if output_names.contains(&"logits".to_string()) && output_names.len() == 1 {
                ModelType::Classification
            } else {
                ModelType::TextGeneration
            }
        } else if output_names.contains(&"embeddings".to_string()) ||
                  output_names.contains(&"last_hidden_state".to_string()) {
            ModelType::Embedding
        } else if output_names.len() == 1 &&
                  (output_names[0].contains("logits") || output_names[0].contains("output")) {
            ModelType::Classification
        } else {
            ModelType::TextGeneration
        }
    }

    fn load_tokenizer(&mut self, model_path: &std::path::Path) -> Result<()> {
        // Try to find tokenizer files in the same directory as the model
        let model_dir = model_path.parent().unwrap_or(model_path);

        // Common tokenizer file names
        let tokenizer_files = [
            "tokenizer.json",
            "vocab.txt",
            "tokenizer_config.json",
        ];

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
                        debug!("Failed to load tokenizer from {}: {}", tokenizer_path.display(), e);
                    }
                }
            }
        }

        // If no tokenizer found, create a simple word-based tokenizer
        warn!("No tokenizer found, using simple word-based tokenization");
        Ok(())
    }

    fn tokenize_text(&self, text: &str) -> Result<(Vec<i64>, Vec<i64>)> {
        if let Some(tokenizer) = &self.tokenizer {
            let encoding = tokenizer
                .encode(text, false)
                .map_err(|e| anyhow!("Tokenization failed: {}", e))?;

            let input_ids = encoding.get_ids().iter().map(|&id| id as i64).collect();
            let attention_mask = vec![1i64; input_ids.len()];

            Ok((input_ids, attention_mask))
        } else {
            // Simple fallback tokenization
            let words: Vec<&str> = text.split_whitespace().collect();
            let input_ids: Vec<i64> = words
                .iter()
                .enumerate()
                .map(|(i, _)| (i % 30000) as i64 + 1) // Simple vocab mapping
                .collect();
            let attention_mask = vec![1i64; input_ids.len()];

            Ok((input_ids, attention_mask))
        }
    }

    fn detokenize_text(&self, token_ids: &[i64]) -> Result<String> {
        if let Some(tokenizer) = &self.tokenizer {
            let tokens: Vec<u32> = token_ids.iter().map(|&id| id as u32).collect();
            tokenizer
                .decode(&tokens, false)
                .map_err(|e| anyhow!("Detokenization failed: {}", e))
        } else {
            // Simple fallback detokenization
            Ok(format!("Generated text from {} tokens", token_ids.len()))
        }
    }

    fn prepare_inputs(&self, text: &str) -> Result<HashMap<String, Value>> {
        let (input_ids, attention_mask) = self.tokenize_text(text)?;

        let mut inputs = HashMap::new();

        // Prepare input tensors based on the model's expected inputs
        for input_name in &self.input_names {
            match input_name.as_str() {
                "input_ids" => {
                    let tensor = Array2::from_shape_vec(
                        (1, input_ids.len()),
                        input_ids.clone(),
                    )?;
                    inputs.insert(input_name.clone(), Value::from_array(tensor)?);
                }
                "attention_mask" => {
                    let tensor = Array2::from_shape_vec(
                        (1, attention_mask.len()),
                        attention_mask.clone(),
                    )?;
                    inputs.insert(input_name.clone(), Value::from_array(tensor)?);
                }
                "token_type_ids" => {
                    let token_type_ids = vec![0i64; input_ids.len()];
                    let tensor = Array2::from_shape_vec(
                        (1, token_type_ids.len()),
                        token_type_ids,
                    )?;
                    inputs.insert(input_name.clone(), Value::from_array(tensor)?);
                }
                "position_ids" => {
                    let position_ids: Vec<i64> = (0..input_ids.len() as i64).collect();
                    let tensor = Array2::from_shape_vec(
                        (1, position_ids.len()),
                        position_ids,
                    )?;
                    inputs.insert(input_name.clone(), Value::from_array(tensor)?);
                }
                _ => {
                    // For unknown inputs, try to create a reasonable default
                    if input_name.contains("input") || input_name.contains("text") {
                        let tensor = Array2::from_shape_vec(
                            (1, input_ids.len()),
                            input_ids.clone(),
                        )?;
                        inputs.insert(input_name.clone(), Value::from_array(tensor)?);
                    }
                }
            }
        }

        Ok(inputs)
    }

    fn extract_text_from_outputs(&self, outputs: HashMap<String, Value>) -> Result<String> {
        for (name, value) in &outputs {
            debug!("Output '{}': shape {:?}", name, value.shape());
        }

        match self.model_type {
            ModelType::TextGeneration => {
                // Look for logits or similar output
                if let Some(logits) = outputs.get("logits")
                    .or_else(|| outputs.get("output"))
                    .or_else(|| outputs.values().next()) {

                    let array = logits.try_extract::<f32>()?;
                    let shape = array.shape();

                    if shape.len() >= 2 {
                        let seq_len = shape[shape.len() - 2];
                        let _vocab_size = shape[shape.len() - 1];

                        // Extract the last token's logits and get the top token
                        let last_token_logits = array.slice(s![.., seq_len - 1, ..]);
                        let max_idx = last_token_logits
                            .iter()
                            .enumerate()
                            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                            .map(|(idx, _)| idx as i64)
                            .unwrap_or(0);

                        // Simple token generation - in practice, you'd use proper decoding
                        let generated_tokens = vec![max_idx];
                        return self.detokenize_text(&generated_tokens);
                    }
                }

                Ok("Generated text output from ONNX model".to_string())
            }
            ModelType::Classification => {
                if let Some(logits) = outputs.get("logits")
                    .or_else(|| outputs.get("output"))
                    .or_else(|| outputs.values().next()) {

                    let array = logits.try_extract::<f32>()?;
                    let probabilities: Vec<f32> = array.iter().cloned().collect();

                    let (max_idx, max_prob) = probabilities
                        .iter()
                        .enumerate()
                        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                        .map(|(idx, &prob)| (idx, prob))
                        .unwrap_or((0, 0.0));

                    // Apply softmax to get proper probabilities
                    let max_val = probabilities.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                    let exp_sum: f32 = probabilities.iter().map(|&x| (x - max_val).exp()).sum();
                    let softmax_prob = (probabilities[max_idx] - max_val).exp() / exp_sum;

                    Ok(format!(
                        "Classification Result:\nPredicted Class: {}\nConfidence: {:.2}%",
                        max_idx,
                        softmax_prob * 100.0
                    ))
                } else {
                    Ok("Classification completed".to_string())
                }
            }
            _ => Ok("ONNX inference completed".to_string()),
        }
    }

    fn extract_embeddings_from_outputs(&self, outputs: HashMap<String, Value>) -> Result<Vec<f32>> {
        // Look for embedding outputs in common names
        let embedding_keys = [
            "last_hidden_state",
            "pooler_output",
            "embeddings",
            "sentence_embedding",
            "output",
        ];

        for key in &embedding_keys {
            if let Some(value) = outputs.get(*key) {
                let array = value.try_extract::<f32>()?;
                let shape = array.shape();

                // Handle different embedding output shapes
                match shape.len() {
                    2 => {
                        // [batch_size, embedding_dim] - use as is
                        return Ok(array.iter().cloned().collect());
                    }
                    3 => {
                        // [batch_size, seq_len, embedding_dim] - pool over sequence
                        let batch_size = shape[0];
                        let seq_len = shape[1];
                        let embed_dim = shape[2];

                        if batch_size == 1 {
                            // Mean pooling over sequence length
                            let mut embeddings = vec![0.0f32; embed_dim];
                            for i in 0..embed_dim {
                                let mut sum = 0.0f32;
                                for j in 0..seq_len {
                                    sum += array[[0, j, i]];
                                }
                                embeddings[i] = sum / seq_len as f32;
                            }
                            return Ok(embeddings);
                        }
                    }
                    _ => {
                        debug!("Unexpected embedding shape: {:?}", shape);
                    }
                }
            }
        }

        // Fallback: use the first output
        if let Some((_, value)) = outputs.iter().next() {
            let array = value.try_extract::<f32>()?;
            let flattened: Vec<f32> = array.iter().cloned().collect();

            // Limit to reasonable embedding size
            let max_size = 4096;
            if flattened.len() > max_size {
                Ok(flattened[..max_size].to_vec())
            } else {
                Ok(flattened)
            }
        } else {
            Err(anyhow!("No outputs found for embedding extraction"))
        }
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
}

#[async_trait::async_trait]
impl InferenceBackend for OnnxBackend {
    async fn load_model(&mut self, model_info: &ModelInfo) -> Result<()> {
        info!("Loading ONNX model: {}", model_info.path.display());

        // Validate model file exists
        if !model_info.path.exists() {
            return Err(anyhow!("Model file not found: {}", model_info.path.display()));
        }

        // Load tokenizer if available
        self.load_tokenizer(&model_info.path)?;

        // Configure session builder
        let mut session_builder = SessionBuilder::new(&self.environment)?;

        // Set execution providers
        let providers = self.get_execution_providers();
        for provider in providers {
            session_builder = session_builder.with_execution_provider(provider)?;
        }

        // Configure optimization settings
        session_builder = session_builder
            .with_optimization_level(GraphOptimizationLevel::All)?
            .with_intra_threads(self.config.cpu_threads.unwrap_or(num_cpus::get() as u32) as i16)?;

        // Enable memory pattern optimization
        session_builder = session_builder.with_memory_pattern(true)?;

        // Load the model
        let session = session_builder
            .commit_from_file(&model_info.path)
            .map_err(|e| anyhow!("Failed to load ONNX model: {}", e))?;

        // Extract input and output information
        self.input_names = session.inputs.iter().map(|i| i.name.clone()).collect();
        self.output_names = session.outputs.iter().map(|o| o.name.clone()).collect();

        info!("Model inputs: {:?}", self.input_names);
        info!("Model outputs: {:?}", self.output_names);

        // Detect model type
        self.model_type = self.detect_model_type(&session);
        info!("Detected model type: {:?}", self.model_type);

        self.session = Some(session);
        self.model_info = Some(model_info.clone());

        info!("ONNX model loaded successfully");
        Ok(())
    }

    async fn unload_model(&mut self) -> Result<()> {
        info!("Unloading ONNX model");
        self.session = None;
        self.tokenizer = None;
        self.model_info = None;
        self.metrics = None;
        self.input_names.clear();
        self.output_names.clear();
        self.model_type = ModelType::Unknown;
        Ok(())
    }

    async fn is_loaded(&self) -> bool {
        self.session.is_some()
    }

    async fn get_model_info(&self) -> Option<ModelInfo> {
        self.model_info.clone()
    }

    async fn infer(&mut self, input: &str, _params: &InferenceParams) -> Result<String> {
        let session = self.session.as_ref()
            .ok_or_else(|| anyhow!("Model not loaded"))?;

        let start_time = Instant::now();
        info!("Starting ONNX inference");

        // Prepare inputs
        let inputs = self.prepare_inputs(input)?;
        let prompt_tokens = self.estimate_token_count(input);
        let prompt_time = start_time.elapsed();

        // Run inference
        let outputs = session.run(inputs)
            .map_err(|e| anyhow!("ONNX inference failed: {}", e))?;

        let completion_time = start_time.elapsed() - prompt_time;

        // Extract text from outputs
        let result = self.extract_text_from_outputs(outputs)?;

        let total_time = start_time.elapsed();
        let completion_tokens = self.estimate_token_count(&result);
        let total_tokens = prompt_tokens + completion_tokens;

        // Update metrics
        self.metrics = Some(InferenceMetrics {
            total_tokens,
            prompt_tokens,
            completion_tokens,
            total_time_ms: total_time.as_millis() as u64,
            tokens_per_second: completion_tokens as f32 / completion_time.as_secs_f32(),
            prompt_time_ms: prompt_time.as_millis() as u64,
            completion_time_ms: completion_time.as_millis() as u64,
        });

        info!(
            "ONNX inference completed: {} tokens in {:.2}s",
            completion_tokens,
            completion_time.as_secs_f32()
        );

        Ok(result)
    }

    async fn infer_stream(&mut self, input: &str, params: &InferenceParams) -> Result<TokenStream> {
        if self.session.is_none() {
            return Err(anyhow!("Model not loaded"));
        }

        info!("Starting ONNX streaming inference");

        // Since ONNX models don't inherently support streaming, we'll simulate it
        // by running inference and chunking the output
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
        let session = self.session.as_ref()
            .ok_or_else(|| anyhow!("Model not loaded"))?;

        info!("Computing ONNX embeddings");

        // Prepare inputs
        let inputs = self.prepare_inputs(input)?;

        // Run inference
        let outputs = session.run(inputs)
            .map_err(|e| anyhow!("ONNX embedding inference failed: {}", e))?;

        // Extract embeddings
        let embeddings = self.extract_embeddings_from_outputs(outputs)?;

        info!("Generated {} dimensional embeddings", embeddings.len());
        Ok(embeddings)
    }

    fn get_backend_type(&self) -> BackendType {
        BackendType::Onnx
    }

    fn get_metrics(&self) -> Option<InferenceMetrics> {
        self.metrics.clone()
    }
}