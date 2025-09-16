use crate::{
    backends::{Backend, InferenceParams},
    config::Config,
    metrics::{InferenceEvent, MetricsCollector},
    batch::{BatchInput, BatchResult, BatchConfig},
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::{Arc, atomic::{AtomicUsize, Ordering}},
    time::{Duration, SystemTime},
};
use tokio::{
    sync::{RwLock, Mutex, mpsc, Semaphore},
    time::{interval, sleep, timeout},
};
use tracing::{debug, error, info, warn};
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
        time: String, // HH:MM format
        weekdays: Vec<u8>, // 0-6, Monday=0
    },
    Weekly {
        day_of_week: u8, // 0-6, Monday=0
        time: String, // HH:MM format
    },
    Monthly {
        day_of_month: u8, // 1-31
        time: String, // HH:MM format
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: Option<f64>,
    pub memory_mb: Option<u64>,
    pub gpu_memory_mb: Option<u64>,
    pub disk_space_mb: Option<u64>,
    pub network_bandwidth_mbps: Option<f64>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_cores: Some(1.0),
            memory_mb: Some(1024),
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
}

impl JobQueueManager {
    pub fn new(config: JobQueueConfig) -> Self {
        Self {
            config,
            queues: Arc::new(RwLock::new(HashMap::new())),
            schedulers: Arc::new(RwLock::new(HashMap::new())),
            workers: Arc::new(RwLock::new(Vec::new())),
            metrics_collector: None,
            shutdown_tx: None,
            resource_monitor: Arc::new(Mutex::new(ResourceMonitor::new())),
        }
    }

