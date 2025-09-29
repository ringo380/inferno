use criterion::{black_box, criterion_group, criterion_main, Criterion};
use inferno::{
    backends::{Backend, BackendConfig, BackendType, InferenceParams},
    metrics::MetricsCollector,
    models::{ModelInfo, ModelManager},
};
use std::{path::PathBuf, sync::Arc, time::Duration};
use sysinfo::{CpuExt, System, SystemExt};
use tempfile::tempdir;
use tokio::{runtime::Runtime, time::Instant};

fn create_mock_model(path: &PathBuf, backend_type: &str) -> ModelInfo {
    let content = match backend_type {
        "gguf" => b"GGUF\x00\x00\x00\x01mock model data for profiling".to_vec(),
        "onnx" => b"mock onnx model data for profiling".to_vec(),
        _ => panic!("Unsupported backend type"),
    };
    std::fs::write(path, content).unwrap();

    ModelInfo {
        name: path.file_name().unwrap().to_string_lossy().to_string(),
        path: path.clone(),
        file_path: path.clone(),
        size: content.len() as u64,
        size_bytes: content.len() as u64,
        modified: chrono::Utc::now(),
        backend_type: backend_type.to_string(),
        format: backend_type.to_string(),
        checksum: None,
        metadata: std::collections::HashMap::new(),
    }
}

fn bench_profile_model_loading(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let gguf_path = temp_dir.path().join("profile_test.gguf");
    let onnx_path = temp_dir.path().join("profile_test.onnx");

    let gguf_model = create_mock_model(&gguf_path, "gguf");
    let onnx_model = create_mock_model(&onnx_path, "onnx");

    let backend_config = BackendConfig::default();

    let mut group = c.benchmark_group("profile_model_loading");

    group.bench_function("gguf_load_with_profiling", |b| {
        b.to_async(&rt).iter(|| async {
            let mut backend = Backend::new(BackendType::Gguf, &backend_config).unwrap();
            let result = backend.load_model(black_box(&gguf_model)).await;
            black_box(result);
            let _ = backend.unload_model().await;
        })
    });

    group.bench_function("onnx_load_with_profiling", |b| {
        b.to_async(&rt).iter(|| async {
            let mut backend = Backend::new(BackendType::Onnx, &backend_config).unwrap();
            let result = backend.load_model(black_box(&onnx_model)).await;
            black_box(result);
            let _ = backend.unload_model().await;
        })
    });

    group.finish();
}

fn bench_profile_inference_pipeline(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let model_path = temp_dir.path().join("inference_profile.gguf");
    let model = create_mock_model(&model_path, "gguf");

    let backend_config = BackendConfig::default();
    let mut backend = rt.block_on(async {
        let mut backend = Backend::new(BackendType::Gguf, &backend_config).unwrap();
        backend.load_model(&model).await.unwrap();
        backend
    });

    // Cache removed for now - will be re-added when cache system is stabilized

    let mut metrics_collector = MetricsCollector::new();
    rt.block_on(async {
        metrics_collector.start_event_processing().await.unwrap();
    });

    let mut group = c.benchmark_group("profile_inference_pipeline");

    group.bench_function("full_inference_pipeline_with_profiling", |b| {
        b.to_async(&rt).iter(|| async {
            let prompt = "This is a test prompt for profiling the complete inference pipeline";
            let cache_key = format!("{}:0.7:50", prompt);

            // Cache disabled for now - perform inference directly
            let result = {
                // Perform inference
                let inference_params = InferenceParams {
                    max_tokens: 50,
                    temperature: 0.7,
                    top_p: 0.9,
                    stream: false,
                    stop_sequences: vec![],
                    seed: None,
                };

                let start = Instant::now();
                let result = backend
                    .infer(black_box(prompt), black_box(&inference_params))
                    .await;
                let duration = start.elapsed();

                // Record metrics
                let event = inferno::metrics::InferenceEvent {
                    model_name: "profile_test".to_string(),
                    input_length: prompt.len(),
                    output_length: result.as_ref().map(|s| s.len()).unwrap_or(0),
                    duration,
                    success: result.is_ok(),
                };
                metrics_collector.record_inference(event);

                result
            };

            black_box(result)
        })
    });

    group.finish();
}

