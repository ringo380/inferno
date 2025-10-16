//! Event Emission System
//!
//! This module handles event emission from the Rust backend to the frontend.
//! Events are used for real-time updates, notifications, and state changes.

use tauri::{AppHandle, Emitter};
use chrono::Utc;

/// Event manager for emitting events to the frontend
pub struct EventManager {
    app_handle: AppHandle,
}

impl EventManager {
    /// Create a new EventManager
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// Emit a unified inferno_event with proper structure
    fn emit_inferno_event(&self, event_type: &str, data: serde_json::Value) -> Result<(), String> {
        let event = serde_json::json!({
            "type": event_type,
            "data": data
        });

        self.app_handle
            .emit("inferno_event", &event)
            .map_err(|e| e.to_string())
    }

    /// Emit a model loaded event
    pub fn emit_model_loaded(&self, model_name: String, backend_id: String) -> Result<(), String> {
        self.emit_inferno_event(
            "ModelLoaded",
            serde_json::json!({
                "model_id": model_name,
                "backend_id": backend_id,
                "timestamp": Utc::now().to_rfc3339(),
            })
        )
    }

    /// Emit a model unloaded event
    pub fn emit_model_unloaded(&self, model_name: String, backend_id: String) -> Result<(), String> {
        self.emit_inferno_event(
            "ModelUnloaded",
            serde_json::json!({
                "model_id": model_name,
                "backend_id": backend_id,
                "timestamp": Utc::now().to_rfc3339(),
            })
        )
    }

    /// Emit an inference started event
    pub fn emit_inference_started(&self, inference_id: String, backend_id: String) -> Result<(), String> {
        self.emit_inferno_event(
            "InferenceStarted",
            serde_json::json!({
                "inference_id": inference_id,
                "model_id": backend_id,
                "timestamp": Utc::now().to_rfc3339(),
            })
        )
    }

    /// Emit an inference completed event
    pub fn emit_inference_completed(&self, inference_id: String, response: String, latency_ms: u64) -> Result<(), String> {
        self.emit_inferno_event(
            "InferenceCompleted",
            serde_json::json!({
                "inference_id": inference_id,
                "response": response,
                "latency_ms": latency_ms,
                "timestamp": Utc::now().to_rfc3339(),
            })
        )
    }

    /// Emit an inference error event
    pub fn emit_inference_error(&self, inference_id: String, error: String) -> Result<(), String> {
        self.emit_inferno_event(
            "InferenceError",
            serde_json::json!({
                "inference_id": inference_id,
                "error": error,
                "timestamp": Utc::now().to_rfc3339(),
            })
        )
    }

    /// Emit a system metrics update event
    pub fn emit_system_metrics(&self, cpu_usage: f32, memory_usage: u64) -> Result<(), String> {
        self.emit_inferno_event(
            "SystemMetricsUpdated",
            serde_json::json!({
                "cpu_usage": cpu_usage,
                "memory_usage": memory_usage,
                "timestamp": Utc::now().to_rfc3339(),
            })
        )
    }

    /// Emit a notification event
    pub fn emit_notification(&self, notification: serde_json::Value) -> Result<(), String> {
        self.app_handle
            .emit("notification", notification)
            .map_err(|e| e.to_string())
    }

    /// Emit an API key created event
    pub fn emit_api_key_created(&self, key_id: String, name: String, permissions: Vec<String>) -> Result<(), String> {
        self.emit_inferno_event(
            "ApiKeyCreated",
            serde_json::json!({
                "key_id": key_id,
                "name": name,
                "permissions": permissions,
                "timestamp": Utc::now().to_rfc3339(),
            })
        )
    }

    /// Emit a metrics update event (periodic)
    pub fn emit_metrics_update(&self, metrics: serde_json::Value) -> Result<(), String> {
        self.app_handle
            .emit("metrics_update", metrics)
            .map_err(|e| e.to_string())
    }

    /// Start periodic metrics emission
    pub fn start_metrics_emission(&self) -> Result<(), String> {
        tracing::info!("📊 Starting periodic metrics emission");

        let app_handle = self.app_handle.clone();

        tokio::spawn(async move {
            use sysinfo::{System, SystemExt, CpuExt};

            let mut system = System::new_all();
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3));

            loop {
                interval.tick().await;

                // Refresh system information
                system.refresh_cpu();
                system.refresh_memory();

                let cpu_usage = system.global_cpu_info().cpu_usage();
                let memory_usage = system.used_memory();

                // Emit system metrics event
                let event = serde_json::json!({
                    "type": "SystemMetricsUpdated",
                    "data": {
                        "cpu_usage": cpu_usage,
                        "memory_usage": memory_usage,
                        "timestamp": Utc::now().to_rfc3339(),
                    }
                });

                if let Err(e) = app_handle.emit("inferno_event", &event) {
                    tracing::warn!("Failed to emit system metrics: {}", e);
                }
            }
        });

        Ok(())
    }
}
