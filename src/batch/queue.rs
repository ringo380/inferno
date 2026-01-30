#![allow(dead_code, unused_imports, unused_variables)]
use crate::{
    backends::InferenceParams,
    batch::{BatchConfig, BatchInput, BatchResult},
    metrics::MetricsCollector,
};
use anyhow::Result;
// use chrono::DateTime; // Reserved for future datetime operations
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
    time::{Duration, SystemTime},
};
use tokio::{
    fs,
    sync::{Mutex, RwLock, mpsc},
    time::sleep,
};
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct JobQueue {
    pub id: String,
    pub name: String,
    pub description: String,
    pub config: JobQueueConfig,
    pub jobs: Arc<RwLock<VecDeque<BatchJob>>>,
    pub active_jobs: Arc<RwLock<HashMap<String, ActiveJob>>>,
    pub completed_jobs: Arc<RwLock<Vec<CompletedJob>>>,
    pub failed_jobs: Arc<RwLock<Vec<FailedJob>>>,
    pub metrics: QueueMetrics,
    pub status: QueueStatus,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobQueueConfig {
    pub max_concurrent_jobs: usize,
    pub max_queue_size: usize,
    pub job_timeout_minutes: u64,
    pub retry_policy: RetryPolicy,
    pub priority_enabled: bool,
    pub scheduling_enabled: bool,
    pub resource_limits: ResourceLimits,
    pub notification_config: NotificationConfig,
}

impl Default for JobQueueConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: 4,
            max_queue_size: 1000,
            job_timeout_minutes: 60,
            retry_policy: RetryPolicy::default(),
            priority_enabled: true,
            scheduling_enabled: true,
            resource_limits: ResourceLimits::default(),
            notification_config: NotificationConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay_seconds: u64,
    pub max_delay_seconds: u64,
    pub backoff_multiplier: f64,
    pub retry_on_timeout: bool,
    pub retry_on_error: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_seconds: 1,
            max_delay_seconds: 300,
            backoff_multiplier: 2.0,
            retry_on_timeout: true,
            retry_on_error: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub backoff_enabled: bool,
    pub retry_on_timeout: bool,
    pub retry_on_error: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 1000,
            backoff_enabled: true,
            retry_on_timeout: true,
            retry_on_error: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<f64>,
    pub max_disk_space_mb: Option<u64>,
    pub max_network_bandwidth_mbps: Option<f64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: Some(4096),
            max_cpu_percent: Some(80.0),
            max_disk_space_mb: Some(10240),
            max_network_bandwidth_mbps: Some(100.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub on_job_complete: bool,
    pub on_job_failed: bool,
    pub on_queue_empty: bool,
    pub on_queue_full: bool,
    pub webhook_url: Option<String>,
    pub email_recipients: Vec<String>,
    pub slack_webhook: Option<String>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            on_job_complete: false,
            on_job_failed: true,
            on_queue_empty: false,
            on_queue_full: true,
            webhook_url: None,
            email_recipients: vec![],
            slack_webhook: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJob {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub priority: JobPriority,
    pub inputs: Vec<BatchInput>,
    pub inference_params: InferenceParams,
    pub model_name: String,
    pub batch_config: BatchConfig,
    pub schedule: Option<JobSchedule>,
    pub dependencies: Vec<String>,
    pub resource_requirements: ResourceRequirements,
    pub timeout_minutes: Option<u64>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub retry_config: RetryConfig,
    pub created_at: SystemTime,
    pub scheduled_at: Option<SystemTime>,
    pub tags: HashMap<String, String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobPriority {
    Low = 1,
    Normal = 5,
    High = 8,
    Critical = 10,
}

impl JobPriority {
    pub fn value(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Normal => 5,
            Self::High => 8,
            Self::Critical => 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSchedule {
    pub schedule_type: ScheduleType,
    pub start_time: Option<SystemTime>,
    pub end_time: Option<SystemTime>,
    pub timezone: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    Once(SystemTime),
    Interval {
        interval_minutes: u64,
        max_runs: Option<u32>,
    },
    Cron {
        expression: String,
        max_runs: Option<u32>,
    },
    Daily {
        time: String,      // HH:MM format
        weekdays: Vec<u8>, // 0-6, Monday=0
    },
    Weekly {
        day_of_week: u8, // 0-6, Monday=0
        time: String,    // HH:MM format
    },
    Monthly {
        day_of_month: u8, // 1-31
        time: String,     // HH:MM format
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: Option<f64>,
    pub memory_mb: Option<u64>,
    pub gpu_required: bool,
    pub gpu_memory_mb: Option<u64>,
    pub disk_space_mb: Option<u64>,
    pub network_bandwidth_mbps: Option<f64>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_cores: Some(1.0),
            memory_mb: Some(1024),
            gpu_required: false,
            gpu_memory_mb: None,
            disk_space_mb: Some(1024),
            network_bandwidth_mbps: Some(10.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveJob {
    pub job: BatchJob,
    pub started_at: SystemTime,
    pub worker_id: String,
    pub progress: JobProgress,
    pub current_attempt: u32,
    pub pid: Option<u32>,
    pub partial_results: Vec<BatchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedJob {
    pub job: BatchJob,
    pub results: Vec<BatchResult>,
    pub started_at: SystemTime,
    pub completed_at: SystemTime,
    pub worker_id: String,
    pub total_items: usize,
    pub successful_items: usize,
    pub failed_items: usize,
    pub total_duration_ms: u64,
    pub average_item_duration_ms: f64,
    pub throughput_items_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedJob {
    pub job: BatchJob,
    pub error: String,
    pub started_at: SystemTime,
    pub failed_at: SystemTime,
    pub worker_id: String,
    pub attempts_made: u32,
    pub partial_results: Vec<BatchResult>,
    pub last_error_details: ErrorDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub error_type: String,
    pub error_message: String,
    pub stack_trace: Option<String>,
    pub system_info: SystemInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub load_average: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    pub total_items: usize,
    pub completed_items: usize,
    pub failed_items: usize,
    pub current_item_index: usize,
    pub estimated_completion_time: Option<SystemTime>,
    pub current_rate_items_per_second: f64,
    pub average_item_duration_ms: f64,
    pub bytes_processed: u64,
    pub phase: JobPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobPhase {
    Queued,
    Starting,
    LoadingModel,
    Processing,
    Saving,
    Finishing,
    Retrying,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMetrics {
    pub total_jobs_submitted: u64,
    pub total_jobs_completed: u64,
    pub total_jobs_failed: u64,
    pub total_items_processed: u64,
    pub average_job_duration_ms: f64,
    pub average_queue_wait_time_ms: f64,
    pub peak_concurrent_jobs: usize,
    pub current_queue_size: usize,
    pub throughput_jobs_per_hour: f64,
    pub throughput_items_per_hour: f64,
    pub success_rate: f64,
    pub uptime_hours: f64,
}

impl Default for QueueMetrics {
    fn default() -> Self {
        Self {
            total_jobs_submitted: 0,
            total_jobs_completed: 0,
            total_jobs_failed: 0,
            total_items_processed: 0,
            average_job_duration_ms: 0.0,
            average_queue_wait_time_ms: 0.0,
            peak_concurrent_jobs: 0,
            current_queue_size: 0,
            throughput_jobs_per_hour: 0.0,
            throughput_items_per_hour: 0.0,
            success_rate: 0.0,
            uptime_hours: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueueStatus {
    Stopped,
    Starting,
    Running,
    Pausing,
    Paused,
    Draining, // No new jobs, finish existing ones
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: QueueStatus,
    pub created_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableJobQueue {
    pub id: String,
    pub name: String,
    pub description: String,
    pub config: JobQueueConfig,
    pub status: QueueStatus,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
    pub metrics: QueueMetrics,
    pub queued_jobs_count: usize,
    pub active_jobs_count: usize,
    pub completed_jobs_count: usize,
    pub failed_jobs_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueExportData {
    pub queue_info: QueueInfo,
    pub jobs: Vec<JobInfo>,
    pub metrics: QueueMetrics,
    pub config: JobQueueConfig,
}

#[derive(Debug)]
pub struct JobQueueManager {
    config: JobQueueConfig,
    queues: Arc<RwLock<HashMap<String, JobQueue>>>,
    schedulers: Arc<RwLock<HashMap<String, JobScheduler>>>,
    workers: Arc<RwLock<Vec<Worker>>>,
    metrics_collector: Option<Arc<MetricsCollector>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    resource_monitor: Arc<Mutex<ResourceMonitor>>,
    data_dir: PathBuf,
}

impl JobQueue {
    pub async fn to_serializable(&self) -> SerializableJobQueue {
        let queued_jobs_count = self.jobs.read().await.len();
        let active_jobs_count = self.active_jobs.read().await.len();
        let completed_jobs_count = self.completed_jobs.read().await.len();
        let failed_jobs_count = self.failed_jobs.read().await.len();

        SerializableJobQueue {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            config: self.config.clone(),
            status: self.status.clone(),
            created_at: self.created_at,
            last_activity: self.last_activity,
            metrics: self.metrics.clone(),
            queued_jobs_count,
            active_jobs_count,
            completed_jobs_count,
            failed_jobs_count,
        }
    }
}

impl JobQueueManager {
    pub fn new(config: JobQueueConfig) -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("inferno")
            .join("batch_queues");

        Self {
            config,
            queues: Arc::new(RwLock::new(HashMap::new())),
            schedulers: Arc::new(RwLock::new(HashMap::new())),
            workers: Arc::new(RwLock::new(Vec::new())),
            metrics_collector: None,
            shutdown_tx: None,
            resource_monitor: Arc::new(Mutex::new(ResourceMonitor::new())),
            data_dir,
        }
    }

    pub async fn create_queue(
        &self,
        queue_id: String,
        name: String,
        description: String,
    ) -> Result<()> {
        let mut queues = self.queues.write().await;

        if queues.contains_key(&queue_id) {
            return Err(anyhow::anyhow!(
                "Queue with ID '{}' already exists",
                queue_id
            ));
        }

        let queue = JobQueue {
            id: queue_id.clone(),
            name,
            description,
            config: self.config.clone(),
            jobs: Arc::new(RwLock::new(VecDeque::new())),
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
            completed_jobs: Arc::new(RwLock::new(Vec::new())),
            failed_jobs: Arc::new(RwLock::new(Vec::new())),
            metrics: QueueMetrics::default(),
            status: QueueStatus::Stopped,
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
        };

        queues.insert(queue_id.clone(), queue);

        // Release the write lock before saving
        drop(queues);

        // Save the queue to persistent storage
        if let Err(e) = self.save_queue(&queue_id).await {
            warn!(
                "Failed to save queue '{}' to persistent storage: {}",
                queue_id, e
            );
        }

        info!("Created job queue: {}", queue_id);
        Ok(())
    }

    pub async fn submit_job(&self, queue_id: &str, mut job: BatchJob) -> Result<String> {
        // Get a clone of the queue (Arc) and drop the read lock immediately to avoid deadlock
        let queue = {
            let queues = self.queues.read().await;
            queues
                .get(queue_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?
        };

        // Check queue capacity
        let queue_jobs = queue.jobs.read().await;
        if queue_jobs.len() >= self.config.max_queue_size {
            return Err(anyhow::anyhow!(
                "Queue '{}' is at maximum capacity",
                queue_id
            ));
        }
        drop(queue_jobs);

        // Validate job
        self.validate_job(&job).await?;

        // Assign unique ID if not provided
        if job.id.is_empty() {
            job.id = Uuid::new_v4().to_string();
        }

        let job_id = job.id.clone();
        let has_schedule = job.schedule.is_some();

        // Trigger scheduler if needed (before moving job)
        if has_schedule {
            self.schedule_job(queue_id, &job).await?;
        }

        // Add to queue
        let mut queue_jobs = queue.jobs.write().await;
        queue_jobs.push_back(job);

        info!("Submitted job '{}' to queue '{}'", job_id, queue_id);
        drop(queue_jobs); // Drop the write lock

        // Update queue metrics - need to acquire write lock on queues
        {
            let mut queues = self.queues.write().await;
            if let Some(queue) = queues.get_mut(queue_id) {
                queue.metrics.total_jobs_submitted += 1;
                // Note: This would need async access in real implementation
                // queue.metrics.current_queue_size = queue.jobs.read().await.len();

                // Calculate throughput (jobs per hour)
                let elapsed_hours = queue
                    .created_at
                    .elapsed()
                    .unwrap_or(Duration::from_secs(1))
                    .as_secs() as f64
                    / 3600.0;
                if elapsed_hours > 0.0 {
                    queue.metrics.throughput_jobs_per_hour =
                        queue.metrics.total_jobs_submitted as f64 / elapsed_hours;
                    queue.metrics.throughput_items_per_hour =
                        queue.metrics.total_items_processed as f64 / elapsed_hours;
                }

                // Update success rate
                let total_finished =
                    queue.metrics.total_jobs_completed + queue.metrics.total_jobs_failed;
                if total_finished > 0 {
                    queue.metrics.success_rate =
                        (queue.metrics.total_jobs_completed as f64 / total_finished as f64) * 100.0;
                }

                debug!(
                    "Updated metrics for queue '{}': {} total jobs submitted",
                    queue_id, queue.metrics.total_jobs_submitted
                );
            }
        }

        Ok(job_id)
    }

    async fn validate_job(&self, job: &BatchJob) -> Result<()> {
        // Validate inputs
        if job.inputs.is_empty() {
            return Err(anyhow::anyhow!("Job must have at least one input"));
        }

        // Validate model name
        if job.model_name.is_empty() {
            return Err(anyhow::anyhow!("Job must specify a model name"));
        }

        // Validate resource requirements
        if let Some(memory_mb) = job.resource_requirements.memory_mb {
            if memory_mb == 0 {
                return Err(anyhow::anyhow!("Memory requirement must be greater than 0"));
            }
        }

        // Validate schedule
        if let Some(schedule) = &job.schedule {
            self.validate_schedule(schedule).await?;
        }

        Ok(())
    }

    async fn validate_schedule(&self, schedule: &JobSchedule) -> Result<()> {
        match &schedule.schedule_type {
            ScheduleType::Once(time) => {
                if time < &SystemTime::now() {
                    return Err(anyhow::anyhow!("Scheduled time cannot be in the past"));
                }
            }
            ScheduleType::Interval {
                interval_minutes, ..
            } => {
                if *interval_minutes == 0 {
                    return Err(anyhow::anyhow!("Interval must be greater than 0"));
                }
            }
            ScheduleType::Cron { expression, .. } => {
                // Validate cron expression syntax
                if expression.is_empty() {
                    return Err(anyhow::anyhow!("Cron expression cannot be empty"));
                }

                // Basic cron expression validation (5 or 6 fields)
                let parts: Vec<&str> = expression.split_whitespace().collect();
                if parts.len() < 5 || parts.len() > 6 {
                    return Err(anyhow::anyhow!(
                        "Invalid cron expression format. Expected 5 or 6 fields, got {}",
                        parts.len()
                    ));
                }

                // Validate each field has valid characters
                for (i, part) in parts.iter().enumerate() {
                    if part.is_empty() {
                        return Err(anyhow::anyhow!("Cron field {} cannot be empty", i + 1));
                    }

                    // Check for valid cron characters
                    let valid_chars = "0123456789*,-/";
                    if !part.chars().all(|c| valid_chars.contains(c)) {
                        return Err(anyhow::anyhow!(
                            "Invalid character in cron field {}: '{}'",
                            i + 1,
                            part
                        ));
                    }
                }

                debug!("Validated cron expression: {}", expression);
            }
            ScheduleType::Daily { time, weekdays } => {
                // Validate time format (HH:MM)
                if let Err(e) = self.validate_time_format(time) {
                    return Err(anyhow::anyhow!(
                        "Invalid time format in daily schedule: {}",
                        e
                    ));
                }

                if weekdays.is_empty() {
                    return Err(anyhow::anyhow!("At least one weekday must be specified"));
                }
                for &day in weekdays {
                    if day > 6 {
                        return Err(anyhow::anyhow!("Invalid weekday: {}", day));
                    }
                }
            }
            ScheduleType::Weekly { day_of_week, .. } => {
                if *day_of_week > 6 {
                    return Err(anyhow::anyhow!("Invalid day of week: {}", day_of_week));
                }
            }
            ScheduleType::Monthly { day_of_month, .. } => {
                if *day_of_month == 0 || *day_of_month > 31 {
                    return Err(anyhow::anyhow!("Invalid day of month: {}", day_of_month));
                }
            }
        }
        Ok(())
    }

    fn validate_time_format(&self, time: &str) -> Result<()> {
        // Validate HH:MM format
        if time.len() != 5 {
            return Err(anyhow::anyhow!(
                "Time must be in HH:MM format, got: '{}'",
                time
            ));
        }

        let parts: Vec<&str> = time.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Time must contain exactly one colon, got: '{}'",
                time
            ));
        }

        // Validate hour (00-23)
        let hour: u8 = parts[0]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid hour in time '{}': not a number", time))?;
        if hour > 23 {
            return Err(anyhow::anyhow!(
                "Invalid hour in time '{}': {} (must be 0-23)",
                time,
                hour
            ));
        }

        // Validate minute (00-59)
        let minute: u8 = parts[1]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid minute in time '{}': not a number", time))?;
        if minute > 59 {
            return Err(anyhow::anyhow!(
                "Invalid minute in time '{}': {} (must be 0-59)",
                time,
                minute
            ));
        }

        debug!("Validated time format: {}", time);
        Ok(())
    }

    async fn schedule_job(&self, queue_id: &str, job: &BatchJob) -> Result<()> {
        let mut schedulers = self.schedulers.write().await;

        if !schedulers.contains_key(queue_id) {
            let scheduler = JobScheduler::new(queue_id.to_string());
            schedulers.insert(queue_id.to_string(), scheduler);
        }

        if let Some(scheduler) = schedulers.get_mut(queue_id) {
            scheduler.add_scheduled_job(job.clone()).await?;
        }

        Ok(())
    }

    pub async fn start_queue(&self, queue_id: &str) -> Result<()> {
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        // Start workers
        for i in 0..self.config.max_concurrent_jobs {
            let worker = Worker::new(
                format!("{}-worker-{}", queue_id, i),
                queue_id.to_string(),
                queue.clone(),
                self.metrics_collector.clone(),
            );

            let mut workers = self.workers.write().await;
            workers.push(worker);
        }

        info!(
            "Started queue '{}' with {} workers",
            queue_id, self.config.max_concurrent_jobs
        );
        Ok(())
    }

    pub async fn stop_queue(&self, queue_id: &str, drain: bool) -> Result<()> {
        let mut queues = self.queues.write().await;
        let queue = queues
            .get_mut(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        // Update queue status
        queue.status = if drain {
            QueueStatus::Draining
        } else {
            QueueStatus::Paused
        };

        if !drain {
            // If not draining, immediately stop processing new jobs
            // Cancel any pending jobs in the queue
            let mut jobs = queue.jobs.write().await;
            let cancelled_count = jobs.len();
            jobs.clear();

            info!(
                "Stopped queue '{}', cancelled {} pending jobs",
                queue_id, cancelled_count
            );
        } else {
            // If draining, let current jobs complete but don't accept new ones
            info!("Queue '{}' set to draining mode", queue_id);
        }

        // Note: Workers will check queue status and stop pulling new jobs
        Ok(())
    }

    pub async fn get_queue_status(&self, queue_id: &str) -> Option<QueueStatus> {
        let queues = self.queues.read().await;
        queues.get(queue_id).map(|q| q.status.clone())
    }

    pub async fn get_queue_metrics(&self, queue_id: &str) -> Option<QueueMetrics> {
        let queues = self.queues.read().await;
        queues.get(queue_id).map(|q| q.metrics.clone())
    }

    pub async fn list_jobs(
        &self,
        queue_id: &str,
        status: Option<JobStatus>,
    ) -> Result<Vec<JobInfo>> {
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        let mut job_infos = Vec::new();

        // Get queued jobs
        let queued_jobs = queue.jobs.read().await;
        for job in queued_jobs.iter() {
            if status.is_none() || matches!(status, Some(JobStatus::Queued)) {
                job_infos.push(JobInfo {
                    id: job.id.clone(),
                    name: job.name.clone(),
                    status: JobStatus::Queued,
                    priority: job.priority.clone(),
                    created_at: job.created_at,
                    started_at: None,
                    completed_at: None,
                    progress: None,
                });
            }
        }

        // Get active jobs
        let active_jobs = queue.active_jobs.read().await;
        for active_job in active_jobs.values() {
            if status.is_none() || matches!(status, Some(JobStatus::Running)) {
                job_infos.push(JobInfo {
                    id: active_job.job.id.clone(),
                    name: active_job.job.name.clone(),
                    status: JobStatus::Running,
                    priority: active_job.job.priority.clone(),
                    created_at: active_job.job.created_at,
                    started_at: Some(active_job.started_at),
                    completed_at: None,
                    progress: Some(active_job.progress.clone()),
                });
            }
        }

        // Get completed jobs
        let completed_jobs = queue.completed_jobs.read().await;
        for completed_job in completed_jobs.iter() {
            if status.is_none() || matches!(status, Some(JobStatus::Completed)) {
                job_infos.push(JobInfo {
                    id: completed_job.job.id.clone(),
                    name: completed_job.job.name.clone(),
                    status: JobStatus::Completed,
                    priority: completed_job.job.priority.clone(),
                    created_at: completed_job.job.created_at,
                    started_at: Some(completed_job.started_at),
                    completed_at: Some(completed_job.completed_at),
                    progress: None,
                });
            }
        }

        // Get failed jobs
        let failed_jobs = queue.failed_jobs.read().await;
        for failed_job in failed_jobs.iter() {
            if status.is_none() || matches!(status, Some(JobStatus::Failed)) {
                job_infos.push(JobInfo {
                    id: failed_job.job.id.clone(),
                    name: failed_job.job.name.clone(),
                    status: JobStatus::Failed,
                    priority: failed_job.job.priority.clone(),
                    created_at: failed_job.job.created_at,
                    started_at: Some(failed_job.started_at),
                    completed_at: Some(failed_job.failed_at),
                    progress: None,
                });
            }
        }

        // Sort by creation time (newest first)
        job_infos.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(job_infos)
    }

    pub async fn cancel_job(&self, queue_id: &str, job_id: &str) -> Result<()> {
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        // Check if job is in active jobs
        let mut active_jobs = queue.active_jobs.write().await;
        if let Some(active_job) = active_jobs.remove(job_id) {
            info!("Cancelled active job '{}' in queue '{}'", job_id, queue_id);

            // Move to failed jobs with cancellation status
            let failed_job = FailedJob {
                job: active_job.job,
                error: "Job cancelled by user".to_string(),
                started_at: active_job.started_at,
                failed_at: SystemTime::now(),
                worker_id: active_job.worker_id,
                attempts_made: active_job.current_attempt,
                partial_results: active_job.partial_results.clone(),
                last_error_details: ErrorDetails {
                    error_type: "Cancellation".to_string(),
                    error_message: "Job cancelled by user".to_string(),
                    stack_trace: None,
                    system_info: SystemInfo {
                        memory_usage_mb: 0.0,
                        cpu_usage_percent: 0.0,
                        disk_usage_percent: 0.0,
                        load_average: vec![],
                    },
                },
            };

            let mut failed_jobs = queue.failed_jobs.write().await;
            failed_jobs.push(failed_job);
            return Ok(());
        }

        // Check if job is in queue
        let mut queued_jobs = queue.jobs.write().await;
        if let Some(pos) = queued_jobs.iter().position(|job| job.id == job_id) {
            queued_jobs.remove(pos);
            info!("Cancelled queued job '{}' in queue '{}'", job_id, queue_id);
            return Ok(());
        }

        Err(anyhow::anyhow!(
            "Job '{}' not found in queue '{}'",
            job_id,
            queue_id
        ))
    }

    pub async fn list_all_queues(&self) -> Result<Vec<SerializableJobQueue>> {
        let queues = self.queues.read().await;
        let mut serializable_queues = Vec::new();

        for queue in queues.values() {
            serializable_queues.push(queue.to_serializable().await);
        }

        Ok(serializable_queues)
    }

    pub async fn get_queue_job_counts(&self, queue_id: &str) -> Result<(usize, usize, usize)> {
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        let queued = queue.jobs.read().await.len();
        let running = queue.active_jobs.read().await.len();
        let completed = queue.completed_jobs.read().await.len();

        Ok((queued, running, completed))
    }

    pub async fn get_job_status(&self, queue_id: &str, job_id: &str) -> Result<Option<JobInfo>> {
        let jobs = self.list_jobs(queue_id, None).await?;
        Ok(jobs.into_iter().find(|job| job.id == job_id))
    }

    pub async fn can_retry_job(&self, queue_id: &str, job_id: &str) -> Result<bool> {
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        let failed_jobs = queue.failed_jobs.read().await;
        if let Some(failed_job) = failed_jobs.iter().find(|job| job.job.id == job_id) {
            Ok(failed_job.attempts_made < failed_job.job.max_retries)
        } else {
            Ok(true) // If not found in failed jobs, it can be retried
        }
    }

    pub async fn retry_job(&self, queue_id: &str, job_id: &str, force: bool) -> Result<()> {
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        // Find job in failed jobs
        let mut failed_jobs = queue.failed_jobs.write().await;
        if let Some(pos) = failed_jobs.iter().position(|job| job.job.id == job_id) {
            let mut failed_job = failed_jobs.remove(pos);

            // Reset retry count if force is used
            if force {
                failed_job.job.retry_count = 0;
            } else {
                failed_job.job.retry_count += 1;
            }

            // Calculate exponential backoff delay
            let delay_seconds = self.config.retry_policy.initial_delay_seconds
                * (self
                    .config
                    .retry_policy
                    .backoff_multiplier
                    .powi(failed_job.job.retry_count as i32) as u64);
            let delay_seconds = delay_seconds.min(self.config.retry_policy.max_delay_seconds);

            // Schedule the job for retry
            failed_job.job.scheduled_at =
                Some(SystemTime::now() + Duration::from_secs(delay_seconds));

            // Add back to queue
            let mut queue_jobs = queue.jobs.write().await;
            queue_jobs.push_back(failed_job.job);

            info!(
                "Job '{}' scheduled for retry in {} seconds",
                job_id, delay_seconds
            );
            return Ok(());
        }

        Err(anyhow::anyhow!(
            "Failed job '{}' not found in queue '{}'",
            job_id,
            queue_id
        ))
    }

    pub async fn get_all_queue_metrics(&self) -> Result<HashMap<String, QueueMetrics>> {
        let queues = self.queues.read().await;
        let mut all_metrics = HashMap::new();

        for (queue_id, queue) in queues.iter() {
            all_metrics.insert(queue_id.clone(), queue.metrics.clone());
        }

        Ok(all_metrics)
    }

    pub async fn get_running_job_count(&self, queue_id: &str) -> Result<usize> {
        let queues = self.queues.read().await;
        if let Some(queue) = queues.get(queue_id) {
            let count = queue.active_jobs.read().await.len();
            Ok(count)
        } else {
            Err(anyhow::anyhow!("Queue '{}' not found", queue_id))
        }
    }

    pub async fn get_recent_jobs(&self, queue_id: &str, limit: usize) -> Result<Vec<JobInfo>> {
        let jobs = self.list_jobs(queue_id, None).await?;
        Ok(jobs.into_iter().take(limit).collect())
    }

    pub async fn pause_queue(&self, queue_id: &str) -> Result<()> {
        let mut queues = self.queues.write().await;
        if let Some(queue) = queues.get_mut(queue_id) {
            queue.status = QueueStatus::Paused;
            info!("Queue '{}' paused", queue_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Queue '{}' not found", queue_id))
        }
    }

    pub async fn resume_queue(&self, queue_id: &str) -> Result<()> {
        let mut queues = self.queues.write().await;
        if let Some(queue) = queues.get_mut(queue_id) {
            queue.status = QueueStatus::Running;
            info!("Queue '{}' resumed", queue_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Queue '{}' not found", queue_id))
        }
    }

    pub async fn clear_queue(&self, queue_id: &str, include_failed: bool) -> Result<usize> {
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        let mut cleared_count = 0;

        // Clear completed jobs
        let mut completed_jobs = queue.completed_jobs.write().await;
        cleared_count += completed_jobs.len();
        completed_jobs.clear();

        // Clear failed jobs if requested
        if include_failed {
            let mut failed_jobs = queue.failed_jobs.write().await;
            cleared_count += failed_jobs.len();
            failed_jobs.clear();
        }

        info!("Cleared {} jobs from queue '{}'", cleared_count, queue_id);
        Ok(cleared_count)
    }

    pub async fn export_jobs(&self, queue_id: &str) -> Result<Vec<JobInfo>> {
        self.list_jobs(queue_id, None).await
    }

    pub async fn export_queue_config(&self, queue_id: &str) -> Result<JobQueueConfig> {
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        Ok(queue.config.clone())
    }

    pub async fn export_all_data(&self, queue_id: &str) -> Result<QueueExportData> {
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        let jobs = self.list_jobs(queue_id, None).await?;
        let metrics = queue.metrics.clone();
        let config = queue.config.clone();

        Ok(QueueExportData {
            queue_info: QueueInfo {
                id: queue.id.clone(),
                name: queue.name.clone(),
                description: queue.description.clone(),
                status: queue.status.clone(),
                created_at: queue.created_at,
            },
            jobs,
            metrics,
            config,
        })
    }

    pub async fn get_queue_config(&self, queue_id: &str) -> Result<JobQueueConfig> {
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        Ok(queue.config.clone())
    }

    pub async fn update_queue_config(
        &self,
        queue_id: &str,
        updates: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let mut queues = self.queues.write().await;
        if let Some(queue) = queues.get_mut(queue_id) {
            for (key, value) in updates {
                match key.as_str() {
                    "max_concurrent_jobs" => {
                        if let Some(val) = value.as_u64() {
                            queue.config.max_concurrent_jobs = val as usize;
                        }
                    }
                    "max_queue_size" => {
                        if let Some(val) = value.as_u64() {
                            queue.config.max_queue_size = val as usize;
                        }
                    }
                    "job_timeout_minutes" => {
                        if let Some(val) = value.as_u64() {
                            queue.config.job_timeout_minutes = val;
                        }
                    }
                    "priority_enabled" => {
                        if let Some(val) = value.as_bool() {
                            queue.config.priority_enabled = val;
                        }
                    }
                    _ => {
                        warn!("Unknown configuration key: {}", key);
                    }
                }
            }
            info!("Updated configuration for queue '{}'", queue_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Queue '{}' not found", queue_id))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    PartiallyCompleted,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: String,
    pub job_name: String,
    pub status: JobStatus,
    pub results: Vec<BatchResult>,
    pub started_at: Option<Duration>,
    pub completed_at: Option<Duration>,
    pub total_items: usize,
    pub processed_items: usize,
    pub failed_items: usize,
    pub success_rate: f64,
    pub retry_count: u32,
    pub partial_results: Vec<String>,
}

impl JobQueueManager {
    /// Save a specific queue after changes
    pub async fn save_queue(&self, queue_id: &str) -> Result<()> {
        let queues = self.queues.read().await;
        if let Some(queue) = queues.get(queue_id) {
            let queue_file = self.data_dir.join(format!("{}.json", queue_id));
            let serializable_queue = queue.to_serializable().await;
            let json_data = serde_json::to_string_pretty(&serializable_queue)?;

            fs::write(&queue_file, json_data).await?;
            debug!("Saved queue '{}' to persistent storage", queue_id);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: String,
    pub name: String,
    pub status: JobStatus,
    pub priority: JobPriority,
    pub created_at: SystemTime,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub progress: Option<JobProgress>,
}

#[derive(Debug)]
struct JobScheduler {
    queue_id: String,
    scheduled_jobs: Vec<(BatchJob, SystemTime)>,
    running: bool,
}

impl JobScheduler {
    pub fn new(queue_id: String) -> Self {
        Self {
            queue_id,
            scheduled_jobs: Vec::new(),
            running: false,
        }
    }

    pub async fn add_scheduled_job(&mut self, job: BatchJob) -> Result<()> {
        if let Some(schedule) = &job.schedule {
            let next_run = self.calculate_next_run(schedule)?;
            self.scheduled_jobs.push((job, next_run));
        }
        Ok(())
    }

    fn calculate_next_run(&self, schedule: &JobSchedule) -> Result<SystemTime> {
        match &schedule.schedule_type {
            ScheduleType::Once(time) => Ok(*time),
            ScheduleType::Interval {
                interval_minutes, ..
            } => Ok(SystemTime::now() + Duration::from_secs(interval_minutes * 60)),
            ScheduleType::Daily { time, weekdays: _ } => {
                // Parse HH:MM format and calculate next daily run
                let parts: Vec<&str> = time.split(':').collect();
                if parts.len() == 2 {
                    let hour: u32 = parts[0].parse().unwrap_or(0);
                    let minute: u32 = parts[1].parse().unwrap_or(0);
                    let seconds_until = (hour * 3600 + minute * 60) as u64;
                    Ok(SystemTime::now() + Duration::from_secs(seconds_until))
                } else {
                    Ok(SystemTime::now() + Duration::from_secs(86400)) // Default to 24 hours
                }
            }
            ScheduleType::Weekly {
                day_of_week,
                time: _,
            } => {
                // Calculate next occurrence of the specified day
                // day_of_week is u8: 0 = Monday, 6 = Sunday
                let days_ahead = (*day_of_week as u64 + 1) % 7;
                Ok(SystemTime::now() + Duration::from_secs(days_ahead * 86400))
            }
            ScheduleType::Cron { expression, .. } => {
                // Basic cron support - for now just schedule hourly
                // Full cron parsing would require a cron library
                info!(
                    "Cron expression '{}' simplified to hourly schedule",
                    expression
                );
                Ok(SystemTime::now() + Duration::from_secs(3600))
            }
            ScheduleType::Monthly { .. } => {
                // For monthly, schedule 30 days from now as a simple approximation
                Ok(SystemTime::now() + Duration::from_secs(30 * 86400))
            }
        }
    }

    pub async fn start(&mut self) {
        self.running = true;
        let queue_id = self.queue_id.clone();

        // Start scheduler loop
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute

            loop {
                interval.tick().await;

                // Check for jobs ready to run
                let now = SystemTime::now();

                // Process scheduled jobs (in real implementation, would check scheduled_jobs)
                info!("Scheduler checking for ready jobs in queue '{}'", queue_id);
            }
        });
    }

    pub async fn stop(&mut self) {
        self.running = false;
    }
}

#[derive(Debug)]
struct Worker {
    id: String,
    queue_id: String,
    queue: JobQueue,
    metrics_collector: Option<Arc<MetricsCollector>>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl Worker {
    pub fn new(
        id: String,
        queue_id: String,
        queue: JobQueue,
        metrics_collector: Option<Arc<MetricsCollector>>,
    ) -> Self {
        Self {
            id,
            queue_id,
            queue,
            metrics_collector,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub async fn start(&self) {
        self.running.store(true, Ordering::SeqCst);

        while self.running.load(Ordering::SeqCst) {
            if let Some(job) = self.get_next_job().await {
                let _ = self.execute_job(job).await;
            } else {
                // No jobs available, sleep briefly
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    async fn get_next_job(&self) -> Option<BatchJob> {
        let mut queue_jobs = self.queue.jobs.write().await;
        queue_jobs.pop_front()
    }

    async fn execute_job(&self, job: BatchJob) -> Result<()> {
        info!(
            "Worker {} starting job {} with {} inputs",
            self.id,
            job.id,
            job.inputs.len()
        );

        let start_time = std::time::Instant::now();
        let mut results: Vec<BatchResult> = Vec::new();
        let mut failed_inputs = Vec::new();

        // 1. Load the specified model (mock implementation for now)
        info!("Loading model: {}", job.model_name);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // Simulate model loading

        // 2. Process the batch inputs
        for (index, input) in job.inputs.iter().enumerate() {
            info!(
                "Processing input {} of {} for job {}",
                index + 1,
                job.inputs.len(),
                job.id
            );

            match self
                .process_single_input(input, &job.inference_params)
                .await
            {
                Ok(result) => {
                    let batch_result = BatchResult {
                        id: input.id.clone(),
                        input: input.content.clone(),
                        output: Some(result),
                        error: None,
                        duration_ms: 100,
                        tokens_generated: Some(50),
                        timestamp: chrono::Utc::now(),
                        metadata: input.metadata.clone(),
                    };
                    results.push(batch_result);
                    info!(
                        "Successfully processed input {} for job {}",
                        index + 1,
                        job.id
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to process input {} for job {}: {}",
                        index + 1,
                        job.id,
                        e
                    );
                    failed_inputs.push((index, e.to_string()));

                    // Handle retries
                    if job.retry_config.max_retries > 0 {
                        info!("Attempting retry for input {} (job {})", index + 1, job.id);
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            job.retry_config.retry_delay_ms,
                        ))
                        .await;

                        // Retry the input
                        match self
                            .process_single_input(input, &job.inference_params)
                            .await
                        {
                            Ok(result) => {
                                let batch_result = BatchResult {
                                    id: input.id.clone(),
                                    input: input.content.clone(),
                                    output: Some(result),
                                    error: None,
                                    duration_ms: 100,
                                    tokens_generated: Some(50),
                                    timestamp: chrono::Utc::now(),
                                    metadata: input.metadata.clone(),
                                };
                                results.push(batch_result);
                                info!("Retry successful for input {} (job {})", index + 1, job.id);
                            }
                            Err(retry_err) => {
                                warn!(
                                    "Retry failed for input {} (job {}): {}",
                                    index + 1,
                                    job.id,
                                    retry_err
                                );
                                // Keep the failure recorded
                            }
                        }
                    }
                }
            }
        }

        // 3. Calculate metrics and progress
        let total_time = start_time.elapsed();
        let success_count = results.len();
        let failure_count = failed_inputs.len();
        let success_rate = (success_count as f64 / job.inputs.len() as f64) * 100.0;

        // 4. Create job result
        let job_result = JobResult {
            job_id: job.id.clone(),
            job_name: job.name.clone(),
            status: if failure_count == 0 {
                JobStatus::Completed
            } else if success_count > 0 {
                JobStatus::PartiallyCompleted
            } else {
                JobStatus::Failed
            },
            results,
            started_at: Some(start_time.elapsed().saturating_sub(total_time)),
            completed_at: Some(total_time),
            total_items: job.inputs.len(),
            processed_items: success_count,
            failed_items: failure_count,
            success_rate,
            retry_count: if job.retry_config.max_retries > 0 {
                1
            } else {
                0
            },
            partial_results: failed_inputs
                .iter()
                .map(|(idx, err)| format!("Input {}: {}", idx + 1, err))
                .collect(),
        };

        // 5. Log completion
        info!(
            "Worker {} completed job {} in {:.2}s: {}/{} inputs processed (success rate: {:.1}%)",
            self.id,
            job.id,
            total_time.as_secs_f64(),
            success_count,
            job.inputs.len(),
            success_rate
        );

        // Save results to persistent storage
        self.save_job_result(&job.id, &job_result).await?;
        debug!("Job result saved: {:?}", job_result);

        Ok(())
    }

    async fn save_job_result(&self, job_id: &str, result: &JobResult) -> Result<()> {
        // In a real implementation, this would save to:
        // - Database (PostgreSQL, MongoDB, etc.)
        // - Object storage (S3, GCS, etc.)
        // - Local filesystem with proper rotation

        let storage_path = std::path::PathBuf::from("job_results");
        tokio::fs::create_dir_all(&storage_path).await?;

        let filename = format!("{}/{}.json", storage_path.display(), job_id);
        let json = serde_json::to_string_pretty(result)?;
        tokio::fs::write(&filename, json).await?;

        info!("Saved job result to {}", filename);
        Ok(())
    }

    async fn process_single_input(
        &self,
        input: &BatchInput,
        params: &InferenceParams,
    ) -> Result<String> {
        // Simulate processing time based on input length
        let processing_time = std::cmp::min(input.content.len() * 2, 1000); // Max 1 second
        tokio::time::sleep(tokio::time::Duration::from_millis(processing_time as u64)).await;

        // Simulate occasional failures (5% failure rate)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        input.content.hash(&mut hasher);
        let hash_value = hasher.finish();
        if (hash_value % 100) < 5 {
            return Err(anyhow::anyhow!("Simulated processing failure"));
        }

        // Create a mock response that includes the input and parameters
        let response = format!(
            "Processed: '{}' (length: {}, max_tokens: {}, temp: {:.2})",
            input.content.chars().take(50).collect::<String>(),
            input.content.len(),
            params.max_tokens,
            params.temperature
        );

        Ok(response)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    // Removed save_queues, load_queues and load_queue_from_file methods
    // These belong to JobQueueManager, not Worker
}

// This initialize method should be part of JobQueueManager
// Moving it to the correct location

#[derive(Debug)]
struct ResourceMonitor {
    memory_usage: f64,
    cpu_usage: f64,
    disk_usage: f64,
    network_usage: f64,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            memory_usage: 0.0,
            cpu_usage: 0.0,
            disk_usage: 0.0,
            network_usage: 0.0,
        }
    }

    pub async fn update_metrics(&mut self) -> Result<()> {
        // Memory usage monitoring
        self.memory_usage = self.get_memory_usage().await?;

        // CPU usage monitoring (approximated)
        self.cpu_usage = self.get_cpu_usage().await?;

        // Disk usage monitoring
        self.disk_usage = self.get_disk_usage().await?;

        // Network usage monitoring (simplified)
        self.network_usage = self.get_network_usage().await?;

        debug!(
            "Updated resource metrics: CPU: {:.1}%, Memory: {:.1}%, Disk: {:.1}%, Network: {:.1}%",
            self.cpu_usage, self.memory_usage, self.disk_usage, self.network_usage
        );

        Ok(())
    }

    async fn get_memory_usage(&self) -> Result<f64> {
        #[cfg(target_os = "linux")]
        {
            match tokio::fs::read_to_string("/proc/meminfo").await {
                Ok(content) => {
                    let mut total_kb = 0u64;
                    let mut available_kb = 0u64;

                    for line in content.lines() {
                        if line.starts_with("MemTotal:") {
                            if let Some(value) = line.split_whitespace().nth(1) {
                                total_kb = value.parse().unwrap_or(0);
                            }
                        } else if line.starts_with("MemAvailable:") {
                            if let Some(value) = line.split_whitespace().nth(1) {
                                available_kb = value.parse().unwrap_or(0);
                            }
                        }
                    }

                    if total_kb > 0 {
                        let used_kb = total_kb.saturating_sub(available_kb);
                        Ok((used_kb as f64 / total_kb as f64) * 100.0)
                    } else {
                        Ok(0.0)
                    }
                }
                Err(_) => Ok(0.0),
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            // Simulate memory usage for non-Linux systems
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Ok(((timestamp % 100) as f64 / 100.0) * 80.0 + 10.0) // 10-90% range
        }
    }

    async fn get_cpu_usage(&self) -> Result<f64> {
        // Simplified CPU usage estimation based on system load
        #[cfg(target_os = "linux")]
        {
            match tokio::fs::read_to_string("/proc/loadavg").await {
                Ok(content) => {
                    if let Some(load_str) = content.split_whitespace().next() {
                        if let Ok(load) = load_str.parse::<f64>() {
                            // Convert load average to approximate CPU percentage
                            let cpu_cores = num_cpus::get() as f64;
                            return Ok((load / cpu_cores * 100.0).min(100.0));
                        }
                    }
                    Ok(0.0)
                }
                Err(_) => Ok(0.0),
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            // Simulate CPU usage for non-Linux systems
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Ok(((timestamp * 7) % 100) as f64 / 100.0 * 60.0 + 5.0) // 5-65% range
        }
    }

    async fn get_disk_usage(&self) -> Result<f64> {
        #[cfg(target_os = "linux")]
        {
            // Check disk usage of current directory
            match tokio::fs::metadata(".").await {
                Ok(_) => {
                    // For simplicity, return a mock value based on available space
                    // A real implementation would use statvfs or similar
                    Ok(25.0) // Mock 25% disk usage
                }
                Err(_) => Ok(0.0),
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            // Simulate disk usage
            Ok(30.0) // Mock 30% disk usage
        }
    }

    async fn get_network_usage(&self) -> Result<f64> {
        // Network usage is complex to measure in real-time
        // For now, return a low simulated value
        Ok(5.0) // Mock 5% network usage
    }

    pub fn check_resource_limits(&self, requirements: &ResourceRequirements) -> Result<()> {
        // Check memory requirements
        if let Some(required_memory_mb) = requirements.memory_mb {
            let available_memory_percent = 100.0 - self.memory_usage;
            let system_memory_gb = 8.0; // Assume 8GB system memory for calculation
            let available_memory_mb =
                (available_memory_percent / 100.0) * system_memory_gb * 1024.0;

            if required_memory_mb as f64 > available_memory_mb {
                return Err(anyhow::anyhow!(
                    "Insufficient memory: required {}MB, available {:.1}MB",
                    required_memory_mb,
                    available_memory_mb
                ));
            }
        }

        // Check CPU requirements
        if let Some(required_cpu_cores) = requirements.cpu_cores {
            let available_cpu_percent = 100.0 - self.cpu_usage;
            if available_cpu_percent < (required_cpu_cores * 20.0) {
                // Approximate 20% per core
                return Err(anyhow::anyhow!(
                    "Insufficient CPU: required {} cores, current usage {:.1}%",
                    required_cpu_cores,
                    self.cpu_usage
                ));
            }
        }

        // Check disk space requirements
        if let Some(required_disk_mb) = requirements.disk_space_mb {
            let available_disk_percent = 100.0 - self.disk_usage;
            let system_disk_gb = 100.0; // Assume 100GB system disk for calculation
            let available_disk_mb = (available_disk_percent / 100.0) * system_disk_gb * 1024.0;

            if required_disk_mb as f64 > available_disk_mb {
                return Err(anyhow::anyhow!(
                    "Insufficient disk space: required {}MB, available {:.1}MB",
                    required_disk_mb,
                    available_disk_mb
                ));
            }
        }

        // Check GPU requirements
        if requirements.gpu_required && self.cpu_usage > 90.0 {
            // Simplified check - if CPU is very high, assume GPU might also be stressed
            return Err(anyhow::anyhow!(
                "GPU resources may be constrained (high system load: {:.1}%)",
                self.cpu_usage
            ));
        }

        debug!(
            "Resource requirements check passed: Memory: {:.1}%, CPU: {:.1}%, Disk: {:.1}%",
            self.memory_usage, self.cpu_usage, self.disk_usage
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_queue() {
        let manager = JobQueueManager::new(JobQueueConfig::default());
        let result = manager
            .create_queue(
                "test-queue".to_string(),
                "Test Queue".to_string(),
                "A test queue".to_string(),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_submit_job() {
        let manager = JobQueueManager::new(JobQueueConfig::default());
        manager
            .create_queue(
                "test-queue".to_string(),
                "Test Queue".to_string(),
                "A test queue".to_string(),
            )
            .await
            .unwrap();

        let job = BatchJob {
            id: "test-job".to_string(),
            name: "Test Job".to_string(),
            description: Some("A test job".to_string()),
            priority: JobPriority::Normal,
            inputs: vec![BatchInput {
                id: "input-1".to_string(),
                content: "test input".to_string(),
                metadata: None,
            }],
            inference_params: InferenceParams::default(),
            model_name: "test-model".to_string(),
            batch_config: BatchConfig::default(),
            schedule: None,
            dependencies: vec![],
            resource_requirements: ResourceRequirements::default(),
            timeout_minutes: Some(30),
            retry_count: 0,
            max_retries: 3,
            retry_config: RetryConfig::default(),
            created_at: SystemTime::now(),
            scheduled_at: None,
            tags: HashMap::new(),
            metadata: HashMap::new(),
        };

        let result = manager.submit_job("test-queue", job).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_all_queues() {
        let manager = JobQueueManager::new(JobQueueConfig::default());

        // Initially no queues
        let queues = manager.list_all_queues().await.unwrap();
        assert_eq!(queues.len(), 0);

        // Create a queue
        manager
            .create_queue(
                "test-queue-1".to_string(),
                "Test Queue 1".to_string(),
                "First test queue".to_string(),
            )
            .await
            .unwrap();

        // Should have one queue
        let queues = manager.list_all_queues().await.unwrap();
        assert_eq!(queues.len(), 1);
        assert_eq!(queues[0].id, "test-queue-1");
    }

    #[tokio::test]
    async fn test_get_queue_job_counts() {
        let manager = JobQueueManager::new(JobQueueConfig::default());
        manager
            .create_queue(
                "count-test".to_string(),
                "Count Test Queue".to_string(),
                "Test queue for job counts".to_string(),
            )
            .await
            .unwrap();

        // Initially should have zero counts
        let (queued, running, completed) =
            manager.get_queue_job_counts("count-test").await.unwrap();
        assert_eq!(queued, 0);
        assert_eq!(running, 0);
        assert_eq!(completed, 0);
    }

    #[tokio::test]
    async fn test_queue_metrics() {
        let manager = JobQueueManager::new(JobQueueConfig::default());
        manager
            .create_queue(
                "metrics-test".to_string(),
                "Metrics Test Queue".to_string(),
                "Test queue for metrics".to_string(),
            )
            .await
            .unwrap();

        // Test getting metrics
        let metrics = manager.get_queue_metrics("metrics-test").await;
        assert!(metrics.is_some());

        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_jobs_submitted, 0);
        assert_eq!(metrics.total_jobs_completed, 0);
        assert_eq!(metrics.current_queue_size, 0);

        // Test getting all metrics
        let all_metrics = manager.get_all_queue_metrics().await.unwrap();
        assert!(all_metrics.contains_key("metrics-test"));
    }

    #[tokio::test]
    async fn test_export_functionality() {
        let manager = JobQueueManager::new(JobQueueConfig::default());
        manager
            .create_queue(
                "export-test".to_string(),
                "Export Test Queue".to_string(),
                "Test queue for export".to_string(),
            )
            .await
            .unwrap();

        // Test job export
        let jobs = manager.export_jobs("export-test").await.unwrap();
        assert_eq!(jobs.len(), 0);

        // Test config export
        let config = manager.export_queue_config("export-test").await.unwrap();
        assert_eq!(config.max_concurrent_jobs, 4); // Default value

        // Test full export
        let all_data = manager.export_all_data("export-test").await.unwrap();
        assert_eq!(all_data.queue_info.id, "export-test");
        assert_eq!(all_data.jobs.len(), 0);
    }
}
