//! Event Emission System
//!
//! This module handles event emission from the Rust backend to the frontend.
//! Events are used for real-time updates, notifications, and state changes.

use tauri::{AppHandle, Manager};

/// Event manager for emitting events to the frontend
pub struct EventManager {
    app_handle: AppHandle,
}

impl EventManager {
    /// Create a new EventManager
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// Emit a model loaded event
    pub fn emit_model_loaded(&self, model_name: String, backend_id: String) -> Result<(), String> {
        self.app_handle
            .emit_all("model_loaded", serde_json::json!({
                "model_name": model_name,
                "backend_id": backend_id,
            }))
            .map_err(|e| e.to_string())
    }

    /// Emit a model unloaded event
    pub fn emit_model_unloaded(&self, model_name: String, backend_id: String) -> Result<(), String> {
        self.app_handle
            .emit_all("model_unloaded", serde_json::json!({
                "model_name": model_name,
                "backend_id": backend_id,
            }))
            .map_err(|e| e.to_string())
    }

    /// Emit an inference started event
    pub fn emit_inference_started(&self, inference_id: String, backend_id: String) -> Result<(), String> {
        self.app_handle
            .emit_all("inference_started", serde_json::json!({
                "inference_id": inference_id,
                "backend_id": backend_id,
            }))
            .map_err(|e| e.to_string())
    }

    /// Emit an inference completed event
    pub fn emit_inference_completed(&self, inference_id: String, response: String, latency_ms: u64) -> Result<(), String> {
        self.app_handle
            .emit_all("inference_completed", serde_json::json!({
                "inference_id": inference_id,
                "response": response,
                "latency_ms": latency_ms,
            }))
            .map_err(|e| e.to_string())
    }

    /// Emit an inference error event
    pub fn emit_inference_error(&self, inference_id: String, error: String) -> Result<(), String> {
        self.app_handle
            .emit_all("inference_error", serde_json::json!({
                "inference_id": inference_id,
                "error": error,
            }))
            .map_err(|e| e.to_string())
    }

    /// Emit a notification event
    pub fn emit_notification(&self, notification: serde_json::Value) -> Result<(), String> {
        self.app_handle
            .emit_all("notification", notification)
            .map_err(|e| e.to_string())
    }

    /// Emit an API key created event
    pub fn emit_api_key_created(&self, key_id: String, name: String, permissions: Vec<String>) -> Result<(), String> {
        self.app_handle
            .emit_all("api_key_created", serde_json::json!({
                "key_id": key_id,
                "name": name,
                "permissions": permissions,
            }))
            .map_err(|e| e.to_string())
    }

    /// Emit a metrics update event (periodic)
    pub fn emit_metrics_update(&self, metrics: serde_json::Value) -> Result<(), String> {
        self.app_handle
            .emit_all("metrics_update", metrics)
            .map_err(|e| e.to_string())
    }

    /// Start periodic metrics emission
    pub fn start_metrics_emission(&self) -> Result<(), String> {
        // TODO: Implement periodic metrics emission using tokio::spawn
        // This will emit metrics every 1-5 seconds
        tracing::info!("📊 Starting periodic metrics emission");
        Ok(())
    }
}
