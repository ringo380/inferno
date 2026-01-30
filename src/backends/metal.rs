//! Metal GPU Backend for Apple Silicon
//!
//! **DEPRECATION NOTICE**: This standalone Metal backend is deprecated.
//! Metal GPU acceleration is now provided through the GGUF backend via
//! llama-cpp-2, which automatically enables Metal on macOS.
//!
//! ## Recommended Usage
//!
//! Use the GGUF backend instead:
//! ```ignore
//! let backend = GgufBackend::new(BackendConfig {
//!     gpu_enabled: true,  // Automatically uses Metal on macOS
//!     ..Default::default()
//! })?;
//! ```
//!
//! ## Why Deprecated?
//!
//! The llama-cpp-2 crate provides battle-tested Metal integration that:
//! - Automatically detects Apple Silicon
//! - Uses 999 GPU layers for maximum Metal utilization
//! - Handles memory management efficiently
//! - Supports all GGUF quantization formats
//!
//! This standalone Metal backend was intended for custom Metal shader work
//! but is not needed for production inference.
//!
//! ## GPU Detection
//!
//! The GPU detection code in this module is still useful and can be used
//! to query Metal device capabilities.
//!
//! ## Legacy Documentation
//!
//! Original features (not implemented in this backend, use GGUF instead):
//! - Native Metal GPU acceleration
//! - Unified memory architecture support
//! - Apple Neural Engine integration
//! - Quantized model support (Q4_0, Q4_1, Q5_0, Q5_1, Q8_0)
//! - Optimized for M1/M2/M3/M4 chips
//!
//! Performance targets (achieved via GGUF backend):
//! - 7B models: >30 tokens/sec on M1 Max
//! - 13B models: >15 tokens/sec on M2 Max
//! - 70B models: >5 tokens/sec on M4 Max (with unified memory)

use crate::{
    InfernoError,
    backends::{BackendType, InferenceBackend, InferenceMetrics, InferenceParams, TokenStream},
    models::ModelInfo,
};
use anyhow::{Result, anyhow};
use async_stream::stream;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Metal GPU backend implementation
///
/// This backend uses Metal Performance Shaders and direct Metal API access
/// for optimal performance on Apple Silicon.
pub struct MetalBackend {
    model_info: Option<ModelInfo>,
    is_loaded: bool,
    metrics: Option<InferenceMetrics>,

    // Metal-specific state
    gpu_memory_allocated: u64,
    max_gpu_memory: u64,
    supports_metal_3: bool,
    device_name: String,
}

impl MetalBackend {
    pub fn new() -> Result<Self> {
        #[cfg(not(target_os = "macos"))]
        {
            return Err(anyhow!("Metal backend is only available on macOS"));
        }

        #[cfg(target_os = "macos")]
        {
            // Detect Metal capabilities
            let (device_name, supports_metal_3, max_gpu_memory) = Self::detect_metal_device()?;

            info!(
                "🔥 Metal backend initialized: {} (Metal 3: {})",
                device_name,
                if supports_metal_3 { "✅" } else { "❌" }
            );

            Ok(Self {
                model_info: None,
                is_loaded: false,
                metrics: None,
                gpu_memory_allocated: 0,
                max_gpu_memory,
                supports_metal_3,
                device_name,
            })
        }
    }

    #[cfg(target_os = "macos")]
    fn detect_metal_device() -> Result<(String, bool, u64)> {
        use std::process::Command;

        // Use system_profiler to detect Metal device
        let output = Command::new("system_profiler")
            .arg("SPDisplaysDataType")
            .arg("-json")
            .output()
            .map_err(|e| anyhow!("Failed to detect Metal device: {}", e))?;

        if !output.status.success() {
            return Err(anyhow!("system_profiler failed"));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);

        // Parse device information
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
            if let Some(displays) = json["SPDisplaysDataType"].as_array() {
                for display in displays {
                    if let Some(chipset) = display["sppci_model"].as_str() {
                        let supports_metal_3 =
                            chipset.contains("Apple M") || chipset.contains("AMD");

                        // Get unified memory size
                        let max_memory = if chipset.contains("Apple") {
                            // For Apple Silicon, use total system memory as GPU can access unified memory
                            let mem_output = Command::new("sysctl")
                                .arg("-n")
                                .arg("hw.memsize")
                                .output()
                                .ok()
                                .and_then(|out| {
                                    String::from_utf8_lossy(&out.stdout)
                                        .trim()
                                        .parse::<u64>()
                                        .ok()
                                })
                                .unwrap_or(8 * 1024 * 1024 * 1024); // Default 8GB

                            mem_output
                        } else {
                            // For discrete GPUs, parse VRAM
                            if let Some(vram) = display["sppci_vram"].as_str() {
                                if vram.contains("GB") {
                                    vram.split_whitespace()
                                        .next()
                                        .and_then(|s| s.parse::<u64>().ok())
                                        .unwrap_or(2)
                                        * 1024
                                        * 1024
                                        * 1024
                                } else {
                                    2 * 1024 * 1024 * 1024 // Default 2GB
                                }
                            } else {
                                2 * 1024 * 1024 * 1024
                            }
                        };

                        return Ok((chipset.to_string(), supports_metal_3, max_memory));
                    }
                }
            }
        }

