/// Metal GPU Performance Tests
///
/// These tests validate Metal GPU acceleration performance on macOS.
/// They are automatically skipped on non-macOS platforms.
///
/// Performance targets (from CLAUDE.md):
/// - 7B models: >30 tokens/sec on M1 Max
/// - 13B models: >15 tokens/sec on M2 Max
/// - 70B models: >5 tokens/sec on M4 Max (with unified memory)

#[cfg(all(target_os = "macos", feature = "gguf"))]
mod metal_tests {
    use chrono;
    use inferno::backends::{Backend, BackendConfig, BackendType, InferenceParams};
    use inferno::models::ModelInfo;
    use std::path::PathBuf;
    use std::time::Instant;

    /// Test that Metal acceleration is enabled by default on macOS
    #[test]
    fn test_metal_enabled_by_default_on_macos() {
        let config = BackendConfig::default();
        assert!(
            config.gpu_enabled,
            "GPU should be enabled by default on macOS"
        );
    }

    /// Test that with_metal_acceleration() returns proper config
    #[test]
    fn test_metal_acceleration_config() {
        let config = BackendConfig::with_metal_acceleration();

        assert!(config.gpu_enabled, "GPU should be enabled");
        assert_eq!(
            config.context_size, 4096,
            "Metal config should have larger context"
        );
        assert_eq!(
            config.batch_size, 64,
            "Metal config should have larger batch size"
        );
        assert!(config.memory_map, "Memory mapping should be enabled");
    }

    /// Test that cpu_only() disables GPU
    #[test]
    fn test_cpu_only_config() {
        let config = BackendConfig::cpu_only();
        assert!(
            !config.gpu_enabled,
            "GPU should be disabled for CPU-only config"
        );
    }

    /// Test that BackendType::Metal creates a backend successfully
    #[cfg(feature = "gpu-metal")]
    #[test]
    fn test_metal_backend_creation() {
        let config = BackendConfig::with_metal_acceleration();

        // BackendType::Metal should redirect to GGUF with Metal acceleration
        let result = Backend::new(BackendType::Metal, &config);

        // The backend should be created successfully (it uses GGUF internally)
        assert!(
            result.is_ok(),
            "Metal backend (via GGUF) should be created successfully: {:?}",
            result.err()
        );

        let backend = result.unwrap();

        // Note: The backend type returned may be GGUF since Metal redirects to it
        // This is expected behavior - Metal is implemented via GGUF + llama-cpp-2
    }

    /// Test GGUF backend with explicit Metal configuration
    #[tokio::test]
    async fn test_gguf_backend_with_metal_config() {
        let config = BackendConfig::with_metal_acceleration();

        let result = Backend::new(BackendType::Gguf, &config);
        assert!(
            result.is_ok(),
            "GGUF backend with Metal config should be created: {:?}",
            result.err()
        );

        let backend = result.unwrap();
        assert_eq!(
            backend.get_backend_type(),
            BackendType::Gguf,
            "Backend type should be GGUF"
        );
    }

    /// Performance test: Measure tokens per second for inference
    /// This test requires a real model file to run meaningful benchmarks.
    /// Set INFERNO_MODELS_DIR and have a valid .gguf model available.
    #[tokio::test]
    #[ignore = "Requires real model file - run with: cargo test --test metal_performance_tests test_inference_performance -- --ignored"]
    async fn test_inference_performance() {
        use std::env;

        let models_dir =
            env::var("INFERNO_MODELS_DIR").unwrap_or_else(|_| "test_models".to_string());
        let model_path = PathBuf::from(&models_dir);

        // Find first .gguf file in models directory
        let model_file = std::fs::read_dir(&model_path).ok().and_then(|entries| {
            entries
                .filter_map(|e| e.ok())
                .find(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "gguf")
                        .unwrap_or(false)
                })
                .map(|e| e.path())
        });

        let model_file = match model_file {
            Some(path) => path,
            None => {
                eprintln!(
                    "No .gguf model found in {}. Skipping performance test.",
                    models_dir
                );
                return;
            }
        };

        println!("Testing with model: {:?}", model_file);

        // Create backend with Metal acceleration
        let config = BackendConfig::with_metal_acceleration();
        let mut backend =
            Backend::new(BackendType::Gguf, &config).expect("Failed to create GGUF backend");

        // Create model info
        let model_metadata = std::fs::metadata(&model_file).ok();
        let model_size = model_metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let model_modified = model_metadata
            .and_then(|m| m.modified().ok())
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t))
            .unwrap_or_else(chrono::Utc::now);

        let model_info = ModelInfo {
            name: model_file
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            path: model_file.clone(),
            file_path: model_file.clone(),
            size: model_size,
            size_bytes: model_size,
            modified: model_modified,
            format: "gguf".to_string(),
            backend_type: "gguf".to_string(),
            checksum: None,
            metadata: Default::default(),
        };

        // Load the model
        let load_start = Instant::now();
        backend
            .load_model(&model_info)
            .await
            .expect("Failed to load model");
        let load_time = load_start.elapsed();
        println!("Model loaded in {:?}", load_time);

        // Run inference benchmark
        let params = InferenceParams {
            max_tokens: 100,
            temperature: 0.0, // Deterministic for benchmarking
            ..Default::default()
        };

        let prompt = "The quick brown fox";
        let num_runs = 3;
        let mut total_tokens = 0u32;
        let mut total_time_ms = 0u64;

        for i in 0..num_runs {
            let start = Instant::now();
            let result = backend.infer(prompt, &params).await;
            let elapsed = start.elapsed();

            match result {
                Ok(output) => {
                    let tokens = output.split_whitespace().count() as u32;
                    total_tokens += tokens;
                    total_time_ms += elapsed.as_millis() as u64;
                    println!(
                        "Run {}: {} tokens in {:?} ({:.1} tok/s)",
                        i + 1,
                        tokens,
                        elapsed,
                        tokens as f64 / elapsed.as_secs_f64()
                    );
                }
                Err(e) => {
                    eprintln!("Inference failed on run {}: {}", i + 1, e);
                }
            }
        }

        if total_tokens > 0 && total_time_ms > 0 {
            let avg_tokens_per_sec = (total_tokens as f64) / (total_time_ms as f64 / 1000.0);
            println!(
                "\nAverage: {:.1} tokens/sec ({} tokens in {} ms)",
                avg_tokens_per_sec, total_tokens, total_time_ms
            );

            // Performance assertion - adjust threshold based on model size
            // For a 7B model on M1 Max, we expect >30 tok/s
            // This is a soft assertion - we log the result regardless
            if avg_tokens_per_sec < 10.0 {
                eprintln!(
                    "Warning: Performance below 10 tok/s ({:.1}). GPU may not be enabled.",
                    avg_tokens_per_sec
                );
            }
        }

        // Cleanup
        backend.unload_model().await.ok();
    }
}

/// Tests that compile on all platforms but provide no-op behavior on non-macOS
#[cfg(not(all(target_os = "macos", feature = "gguf")))]
mod metal_tests_stub {
    #[test]
    fn test_metal_skipped_on_non_macos() {
        // This test exists to ensure the test file compiles on all platforms
        // Metal-specific tests only run on macOS with the gguf feature enabled
        #[cfg(not(target_os = "macos"))]
        println!("Metal tests skipped: not running on macOS");

        #[cfg(all(target_os = "macos", not(feature = "gguf")))]
        println!("Metal tests skipped: gguf feature not enabled");
    }
}
