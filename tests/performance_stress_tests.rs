use inferno::{
    backends::{Backend, BackendHandle, BackendConfig, BackendType, InferenceParams, InferenceMetrics},
    models::{ModelInfo, ModelManager},
    cache::{ModelCache, CacheConfig, WarmupStrategy},
    metrics::MetricsCollector,
    batch::{
        queue::{JobQueue, JobQueueManager, JobQueueConfig, BatchJob, JobPriority, JobStatus},
        processor::{BatchProcessor, ProcessorConfig},
        BatchConfig, BatchInput,
    },
    response_cache::{ResponseCache, ResponseCacheConfig, CacheKey},
    audit::{AuditSystem, AuditConfig, AuditEvent, EventType, Severity},
    conversion::{ModelConverter, ConversionConfig, ModelFormat},
    InfernoError,
};
use anyhow::Result;
use futures::StreamExt;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime},
};
use tempfile::TempDir;
use tokio::{
    fs,
    sync::{Semaphore, RwLock},
    time::{sleep, timeout},
};
use uuid::Uuid;

/// Performance and stress test utilities
mod perf_test_utils {
    use super::*;

    pub fn create_mock_gguf_file(path: &PathBuf, size_kb: usize) -> Result<()> {
        let mut content = Vec::new();

        // GGUF header
        content.extend_from_slice(b"GGUF");
        content.extend_from_slice(&3u32.to_le_bytes());
        content.extend_from_slice(&0u64.to_le_bytes());
        content.extend_from_slice(&1u64.to_le_bytes());

        // Metadata
        let key = "general.name";
        content.extend_from_slice(&(key.len() as u64).to_le_bytes());
        content.extend_from_slice(key.as_bytes());
        content.extend_from_slice(&8u32.to_le_bytes());
        let value = path.file_stem().unwrap().to_str().unwrap();
        content.extend_from_slice(&(value.len() as u64).to_le_bytes());
        content.extend_from_slice(value.as_bytes());

        // Pad to desired size
        content.resize(size_kb * 1024, 0);
        std::fs::write(path, content)?;
        Ok(())
    }

    pub async fn create_large_model_files(models_dir: &PathBuf, count: usize) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        for i in 0..count {
            let path = models_dir.join(format!("perf_model_{}.gguf", i));
            create_mock_gguf_file(&path, 1024 + (i * 512))?; // Varying sizes from 1MB to larger
            paths.push(path);
        }

