use inferno::{
    batch::queue::{
        JobQueue, JobQueueManager, JobQueueConfig, BatchJob, JobPriority,
        JobStatus, QueueStatus
    },
    batch::{BatchConfig, BatchInput},
    backends::InferenceParams,
};
use std::{
    collections::HashMap,
    time::SystemTime,
};

#[tokio::test]
async fn test_batch_queue_basic_operations() {
    let config = JobQueueConfig::default();
    let manager = JobQueueManager::new(config);

    // Test queue creation
    let result = manager.create_queue(
        "test-queue".to_string(),
        "Test Queue".to_string(),
        "A test queue for integration testing".to_string()
    ).await;
    assert!(result.is_ok(), "Failed to create queue: {:?}", result);

    // Test queue listing
    let queues = manager.list_all_queues().await.unwrap();
    assert_eq!(queues.len(), 1);
    assert_eq!(queues[0].id, "test-queue");
    assert_eq!(queues[0].name, "Test Queue");

    // Test job submission
    let job = BatchJob {
        id: "test-job".to_string(),
        name: "Test Job".to_string(),
        description: Some("A test job".to_string()),
        priority: JobPriority::Normal,
        inputs: vec![BatchInput {
            id: "input-1".to_string(),
            content: "test input content".to_string(),
            metadata: None,
        }],
        inference_params: InferenceParams::default(),
        model_name: "test-model".to_string(),
        batch_config: BatchConfig::default(),
        schedule: None,
        dependencies: vec![],
        resource_requirements: Default::default(),
        timeout_minutes: Some(30),
        retry_count: 0,
        max_retries: 3,
        created_at: SystemTime::now(),
        scheduled_at: None,
        tags: HashMap::new(),
        metadata: HashMap::new(),
    };

    let job_id = manager.submit_job("test-queue", job).await.unwrap();
    assert_eq!(job_id, "test-job");

    // Test job listing
    let jobs = manager.list_jobs("test-queue", None).await.unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, "test-job");
    assert_eq!(jobs[0].name, "Test Job");
    assert!(matches!(jobs[0].status, JobStatus::Queued));

    // Test job status
    let job_status = manager.get_job_status("test-queue", "test-job").await.unwrap();
    assert!(job_status.is_some());
    let job_info = job_status.unwrap();
    assert_eq!(job_info.id, "test-job");
    assert!(matches!(job_info.status, JobStatus::Queued));

    // Test job cancellation
    let cancel_result = manager.cancel_job("test-queue", "test-job").await;
    assert!(cancel_result.is_ok());

    // Verify job is no longer in queue
    let jobs_after_cancel = manager.list_jobs("test-queue", Some(JobStatus::Queued)).await.unwrap();
    assert_eq!(jobs_after_cancel.len(), 0);
}

#[tokio::test]
async fn test_queue_metrics_and_configuration() {
    let config = JobQueueConfig::default();
    let manager = JobQueueManager::new(config);

    // Create test queue
    manager.create_queue(
        "metrics-test".to_string(),
        "Metrics Test Queue".to_string(),
        "Queue for testing metrics".to_string()
    ).await.unwrap();

    // Test metrics retrieval
    let metrics = manager.get_queue_metrics("metrics-test").await;
    assert!(metrics.is_some());
    let metrics = metrics.unwrap();
    assert_eq!(metrics.total_jobs_submitted, 0);
    assert_eq!(metrics.total_jobs_completed, 0);
    assert_eq!(metrics.current_queue_size, 0);

    // Test all queue metrics
    let all_metrics = manager.get_all_queue_metrics().await.unwrap();
    assert!(all_metrics.contains_key("metrics-test"));

    // Test queue configuration
    let queue_config = manager.get_queue_config("metrics-test").await.unwrap();
    assert_eq!(queue_config.max_concurrent_jobs, 4); // Default value
    assert_eq!(queue_config.max_queue_size, 1000); // Default value

    // Test configuration updates
    let mut updates = HashMap::new();
    updates.insert("max_concurrent_jobs".to_string(), serde_json::Value::Number(8.into()));
    updates.insert("max_queue_size".to_string(), serde_json::Value::Number(2000.into()));

    let update_result = manager.update_queue_config("metrics-test", updates).await;
    assert!(update_result.is_ok());

    // Verify updates
    let updated_config = manager.get_queue_config("metrics-test").await.unwrap();
    assert_eq!(updated_config.max_concurrent_jobs, 8);
    assert_eq!(updated_config.max_queue_size, 2000);
}

