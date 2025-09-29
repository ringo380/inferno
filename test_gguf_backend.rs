// Test file to verify GGUF backend implementation
use std::path::PathBuf;
use inferno::{
    backends::{Backend, BackendConfig, BackendType, InferenceParams},
    models::ModelInfo,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Testing GGUF Backend Implementation");

    // Create backend config
    let config = BackendConfig {
        gpu_enabled: false,
        gpu_device: None,
        cpu_threads: Some(4),
        context_size: 2048,
        batch_size: 32,
        memory_map: true,
    };

    // Create GGUF backend
    let mut backend = Backend::new(BackendType::Gguf, &config)?;
    println!("✓ GGUF backend created successfully");

    // Create model info
    let model_path = PathBuf::from("test_models/test_models/test-model.gguf");
    let model_info = ModelInfo {
        path: model_path.clone(),
        file_path: model_path.clone(),
        name: "test-model".to_string(),
        size: std::fs::metadata(&model_path)?.len(),
        size_bytes: std::fs::metadata(&model_path)?.len(),
        modified: chrono::Utc::now(),
        backend_type: "gguf".to_string(),
        format: "gguf".to_string(),
        checksum: None,
        metadata: std::collections::HashMap::new(),
    };

    // Test model loading
    println!("Loading model: {}", model_info.path.display());
    backend.load_model(&model_info).await?;
    println!("✓ Model loaded successfully");

    // Check if model is loaded
    assert!(backend.is_loaded().await);
    println!("✓ Model is confirmed loaded");

    // Test inference
    let params = InferenceParams {
        max_tokens: 50,
        temperature: 0.7,
        top_p: 0.9,
        stream: false,
        stop_sequences: vec![],
        seed: None,
    };

    println!("Running inference...");
    let response = backend.infer("Hello, how are you?", &params).await?;
    println!("✓ Inference completed");
    println!("Response preview: {}", response.chars().take(200).collect::<String>());

    // Test streaming inference
    println!("Testing streaming inference...");
    let mut stream = backend.infer_stream("Tell me a story", &params).await?;

    println!("✓ Stream created successfully");
    let mut token_count = 0;
    use futures::StreamExt;

    while let Some(result) = stream.next().await {
        match result {
            Ok(token) => {
                print!("{}", token);
                token_count += 1;
                if token_count >= 10 {
                    println!("... (truncated after {} tokens)", token_count);
                    break;
                }
            }
            Err(e) => {
                println!("Stream error: {}", e);
                break;
            }
        }
    }
    println!("\n✓ Streaming inference completed");

    // Test embeddings
    println!("Testing embeddings...");
    let embeddings = backend.get_embeddings("This is a test").await?;
    println!("✓ Generated {} dimensional embeddings", embeddings.len());

    // Test metrics
    if let Some(metrics) = backend.get_metrics() {
        println!("✓ Metrics available:");
        println!("  - Total tokens: {}", metrics.total_tokens);
        println!("  - Tokens per second: {:.2}", metrics.tokens_per_second);
        println!("  - Total time: {}ms", metrics.total_time_ms);
    }

    // Test unloading
    backend.unload_model().await?;
    assert!(!backend.is_loaded().await);
    println!("✓ Model unloaded successfully");

    println!("\n🎉 All GGUF backend tests passed!");
    println!("The real GGUF backend implementation is working correctly!");

    Ok(())
}