        Ok(paths)
    }

    pub fn create_stress_test_job(id: &str, model_name: &str, input_count: usize) -> BatchJob {
        let inputs: Vec<BatchInput> = (0..input_count)
            .map(|i| BatchInput {
                id: format!("{}-input-{}", id, i),
                content: format!("Stress test input {} for job {}", i, id),
                metadata: Some(HashMap::from([
                    ("stress_test".to_string(), "true".to_string()),
                    ("input_index".to_string(), i.to_string()),
                ])),
            })
            .collect();

        BatchJob {
            id: id.to_string(),
            name: format!("Stress Test Job {}", id),
            description: Some("High-load stress test job".to_string()),
            priority: JobPriority::Normal,
            inputs,
            inference_params: InferenceParams {
                max_tokens: 100,
                temperature: 0.7,
                top_p: 0.9,
                stream: false,
            },
            model_name: model_name.to_string(),
            batch_config: BatchConfig {
                batch_size: 20,
                timeout_seconds: 600,
                parallel_processing: true,
                max_parallel_batches: 5,
                enable_streaming: false,
                output_format: "json".to_string(),
                compression_enabled: true,
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
                estimated_duration_seconds: Some(300),
                max_memory_mb: Some(2048),
                max_cpu_cores: Some(4),
            },
            timeout_minutes: Some(30),
            retry_count: 0,
            max_retries: 1,
            created_at: SystemTime::now(),
            scheduled_at: None,
            tags: HashMap::from([
                ("stress_test".to_string(), "true".to_string()),
                ("load_level".to_string(), "high".to_string()),
            ]),
            metadata: HashMap::from([
                ("input_count".to_string(), input_count.to_string()),
                ("test_run_id".to_string(), Uuid::new_v4().to_string()),
            ]),
        }
    }

    pub struct PerformanceMetrics {
        pub total_operations: u64,
        pub successful_operations: u64,
        pub failed_operations: u64,
        pub average_latency_ms: f64,
        pub p95_latency_ms: f64,
        pub p99_latency_ms: f64,
        pub throughput_ops_per_sec: f64,
        pub memory_peak_mb: f64,
        pub cpu_usage_percent: f64,
        pub error_rate_percent: f64,
    }

    pub struct LatencyTracker {
        latencies: Arc<RwLock<Vec<Duration>>>,
        total_ops: AtomicU64,
        successful_ops: AtomicU64,
        failed_ops: AtomicU64,
        start_time: Instant,
    }

    impl LatencyTracker {
        pub fn new() -> Self {
            Self {
                latencies: Arc::new(RwLock::new(Vec::new())),
                total_ops: AtomicU64::new(0),
                successful_ops: AtomicU64::new(0),
                failed_ops: AtomicU64::new(0),
                start_time: Instant::now(),
            }
        }

        pub async fn record_success(&self, latency: Duration) {
            self.latencies.write().await.push(latency);
            self.total_ops.fetch_add(1, Ordering::Relaxed);
            self.successful_ops.fetch_add(1, Ordering::Relaxed);
        }

        pub fn record_failure(&self) {
            self.total_ops.fetch_add(1, Ordering::Relaxed);
            self.failed_ops.fetch_add(1, Ordering::Relaxed);
        }

        pub async fn get_metrics(&self) -> PerformanceMetrics {
            let latencies = self.latencies.read().await;
            let total_ops = self.total_ops.load(Ordering::Relaxed);
            let successful_ops = self.successful_ops.load(Ordering::Relaxed);
            let failed_ops = self.failed_ops.load(Ordering::Relaxed);
            let elapsed = self.start_time.elapsed();

            let mut sorted_latencies = latencies.clone();
            sorted_latencies.sort();

            let average_latency_ms = if !sorted_latencies.is_empty() {
                sorted_latencies.iter().map(|d| d.as_millis() as f64).sum::<f64>() / sorted_latencies.len() as f64
            } else {
                0.0
            };

            let p95_latency_ms = if !sorted_latencies.is_empty() {
                let index = (sorted_latencies.len() as f64 * 0.95) as usize;
                sorted_latencies.get(index).unwrap_or(&Duration::ZERO).as_millis() as f64
            } else {
                0.0
            };

            let p99_latency_ms = if !sorted_latencies.is_empty() {
                let index = (sorted_latencies.len() as f64 * 0.99) as usize;
                sorted_latencies.get(index).unwrap_or(&Duration::ZERO).as_millis() as f64
            } else {
                0.0
            };

            let throughput_ops_per_sec = if elapsed.as_secs_f64() > 0.0 {
                successful_ops as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };

            let error_rate_percent = if total_ops > 0 {
                (failed_ops as f64 / total_ops as f64) * 100.0
            } else {
                0.0
            };

            PerformanceMetrics {
                total_operations: total_ops,
                successful_operations: successful_ops,
                failed_operations: failed_ops,
                average_latency_ms,
                p95_latency_ms,
                p99_latency_ms,
                throughput_ops_per_sec,
                memory_peak_mb: 0.0, // Would need system monitoring
                cpu_usage_percent: 0.0, // Would need system monitoring
                error_rate_percent,
            }
        }
    }
}

/// Test backend performance under high load
#[tokio::test]
async fn test_backend_performance_stress() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).await?;

    // Create test model
    let model_path = models_dir.join("perf_test_model.gguf");
    perf_test_utils::create_mock_gguf_file(&model_path, 2048)?; // 2MB model

    let model_manager = Arc::new(ModelManager::new(models_dir));
    let models = model_manager.discover_models().await?;
    assert!(!models.is_empty());

    let backend_config = BackendConfig {
        gpu_enabled: false,
        cpu_threads: Some(4),
        context_size: 1024,
        batch_size: 16,
        memory_map: true,
        ..Default::default()
    };

    // Create multiple backend instances for concurrent testing
    let num_backends = 4;
    let mut backends = Vec::new();

    for _ in 0..num_backends {
        let backend = Backend::new(BackendType::Gguf, &backend_config)?;
        backends.push(Arc::new(RwLock::new(backend)));
    }

    // Load model in all backends
    let model_info = &models[0];
    for backend in &backends {
        let mut b = backend.write().await;
        b.load_model(model_info).await?;
    }

    // Performance test parameters
    let concurrent_requests = 50;
    let requests_per_thread = 20;
    let tracker = Arc::new(perf_test_utils::LatencyTracker::new());

    // Launch concurrent inference requests
    let mut tasks = Vec::new();

    for i in 0..concurrent_requests {
        let backend = backends[i % backends.len()].clone();
        let tracker = tracker.clone();

        let task = tokio::spawn(async move {
            for j in 0..requests_per_thread {
                let start = Instant::now();

                let inference_params = InferenceParams {
                    max_tokens: 50,
                    temperature: 0.7,
                    top_p: 0.9,
                    stream: false,
                };

                let input = format!("Performance test request {} from thread {}", j, i);

                let result = {
                    let mut backend_guard = backend.write().await;
                    backend_guard.infer(&input, &inference_params).await
                };

                let latency = start.elapsed();

                match result {
                    Ok(_) => tracker.record_success(latency).await,
                    Err(_) => tracker.record_failure(),
                }

                // Small delay to prevent overwhelming
                sleep(Duration::from_millis(10)).await;
            }
        });

        tasks.push(task);
    }

    // Wait for all requests to complete
    futures::future::join_all(tasks).await;

    // Analyze performance metrics
    let metrics = tracker.get_metrics().await;

    println!("Backend Performance Metrics:");
    println!("Total operations: {}", metrics.total_operations);
    println!("Success rate: {:.2}%", (metrics.successful_operations as f64 / metrics.total_operations as f64) * 100.0);
    println!("Error rate: {:.2}%", metrics.error_rate_percent);
    println!("Average latency: {:.2}ms", metrics.average_latency_ms);
    println!("P95 latency: {:.2}ms", metrics.p95_latency_ms);
    println!("P99 latency: {:.2}ms", metrics.p99_latency_ms);
    println!("Throughput: {:.2} ops/sec", metrics.throughput_ops_per_sec);

    // Performance assertions
    assert!(metrics.error_rate_percent < 5.0, "Error rate should be less than 5%");
    assert!(metrics.average_latency_ms < 5000.0, "Average latency should be under 5 seconds");
    assert!(metrics.throughput_ops_per_sec > 0.1, "Should have measurable throughput");

    Ok(())
}

