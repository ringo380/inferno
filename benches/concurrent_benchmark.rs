use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use futures::future::join_all;
use inferno::{
    backends::{Backend, BackendConfig, BackendHandle, BackendType, InferenceParams},
    metrics::MetricsCollector,
    models::{ModelInfo, ModelManager},
};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tempfile::tempdir;
use tokio::{fs, runtime::Runtime, sync::Semaphore, time::Instant};

fn create_mock_model(path: &PathBuf, backend_type: &str) -> ModelInfo {
    let content = match backend_type {
        "gguf" => b"GGUF\x00\x00\x00\x01mock model data for concurrent testing".to_vec(),
        "onnx" => b"mock onnx model data for concurrent testing".to_vec(),
        _ => panic!("Unsupported backend type"),
    };
    std::fs::write(path, content).unwrap();

    ModelInfo {
        name: path.file_name().unwrap().to_string_lossy().to_string(),
        path: path.clone(),
        size: content.len() as u64,
        modified: chrono::Utc::now(),
        backend_type: backend_type.to_string(),
        checksum: None,
    }
}

fn bench_concurrent_model_loading(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let concurrency_levels = vec![1, 2, 4, 8, 16, 32];

    let mut group = c.benchmark_group("concurrent_model_loading");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    for concurrency in concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("gguf_concurrent_load", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let handles: Vec<_> = (0..concurrency)
                        .map(|i| {
                            let temp_dir = temp_dir.path().to_path_buf();
                            tokio::spawn(async move {
                                let model_path = temp_dir.join(format!("model_{}.gguf", i));
                                let model = create_mock_model(&model_path, "gguf");
                                let backend_config = BackendConfig::default();

                                let mut backend =
                                    Backend::new(BackendType::Gguf, &backend_config).unwrap();
                                let start = Instant::now();
                                let result = backend.load_model(black_box(&model)).await;
                                let duration = start.elapsed();
                                (result, duration)
                            })
                        })
                        .collect();

                    let results = join_all(handles).await;
                    let total_time: Duration = results.into_iter().map(|r| r.unwrap().1).sum();

                    black_box(total_time)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("onnx_concurrent_load", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let handles: Vec<_> = (0..concurrency)
                        .map(|i| {
                            let temp_dir = temp_dir.path().to_path_buf();
                            tokio::spawn(async move {
                                let model_path = temp_dir.join(format!("model_{}.onnx", i));
                                let model = create_mock_model(&model_path, "onnx");
                                let backend_config = BackendConfig::default();

                                let mut backend =
                                    Backend::new(BackendType::Onnx, &backend_config).unwrap();
                                let start = Instant::now();
                                let result = backend.load_model(black_box(&model)).await;
                                let duration = start.elapsed();
                                (result, duration)
                            })
                        })
                        .collect();

                    let results = join_all(handles).await;
                    let total_time: Duration = results.into_iter().map(|r| r.unwrap().1).sum();

                    black_box(total_time)
                })
            },
        );
    }

    group.finish();
}

