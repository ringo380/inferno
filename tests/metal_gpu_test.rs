#[cfg(all(feature = "gguf", target_os = "macos"))]
mod metal_gpu_tests {
    use inferno::backends::{BackendConfig, BackendHandle, BackendType, InferenceParams};
    use inferno::models::ModelManager;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_metal_gpu_inference() {
        // Initialize model manager
        let models_dir = PathBuf::from("models");
        let model_manager = ModelManager::new(&models_dir);

        // Find TinyLlama model
        let model = model_manager
            .resolve_model("tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf")
            .await
            .expect("Failed to find TinyLlama model");

        // Create backend config with Metal GPU enabled
        let backend_config = BackendConfig {
            gpu_enabled: true,
            gpu_device: None,
            cpu_threads: None,
            context_size: 512, // Small context for fast test
            batch_size: 128,
            memory_map: true,
        };

        // Create GGUF backend with Metal
        let backend_handle = BackendHandle::new_shared(BackendType::Gguf, &backend_config)
            .expect("Failed to create GGUF backend");

        // Load model
        backend_handle
            .load_model(&model)
            .await
            .expect("Failed to load model");

        // Verify model is loaded
        assert!(backend_handle.is_loaded().await, "Model should be loaded");

        // Run inference with a simple prompt
        let params = InferenceParams {
            max_tokens: 20, // Short output for fast test
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            stream: false,
            stop_sequences: vec![],
            seed: Some(42), // Deterministic output
        };

        let result = backend_handle
            .infer("What is 2+2?", &params)
            .await
            .expect("Inference should succeed");

        // Verify we got a response
        assert!(!result.is_empty(), "Response should not be empty");
        println!("✅ Metal GPU inference successful!");
        println!("   Prompt: What is 2+2?");
        println!("   Response: {}", result);

        // Unload model
        backend_handle
            .unload_model()
            .await
            .expect("Failed to unload model");
    }
}

#[cfg(not(all(feature = "gguf", target_os = "macos")))]
#[test]
fn metal_gpu_test_skipped() {
    println!("⚠️  Metal GPU tests require 'gguf' feature and macOS");
}
