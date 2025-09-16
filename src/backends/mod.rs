mod gguf;
mod onnx;

use crate::{models::ModelInfo, InfernoError};
use anyhow::Result;
use clap::ValueEnum;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::{path::Path, pin::Pin, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
pub enum BackendType {
    #[value(name = "gguf")]
    Gguf,
    #[value(name = "onnx")]
    Onnx,
}

impl BackendType {
    pub fn from_model_path(path: &Path) -> Self {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("gguf") => BackendType::Gguf,
            Some("onnx") => BackendType::Onnx,
            _ => {
                // Try to infer from filename patterns
                let filename = path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if filename.contains("gguf") || filename.contains("llama") || filename.contains("gpt") {
                    BackendType::Gguf
                } else if filename.contains("onnx") {
                    BackendType::Onnx
                } else {
                    // Default to GGUF for text models
                    BackendType::Gguf
                }
            }
        }
    }
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendType::Gguf => write!(f, "gguf"),
            BackendType::Onnx => write!(f, "onnx"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub gpu_enabled: bool,
    pub gpu_device: Option<String>,
    pub cpu_threads: Option<u32>,
    pub context_size: u32,
    pub batch_size: u32,
    pub memory_map: bool,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            gpu_enabled: false,
            gpu_device: None,
            cpu_threads: None,
            context_size: 2048,
            batch_size: 32,
            memory_map: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceParams {
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub stream: bool,
}

impl Default for InferenceParams {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            stream: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InferenceMetrics {
    pub total_tokens: u32,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_time_ms: u64,
    pub tokens_per_second: f32,
    pub prompt_time_ms: u64,
    pub completion_time_ms: u64,
}

pub type TokenStream = Pin<Box<dyn Stream<Item = Result<String, InfernoError>> + Send>>;

#[async_trait::async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn load_model(&mut self, model_info: &ModelInfo) -> Result<()>;
    async fn unload_model(&mut self) -> Result<()>;
    async fn is_loaded(&self) -> bool;
    async fn get_model_info(&self) -> Option<ModelInfo>;

    async fn infer(&mut self, input: &str, params: &InferenceParams) -> Result<String>;
    async fn infer_stream(&mut self, input: &str, params: &InferenceParams) -> Result<TokenStream>;
    async fn get_embeddings(&mut self, input: &str) -> Result<Vec<f32>>;

    fn get_backend_type(&self) -> BackendType;
    fn get_metrics(&self) -> Option<InferenceMetrics>;
}

pub struct Backend {
    backend_impl: Box<dyn InferenceBackend>,
}

impl Backend {
    pub fn new(backend_type: BackendType, config: &BackendConfig) -> Result<Self> {
        let backend_impl: Box<dyn InferenceBackend> = match backend_type {
            BackendType::Gguf => Box::new(gguf::GgufBackend::new(config.clone())?),
            BackendType::Onnx => Box::new(onnx::OnnxBackend::new(config.clone())?),
        };

        Ok(Self { backend_impl })
    }

    /// Create a new shared backend instance wrapped in Arc<Mutex<_>>
    pub fn new_shared(backend_type: BackendType, config: &BackendConfig) -> Result<BackendHandle> {
        let backend = Self::new(backend_type, config)?;
        Ok(BackendHandle::new(backend))
    }

    pub async fn load_model(&mut self, model_info: &ModelInfo) -> Result<()> {
        self.backend_impl.load_model(model_info).await
    }

    pub async fn unload_model(&mut self) -> Result<()> {
        self.backend_impl.unload_model().await
    }

    pub async fn is_loaded(&self) -> bool {
        self.backend_impl.is_loaded().await
    }

    pub async fn get_model_info(&self) -> Option<ModelInfo> {
        self.backend_impl.get_model_info().await
    }

    pub async fn infer(&mut self, input: &str, params: &InferenceParams) -> Result<String> {
        self.backend_impl.infer(input, params).await
    }

    pub async fn infer_stream(&mut self, input: &str, params: &InferenceParams) -> Result<TokenStream> {
        self.backend_impl.infer_stream(input, params).await
    }

    pub async fn get_embeddings(&mut self, input: &str) -> Result<Vec<f32>> {
        self.backend_impl.get_embeddings(input).await
    }

    pub fn get_backend_type(&self) -> BackendType {
        self.backend_impl.get_backend_type()
    }

    pub fn get_metrics(&self) -> Option<InferenceMetrics> {
        self.backend_impl.get_metrics()
    }
}

/// Thread-safe, cloneable handle to a shared Backend instance
#[derive(Clone)]
pub struct BackendHandle {
    inner: Arc<Mutex<Backend>>,
    backend_type: BackendType,
}

impl BackendHandle {
    /// Create a new backend handle from a backend instance
    pub fn new(backend: Backend) -> Self {
        let backend_type = backend.get_backend_type();
        Self {
            inner: Arc::new(Mutex::new(backend)),
            backend_type,
        }
    }

    /// Create a new shared backend handle
    pub fn new_shared(backend_type: BackendType, config: &BackendConfig) -> Result<Self> {
        let backend = Backend::new(backend_type, config)?;
        Ok(Self::new(backend))
    }

    /// Load a model into this backend
    pub async fn load_model(&self, model_info: &ModelInfo) -> Result<()> {
        let mut backend = self.inner.lock().await;
        backend.load_model(model_info).await
    }

    /// Unload the current model from this backend
    pub async fn unload_model(&self) -> Result<()> {
        let mut backend = self.inner.lock().await;
        backend.unload_model().await
    }

    /// Check if a model is currently loaded
    pub async fn is_loaded(&self) -> bool {
        let backend = self.inner.lock().await;
        backend.is_loaded().await
    }

    /// Get information about the currently loaded model
    pub async fn get_model_info(&self) -> Option<ModelInfo> {
        let backend = self.inner.lock().await;
        backend.get_model_info().await
    }

    /// Perform inference with the loaded model
    pub async fn infer(&self, input: &str, params: &InferenceParams) -> Result<String> {
        let mut backend = self.inner.lock().await;
        backend.infer(input, params).await
    }

    /// Perform streaming inference with the loaded model
    pub async fn infer_stream(&self, input: &str, params: &InferenceParams) -> Result<TokenStream> {
        let mut backend = self.inner.lock().await;
        backend.infer_stream(input, params).await
    }

    /// Get embeddings from the loaded model
    pub async fn get_embeddings(&self, input: &str) -> Result<Vec<f32>> {
        let mut backend = self.inner.lock().await;
        backend.get_embeddings(input).await
    }

    /// Get the backend type
    pub fn get_backend_type(&self) -> BackendType {
        self.backend_type
    }

    /// Get current metrics from the backend
    pub async fn get_metrics(&self) -> Option<InferenceMetrics> {
        let backend = self.inner.lock().await;
        backend.get_metrics()
    }

    /// Get a reference to the underlying Arc<Mutex<Backend>> for advanced usage
    pub fn inner(&self) -> &Arc<Mutex<Backend>> {
        &self.inner
    }
}

impl std::fmt::Debug for BackendHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendHandle")
            .field("backend_type", &self.backend_type)
            .finish()
    }
}