//! Cross-component integration tests exercised against the real Inferno APIs.
//!
//! These verify that the independently-tested components (model discovery,
//! model cache, response cache, audit log, job queue) cooperate correctly.
//!
//! Model-free tests (resilience, concurrency) run everywhere. The two tests
//! that load and run a model are gated on `INFERNO_TEST_MODEL` (a path to a
//! small `.gguf`) and skip cleanly when it is unset. Audit event counts are
//! asserted after every spawned task has joined and are isolated by a unique
//! actor, so they are exact rather than timing-dependent.

use anyhow::Result;
use inferno::{
    audit::{
        Actor, ActorType, AuditConfiguration, AuditEvent, AuditLogger, AuditQuery, EventContext,
        EventDetails, EventOutcome, EventType, LogLevel, Resource, ResourceType, Severity,
    },
    backends::{BackendConfig, InferenceParams},
    batch::{
        BatchConfig, BatchInput,
        queue::{
            BatchJob, JobPriority, JobQueueConfig, JobQueueManager, JobStatus,
            ResourceRequirements, RetryConfig,
        },
    },
    cache::{CacheConfig, ModelCache},
    models::{ModelInfo, ModelManager},
    response_cache::{
        CacheKey, HashAlgorithm, ResponseCache, ResponseCacheConfig, ResponseMetadata,
    },
};
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::SystemTime};
use tempfile::TempDir;
use uuid::Uuid;

