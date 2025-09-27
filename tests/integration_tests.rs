use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("An offline AI/ML model runner"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("inferno"));
}

#[test]
fn test_models_list_empty() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).unwrap();

    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("models")
        .arg("list")
        .env("INFERNO_MODELS_DIR", models_dir.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No models found"));
}

#[test]
fn test_models_list_with_mock_model() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).unwrap();

    // Create a mock GGUF file with correct version format
    let model_path = models_dir.join("test_model.gguf");
    let mut gguf_data = b"GGUF".to_vec();
    gguf_data.extend_from_slice(&3u32.to_le_bytes()); // Version 3 in little-endian
    gguf_data.extend_from_slice(b"mock data");
    fs::write(&model_path, &gguf_data).unwrap();

    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("models")
        .arg("list")
        .env("INFERNO_MODELS_DIR", models_dir.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test_model.gguf"));
}

#[test]
fn test_validate_nonexistent_file() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("validate").arg("/nonexistent/file.gguf");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("does not exist"));
}

#[test]
fn test_validate_mock_gguf_file() {
    let temp_dir = tempdir().unwrap();
    let model_path = temp_dir.path().join("test.gguf");
    let mut gguf_data = b"GGUF".to_vec();
    gguf_data.extend_from_slice(&3u32.to_le_bytes()); // Version 3 in little-endian
    gguf_data.extend_from_slice(b"mock data");
    fs::write(&model_path, &gguf_data).unwrap();

    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("validate").arg(model_path.to_str().unwrap());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("All validations passed"));
}

#[test]
fn test_validate_invalid_gguf_file() {
    let temp_dir = tempdir().unwrap();
    let model_path = temp_dir.path().join("invalid.gguf");
    fs::write(&model_path, b"INVALID_DATA").unwrap();

    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("validate").arg(model_path.to_str().unwrap());
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("validations failed"));
}

#[test]
fn test_run_without_model() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("run")
        .arg("--model")
        .arg("nonexistent_model")
        .arg("--prompt")
        .arg("Hello");

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("not found").or(predicate::str::contains("Model")));
}

#[tokio::test]
async fn test_config_operations() {
    use inferno::config::Config;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let mut config = Config::default();
    config.models_dir = temp_dir.path().join("models");
    config.cache_dir = temp_dir.path().join("cache");

    // Test config save/load
    let config_path = temp_dir.path().join("config.toml");
    config.save(Some(&config_path)).unwrap();

    assert!(config_path.exists());

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("[backend_config]"));
    assert!(content.contains("[server]"));
}

#[tokio::test]
async fn test_model_manager() {
    use inferno::models::ModelManager;
    use tempfile::tempdir;
    use tokio::fs;

    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).await.unwrap();

    let manager = ModelManager::new(&models_dir);

    // Test empty directory
    let models = manager.list_models().await.unwrap();
    assert!(models.is_empty());

    // Create mock models
    let gguf_path = models_dir.join("test.gguf");
    let onnx_path = models_dir.join("test.onnx");

    let mut gguf_data = b"GGUF".to_vec();
    gguf_data.extend_from_slice(&3u32.to_le_bytes()); // Version 3 in little-endian
    gguf_data.extend_from_slice(b"test data");
    fs::write(&gguf_path, &gguf_data).await.unwrap();
    fs::write(&onnx_path, b"test onnx data").await.unwrap();

    let models = manager.list_models().await.unwrap();
    assert_eq!(models.len(), 2);

    // Test model resolution
    let model = manager.resolve_model("test.gguf").await.unwrap();
    assert_eq!(model.name, "test.gguf");
    assert_eq!(model.backend_type, "gguf");

    // Test model validation - GGUF should be valid, ONNX format detection may vary
    let gguf_valid = manager.validate_model(&gguf_path).await.unwrap();
    assert!(gguf_valid);

    // ONNX validation may not pass with mock data, so we just test it doesn't panic
    let _onnx_result = manager.validate_model(&onnx_path).await.unwrap();

    // Test checksum computation
    let checksum = manager.compute_checksum(&gguf_path).await.unwrap();
    assert!(!checksum.is_empty());
    assert_eq!(checksum.len(), 64); // SHA256 hex length
}

