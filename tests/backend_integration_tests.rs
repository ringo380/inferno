use anyhow::Result;
use futures::StreamExt;
use inferno::{
    InfernoError,
    backends::{Backend, BackendConfig, BackendHandle, BackendType, InferenceParams},
    cache::{CacheConfig, ModelCache},
    config::Config,
    models::{ModelInfo, ModelManager},
};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tempfile::TempDir;
use tokio::time::timeout;

/// Test utilities for backend integration tests
mod test_utils {
    use super::*;
    use std::fs;

    pub fn create_test_model_files(dir: &TempDir) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        // Create mock GGUF file
        let gguf_path = dir.path().join("test_model.gguf");
        let gguf_content = create_mock_gguf_file();
        fs::write(&gguf_path, gguf_content)?;
        files.push(gguf_path);

        // Create mock ONNX file
        let onnx_path = dir.path().join("test_model.onnx");
        let onnx_content = create_mock_onnx_file();
        fs::write(&onnx_path, onnx_content)?;
        files.push(onnx_path);

        Ok(files)
    }

    pub fn create_mock_gguf_file() -> Vec<u8> {
        let mut content = Vec::new();
        // GGUF magic number
        content.extend_from_slice(b"GGUF");
        // Version (little endian)
        content.extend_from_slice(&3u32.to_le_bytes());
        // Tensor count
        content.extend_from_slice(&0u64.to_le_bytes());
        // Metadata count
        content.extend_from_slice(&1u64.to_le_bytes());

        // Add a simple metadata entry for model name
        let key = "general.name";
        content.extend_from_slice(&(key.len() as u64).to_le_bytes());
        content.extend_from_slice(key.as_bytes());

        // Value type (string = 8)
        content.extend_from_slice(&8u32.to_le_bytes());
        let value = "test_model";
        content.extend_from_slice(&(value.len() as u64).to_le_bytes());
        content.extend_from_slice(value.as_bytes());

        // Pad to reasonable size
        content.resize(1024, 0);
        content
    }

    pub fn create_mock_onnx_file() -> Vec<u8> {
        // Create a minimal ONNX file structure
        let mut content = Vec::new();
        // ONNX magic header
        content.extend_from_slice(&[0x08, 0x01, 0x12, 0x04]);
        content.extend_from_slice(b"test");
        // Pad to reasonable size
        content.resize(1024, 0);
        content
    }

    pub fn create_test_config() -> BackendConfig {
        BackendConfig {
            gpu_enabled: false,
            gpu_device: None,
            cpu_threads: Some(2),
            context_size: 512,
            batch_size: 8,
            memory_map: true,
        }
    }

    pub async fn wait_for_condition<F, Fut>(
        mut condition: F,
        timeout_duration: Duration,
    ) -> Result<()>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout_duration {
            if condition().await {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        anyhow::bail!("Condition not met within timeout")
    }
}

/// Test backend creation and basic operations
#[tokio::test]
async fn test_backend_creation_and_basic_ops() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let model_files = test_utils::create_test_model_files(&temp_dir)?;
    let config = test_utils::create_test_config();

    // Test GGUF backend creation
    let mut gguf_backend = Backend::new(BackendType::Gguf, &config)?;
    assert_eq!(gguf_backend.get_backend_type(), BackendType::Gguf);
    assert!(!gguf_backend.is_loaded().await);
    assert!(gguf_backend.get_model_info().await.is_none());

    // Test ONNX backend creation
    let mut onnx_backend = Backend::new(BackendType::Onnx, &config)?;
    assert_eq!(onnx_backend.get_backend_type(), BackendType::Onnx);
    assert!(!onnx_backend.is_loaded().await);
    assert!(onnx_backend.get_model_info().await.is_none());

    Ok(())
}

/// Test model loading and unloading
#[tokio::test]
async fn test_model_loading_lifecycle() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let model_files = test_utils::create_test_model_files(&temp_dir)?;
    let config = test_utils::create_test_config();

    let mut model_manager = ModelManager::new(temp_dir.path().to_path_buf());
    let models = model_manager.discover_models().await?;
    assert!(!models.is_empty(), "Should discover test models");

    for (backend_type, model_path) in [
        (BackendType::Gguf, &model_files[0]),
        (BackendType::Onnx, &model_files[1]),
    ] {
        let mut backend = Backend::new(backend_type, &config)?;

        // Find the corresponding model info
        let model_info = models
            .iter()
            .find(|m| m.path == *model_path)
            .expect("Should find model info");

        // Test loading
        assert!(!backend.is_loaded().await);
        backend.load_model(model_info).await?;
        assert!(backend.is_loaded().await);

        let loaded_info = backend.get_model_info().await;
        assert!(loaded_info.is_some());
        assert_eq!(loaded_info.unwrap().path, model_info.path);

        // Test unloading
        backend.unload_model().await?;
        assert!(!backend.is_loaded().await);
        assert!(backend.get_model_info().await.is_none());
    }

    Ok(())
}

