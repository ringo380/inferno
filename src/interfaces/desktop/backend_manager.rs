use super::activity_logger::{ActivityLogger, ActivityStatus, ActivityType};
use crate::backends::{
    BackendConfig, BackendHandle, BackendType, InferenceParams as InfernoInferenceParams,
    TokenStream,
};
use crate::models::{ModelInfo as CoreModelInfo, ModelManager};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub format: String,
    pub size: u64,
    pub checksum: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InferenceParams {
    pub temperature: Option<f32>,
    pub top_k: Option<u32>,
    pub top_p: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
    pub stop_sequences: Option<Vec<String>>,
    pub seed: Option<u64>,
}

pub struct BackendManager {
    model_manager: Arc<RwLock<ModelManager>>,
    loaded_backends: Arc<Mutex<HashMap<String, BackendHandle>>>,
    global_metrics: Arc<Mutex<GlobalMetrics>>,
    activity_logger: Arc<ActivityLogger>,
}

#[derive(Debug, Clone, Default)]
pub struct GlobalMetrics {
    pub inference_count: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub average_latency: f64,
    pub models_loaded: u32,
    pub active_inferences: u32,
    pub active_streaming_sessions: u32,
}

/// Guard that decrements active streaming sessions when dropped.
pub struct StreamingSessionGuard {
    metrics: Arc<Mutex<GlobalMetrics>>,
}

impl StreamingSessionGuard {
    fn new(metrics: Arc<Mutex<GlobalMetrics>>) -> Self {
        {
            let mut guard_metrics = metrics.lock().unwrap();
            guard_metrics.active_streaming_sessions += 1;
        }

        Self { metrics }
    }
}

impl Drop for StreamingSessionGuard {
    fn drop(&mut self) {
        let mut metrics = self.metrics.lock().unwrap();
        if metrics.active_streaming_sessions > 0 {
            metrics.active_streaming_sessions -= 1;
        }
    }
}

/// Guard that tracks active inference operations and auto-decrements when dropped.
pub struct InferenceGuard {
    metrics: Arc<Mutex<GlobalMetrics>>,
}

impl InferenceGuard {
    fn new(metrics: Arc<Mutex<GlobalMetrics>>) -> Self {
        {
            let mut guard_metrics = metrics.lock().unwrap();
            guard_metrics.active_inferences += 1;
            guard_metrics.inference_count += 1;
        }

        Self { metrics }
    }
}

impl Drop for InferenceGuard {
    fn drop(&mut self) {
        let mut metrics = self.metrics.lock().unwrap();
        if metrics.active_inferences > 0 {
            metrics.active_inferences -= 1;
        }
    }
}

impl BackendManager {
    pub async fn new(activity_logger: Arc<ActivityLogger>) -> Result<Self> {
        // Use default models directory (will be overridden by settings)
        let default_models_dir = std::path::PathBuf::from("test_models/test_models");
        let model_manager = ModelManager::new(&default_models_dir);

        Ok(Self {
            model_manager: Arc::new(RwLock::new(model_manager)),
            loaded_backends: Arc::new(Mutex::new(HashMap::new())),
            global_metrics: Arc::new(Mutex::new(GlobalMetrics::default())),
            activity_logger,
        })
    }

    /// Create BackendManager with custom models directory
    pub async fn with_models_dir(
        activity_logger: Arc<ActivityLogger>,
        models_dir: PathBuf,
    ) -> Result<Self> {
        let model_manager = ModelManager::new(&models_dir);

        Ok(Self {
            model_manager: Arc::new(RwLock::new(model_manager)),
            loaded_backends: Arc::new(Mutex::new(HashMap::new())),
            global_metrics: Arc::new(Mutex::new(GlobalMetrics::default())),
            activity_logger,
        })
    }

    pub async fn discover_models(&self) -> Result<Vec<ModelInfo>> {
        let model_manager = self.model_manager.read().await;
        let models = model_manager.list_models().await?;

        Ok(models
            .into_iter()
            .map(|m| ModelInfo {
                id: m.name.clone(), // Use name as ID
                name: m.name.clone(),
                path: m.path.to_string_lossy().to_string(),
                format: m.backend_type.clone(),
                size: m.size,
                checksum: m.checksum.unwrap_or_else(|| "unknown".to_string()),
                status: "available".to_string(),
            })
            .collect())
    }

