//! Tauri Command Handlers Implementation
//!
//! This file contains all 51 Tauri command handlers migrated from dashboard/src-tauri.
//! Commands are organized by functionality for maintainability.
//!
//! ## Command Categories:
//! - Core Model Operations (5 commands)
//! - Inference Operations (2 commands)
//! - System Information (4 commands)
//! - File Operations (2 commands)
//! - Settings Management (2 commands)
//! - Activity Logging (3 commands)
//! - Notifications (7 commands)
//! - Batch Jobs (9 commands)
//! - Security/API Keys (8 commands)
//! - Model Repository (10 commands)

use tauri::{command, AppHandle, State, Window};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    AppState, ActivityLog, ActivityStats, BackendManager, ModelInfo, InferenceParams,
    GlobalMetrics, SecurityManager, ApiKey, SecurityEvent, SecurityMetrics,
    CreateApiKeyRequest, CreateApiKeyResponse, ExternalModelInfo, ModelSearchQuery,
    ModelSearchResponse, DownloadProgress,
};

// ============================================================================
// Type Definitions (from dashboard/src-tauri/src/main.rs)
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

// Additional types from dashboard - these would need full definitions
pub type AppSettings = serde_json::Value; // TODO: Define proper type
pub type Notification = serde_json::Value; // TODO: Define proper type
pub type BatchJob = serde_json::Value; // TODO: Define proper type

// This file is getting too large. Let me use Write tool instead to create the full file properly.
