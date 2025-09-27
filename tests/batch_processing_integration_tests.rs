use anyhow::Result;
use inferno::{
    backends::{BackendConfig, BackendType, InferenceParams},
    batch::{
        processor::{BatchProcessor, ProcessingResult, ProcessorConfig},
        queue::{
            BatchJob, JobExecutionContext, JobMetrics, JobPriority, JobQueue, JobQueueConfig,
            JobQueueManager, JobResult, JobStatus, QueueMetrics, QueueStatus, ResourceRequirements,
            RetryPolicy,
        },
        scheduler::{
            BatchScheduler, CronSchedule, IntervalSchedule, OneTimeSchedule, ScheduleEntry,
            ScheduleType, SchedulerConfig,
        },
        BatchConfig, BatchContext, BatchInput, BatchOutput,
    },
    cache::{CacheConfig, ModelCache},
    cron::{CronExpression, CronSchedule as CronScheduleParser},
    metrics::MetricsCollector,
    models::{ModelInfo, ModelManager},
    InfernoError,
};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tempfile::TempDir;
use tokio::{
    fs,
    sync::{Mutex, RwLock},
    time::{sleep, timeout},
};

/// Test utilities for batch processing integration tests
mod batch_test_utils {
    use super::*;

    pub fn create_test_queue_config() -> JobQueueConfig {
        JobQueueConfig {
            max_queues: 10,
            max_jobs_per_queue: 100,
            default_timeout_minutes: 30,
            max_retries: 3,
            cleanup_interval_seconds: 60,
            metrics_retention_hours: 24,
            persistent_storage: false,
            storage_path: None,
            enable_metrics: true,
            enable_deadletter_queue: true,
            max_concurrent_jobs: 5,
            job_timeout_seconds: 300,
            retry_delay_seconds: 30,
            max_retry_delay_seconds: 300,
            exponential_backoff: true,
        }
    }

    pub fn create_test_scheduler_config() -> SchedulerConfig {
        SchedulerConfig {
            enable_scheduler: true,
            max_concurrent_schedules: 50,
            schedule_check_interval_seconds: 10,
            missed_schedule_tolerance_seconds: 30,
            enable_schedule_persistence: false,
            persistence_path: None,
            timezone: "UTC".to_string(),
            enable_metrics: true,
            max_schedule_history: 100,
        }
    }

    pub fn create_test_processor_config() -> ProcessorConfig {
        ProcessorConfig {
            max_concurrent_jobs: 3,
            worker_pool_size: 2,
            enable_batching: true,
            batch_size: 5,
            batch_timeout_seconds: 30,
            enable_monitoring: true,
            heartbeat_interval_seconds: 10,
            failure_threshold: 3,
            recovery_interval_seconds: 60,
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_seconds: 30,
        }
    }

    pub fn create_test_batch_config() -> BatchConfig {
        BatchConfig {
            batch_size: 10,
            timeout_seconds: 300,
            parallel_processing: true,
            max_parallel_batches: 3,
            enable_streaming: false,
            output_format: "json".to_string(),
            compression_enabled: false,
            checkpointing_enabled: true,
            checkpoint_interval_seconds: 60,
        }
    }

    pub fn create_test_job(id: &str, model_name: &str, priority: JobPriority) -> BatchJob {
        BatchJob {
            id: id.to_string(),
            name: format!("Test Job {}", id),
            description: Some(format!("Test job for {}", model_name)),
            priority,
            inputs: vec![
                BatchInput {
                    id: format!("{}-input-1", id),
                    content: "What is the capital of France?".to_string(),
                    metadata: Some(HashMap::from([
                        ("type".to_string(), "question".to_string()),
                        ("category".to_string(), "geography".to_string()),
                    ])),
                },
                BatchInput {
                    id: format!("{}-input-2", id),
                    content: "Explain quantum computing in simple terms.".to_string(),
                    metadata: Some(HashMap::from([
                        ("type".to_string(), "explanation".to_string()),
                        ("category".to_string(), "science".to_string()),
                    ])),
                },
            ],
            inference_params: InferenceParams {
                max_tokens: 100,
                temperature: 0.7,
                top_p: 0.9,
                stream: false,
            },
            model_name: model_name.to_string(),
            batch_config: create_test_batch_config(),
            schedule: None,
            dependencies: vec![],
            resource_requirements: ResourceRequirements {
                min_memory_mb: 512,
                min_cpu_cores: 1,
                min_gpu_memory_mb: None,
                required_gpu: false,
                estimated_duration_seconds: Some(60),
                max_memory_mb: Some(2048),
                max_cpu_cores: Some(4),
            },
            timeout_minutes: Some(30),
            retry_count: 0,
            max_retries: 3,
            created_at: SystemTime::now(),
            scheduled_at: None,
            tags: HashMap::from([
                ("environment".to_string(), "test".to_string()),
                ("priority".to_string(), priority.to_string()),
            ]),
            metadata: HashMap::from([
                ("created_by".to_string(), "integration_test".to_string()),
                ("test_run_id".to_string(), uuid::Uuid::new_v4().to_string()),
            ]),
        }
    }