/// Test inference operations
#[tokio::test]
async fn test_backend_inference() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let model_files = test_utils::create_test_model_files(&temp_dir)?;
    let config = test_utils::create_test_config();

    let mut model_manager = ModelManager::new(temp_dir.path().to_path_buf());
    let models = model_manager.discover_models().await?;

    for (backend_type, model_path) in [
        (BackendType::Gguf, &model_files[0]),
        (BackendType::Onnx, &model_files[1]),
    ] {
        let mut backend = Backend::new(backend_type, &config)?;

        let model_info = models
            .iter()
            .find(|m| m.path == *model_path)
            .expect("Should find model info");

        backend.load_model(model_info).await?;

        // Test basic inference
        let params = InferenceParams::default();
        let input = "Hello, world!";

        let result = timeout(Duration::from_secs(10), backend.infer(input, &params)).await?;
        assert!(result.is_ok(), "Inference should succeed");

        let output = result?;
        assert!(!output.is_empty(), "Output should not be empty");

        // Test embeddings
        let embeddings = timeout(Duration::from_secs(10), backend.get_embeddings(input)).await?;
        assert!(embeddings.is_ok(), "Embeddings should succeed");

        let embedding_vec = embeddings?;
        assert!(!embedding_vec.is_empty(), "Embeddings should not be empty");

        // Test metrics
        let metrics = backend.get_metrics();
        assert!(
            metrics.is_some(),
            "Metrics should be available after inference"
        );

        backend.unload_model().await?;
    }

    Ok(())
}

/// Test streaming inference
#[tokio::test]
async fn test_streaming_inference() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let model_files = test_utils::create_test_model_files(&temp_dir)?;
    let config = test_utils::create_test_config();

    let mut model_manager = ModelManager::new(temp_dir.path().to_path_buf());
    let models = model_manager.discover_models().await?;

    let mut backend = Backend::new(BackendType::Gguf, &config)?;
    let model_info = models
        .iter()
        .find(|m| m.path == model_files[0])
        .expect("Should find GGUF model info");

    backend.load_model(model_info).await?;

    let mut params = InferenceParams::default();
    params.stream = true;
    params.max_tokens = 50;

    let input = "Once upon a time";
    let stream_result = timeout(
        Duration::from_secs(10),
        backend.infer_stream(input, &params),
    )
    .await?;
    assert!(stream_result.is_ok(), "Stream inference should succeed");

    let mut stream = stream_result?;
    let mut token_count = 0;
    let mut collected_output = String::new();

    // Collect tokens with timeout
    while let Some(token_result) = timeout(Duration::from_secs(5), stream.next()).await? {
        match token_result {
            Ok(token) => {
                collected_output.push_str(&token);
                token_count += 1;
                if token_count >= 10 {
                    break; // Limit for test
                }
            }
            Err(e) => {
                eprintln!("Stream error: {}", e);
                break;
            }
        }
    }

    assert!(token_count > 0, "Should receive at least one token");
    assert!(
        !collected_output.is_empty(),
        "Collected output should not be empty"
    );

    backend.unload_model().await?;
    Ok(())
}

/// Test BackendHandle thread safety
#[tokio::test]
async fn test_backend_handle_thread_safety() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let model_files = test_utils::create_test_model_files(&temp_dir)?;
    let config = test_utils::create_test_config();

    let mut model_manager = ModelManager::new(temp_dir.path().to_path_buf());
    let models = model_manager.discover_models().await?;

    let handle = BackendHandle::new_shared(BackendType::Gguf, &config)?;
    let model_info = models
        .iter()
        .find(|m| m.path == model_files[0])
        .expect("Should find GGUF model info");

    // Load model in handle
    handle.load_model(model_info).await?;
    assert!(handle.is_loaded().await);

    // Test concurrent access
    let handle1 = handle.clone();
    let handle2 = handle.clone();
    let handle3 = handle.clone();

    let params = InferenceParams::default();
    let input = "Test concurrent access";

    let tasks = vec![
        tokio::spawn(async move { handle1.infer(input, &params).await }),
        tokio::spawn(async move { handle2.infer(input, &params).await }),
        tokio::spawn(async move { handle3.infer(input, &params).await }),
    ];

    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;

    for task_result in results {
        let inference_result = task_result?;
        assert!(
            inference_result.is_ok(),
            "Concurrent inference should succeed"
        );
    }

    handle.unload_model().await?;
    Ok(())
}

/// Test backend configuration validation
#[tokio::test]
async fn test_backend_config_validation() -> Result<()> {
    // Test invalid context size
    let invalid_config = BackendConfig {
        context_size: 100, // Too small
        ..test_utils::create_test_config()
    };

    let temp_dir = TempDir::new()?;
    let model_files = test_utils::create_test_model_files(&temp_dir)?;

    let mut model_manager = ModelManager::new(temp_dir.path().to_path_buf());
    let models = model_manager.discover_models().await?;

    let mut backend = Backend::new(BackendType::Gguf, &invalid_config)?;
    let model_info = models
        .iter()
        .find(|m| m.path == model_files[0])
        .expect("Should find GGUF model info");

    // Should fail to load with invalid config
    let load_result = backend.load_model(model_info).await;
    assert!(load_result.is_err(), "Should fail with invalid config");

    Ok(())
}

