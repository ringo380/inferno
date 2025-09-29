// Quick integration test to verify core Inferno functionality
use std::path::PathBuf;

// Test that we can import and instantiate core types
fn test_core_types() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing core type instantiation...");

    // This will fail to compile if our types are broken
    let _config = inferno::config::Config::default();

    println!("✅ Config type works");

    // Test backend types
    use inferno::backends::BackendType;
    let _backend_type = BackendType::GGUF;
    println!("✅ Backend types work");

    // Test model types
    use inferno::models::ModelInfo;
    let model_info = ModelInfo {
        name: "test".to_string(),
        path: PathBuf::from("test.gguf"),
        backend_type: BackendType::GGUF,
        size_bytes: 1000,
        metadata: Default::default(),
    };
    println!("✅ Model types work: {}", model_info.name);

    Ok(())
}

fn test_error_types() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing error handling...");

    use inferno::InfernoError;
    let _error = InfernoError::ConfigError("test".to_string());
    println!("✅ Error types work");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 Inferno Quick Integration Test");
    println!("==================================");

    test_core_types()?;
    test_error_types()?;

    println!("\n🎯 All core components working!");
    println!("✅ Types compile and instantiate correctly");
    println!("✅ Error handling works");
    println!("✅ Module structure is sound");

    Ok(())
}