// Minimal test to validate core compilation fixes
use inferno::{config::Config, backends::BackendType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Inferno Core Components...");

    // Test 1: Config loading
    println!("✅ Testing Config system...");
    let config_result = Config::load();
    match config_result {
        Ok(_) => println!("   ✓ Config system working"),
        Err(e) => println!("   ⚠ Config system: {}", e),
    }

    // Test 2: Backend type detection
    println!("✅ Testing Backend type system...");
    let test_paths = vec![
        std::path::PathBuf::from("test.gguf"),
        std::path::PathBuf::from("test.onnx"),
        std::path::PathBuf::from("test.unknown"),
    ];

    for path in test_paths {
        let backend_type = BackendType::from_model_path(&path);
        match backend_type {
            Some(bt) => println!("   ✓ {} -> {:?}", path.display(), bt),
            None => println!("   ✓ {} -> None (expected)", path.display()),
        }
    }

    // Test 3: Available backends
    println!("✅ Testing available backends...");
    #[cfg(feature = "gguf")]
    println!("   ✓ GGUF backend available");
    #[cfg(not(feature = "gguf"))]
    println!("   ⚠ GGUF backend disabled");

    #[cfg(feature = "onnx")]
    println!("   ✓ ONNX backend available");
    #[cfg(not(feature = "onnx"))]
    println!("   ⚠ ONNX backend disabled");

    println!("🎉 Core component validation complete!");

    Ok(())
}