    pub fn create_mock_gguf_file(path: &PathBuf) -> Result<()> {
        let mut content = Vec::new();
        content.extend_from_slice(b"GGUF");
        content.extend_from_slice(&3u32.to_le_bytes());
        content.extend_from_slice(&0u64.to_le_bytes());
        content.extend_from_slice(&1u64.to_le_bytes());

        let key = "general.name";
        content.extend_from_slice(&(key.len() as u64).to_le_bytes());
        content.extend_from_slice(key.as_bytes());
        content.extend_from_slice(&8u32.to_le_bytes());
        let value = path.file_stem().unwrap().to_str().unwrap();
        content.extend_from_slice(&(value.len() as u64).to_le_bytes());
        content.extend_from_slice(value.as_bytes());

        content.resize(2048, 0);
        std::fs::write(path, content)?;
        Ok(())
    }

    pub async fn wait_for_job_status(
        manager: &JobQueueManager,
        queue_id: &str,
        job_id: &str,
        expected_status: JobStatus,
        timeout_duration: Duration,
    ) -> Result<bool> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout_duration {
            if let Some(job_info) = manager.get_job_status(queue_id, job_id).await? {
                if std::mem::discriminant(&job_info.status)
                    == std::mem::discriminant(&expected_status)
                {
                    return Ok(true);
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
        Ok(false)
    }

    pub async fn wait_for_queue_empty(
        manager: &JobQueueManager,
        queue_id: &str,
        timeout_duration: Duration,
    ) -> Result<bool> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout_duration {
            let jobs = manager.list_jobs(queue_id, Some(JobStatus::Queued)).await?;
            if jobs.is_empty() {
                return Ok(true);
            }
            sleep(Duration::from_millis(100)).await;
        }
        Ok(false)
    }
}

/// Test complete batch processing workflow
#[tokio::test]
async fn test_complete_batch_workflow() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).await?;

    // Create test model file
    let model_path = models_dir.join("test_model.gguf");
    batch_test_utils::create_mock_gguf_file(&model_path)?;

    // Initialize components
    let queue_config = batch_test_utils::create_test_queue_config();
    let queue_manager = Arc::new(JobQueueManager::new(queue_config));

    let scheduler_config = batch_test_utils::create_test_scheduler_config();
    let scheduler = Arc::new(BatchScheduler::new(scheduler_config, queue_manager.clone()).await?);

    let processor_config = batch_test_utils::create_test_processor_config();
    let model_manager = Arc::new(ModelManager::new(models_dir));
    let cache_config = CacheConfig::default();
    let backend_config = BackendConfig::default();
    let cache = Arc::new(
        ModelCache::new(
            cache_config,
            backend_config.clone(),
            model_manager.clone(),
            None,
        )
        .await?,
    );

    let processor =
        Arc::new(BatchProcessor::new(processor_config, queue_manager.clone(), cache, None).await?);

    // 1. Create a queue
    let queue_id = "integration-test-queue";
    queue_manager
        .create_queue(
            queue_id.to_string(),
            "Integration Test Queue".to_string(),
            "Queue for end-to-end integration testing".to_string(),
        )
        .await?;

    // 2. Submit multiple jobs with different priorities
    let jobs = vec![
        batch_test_utils::create_test_job("job-1", "test_model", JobPriority::High),
        batch_test_utils::create_test_job("job-2", "test_model", JobPriority::Normal),
        batch_test_utils::create_test_job("job-3", "test_model", JobPriority::Low),
    ];

    for job in jobs {
        queue_manager.submit_job(queue_id, job).await?;
    }

    // 3. Start processor
    let processor_handle = tokio::spawn({
        let processor = processor.clone();
        async move { processor.start_processing().await }
    });

    // 4. Wait for jobs to be processed
    let processing_timeout = Duration::from_secs(30);
    let all_processed =
        batch_test_utils::wait_for_queue_empty(&queue_manager, queue_id, processing_timeout)
            .await?;

    // Stop processor
    processor.stop_processing().await?;
    let _ = timeout(Duration::from_secs(5), processor_handle).await;

    assert!(all_processed, "All jobs should be processed within timeout");

    // 5. Verify job results
    let final_jobs = queue_manager.list_jobs(queue_id, None).await?;
    assert_eq!(final_jobs.len(), 3);

    for job in final_jobs {
        assert!(
            matches!(job.status, JobStatus::Completed | JobStatus::Failed),
            "Job {} should be completed or failed, got {:?}",
            job.id,
            job.status
        );
    }

    // 6. Check metrics
    let queue_metrics = queue_manager.get_queue_metrics(queue_id).await;
    assert!(queue_metrics.is_some());

    let metrics = queue_metrics.unwrap();
    assert!(metrics.total_jobs >= 3);
    assert!(metrics.completed_jobs + metrics.failed_jobs >= 3);

    Ok(())
}

