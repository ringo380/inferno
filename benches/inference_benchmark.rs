use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use inferno::{
    backends::{Backend, BackendConfig, BackendType, InferenceParams},
    config::Config,
    models::{ModelInfo, ModelManager},
};
use std::{path::PathBuf, time::Duration};
use tempfile::tempdir;
use tokio::{fs, runtime::Runtime};

fn create_mock_gguf_model(path: &PathBuf) -> ModelInfo {
    std::fs::write(
        path,
        b"GGUF\x00\x00\x00\x01mock model data for benchmarking",
    )
    .unwrap();

    ModelInfo {
        name: path.file_name().unwrap().to_string_lossy().to_string(),
        path: path.clone(),
        file_path: path.clone(),
        size: 1024 * 1024, // 1MB
        size_bytes: 1024 * 1024,
        modified: chrono::Utc::now(),
        backend_type: "gguf".to_string(),
        format: "gguf".to_string(),
        checksum: None,
        metadata: std::collections::HashMap::new(),
    }
}

fn create_mock_onnx_model(path: &PathBuf) -> ModelInfo {
    std::fs::write(path, b"mock onnx model data for benchmarking purposes").unwrap();

    ModelInfo {
        name: path.file_name().unwrap().to_string_lossy().to_string(),
        path: path.clone(),
        file_path: path.clone(),
        size: 2 * 1024 * 1024, // 2MB
        size_bytes: 2 * 1024 * 1024,
        modified: chrono::Utc::now(),
        backend_type: "onnx".to_string(),
        format: "onnx".to_string(),
        checksum: None,
        metadata: std::collections::HashMap::new(),
    }
}

fn bench_model_loading(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let gguf_path = temp_dir.path().join("test.gguf");
    let onnx_path = temp_dir.path().join("test.onnx");

    let gguf_model = create_mock_gguf_model(&gguf_path);
    let onnx_model = create_mock_onnx_model(&onnx_path);

    let backend_config = BackendConfig::default();

    let mut group = c.benchmark_group("model_loading");

    group.bench_function("gguf_backend_creation", |b| {
        b.iter(|| {
            let backend = Backend::new(BackendType::Gguf, black_box(&backend_config));
            black_box(backend)
        })
    });

    group.bench_function("onnx_backend_creation", |b| {
        b.iter(|| {
            let backend = Backend::new(BackendType::Onnx, black_box(&backend_config));
            black_box(backend)
        })
    });

    group.bench_function("gguf_model_load", |b| {
        b.to_async(&rt).iter(|| async {
            let mut backend = Backend::new(BackendType::Gguf, &backend_config).unwrap();
            let result = backend.load_model(black_box(&gguf_model)).await;
            black_box(result)
        })
    });

    group.bench_function("onnx_model_load", |b| {
        b.to_async(&rt).iter(|| async {
            let mut backend = Backend::new(BackendType::Onnx, &backend_config).unwrap();
            let result = backend.load_model(black_box(&onnx_model)).await;
            black_box(result)
        })
    });

    group.finish();
}

