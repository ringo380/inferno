use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use inferno::{
    advanced_cache::{AdvancedCache, CacheConfig, CompressionType},
    backends::{Backend, BackendConfig, BackendType, InferenceParams},
    config::Config,
    models::{ModelInfo, ModelManager},
};
use std::process;
use std::{path::PathBuf, sync::Arc, time::Duration};
use sysinfo::{ProcessExt, System, SystemExt};
use tempfile::tempdir;
use tokio::{fs, runtime::Runtime};

/// Memory usage tracker for benchmarks
struct MemoryTracker {
    system: System,
    pid: u32,
    initial_memory: u64,
}

impl MemoryTracker {
    fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let pid = process::id();

        let initial_memory = system
            .process(sysinfo::Pid::from(pid as usize))
            .map(|p| p.memory())
            .unwrap_or(0);

        Self {
            system,
            pid,
            initial_memory,
        }
    }

    fn current_memory_usage(&mut self) -> u64 {
        self.system
            .refresh_process(sysinfo::Pid::from(self.pid as usize));
        self.system
            .process(sysinfo::Pid::from(self.pid as usize))
            .map(|p| p.memory())
            .unwrap_or(0)
    }

    fn memory_delta(&mut self) -> i64 {
        let current = self.current_memory_usage();
        current as i64 - self.initial_memory as i64
    }
}

fn create_large_mock_model(path: &PathBuf, size_mb: usize) -> ModelInfo {
    let data = vec![0u8; size_mb * 1024 * 1024];
    let mut content = b"GGUF\x00\x00\x00\x01".to_vec();
    content.extend(data);
    std::fs::write(path, content).unwrap();

    ModelInfo {
        name: path.file_name().unwrap().to_string_lossy().to_string(),
        path: path.clone(),
        size: (size_mb * 1024 * 1024) as u64,
        modified: chrono::Utc::now(),
        backend_type: "gguf".to_string(),
        checksum: None,
    }
}

fn bench_memory_model_loading(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    // Create models of different sizes
    let model_sizes = vec![1, 5, 10, 25, 50]; // MB

    let mut group = c.benchmark_group("memory_model_loading");
    group.sample_size(10); // Fewer samples for memory-intensive tests
    group.measurement_time(Duration::from_secs(30));

    for size in model_sizes {
        let model_path = temp_dir.path().join(format!("model_{}mb.gguf", size));
        let model = create_large_mock_model(&model_path, size);
        let backend_config = BackendConfig::default();

        group.bench_with_input(
            BenchmarkId::new("model_load_memory", size),
            &size,
            |b, _| {
                b.iter_custom(|iters| {
                    let mut total_time = Duration::new(0, 0);
                    let mut tracker = MemoryTracker::new();
                    let initial_memory = tracker.current_memory_usage();

                    for _ in 0..iters {
                        let start = std::time::Instant::now();

                        rt.block_on(async {
                            let mut backend =
                                Backend::new(BackendType::Gguf, &backend_config).unwrap();
                            let _ = backend.load_model(black_box(&model)).await;
                            let _ = backend.unload_model().await;
                        });

                        total_time += start.elapsed();
                    }

                    let final_memory = tracker.current_memory_usage();
                    let memory_delta = final_memory as i64 - initial_memory as i64;

                    eprintln!(
                        "Memory delta for {}MB model: {} KB",
                        size,
                        memory_delta / 1024
                    );
                    total_time
                })
            },
        );
    }

    group.finish();
}

fn bench_memory_concurrent_models(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let model_path = temp_dir.path().join("concurrent_test.gguf");
    let model = create_large_mock_model(&model_path, 5); // 5MB model
    let backend_config = BackendConfig::default();

    let concurrent_counts = vec![1, 2, 4, 8];

    let mut group = c.benchmark_group("memory_concurrent_models");
    group.sample_size(5);
    group.measurement_time(Duration::from_secs(20));

    for count in concurrent_counts {
        group.bench_with_input(
            BenchmarkId::new("concurrent_load", count),
            &count,
            |b, &count| {
                b.iter_custom(|iters| {
                    let mut total_time = Duration::new(0, 0);

                    for _ in 0..iters {
                        let mut tracker = MemoryTracker::new();
                        let initial_memory = tracker.current_memory_usage();

                        let start = std::time::Instant::now();

                        rt.block_on(async {
                            let handles: Vec<_> = (0..count)
                                .map(|_| {
                                    let model = model.clone();
                                    let config = backend_config.clone();
                                    tokio::spawn(async move {
                                        let mut backend =
                                            Backend::new(BackendType::Gguf, &config).unwrap();
                                        let _ = backend.load_model(&model).await;
                                        tokio::time::sleep(Duration::from_millis(100)).await;
                                        let _ = backend.unload_model().await;
                                    })
                                })
                                .collect();

                            for handle in handles {
                                let _ = handle.await;
                            }
                        });

                        total_time += start.elapsed();

                        let final_memory = tracker.current_memory_usage();
                        let memory_delta = final_memory as i64 - initial_memory as i64;
                        eprintln!(
                            "Memory delta for {} concurrent models: {} KB",
                            count,
                            memory_delta / 1024
                        );
                    }

                    total_time
                })
            },
        );
    }

    group.finish();
}