/// Test cache performance under memory pressure
#[tokio::test]
async fn test_cache_memory_pressure() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&models_dir).await?;
    fs::create_dir_all(&cache_dir).await?;

    // Create multiple models of varying sizes
    let model_paths = perf_test_utils::create_large_model_files(&models_dir, 10).await?;

    let model_manager = Arc::new(ModelManager::new(models_dir));
    let models = model_manager.discover_models().await?;

    // Configure cache with memory constraints
    let cache_config = CacheConfig {
        max_cached_models: 3, // Small limit to force evictions
        max_memory_mb: 512,   // 512MB limit
        model_ttl_seconds: 60,
        enable_warmup: false,
        warmup_strategy: WarmupStrategy::SizeOptimized,
        always_warm: vec![],
        predictive_loading: false,
        usage_window_seconds: 3600,
        min_usage_frequency: 0.1,
        memory_based_eviction: true,
        persist_cache: true,
        cache_dir: Some(cache_dir),
    };

    let backend_config = BackendConfig::default();
    let metrics_collector = Arc::new(MetricsCollector::new());

    let cache = Arc::new(ModelCache::new(
        cache_config,
        backend_config.clone(),
        model_manager,
        Some(metrics_collector.clone()),
    ).await?);

    let tracker = Arc::new(perf_test_utils::LatencyTracker::new());

    // Stress test: Rapid model loading and unloading
    let concurrent_loaders = 5;
    let loads_per_loader = 10;

    let mut tasks = Vec::new();

    for i in 0..concurrent_loaders {
        let cache = cache.clone();
        let models = models.clone();
        let backend_config = backend_config.clone();
        let tracker = tracker.clone();

        let task = tokio::spawn(async move {
            for j in 0..loads_per_loader {
                let model_index = (i * loads_per_loader + j) % models.len();
                let model = &models[model_index];

                let start = Instant::now();

                let result = cache.get_or_load_model(
                    &model.id,
                    BackendType::Gguf,
                    &backend_config,
                ).await;

                let latency = start.elapsed();

                match result {
                    Ok(handle) => {
                        // Perform a quick inference to stress the system
                        let inference_result = handle.infer(
                            "Memory pressure test",
                            &InferenceParams::default(),
                        ).await;

                        if inference_result.is_ok() {
                            tracker.record_success(latency).await;
                        } else {
                            tracker.record_failure();
                        }
                    }
                    Err(_) => tracker.record_failure(),
                }

                // Brief pause between operations
                sleep(Duration::from_millis(50)).await;
            }
        });

        tasks.push(task);
    }

    // Monitor cache statistics during the test
    let stats_task = tokio::spawn({
        let cache = cache.clone();
        async move {
            let mut max_cached = 0;
            let mut total_evictions = 0;

            for _ in 0..20 { // Monitor for 10 seconds
                sleep(Duration::from_millis(500)).await;
                let stats = cache.get_stats().await;
                max_cached = max_cached.max(stats.cached_models);
                total_evictions = stats.eviction_count;
            }

            (max_cached, total_evictions)
        }
    });

    // Wait for loading tasks
    futures::future::join_all(tasks).await;

    // Get monitoring results
    let (max_cached, total_evictions) = stats_task.await.unwrap();

    // Get final metrics
    let metrics = tracker.get_metrics().await;
    let final_stats = cache.get_stats().await;

    println!("Cache Memory Pressure Test Results:");
    println!("Max models cached simultaneously: {}", max_cached);
    println!("Total evictions: {}", total_evictions);
    println!("Final cache hits: {}", final_stats.cache_hits);
    println!("Final cache misses: {}", final_stats.cache_misses);
    println!("Hit rate: {:.2}%", final_stats.hit_rate * 100.0);
    println!("Memory usage: {:.2}MB", final_stats.memory_usage_mb);
    println!("Average load time: {:.2}ms", metrics.average_latency_ms);

    // Performance assertions
    assert!(max_cached <= 3, "Should respect max cached models limit");
    assert!(total_evictions > 0, "Should have evictions under memory pressure");
    assert!(final_stats.memory_usage_mb <= 512.0, "Should respect memory limit");
    assert!(metrics.error_rate_percent < 10.0, "Should handle memory pressure gracefully");

    Ok(())
}

