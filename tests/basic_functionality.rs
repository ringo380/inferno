// Basic functionality tests for Inferno
use inferno::*;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_config_creation() {
    let config = config::Config::default();
    // models_dir is now a PathBuf directly, not Option<PathBuf>
    assert!(!config.models_dir.as_os_str().is_empty());
    println!("✅ Config creation works");
}

#[test]
fn test_backend_types() {
    use backends::BackendType;

    // Note: BackendType variants are feature-gated, using Gguf as default
    #[cfg(feature = "gguf")]
    {
        let gguf_type = BackendType::Gguf;
        assert_eq!(format!("{:?}", gguf_type), "Gguf");
    }
    println!("✅ Backend types work");
}

#[test]
fn test_model_info() {
    use models::ModelInfo;

    let model = ModelInfo {
        name: "test-model".to_string(),
        path: PathBuf::from("test.gguf"),
        file_path: PathBuf::from("test.gguf"),
        size: 1024,
        size_bytes: 1024,
        modified: chrono::Utc::now(),
        backend_type: "gguf".to_string(),
        format: "gguf".to_string(),
        checksum: None,
        metadata: HashMap::new(),
    };

    assert_eq!(model.name, "test-model");
    assert_eq!(model.size_bytes, 1024);
    println!("✅ Model info creation works");
}

#[test]
fn test_error_types() {
    // InfernoError::Backend is a simple string variant
    let error = InfernoError::Backend("test error".to_string());

    match error {
        InfernoError::Backend(msg) => {
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
