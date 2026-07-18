//! Integration tests for the batch job-queue subsystem.
//!
//! These exercise the real `inferno::batch` API: the `JobQueueManager` job
//! lifecycle, the `JobScheduler`, and (gated on a real model) the
//! `BatchProcessor` execution path.
//!
//! The suite deliberately covers only capabilities that exist in the module.
//! The prior version targeted an imagined feature set - dependency graphs,
//! dead-letter queues, resource scheduling, historical metrics, and queue
//! state persistence/recovery - none of which exist in `src/batch`, so those
//! tests are gone rather than asserted against stubs.
//!
//! Behavioral facts that shape the assertions:
//! - `JobQueueManager::new` is synchronous and takes a `JobQueueConfig`.
//! - `submit_job` synchronously increments `total_jobs_submitted` but does NOT
//!   update `QueueMetrics::current_queue_size`, so live counts come from
//!   `get_queue_job_counts` / `list_jobs`, not the metric.
//! - New queues start in `QueueStatus::Stopped`.
//! - `cancel_job` on a queued job removes it entirely.
//! - `clear_queue` clears only completed/failed jobs, never queued ones.
//! - `JobScheduler` wires its channel in `start()`, and `add_scheduled_job`
//!   flows through that channel, so callers must `start()` and then yield
//!   before `list_scheduled_jobs()` reflects the add.