fn bench_concurrent_inference(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    // Pre-load models for inference testing
    let (gguf_backend, onnx_backend) = rt.block_on(async {
        let gguf_path = temp_dir.path().join("inference_test.gguf");
        let onnx_path = temp_dir.path().join("inference_test.onnx");

        let gguf_model = create_mock_model(&gguf_path, "gguf");
        let onnx_model = create_mock_model(&onnx_path, "onnx");

        let backend_config = BackendConfig::default();

        let mut gguf_backend = Backend::new(BackendType::Gguf, &backend_config).unwrap();
        let mut onnx_backend = Backend::new(BackendType::Onnx, &backend_config).unwrap();

        gguf_backend.load_model(&gguf_model).await.unwrap();
        onnx_backend.load_model(&onnx_model).await.unwrap();

        (
            Arc::new(BackendHandle::new(gguf_backend)),
            Arc::new(BackendHandle::new(onnx_backend)),
        )
    });

    let concurrency_levels = vec![1, 5, 10, 25, 50, 100];
    let test_prompts = vec![
        "Hello world!",
        "Explain artificial intelligence",
        "Write a short poem about technology",
        "What is the meaning of life?",
    ];

    let mut group = c.benchmark_group("concurrent_inference");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    for concurrency in concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("gguf_concurrent_inference", concurrency),
            &concurrency,
            |b, &concurrency| {
                let backend = gguf_backend.clone();
                b.to_async(&rt).iter(|| async {
                    let inference_params = InferenceParams {
                        max_tokens: 50,
                        temperature: 0.7,
                        top_p: 0.9,
                        stream: false,
                    };

                    let semaphore = Arc::new(Semaphore::new(concurrency));
                    let handles: Vec<_> = (0..concurrency)
                        .map(|i| {
                            let backend = backend.clone();
                            let semaphore = semaphore.clone();
                            let prompt = test_prompts[i % test_prompts.len()];
                            let params = inference_params.clone();

                            tokio::spawn(async move {
                                let _permit = semaphore.acquire().await.unwrap();
                                let start = Instant::now();
                                let result =
                                    backend.infer(black_box(prompt), black_box(&params)).await;
                                let duration = start.elapsed();
                                (result, duration)
                            })
                        })
                        .collect();

                    let results = join_all(handles).await;
                    let successful_requests = results
                        .into_iter()
                        .filter_map(|r| r.ok())
                        .filter(|(result, _)| result.is_ok())
                        .count();

                    black_box(successful_requests)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("onnx_concurrent_inference", concurrency),
            &concurrency,
            |b, &concurrency| {
                let backend = onnx_backend.clone();
                b.to_async(&rt).iter(|| async {
                    let inference_params = InferenceParams {
                        max_tokens: 50,
                        temperature: 0.7,
                        top_p: 0.9,
                        stream: false,
                    };

                    let semaphore = Arc::new(Semaphore::new(concurrency));
                    let handles: Vec<_> = (0..concurrency)
                        .map(|i| {
                            let backend = backend.clone();
                            let semaphore = semaphore.clone();
                            let prompt = test_prompts[i % test_prompts.len()];
                            let params = inference_params.clone();

                            tokio::spawn(async move {
                                let _permit = semaphore.acquire().await.unwrap();
                                let start = Instant::now();
                                let result =
                                    backend.infer(black_box(prompt), black_box(&params)).await;
                                let duration = start.elapsed();
                                (result, duration)
                            })
                        })
                        .collect();

                    let results = join_all(handles).await;
                    let successful_requests = results
                        .into_iter()
                        .filter_map(|r| r.ok())
                        .filter(|(result, _)| result.is_ok())
                        .count();

                    black_box(successful_requests)
                })
            },
        );
    }

    group.finish();
}

fn bench_concurrent_http_requests(_c: &mut Criterion) {
    // HTTP benchmark temporarily disabled due to server dependency issues
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    // Setup HTTP server for testing
    let server_handle = rt.spawn(async {
        let model_path = temp_dir.path().join("http_test.gguf");
        let model = create_mock_model(&model_path, "gguf");

        let backend_config = BackendConfig::default();
        let mut backend = Backend::new(BackendType::Gguf, &backend_config).unwrap();
        backend.load_model(&model).await.unwrap();

        let server_config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8081,
            cors_origins: vec!["*".to_string()],
            request_timeout: Duration::from_secs(30),
        };

        let server = OpenAIServer::new(Arc::new(BackendHandle::new(backend)), server_config);
        server.run().await
    });

    // Give server time to start
    rt.block_on(async {
        tokio::time::sleep(Duration::from_millis(500)).await;
    });

    let concurrency_levels = vec![1, 5, 10, 25, 50];

    let mut group = c.benchmark_group("concurrent_http_requests");
    group.sample_size(5);
    group.measurement_time(Duration::from_secs(20));

    for concurrency in concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("http_completions", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let client = reqwest::Client::new();
                    let base_url = "http://127.0.0.1:8081";

                    let handles: Vec<_> = (0..concurrency)
                        .map(|i| {
                            let client = client.clone();
                            let url = format!("{}/v1/completions", base_url);

                            tokio::spawn(async move {
                                let request_body = serde_json::json!({
                                    "model": "test_model",
                                    "prompt": format!("Test prompt number {}", i),
                                    "max_tokens": 50,
                                    "temperature": 0.7
                                });

                                let start = Instant::now();
                                let response = client.post(&url).json(&request_body).send().await;
                                let duration = start.elapsed();

                                match response {
                                    Ok(resp) => (resp.status().is_success(), duration),
                                    Err(_) => (false, duration),
                                }
                            })
                        })
                        .collect();

                    let results = join_all(handles).await;
                    let successful_requests = results
                        .into_iter()
                        .filter_map(|r| r.ok())
                        .filter(|(success, _)| *success)
                        .count();

                    black_box(successful_requests)
                })
            },
        );
    }

    group.finish();
}