fn bench_memory_cache_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let cache_config = CacheConfig {
        max_size: 100 * 1024 * 1024, // 100MB
        ttl: Duration::from_secs(3600),
        compression: CompressionType::Gzip,
        persistent: true,
        cache_dir: temp_dir.path().join("cache"),
    };

    let cache_sizes = vec![10, 50, 100, 500]; // Number of cache entries

    let mut group = c.benchmark_group("memory_cache_operations");
    group.sample_size(10);

    for size in cache_sizes {
        group.bench_with_input(BenchmarkId::new("cache_fill", size), &size, |b, &size| {
            b.iter_custom(|iters| {
                let mut total_time = Duration::new(0, 0);

                for _ in 0..iters {
                    let mut tracker = MemoryTracker::new();
                    let initial_memory = tracker.current_memory_usage();

                    let start = std::time::Instant::now();

                    rt.block_on(async {
                        let cache = AdvancedCache::new(cache_config.clone()).await.unwrap();

                        // Fill cache with data
                        for i in 0..size {
                            let key = format!("key_{}", i);
                            let value = format!("value_{}", "x".repeat(1024)); // 1KB per entry
                            cache.set(key, value).await.unwrap();
                        }

                        // Read all entries
                        for i in 0..size {
                            let key = format!("key_{}", i);
                            let _ = cache.get(&key).await;
                        }
                    });

                    total_time += start.elapsed();

                    let final_memory = tracker.current_memory_usage();
                    let memory_delta = final_memory as i64 - initial_memory as i64;
                    eprintln!(
                        "Memory delta for {} cache entries: {} KB",
                        size,
                        memory_delta / 1024
                    );
                }

                total_time
            })
        });
    }

    group.finish();
}

fn bench_memory_inference_batches(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let model_path = temp_dir.path().join("inference_test.gguf");
    let model = create_large_mock_model(&model_path, 10); // 10MB model
    let backend_config = BackendConfig::default();

    let batch_sizes = vec![1, 5, 10, 20, 50];

    let mut group = c.benchmark_group("memory_inference_batches");
    group.sample_size(5);
    group.measurement_time(Duration::from_secs(30));

    for batch_size in batch_sizes {
        group.bench_with_input(
            BenchmarkId::new("inference_batch", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter_custom(|iters| {
                    let mut total_time = Duration::new(0, 0);

                    for _ in 0..iters {
                        let mut tracker = MemoryTracker::new();
                        let initial_memory = tracker.current_memory_usage();

                        let start = std::time::Instant::now();

                        rt.block_on(async {
                            let mut backend =
                                Backend::new(BackendType::Gguf, &backend_config).unwrap();
                            backend.load_model(&model).await.unwrap();

                            let inference_params = InferenceParams {
                                max_tokens: 50,
                                temperature: 0.7,
                                top_p: 0.9,
                                stream: false,
                            };

                            // Run batch of inferences
                            let handles: Vec<_> = (0..batch_size)
                                .map(|i| {
                                    let prompt =
                                        format!("Test prompt number {} with some content", i);
                                    async move { backend.infer(&prompt, &inference_params).await }
                                })
                                .collect();

                            for handle in handles {
                                let _ = handle.await;
                            }

                            backend.unload_model().await.unwrap();
                        });

                        total_time += start.elapsed();

                        let final_memory = tracker.current_memory_usage();
                        let memory_delta = final_memory as i64 - initial_memory as i64;
                        eprintln!(
                            "Memory delta for batch size {}: {} KB",
                            batch_size,
                            memory_delta / 1024
                        );
                    }

                    total_time
                })
            },
        );
    }

    group.finish();
}

fn bench_memory_stress_test(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let mut group = c.benchmark_group("memory_stress_test");
    group.sample_size(3);
    group.measurement_time(Duration::from_secs(60));

    group.bench_function("sustained_load_unload", |b| {
        b.iter_custom(|iters| {
            let mut total_time = Duration::new(0, 0);

            for _ in 0..iters {
                let mut tracker = MemoryTracker::new();
                let initial_memory = tracker.current_memory_usage();

                let start = std::time::Instant::now();

                rt.block_on(async {
                    let backend_config = BackendConfig::default();

                    // Sustained load/unload cycles
                    for cycle in 0..20 {
                        let model_path = temp_dir
                            .path()
                            .join(format!("stress_model_{}.gguf", cycle % 5));
                        let model = create_large_mock_model(&model_path, 5); // 5MB models

                        let mut backend = Backend::new(BackendType::Gguf, &backend_config).unwrap();
                        backend.load_model(&model).await.unwrap();

                        // Simulate some work
                        tokio::time::sleep(Duration::from_millis(10)).await;

                        backend.unload_model().await.unwrap();

                        // Check memory every 5 cycles
                        if cycle % 5 == 0 {
                            let current_memory = tracker.current_memory_usage();
                            let delta = current_memory as i64 - initial_memory as i64;
                            eprintln!("Memory delta at cycle {}: {} KB", cycle, delta / 1024);
                        }
                    }
                });

                total_time += start.elapsed();
            }

            total_time
        })
    });

    group.finish();
}

criterion_group!(
    memory_benches,
    bench_memory_model_loading,
    bench_memory_concurrent_models,
    bench_memory_cache_operations,
    bench_memory_inference_batches,
    bench_memory_stress_test
);
criterion_main!(memory_benches);