/// Test batch scheduling functionality
#[tokio::test]
async fn test_batch_scheduling() -> Result<()> {
    let queue_config = batch_test_utils::create_test_queue_config();
    let queue_manager = Arc::new(JobQueueManager::new(queue_config));

    let scheduler_config = batch_test_utils::create_test_scheduler_config();
    let scheduler = Arc::new(BatchScheduler::new(scheduler_config, queue_manager.clone()).await?);

    // Create queue
    let queue_id = "scheduled-queue";
    queue_manager
        .create_queue(
            queue_id.to_string(),
            "Scheduled Queue".to_string(),
            "Queue for scheduled jobs".to_string(),
        )
        .await?;

    // 1. Test one-time schedule
    let one_time_schedule = ScheduleEntry {
        id: "one-time-1".to_string(),
        name: "One Time Test".to_string(),
        description: Some("Test one-time scheduling".to_string()),
        schedule_type: ScheduleType::OneTime(OneTimeSchedule {
            execute_at: SystemTime::now() + Duration::from_secs(2),
        }),
        queue_id: queue_id.to_string(),
        job_template: batch_test_utils::create_test_job(
            "scheduled-job-1",
            "test_model",
            JobPriority::Normal,
        ),
        enabled: true,
        created_at: SystemTime::now(),
        last_executed: None,
        next_execution: None,
        execution_count: 0,
        max_executions: Some(1),
        timezone: "UTC".to_string(),
    };

    scheduler.add_schedule(one_time_schedule).await?;

    // 2. Test interval schedule
    let interval_schedule = ScheduleEntry {
        id: "interval-1".to_string(),
        name: "Interval Test".to_string(),
        description: Some("Test interval scheduling".to_string()),
        schedule_type: ScheduleType::Interval(IntervalSchedule {
            interval_seconds: 5,
            start_time: Some(SystemTime::now()),
            end_time: Some(SystemTime::now() + Duration::from_secs(15)),
        }),
        queue_id: queue_id.to_string(),
        job_template: batch_test_utils::create_test_job(
            "scheduled-job-2",
            "test_model",
            JobPriority::Normal,
        ),
        enabled: true,
        created_at: SystemTime::now(),
        last_executed: None,
        next_execution: None,
        execution_count: 0,
        max_executions: Some(3),
        timezone: "UTC".to_string(),
    };

    scheduler.add_schedule(interval_schedule).await?;

    // 3. Test cron schedule
    let cron_expr = CronExpression::parse("*/10 * * * * *")?; // Every 10 seconds
    let cron_schedule = ScheduleEntry {
        id: "cron-1".to_string(),
        name: "Cron Test".to_string(),
        description: Some("Test cron scheduling".to_string()),
        schedule_type: ScheduleType::Cron(CronSchedule {
            expression: cron_expr,
            start_time: Some(SystemTime::now()),
            end_time: Some(SystemTime::now() + Duration::from_secs(25)),
        }),
        queue_id: queue_id.to_string(),
        job_template: batch_test_utils::create_test_job(
            "scheduled-job-3",
            "test_model",
            JobPriority::Normal,
        ),
        enabled: true,
        created_at: SystemTime::now(),
        last_executed: None,
        next_execution: None,
        execution_count: 0,
        max_executions: Some(2),
        timezone: "UTC".to_string(),
    };

    scheduler.add_schedule(cron_schedule).await?;

    // Start scheduler
    let scheduler_handle = tokio::spawn({
        let scheduler = scheduler.clone();
        async move { scheduler.start().await }
    });

    // Wait for schedules to execute
    sleep(Duration::from_secs(30)).await;

    // Stop scheduler
    scheduler.stop().await?;
    let _ = timeout(Duration::from_secs(5), scheduler_handle).await;

    // Verify scheduled jobs were created
    let jobs = queue_manager.list_jobs(queue_id, None).await?;
    assert!(
        jobs.len() >= 3,
        "Should have at least 3 scheduled jobs, got {}",
        jobs.len()
    );

    // Check schedule execution counts
    let schedules = scheduler.list_schedules().await?;
    for schedule in schedules {
        assert!(
            schedule.execution_count > 0,
            "Schedule {} should have executed",
            schedule.id
        );
    }

    Ok(())
}