fn bench_throughput_sustained_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let model_path = temp_dir.path().join("throughput_test.gguf");
    let model = create_mock_model(&model_path, "gguf");

    let backend_config = BackendConfig::default();
    let backend = rt.block_on(async {
        let mut backend = Backend::new(BackendType::Gguf, &backend_config).unwrap();
        backend.load_model(&model).await.unwrap();
        Arc::new(BackendHandle::new(backend))
    });

    let duration_tests = vec![
        Duration::from_secs(5),
        Duration::from_secs(10),
        Duration::from_secs(30),
    ];

    let mut group = c.benchmark_group("throughput_sustained_load");
    group.sample_size(3);

    for test_duration in duration_tests {
        group.bench_with_input(
            BenchmarkId::new("sustained_requests", test_duration.as_secs()),
            &test_duration,
            |b, &test_duration| {
                let backend = backend.clone();
                b.iter_custom(|_| {
                    rt.block_on(async {
                        let start = Instant::now();
                        let end_time = start + test_duration;
                        let mut request_count = 0u64;

                        let inference_params = InferenceParams {
                            max_tokens: 20,
                            temperature: 0.7,
                            top_p: 0.9,
                            stream: false,
                        };

                        while Instant::now() < end_time {
                            let handles: Vec<_> = (0..10) // Batch of 10 requests
                                .map(|i| {
                                    let backend = backend.clone();
                                    let params = inference_params.clone();
                                    tokio::spawn(async move {
                                        backend.infer(&format!("Request {}", i), &params).await
                                    })
                                })
                                .collect();

                            let results = join_all(handles).await;
                            request_count += results
                                .into_iter()
                                .filter_map(|r| r.ok())
                                .filter(|r| r.is_ok())
                                .count() as u64;
                        }

                        let actual_duration = start.elapsed();
                        let requests_per_second =
                            request_count as f64 / actual_duration.as_secs_f64();

                        eprintln!(
                            "Sustained load for {:?}: {} requests/sec",
                            test_duration, requests_per_second
                        );
                        actual_duration
                    })
                })
            },
        );
    }

    group.finish();
}

fn bench_metrics_concurrent_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let concurrency_levels = vec![1, 10, 50, 100, 500];

    let mut group = c.benchmark_group("metrics_concurrent_collection");
    group.sample_size(10);

    for concurrency in concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("concurrent_metrics", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let mut collector = MetricsCollector::new();
                    collector.start_event_processing().await.unwrap();

                    let handles: Vec<_> = (0..concurrency)
                        .map(|i| {
                            let collector = collector.clone();
                            tokio::spawn(async move {
                                let event = inferno::metrics::InferenceEvent {
                                    model_name: format!("model_{}", i % 5),
                                    input_length: 50 + (i % 20) as usize,
                                    output_length: 100 + (i % 50) as usize,
                                    duration: Duration::from_millis(50 + (i % 100) as u64),
                                    success: i % 10 != 0, // 10% failure rate
                                };
                                collector.record_inference(black_box(event));
                            })
                        })
                        .collect();

                    join_all(handles).await;

                    // Give time for events to be processed
                    tokio::time::sleep(Duration::from_millis(100)).await;

                    let snapshot = collector.get_snapshot().await;
                    black_box(snapshot)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    concurrent_benches,
    bench_concurrent_model_loading,
    bench_concurrent_inference,
    // bench_concurrent_http_requests, // Disabled due to server dependencies
    bench_throughput_sustained_load,
    bench_metrics_concurrent_collection
);
criterion_main!(concurrent_benches);
