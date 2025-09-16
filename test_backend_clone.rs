use std::path::PathBuf;
use tempfile::tempdir;

mod src {
    pub mod lib;
    pub mod models;
    pub mod backends {
        pub mod mod;
        pub mod gguf;
        pub mod onnx;
    }
    pub mod cache;
}

use src::backends::{BackendHandle, BackendType, BackendConfig};
use src::models::{ModelInfo, ModelManager};
use src::cache::CachedModel;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Test that BackendHandle can be cloned
    let config = BackendConfig::default();
    let backend1 = BackendHandle::new_shared(BackendType::Gguf, &config)?;
    let backend2 = backend1.clone(); // This should work now!

    println!("✓ BackendHandle cloning works!");

    // Test that CachedModel can be cloned
    let temp_dir = tempdir()?;
    let model_file = temp_dir.path().join("test.gguf");
    std::fs::write(&model_file, "GGUF")?; // Mock GGUF file

    let model_info = ModelInfo {
        name: "test_model".to_string(),
        path: model_file,
        size: 4,
        backend_type: BackendType::Gguf,
        loaded: false,
        checksum: None,
        metadata: std::collections::HashMap::new(),
    };

    let cached_model = CachedModel {
        backend: backend1,
        model_info,
        last_used: std::time::Instant::now(),
        created_at: std::time::Instant::now(),
        usage_count: std::sync::atomic::AtomicU64::new(0),
        memory_estimate: 1024,
        warmup_priority: 1,
    };

    let cached_model_clone = cached_model.clone(); // This should work now!

    println!("✓ CachedModel cloning works!");
    println!("🎉 Backend cloning architecture successfully implemented!");

    Ok(())
}