/// Test retry mechanisms and error handling
#[tokio::test]
async fn test_retry_and_error_handling() -> Result<()> {
    let mut queue_config = batch_test_utils::create_test_queue_config();
    queue_config.max_retries = 2;
    queue_config.retry_delay_seconds = 1; // Fast retries for testing
    queue_config.exponential_backoff = true;

    let queue_manager = Arc::new(JobQueueManager::new(queue_config));

    // Create queue
    let queue_id = "retry-queue";
    queue_manager
        .create_queue(
            queue_id.to_string(),
            "Retry Test Queue".to_string(),
            "Queue for testing retry mechanisms".to_string(),
        )
        .await?;

    // Create a job that will fail (using non-existent model)
    let mut failing_job =
        batch_test_utils::create_test_job("failing-job", "nonexistent_model", JobPriority::Normal);
    failing_job.max_retries = 2;

    queue_manager.submit_job(queue_id, failing_job).await?;

    // Create processor (will fail to process the job)
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).await?;

    let processor_config = batch_test_utils::create_test_processor_config();
    let model_manager = Arc::new(ModelManager::new(models_dir));
    let cache_config = CacheConfig::default();
    let backend_config = BackendConfig::default();
    let cache = Arc::new(ModelCache::new(cache_config, backend_config, model_manager, None).await?);

    let processor =
        Arc::new(BatchProcessor::new(processor_config, queue_manager.clone(), cache, None).await?);

    // Start processor
    let processor_handle = tokio::spawn({
        let processor = processor.clone();
        async move { processor.start_processing().await }
    });

    // Wait for retries to complete
    sleep(Duration::from_secs(10)).await;

    // Stop processor
    processor.stop_processing().await?;
    let _ = timeout(Duration::from_secs(5), processor_handle).await;

    // Verify the job failed after retries
    let job_status = queue_manager
        .get_job_status(queue_id, "failing-job")
        .await?;
    assert!(job_status.is_some());

    let job_info = job_status.unwrap();
    assert!(matches!(job_info.status, JobStatus::Failed));
    assert_eq!(
        job_info.retry_count, 2,
        "Job should have been retried 2 times"
    );

    // Check dead letter queue if enabled
    if queue_manager.has_deadletter_queue(queue_id).await? {
        let deadletter_jobs = queue_manager.list_deadletter_jobs(queue_id).await?;
        assert_eq!(deadletter_jobs.len(), 1);
        assert_eq!(deadletter_jobs[0].id, "failing-job");
    }

    Ok(())
}

