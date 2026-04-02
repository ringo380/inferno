#![allow(
    clippy::redundant_closure,
    clippy::needless_borrow,
    clippy::let_unit_value,
    unused_imports,
    dead_code,
    unused_variables
)]

//! Metal GPU Performance Tests
//!
//! These tests validate Metal GPU acceleration performance on macOS.
//! They are automatically skipped on non-macOS platforms.
//!
//! Performance targets (from CLAUDE.md):
//! - 7B models: >30 tokens/sec on M1 Max
//! - 13B models: >15 tokens/sec on M2 Max
//! - 70B models: >5 tokens/sec on M4 Max (with unified memory)

#[cfg(all(target_os = "macos", feature = "gguf"))]
mod metal_tests {
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

    /// Performance test: Measure tokens per second for inference.
    ///
    /// Collects the full metric set needed for issue #7:
    /// throughput, load time, GPU config, and hardware info.
    ///
    /// Requires a real model file. Set INFERNO_MODELS_DIR and ensure a
    /// valid .gguf model is present, then run:
    ///
    ///   cargo test --test metal_performance_tests test_inference_performance -- --ignored
    ///
    /// To contribute results to issue #7, use scripts/benchmark_metal.sh instead,
    /// which writes a structured JSON file you can share.
    #[tokio::test]
    #[ignore = "Requires real model file — see doc comment above"]
    async fn test_inference_performance() {
        use std::env;
        use sysinfo::{System, SystemExt};

        // ── Hardware / environment info ────────────────────────────────────
        let mut sys = System::new_all();
        sys.refresh_all();
        let hostname = sys.host_name().unwrap_or_else(|| "unknown".to_string());
        let total_memory_gb = sys.total_memory() as f64 / 1_073_741_824.0;

        println!("=== Metal GPU Performance Test ===");
        println!("Host:         {}", hostname);
        println!("Total RAM:    {:.1} GB", total_memory_gb);
        println!("Platform:     macOS (Apple Silicon)");

        // ── Locate model ────────────────────────────────────────────────────
        let models_dir =
            env::var("INFERNO_MODELS_DIR").unwrap_or_else(|_| "test_models".to_string());
        let model_path = PathBuf::from(&models_dir);

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

        let model_size_bytes = std::fs::metadata(&model_file)
            .map(|m| m.len())
            .unwrap_or(0);
        let model_size_mb = model_size_bytes as f64 / 1_048_576.0;
        println!("Model:        {}", model_file.display());
        println!("Model size:   {:.0} MB", model_size_mb);

        // ── Backend config ──────────────────────────────────────────────────
        let config = BackendConfig::with_metal_acceleration();
        println!("GPU enabled:  {}", config.gpu_enabled);
        println!("Context size: {}", config.context_size);
        println!("Batch size:   {}", config.batch_size);

        let mut backend =
            Backend::new(BackendType::Gguf, &config).expect("Failed to create GGUF backend");

        let model_metadata = std::fs::metadata(&model_file).ok();
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
            size: model_size_bytes,
            size_bytes: model_size_bytes,
            modified: model_modified,
            format: "gguf".to_string(),
            backend_type: "gguf".to_string(),
            checksum: None,
            metadata: Default::default(),
        };

        // ── Load model ──────────────────────────────────────────────────────
        let load_start = Instant::now();
        backend
            .load_model(&model_info)
            .await
            .expect("Failed to load model");
        let load_time = load_start.elapsed();
        println!("\nLoad time:    {:?}", load_time);

        // ── Inference benchmark ─────────────────────────────────────────────
        let params = InferenceParams {
            max_tokens: 100,
            temperature: 0.0, // Deterministic
            ..Default::default()
        };

        let prompt = "The quick brown fox";
        let num_runs = 5;
        let mut run_tokens = Vec::new();
        let mut run_times_ms = Vec::new();

        println!("\nRunning {} inference iterations...", num_runs);
        for i in 0..num_runs {
            let start = Instant::now();
            let result = backend.infer(prompt, &params).await;
            let elapsed = start.elapsed();
            let elapsed_ms = elapsed.as_millis() as u64;

            match result {
                Ok(output) => {
                    // Use word-count heuristic (~0.75 words per token for English)
                    let words = output.split_whitespace().count();
                    let tokens = ((words as f64) / 0.75).ceil() as u32;
                    let tps = tokens as f64 / elapsed.as_secs_f64();
                    run_tokens.push(tokens);
                    run_times_ms.push(elapsed_ms);
                    println!(
                        "  Run {:2}: {:>5} est. tokens  {:>7.1} ms  {:>7.1} tok/s",
                        i + 1,
                        tokens,
                        elapsed_ms,
                        tps
                    );
                }
                Err(e) => {
                    eprintln!("  Run {}: inference failed — {}", i + 1, e);
                }
            }
        }

        // ── Summary ─────────────────────────────────────────────────────────
        if !run_tokens.is_empty() {
            let total_tokens: u32 = run_tokens.iter().sum();
            let total_ms: u64 = run_times_ms.iter().sum();
            let avg_tps = (total_tokens as f64) / (total_ms as f64 / 1000.0);

            let mut sorted_ms = run_times_ms.clone();
            sorted_ms.sort_unstable();
            let min_ms = *sorted_ms.first().unwrap() as f64;
            let max_ms = *sorted_ms.last().unwrap() as f64;
            let median_ms = sorted_ms[sorted_ms.len() / 2] as f64;

            println!("\n=== Results ===");
            println!("Throughput:   {:.1} tok/s (average)", avg_tps);
            println!("Latency min:  {:.1} ms", min_ms);
            println!("Latency max:  {:.1} ms", max_ms);
            println!("Latency med:  {:.1} ms", median_ms);
            println!("Total tokens: {}", total_tokens);
            println!("Load time:    {:?}", load_time);
            println!(
                "\nGPU memory:   check Activity Monitor > GPU tab during inference"
            );
            println!(
                "Layer offload: check logs for 'offloaded X/X layers to GPU'"
            );
            println!(
                "\nTo contribute these results to issue #7:\n  https://github.com/ringo380/inferno/issues/7"
            );

            // Sanity check: warn if GPU doesn't appear to be active
            if avg_tps < 10.0 {
                eprintln!(
                    "\nWARNING: throughput {:.1} tok/s is below 10 tok/s. \
                     GPU acceleration may not be active — check logs for GPU offload confirmation.",
                    avg_tps
                );
            }
        }

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