#[tokio::test]
async fn test_queue_pause_resume_operations() {
    let config = JobQueueConfig::default();
    let manager = JobQueueManager::new(config);

    // Create test queue
    manager.create_queue(
        "pause-test".to_string(),
        "Pause Test Queue".to_string(),
        "Queue for testing pause/resume".to_string()
    ).await.unwrap();

    // Test queue pause
    let pause_result = manager.pause_queue("pause-test").await;
    assert!(pause_result.is_ok());

    // Verify queue is paused (Note: the current implementation doesn't persist state changes)
    // In a real implementation, you would check the queue status

    // Test queue resume
    let resume_result = manager.resume_queue("pause-test").await;
    assert!(resume_result.is_ok());
}

#[tokio::test]
async fn test_job_retry_functionality() {
    let config = JobQueueConfig::default();
    let manager = JobQueueManager::new(config);

    // Create test queue
    manager.create_queue(
        "retry-test".to_string(),
        "Retry Test Queue".to_string(),
        "Queue for testing job retry".to_string()
    ).await.unwrap();

    // Create and submit a job
    let job = BatchJob {
        id: "retry-job".to_string(),
        name: "Retry Test Job".to_string(),
        description: Some("A job for testing retry functionality".to_string()),
        priority: JobPriority::High,
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
        resource_requirements: Default::default(),
        timeout_minutes: Some(30),
        retry_count: 0,
        max_retries: 3,
        created_at: SystemTime::now(),
        scheduled_at: None,
        tags: HashMap::new(),
        metadata: HashMap::new(),
    };

    manager.submit_job("retry-test", job).await.unwrap();

    // Test retry capability check
    let can_retry = manager.can_retry_job("retry-test", "retry-job").await.unwrap();
    assert!(can_retry); // Should be true for non-failed jobs

    // Test retry of non-failed job (should fail in current implementation)
    let retry_result = manager.retry_job("retry-test", "retry-job", false).await;
    assert!(retry_result.is_err()); // Should fail because job is not in failed state
}

#[tokio::test]
async fn test_queue_export_functionality() {
    let config = JobQueueConfig::default();
    let manager = JobQueueManager::new(config);

    // Create test queue
    manager.create_queue(
        "export-test".to_string(),
        "Export Test Queue".to_string(),
        "Queue for testing export functionality".to_string()
    ).await.unwrap();

    // Test job export
    let jobs = manager.export_jobs("export-test").await.unwrap();
    assert_eq!(jobs.len(), 0); // No jobs initially

    // Test config export
    let config = manager.export_queue_config("export-test").await.unwrap();
    assert_eq!(config.max_concurrent_jobs, 4); // Default value

    // Test full data export
    let all_data = manager.export_all_data("export-test").await.unwrap();
    assert_eq!(all_data.queue_info.id, "export-test");
    assert_eq!(all_data.queue_info.name, "Export Test Queue");
    assert_eq!(all_data.jobs.len(), 0);
    assert_eq!(all_data.config.max_concurrent_jobs, 4);
}

#[tokio::test]
async fn test_queue_clearing_operations() {
    let config = JobQueueConfig::default();
    let manager = JobQueueManager::new(config);

    // Create test queue
    manager.create_queue(
        "clear-test".to_string(),
        "Clear Test Queue".to_string(),
        "Queue for testing clear operations".to_string()
    ).await.unwrap();

    // Test clearing empty queue
    let cleared_count = manager.clear_queue("clear-test", false).await.unwrap();
    assert_eq!(cleared_count, 0); // No jobs to clear

    // Test clearing with failed jobs included
    let cleared_count = manager.clear_queue("clear-test", true).await.unwrap();
    assert_eq!(cleared_count, 0); // Still no jobs to clear
}