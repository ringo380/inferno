/// Error Enum Size Analysis
///
/// This test analyzes the size of InfernoError variants to identify
/// which variants should be boxed to reduce the overall enum size.
///
/// Target: Reduce from ~208 bytes to <64 bytes
use inferno::InfernoError;
use std::mem::size_of;

#[test]
fn analyze_error_sizes() {
    println!("\n=== InfernoError Size Analysis ===");
    println!("Overall enum size: {} bytes", size_of::<InfernoError>());
    println!("\nComponent sizes:");
    println!("  String: {} bytes", size_of::<String>());
    println!("  std::io::Error: {} bytes", size_of::<std::io::Error>());
    println!(
        "  serde_json::Error: {} bytes",
        size_of::<serde_json::Error>()
    );
    println!("  figment::Error: {} bytes", size_of::<figment::Error>());

    // Calculate expected enum size (largest variant + discriminant)
    let max_variant_size = size_of::<figment::Error>()
        .max(size_of::<std::io::Error>())
        .max(size_of::<serde_json::Error>())
        .max(size_of::<String>());

    println!("\nLargest variant: {} bytes", max_variant_size);
    println!(
        "Expected enum size (variant + discriminant): ~{} bytes",
        max_variant_size + 8
    );

    // Recommendations
    println!("\n=== Optimization Recommendations ===");
    if size_of::<figment::Error>() > 64 {
        println!(
            "⚠️  Box figment::Error ({} bytes)",
            size_of::<figment::Error>()
        );
    }
    if size_of::<std::io::Error>() > 64 {
        println!(
            "⚠️  Box std::io::Error ({} bytes)",
            size_of::<std::io::Error>()
        );
    }
    if size_of::<serde_json::Error>() > 64 {
        println!(
            "⚠️  Box serde_json::Error ({} bytes)",
            size_of::<serde_json::Error>()
        );
    }

    let target_size = 64;
    if size_of::<InfernoError>() > target_size {
        println!(
            "\n⚠️  Current size ({} bytes) exceeds target ({} bytes)",
            size_of::<InfernoError>(),
            target_size
        );
        println!(
            "   Reduction needed: {} bytes",
            size_of::<InfernoError>() - target_size
        );
    } else {
        println!(
            "\n✅ Size ({} bytes) within target ({} bytes)",
            size_of::<InfernoError>(),
            target_size
        );
    }
}

#[test]
fn measure_boxed_error_sizes() {
    println!("\n=== Boxed Error Sizes ===");
    println!(
        "Box<figment::Error>: {} bytes",
        size_of::<Box<figment::Error>>()
    );
    println!(
        "Box<std::io::Error>: {} bytes",
        size_of::<Box<std::io::Error>>()
    );
    println!(
        "Box<serde_json::Error>: {} bytes",
        size_of::<Box<serde_json::Error>>()
    );
    println!("\nNote: Box is typically 8 bytes (pointer) on 64-bit systems");
}
