use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use sysinfo::System;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "data")]
pub enum InfernoEvent {
    // System Events
    SystemMetricsUpdated {
        cpu_usage: f32,
        memory_usage: u64,
        timestamp: String,
    },

    // Model Events
    ModelLoaded {
        model_id: String,
        backend_id: String,
        timestamp: String,
    },
    ModelUnloaded {
        model_id: String,
        backend_id: String,
        timestamp: String,
    },
    ModelDiscovered {
        model_info: crate::backend_manager::ModelInfo,
        timestamp: String,
    },

    // Inference Events
    InferenceStarted {
        inference_id: String,
        model_id: String,
        timestamp: String,
    },
    InferenceProgress {
        inference_id: String,
        progress: f32,
        partial_response: Option<String>,
        timestamp: String,
    },
    InferenceCompleted {
        inference_id: String,
        response: String,
        latency_ms: u64,
        timestamp: String,
    },
    InferenceError {
        inference_id: String,
        error: String,
        timestamp: String,
    },

    // Batch Job Events
    BatchJobCreated {
        job_id: String,
        name: String,
        timestamp: String,
    },
    BatchJobStarted {
        job_id: String,
        timestamp: String,
    },
    BatchJobProgress {
        job_id: String,
        progress: f64,
        completed_tasks: u32,
        failed_tasks: u32,
        timestamp: String,
    },
    BatchJobCompleted {
        job_id: String,
        timestamp: String,
    },
    BatchJobFailed {
        job_id: String,
        error: String,
        timestamp: String,
    },

    // Security Events
    SecurityEvent {
        event_id: String,
        event_type: String,
        severity: String,
        description: String,
        source_ip: Option<String>,
        timestamp: String,
    },
    ApiKeyCreated {
        key_id: String,
        name: String,
        permissions: Vec<String>,
        timestamp: String,
    },
    ApiKeyRevoked {
        key_id: String,
        reason: String,
        timestamp: String,
    },

    // Settings Events
    SettingsUpdated {
        settings: serde_json::Value,
        timestamp: String,
    },

    // Activity Events
    ActivityLogged {
        activity_type: String,
        description: String,
        timestamp: String,
    },

    // Connection Events
    ConnectionStatusChanged {
        status: ConnectionStatus,
        timestamp: String,
    },

    // Notification Events
    NotificationCreated {
        notification: crate::Notification,
        timestamp: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Reconnecting,
    Error { message: String },
}

#[derive(Clone)]
pub struct EventManager {
    app_handle: AppHandle,
    event_history: std::sync::Arc<std::sync::Mutex<Vec<InfernoEvent>>>,
    subscribers: std::sync::Arc<std::sync::Mutex<HashMap<String, Vec<String>>>>, // event_type -> client_ids
    system: Arc<Mutex<System>>,
}

impl EventManager {
    pub fn new(app_handle: AppHandle, system: Arc<Mutex<System>>) -> Self {
        Self {
            app_handle,
            event_history: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            subscribers: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            system,
        }
    }

    pub fn emit_event(&self, event: InfernoEvent) -> Result<(), String> {
        // Store in history
        if let Ok(mut history) = self.event_history.lock() {
            history.push(event.clone());
            // Keep only last 1000 events
            if history.len() > 1000 {
                let len = history.len();
                history.drain(0..len - 1000);
            }
        }

        // Emit to frontend
        self.app_handle
            .emit("inferno_event", &event)
            .map_err(|e| e.to_string())?;

        // Also emit specific event types for targeted listening
        match &event {
            InfernoEvent::SystemMetricsUpdated { .. } => {
                self.app_handle
                    .emit("system_metrics_updated", &event)
                    .map_err(|e| e.to_string())?;
            }
            InfernoEvent::ModelLoaded { .. } | InfernoEvent::ModelUnloaded { .. } | InfernoEvent::ModelDiscovered { .. } => {
                self.app_handle
                    .emit("model_updated", &event)
                    .map_err(|e| e.to_string())?;
            }
            InfernoEvent::InferenceStarted { .. } | InfernoEvent::InferenceProgress { .. } |
            InfernoEvent::InferenceCompleted { .. } | InfernoEvent::InferenceError { .. } => {
                self.app_handle
                    .emit("inference_updated", &event)
                    .map_err(|e| e.to_string())?;
            }
            InfernoEvent::BatchJobCreated { .. } | InfernoEvent::BatchJobStarted { .. } |
            InfernoEvent::BatchJobProgress { .. } | InfernoEvent::BatchJobCompleted { .. } |
            InfernoEvent::BatchJobFailed { .. } => {
                self.app_handle
                    .emit("batch_job_updated", &event)
                    .map_err(|e| e.to_string())?;
            }
            InfernoEvent::SecurityEvent { .. } | InfernoEvent::ApiKeyCreated { .. } |
            InfernoEvent::ApiKeyRevoked { .. } => {
                self.app_handle
                    .emit("security_updated", &event)
                    .map_err(|e| e.to_string())?;
            }
            _ => {}
        }

        Ok(())
    }

