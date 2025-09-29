use inferno::{
    backends::{Backend, BackendConfig, BackendType, InferenceParams},
    models::ModelInfo,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔥 Inferno Simple Inference Example");
    println!("===================================");

    // Initialize backend configuration
    let config = BackendConfig::default();
    let mut backend = Backend::new(BackendType::GGUF, &config)?;

    // Use real tinyllama model for testing
    let model_info = ModelInfo {
        name: "tinyllama.gguf".to_string(),
        path: PathBuf::from("dashboard/test_models/tinyllama.gguf"),
        file_path: PathBuf::from("dashboard/test_models/tinyllama.gguf"),
        size: 94 * 1024 * 1024, // 94MB
        size_bytes: 94 * 1024 * 1024,
        modified: chrono::Utc::now(),
        backend_type: "gguf".to_string(),
        format: "gguf".to_string(),
        checksum: None,
        metadata: std::collections::HashMap::new(),
    };

    println!("📦 Loading model: {}", model_info.name);

    // Note: In a real scenario, you would have an actual model file
    // This example shows the API structure
    match backend.load_model(&model_info).await {
        Ok(_) => println!("✅ Model loaded successfully!"),
        Err(e) => {
            println!("❌ Model loading failed: {}", e);
            println!("💡 To run this example with a real model:");
            println!("   1. Place a .gguf model file in the models directory");
            println!("   2. Update the model path in this example");
            return Ok(());
        }
    }

    // Set up inference parameters
    let params = InferenceParams {
        max_tokens: 10, // Start with just 10 tokens for quick test
        temperature: 0.7,
        top_p: 0.9,
        stream: false,
        stop_sequences: vec![],
        seed: None,
    };

    let prompt = "Hello, world! Please introduce yourself.";
    println!("🤖 Running inference with prompt: \"{}\"", prompt);

    // Run inference
    match backend.infer(prompt, &params).await {
        Ok(result) => {
            println!("📝 Result:");
            println!("{}", result);
        }
        Err(e) => {
            println!("❌ Inference failed: {}", e);
        }
    }

    Ok(())
}