        // Fallback values
        Ok((
            "Unknown Metal Device".to_string(),
            false,
            2 * 1024 * 1024 * 1024,
        ))
    }

    fn estimate_model_memory(&self, model_info: &ModelInfo) -> u64 {
        // Estimate based on model size
        // For GGUF models, the file size is a good approximation
        model_info.size_bytes
    }

    fn check_memory_available(&self, required: u64) -> Result<()> {
        if required > self.max_gpu_memory {
            return Err(anyhow!(
                "Model requires {}GB but only {}GB available",
                required / (1024 * 1024 * 1024),
                self.max_gpu_memory / (1024 * 1024 * 1024)
            ));
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl InferenceBackend for MetalBackend {
    async fn load_model(&mut self, model_info: &ModelInfo) -> Result<()> {
        info!("📥 Loading model with Metal backend: {}", model_info.name);

        // Check memory requirements
        let required_memory = self.estimate_model_memory(model_info);
        self.check_memory_available(required_memory)?;

        // Validate model format
        if !model_info.path.to_str().unwrap_or("").ends_with(".gguf") {
            return Err(anyhow!(
                "Metal backend currently only supports GGUF models. Got: {:?}",
                model_info.path.extension()
            ));
        }

        // TODO: Phase 2.3 - Actual Metal model loading
        // For now, we'll use a placeholder implementation

        info!(
            "⚡ Metal GPU loading model: {} ({}MB)",
            model_info.name,
            model_info.size_bytes / (1024 * 1024)
        );

        self.model_info = Some(model_info.clone());
        self.is_loaded = true;
        self.gpu_memory_allocated = required_memory;

        info!(
            "✅ Model loaded on Metal GPU ({}/{}GB used)",
            self.gpu_memory_allocated / (1024 * 1024 * 1024),
            self.max_gpu_memory / (1024 * 1024 * 1024)
        );

        Ok(())
    }

    async fn unload_model(&mut self) -> Result<()> {
        if !self.is_loaded {
            return Ok(());
        }

        info!("🗑️  Unloading model from Metal GPU");

        // TODO: Phase 2.3 - Actual Metal cleanup

        self.model_info = None;
        self.is_loaded = false;
        self.gpu_memory_allocated = 0;
        self.metrics = None;

        Ok(())
    }

    async fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    async fn get_model_info(&self) -> Option<ModelInfo> {
        self.model_info.clone()
    }

    async fn infer(&mut self, input: &str, params: &InferenceParams) -> Result<String> {
        if !self.is_loaded {
            return Err(anyhow!("No model loaded"));
        }

        let start = Instant::now();

        debug!(
            "🔮 Metal inference: {} tokens, temp={}, top_p={}",
            params.max_tokens, params.temperature, params.top_p
        );

        // TODO: Phase 2.3 - Actual Metal inference
        // Placeholder implementation for now

        let response = format!(
            "[Metal GPU Inference Placeholder]\nInput: {}\nModel: {}\nDevice: {}\nMetal 3: {}",
            input,
            self.model_info
                .as_ref()
                .map(|m| m.name.as_str())
                .unwrap_or("unknown"),
            self.device_name,
            if self.supports_metal_3 { "Yes" } else { "No" }
        );

        let elapsed = start.elapsed();

        // Update metrics
        let prompt_tokens = input.split_whitespace().count() as u32;
        let completion_tokens = response.split_whitespace().count() as u32;
        let total_tokens = prompt_tokens + completion_tokens;

        self.metrics = Some(InferenceMetrics {
            total_tokens,
            prompt_tokens,
            completion_tokens,
            total_time_ms: elapsed.as_millis() as u64,
            tokens_per_second: (completion_tokens as f32) / elapsed.as_secs_f32(),
            prompt_time_ms: 0,
            completion_time_ms: elapsed.as_millis() as u64,
        });

        debug!(
            "✅ Metal inference complete: {:.2} tokens/sec",
            (completion_tokens as f32) / elapsed.as_secs_f32()
        );

        Ok(response)
    }

    async fn infer_stream(&mut self, input: &str, params: &InferenceParams) -> Result<TokenStream> {
        if !self.is_loaded {
            return Err(anyhow!("No model loaded"));
        }

        debug!("🌊 Metal streaming inference");

        // TODO: Phase 2.3 - Actual Metal streaming
        // Placeholder implementation

        let response = self.infer(input, params).await?;
        let tokens: Vec<String> = response.split_whitespace().map(|s| s.to_string()).collect();

        let token_stream = stream! {
            for token in tokens {
                yield Ok(format!("{} ", token));
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        };

        Ok(Box::pin(token_stream))
    }

    async fn get_embeddings(&mut self, input: &str) -> Result<Vec<f32>> {
        if !self.is_loaded {
            return Err(anyhow!("No model loaded"));
        }

        debug!("🔢 Metal embeddings generation");

        // TODO: Phase 2.3 - Actual Metal embeddings
        // Placeholder implementation

        warn!("Metal embeddings not yet implemented, returning placeholder");

        // Return placeholder embeddings (768-dim for compatibility)
        Ok(vec![0.0; 768])
    }

    fn get_backend_type(&self) -> BackendType {
        BackendType::Metal
    }

    fn get_metrics(&self) -> Option<InferenceMetrics> {
        self.metrics.clone()
    }
}

impl Default for MetalBackend {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            panic!("Failed to initialize Metal backend: {}", e);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(target_os = "macos")]
    async fn test_metal_backend_creation() {
        let backend = MetalBackend::new();
        assert!(backend.is_ok(), "Should create Metal backend on macOS");

        let backend = backend.unwrap();
        assert!(!backend.is_loaded);
        assert!(backend.max_gpu_memory > 0);
    }

    #[tokio::test]
    #[cfg(not(target_os = "macos"))]
    async fn test_metal_backend_non_macos() {
        let backend = MetalBackend::new();
        assert!(backend.is_err(), "Should fail on non-macOS platforms");
    }
}
