// Basic functionality tests for Inferno
use inferno::*;
use std::path::PathBuf;

#[test]
fn test_config_creation() {
    let config = config::Config::default();
    assert!(config.models_dir.is_some());
    println!("✅ Config creation works");
}

#[test]
fn test_backend_types() {
    use backends::BackendType;

    let gguf_type = BackendType::GGUF;
    let onnx_type = BackendType::ONNX;

    assert_ne!(gguf_type, onnx_type);
    println!("✅ Backend types work");
}

#[test]
fn test_model_info() {
    use backends::BackendType;
    use models::ModelInfo;

    let model = ModelInfo {
        name: "test-model".to_string(),
        path: PathBuf::from("test.gguf"),
        backend_type: BackendType::GGUF,
        size_bytes: 1024,
        metadata: Default::default(),
    };

    assert_eq!(model.name, "test-model");
    assert_eq!(model.size_bytes, 1024);
    println!("✅ Model info creation works");
}

#[test]
fn test_error_types() {
    let error = InfernoError::ConfigError("test error".to_string());

    match error {
        InfernoError::ConfigError(msg) => {
            assert_eq!(msg, "test error");
            println!("✅ Error handling works");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_inference_params() {
    use backends::InferenceParams;

    let params = InferenceParams::default();
    assert!(params.max_tokens > 0);
    println!("✅ Inference params work");
}