    pub async fn load_model(&self, model_name: String, backend_type_str: String) -> Result<String> {
        // Log the start of the operation
        self.activity_logger.log_model_operation(
            ActivityType::ModelLoad,
            &model_name,
            ActivityStatus::InProgress,
            Some(&format!("Loading model with {} backend", backend_type_str)),
        );

        let model_manager = self.model_manager.read().await;

        // Find the model
        let model = model_manager
            .resolve_model(&model_name)
            .await
            .map_err(|e| {
                self.activity_logger.log_model_operation(
                    ActivityType::ModelLoad,
                    &model_name,
                    ActivityStatus::Error,
                    Some(&format!("Failed to resolve model: {}", e)),
                );
                e
            })?;

        // Parse backend type
        let backend_type = match backend_type_str.to_lowercase().as_str() {
            "gguf" => BackendType::Gguf,
            _ => BackendType::Gguf, // Default fallback (ONNX support disabled)
        };

        // Create backend config
        let backend_config = BackendConfig::default();

        // Create and load backend
        let backend_handle =
            BackendHandle::new_shared(backend_type, &backend_config).map_err(|e| {
                self.activity_logger.log_model_operation(
                    ActivityType::ModelLoad,
                    &model_name,
                    ActivityStatus::Error,
                    Some(&format!("Failed to create backend: {}", e)),
                );
                e
            })?;

        backend_handle.load_model(&model).await.map_err(|e| {
            self.activity_logger.log_model_operation(
                ActivityType::ModelLoad,
                &model_name,
                ActivityStatus::Error,
                Some(&format!("Failed to load model into backend: {}", e)),
            );
            e
        })?;

        let backend_id = Uuid::new_v4().to_string();

        {
            let mut loaded_backends = self.loaded_backends.lock().unwrap();
            loaded_backends.insert(backend_id.clone(), backend_handle);
        }

        // Update metrics
        {
            let mut metrics = self.global_metrics.lock().unwrap();
            metrics.models_loaded += 1;
        }

        // Log successful completion
        self.activity_logger.log_model_operation(
            ActivityType::ModelLoad,
            &model_name,
            ActivityStatus::Success,
            Some(&format!(
                "Model loaded successfully with backend ID: {}",
                backend_id
            )),
        );

        Ok(backend_id)
    }

    pub async fn unload_model(&self, backend_id: String) -> Result<()> {
        // Log the start of unload operation
        self.activity_logger.log_model_operation(
            ActivityType::ModelUnload,
            &backend_id,
            ActivityStatus::InProgress,
            Some("Unloading model from backend"),
        );

        let backend_handle = {
            let mut loaded_backends = self.loaded_backends.lock().unwrap();
            loaded_backends.remove(&backend_id)
        };

        if let Some(handle) = backend_handle {
            // Unload the model from the backend
            handle.unload_model().await.map_err(|e| {
                self.activity_logger.log_model_operation(
                    ActivityType::ModelUnload,
                    &backend_id,
                    ActivityStatus::Error,
                    Some(&format!("Failed to unload model: {}", e)),
                );
                e
            })?;

            // Update metrics
            let mut metrics = self.global_metrics.lock().unwrap();
            if metrics.models_loaded > 0 {
                metrics.models_loaded -= 1;
            }

            // Log successful completion
            self.activity_logger.log_model_operation(
                ActivityType::ModelUnload,
                &backend_id,
                ActivityStatus::Success,
                Some("Model unloaded successfully"),
            );

            Ok(())
        } else {
            self.activity_logger.log_model_operation(
                ActivityType::ModelUnload,
                &backend_id,
                ActivityStatus::Error,
                Some("Backend not found"),
            );
            Err(anyhow::anyhow!("Backend not found: {}", backend_id))
        }
    }

    pub fn get_loaded_models(&self) -> Vec<String> {
        let loaded_backends = self.loaded_backends.lock().unwrap();
        loaded_backends.keys().cloned().collect()
    }

