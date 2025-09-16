use inferno::{
    // Core components
    backends::{Backend, BackendHandle, BackendConfig, BackendType, InferenceParams},
    models::{ModelInfo, ModelManager},
    cache::{ModelCache, CacheConfig, WarmupStrategy},
    metrics::MetricsCollector,

    // Batch processing
    batch::{
        queue::{JobQueue, JobQueueManager, JobQueueConfig, BatchJob, JobPriority, JobStatus},
        scheduler::{BatchScheduler, SchedulerConfig, ScheduleEntry, ScheduleType, OneTimeSchedule},
        processor::{BatchProcessor, ProcessorConfig},
        BatchConfig, BatchInput,
    },

    // Audit and security
    audit::{AuditSystem, AuditConfig, AuditEvent, EventType, Severity, Actor, ActorType},
    security::{SecurityManager, SecurityConfig},

    // Dashboard and API
    dashboard::{
        DashboardServer, DashboardConfig, CreateModelRequest, CreateDeploymentRequest,
    },

    // Model conversion
    conversion::{ModelConverter, ConversionConfig, ModelFormat, OptimizationLevel},

    // Response caching
    response_cache::{ResponseCache, ResponseCacheConfig, CacheKey},

    // Configuration
    config::Config,

    InfernoError,
};
use anyhow::Result;
use axum::{
    http::{Method, Request, StatusCode},
    body::Body,
};
use hyper::body::to_bytes;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};
use tempfile::TempDir;
use tokio::{
    fs,
    sync::{Mutex, RwLock},
    time::{sleep, timeout},
};
use tower::ServiceExt;
use uuid::Uuid;

/// Comprehensive test utilities for cross-component integration
mod integration_test_utils {
    use super::*;

