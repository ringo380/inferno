//! Type Definitions for Desktop Interface
//!
//! This module contains all the shared types used by the desktop application.
//! These types are used for serialization/deserialization between Rust and
//! the frontend TypeScript code.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// System Information Types
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemInfo {
    pub cpu_name: String,
    pub cpu_usage: f32,
    pub cpu_cores: usize,
    pub total_memory: u64,
    pub used_memory: u64,
    pub available_memory: u64,
    pub platform: String,
    pub arch: String,
    // New GPU and chip info (Phase 2)
    pub gpu_info: Option<GpuInfo>,
    pub chip_info: Option<ChipInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GpuInfo {
    pub available: bool,
    pub device_name: String,
    pub memory_gb: f64,
    pub supports_metal_3: bool,
    pub vendor: String, // "Apple", "AMD", "NVIDIA", etc.
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChipInfo {
    pub is_apple_silicon: bool,
    pub chip_name: String,
    pub performance_cores: u32,
    pub efficiency_cores: u32,
    pub neural_engine: bool,
    pub total_cores: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MetricsSnapshot {
    pub inference_count: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub average_latency: f64,
    pub models_loaded: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InfernoMetrics {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub gpu_usage: Option<f32>,
    pub active_models: u32,
    pub active_inferences: u32,
    pub inference_count: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub average_latency: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActiveProcessInfo {
    pub active_models: Vec<String>,
    pub active_inferences: u32,
    pub batch_jobs: u32,
    pub streaming_sessions: u32,
}

// ============================================================================
// Settings Types
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub types: NotificationTypes,
    pub priority_filter: String,
    pub auto_dismiss_after: u32,
    pub max_notifications: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotificationTypes {
    pub system: bool,
    pub inference: bool,
    pub security: bool,
    pub batch: bool,
    pub model: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    // Model Settings
    pub models_directory: String,
    pub auto_discover_models: bool,

    // Inference Settings
    pub default_temperature: f64,
    pub default_max_tokens: u32,
    pub default_top_p: f64,
    pub default_top_k: u32,

    // System Settings
    pub max_memory_usage: u32,
    pub prefer_gpu: bool,
    pub max_concurrent_inferences: u32,

    // Cache Settings
    pub enable_cache: bool,
    pub cache_directory: String,
    pub max_cache_size: u32,

    // API Settings
    pub enable_rest_api: bool,
    pub api_port: u32,
    pub enable_cors: bool,

    // Security Settings
    pub require_authentication: bool,
    pub enable_audit_log: bool,
    pub log_level: String,

    // Notification Settings
    pub notifications: NotificationSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            models_directory: "test_models/test_models".to_string(),
            auto_discover_models: true,
            default_temperature: 0.7,
            default_max_tokens: 512,
            default_top_p: 0.9,
            default_top_k: 40,
            max_memory_usage: 80,
            prefer_gpu: true,
            max_concurrent_inferences: 3,
            enable_cache: true,
            cache_directory: ".cache".to_string(),
            max_cache_size: 1024,
            enable_rest_api: false,
            api_port: 8080,
            enable_cors: true,
            require_authentication: false,
            enable_audit_log: true,
            log_level: "info".to_string(),
            notifications: NotificationSettings {
                enabled: true,
                types: NotificationTypes {
                    system: true,
                    inference: true,
                    security: true,
                    batch: true,
                    model: true,
                },
                priority_filter: "all".to_string(),
                auto_dismiss_after: 0,
                max_notifications: 100,
            },
        }
    }
}

// ============================================================================
// Notification Types
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotificationAction {
    pub label: String,
    pub url: Option<String>,
    pub callback: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: String, // 'info' | 'success' | 'warning' | 'error'
    pub timestamp: String,
    pub read: bool,
    pub action: Option<NotificationAction>,
    pub source: String, // 'system' | 'inference' | 'security' | 'batch' | 'model'
    pub priority: String, // 'low' | 'medium' | 'high' | 'critical'
    pub metadata: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Batch Job Types
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BatchJobConfig {
    pub inputs: Vec<String>,
    pub output_format: String,
    pub batch_size: u32,
    pub parallel_workers: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchJobMetrics {
    pub total_time: f64,
    pub avg_time_per_task: f64,
    pub throughput: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchJobResults {
    pub outputs: Vec<String>,
    pub errors: Vec<String>,
    pub metrics: BatchJobMetrics,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchJob {
    pub id: String,
    pub name: String,
    pub status: String, // 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
    pub model_id: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub progress: f64,
    pub total_tasks: u32,
    pub completed_tasks: u32,
    pub failed_tasks: u32,
    pub schedule: Option<String>,
    pub next_run: Option<String>,
    pub config: BatchJobConfig,
    pub results: Option<BatchJobResults>,
}