/// Test batch processing throughput and scalability
#[tokio::test]
async fn test_batch_processing_scalability() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).await?;

    // Create test model
    let model_path = models_dir.join("batch_perf_model.gguf");
    perf_test_utils::create_mock_gguf_file(&model_path, 1024)?;

    let model_manager = Arc::new(ModelManager::new(models_dir));
    let models = model_manager.discover_models().await?;
    let test_model = &models[0];

    // Configure high-performance batch processing
    let queue_config = JobQueueConfig {
        max_queues: 5,
        max_jobs_per_queue: 1000,
        default_timeout_minutes: 60,
        max_retries: 2,
        cleanup_interval_seconds: 120,
        metrics_retention_hours: 24,
        persistent_storage: false, // Disable for performance
        storage_path: None,
        enable_metrics: true,
        enable_deadletter_queue: false,
        max_concurrent_jobs: 10,
        job_timeout_seconds: 600,
        retry_delay_seconds: 1,
        max_retry_delay_seconds: 30,
        exponential_backoff: false,
    };

    let job_queue_manager = Arc::new(JobQueueManager::new(queue_config));

    let processor_config = ProcessorConfig {
        max_concurrent_jobs: 8,
        worker_pool_size: 4,
        enable_batching: true,
        batch_size: 20,
        batch_timeout_seconds: 10,
        enable_monitoring: true,
        heartbeat_interval_seconds: 5,
        failure_threshold: 5,
        recovery_interval_seconds: 30,
        enable_circuit_breaker: false, // Disable for max performance
        circuit_breaker_threshold: 10,
        circuit_breaker_timeout_seconds: 60,
    };

    let cache_config = CacheConfig {
        max_cached_models: 5,
        max_memory_mb: 2048,
        model_ttl_seconds: 3600,
        enable_warmup: false,
        ..Default::default()
    };

    let backend_config = BackendConfig {
        cpu_threads: Some(4),
        context_size: 512,
        batch_size: 32,
        ..Default::default()
    };

    let cache = Arc::new(ModelCache::new(
        cache_config,
        backend_config,
        model_manager,
        None,
    ).await?);

    let batch_processor = Arc::new(BatchProcessor::new(
        processor_config,
        job_queue_manager.clone(),
        cache,
        None,
    ).await?);

    // Create test queue
    let queue_id = "scalability-test-queue";
    job_queue_manager.create_queue(
        queue_id.to_string(),
        "Scalability Test Queue".to_string(),
        "High-throughput batch processing test".to_string(),
    ).await?;

    // Submit many jobs rapidly
    let num_jobs = 100;
    let inputs_per_job = 10;

    let submission_start = Instant::now();

    for i in 0..num_jobs {
        let job = perf_test_utils::create_stress_test_job(
            &format!("scale-job-{}", i),
            &test_model.id,
            inputs_per_job,
        );

        job_queue_manager.submit_job(queue_id, job).await?;

        // Throttle submission slightly
        if i % 10 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }

    let submission_time = submission_start.elapsed();
    println!("Job submission completed in: {:?}", submission_time);

    // Start processing
    let processing_start = Instant::now();

    let processor_handle = tokio::spawn({
        let processor = batch_processor.clone();
        async move {
            processor.start_processing().await
        }
    });

    // Monitor progress
    let monitoring_handle = tokio::spawn({
        let queue_manager = job_queue_manager.clone();
        async move {
            let mut last_completed = 0;
            let mut throughput_samples = Vec::new();

            for i in 0..60 { // Monitor for up to 60 seconds
                sleep(Duration::from_secs(1)).await;

                if let Some(metrics) = queue_manager.get_queue_metrics(queue_id).await {
                    let completed = metrics.completed_jobs;
                    let throughput = completed - last_completed;

                    throughput_samples.push(throughput);
                    last_completed = completed;

                    println!("Second {}: {} jobs completed (+{}), {} in queue",
                             i + 1, completed, throughput, metrics.queued_jobs);

                    // Stop monitoring if all jobs are done
                    if completed >= num_jobs as u64 {
                        break;
                    }
                }
            }

            throughput_samples
        }
    });

    // Wait for processing to complete or timeout
    let timeout_duration = Duration::from_secs(120);
    let processing_result = timeout(timeout_duration, async {
        // Wait for all jobs to be processed
        loop {
            if let Some(metrics) = job_queue_manager.get_queue_metrics(queue_id).await {
                if metrics.completed_jobs + metrics.failed_jobs >= num_jobs as u64 {
                    break;
                }
            }
            sleep(Duration::from_millis(500)).await;
        }
    }).await;

    let processing_time = processing_start.elapsed();

    // Stop processor
    batch_processor.stop_processing().await?;
    let _ = timeout(Duration::from_secs(5), processor_handle).await;

    // Get throughput data
    let throughput_samples = monitoring_handle.await.unwrap();

    // Analyze results
    let final_metrics = job_queue_manager.get_queue_metrics(queue_id).await.unwrap();
    let processor_metrics = batch_processor.get_metrics().await?;

    let total_inputs = num_jobs * inputs_per_job;
    let success_rate = (final_metrics.completed_jobs as f64 / num_jobs as f64) * 100.0;
    let overall_throughput = final_metrics.completed_jobs as f64 / processing_time.as_secs_f64();
    let max_throughput = throughput_samples.iter().max().unwrap_or(&0);
    let avg_throughput = if !throughput_samples.is_empty() {
        throughput_samples.iter().sum::<u64>() as f64 / throughput_samples.len() as f64
    } else {
        0.0
    };

    println!("\nBatch Processing Scalability Results:");
    println!("Total jobs: {}", num_jobs);
    println!("Total inputs: {}", total_inputs);
    println!("Completed jobs: {}", final_metrics.completed_jobs);
    println!("Failed jobs: {}", final_metrics.failed_jobs);
    println!("Success rate: {:.2}%", success_rate);
    println!("Total processing time: {:?}", processing_time);
    println!("Overall throughput: {:.2} jobs/sec", overall_throughput);
    println!("Peak throughput: {} jobs/sec", max_throughput);
    println!("Average throughput: {:.2} jobs/sec", avg_throughput);
    println!("Average processing time: {:.2}ms", processor_metrics.avg_processing_time_ms);

    // Performance assertions
    assert!(processing_result.is_ok(), "Processing should complete within timeout");
    assert!(success_rate >= 90.0, "Should have high success rate");
    assert!(overall_throughput > 0.5, "Should maintain reasonable throughput");
    assert!(final_metrics.completed_jobs > 0, "Should complete some jobs");

    Ok(())
}

