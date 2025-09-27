use anyhow::Result;
use inferno::backends::{Backend, BackendHandle, BackendType, BackendConfig, InferenceParams as InfernoInferenceParams, InferenceMetrics};
use inferno::models::{ModelInfo as InfernoModelInfo, ModelManager};
use inferno::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use uuid::Uuid;
use std::path::PathBuf;
use crate::activity_logger::{ActivityLogger, ActivityType, ActivityStatus};

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
}

pub struct BackendManager {
    config: Arc<RwLock<Config>>,
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
}

impl BackendManager {
    pub async fn new(activity_logger: Arc<ActivityLogger>) -> Result<Self> {
        // Try to load config from .inferno.toml, fallback to default
        let config = match std::env::current_dir()
            .and_then(|mut path| {
                // Try current directory first, then parent directories
                for _ in 0..3 {
                    path.push(".inferno.toml");
                    if path.exists() {
                        return Ok(path);
                    }
                    path.pop();
                    path.pop();
                }
                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Config not found"))
            })
            .and_then(|config_path| {
                let content = std::fs::read_to_string(&config_path)?;
                toml::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            }) {
            Ok(config) => config,
            Err(_) => Config::default(),
        };

        let models_dir = config.models_dir.clone();
        let model_manager = ModelManager::new(&models_dir);

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            model_manager: Arc::new(RwLock::new(model_manager)),
            loaded_backends: Arc::new(Mutex::new(HashMap::new())),
            global_metrics: Arc::new(Mutex::new(GlobalMetrics::default())),
            activity_logger,
        })
    }

    pub async fn discover_models(&self) -> Result<Vec<ModelInfo>> {
        let model_manager = self.model_manager.read().await;
        let models = model_manager.list_models().await?;

        Ok(models.into_iter().map(|m| ModelInfo {
            id: m.name.clone(), // Use name as ID
            name: m.name.clone(),
            path: m.path.to_string_lossy().to_string(),
            format: m.backend_type.clone(),
            size: m.size,
            checksum: m.checksum.unwrap_or_else(|| "unknown".to_string()),
            status: "available".to_string(),
        }).collect())
    }

    pub async fn load_model(&self, model_name: String, backend_type_str: String) -> Result<String> {
        // Log the start of the operation
        self.activity_logger.log_model_operation(
            ActivityType::ModelLoad,
            &model_name,
            ActivityStatus::InProgress,
            Some(&format!("Loading model with {} backend", backend_type_str))
        );

        let model_manager = self.model_manager.read().await;

        // Find the model
        let model = model_manager.resolve_model(&model_name).await.map_err(|e| {
            self.activity_logger.log_model_operation(
                ActivityType::ModelLoad,
                &model_name,
                ActivityStatus::Error,
                Some(&format!("Failed to resolve model: {}", e))
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
        let backend_handle = BackendHandle::new_shared(backend_type, &backend_config).map_err(|e| {
            self.activity_logger.log_model_operation(
                ActivityType::ModelLoad,
                &model_name,
                ActivityStatus::Error,
                Some(&format!("Failed to create backend: {}", e))
            );
            e
        })?;

        backend_handle.load_model(&model).await.map_err(|e| {
            self.activity_logger.log_model_operation(
                ActivityType::ModelLoad,
                &model_name,
                ActivityStatus::Error,
                Some(&format!("Failed to load model into backend: {}", e))
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
            Some(&format!("Model loaded successfully with backend ID: {}", backend_id))
        );

        Ok(backend_id)
    }

    pub async fn unload_model(&self, backend_id: String) -> Result<()> {
        // Log the start of unload operation
        self.activity_logger.log_model_operation(
            ActivityType::ModelUnload,
            &backend_id,
            ActivityStatus::InProgress,
            Some("Unloading model from backend")
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
                    Some(&format!("Failed to unload model: {}", e))
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
                Some("Model unloaded successfully")
            );

            Ok(())
        } else {
            self.activity_logger.log_model_operation(
                ActivityType::ModelUnload,
                &backend_id,
                ActivityStatus::Error,
                Some("Backend not found")
            );
            Err(anyhow::anyhow!("Backend not found: {}", backend_id))
        }
    }

    pub fn get_loaded_models(&self) -> Vec<String> {
        let loaded_backends = self.loaded_backends.lock().unwrap();
        loaded_backends.keys().cloned().collect()
    }

    pub async fn infer(&self, backend_id: String, prompt: String, params: InferenceParams) -> Result<String> {
        let start_time = std::time::Instant::now();

        // Log the start of inference
        let prompt_tokens = prompt.split_whitespace().count() as u32;
        self.activity_logger.log_inference(
            &backend_id,
            prompt_tokens,
            0, // completion_tokens not known yet
            0, // duration not known yet
            ActivityStatus::InProgress
        );

        // Get the backend handle
        let backend_handle = {
            let loaded_backends = self.loaded_backends.lock().unwrap();
            loaded_backends.get(&backend_id)
                .ok_or_else(|| {
                    self.activity_logger.log_inference(
                        &backend_id,
                        prompt_tokens,
                        0, // no completion tokens for error
                        start_time.elapsed().as_millis() as u64,
                        ActivityStatus::Error
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
        };

        // Perform inference
        let result = backend_handle.infer(&prompt, &inferno_params).await;

        // Update metrics and log result
        {
            let mut metrics = self.global_metrics.lock().unwrap();
            let elapsed = start_time.elapsed().as_millis() as f64;

            match &result {
                Ok(output) => {
                    metrics.inference_count += 1;
                    metrics.success_count += 1;

                    // Update rolling average latency
                    let current_avg = metrics.average_latency;
                    let count = metrics.inference_count as f64;
                    metrics.average_latency = ((current_avg * (count - 1.0)) + elapsed) / count;

                    // Log successful inference
                    let completion_tokens = output.split_whitespace().count() as u32;
                    self.activity_logger.log_inference(
                        &backend_id,
                        prompt_tokens,
                        completion_tokens,
                        elapsed as u64,
                        ActivityStatus::Success
                    );
                }
                Err(e) => {
                    metrics.inference_count += 1;
                    metrics.error_count += 1;

                    // Log failed inference
                    self.activity_logger.log_inference(
                        &backend_id,
                        prompt_tokens,
                        0, // no completion tokens for error
                        elapsed as u64,
                        ActivityStatus::Error
                    );
                }
            }
        }

        result
    }

    pub fn get_metrics(&self) -> GlobalMetrics {
        let metrics = self.global_metrics.lock().unwrap();
        metrics.clone()
    }

    pub async fn validate_model(&self, model_path: String) -> Result<bool> {
        let model_manager = self.model_manager.read().await;
        Ok(model_manager.validate_model(&std::path::PathBuf::from(model_path)).await?)
    }
}