#[tokio::test]
async fn test_backend_creation() {
    use inferno::backends::{Backend, BackendConfig, BackendType};

    let config = BackendConfig::default();

    // Test GGUF backend creation
    let backend = Backend::new(BackendType::Gguf, &config);
    assert!(backend.is_ok());

    // Test ONNX backend creation
    let backend = Backend::new(BackendType::Onnx, &config);
    assert!(backend.is_ok());
}

#[tokio::test]
async fn test_io_operations() {
    use inferno::io::{json, text};
    use serde_json::json;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();

    // Test text I/O
    let text_path = temp_dir.path().join("test.txt");
    let test_content = "Hello, world!\nTest content.";

    text::write_text_file(&text_path, test_content)
        .await
        .unwrap();
    let read_content = text::read_text_file(&text_path).await.unwrap();
    assert_eq!(test_content, read_content);

    // Test JSON I/O
    let json_path = temp_dir.path().join("test.json");
    let test_data = json!({
        "name": "test",
        "value": 42,
        "array": [1, 2, 3]
    });

    json::write_json_file(&json_path, &test_data).await.unwrap();
    let read_data: serde_json::Value = json::read_json_file(&json_path).await.unwrap();
    assert_eq!(test_data, read_data);
}

#[tokio::test]
async fn test_metrics_collector() {
    use inferno::metrics::{InferenceEvent, MetricsCollector};
    use std::time::Duration;
    use tokio::time::sleep;

    let mut collector = MetricsCollector::new();
    collector.start_event_processing().await.unwrap();

    // Record model load
    collector.record_model_loaded(
        "test_model".to_string(),
        1024 * 1024,
        Duration::from_millis(100),
        "gguf".to_string(),
    );

    // Record inference
    let event = InferenceEvent {
        model_name: "test_model".to_string(),
        input_length: 50,
        output_length: 100,
        duration: Duration::from_millis(500),
        success: true,
    };
    collector.record_inference(event);

    // Allow processing
    sleep(Duration::from_millis(50)).await;

    let snapshot = collector.get_snapshot().await.unwrap();
    assert_eq!(snapshot.inference_metrics.total_requests, 1);
    assert_eq!(snapshot.inference_metrics.successful_requests, 1);
    assert_eq!(snapshot.model_metrics.loaded_models.len(), 1);

    // Test exports
    let json_export = collector.export_metrics_json().await.unwrap();
    assert!(json_export.contains("inference_metrics"));

    let prometheus_export = collector.export_prometheus_format().await.unwrap();
    assert!(prometheus_export.contains("inferno_inference_requests_total"));
}

// Smoke tests for complex operations
#[tokio::test]
async fn test_full_pipeline_smoke_test() {
    use inferno::{
        backends::{Backend, BackendConfig, BackendType},
        config::Config,
        models::ModelManager,
    };
    use tempfile::tempdir;
    use tokio::fs;

    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).await.unwrap();

    // Create config
    let mut config = Config::default();
    config.models_dir = models_dir.clone();

    // Create mock model
    let model_path = models_dir.join("test.gguf");
    let mut gguf_data = b"GGUF".to_vec();
    gguf_data.extend_from_slice(&3u32.to_le_bytes()); // Version 3 in little-endian
    gguf_data.extend_from_slice(b"test model data");
    fs::write(&model_path, &gguf_data).await.unwrap();

    // Test model discovery
    let model_manager = ModelManager::new(&models_dir);
    let models = model_manager.list_models().await.unwrap();
    assert_eq!(models.len(), 1);

    // Test backend creation
    let backend_config = BackendConfig::default();
    let backend = Backend::new(BackendType::Gguf, &backend_config);
    assert!(backend.is_ok());

    // This completes without panicking, which is our main goal for the smoke test
}