/// Real components wired together for a single test. `_temp` keeps the backing
/// directories alive for the lifetime of the environment.
struct TestEnv {
    model_cache: Arc<ModelCache>,
    audit: Arc<AuditLogger>,
    queue: Arc<JobQueueManager>,
    response_cache: Arc<ResponseCache>,
    models: Vec<ModelInfo>,
    _temp: TempDir,
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

fn write_mock_gguf(path: &PathBuf, model_name: &str) -> Result<()> {
    let mut content = Vec::new();
    content.extend_from_slice(b"GGUF");
    content.extend_from_slice(&3u32.to_le_bytes());
    content.extend_from_slice(&0u64.to_le_bytes());
    content.extend_from_slice(&1u64.to_le_bytes());
    let key = "general.name";
    content.extend_from_slice(&(key.len() as u64).to_le_bytes());
    content.extend_from_slice(key.as_bytes());
    content.extend_from_slice(&8u32.to_le_bytes());
    content.extend_from_slice(&(model_name.len() as u64).to_le_bytes());
    content.extend_from_slice(model_name.as_bytes());
    content.resize(4096, 0);
    std::fs::write(path, content)?;
    Ok(())
}

/// Builds a full component environment. When `real_model` is true the models
/// directory is populated with copies of `INFERNO_TEST_MODEL` (returning None
/// if it is unset so the caller can skip); otherwise it is populated with mock
/// GGUF files that are discoverable but not loadable.
async fn build_env(real_model: bool) -> Result<Option<TestEnv>> {
    let temp = TempDir::new()?;
    let models_dir = temp.path().join("models");
    std::fs::create_dir_all(&models_dir)?;

    if real_model {
        let Ok(src) = std::env::var("INFERNO_TEST_MODEL") else {
            return Ok(None);
        };
        for name in ["model_a", "model_b"] {
            std::fs::copy(&src, models_dir.join(format!("{name}.gguf")))?;
        }
    } else {
        write_mock_gguf(&models_dir.join("test_model_1.gguf"), "Test Model 1")?;
        write_mock_gguf(&models_dir.join("test_model_2.gguf"), "Test Model 2")?;
    }

    let model_manager = Arc::new(ModelManager::new(&models_dir));
    let models = model_manager.list_models().await?;

    let cache_config = CacheConfig {
        max_cached_models: 3,
        max_memory_mb: 4096,
        model_ttl_seconds: 300,
        memory_based_eviction: true,
        ..Default::default()
    };
    let model_cache =
        Arc::new(ModelCache::new(cache_config, cpu_backend_config(), model_manager, None).await?);

    let audit_config = AuditConfiguration {
        enabled: true,
        log_level: LogLevel::All,
        storage_path: temp.path().join("audit"),
        alert_on_critical: false,
        batch_size: 20_000,
        ..Default::default()
    };
    let audit = Arc::new(AuditLogger::new(audit_config).await?);

    let queue = Arc::new(JobQueueManager::new(JobQueueConfig {
        max_queue_size: 1000,
        max_concurrent_jobs: 3,
        ..Default::default()
    }));

    let response_cache_config = ResponseCacheConfig {
        enabled: true,
        max_entries: 1000,
        ttl_seconds: 3600,
        max_memory_mb: 100,
        compression_enabled: false,
        ..Default::default()
    };
    let response_cache = Arc::new(ResponseCache::new(response_cache_config, None).await?);

    Ok(Some(TestEnv {
        model_cache,
        audit,
        queue,
        response_cache,
        models,
        _temp: temp,
    }))
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

fn make_job(id: &str, model_name: &str) -> BatchJob {
    BatchJob {
        id: id.to_string(),
        name: format!("Integration Test Job {id}"),
        description: None,
        priority: JobPriority::Normal,
        inputs: vec![BatchInput {
            id: format!("{id}-input"),
            content: "What is machine learning?".to_string(),
            metadata: None,
        }],
        inference_params: InferenceParams::default(),
        model_name: model_name.to_string(),
        batch_config: BatchConfig::default(),
        schedule: None,
        dependencies: vec![],
        resource_requirements: ResourceRequirements::default(),
        timeout_minutes: Some(10),
        retry_count: 0,
        max_retries: 2,
        retry_config: RetryConfig::default(),
        created_at: SystemTime::now(),
        scheduled_at: None,
        tags: HashMap::new(),
        metadata: HashMap::new(),
    }
}

/// Builds a fully-populated audit event. `actor_name` is used to isolate a
/// test's events for exact-count queries.
fn make_event(actor_id: &str, actor_name: &str, action: &str) -> AuditEvent {
    AuditEvent {
        id: Uuid::new_v4().to_string(),
        timestamp: SystemTime::now(),
        event_type: EventType::ModelManagement,
        severity: Severity::Info,
        actor: Actor {
            actor_type: ActorType::System,
            id: actor_id.to_string(),
            name: actor_name.to_string(),
            ip_address: None,
            user_agent: None,
            session_id: None,
        },
        resource: Resource {
            resource_type: ResourceType::Model,
            id: "model-resource".to_string(),
            name: "model".to_string(),
            path: None,
            owner: None,
            tags: vec![],
        },
        action: action.to_string(),
        details: EventDetails {
            description: format!("cross-component test event: {action}"),
            parameters: HashMap::new(),
            request_id: None,
            correlation_id: None,
            trace_id: None,
            parent_event_id: None,
        },
        context: EventContext {
            environment: "test".to_string(),
            application: "cross_component_test".to_string(),
            version: "1.0.0".to_string(),
            hostname: "localhost".to_string(),
            process_id: 0,
            thread_id: None,
            request_path: None,
            request_method: None,
            client_info: None,
        },
        outcome: EventOutcome {
            success: true,
            status_code: Some(200),
            error_code: None,
            error_message: None,
            duration_ms: Some(10),
            bytes_processed: None,
            records_affected: Some(1),
        },
        metadata: HashMap::new(),
    }
}

fn query_by_actor(actor_name: &str) -> AuditQuery {
    AuditQuery {
        actors: Some(vec![actor_name.to_string()]),
        limit: Some(500),
        ..Default::default()
    }
}

/// End-to-end model lifecycle: discover -> audit -> cache-load+infer ->
/// response-cache roundtrip -> queue submit. Gated on `INFERNO_TEST_MODEL`.
#[tokio::test]
async fn test_end_to_end_model_lifecycle() -> Result<()> {
    let Some(env) = build_env(true).await? else {
        eprintln!("INFERNO_TEST_MODEL not set; skipping end-to-end lifecycle test");
        return Ok(());
    };

    assert!(!env.models.is_empty(), "should discover copied models");
    let model = env.models[0].clone();

    // 1. Audit the discovery through the real logger.
    env.audit
        .log_event(make_event(
            "model_manager",
            "lifecycle_actor",
            "model_discovered",
        ))
        .await?;

    // 2. Load the model through the cache and run inference.
    let cached = env.model_cache.get_model(&model.name).await?;
    let input = "What is artificial intelligence?";
    let result = cached
        .backend
        .infer(input, &InferenceParams::default())
        .await?;
    assert!(!result.is_empty(), "inference should produce output");

    // 3. Store the response and read it straight back.
    let key = CacheKey::new(input, &model.name, "default", &HashAlgorithm::Sha256);
    env.response_cache
        .put(&key, result.clone().into_bytes(), make_meta())
        .await?;
    let cached_bytes = env.response_cache.get(&key).await;
    assert_eq!(
        cached_bytes,
        Some(result.into_bytes()),
        "cached response should match the inference output"
    );

    // 4. Submit a batch job describing the same model.
    env.queue
        .create_queue(
            "lifecycle".to_string(),
            "Lifecycle Queue".to_string(),
            "Cross-component lifecycle test".to_string(),
        )
        .await?;
    env.queue
        .submit_job("lifecycle", make_job("lifecycle-job", &model.name))
        .await?;
    let metrics = env
        .queue
        .get_queue_metrics("lifecycle")
        .await
        .expect("queue exists");
    assert_eq!(metrics.total_jobs_submitted, 1);

    // 5. The discovery event is queryable.
    let events = env
        .audit
        .query_events(query_by_actor("lifecycle_actor"))
        .await?;
    assert_eq!(events.len(), 1, "the discovery event should be recorded");
    assert_eq!(events[0].action, "model_discovered");

    // 6. The cache registered the loaded model.
    let stats = env.model_cache.get_stats().await;
    assert!(
        stats.total_models >= 1,
        "cache should hold the loaded model"
    );

    env.audit.shutdown().await;
    Ok(())
}

/// Resilience: failures in one component surface as errors without corrupting
/// the others. Model-free (the model load targets a nonexistent name).
#[tokio::test]
async fn test_system_resilience_and_error_handling() -> Result<()> {
    let env = build_env(false).await?.expect("mock env always builds");

    // Loading a model that does not exist fails cleanly.
    let load_result = env.model_cache.get_model("nonexistent_model").await;
    assert!(
        load_result.is_err(),
        "loading a nonexistent model should fail"
    );

    // A job naming an invalid model is still accepted by the queue (the queue
    // does not validate model existence) and sits in the Queued state.
    env.queue
        .create_queue(
            "error".to_string(),
            "Error Queue".to_string(),
            "Error handling test".to_string(),
        )
        .await?;
    env.queue
        .submit_job("error", make_job("error-job", "nonexistent_model"))
        .await?;
    let status = env.queue.get_job_status("error", "error-job").await?;
    let job_info = status.expect("submitted job should have a status");
    assert!(
        matches!(job_info.status, JobStatus::Queued),
        "an unprocessed job should be queued"
    );

    // A miss on the response cache returns None rather than erroring.
    let miss_key = CacheKey::new("absent", "test_model", "default", &HashAlgorithm::Sha256);
    assert!(
        env.response_cache.get(&miss_key).await.is_none(),
        "an empty cache should return None"
    );

    env.audit.shutdown().await;
    Ok(())
}

/// Concurrent operations across the response cache, job queue, and audit log.
/// Model-free. Exact counts are asserted after all tasks join.
#[tokio::test]
async fn test_concurrent_cross_component_operations() -> Result<()> {
    let env = build_env(false).await?.expect("mock env always builds");

    env.queue
        .create_queue(
            "concurrent".to_string(),
            "Concurrent Queue".to_string(),
            "Concurrent cross-component test".to_string(),
        )
        .await?;

    let mut handles = Vec::new();

    // Concurrent response caching: 3 tasks x 5 store+read.
    for i in 0..3 {
        let response_cache = env.response_cache.clone();
        handles.push(tokio::spawn(async move {
            for j in 0..5 {
                let key = CacheKey::new(
                    &format!("concurrent_input_{i}_{j}"),
                    "test_model",
                    "default",
                    &HashAlgorithm::Sha256,
                );
                let _ = response_cache
                    .put(&key, format!("response {i}:{j}").into_bytes(), make_meta())
                    .await;
                let _ = response_cache.get(&key).await;
            }
        }));
    }

    // Concurrent job submission: 5 jobs.
    for i in 0..5 {
        let queue = env.queue.clone();
        handles.push(tokio::spawn(async move {
            let _ = queue
                .submit_job(
                    "concurrent",
                    make_job(&format!("concurrent-job-{i}"), "test_model"),
                )
                .await;
        }));
    }

    // Concurrent audit logging: 3 tasks x 10 events, isolated by actor name.
    for i in 0..3 {
        let audit = env.audit.clone();
        handles.push(tokio::spawn(async move {
            for j in 0..10 {
                let _ = audit
                    .log_event(make_event(
                        &format!("user_{i}"),
                        "concurrent_actor",
                        &format!("concurrent_action_{j}"),
                    ))
                    .await;
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    // Exactly 30 audit events carry the concurrent actor.
    let events = env
        .audit
        .query_events(query_by_actor("concurrent_actor"))
        .await?;
    assert_eq!(
        events.len(),
        30,
        "should record 30 concurrent audit events (3 tasks * 10)"
    );

    // All 5 job submissions were counted.
    let metrics = env
        .queue
        .get_queue_metrics("concurrent")
        .await
        .expect("queue exists");
    assert_eq!(metrics.total_jobs_submitted, 5);

    // The response cache holds the stored entries.
    let cache_stats = env.response_cache.get_stats().await;
    assert!(
        cache_stats.total_entries > 0,
        "response cache should hold entries"
    );

    env.audit.shutdown().await;
    Ok(())
}

/// Data-flow consistency: a loaded model is visible in the cache, inference
/// output roundtrips through the response cache, and hits are counted. Gated
/// on `INFERNO_TEST_MODEL`.
#[tokio::test]
async fn test_data_flow_consistency() -> Result<()> {
    let Some(env) = build_env(true).await? else {
        eprintln!("INFERNO_TEST_MODEL not set; skipping data-flow consistency test");
        return Ok(());
    };

    let model = env.models[0].clone();

    // A loaded model appears in the cache's active set.
    let cached = env.model_cache.get_model(&model.name).await?;
    let stats = env.model_cache.get_stats().await;
    assert!(
        stats.active_models.contains(&model.name),
        "loaded model should appear in active_models"
    );

    // Inference produces output.
    let input = "What is the meaning of life?";
    let params = InferenceParams::default();
    let result = cached.backend.infer(input, &params).await?;
    assert!(!result.is_empty());

    // Response-cache roundtrip is byte-consistent.
    let key = CacheKey::new(input, &model.name, "default", &HashAlgorithm::Sha256);
    env.response_cache
        .put(&key, result.clone().into_bytes(), make_meta())
        .await?;
    assert_eq!(
        env.response_cache.get(&key).await,
        Some(result.into_bytes())
    );

    // A subsequent get is a counted hit.
    let hits_before = env.response_cache.get_stats().await.cache_hits;
    let _ = env.response_cache.get(&key).await;
    let hits_after = env.response_cache.get_stats().await.cache_hits;
    assert!(
        hits_after > hits_before,
        "a repeat read should register a hit"
    );

    // Audit the consistency check and read exactly that event back.
    env.audit
        .log_event(make_event(
            "consistency",
            "consistency_actor",
            "consistency_check",
        ))
        .await?;
    let events = env
        .audit
        .query_events(query_by_actor("consistency_actor"))
        .await?;
    assert_eq!(events.len(), 1, "should find the consistency check event");
    assert_eq!(events[0].action, "consistency_check");

    env.audit.shutdown().await;
    Ok(())
}