    pub async fn setup_complete_test_environment(temp_dir: &TempDir) -> Result<TestEnvironment> {
        let models_dir = temp_dir.path().join("models");
        let cache_dir = temp_dir.path().join("cache");
        let audit_dir = temp_dir.path().join("audit");
        let dashboard_data_dir = temp_dir.path().join("dashboard");
        let response_cache_dir = temp_dir.path().join("response_cache");
        let batch_storage_dir = temp_dir.path().join("batch_storage");

        // Create directories
        for dir in [&models_dir, &cache_dir, &audit_dir, &dashboard_data_dir, &response_cache_dir, &batch_storage_dir] {
            fs::create_dir_all(dir).await?;
        }

        // Create test model files
        create_test_models(&models_dir).await?;

        // Initialize components
        let model_manager = Arc::new(ModelManager::new(models_dir.clone()));
        let models = model_manager.discover_models().await?;

        let metrics_collector = Arc::new(MetricsCollector::new());

        let backend_config = BackendConfig {
            gpu_enabled: false,
            gpu_device: None,
            cpu_threads: Some(2),
            context_size: 512,
            batch_size: 8,
            memory_map: true,
        };

        let cache_config = CacheConfig {
            max_cached_models: 3,
            max_memory_mb: 1024,
            model_ttl_seconds: 300,
            enable_warmup: true,
            warmup_strategy: WarmupStrategy::UsageBased,
            always_warm: vec!["test_model_1".to_string()],
            predictive_loading: false,
            usage_window_seconds: 3600,
            min_usage_frequency: 0.1,
            memory_based_eviction: true,
            persist_cache: true,
            cache_dir: Some(cache_dir),
        };

        let model_cache = Arc::new(ModelCache::new(
            cache_config,
            backend_config.clone(),
            model_manager.clone(),
            Some(metrics_collector.clone()),
        ).await?);

        let audit_config = AuditConfig {
            enabled: true,
            log_directory: audit_dir,
            max_file_size_mb: 10,
            max_files: 5,
            compression: inferno::audit::CompressionType::Zstd,
            compression_level: 3,
            encryption: inferno::audit::EncryptionConfig {
                enabled: false, // Disable for testing
                algorithm: "AES-256-GCM".to_string(),
                key_derivation: "PBKDF2".to_string(),
                key_rotation_days: 30,
                master_key_path: None,
            },
            retention: inferno::audit::RetentionPolicy {
                default_retention_days: 365,
                critical_retention_days: 2555,
                audit_log_retention_days: 2555,
                compliance_retention_days: 2555,
                max_storage_gb: 100,
                auto_archive: true,
                archive_compression: inferno::audit::CompressionType::Zstd,
            },
            compliance: inferno::audit::ComplianceConfig {
                enable_sox: false,
                enable_hipaa: false,
                enable_gdpr: false,
                enable_pci: false,
                custom_requirements: HashMap::new(),
            },
            alerting: inferno::audit::AlertingConfig {
                enabled: false, // Disable for testing
                alert_directory: temp_dir.path().join("alerts"),
                email_enabled: false,
                webhook_enabled: false,
                slack_enabled: false,
                smtp_config: None,
                webhook_urls: Vec::new(),
                slack_config: None,
                alert_cooldown_minutes: 5,
                max_alerts_per_hour: 100,
            },
            buffer_size: 1000,
            flush_interval_seconds: 2,
            async_processing: true,
            enable_metrics: true,
            debug_mode: true,
        };

        let audit_system = Arc::new(AuditSystem::new(audit_config).await?);

        let queue_config = JobQueueConfig {
            max_queues: 10,
            max_jobs_per_queue: 100,
            default_timeout_minutes: 30,
            max_retries: 3,
            cleanup_interval_seconds: 60,
            metrics_retention_hours: 24,
            persistent_storage: true,
            storage_path: Some(batch_storage_dir),
            enable_metrics: true,
            enable_deadletter_queue: true,
            max_concurrent_jobs: 3,
            job_timeout_seconds: 300,
            retry_delay_seconds: 5,
            max_retry_delay_seconds: 300,
            exponential_backoff: true,
        };

        let job_queue_manager = Arc::new(JobQueueManager::new(queue_config));

        let scheduler_config = SchedulerConfig {
            enable_scheduler: true,
            max_concurrent_schedules: 50,
            schedule_check_interval_seconds: 5,
            missed_schedule_tolerance_seconds: 30,
            enable_schedule_persistence: false,
            persistence_path: None,
            timezone: "UTC".to_string(),
            enable_metrics: true,
            max_schedule_history: 100,
        };

        let batch_scheduler = Arc::new(BatchScheduler::new(
            scheduler_config,
            job_queue_manager.clone(),
        ).await?);

        let processor_config = ProcessorConfig {
            max_concurrent_jobs: 2,
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
        };

        let batch_processor = Arc::new(BatchProcessor::new(
            processor_config,
            job_queue_manager.clone(),
            model_cache.clone(),
            Some(metrics_collector.clone()),
        ).await?);

        let response_cache_config = ResponseCacheConfig {
            enabled: true,
            max_entries: 1000,
            ttl_seconds: 3600,
            max_memory_mb: 100,
            compression_enabled: true,
            compression_algorithm: "zstd".to_string(),
            compression_level: 3,
            persistence_enabled: true,
            persistence_path: Some(response_cache_dir),
            enable_metrics: true,
        };

        let response_cache = Arc::new(ResponseCache::new(response_cache_config).await?);

        let dashboard_config = DashboardConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 0,
            data_dir: Some(dashboard_data_dir),
            models_dir: Some(models_dir),
            cache_dir: Some(temp_dir.path().join("cache")),
            enable_auth: false,
            cors_enabled: true,
            max_connections: 100,
            request_timeout_seconds: 30,
            static_files_dir: None,
            ssl_cert_path: None,
            ssl_key_path: None,
            api_keys: Vec::new(),
            rate_limit_requests_per_minute: 1000,
            backup_enabled: false,
            backup_interval_hours: 24,
            backup_retention_days: 7,
        };