    pub async fn create_queue(&self, queue_id: String, name: String, description: String) -> Result<()> {
        let mut queues = self.queues.write().await;

        if queues.contains_key(&queue_id) {
            return Err(anyhow::anyhow!("Queue with ID '{}' already exists", queue_id));
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
        info!("Created job queue: {}", queue_id);
        Ok(())
    }

    pub async fn submit_job(&self, queue_id: &str, mut job: BatchJob) -> Result<String> {
        let queues = self.queues.read().await;
        let queue = queues.get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        // Check queue capacity
        let queue_jobs = queue.jobs.read().await;
        if queue_jobs.len() >= self.config.max_queue_size {
            return Err(anyhow::anyhow!("Queue '{}' is at maximum capacity", queue_id));
        }
        drop(queue_jobs);

        // Validate job
        self.validate_job(&job).await?;

        // Assign unique ID if not provided
        if job.id.is_empty() {
            job.id = Uuid::new_v4().to_string();
        }

        // Add to queue
        let mut queue_jobs = queue.jobs.write().await;
        queue_jobs.push_back(job.clone());

        info!("Submitted job '{}' to queue '{}'", job.id, queue_id);

        // Update metrics
        // TODO: Update queue metrics

        // Trigger scheduler if needed
        if job.schedule.is_some() {
            self.schedule_job(queue_id, &job).await?;
        }

        Ok(job.id)
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
            ScheduleType::Interval { interval_minutes, .. } => {
                if *interval_minutes == 0 {
                    return Err(anyhow::anyhow!("Interval must be greater than 0"));
                }
            }
            ScheduleType::Cron { expression, .. } => {
                // TODO: Validate cron expression syntax
                if expression.is_empty() {
                    return Err(anyhow::anyhow!("Cron expression cannot be empty"));
                }
            }
            ScheduleType::Daily { time, weekdays } => {
                // TODO: Validate time format (HH:MM)
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
        let queue = queues.get(queue_id)
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

        info!("Started queue '{}' with {} workers", queue_id, self.config.max_concurrent_jobs);
        Ok(())
    }

    pub async fn stop_queue(&self, queue_id: &str, drain: bool) -> Result<()> {
        // TODO: Implement queue stopping logic
        info!("Stopping queue '{}' (drain: {})", queue_id, drain);
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

    pub async fn list_jobs(&self, queue_id: &str, status: Option<JobStatus>) -> Result<Vec<JobInfo>> {
        let queues = self.queues.read().await;
        let queue = queues.get(queue_id)
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
                    started_at: None, // TODO: Track start time for failed jobs
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
        let queue = queues.get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        // Check if job is in active jobs
        let mut active_jobs = queue.active_jobs.write().await;
        if let Some(active_job) = active_jobs.remove(job_id) {
            info!("Cancelled active job '{}' in queue '{}'", job_id, queue_id);

            // Move to failed jobs with cancellation status
            let failed_job = FailedJob {
                job: active_job.job,
                error: "Job cancelled by user".to_string(),
                failed_at: SystemTime::now(),
                worker_id: active_job.worker_id,
                attempts_made: active_job.current_attempt,
                partial_results: vec![], // TODO: Collect partial results
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

        Err(anyhow::anyhow!("Job '{}' not found in queue '{}'", job_id, queue_id))
    }

    pub async fn list_all_queues(&self) -> Result<Vec<JobQueue>> {
        let queues = self.queues.read().await;
        Ok(queues.values().cloned().collect())
    }

    pub async fn get_queue_job_counts(&self, queue_id: &str) -> Result<(usize, usize, usize)> {
        let queues = self.queues.read().await;
        let queue = queues.get(queue_id)
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
        let queue = queues.get(queue_id)
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
        let queue = queues.get(queue_id)
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
                * (self.config.retry_policy.backoff_multiplier.powi(failed_job.job.retry_count as i32) as u64);
            let delay_seconds = delay_seconds.min(self.config.retry_policy.max_delay_seconds);

            // Schedule the job for retry
            failed_job.job.scheduled_at = Some(SystemTime::now() + Duration::from_secs(delay_seconds));

            // Add back to queue
            let mut queue_jobs = queue.jobs.write().await;
            queue_jobs.push_back(failed_job.job);

            info!("Job '{}' scheduled for retry in {} seconds", job_id, delay_seconds);
            return Ok(());
        }

        Err(anyhow::anyhow!("Failed job '{}' not found in queue '{}'", job_id, queue_id))
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
        let queue = queues.get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        Ok(queue.active_jobs.read().await.len())
    }

    pub async fn get_recent_jobs(&self, queue_id: &str, limit: usize) -> Result<Vec<JobInfo>> {
        let jobs = self.list_jobs(queue_id, None).await?;
        Ok(jobs.into_iter().take(limit).collect())
    }

    pub async fn pause_queue(&self, queue_id: &str) -> Result<()> {
        let queues = self.queues.write().await;
        if let Some(mut queue) = queues.get(queue_id).cloned() {
            queue.status = QueueStatus::Paused;
            info!("Queue '{}' paused", queue_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Queue '{}' not found", queue_id))
        }
    }

    pub async fn resume_queue(&self, queue_id: &str) -> Result<()> {
        let queues = self.queues.write().await;
        if let Some(mut queue) = queues.get(queue_id).cloned() {
            queue.status = QueueStatus::Running;
            info!("Queue '{}' resumed", queue_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Queue '{}' not found", queue_id))
        }
    }

    pub async fn clear_queue(&self, queue_id: &str, include_failed: bool) -> Result<usize> {
        let queues = self.queues.read().await;
        let queue = queues.get(queue_id)
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
        let queue = queues.get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        Ok(queue.config.clone())
    }

    pub async fn export_all_data(&self, queue_id: &str) -> Result<QueueExportData> {
        let queues = self.queues.read().await;
        let queue = queues.get(queue_id)
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
        let queue = queues.get(queue_id)
            .ok_or_else(|| anyhow::anyhow!("Queue '{}' not found", queue_id))?;

        Ok(queue.config.clone())
    }

    pub async fn update_queue_config(&self, queue_id: &str, updates: HashMap<String, serde_json::Value>) -> Result<()> {
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
    Failed,
    Cancelled,
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
            ScheduleType::Interval { interval_minutes, .. } => {
                Ok(SystemTime::now() + Duration::from_secs(interval_minutes * 60))
            }
            _ => {
                // TODO: Implement other schedule types
                Ok(SystemTime::now() + Duration::from_secs(3600)) // Default to 1 hour
            }
        }
    }

    pub async fn start(&mut self) {
        self.running = true;
        // TODO: Start scheduler loop
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
                self.execute_job(job).await;
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
        info!("Worker {} starting job {}", self.id, job.id);

        // TODO: Implement job execution logic
        // This would involve:
        // 1. Loading the specified model
        // 2. Processing the batch inputs
        // 3. Recording progress and metrics
        // 4. Handling retries and failures
        // 5. Saving results

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

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
        // TODO: Implement actual system resource monitoring
        Ok(())
    }

    pub fn check_resource_limits(&self, _requirements: &ResourceRequirements) -> Result<()> {
        // TODO: Implement resource limit checking
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_queue() {
        let manager = JobQueueManager::new(JobQueueConfig::default());
        let result = manager.create_queue(
            "test-queue".to_string(),
            "Test Queue".to_string(),
            "A test queue".to_string()
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_submit_job() {
        let manager = JobQueueManager::new(JobQueueConfig::default());
        manager.create_queue(
            "test-queue".to_string(),
            "Test Queue".to_string(),
            "A test queue".to_string()
        ).await.unwrap();

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
        manager.create_queue(
            "test-queue-1".to_string(),
            "Test Queue 1".to_string(),
            "First test queue".to_string()
        ).await.unwrap();

        // Should have one queue
        let queues = manager.list_all_queues().await.unwrap();
        assert_eq!(queues.len(), 1);
        assert_eq!(queues[0].id, "test-queue-1");
    }

    #[tokio::test]
    async fn test_get_queue_job_counts() {
        let manager = JobQueueManager::new(JobQueueConfig::default());
        manager.create_queue(
            "count-test".to_string(),
            "Count Test Queue".to_string(),
            "Test queue for job counts".to_string()
        ).await.unwrap();

        // Initially should have zero counts
        let (queued, running, completed) = manager.get_queue_job_counts("count-test").await.unwrap();
        assert_eq!(queued, 0);
        assert_eq!(running, 0);
        assert_eq!(completed, 0);
    }

    #[tokio::test]
    async fn test_queue_metrics() {
        let manager = JobQueueManager::new(JobQueueConfig::default());
        manager.create_queue(
            "metrics-test".to_string(),
            "Metrics Test Queue".to_string(),
            "Test queue for metrics".to_string()
        ).await.unwrap();

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
        manager.create_queue(
            "export-test".to_string(),
            "Export Test Queue".to_string(),
            "Test queue for export".to_string()
        ).await.unwrap();

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