/// Test concurrent batch processing
#[tokio::test]
async fn test_concurrent_batch_processing() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).await?;

    // Create test model file
    let model_path = models_dir.join("concurrent_model.gguf");
    batch_test_utils::create_mock_gguf_file(&model_path)?;

    let mut queue_config = batch_test_utils::create_test_queue_config();
    queue_config.max_concurrent_jobs = 3;

    let queue_manager = Arc::new(JobQueueManager::new(queue_config));

    // Create queue
    let queue_id = "concurrent-queue";
    queue_manager
        .create_queue(
            queue_id.to_string(),
            "Concurrent Test Queue".to_string(),
            "Queue for testing concurrent processing".to_string(),
        )
        .await?;

    // Submit multiple jobs
    for i in 0..10 {
        let job = batch_test_utils::create_test_job(
            &format!("concurrent-job-{}", i),
            "concurrent_model",
            JobPriority::Normal,
        );
        queue_manager.submit_job(queue_id, job).await?;
    }

    // Create processor with multiple workers
    let mut processor_config = batch_test_utils::create_test_processor_config();
    processor_config.max_concurrent_jobs = 3;
    processor_config.worker_pool_size = 3;

    let model_manager = Arc::new(ModelManager::new(models_dir));
    let cache_config = CacheConfig::default();
    let backend_config = BackendConfig::default();
    let cache = Arc::new(ModelCache::new(cache_config, backend_config, model_manager, None).await?);

    let processor =
        Arc::new(BatchProcessor::new(processor_config, queue_manager.clone(), cache, None).await?);

    // Start processor
    let start_time = std::time::Instant::now();
    let processor_handle = tokio::spawn({
        let processor = processor.clone();
        async move { processor.start_processing().await }
    });

    // Wait for all jobs to be processed
    let all_processed =
        batch_test_utils::wait_for_queue_empty(&queue_manager, queue_id, Duration::from_secs(60))
            .await?;

    let processing_time = start_time.elapsed();

    // Stop processor
    processor.stop_processing().await?;
    let _ = timeout(Duration::from_secs(5), processor_handle).await;

    assert!(all_processed, "All jobs should be processed");

    // Verify concurrent processing improved performance
    // With 3 concurrent workers, it should be faster than sequential processing
    assert!(
        processing_time < Duration::from_secs(50),
        "Concurrent processing should complete faster"
    );

    // Check that jobs were processed concurrently
    let queue_metrics = queue_manager.get_queue_metrics(queue_id).await;
    assert!(queue_metrics.is_some());

    let metrics = queue_metrics.unwrap();
    assert_eq!(metrics.total_jobs, 10);
    assert!(metrics.avg_processing_time_seconds > 0.0);

    Ok(())
}

/// Test resource requirements and constraints
#[tokio::test]
async fn test_resource_constraints() -> Result<()> {
    let mut queue_config = batch_test_utils::create_test_queue_config();
    queue_config.max_concurrent_jobs = 2; // Limit to test resource constraints

    let queue_manager = Arc::new(JobQueueManager::new(queue_config));

    // Create queue
    let queue_id = "resource-queue";
    queue_manager
        .create_queue(
            queue_id.to_string(),
            "Resource Test Queue".to_string(),
            "Queue for testing resource constraints".to_string(),
        )
        .await?;

    // Create jobs with different resource requirements
    let mut high_memory_job =
        batch_test_utils::create_test_job("high-mem-job", "test_model", JobPriority::Normal);
    high_memory_job.resource_requirements.min_memory_mb = 4096; // High memory requirement

    let mut low_memory_job =
        batch_test_utils::create_test_job("low-mem-job", "test_model", JobPriority::Normal);
    low_memory_job.resource_requirements.min_memory_mb = 256; // Low memory requirement

    let mut gpu_job =
        batch_test_utils::create_test_job("gpu-job", "test_model", JobPriority::Normal);
    gpu_job.resource_requirements.required_gpu = true;
    gpu_job.resource_requirements.min_gpu_memory_mb = Some(2048);

    queue_manager.submit_job(queue_id, high_memory_job).await?;
    queue_manager.submit_job(queue_id, low_memory_job).await?;
    queue_manager.submit_job(queue_id, gpu_job).await?;

    // Get resource availability
    let resource_status = queue_manager.get_resource_status().await?;
    assert!(resource_status.total_memory_mb > 0);
    assert!(resource_status.available_memory_mb >= 0);

    // Test job filtering by resource requirements
    let eligible_jobs = queue_manager
        .get_eligible_jobs(queue_id, &resource_status)
        .await?;

    // Only jobs that can run with current resources should be eligible
    for job in eligible_jobs {
        assert!(job.resource_requirements.min_memory_mb <= resource_status.available_memory_mb);
        if job.resource_requirements.required_gpu {
            assert!(resource_status.gpu_available);
        }
    }

    Ok(())
}