    pub fn get_event_history(&self, limit: Option<usize>) -> Vec<InfernoEvent> {
        if let Ok(history) = self.event_history.lock() {
            let limit = limit.unwrap_or(100);
            if history.len() <= limit {
                history.clone()
            } else {
                history[history.len() - limit..].to_vec()
            }
        } else {
            Vec::new()
        }
    }

    pub fn emit_system_metrics(&self, cpu_usage: f32, memory_usage: u64) -> Result<(), String> {
        self.emit_event(InfernoEvent::SystemMetricsUpdated {
            cpu_usage,
            memory_usage,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn emit_model_loaded(&self, model_id: String, backend_id: String) -> Result<(), String> {
        self.emit_event(InfernoEvent::ModelLoaded {
            model_id,
            backend_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn emit_model_unloaded(&self, model_id: String, backend_id: String) -> Result<(), String> {
        self.emit_event(InfernoEvent::ModelUnloaded {
            model_id,
            backend_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn emit_inference_started(&self, inference_id: String, model_id: String) -> Result<(), String> {
        self.emit_event(InfernoEvent::InferenceStarted {
            inference_id,
            model_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn emit_inference_progress(&self, inference_id: String, progress: f32, partial_response: Option<String>) -> Result<(), String> {
        self.emit_event(InfernoEvent::InferenceProgress {
            inference_id,
            progress,
            partial_response,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn emit_inference_completed(&self, inference_id: String, response: String, latency_ms: u64) -> Result<(), String> {
        self.emit_event(InfernoEvent::InferenceCompleted {
            inference_id,
            response,
            latency_ms,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn emit_inference_error(&self, inference_id: String, error: String) -> Result<(), String> {
        self.emit_event(InfernoEvent::InferenceError {
            inference_id,
            error,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn emit_batch_job_progress(&self, job_id: String, progress: f64, completed_tasks: u32, failed_tasks: u32) -> Result<(), String> {
        self.emit_event(InfernoEvent::BatchJobProgress {
            job_id,
            progress,
            completed_tasks,
            failed_tasks,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn emit_security_event(&self, event_type: String, severity: String, description: String, source_ip: Option<String>) -> Result<(), String> {
        self.emit_event(InfernoEvent::SecurityEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type,
            severity,
            description,
            source_ip,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn emit_api_key_created(&self, key_id: String, name: String, permissions: Vec<String>) -> Result<(), String> {
        self.emit_event(InfernoEvent::ApiKeyCreated {
            key_id,
            name,
            permissions,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn emit_settings_updated(&self, settings: serde_json::Value) -> Result<(), String> {
        self.emit_event(InfernoEvent::SettingsUpdated {
            settings,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn emit_activity_logged(&self, activity_type: String, description: String) -> Result<(), String> {
        self.emit_event(InfernoEvent::ActivityLogged {
            activity_type,
            description,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn emit_notification(&self, notification: crate::Notification) -> Result<(), String> {
        self.emit_event(InfernoEvent::NotificationCreated {
            notification,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn emit_connection_status(&self, status: ConnectionStatus) -> Result<(), String> {
        self.emit_event(InfernoEvent::ConnectionStatusChanged {
            status,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    // Helper method to start periodic system metrics emission
    pub fn start_metrics_emission(&self) -> Result<(), String> {
        let event_manager = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

            loop {
                interval.tick().await;

                // Get real system metrics
                if let Ok(mut system) = event_manager.system.lock() {
                    system.refresh_cpu();
                    system.refresh_memory();

                    let cpu_usage = system.global_cpu_info().cpu_usage();
                    let memory_usage = system.used_memory();

                    if let Err(e) = event_manager.emit_system_metrics(cpu_usage, memory_usage) {
                        eprintln!("Failed to emit system metrics: {}", e);
                    }
                } else {
                    eprintln!("Failed to acquire system lock for metrics");
                }
            }
        });

        // Emit initial connection status
        self.emit_connection_status(ConnectionStatus::Connected)
    }
}

// Event subscription management for frontend
#[derive(Serialize, Deserialize)]
pub struct EventSubscription {
    pub client_id: String,
    pub event_types: Vec<String>,
}

impl EventManager {
    pub fn subscribe(&self, client_id: String, event_types: Vec<String>) -> Result<(), String> {
        let mut subscribers = self.subscribers.lock().map_err(|e| e.to_string())?;

        for event_type in event_types {
            subscribers
                .entry(event_type)
                .or_insert_with(Vec::new)
                .push(client_id.clone());
        }

        Ok(())
    }

    pub fn unsubscribe(&self, client_id: String) -> Result<(), String> {
        let mut subscribers = self.subscribers.lock().map_err(|e| e.to_string())?;

        for clients in subscribers.values_mut() {
            clients.retain(|id| id != &client_id);
        }

        Ok(())
    }
}