fn bench_inference(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let gguf_path = temp_dir.path().join("test.gguf");
    let onnx_path = temp_dir.path().join("test.onnx");

    let gguf_model = create_mock_gguf_model(&gguf_path);
    let onnx_model = create_mock_onnx_model(&onnx_path);

    let backend_config = BackendConfig::default();

    // Pre-load models for inference benchmarks
    let mut gguf_backend = rt.block_on(async {
        let mut backend = Backend::new(BackendType::Gguf, &backend_config).unwrap();
        backend.load_model(&gguf_model).await.unwrap();
        backend
    });

    let mut onnx_backend = rt.block_on(async {
        let mut backend = Backend::new(BackendType::Onnx, &backend_config).unwrap();
        backend.load_model(&onnx_model).await.unwrap();
        backend
    });

    let inference_params = InferenceParams {
        max_tokens: 50,
        temperature: 0.7,
        top_p: 0.9,
        stream: false,
        stop_sequences: vec![],
        seed: None,
    };

    let test_prompts = vec![
        "Hello, world!",
        "The quick brown fox jumps over the lazy dog.",
        "Explain the concept of artificial intelligence.",
        "Write a short story about a robot.",
    ];

    let mut group = c.benchmark_group("inference");

    for prompt in &test_prompts {
        group.bench_with_input(
            BenchmarkId::new("gguf", prompt.len()),
            prompt,
            |b, prompt| {
                b.to_async(&rt).iter(|| async {
                    let result = gguf_backend
                        .infer(black_box(prompt), black_box(&inference_params))
                        .await;
                    black_box(result)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("onnx", prompt.len()),
            prompt,
            |b, prompt| {
                b.to_async(&rt).iter(|| async {
                    let result = onnx_backend
                        .infer(black_box(prompt), black_box(&inference_params))
                        .await;
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

fn bench_model_manager_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");

    rt.block_on(async {
        fs::create_dir_all(&models_dir).await.unwrap();

        // Create multiple mock models
        for i in 0..10 {
            let gguf_path = models_dir.join(format!("model_{}.gguf", i));
            let onnx_path = models_dir.join(format!("model_{}.onnx", i));

            create_mock_gguf_model(&gguf_path);
            create_mock_onnx_model(&onnx_path);
        }
    });

    let model_manager = ModelManager::new(&models_dir);

    let mut group = c.benchmark_group("model_manager");

    group.bench_function("list_models", |b| {
        b.to_async(&rt).iter(|| async {
            let result = model_manager.list_models().await;
            black_box(result)
        })
    });

    group.bench_function("resolve_model", |b| {
        b.to_async(&rt).iter(|| async {
            let result = model_manager.resolve_model(black_box("model_0.gguf")).await;
            black_box(result)
        })
    });

    // Create a model for validation benchmarks
    let test_model_path = models_dir.join("bench_test.gguf");
    create_mock_gguf_model(&test_model_path);

    group.bench_function("validate_model", |b| {
        b.to_async(&rt).iter(|| async {
            let result = model_manager
                .validate_model(black_box(&test_model_path))
                .await;
            black_box(result)
        })
    });

    group.bench_function("compute_checksum", |b| {
        b.to_async(&rt).iter(|| async {
            let result = model_manager
                .compute_checksum(black_box(&test_model_path))
                .await;
            black_box(result)
        })
    });

    group.finish();
}

fn bench_configuration_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let mut group = c.benchmark_group("configuration");

    group.bench_function("config_creation", |b| {
        b.iter(|| {
            let config = Config::default();
            black_box(config)
        })
    });

    group.bench_function("config_validation", |b| {
        b.iter(|| {
            let mut config = Config::default();
            config.models_dir = temp_dir.path().join("models");
            config.cache_dir = temp_dir.path().join("cache");

            // Create directories for validation
            std::fs::create_dir_all(&config.models_dir).unwrap();
            std::fs::create_dir_all(&config.cache_dir).unwrap();

            let result = config.validate();
            black_box(result)
        })
    });

    group.bench_function("config_serialization", |b| {
        b.iter(|| {
            let config = Config::default();
            let result = toml::to_string(&config);
            black_box(result)
        })
    });

    group.finish();
}

fn bench_io_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let text_data = "The quick brown fox jumps over the lazy dog. ".repeat(100);
    let json_data = serde_json::json!({
        "model": "test_model",
        "prompt": text_data.clone(),
        "parameters": {
            "temperature": 0.7,
            "max_tokens": 100
        },
        "metadata": {
            "timestamp": "2023-01-01T00:00:00Z",
            "version": "1.0.0"
        }
    });

    let mut group = c.benchmark_group("io_operations");

    group.bench_function("text_write_read", |b| {
        b.to_async(&rt).iter(|| async {
            let path = temp_dir.path().join("bench_text.txt");

            inferno::io::text::write_text_file(&path, black_box(&text_data))
                .await
                .unwrap();
            let result = inferno::io::text::read_text_file(black_box(&path)).await;
            black_box(result)
        })
    });

    group.bench_function("json_write_read", |b| {
        b.to_async(&rt).iter(|| async {
            let path = temp_dir.path().join("bench_data.json");

            inferno::io::json::write_json_file(&path, black_box(&json_data))
                .await
                .unwrap();
            let result: serde_json::Value = inferno::io::json::read_json_file(black_box(&path))
                .await
                .unwrap();
            black_box(result)
        })
    });

    group.bench_function("jsonl_append", |b| {
        b.to_async(&rt).iter(|| async {
            let path = temp_dir.path().join("bench_data.jsonl");
            let result = inferno::io::json::append_jsonl_file(&path, black_box(&json_data)).await;
            black_box(result)
        })
    });

    group.finish();
}

fn bench_metrics_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("metrics");

    group.bench_function("metrics_collector_creation", |b| {
        b.iter(|| {
            let collector = inferno::metrics::MetricsCollector::new();
            black_box(collector)
        })
    });

    group.bench_function("inference_event_recording", |b| {
        b.to_async(&rt).iter(|| async {
            let mut collector = inferno::metrics::MetricsCollector::new();
            collector.start_event_processing().await.unwrap();

            let event = inferno::metrics::InferenceEvent {
                model_name: "test_model".to_string(),
                input_length: 50,
                output_length: 100,
                duration: Duration::from_millis(100),
                success: true,
            };

            collector.record_inference(black_box(event));
            black_box(())
        })
    });

    group.bench_function("metrics_snapshot", |b| {
        b.to_async(&rt).iter(|| async {
            let collector = inferno::metrics::MetricsCollector::new();
            let result = collector.get_snapshot().await;
            black_box(result)
        })
    });

    group.bench_function("prometheus_export", |b| {
        b.to_async(&rt).iter(|| async {
            let collector = inferno::metrics::MetricsCollector::new();
            let result = collector.export_prometheus_format().await;
            black_box(result)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_model_loading,
    bench_inference,
    bench_model_manager_operations,
    bench_configuration_operations,
    bench_io_operations,
    bench_metrics_operations
);
criterion_main!(benches);