/// Test batch processing metrics and monitoring
#[tokio::test]
async fn test_batch_metrics_monitoring() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).await?;

    let model_path = models_dir.join("metrics_model.gguf");
    batch_test_utils::create_mock_gguf_file(&model_path)?;

    let queue_config = batch_test_utils::create_test_queue_config();
    let queue_manager = Arc::new(JobQueueManager::new(queue_config));

    // Create queue
    let queue_id = "metrics-queue";
    queue_manager
        .create_queue(
            queue_id.to_string(),
            "Metrics Test Queue".to_string(),
            "Queue for testing metrics collection".to_string(),
        )
        .await?;

    // Submit various types of jobs
    let jobs = vec![
        batch_test_utils::create_test_job("metrics-job-1", "metrics_model", JobPriority::High),
        batch_test_utils::create_test_job("metrics-job-2", "metrics_model", JobPriority::Normal),
        batch_test_utils::create_test_job("metrics-job-3", "metrics_model", JobPriority::Low),
    ];

    for job in jobs {
        queue_manager.submit_job(queue_id, job).await?;
    }

    // Create processor with metrics enabled
    let processor_config = batch_test_utils::create_test_processor_config();
    let model_manager = Arc::new(ModelManager::new(models_dir));
    let cache_config = CacheConfig::default();
    let backend_config = BackendConfig::default();
    let cache = Arc::new(ModelCache::new(cache_config, backend_config, model_manager, None).await?);
    let metrics_collector = Arc::new(MetricsCollector::new());

    let processor = Arc::new(
        BatchProcessor::new(
            processor_config,
            queue_manager.clone(),
            cache,
            Some(metrics_collector.clone()),
        )
        .await?,
    );

    // Start processor
    let processor_handle = tokio::spawn({
        let processor = processor.clone();
        async move { processor.start_processing().await }
    });

    // Wait for some processing
    sleep(Duration::from_secs(10)).await;

    // Check queue metrics
    let queue_metrics = queue_manager.get_queue_metrics(queue_id).await;
    assert!(queue_metrics.is_some());

    let metrics = queue_metrics.unwrap();
    assert!(metrics.total_jobs >= 3);
    assert!(metrics.processing_jobs >= 0);
    assert!(metrics.avg_processing_time_seconds >= 0.0);

    // Check processor metrics
    let processor_metrics = processor.get_metrics().await?;
    assert!(processor_metrics.jobs_processed >= 0);
    assert!(processor_metrics.avg_processing_time_ms >= 0.0);
    assert!(processor_metrics.active_workers >= 0);

    // Check system metrics
    if let Some(system_metrics) = processor_metrics.system_metrics {
        assert!(system_metrics.cpu_usage_percent >= 0.0);
        assert!(system_metrics.memory_usage_mb >= 0);
    }

    // Stop processor
    processor.stop_processing().await?;
    let _ = timeout(Duration::from_secs(5), processor_handle).await;

    // Check historical metrics
    let historical_metrics = queue_manager
        .get_historical_metrics(
            queue_id,
            SystemTime::now() - Duration::from_secs(3600),
            SystemTime::now(),
        )
        .await?;

    assert!(!historical_metrics.is_empty());

    Ok(())
}