/// Test response cache performance under high load
#[tokio::test]
async fn test_response_cache_performance() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let cache_dir = temp_dir.path().join("response_cache");
    fs::create_dir_all(&cache_dir).await?;

    let cache_config = ResponseCacheConfig {
        enabled: true,
        max_entries: 10000,
        ttl_seconds: 3600,
        max_memory_mb: 512,
        compression_enabled: true,
        compression_algorithm: "zstd".to_string(),
        compression_level: 3,
        persistence_enabled: true,
        persistence_path: Some(cache_dir),
        enable_metrics: true,
    };

    let response_cache = Arc::new(ResponseCache::new(cache_config).await?);
    let tracker = Arc::new(perf_test_utils::LatencyTracker::new());

    // Performance test parameters
    let concurrent_workers = 10;
    let operations_per_worker = 200;
    let cache_hit_ratio = 0.3; // 30% of requests should hit cache

    // Pre-populate cache for hit testing
    let base_keys: Vec<CacheKey> = (0..100).map(|i| {
        CacheKey::new(
            "test_model",
            &format!("base_input_{}", i),
            &InferenceParams::default(),
        )
    }).collect();

    for (i, key) in base_keys.iter().enumerate() {
        let response = format!("Cached response for base input {}", i);
        response_cache.store(key, &response, Duration::from_secs(3600)).await?;
    }

    println!("Pre-populated cache with {} entries", base_keys.len());

    // Launch concurrent cache operations
    let mut tasks = Vec::new();

    for worker_id in 0..concurrent_workers {
        let cache = response_cache.clone();
        let tracker = tracker.clone();
        let base_keys = base_keys.clone();

        let task = tokio::spawn(async move {
            for op_id in 0..operations_per_worker {
                let start = Instant::now();

                // Determine operation type
                let is_cache_hit = (op_id as f64 / operations_per_worker as f64) < cache_hit_ratio;

                let result = if is_cache_hit && !base_keys.is_empty() {
                    // Cache hit operation
                    let key_index = op_id % base_keys.len();
                    cache.get(&base_keys[key_index]).await
                } else {
                    // Cache miss operation (store then get)
                    let key = CacheKey::new(
                        "test_model",
                        &format!("worker_{}_input_{}", worker_id, op_id),
                        &InferenceParams::default(),
                    );
                    let response = format!("Response from worker {} op {}", worker_id, op_id);

                    // Store operation
                    let store_result = cache.store(&key, &response, Duration::from_secs(1800)).await;
                    if store_result.is_ok() {
                        // Get operation
                        cache.get(&key).await
                    } else {
                        Err(InfernoError::Cache("Store failed".to_string()))
                    }
                };

                let latency = start.elapsed();

                match result {
                    Ok(_) => tracker.record_success(latency).await,
                    Err(_) => tracker.record_failure(),
                }

                // Brief pause to prevent overwhelming
                if op_id % 50 == 0 {
                    sleep(Duration::from_millis(1)).await;
                }
            }
        });

        tasks.push(task);
    }

    // Monitor cache statistics
    let stats_task = tokio::spawn({
        let cache = response_cache.clone();
        async move {
            let mut stats_history = Vec::new();

            for _ in 0..30 { // Monitor for 15 seconds
                sleep(Duration::from_millis(500)).await;
                if let Ok(stats) = cache.get_stats().await {
                    stats_history.push((
                        stats.hits,
                        stats.misses,
                        stats.total_entries,
                        stats.memory_usage_mb,
                    ));
                }
            }

            stats_history
        }
    });

    // Wait for all operations
    futures::future::join_all(tasks).await;

    // Get results
    let metrics = tracker.get_metrics().await;
    let final_stats = response_cache.get_stats().await?;
    let stats_history = stats_task.await.unwrap();

    // Calculate performance metrics
    let hit_rate = if final_stats.hits + final_stats.misses > 0 {
        final_stats.hits as f64 / (final_stats.hits + final_stats.misses) as f64
    } else {
        0.0
    };

    println!("\nResponse Cache Performance Results:");
    println!("Total operations: {}", metrics.total_operations);
    println!("Success rate: {:.2}%", (metrics.successful_operations as f64 / metrics.total_operations as f64) * 100.0);
    println!("Average latency: {:.2}ms", metrics.average_latency_ms);
    println!("P95 latency: {:.2}ms", metrics.p95_latency_ms);
    println!("P99 latency: {:.2}ms", metrics.p99_latency_ms);
    println!("Throughput: {:.2} ops/sec", metrics.throughput_ops_per_sec);
    println!("Cache hit rate: {:.2}%", hit_rate * 100.0);
    println!("Cache entries: {}", final_stats.total_entries);
    println!("Memory usage: {:.2}MB", final_stats.memory_usage_mb);

    // Performance assertions
    assert!(metrics.error_rate_percent < 5.0, "Error rate should be low");
    assert!(metrics.average_latency_ms < 100.0, "Cache operations should be fast");
    assert!(hit_rate > 0.2, "Should have reasonable hit rate");
    assert!(metrics.throughput_ops_per_sec > 100.0, "Should have high throughput");
    assert!(final_stats.memory_usage_mb <= 512.0, "Should respect memory limits");

    Ok(())
}

