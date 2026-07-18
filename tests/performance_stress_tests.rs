//! Performance and concurrency stress tests exercised against the real Inferno
//! module APIs.
//!
//! The model-free tests (response cache, audit logging, job-queue submission)
//! run everywhere and assert on exact observable state after every spawned task
//! has joined, so their counts are race-free rather than timing-dependent.
//!
//! The two tests that need real inference are gated on `INFERNO_TEST_MODEL`
//! (a path to a small `.gguf`); they skip cleanly when it is unset. To force
//! cache eviction with a single real model, the model file is copied to several
//! distinct names so `ModelManager::list_models` reports several models.

use anyhow::Result;
use inferno::{
    audit::{
        Actor, ActorType, AuditConfiguration, AuditEvent, AuditLogger, EventContext, EventDetails,
        EventOutcome, EventType, LogLevel, Resource, ResourceType, Severity,
    },
    backends::{Backend, BackendConfig, BackendType, InferenceParams},
    batch::{
        BatchConfig, BatchInput,
        queue::{
            BatchJob, JobPriority, JobQueueConfig, JobQueueManager, ResourceRequirements,
            RetryConfig,
        },
    },
    cache::{CacheConfig, ModelCache},
    models::ModelManager,
    response_cache::{
        CacheKey, HashAlgorithm, ResponseCache, ResponseCacheConfig, ResponseMetadata,
    },
};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::SystemTime,
};
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Thread-safe success/failure tally shared across spawned worker tasks.
#[derive(Default)]
struct OpCounter {
    total: AtomicU64,
    success: AtomicU64,
    failure: AtomicU64,
}