use inferno::backends::InferenceParams;
use inferno::batch::{
    BatchConfig, BatchInput,
    queue::{
        BatchJob, JobPriority, JobQueueConfig, JobQueueManager, JobSchedule, JobStatus,
        ResourceRequirements, ScheduleType,
    },
    scheduler::{JobScheduler, ScheduledJobEntry},
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

/// Build a valid `BatchJob`. Passes `validate_job` (non-empty inputs + model
/// name, non-zero memory). `schedule` defaults to `None`; callers that need a
/// schedule set it on the returned job.
fn make_job(name: &str, priority: JobPriority) -> BatchJob {
    BatchJob {
        id: String::new(), // submit_job assigns a UUID when empty
        name: name.to_string(),
        description: Some(format!("test job {name}")),
        priority,
        inputs: vec![BatchInput {
            id: "input-1".to_string(),
            content: "hello".to_string(),
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
        retry_config: Default::default(),
        created_at: SystemTime::now(),
        scheduled_at: None,
        tags: HashMap::new(),
        metadata: HashMap::new(),
    }
}

/// A manager over a default config, plus one created (Stopped) queue.
async fn manager_with_queue(queue_id: &str) -> Arc<JobQueueManager> {
    let manager = Arc::new(JobQueueManager::new(JobQueueConfig::default()));
    manager
        .create_queue(
            queue_id.to_string(),
            "Test Queue".to_string(),
            "desc".to_string(),
        )
        .await
        .expect("create_queue should succeed");
    manager
}

/// Submitting jobs makes them queryable and updates the submitted-jobs metric.
#[tokio::test]
async fn test_submit_and_list_jobs() {
    let qid = "batch-submit-list";
    let manager = manager_with_queue(qid).await;

    for i in 0..3 {
        let id = manager
            .submit_job(qid, make_job(&format!("job-{i}"), JobPriority::Normal))
            .await
            .expect("submit_job should succeed");
        assert!(!id.is_empty(), "submit_job returns the assigned job id");
    }

    // All three land in the queued state.
    let (queued, running, completed) = manager.get_queue_job_counts(qid).await.unwrap();
    assert_eq!((queued, running, completed), (3, 0, 0));

    let jobs = manager.list_jobs(qid, None).await.unwrap();
    assert_eq!(jobs.len(), 3);
    assert!(jobs.iter().all(|j| matches!(j.status, JobStatus::Queued)));

    // Filtering by a non-queued status yields nothing (nothing has run).
    let running_jobs = manager
        .list_jobs(qid, Some(JobStatus::Running))
        .await
        .unwrap();
    assert!(running_jobs.is_empty());

    // The submitted-jobs counter reflects the three submissions.
    let metrics = manager.get_queue_metrics(qid).await.unwrap();
    assert_eq!(metrics.total_jobs_submitted, 3);

    manager.save_queue(qid).await.ok();
}

/// A submitted job is individually retrievable and preserves its payload.
#[tokio::test]
async fn test_get_job_status_roundtrip() {
    let qid = "batch-get-status";
    let manager = manager_with_queue(qid).await;

    let job_id = manager
        .submit_job(qid, make_job("solo", JobPriority::High))
        .await
        .unwrap();

    let info = manager
        .get_job_status(qid, &job_id)
        .await
        .unwrap()
        .expect("job should be found");
    assert_eq!(info.id, job_id);
    assert_eq!(info.name, "solo");
    assert!(matches!(info.status, JobStatus::Queued));
    assert!(matches!(info.priority, JobPriority::High));

    // Unknown ids return Ok(None), not an error.
    let missing = manager.get_job_status(qid, "no-such-job").await.unwrap();
    assert!(missing.is_none());
}

/// Cancelling a queued job removes it from the queue.
#[tokio::test]
async fn test_cancel_queued_job() {
    let qid = "batch-cancel";
    let manager = manager_with_queue(qid).await;

    let keep = manager
        .submit_job(qid, make_job("keep", JobPriority::Normal))
        .await
        .unwrap();
    let drop = manager
        .submit_job(qid, make_job("drop", JobPriority::Normal))
        .await
        .unwrap();

    manager
        .cancel_job(qid, &drop)
        .await
        .expect("cancel should succeed");

    let (queued, _, _) = manager.get_queue_job_counts(qid).await.unwrap();
    assert_eq!(queued, 1, "one job remains after cancelling the other");
    assert!(
        manager.get_job_status(qid, &drop).await.unwrap().is_none(),
        "cancelled job is gone"
    );
    assert!(
        manager.get_job_status(qid, &keep).await.unwrap().is_some(),
        "the other job is untouched"
    );

    // Cancelling an unknown job is an error.
    assert!(manager.cancel_job(qid, "no-such-job").await.is_err());
}

/// Pause/resume flips the queue status; new queues start Stopped.
#[tokio::test]
async fn test_pause_resume_status() {
    let qid = "batch-pause";
    let manager = manager_with_queue(qid).await;

    // A freshly created queue is Stopped.
    assert!(matches!(
        manager.get_queue_status(qid).await,
        Some(inferno::batch::queue::QueueStatus::Stopped)
    ));

    manager.pause_queue(qid).await.unwrap();
    assert!(matches!(
        manager.get_queue_status(qid).await,
        Some(inferno::batch::queue::QueueStatus::Paused)
    ));

    manager.resume_queue(qid).await.unwrap();
    assert!(matches!(
        manager.get_queue_status(qid).await,
        Some(inferno::batch::queue::QueueStatus::Running)
    ));
}

/// `clear_queue` clears only terminal (completed/failed) jobs; queued jobs are
/// left in place. With nothing run yet, it clears zero and the queue is intact.
#[tokio::test]
async fn test_clear_queue_leaves_queued_jobs() {
    let qid = "batch-clear";
    let manager = manager_with_queue(qid).await;

    for i in 0..2 {
        manager
            .submit_job(qid, make_job(&format!("job-{i}"), JobPriority::Normal))
            .await
            .unwrap();
    }

    let cleared = manager.clear_queue(qid, true).await.unwrap();
    assert_eq!(cleared, 0, "no completed/failed jobs to clear");

    let (queued, _, _) = manager.get_queue_job_counts(qid).await.unwrap();
    assert_eq!(queued, 2, "queued jobs are not affected by clear_queue");
}

/// Export surfaces the queue's jobs, config, and identity.
#[tokio::test]
async fn test_export_queue_data() {
    let qid = "batch-export";
    let manager = manager_with_queue(qid).await;

    for i in 0..2 {
        manager
            .submit_job(qid, make_job(&format!("job-{i}"), JobPriority::Normal))
            .await
            .unwrap();
    }

    let jobs = manager.export_jobs(qid).await.unwrap();
    assert_eq!(jobs.len(), 2);

    // A queue inherits the manager's config (default max_concurrent_jobs = 4).
    let config = manager.export_queue_config(qid).await.unwrap();
    assert_eq!(config.max_concurrent_jobs, 4);

    let all = manager.export_all_data(qid).await.unwrap();
    assert_eq!(all.queue_info.id, qid);
    assert_eq!(all.jobs.len(), 2);
}

/// Multiple queues are independent, and `list_all_queues` sees them all.
#[tokio::test]
async fn test_multiple_queues_independent() {
    let manager = Arc::new(JobQueueManager::new(JobQueueConfig::default()));
    manager
        .create_queue("q-alpha".into(), "Alpha".into(), "a".into())
        .await
        .unwrap();
    manager
        .create_queue("q-beta".into(), "Beta".into(), "b".into())
        .await
        .unwrap();

    // Creating a duplicate id is rejected.
    assert!(
        manager
            .create_queue("q-alpha".into(), "Dup".into(), "d".into())
            .await
            .is_err()
    );

    manager
        .submit_job("q-alpha", make_job("a1", JobPriority::Normal))
        .await
        .unwrap();
    manager
        .submit_job("q-alpha", make_job("a2", JobPriority::Normal))
        .await
        .unwrap();
    manager
        .submit_job("q-beta", make_job("b1", JobPriority::Normal))
        .await
        .unwrap();

    assert_eq!(manager.get_queue_job_counts("q-alpha").await.unwrap().0, 2);
    assert_eq!(manager.get_queue_job_counts("q-beta").await.unwrap().0, 1);

    let all = manager.list_all_queues().await.unwrap();
    assert_eq!(all.len(), 2);

    // Per-queue metrics are tracked independently.
    let per_queue = manager.get_all_queue_metrics().await.unwrap();
    assert_eq!(per_queue.get("q-alpha").unwrap().total_jobs_submitted, 2);
    assert_eq!(per_queue.get("q-beta").unwrap().total_jobs_submitted, 1);
}

/// The scheduler registers scheduled entries and lists them back.
#[tokio::test]
async fn test_scheduler_add_and_list() {
    let qid = "batch-sched";
    let manager = manager_with_queue(qid).await;

    let mut scheduler = JobScheduler::new(manager.clone());
    // start() wires the channel that add_scheduled_job feeds.
    scheduler.start().await.expect("scheduler start");

    let mut job = make_job("scheduled", JobPriority::Normal);
    job.id = "sched-1".to_string();
    job.schedule = Some(JobSchedule {
        schedule_type: ScheduleType::Interval {
            interval_minutes: 60,
            max_runs: None,
        },
        start_time: None,
        end_time: None,
        timezone: "UTC".to_string(),
        enabled: true,
    });

    let entry = ScheduledJobEntry {
        job,
        next_run: SystemTime::now() + Duration::from_secs(3600),
        last_run: None,
        run_count: 0,
        enabled: true,
        queue_id: qid.to_string(),
    };
    scheduler
        .add_scheduled_job(entry)
        .await
        .expect("add_scheduled_job");

    // The add flows through an mpsc channel processed by the spawned loop, so
    // poll until it lands rather than sleeping a fixed interval.
    let mut listed = scheduler.list_scheduled_jobs().await;
    for _ in 0..40 {
        if !listed.is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        listed = scheduler.list_scheduled_jobs().await;
    }
    assert_eq!(listed.len(), 1, "the scheduled job should be registered");
    assert_eq!(listed[0].queue_id, qid);
    assert_eq!(listed[0].job.id, "sched-1");

    // The entry is individually retrievable and can be toggled.
    assert!(scheduler.get_scheduled_job("sched-1").await.is_some());
    scheduler.disable_job("sched-1").await.expect("disable");
    scheduler.enable_job("sched-1").await.expect("enable");

    scheduler.stop().await.expect("scheduler stop");
}

/// Real end-to-end batch execution through a GGUF backend. Gated on
/// INFERNO_TEST_MODEL (a path to a small `.gguf`) since it loads a model and
/// runs inference; skipped silently when the env var is absent.
#[tokio::test]
async fn test_batch_processor_real_inference() {
    let Ok(model_path) = std::env::var("INFERNO_TEST_MODEL") else {
        eprintln!("INFERNO_TEST_MODEL not set; skipping real batch inference test");
        return;
    };

    use inferno::backends::{Backend, BackendConfig, BackendType};
    use inferno::batch::BatchProcessor;
    use inferno::models::ModelManager;
    use std::path::PathBuf;

    let path = PathBuf::from(&model_path);
    let models_dir = path.parent().expect("model path has a parent");
    // resolve_model reads the file and builds a correct ModelInfo.
    let model_info = ModelManager::new(models_dir)
        .resolve_model(path.to_str().expect("model path is UTF-8"))
        .await
        .expect("resolve model");

    // CPU-only backend keeps the test deterministic and Metal-independent.
    let backend_config = BackendConfig {
        gpu_enabled: false,
        gpu_device: None,
        cpu_threads: Some(2),
        context_size: 512,
        batch_size: 8,
        memory_map: true,
    };
    let mut backend =
        Backend::new(BackendType::Gguf, &backend_config).expect("create gguf backend");
    backend.load_model(&model_info).await.expect("load model");

    let inputs = vec![
        BatchInput {
            id: "1".into(),
            content: "The sky is".into(),
            metadata: None,
        },
        BatchInput {
            id: "2".into(),
            content: "Once upon a".into(),
            metadata: None,
        },
        BatchInput {
            id: "3".into(),
            content: "Numbers go".into(),
            metadata: None,
        },
    ];
    let params = InferenceParams {
        max_tokens: 8,
        ..Default::default()
    };

    let processor = BatchProcessor::new(BatchConfig::default(), inputs.len());
    let progress = processor
        .process_inputs(&mut backend, inputs, None, &params)
        .await
        .expect("process_inputs should succeed");

    assert_eq!(progress.total_items, 3);
    assert_eq!(progress.completed_items, 3, "all inputs should complete");
    assert_eq!(progress.failed_items, 0);
}