fn bench_profile_memory_intensive_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    // Create larger mock model
    let model_path = temp_dir.path().join("large_profile.gguf");
    let large_content = vec![0u8; 10 * 1024 * 1024]; // 10MB
    let mut content = b"GGUF\x00\x00\x00\x01".to_vec();
    content.extend(large_content);
    std::fs::write(&model_path, content).unwrap();

    let model = ModelInfo {
        name: "large_profile.gguf".to_string(),
        path: model_path.clone(),
        file_path: model_path,
        size: 10 * 1024 * 1024,
        size_bytes: 10 * 1024 * 1024,
        modified: chrono::Utc::now(),
        backend_type: "gguf".to_string(),
        format: "gguf".to_string(),
        checksum: None,
        metadata: std::collections::HashMap::new(),
    };

    let backend_config = BackendConfig::default();

    let mut group = c.benchmark_group("profile_memory_intensive");

    group.bench_function("large_model_operations_with_profiling", |b| {
        b.to_async(&rt).iter(|| async {
            // Load large model
            let mut backend = Backend::new(BackendType::Gguf, &backend_config).unwrap();
            backend.load_model(black_box(&model)).await.unwrap();

            // Perform multiple inferences to stress memory
            let inference_params = InferenceParams {
                max_tokens: 100,
                temperature: 0.7,
                top_p: 0.9,
                stream: false,
                stop_sequences: vec![],
                seed: None,
            };

            for i in 0..10 {
                let prompt = format!("Large inference test number {} with significant content", i);
                let result = backend.infer(&prompt, &inference_params).await;
                black_box(result);
            }

            // Unload model
            backend.unload_model().await.unwrap();
        })
    });

    group.finish();
}

fn bench_profile_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let model_path = temp_dir.path().join("concurrent_profile.gguf");
    let model = create_mock_model(&model_path, "gguf");

    let backend_config = BackendConfig::default();

    let mut group = c.benchmark_group("profile_concurrent_operations");

    group.bench_function("concurrent_inference_with_profiling", |b| {
        b.to_async(&rt).iter(|| async {
            // Create multiple backend instances
            let backends: Vec<_> = (0..4)
                .map(|_| {
                    let mut backend = Backend::new(BackendType::Gguf, &backend_config).unwrap();
                    rt.block_on(async {
                        backend.load_model(&model).await.unwrap();
                        Arc::new(tokio::sync::Mutex::new(backend))
                    })
                })
                .collect();

            let inference_params = InferenceParams {
                max_tokens: 50,
                temperature: 0.7,
                top_p: 0.9,
                stream: false,
                stop_sequences: vec![],
                seed: None,
            };

            // Run concurrent inferences
            let handles: Vec<_> = (0..8)
                .map(|i| {
                    let backend = backends[i % backends.len()].clone();
                    let params = inference_params.clone();
                    tokio::spawn(async move {
                        let prompt = format!("Concurrent inference test {}", i);
                        let mut backend = backend.lock().await;
                        let result = backend.infer(&prompt, &params).await;
                        black_box(result)
                    })
                })
                .collect();

            for handle in handles {
                let _ = handle.await;
            }
        })
    });

    group.finish();
}

// Cache compression benchmark disabled - will be re-enabled when cache system is stabilized
// fn bench_profile_cache_compression(c: &mut Criterion) {
//     // Temporarily disabled
// }

fn bench_profile_metrics_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("profile_metrics_collection");

    group.bench_function("intensive_metrics_collection_with_profiling", |b| {
        b.to_async(&rt).iter(|| async {
            let mut collector = MetricsCollector::new();
            collector.start_event_processing().await.unwrap();

            // Generate many events rapidly
            for i in 0..1000 {
                let event = inferno::metrics::InferenceEvent {
                    model_name: format!("model_{}", i % 10),
                    input_length: 50 + (i % 100),
                    output_length: 100 + (i % 200),
                    duration: Duration::from_millis(50 + (i % 100) as u64),
                    success: i % 10 != 0,
                };
                collector.record_inference(black_box(event));
            }

            // Export multiple formats
            let prometheus_export = collector.export_prometheus_format().await;
            let json_export = collector.export_json_format().await;
            let snapshot = collector.get_snapshot().await;

            black_box((prometheus_export, json_export, snapshot));
        })
    });

    group.finish();
}

fn bench_profile_system_resource_monitoring(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("profile_system_monitoring");

    group.bench_function("system_resource_collection_with_profiling", |b| {
        b.to_async(&rt).iter(|| async {
            // Simulate intensive system monitoring
            let mut system = System::new_all();

            for _ in 0..100 {
                system.refresh_all();

                let cpu_usage = system.global_cpu_info().cpu_usage();
                let memory_usage = system.used_memory();
                let total_memory = system.total_memory();
                let processes = system.processes().len();

                // Simulate some processing of the data
                let memory_percentage = (memory_usage as f64 / total_memory as f64) * 100.0;

                black_box((cpu_usage, memory_percentage, processes));

                // Small delay to simulate real monitoring
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
    });

    group.finish();
}

criterion_group!(
    profiling_benches,
    bench_profile_model_loading,
    bench_profile_inference_pipeline,
    bench_profile_memory_intensive_operations,
    bench_profile_concurrent_operations,
    // bench_profile_cache_compression,  // Temporarily disabled
    bench_profile_metrics_collection,
    bench_profile_system_resource_monitoring
);
criterion_main!(profiling_benches);