impl OpCounter {
    fn record(&self, ok: bool) {
        self.total.fetch_add(1, Ordering::Relaxed);
        if ok {
            self.success.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failure.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Returns (total, success, failure).
    fn totals(&self) -> (u64, u64, u64) {
        (
            self.total.load(Ordering::Relaxed),
            self.success.load(Ordering::Relaxed),
            self.failure.load(Ordering::Relaxed),
        )
    }
}

fn make_meta() -> ResponseMetadata {
    ResponseMetadata {
        model_id: "test_model".to_string(),
        response_type: "text".to_string(),
        token_count: None,
        processing_time_ms: 0,
        quality_score: None,
        content_type: "text/plain".to_string(),
    }
}

fn make_job(id: &str, priority: JobPriority) -> BatchJob {
    BatchJob {
        id: id.to_string(),
        name: format!("stress job {id}"),
        description: None,
        priority,
        inputs: vec![BatchInput {
            id: format!("{id}-input"),
            content: "stress test input".to_string(),
            metadata: None,
        }],
        inference_params: InferenceParams::default(),
        model_name: "test_model".to_string(),
        batch_config: BatchConfig::default(),
        schedule: None,
        dependencies: vec![],
        resource_requirements: ResourceRequirements::default(),
        timeout_minutes: Some(30),
        retry_count: 0,
        max_retries: 1,
        retry_config: RetryConfig::default(),
        created_at: SystemTime::now(),
        scheduled_at: None,
        tags: HashMap::new(),
        metadata: HashMap::new(),
    }
}

fn make_event(logger_id: usize, event_id: usize) -> AuditEvent {
    AuditEvent {
        id: format!("evt-{logger_id}-{event_id}"),
        timestamp: SystemTime::now(),
        event_type: match event_id % 5 {
            0 => EventType::ApiCall,
            1 => EventType::UserAction,
            2 => EventType::ModelManagement,
            3 => EventType::DataAccess,
            _ => EventType::PerformanceEvent,
        },
        severity: match event_id % 4 {
            0 => Severity::Info,
            1 => Severity::Low,
            2 => Severity::Medium,
            _ => Severity::High,
        },
        actor: Actor {
            actor_type: ActorType::User,
            id: format!("user_{logger_id}"),
            name: format!("User {logger_id}"),
            ip_address: None,
            user_agent: None,
            session_id: None,
        },
        resource: Resource {
            resource_type: ResourceType::Api,
            id: format!("resource_{logger_id}_{event_id}"),
            name: format!("Resource {logger_id} {event_id}"),
            path: None,
            owner: None,
            tags: vec![],
        },
        action: format!("perf_test_action_{}", event_id % 10),
        details: EventDetails {
            description: format!("Performance test event {event_id} from logger {logger_id}"),
            parameters: HashMap::new(),
            request_id: None,
            correlation_id: None,
            trace_id: None,
            parent_event_id: None,
        },
        context: EventContext {
            environment: "test".to_string(),
            application: "performance_test".to_string(),
            version: "1.0.0".to_string(),
            hostname: "localhost".to_string(),
            process_id: 0,
            thread_id: None,
            request_path: None,
            request_method: None,
            client_info: None,
        },
        outcome: EventOutcome {
            success: event_id % 20 != 0,
            status_code: None,
            error_code: None,
            error_message: None,
            duration_ms: Some(10),
            bytes_processed: None,
            records_affected: None,
        },
        metadata: HashMap::new(),
    }
}

fn cpu_backend_config() -> BackendConfig {
    BackendConfig {
        gpu_enabled: false,
        gpu_device: None,
        cpu_threads: Some(2),
        context_size: 512,
        batch_size: 8,
        memory_map: true,
    }
}

fn small_params() -> InferenceParams {
    InferenceParams {
        max_tokens: 16,
        ..Default::default()
    }
}

/// Copies `INFERNO_TEST_MODEL` to `count` distinct filenames under a fresh temp
/// models directory. Returns None (test skips) when the env var is unset. The
/// TempDir is returned so the caller keeps it alive for the test's duration.
fn copy_test_model(count: usize) -> Option<(TempDir, PathBuf)> {
    let src = std::env::var("INFERNO_TEST_MODEL").ok()?;
    let tmp = TempDir::new().expect("create tempdir");
    let models_dir = tmp.path().join("models");
    std::fs::create_dir_all(&models_dir).expect("create models dir");
    for i in 0..count {
        std::fs::copy(&src, models_dir.join(format!("perf_model_{i}.gguf")))
            .expect("copy test model");
    }
    Some((tmp, models_dir))
}

/// Concurrent response-cache load: many workers mixing hits against a
/// pre-populated set and misses that store-then-read new keys.
#[tokio::test]
async fn test_response_cache_concurrent_load() -> Result<()> {
    let config = ResponseCacheConfig {
        enabled: true,
        max_entries: 10_000,
        max_memory_mb: 512,
        ttl_seconds: 3600,
        compression_enabled: false,
        ..Default::default()
    };
    let cache = Arc::new(ResponseCache::new(config, None).await?);

    // Pre-populate 100 entries for guaranteed hits.
    let base_keys: Vec<CacheKey> = (0..100)
        .map(|i| {
            CacheKey::new(
                &format!("base_input_{i}"),
                "test_model",
                "default",
                &HashAlgorithm::Sha256,
            )
        })
        .collect();
    for (i, key) in base_keys.iter().enumerate() {
        cache
            .put(
                key,
                format!("cached response {i}").into_bytes(),
                make_meta(),
            )
            .await?;
    }

    let counter = Arc::new(OpCounter::default());
    let workers = 10usize;
    let ops_per_worker = 200usize;
    // 30% of each worker's ops target the pre-populated keys (guaranteed hits).
    let hits_per_worker = 60usize; // ops 0..60 satisfy op/200 < 0.30

    let mut handles = Vec::new();
    for w in 0..workers {
        let cache = cache.clone();
        let counter = counter.clone();
        let base_keys = base_keys.clone();
        handles.push(tokio::spawn(async move {
            for op in 0..ops_per_worker {
                let is_hit = (op as f64 / ops_per_worker as f64) < 0.30;
                let ok = if is_hit {
                    cache.get(&base_keys[op % base_keys.len()]).await.is_some()
                } else {
                    let key = CacheKey::new(
                        &format!("worker_{w}_input_{op}"),
                        "test_model",
                        "default",
                        &HashAlgorithm::Sha256,
                    );
                    let stored = cache
                        .put(&key, format!("response {w}:{op}").into_bytes(), make_meta())
                        .await
                        .is_ok();
                    stored && cache.get(&key).await.is_some()
                };
                counter.record(ok);
            }
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    let (total, _success, failure) = counter.totals();
    let stats = cache.get_stats().await;

    assert_eq!(
        total,
        (workers * ops_per_worker) as u64,
        "every worker operation should be tallied"
    );
    assert_eq!(failure, 0, "all cache operations should succeed");
    assert!(
        stats.cache_hits >= (workers * hits_per_worker) as u64,
        "reads against the pre-populated keys must register as hits (got {})",
        stats.cache_hits
    );
    assert!(
        stats.total_entries >= base_keys.len(),
        "pre-populated entries should persist"
    );
    assert!(
        stats.memory_usage_mb <= 512.0,
        "memory footprint should stay within the configured limit"
    );

    Ok(())
}

/// High-volume concurrent audit logging. `LogLevel::All` keeps every severity
/// and a large batch_size prevents the query buffer from draining, so the
/// statistics count is exactly the number of events logged.
#[tokio::test]
async fn test_audit_logging_throughput() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = AuditConfiguration {
        enabled: true,
        log_level: LogLevel::All,
        storage_path: temp_dir.path().join("audit"),
        alert_on_critical: false,
        // Large enough that buffer.len() never exceeds batch_size * 2 for the
        // 4000 events below, so get_statistics() reflects the full count.
        batch_size: 20_000,
        ..Default::default()
    };
    let logger = Arc::new(AuditLogger::new(config).await?);

    let counter = Arc::new(OpCounter::default());
    let loggers = 8usize;
    let events_per_logger = 500usize;

    let mut handles = Vec::new();
    for logger_id in 0..loggers {
        let logger = logger.clone();
        let counter = counter.clone();
        handles.push(tokio::spawn(async move {
            for event_id in 0..events_per_logger {
                let ok = logger
                    .log_event(make_event(logger_id, event_id))
                    .await
                    .is_ok();
                counter.record(ok);
            }
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    let (total, _success, failure) = counter.totals();
    assert_eq!(total, (loggers * events_per_logger) as u64);
    assert_eq!(failure, 0, "audit logging should not fail under load");

    let stats = logger.get_statistics().await?;
    assert_eq!(
        stats.total_events,
        loggers * events_per_logger,
        "every logged event should be present in the query buffer"
    );

    logger.shutdown().await;
    Ok(())
}

/// Concurrent job submission across many tasks into a single queue. The queue's
/// submitted-job counter must equal the total number of successful submissions.
#[tokio::test]
async fn test_job_queue_submission_scalability() -> Result<()> {
    let config = JobQueueConfig {
        max_queue_size: 10_000,
        max_concurrent_jobs: 10,
        ..Default::default()
    };
    let manager = Arc::new(JobQueueManager::new(config));
    manager
        .create_queue(
            "scale".to_string(),
            "Scalability Queue".to_string(),
            "High-throughput submission test".to_string(),
        )
        .await?;

    let counter = Arc::new(OpCounter::default());
    let submitters = 8usize;
    let jobs_per_submitter = 25usize;

    let mut handles = Vec::new();
    for s in 0..submitters {
        let manager = manager.clone();
        let counter = counter.clone();
        handles.push(tokio::spawn(async move {
            for j in 0..jobs_per_submitter {
                let job = make_job(&format!("job-{s}-{j}"), JobPriority::Normal);
                let ok = manager.submit_job("scale", job).await.is_ok();
                counter.record(ok);
            }
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    let (total, _success, failure) = counter.totals();
    let expected = (submitters * jobs_per_submitter) as u64;
    assert_eq!(total, expected);
    assert_eq!(
        failure, 0,
        "submissions should not fail under the queue limit"
    );

    let metrics = manager
        .get_queue_metrics("scale")
        .await
        .expect("queue should exist");
    assert_eq!(
        metrics.total_jobs_submitted, expected,
        "every submitted job must be counted in queue metrics"
    );

    Ok(())
}

/// Concurrent inference stress across several backend instances sharing one
/// loaded model. Gated on `INFERNO_TEST_MODEL`.
#[tokio::test]
async fn test_backend_concurrent_inference_stress() -> Result<()> {
    let Some((_tmp, models_dir)) = copy_test_model(1) else {
        eprintln!("INFERNO_TEST_MODEL not set; skipping backend concurrent inference stress test");
        return Ok(());
    };

    let manager = ModelManager::new(&models_dir);
    let models = manager.list_models().await?;
    assert!(!models.is_empty(), "copied model should be discovered");
    let model_info = models[0].clone();

    let backend_config = cpu_backend_config();
    let num_backends = 3usize;
    let mut backends = Vec::new();
    for _ in 0..num_backends {
        let mut backend = Backend::new(BackendType::Gguf, &backend_config)?;
        backend.load_model(&model_info).await?;
        backends.push(Arc::new(RwLock::new(backend)));
    }

    let counter = Arc::new(OpCounter::default());
    let concurrent_tasks = 6usize;
    let requests_per_task = 2usize;

    let mut handles = Vec::new();
    for i in 0..concurrent_tasks {
        let backend = backends[i % backends.len()].clone();
        let counter = counter.clone();
        handles.push(tokio::spawn(async move {
            for j in 0..requests_per_task {
                let input = format!("Performance test request {j} from task {i}");
                let result = {
                    let mut guard = backend.write().await;
                    guard.infer(&input, &small_params()).await
                };
                counter.record(result.is_ok());
            }
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    let (total, success, _failure) = counter.totals();
    assert_eq!(
        total,
        (concurrent_tasks * requests_per_task) as u64,
        "every request should be attempted"
    );
    assert_eq!(
        success, total,
        "all concurrent inferences against a valid model should succeed"
    );

    Ok(())
}

/// Cache eviction under pressure: load more distinct models than the cache
/// capacity and confirm the cache both bounds its size and records evictions.
/// Gated on `INFERNO_TEST_MODEL` (the model is copied to 5 distinct names).
#[tokio::test]
async fn test_cache_eviction_under_pressure() -> Result<()> {
    let model_count = 5usize;
    let max_cached = 2usize;
    let Some((_tmp, models_dir)) = copy_test_model(model_count) else {
        eprintln!("INFERNO_TEST_MODEL not set; skipping cache eviction test");
        return Ok(());
    };

    let manager = Arc::new(ModelManager::new(&models_dir));
    let models = manager.list_models().await?;
    assert_eq!(
        models.len(),
        model_count,
        "all copied models should be discovered"
    );

    let cache_config = CacheConfig {
        max_cached_models: max_cached,
        max_memory_mb: 4096,
        model_ttl_seconds: 3600,
        memory_based_eviction: true,
        ..Default::default()
    };
    let cache = Arc::new(ModelCache::new(cache_config, cpu_backend_config(), manager, None).await?);

    // Load every model once, running a tiny inference to exercise the backend.
    for model in &models {
        let cached = cache.get_model(&model.name).await?;
        let _ = cached.backend.infer("hi", &small_params()).await?;
    }

    let stats = cache.get_stats().await;
    assert!(
        stats.total_models <= max_cached,
        "cache must not hold more than max_cached_models (held {})",
        stats.total_models
    );
    assert!(
        stats.eviction_count > 0,
        "loading {model_count} distinct models with a cap of {max_cached} must evict"
    );

    Ok(())
}