/// Test batch job dependencies and workflows
#[tokio::test]
async fn test_job_dependencies() -> Result<()> {
    let queue_config = batch_test_utils::create_test_queue_config();
    let queue_manager = Arc::new(JobQueueManager::new(queue_config));

    // Create queue
    let queue_id = "dependency-queue";
    queue_manager
        .create_queue(
            queue_id.to_string(),
            "Dependency Test Queue".to_string(),
            "Queue for testing job dependencies".to_string(),
        )
        .await?;

    // Create jobs with dependencies
    let job1 = batch_test_utils::create_test_job("dep-job-1", "test_model", JobPriority::Normal);

    let mut job2 =
        batch_test_utils::create_test_job("dep-job-2", "test_model", JobPriority::Normal);
    job2.dependencies = vec!["dep-job-1".to_string()];

    let mut job3 =
        batch_test_utils::create_test_job("dep-job-3", "test_model", JobPriority::Normal);
    job3.dependencies = vec!["dep-job-1".to_string(), "dep-job-2".to_string()];

    // Submit jobs in reverse order to test dependency resolution
    queue_manager.submit_job(queue_id, job3).await?;
    queue_manager.submit_job(queue_id, job2).await?;
    queue_manager.submit_job(queue_id, job1).await?;

    // Check dependency graph
    let dependency_graph = queue_manager.get_dependency_graph(queue_id).await?;
    assert!(!dependency_graph.nodes.is_empty());
    assert!(!dependency_graph.edges.is_empty());

    // Verify dependency validation
    let can_execute_job1 = queue_manager.can_execute_job(queue_id, "dep-job-1").await?;
    let can_execute_job2 = queue_manager.can_execute_job(queue_id, "dep-job-2").await?;
    let can_execute_job3 = queue_manager.can_execute_job(queue_id, "dep-job-3").await?;

    assert!(
        can_execute_job1,
        "Job 1 should be executable (no dependencies)"
    );
    assert!(
        !can_execute_job2,
        "Job 2 should not be executable (depends on job 1)"
    );
    assert!(
        !can_execute_job3,
        "Job 3 should not be executable (depends on job 1 and 2)"
    );

    // Simulate job 1 completion
    queue_manager
        .mark_job_completed(
            queue_id,
            "dep-job-1",
            JobResult {
                success: true,
                outputs: vec![],
                error_message: None,
                execution_time_seconds: 5.0,
                resources_used: ResourceRequirements::default(),
                metrics: HashMap::new(),
            },
        )
        .await?;

    // Now job 2 should be executable
    let can_execute_job2_after = queue_manager.can_execute_job(queue_id, "dep-job-2").await?;
    assert!(
        can_execute_job2_after,
        "Job 2 should be executable after job 1 completes"
    );

    Ok(())
}

/// Test batch queue persistence and recovery
#[tokio::test]
async fn test_queue_persistence_recovery() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let storage_path = temp_dir.path().join("queue_storage");

    let mut queue_config = batch_test_utils::create_test_queue_config();
    queue_config.persistent_storage = true;
    queue_config.storage_path = Some(storage_path.clone());

    // First queue manager instance
    {
        let queue_manager = Arc::new(JobQueueManager::new(queue_config.clone()));

        // Create queue and jobs
        let queue_id = "persistent-queue";
        queue_manager
            .create_queue(
                queue_id.to_string(),
                "Persistent Test Queue".to_string(),
                "Queue for testing persistence".to_string(),
            )
            .await?;

        for i in 0..3 {
            let job = batch_test_utils::create_test_job(
                &format!("persistent-job-{}", i),
                "test_model",
                JobPriority::Normal,
            );
            queue_manager.submit_job(queue_id, job).await?;
        }

        // Save state
        queue_manager.save_state().await?;

        // Verify storage file exists
        assert!(storage_path.exists());
    }

    // Second queue manager instance - should recover state
    {
        let queue_manager = Arc::new(JobQueueManager::new(queue_config));

        // Load state
        queue_manager.load_state().await?;

        // Verify queue and jobs were recovered
        let queues = queue_manager.list_all_queues().await?;
        assert_eq!(queues.len(), 1);
        assert_eq!(queues[0].id, "persistent-queue");

        let jobs = queue_manager.list_jobs("persistent-queue", None).await?;
        assert_eq!(jobs.len(), 3);

        for (i, job) in jobs.iter().enumerate() {
            assert_eq!(job.id, format!("persistent-job-{}", i));
            assert!(matches!(job.status, JobStatus::Queued));
        }
    }

    Ok(())
}