/// Test backend error handling
#[tokio::test]
async fn test_backend_error_handling() -> Result<()> {
    let config = test_utils::create_test_config();
    let mut backend = Backend::new(BackendType::Gguf, &config)?;

    // Test inference without loaded model
    let params = InferenceParams::default();
    let result = backend.infer("test", &params).await;
    assert!(
        result.is_err(),
        "Inference should fail without loaded model"
    );

    // Test embeddings without loaded model
    let embeddings_result = backend.get_embeddings("test").await;
    assert!(
        embeddings_result.is_err(),
        "Embeddings should fail without loaded model"
    );

    // Test streaming without loaded model
    let stream_result = backend.infer_stream("test", &params).await;
    assert!(
        stream_result.is_err(),
        "Streaming should fail without loaded model"
    );

    Ok(())
}

/// Test backend type detection from file paths
#[tokio::test]
async fn test_backend_type_detection() -> Result<()> {
    // Test extension-based detection
    assert_eq!(
        BackendType::from_model_path(&PathBuf::from("model.gguf")),
        BackendType::Gguf
    );
    assert_eq!(
        BackendType::from_model_path(&PathBuf::from("model.onnx")),
        BackendType::Onnx
    );

    // Test filename pattern-based detection
    assert_eq!(
        BackendType::from_model_path(&PathBuf::from("llama-7b-chat")),
        BackendType::Gguf
    );
    assert_eq!(
        BackendType::from_model_path(&PathBuf::from("bert-base-onnx")),
        BackendType::Onnx
    );

    // Test default fallback
    assert_eq!(
        BackendType::from_model_path(&PathBuf::from("unknown")),
        BackendType::Gguf
    );

    Ok(())
}

/// Test backend performance metrics
#[tokio::test]
async fn test_backend_metrics_collection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let model_files = test_utils::create_test_model_files(&temp_dir)?;
    let config = test_utils::create_test_config();

    let mut model_manager = ModelManager::new(temp_dir.path().to_path_buf());
    let models = model_manager.discover_models().await?;

    let mut backend = Backend::new(BackendType::Gguf, &config)?;
    let model_info = models
        .iter()
        .find(|m| m.path == model_files[0])
        .expect("Should find GGUF model info");

    backend.load_model(model_info).await?;

    // Perform multiple inferences to generate metrics
    let params = InferenceParams::default();
    for i in 0..5 {
        let input = format!("Test inference {}", i);
        let _ = backend.infer(&input, &params).await?;
    }

    // Check metrics are collected
    let metrics = backend.get_metrics();
    assert!(metrics.is_some(), "Metrics should be available");

    let m = metrics.unwrap();
    assert!(m.total_tokens > 0, "Should have token count");
    assert!(m.total_time_ms > 0, "Should have execution time");
    assert!(
        m.tokens_per_second > 0.0,
        "Should have calculated tokens per second"
    );

    backend.unload_model().await?;
    Ok(())
}

/// Integration test combining backends with caching
#[tokio::test]
async fn test_backend_cache_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let model_files = test_utils::create_test_model_files(&temp_dir)?;

    let cache_config = CacheConfig {
        max_cached_models: 2,
        model_ttl_seconds: 60,
        enable_warmup: false,
        ..Default::default()
    };

    let backend_config = test_utils::create_test_config();
    let model_manager = Arc::new(ModelManager::new(temp_dir.path().to_path_buf()));
    let models = model_manager.discover_models().await?;

    let cache = ModelCache::new(cache_config, model_manager.clone()).await?;

    // Get cached backend for GGUF model
    let gguf_model = models
        .iter()
        .find(|m| m.path == model_files[0])
        .expect("Should find GGUF model");

    let handle1 = cache
        .get_or_load_model(&gguf_model.id, BackendType::Gguf, &backend_config)
        .await?;
    assert!(handle1.is_loaded().await);

    // Get the same model again - should be cached
    let handle2 = cache
        .get_or_load_model(&gguf_model.id, BackendType::Gguf, &backend_config)
        .await?;
    assert!(handle2.is_loaded().await);

    // Perform inference on both handles
    let params = InferenceParams::default();
    let result1 = handle1.infer("test1", &params).await?;
    let result2 = handle2.infer("test2", &params).await?;

    assert!(!result1.is_empty());
    assert!(!result2.is_empty());

    // Verify cache statistics
    let stats = cache.get_stats().await;
    assert!(stats.cache_hits > 0, "Should have cache hits");
    assert!(stats.cached_models > 0, "Should have cached models");

    Ok(())
}