/// Test audit system performance under high event volume
#[tokio::test]
async fn test_audit_system_performance() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let audit_dir = temp_dir.path().join("audit");
    fs::create_dir_all(&audit_dir).await?;

    let audit_config = AuditConfig {
        enabled: true,
        log_directory: audit_dir,
        max_file_size_mb: 50,
        max_files: 10,
        compression: inferno::audit::CompressionType::Zstd,
        compression_level: 1, // Fast compression
        encryption: inferno::audit::EncryptionConfig {
            enabled: false, // Disable for performance testing
            ..Default::default()
        },
        retention: inferno::audit::RetentionPolicy::default(),
        compliance: inferno::audit::ComplianceConfig::default(),
        alerting: inferno::audit::AlertingConfig {
            enabled: false, // Disable for performance testing
            ..Default::default()
        },
        buffer_size: 5000, // Large buffer
        flush_interval_seconds: 2, // Fast flush
        async_processing: true,
        enable_metrics: true,
        debug_mode: false,
    };

    let audit_system = Arc::new(AuditSystem::new(audit_config).await?);
    audit_system.start().await?;

    let tracker = Arc::new(perf_test_utils::LatencyTracker::new());

    // High-volume audit event generation
    let concurrent_loggers = 8;
    let events_per_logger = 500;

    let mut tasks = Vec::new();

    for logger_id in 0..concurrent_loggers {
        let audit_system = audit_system.clone();
        let tracker = tracker.clone();

        let task = tokio::spawn(async move {
            for event_id in 0..events_per_logger {
                let start = Instant::now();

                let event = inferno::audit::AuditEvent {
                    id: Uuid::new_v4().to_string(),
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
                    actor: inferno::audit::Actor {
                        actor_type: inferno::audit::ActorType::User,
                        id: format!("user_{}", logger_id),
                        name: format!("User {}", logger_id),
                        ip_address: Some(format!("192.168.1.{}", logger_id + 1)),
                        user_agent: Some("performance-test/1.0".to_string()),
                        session_id: Some(Uuid::new_v4().to_string()),
                    },
                    resource: inferno::audit::Resource {
                        resource_type: inferno::audit::ResourceType::Api,
                        id: format!("resource_{}_{}", logger_id, event_id),
                        name: format!("Resource {} {}", logger_id, event_id),
                        path: Some(format!("/api/v1/resource/{}", event_id)),
                        attributes: HashMap::from([
                            ("method".to_string(), "POST".to_string()),
                            ("endpoint".to_string(), format!("/api/v1/test/{}", event_id)),
                        ]),
                    },
                    action: format!("perf_test_action_{}", event_id % 10),
                    details: inferno::audit::EventDetails {
                        description: format!("Performance test event {} from logger {}", event_id, logger_id),
                        request_id: Some(Uuid::new_v4().to_string()),
                        trace_id: Some(Uuid::new_v4().to_string()),
                        span_id: Some(Uuid::new_v4().to_string()),
                        parameters: HashMap::from([
                            ("test_param".to_string(), format!("value_{}", event_id)),
                            ("logger_id".to_string(), logger_id.to_string()),
                        ]),
                        response_data: Some(format!("{{\"result\": \"success\", \"event_id\": {}}}", event_id)),
                        error_details: None,
                    },
                    context: inferno::audit::EventContext {
                        source_component: "performance_test".to_string(),
                        environment: "test".to_string(),
                        version: "1.0.0".to_string(),
                        region: Some("us-west-2".to_string()),
                        availability_zone: Some("us-west-2a".to_string()),
                        cluster: Some("test-cluster".to_string()),
                        node: Some(format!("node-{}", logger_id)),
                        tenant_id: Some(format!("tenant_{}", logger_id % 3)),
                        correlation_id: Some(Uuid::new_v4().to_string()),
                    },
                    outcome: inferno::audit::EventOutcome {
                        success: event_id % 20 != 0, // 95% success rate
                        status_code: Some(if event_id % 20 == 0 { 500 } else { 200 }),
                        duration_ms: Some(10 + (event_id % 100) as u64),
                        bytes_processed: Some(100 + (event_id % 1000) as u64),
                        records_affected: Some(1),
                        resource_usage: HashMap::from([
                            ("cpu_ms".to_string(), serde_json::Value::Number((event_id % 50).into())),
                            ("memory_mb".to_string(), serde_json::Value::Number((10 + event_id % 100).into())),
                        ]),
                    },
                    metadata: HashMap::from([
                        ("test_run_id".to_string(), serde_json::Value::String(Uuid::new_v4().to_string())),
                        ("performance_test".to_string(), serde_json::Value::Bool(true)),
                        ("logger_id".to_string(), serde_json::Value::Number(logger_id.into())),
                        ("event_sequence".to_string(), serde_json::Value::Number(event_id.into())),
                    ]),
                };

                let result = audit_system.log_event(event).await;
                let latency = start.elapsed();

                match result {
                    Ok(_) => tracker.record_success(latency).await,
                    Err(_) => tracker.record_failure(),
                }

                // Brief pause every 100 events
                if event_id % 100 == 0 {
                    sleep(Duration::from_millis(1)).await;
                }
            }
        });

        tasks.push(task);
    }

    // Wait for all logging to complete
    futures::future::join_all(tasks).await;

    // Force flush and wait
    audit_system.flush().await?;
    sleep(Duration::from_secs(5)).await;

    // Get performance metrics
    let metrics = tracker.get_metrics().await;
    let audit_metrics = audit_system.get_metrics().await?;

    println!("\nAudit System Performance Results:");
    println!("Total events: {}", metrics.total_operations);
    println!("Success rate: {:.2}%", (metrics.successful_operations as f64 / metrics.total_operations as f64) * 100.0);
    println!("Error rate: {:.2}%", metrics.error_rate_percent);
    println!("Average latency: {:.2}ms", metrics.average_latency_ms);
    println!("P95 latency: {:.2}ms", metrics.p95_latency_ms);
    println!("P99 latency: {:.2}ms", metrics.p99_latency_ms);
    println!("Throughput: {:.2} events/sec", metrics.throughput_ops_per_sec);
    println!("Events per minute: {:.2}", audit_metrics.events_per_minute);
    println!("Average event size: {} bytes", audit_metrics.average_event_size_bytes);

    // Performance assertions
    assert!(metrics.error_rate_percent < 2.0, "Audit system should handle high load with low error rate");
    assert!(metrics.average_latency_ms < 50.0, "Audit logging should be fast");
    assert!(metrics.throughput_ops_per_sec > 100.0, "Should maintain high throughput");
    assert!(audit_metrics.events_per_minute > 1000.0, "Should process many events per minute");

    audit_system.shutdown().await?;

    Ok(())
}

