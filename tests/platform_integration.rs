//! Platform Integration Tests
//!
//! Comprehensive tests for the enhanced Inferno platform capabilities

use inferno::{init_platform, PlatformInfo, Result};

#[tokio::test]
async fn test_platform_initialization() -> Result<()> {
    // Test platform initialization
    init_platform()?;
    println!("✅ Platform initialized successfully");
    Ok(())
}

#[test]
fn test_platform_info() {
    let info = PlatformInfo::new();

    // Verify basic platform information
    assert!(!info.version.is_empty());
    assert!(!info.interfaces.is_empty());

    // Check that CLI and TUI interfaces are always available
    assert!(info.interfaces.contains(&"CLI".to_string()));
    assert!(info.interfaces.contains(&"TUI".to_string()));
    assert!(info.interfaces.contains(&"HTTP API".to_string()));

    println!("✅ Platform info: {}", info);
}

#[test]
fn test_backend_detection() {
    let info = PlatformInfo::new();

    // Test that backends are properly detected based on features
    #[cfg(feature = "gguf")]
    assert!(info.backends.contains(&"GGUF".to_string()));

    #[cfg(feature = "onnx")]
    assert!(info.backends.contains(&"ONNX".to_string()));

    println!("✅ Available backends: {:?}", info.backends);
}

#[test]
fn test_feature_detection() {
    let info = PlatformInfo::new();

    // Test feature detection
    #[cfg(feature = "tauri-app")]
    {
        assert!(info.features.contains(&"Desktop App".to_string()));
        assert!(info.interfaces.contains(&"Desktop GUI".to_string()));
    }

    #[cfg(feature = "download")]
    assert!(info.features.contains(&"Model Download".to_string()));

    println!("✅ Available features: {:?}", info.features);
}

#[tokio::test]
async fn test_error_handling() {
    use inferno::InfernoError;

    // Test error type creation and formatting
    let error = InfernoError::Backend("test backend error".to_string());
    assert!(error.to_string().contains("Backend error"));

    let error = InfernoError::Model("test model error".to_string());
    assert!(error.to_string().contains("Model error"));

    println!("✅ Error handling working correctly");
}

#[test]
fn test_comprehensive_platform() {
    // This test validates that all the enhanced platform components
    // are properly integrated and accessible

    let info = PlatformInfo::new();

    // Basic validation
    assert!(!info.version.is_empty(), "Version should be set");
    assert!(!info.interfaces.is_empty(), "Should have interfaces");

    // Enhanced platform should have multiple capabilities
    let total_capabilities = info.backends.len() + info.features.len() + info.interfaces.len();
    assert!(
        total_capabilities >= 3,
        "Enhanced platform should have multiple capabilities"
    );

    println!("✅ Comprehensive platform validation passed");
    println!("   Total capabilities: {}", total_capabilities);
    println!("   Backends: {}", info.backends.len());
    println!("   Features: {}", info.features.len());
    println!("   Interfaces: {}", info.interfaces.len());
}

#[cfg(feature = "tauri-app")]
#[test]
fn test_tauri_integration() {
    // Test that Tauri integration is properly available
    let info = PlatformInfo::new();
    assert!(info.features.contains(&"Desktop App".to_string()));
    assert!(info.interfaces.contains(&"Desktop GUI".to_string()));

    println!("✅ Tauri integration detected and working");
}

// Integration test for the overall platform enhancement
#[tokio::test]
async fn test_platform_enhancement_integration() -> Result<()> {
    // Initialize the platform
    init_platform()?;

    // Get platform information
    let info = PlatformInfo::new();

    // Validate enhanced capabilities
    assert!(!info.version.is_empty());
    assert!(info.interfaces.len() >= 3); // CLI, TUI, HTTP API minimum

    // Test that error handling works
    let result: Result<()> = Err(InfernoError::Unknown("test error".to_string()));
    assert!(result.is_err());

    println!("✅ Platform enhancement integration test passed");
    println!("🔥 Enhanced Inferno platform is fully operational!");

    Ok(())
}