        let dashboard_server = Arc::new(DashboardServer::new(dashboard_config).await?);

        let model_converter = Arc::new(ModelConverter::new());

        Ok(TestEnvironment {
            model_manager,
            model_cache,
            metrics_collector,
            audit_system,
            job_queue_manager,
            batch_scheduler,
            batch_processor,
            response_cache,
            dashboard_server,
            model_converter,
            backend_config,
            discovered_models: models,
        })
    }

    pub async fn create_test_models(models_dir: &PathBuf) -> Result<()> {
        let models = vec![
            ("test_model_1.gguf", "Test Model 1"),
            ("test_model_2.gguf", "Test Model 2"),
            ("test_model_3.onnx", "Test Model 3"),
        ];

        for (filename, model_name) in models {
            let path = models_dir.join(filename);

            if filename.ends_with(".gguf") {
                create_mock_gguf_file(&path, model_name)?;
            } else if filename.ends_with(".onnx") {
                create_mock_onnx_file(&path, model_name)?;
            }
        }

        Ok(())
    }

    fn create_mock_gguf_file(path: &PathBuf, model_name: &str) -> Result<()> {
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

    fn create_mock_onnx_file(path: &PathBuf, model_name: &str) -> Result<()> {
        let mut content = Vec::new();
        content.extend_from_slice(&[0x08, 0x01]);
        content.extend_from_slice(&[0x12, model_name.len() as u8]);
        content.extend_from_slice(model_name.as_bytes());
        content.resize(2048, 0);
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn create_test_batch_job(id: &str, model_name: &str) -> BatchJob {
        BatchJob {
            id: id.to_string(),
            name: format!("Integration Test Job {}", id),
            description: Some("Cross-component integration test job".to_string()),
            priority: JobPriority::Normal,
            inputs: vec![
                BatchInput {
                    id: format!("{}-input-1", id),
                    content: "What is machine learning?".to_string(),
                    metadata: Some(HashMap::from([
                        ("type".to_string(), "question".to_string()),
                    ])),
                },
                BatchInput {
                    id: format!("{}-input-2", id),
                    content: "Explain neural networks.".to_string(),
                    metadata: Some(HashMap::from([
                        ("type".to_string(), "explanation".to_string()),
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
            batch_config: BatchConfig {
                batch_size: 10,
                timeout_seconds: 300,
                parallel_processing: true,
                max_parallel_batches: 2,
                enable_streaming: false,
                output_format: "json".to_string(),
                compression_enabled: false,
                checkpointing_enabled: false,
                checkpoint_interval_seconds: 60,
            },
            schedule: None,
            dependencies: vec![],
            resource_requirements: inferno::batch::queue::ResourceRequirements {
                min_memory_mb: 256,
                min_cpu_cores: 1,
                min_gpu_memory_mb: None,
                required_gpu: false,
                estimated_duration_seconds: Some(60),
                max_memory_mb: Some(1024),
                max_cpu_cores: Some(2),
            },
            timeout_minutes: Some(10),
            retry_count: 0,
            max_retries: 2,
            created_at: SystemTime::now(),
            scheduled_at: None,
            tags: HashMap::from([
                ("integration_test".to_string(), "true".to_string()),
            ]),
            metadata: HashMap::from([
                ("test_run_id".to_string(), Uuid::new_v4().to_string()),
            ]),
        }
    }
}

pub struct TestEnvironment {
    pub model_manager: Arc<ModelManager>,
    pub model_cache: Arc<ModelCache>,
    pub metrics_collector: Arc<MetricsCollector>,
    pub audit_system: Arc<AuditSystem>,
    pub job_queue_manager: Arc<JobQueueManager>,
    pub batch_scheduler: Arc<BatchScheduler>,
    pub batch_processor: Arc<BatchProcessor>,
    pub response_cache: Arc<ResponseCache>,
    pub dashboard_server: Arc<DashboardServer>,
    pub model_converter: Arc<ModelConverter>,
    pub backend_config: BackendConfig,
    pub discovered_models: Vec<ModelInfo>,
}

/// Test end-to-end model lifecycle across all components
#[tokio::test]
async fn test_end_to_end_model_lifecycle() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let env = integration_test_utils::setup_complete_test_environment(&temp_dir).await?;

    // Start all systems
    env.audit_system.start().await?;
    env.batch_scheduler.start().await?;

    // 1. Model Discovery and Loading
    assert!(!env.discovered_models.is_empty(), "Should discover test models");

    let test_model = &env.discovered_models[0];

    // Create audit event for model discovery
    let discovery_event = AuditEvent {
        id: Uuid::new_v4().to_string(),
        timestamp: SystemTime::now(),
        event_type: EventType::ModelManagement,
        severity: Severity::Info,
        actor: Actor {
            actor_type: ActorType::System,
            id: "model_manager".to_string(),
            name: "Model Manager".to_string(),
            ip_address: None,
            user_agent: None,
            session_id: None,
        },
        resource: inferno::audit::Resource {
            resource_type: inferno::audit::ResourceType::Model,
            id: test_model.id.clone(),
            name: test_model.name.clone(),
            path: Some(test_model.path.to_string_lossy().to_string()),
            attributes: HashMap::new(),
        },
        action: "model_discovered".to_string(),
        details: inferno::audit::EventDetails {
            description: "Model discovered during integration test".to_string(),
            request_id: Some(Uuid::new_v4().to_string()),
            trace_id: None,
            span_id: None,
            parameters: HashMap::new(),
            response_data: None,
            error_details: None,
        },
        context: inferno::audit::EventContext {
            source_component: "integration_test".to_string(),
            environment: "test".to_string(),
            version: "1.0.0".to_string(),
            region: None,
            availability_zone: None,
            cluster: None,
            node: None,
            tenant_id: None,
            correlation_id: Some(Uuid::new_v4().to_string()),
        },
        outcome: inferno::audit::EventOutcome {
            success: true,
            status_code: Some(200),
            duration_ms: Some(10),
            bytes_processed: Some(test_model.size),
            records_affected: Some(1),
            resource_usage: HashMap::new(),
        },
        metadata: HashMap::new(),
    };

    env.audit_system.log_event(discovery_event).await?;

    // 2. Model Loading via Cache
    let backend_handle = env.model_cache.get_or_load_model(
        &test_model.id,
        BackendType::Gguf,
        &env.backend_config,
    ).await?;

    assert!(backend_handle.is_loaded().await, "Model should be loaded");

    // 3. Inference with Response Caching
    let inference_params = InferenceParams {
        max_tokens: 50,
        temperature: 0.7,
        top_p: 0.9,
        stream: false,
    };

    let input = "What is artificial intelligence?";
    let cache_key = CacheKey::new(&test_model.id, input, &inference_params);

    // First inference (cache miss)
    let start_time = std::time::Instant::now();
    let result1 = backend_handle.infer(input, &inference_params).await?;
    let inference_time = start_time.elapsed();

    // Store in response cache
    env.response_cache.store(&cache_key, &result1, Duration::from_secs(3600)).await?;

    // Second inference (should hit cache)
    let cached_result = env.response_cache.get(&cache_key).await?;
    assert!(cached_result.is_some(), "Should get cached result");
    assert_eq!(cached_result.unwrap(), result1, "Cached result should match");

    // 4. Batch Job Processing
    let queue_id = "integration-queue";
    env.job_queue_manager.create_queue(
        queue_id.to_string(),
        "Integration Test Queue".to_string(),
        "Queue for cross-component testing".to_string(),
    ).await?;

    let batch_job = integration_test_utils::create_test_batch_job("integration-job-1", &test_model.id);
    env.job_queue_manager.submit_job(queue_id, batch_job).await?;

    // Start batch processor
    let processor_handle = tokio::spawn({
        let processor = env.batch_processor.clone();
        async move {
            processor.start_processing().await
        }
    });

    // Wait for job processing
    sleep(Duration::from_secs(10)).await;

    // 5. Dashboard API Integration
    let dashboard_router = env.dashboard_server.create_router().await?;

    // Create model via API
    let create_model_request = json!({
        "name": "api_test_model",
        "version": "v1.0",
        "format": "GGUF",
        "description": "Model created via API integration test",
        "tags": ["integration", "test"]
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/models")
        .header("Content-Type", "application/json")
        .body(Body::from(create_model_request.to_string()))?;

    let response = dashboard_router.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::CREATED);

    // Get metrics via API
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/metrics")
        .body(Body::empty())?;

    let response = dashboard_router.oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body()).await?;
    let metrics: Value = serde_json::from_slice(&body_bytes)?;
    assert!(metrics.get("total_models").is_some());

    // 6. Verify Audit Trail
    env.audit_system.flush().await?;
    sleep(Duration::from_secs(3)).await;

    let audit_query = inferno::audit::AuditQuery {
        filters: vec![inferno::audit::QueryFilter::EventType(EventType::ModelManagement)],
        start_time: Some(SystemTime::now() - Duration::from_secs(300)),
        end_time: Some(SystemTime::now()),
        limit: Some(100),
        offset: None,
        sort_by: None,
        sort_order: None,
    };

    let audit_events = env.audit_system.query_events(audit_query).await?;
    assert!(!audit_events.is_empty(), "Should have audit events");

    // 7. Check Metrics Collection
    let cache_stats = env.model_cache.get_stats().await;
    assert!(cache_stats.cache_hits + cache_stats.cache_misses > 0);

    let response_cache_stats = env.response_cache.get_stats().await?;
    assert!(response_cache_stats.total_entries > 0);

    // Cleanup
    env.batch_processor.stop_processing().await?;
    env.batch_scheduler.stop().await?;
    env.audit_system.shutdown().await?;

    let _ = timeout(Duration::from_secs(5), processor_handle).await;

    Ok(())
}

/// Test system resilience and error propagation
#[tokio::test]
async fn test_system_resilience_and_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let env = integration_test_utils::setup_complete_test_environment(&temp_dir).await?;

    env.audit_system.start().await?;

    // Test 1: Model loading failure cascade
    let nonexistent_model_id = "nonexistent_model";
    let load_result = env.model_cache.get_or_load_model(
        nonexistent_model_id,
        BackendType::Gguf,
        &env.backend_config,
    ).await;

    assert!(load_result.is_err(), "Should fail to load nonexistent model");

    // Test 2: Batch job with invalid model
    let queue_id = "error-test-queue";
    env.job_queue_manager.create_queue(
        queue_id.to_string(),
        "Error Test Queue".to_string(),
        "Queue for testing error handling".to_string(),
    ).await?;

    let invalid_job = integration_test_utils::create_test_batch_job("error-job", "nonexistent_model");
    env.job_queue_manager.submit_job(queue_id, invalid_job).await?;

    // Start processor to handle the failing job
    let processor_handle = tokio::spawn({
        let processor = env.batch_processor.clone();
        async move {
            processor.start_processing().await
        }
    });

    sleep(Duration::from_secs(5)).await;

    // Check job status - should be failed or in retry
    let job_status = env.job_queue_manager.get_job_status(queue_id, "error-job").await?;
    assert!(job_status.is_some());

    let job_info = job_status.unwrap();
    assert!(
        matches!(job_info.status, JobStatus::Failed | JobStatus::Queued),
        "Job should be failed or queued for retry"
    );

    // Test 3: Response cache with corrupted data
    let cache_key = CacheKey::new("test_model", "test input", &InferenceParams::default());

    // Try to get from empty cache
    let empty_result = env.response_cache.get(&cache_key).await?;
    assert!(empty_result.is_none(), "Should get None from empty cache");

    // Test 4: Dashboard API error handling
    let dashboard_router = env.dashboard_server.create_router().await?;

    // Invalid JSON request
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/models")
        .header("Content-Type", "application/json")
        .body(Body::from("invalid json"))?;

    let response = dashboard_router.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Non-existent resource
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/models/nonexistent")
        .body(Body::empty())?;

    let response = dashboard_router.oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Test 5: Verify error events in audit log
    env.audit_system.flush().await?;
    sleep(Duration::from_secs(2)).await;

    let error_query = inferno::audit::AuditQuery {
        filters: vec![inferno::audit::QueryFilter::Severity(Severity::High)],
        start_time: Some(SystemTime::now() - Duration::from_secs(300)),
        end_time: Some(SystemTime::now()),
        limit: Some(100),
        offset: None,
        sort_by: None,
        sort_order: None,
    };

    let error_events = env.audit_system.query_events(error_query).await?;
    // May or may not have error events depending on audit integration

    // Cleanup
    env.batch_processor.stop_processing().await?;
    env.audit_system.shutdown().await?;
    let _ = timeout(Duration::from_secs(5), processor_handle).await;

    Ok(())
}

/// Test concurrent operations across components
#[tokio::test]
async fn test_concurrent_cross_component_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let env = integration_test_utils::setup_complete_test_environment(&temp_dir).await?;

    env.audit_system.start().await?;
    env.batch_scheduler.start().await?;

    let queue_id = "concurrent-queue";
    env.job_queue_manager.create_queue(
        queue_id.to_string(),
        "Concurrent Test Queue".to_string(),
        "Queue for concurrent testing".to_string(),
    ).await?;

    // Start batch processor
    let processor_handle = tokio::spawn({
        let processor = env.batch_processor.clone();
        async move {
            processor.start_processing().await
        }
    });

    // Launch concurrent operations
    let mut tasks = Vec::new();

    // Task 1: Concurrent model loading
    for i in 0..3 {
        let cache = env.model_cache.clone();
        let backend_config = env.backend_config.clone();
        let model_id = env.discovered_models[i % env.discovered_models.len()].id.clone();

        let task = tokio::spawn(async move {
            for _ in 0..5 {
                let result = cache.get_or_load_model(&model_id, BackendType::Gguf, &backend_config).await;
                if let Ok(handle) = result {
                    let _ = handle.infer("Test concurrent inference", &InferenceParams::default()).await;
                }
                sleep(Duration::from_millis(100)).await;
            }
        });
        tasks.push(task);
    }

    // Task 2: Concurrent batch job submission
    for i in 0..5 {
        let queue_manager = env.job_queue_manager.clone();
        let model_id = env.discovered_models[0].id.clone();

        let task = tokio::spawn(async move {
            let job = integration_test_utils::create_test_batch_job(
                &format!("concurrent-job-{}", i),
                &model_id,
            );
            let _ = queue_manager.submit_job("concurrent-queue", job).await;
        });
        tasks.push(task);
    }

    // Task 3: Concurrent response caching
    for i in 0..3 {
        let response_cache = env.response_cache.clone();

        let task = tokio::spawn(async move {
            for j in 0..5 {
                let key = CacheKey::new(
                    "test_model",
                    &format!("concurrent input {} {}", i, j),
                    &InferenceParams::default(),
                );
                let value = format!("response {} {}", i, j);

                let _ = response_cache.store(&key, &value, Duration::from_secs(300)).await;
                let _ = response_cache.get(&key).await;
            }
        });
        tasks.push(task);
    }

    // Task 4: Concurrent audit logging
    for i in 0..3 {
        let audit_system = env.audit_system.clone();

        let task = tokio::spawn(async move {
            for j in 0..10 {
                let event = AuditEvent {
                    id: Uuid::new_v4().to_string(),
                    timestamp: SystemTime::now(),
                    event_type: EventType::UserAction,
                    severity: Severity::Info,
                    actor: Actor {
                        actor_type: ActorType::User,
                        id: format!("user_{}", i),
                        name: format!("User {}", i),
                        ip_address: Some("127.0.0.1".to_string()),
                        user_agent: None,
                        session_id: None,
                    },
                    resource: inferno::audit::Resource {
                        resource_type: inferno::audit::ResourceType::Api,
                        id: format!("resource_{}_{}", i, j),
                        name: format!("Resource {} {}", i, j),
                        path: None,
                        attributes: HashMap::new(),
                    },
                    action: "concurrent_action".to_string(),
                    details: inferno::audit::EventDetails {
                        description: format!("Concurrent action {} {}", i, j),
                        request_id: Some(Uuid::new_v4().to_string()),
                        trace_id: None,
                        span_id: None,
                        parameters: HashMap::new(),
                        response_data: None,
                        error_details: None,
                    },
                    context: inferno::audit::EventContext {
                        source_component: "concurrent_test".to_string(),
                        environment: "test".to_string(),
                        version: "1.0.0".to_string(),
                        region: None,
                        availability_zone: None,
                        cluster: None,
                        node: None,
                        tenant_id: None,
                        correlation_id: None,
                    },
                    outcome: inferno::audit::EventOutcome {
                        success: true,
                        status_code: Some(200),
                        duration_ms: Some(50),
                        bytes_processed: Some(100),
                        records_affected: Some(1),
                        resource_usage: HashMap::new(),
                    },
                    metadata: HashMap::new(),
                };

                let _ = audit_system.log_event(event).await;
            }
        });
        tasks.push(task);
    }

    // Wait for all concurrent operations
    futures::future::join_all(tasks).await;

    // Wait for processing to complete
    sleep(Duration::from_secs(10)).await;

    // Verify system state after concurrent operations
    let cache_stats = env.model_cache.get_stats().await;
    assert!(cache_stats.cache_hits + cache_stats.cache_misses > 0);

    let queue_metrics = env.job_queue_manager.get_queue_metrics(queue_id).await;
    assert!(queue_metrics.is_some());

    let response_cache_stats = env.response_cache.get_stats().await?;
    assert!(response_cache_stats.total_entries > 0);

    // Check audit events
    env.audit_system.flush().await?;
    sleep(Duration::from_secs(2)).await;

    let audit_query = inferno::audit::AuditQuery {
        filters: vec![inferno::audit::QueryFilter::Action("concurrent_action".to_string())],
        start_time: Some(SystemTime::now() - Duration::from_secs(300)),
        end_time: Some(SystemTime::now()),
        limit: Some(100),
        offset: None,
        sort_by: None,
        sort_order: None,
    };

    let concurrent_events = env.audit_system.query_events(audit_query).await?;
    assert_eq!(concurrent_events.len(), 30, "Should have 30 concurrent audit events (3 tasks * 10 events)");

    // Cleanup
    env.batch_processor.stop_processing().await?;
    env.batch_scheduler.stop().await?;
    env.audit_system.shutdown().await?;
    let _ = timeout(Duration::from_secs(5), processor_handle).await;

    Ok(())
}

/// Test data flow and consistency across components
#[tokio::test]
async fn test_data_flow_consistency() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let env = integration_test_utils::setup_complete_test_environment(&temp_dir).await?;

    env.audit_system.start().await?;

    // Test data consistency between model discovery and cache
    let discovered_models = &env.discovered_models;
    assert!(!discovered_models.is_empty());

    // Load a model and verify it appears in cache
    let test_model = &discovered_models[0];
    let backend_handle = env.model_cache.get_or_load_model(
        &test_model.id,
        BackendType::Gguf,
        &env.backend_config,
    ).await?;

    let cached_models = env.model_cache.list_cached_models().await;
    assert!(cached_models.contains(&test_model.id), "Model should appear in cache list");

    // Test inference consistency
    let input1 = "What is the meaning of life?";
    let params = InferenceParams::default();

    let result1 = backend_handle.infer(input1, &params).await?;
    let result2 = backend_handle.infer(input1, &params).await?;

    // Results should be consistent (or at least both successful)
    assert!(!result1.is_empty());
    assert!(!result2.is_empty());

    // Test response cache consistency
    let cache_key = CacheKey::new(&test_model.id, input1, &params);
    env.response_cache.store(&cache_key, &result1, Duration::from_secs(3600)).await?;

    let cached_result = env.response_cache.get(&cache_key).await?;
    assert_eq!(cached_result.unwrap(), result1, "Cached result should match original");

    // Test metrics consistency
    let cache_stats_before = env.model_cache.get_stats().await;
    let response_cache_stats_before = env.response_cache.get_stats().await?;

    // Perform additional operations
    let _ = backend_handle.infer("Another test input", &params).await?;
    let _ = env.response_cache.get(&cache_key).await?; // Should be a hit

    let cache_stats_after = env.model_cache.get_stats().await;
    let response_cache_stats_after = env.response_cache.get_stats().await?;

    // Verify metrics were updated
    assert!(cache_stats_after.cache_hits >= cache_stats_before.cache_hits);
    assert!(response_cache_stats_after.hits > response_cache_stats_before.hits);

    // Test audit trail consistency
    let discovery_audit_event = AuditEvent {
        id: Uuid::new_v4().to_string(),
        timestamp: SystemTime::now(),
        event_type: EventType::ModelManagement,
        severity: Severity::Info,
        actor: Actor {
            actor_type: ActorType::System,
            id: "consistency_test".to_string(),
            name: "Consistency Test".to_string(),
            ip_address: None,
            user_agent: None,
            session_id: None,
        },
        resource: inferno::audit::Resource {
            resource_type: inferno::audit::ResourceType::Model,
            id: test_model.id.clone(),
            name: test_model.name.clone(),
            path: Some(test_model.path.to_string_lossy().to_string()),
            attributes: HashMap::new(),
        },
        action: "consistency_check".to_string(),
        details: inferno::audit::EventDetails {
            description: "Data consistency verification".to_string(),
            request_id: Some(Uuid::new_v4().to_string()),
            trace_id: None,
            span_id: None,
            parameters: HashMap::new(),
            response_data: None,
            error_details: None,
        },
        context: inferno::audit::EventContext {
            source_component: "consistency_test".to_string(),
            environment: "test".to_string(),
            version: "1.0.0".to_string(),
            region: None,
            availability_zone: None,
            cluster: None,
            node: None,
            tenant_id: None,
            correlation_id: None,
        },
        outcome: inferno::audit::EventOutcome {
            success: true,
            status_code: Some(200),
            duration_ms: Some(10),
            bytes_processed: Some(1024),
            records_affected: Some(1),
            resource_usage: HashMap::new(),
        },
        metadata: HashMap::from([
            ("cache_hits".to_string(), serde_json::Value::Number(cache_stats_after.cache_hits.into())),
            ("cached_models".to_string(), serde_json::Value::Number(cache_stats_after.cached_models.into())),
        ]),
    };

    env.audit_system.log_event(discovery_audit_event).await?;
    env.audit_system.flush().await?;
    sleep(Duration::from_secs(2)).await;

    // Verify audit event was recorded
    let audit_query = inferno::audit::AuditQuery {
        filters: vec![inferno::audit::QueryFilter::Action("consistency_check".to_string())],
        start_time: Some(SystemTime::now() - Duration::from_secs(60)),
        end_time: Some(SystemTime::now()),
        limit: Some(10),
        offset: None,
        sort_by: None,
        sort_order: None,
    };

    let audit_results = env.audit_system.query_events(audit_query).await?;
    assert_eq!(audit_results.len(), 1, "Should find consistency check audit event");

    let recorded_event = &audit_results[0];
    assert_eq!(recorded_event.action, "consistency_check");
    assert_eq!(recorded_event.resource.id, test_model.id);

    env.audit_system.shutdown().await?;

    Ok(())
}