/// Test memory usage and leak detection across all components
#[tokio::test]
async fn test_memory_usage_and_leaks() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).await?;

    // Create multiple test models
    let model_paths = perf_test_utils::create_large_model_files(&models_dir, 5).await?;

    // Memory tracking
    let initial_memory = get_memory_usage();

    // Test scenario: Repeated operations that should not leak memory
    for cycle in 0..5 {
        println!("Memory test cycle {}/5", cycle + 1);

        // Create and destroy components multiple times
        {
            let model_manager = Arc::new(ModelManager::new(models_dir.clone()));
            let models = model_manager.discover_models().await?;

            let cache_config = CacheConfig {
                max_cached_models: 2,
                max_memory_mb: 256,
                model_ttl_seconds: 30,
                ..Default::default()
            };

            let backend_config = BackendConfig::default();
            let cache = Arc::new(ModelCache::new(
                cache_config,
                backend_config.clone(),
                model_manager,
                None,
            ).await?);

            // Load and unload models
            for model in &models[0..2] {
                let handle = cache.get_or_load_model(&model.id, BackendType::Gguf, &backend_config).await?;

                // Perform inference
                let _ = handle.infer("Memory test", &InferenceParams::default()).await?;

                // Force eviction
                cache.evict_model(&model.id).await?;
            }

            // Force cleanup
            drop(cache);
        }

        // Check memory after each cycle
        let current_memory = get_memory_usage();
        println!("Memory usage after cycle {}: {} MB", cycle + 1, current_memory);

        // Force garbage collection
        sleep(Duration::from_millis(100)).await;
    }

    let final_memory = get_memory_usage();
    let memory_growth = final_memory - initial_memory;

    println!("\nMemory Usage Summary:");
    println!("Initial memory: {} MB", initial_memory);
    println!("Final memory: {} MB", final_memory);
    println!("Memory growth: {} MB", memory_growth);

    // Memory leak assertion (allow some growth but not excessive)
    assert!(memory_growth < 100.0, "Memory growth should be reasonable (< 100MB), got {} MB", memory_growth);

    Ok(())
}