    pub async fn infer(
        &self,
        backend_id: String,
        prompt: String,
        params: InferenceParams,
    ) -> Result<String> {
        let start_time = std::time::Instant::now();

        // Log the start of inference
        let prompt_tokens = prompt.split_whitespace().count() as u32;
        self.activity_logger.log_inference(
            &backend_id,
            prompt_tokens,
            0, // completion_tokens not known yet
            0, // duration not known yet
            ActivityStatus::InProgress,
        );

        // Get the backend handle
        let backend_handle = {
            let loaded_backends = self.loaded_backends.lock().unwrap();
            loaded_backends
                .get(&backend_id)
                .ok_or_else(|| {
                    self.activity_logger.log_inference(
                        &backend_id,
                        prompt_tokens,
                        0, // no completion tokens for error
                        start_time.elapsed().as_millis() as u64,
                        ActivityStatus::Error,
                    );
                    anyhow::anyhow!("Backend not found: {}", backend_id)
                })?
                .clone()
        };

        // Convert parameters
        let inferno_params = InfernoInferenceParams {
            max_tokens: params.max_tokens.unwrap_or(512),
            temperature: params.temperature.unwrap_or(0.7),
            top_p: params.top_p.unwrap_or(0.9),
            stream: params.stream.unwrap_or(false),
            stop_sequences: params.stop_sequences.clone().unwrap_or_default(),
            seed: params.seed,
        };

        // Track active inference count while the request is in-flight
        {
            let mut metrics = self.global_metrics.lock().unwrap();
            metrics.active_inferences += 1;
        }

        // Perform inference
        let result = backend_handle.infer(&prompt, &inferno_params).await;
        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        let (status, completion_tokens) = match &result {
            Ok(output) => (
                ActivityStatus::Success,
                output.split_whitespace().count() as u32,
            ),
            Err(_) => (ActivityStatus::Error, 0),
        };

        // Update metrics and log result
        {
            let mut metrics = self.global_metrics.lock().unwrap();

            if metrics.active_inferences > 0 {
                metrics.active_inferences -= 1;
            }

            metrics.inference_count += 1;

            match &status {
                ActivityStatus::Success => {
                    metrics.success_count += 1;

                    // Update rolling average latency
                    let current_avg = metrics.average_latency;
                    let count = metrics.inference_count as f64;
                    metrics.average_latency =
                        ((current_avg * (count - 1.0)) + elapsed_ms as f64) / count;
                }
                ActivityStatus::Error => {
                    metrics.error_count += 1;
                }
                _ => {}
            }
        }

        self.activity_logger.log_inference(
            &backend_id,
            prompt_tokens,
            completion_tokens,
            elapsed_ms,
            status,
        );

        result
    }

    pub fn begin_streaming_session(&self) -> StreamingSessionGuard {
        StreamingSessionGuard::new(Arc::clone(&self.global_metrics))
    }

    pub async fn get_model_info(&self, backend_id: &str) -> Option<ModelInfo> {
        let handle = {
            let loaded_backends = self.loaded_backends.lock().unwrap();
            loaded_backends.get(backend_id).cloned()
        };

        if let Some(handle) = handle {
            handle.get_model_info().await.map(Self::map_core_model_info)
        } else {
            None
        }
    }

    fn map_core_model_info(model: CoreModelInfo) -> ModelInfo {
        ModelInfo {
            id: model.name.clone(),
            name: model.name,
            path: model.path.to_string_lossy().to_string(),
            format: model.format,
            size: model.size_bytes,
            checksum: model.checksum.unwrap_or_else(|| "unknown".to_string()),
            status: "loaded".to_string(),
        }
    }

    pub async fn infer_stream(
        &self,
        backend_id: &str,
        prompt: &str,
        params: &InferenceParams,
    ) -> Result<TokenStream> {
        let backend_handle = {
            let loaded_backends = self.loaded_backends.lock().unwrap();
            loaded_backends
                .get(backend_id)
                .ok_or_else(|| anyhow::anyhow!("Backend not found: {}", backend_id))?
                .clone()
        };

        let inferno_params = InfernoInferenceParams {
            max_tokens: params.max_tokens.unwrap_or(512),
            temperature: params.temperature.unwrap_or(0.7),
            top_p: params.top_p.unwrap_or(0.9),
            stream: true,
            stop_sequences: params.stop_sequences.clone().unwrap_or_default(),
            seed: params.seed,
        };

        backend_handle.infer_stream(prompt, &inferno_params).await
    }

    pub fn get_metrics(&self) -> GlobalMetrics {
        let metrics = self.global_metrics.lock().unwrap();
        metrics.clone()
    }

    pub async fn validate_model(&self, model_path: String) -> Result<bool> {
        let model_manager = self.model_manager.read().await;
        Ok(model_manager
            .validate_model(&std::path::PathBuf::from(model_path))
            .await?)
    }

    /// Begin tracking an inference operation (returns a guard that auto-decrements on drop)
    pub fn begin_inference(&self) -> InferenceGuard {
        InferenceGuard::new(Arc::clone(&self.global_metrics))
    }

    /// Record the result of an inference operation
    pub fn record_inference_result(
        &self,
        _backend_id: &str,
        _prompt_tokens: u32,
        _completion_tokens: u32,
        latency_ms: u64,
        status: ActivityStatus,
    ) {
        let mut metrics = self.global_metrics.lock().unwrap();

        match status {
            ActivityStatus::Success => {
                metrics.success_count += 1;

                // Update rolling average latency
                let current_avg = metrics.average_latency;
                let count = metrics.inference_count as f64;
                if count > 0.0 {
                    metrics.average_latency =
                        ((current_avg * (count - 1.0)) + latency_ms as f64) / count;
                }
            }
            ActivityStatus::Error => {
                metrics.error_count += 1;
            }
            _ => {}
        }
    }

    /// Get the current number of active inferences
    pub fn get_active_inference_count(&self) -> u32 {
        let metrics = self.global_metrics.lock().unwrap();
        metrics.active_inferences
    }
}
