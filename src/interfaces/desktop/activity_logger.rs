use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActivityLog {
    pub id: String,
    pub activity_type: ActivityType,
    pub title: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub status: ActivityStatus,
    pub user: String,
    pub metadata: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ActivityType {
    Inference,
    ModelLoad,
    ModelUnload,
    ModelValidation,
    ModelUpload,
    Configuration,
    System,
    Error,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ActivityStatus {
    Success,
    Warning,
    Error,
    InProgress,
    Cancelled,
}

#[derive(Clone)]
pub struct ActivityLogger {
    activities: Arc<Mutex<VecDeque<ActivityLog>>>,
    max_activities: usize,
}

impl ActivityLogger {
    pub fn new(max_activities: usize) -> Self {
        Self {
            activities: Arc::new(Mutex::new(VecDeque::new())),
            max_activities,
        }
    }

    pub fn log(&self, activity: ActivityLog) {
        let mut activities = self.activities.lock().unwrap();

        // Add new activity
        activities.push_front(activity);

        // Keep only the most recent activities
        while activities.len() > self.max_activities {
            activities.pop_back();
        }
    }

    pub fn log_simple(
        &self,
        activity_type: ActivityType,
        title: String,
        description: String,
        status: ActivityStatus,
    ) {
        self.log(ActivityLog {
            id: Uuid::new_v4().to_string(),
            activity_type,
            title,
            description,
            timestamp: Utc::now(),
            status,
            user: "system".to_string(),
            metadata: serde_json::json!({}),
        });
    }

    pub fn log_model_operation(
        &self,
        activity_type: ActivityType,
        model_name: &str,
        status: ActivityStatus,
        details: Option<&str>,
    ) {
        let (title, description) = match (&activity_type, &status) {
            (ActivityType::ModelLoad, ActivityStatus::Success) => (
                "Model loaded successfully".to_string(),
                format!("Model '{}' loaded and ready for inference", model_name),
            ),
            (ActivityType::ModelLoad, ActivityStatus::Error) => (
                "Model load failed".to_string(),
                format!(
                    "Failed to load model '{}': {}",
                    model_name,
                    details.unwrap_or("Unknown error")
                ),
            ),
            (ActivityType::ModelLoad, ActivityStatus::InProgress) => (
                "Loading model".to_string(),
                format!("Loading model '{}'...", model_name),
            ),
            (ActivityType::ModelUnload, ActivityStatus::Success) => (
                "Model unloaded".to_string(),
                format!("Model '{}' unloaded from memory", model_name),
            ),
            (ActivityType::ModelValidation, ActivityStatus::Success) => (
                "Model validated".to_string(),
                format!("Model '{}' passed validation checks", model_name),
            ),
            (ActivityType::ModelValidation, ActivityStatus::Error) => (
                "Model validation failed".to_string(),
                format!(
                    "Model '{}' failed validation: {}",
                    model_name,
                    details.unwrap_or("Unknown error")
                ),
            ),
            _ => (
                format!("{:?} operation", activity_type),
                format!("Model '{}' operation completed", model_name),
            ),
        };

        self.log(ActivityLog {
            id: Uuid::new_v4().to_string(),
            activity_type,
            title,
            description,
            timestamp: Utc::now(),
            status,
            user: "system".to_string(),
            metadata: serde_json::json!({
                "model_name": model_name,
                "details": details
            }),
        });
    }

    pub fn log_inference(
        &self,
        model_name: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        duration_ms: u64,
        status: ActivityStatus,
    ) {
        let title = match status {
            ActivityStatus::Success => "Inference completed".to_string(),
            ActivityStatus::Error => "Inference failed".to_string(),
            _ => "Inference in progress".to_string(),
        };

        let description = match status {
            ActivityStatus::Success => format!(
                "{} processed {} prompt tokens, generated {} completion tokens in {:.2}s",
                model_name,
                prompt_tokens,
                completion_tokens,
                duration_ms as f64 / 1000.0
            ),
            ActivityStatus::Error => format!("Inference failed with model '{}'", model_name),
            _ => format!("Running inference with model '{}'", model_name),
        };

        self.log(ActivityLog {
            id: Uuid::new_v4().to_string(),
            activity_type: ActivityType::Inference,
            title,
            description,
            timestamp: Utc::now(),
            status,
            user: "system".to_string(),
            metadata: serde_json::json!({
                "model_name": model_name,
                "prompt_tokens": prompt_tokens,
                "completion_tokens": completion_tokens,
                "duration_ms": duration_ms
            }),
        });
    }

    pub fn get_recent_activities(&self, limit: usize) -> Vec<ActivityLog> {
        let activities = self.activities.lock().unwrap();
        activities.iter().take(limit).cloned().collect()
    }

    pub fn get_activities_by_type(
        &self,
        activity_type: ActivityType,
        limit: usize,
    ) -> Vec<ActivityLog> {
        let activities = self.activities.lock().unwrap();
        activities
            .iter()
            .filter(|activity| {
                std::mem::discriminant(&activity.activity_type)
                    == std::mem::discriminant(&activity_type)
            })
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        let mut activities = self.activities.lock().unwrap();
        activities.clear();
    }

    pub fn get_stats(&self) -> ActivityStats {
        let activities = self.activities.lock().unwrap();

        let mut stats = ActivityStats::default();

        for activity in activities.iter() {
            match activity.status {
                ActivityStatus::Success => stats.success_count += 1,
                ActivityStatus::Error => stats.error_count += 1,
                ActivityStatus::Warning => stats.warning_count += 1,
                ActivityStatus::InProgress => stats.in_progress_count += 1,
                ActivityStatus::Cancelled => stats.cancelled_count += 1,
            }

            match activity.activity_type {
                ActivityType::Inference => stats.inference_count += 1,
                ActivityType::ModelLoad => stats.model_load_count += 1,
                ActivityType::ModelUnload => stats.model_unload_count += 1,
                _ => {}
            }
        }

        stats.total_count = activities.len();
        stats
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ActivityStats {
    pub total_count: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub in_progress_count: usize,
    pub cancelled_count: usize,
    pub inference_count: usize,
    pub model_load_count: usize,
    pub model_unload_count: usize,
}

impl Default for ActivityLogger {
    fn default() -> Self {
        Self::new(100) // Keep last 100 activities by default
    }
}