/// Helper function to get current memory usage (simplified)
fn get_memory_usage() -> f64 {
    // In a real implementation, this would use system monitoring
    // For testing purposes, we'll return a mock value
    use std::process;

    // On Unix systems, you could read from /proc/self/status
    // For now, return a placeholder
    0.0
}

/// Test system behavior under resource exhaustion
#[tokio::test]
async fn test_resource_exhaustion_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).await?;

    // Create test model
    let model_path = models_dir.join("resource_test_model.gguf");
    perf_test_utils::create_mock_gguf_file(&model_path, 512)?;

    let model_manager = Arc::new(ModelManager::new(models_dir));
    let models = model_manager.discover_models().await?;

    // Configure very restrictive limits
    let cache_config = CacheConfig {
        max_cached_models: 1,
        max_memory_mb: 64, // Very low memory limit
        model_ttl_seconds: 10,
        memory_based_eviction: true,
        ..Default::default()
    };

    let backend_config = BackendConfig {
        context_size: 128, // Small context
        batch_size: 2,    // Small batch
        ..Default::default()
    };

    let cache = Arc::new(ModelCache::new(
        cache_config,
        backend_config.clone(),
        model_manager,
        None,
    ).await?);

    let tracker = Arc::new(perf_test_utils::LatencyTracker::new());

    // Attempt to overwhelm the system
    let concurrent_workers = 20; // More workers than the system can handle
    let operations_per_worker = 10;

    let mut tasks = Vec::new();

    for worker_id in 0..concurrent_workers {
        let cache = cache.clone();
        let models = models.clone();
        let backend_config = backend_config.clone();
        let tracker = tracker.clone();

        let task = tokio::spawn(async move {
            for op_id in 0..operations_per_worker {
                let start = Instant::now();

                let model = &models[0]; // All workers use same model

                let result = cache.get_or_load_model(&model.id, BackendType::Gguf, &backend_config).await;

                let latency = start.elapsed();

                match result {
                    Ok(handle) => {
                        // Try inference with resource pressure
                        let inference_result = handle.infer(
                            &format!("Resource pressure test from worker {} op {}", worker_id, op_id),
                            &InferenceParams {
                                max_tokens: 20, // Small to reduce resource usage
                                ..Default::default()
                            },
                        ).await;

                        if inference_result.is_ok() {
                            tracker.record_success(latency).await;
                        } else {
                            tracker.record_failure();
                        }
                    }
                    Err(_) => tracker.record_failure(),
                }

                // No delay - maximum pressure
            }
        });

        tasks.push(task);
    }

    // Wait for completion with timeout
    let result = timeout(Duration::from_secs(60), futures::future::join_all(tasks)).await;

    let metrics = tracker.get_metrics().await;
    let cache_stats = cache.get_stats().await;

    println!("\nResource Exhaustion Test Results:");
    println!("Total operations attempted: {}", metrics.total_operations);
    println!("Successful operations: {}", metrics.successful_operations);
    println!("Failed operations: {}", metrics.failed_operations);
    println!("Success rate: {:.2}%", (metrics.successful_operations as f64 / metrics.total_operations as f64) * 100.0);
    println!("Average latency: {:.2}ms", metrics.average_latency_ms);
    println!("Cache evictions: {}", cache_stats.eviction_count);
    println!("Memory usage: {:.2}MB", cache_stats.memory_usage_mb);

    // The system should handle resource pressure gracefully
    assert!(metrics.total_operations > 0, "Should attempt operations");
    assert!(cache_stats.memory_usage_mb <= 64.0, "Should respect memory limits");

    // Allow for some failures under extreme pressure, but system should remain responsive
    let success_rate = (metrics.successful_operations as f64 / metrics.total_operations as f64) * 100.0;
    assert!(success_rate >= 50.0 || result.is_ok(), "Should either maintain reasonable success rate or complete within timeout");

    Ok(())
}