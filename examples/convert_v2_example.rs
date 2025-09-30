//! Convert Command v2 Example
//!
//! Demonstrates the new CLI architecture for the convert command.
//! Shows model conversion, quantization, and analysis operations.
//!
//! Run with: cargo run --example convert_v2_example

use anyhow::Result;
use inferno::cli::convert_v2::{AnalyzeModel, ConvertModel, QuantizeModel};
use inferno::config::Config;
use inferno::conversion::{ModelFormat, OptimizationLevel, Precision, QuantizationType};
use inferno::core::config::ConfigBuilder;
use inferno::interfaces::cli::{
    CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Convert Command v2 Examples\n");

    // Create configuration
    let config = Config::default();
    let _core_config = ConfigBuilder::new().build_unchecked();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Model conversion
    // ========================================================================
    println!("Example 1: Model Conversion");
    println!("{}", "─".repeat(80));
    println!("Note: This example requires actual model files to run.");
    println!("Usage example:");
    println!("  let convert_cmd = ConvertModel::new(");
    println!("      config.clone(),");
    println!("      PathBuf::from(\"model.gguf\"),       // input");
    println!("      PathBuf::from(\"model.onnx\"),       // output");
    println!("      ModelFormat::Onnx,                // target format");
    println!("      OptimizationLevel::Balanced,      // optimization");
    println!("      None,                             // quantization");
    println!("      Some(Precision::Float16),         // precision");
    println!("      Some(2048),                       // context length");
    println!("      Some(32),                         // batch size");
    println!("      true,                             // preserve metadata");
    println!("      true,                             // verify output");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  Converting model: model.gguf -> model.onnx");
    println!("  Target format: Onnx");
    println!("  Optimization: Balanced");
    println!("  ✓ Conversion completed successfully!");
    println!("    Input size: 4256.34 MB");
    println!("    Output size: 3847.21 MB");
    println!("    Compression ratio: 1.11x");
    println!("    Conversion time: 2m 34s");
    println!("    Metadata preserved: true");

    println!("\n");

    // ========================================================================
    // Example 2: Validation examples
    // ========================================================================
    println!("Example 2: Input Validation");
    println!("{}", "─".repeat(80));

    // Test with non-existent file
    let nonexistent = ConvertModel::new(
        config.clone(),
        PathBuf::from("/nonexistent/model.gguf"),
        PathBuf::from("/tmp/output.onnx"),
        ModelFormat::Onnx,
        OptimizationLevel::Balanced,
        None,
        None,
        None,
        None,
        true,
        true,
    );
    let ctx_nonexistent = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(nonexistent), &mut ctx_nonexistent.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation correctly caught missing input file:");
            println!("  {}", e);
        }
    }

    println!();

    // Test with invalid context length
    let temp_file = std::env::temp_dir().join("test_model.gguf");
    std::fs::write(&temp_file, b"test")?;

    let invalid_context = ConvertModel::new(
        config.clone(),
        temp_file.clone(),
        PathBuf::from("/tmp/output.onnx"),
        ModelFormat::Onnx,
        OptimizationLevel::Balanced,
        None,
        None,
        Some(0), // Invalid - zero context length
        None,
        true,
        true,
    );

    match pipeline
        .execute(Box::new(invalid_context), &mut ctx_nonexistent.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation correctly caught zero context length:");
            println!("  {}", e);
        }
    }

    println!();

    // Test with excessive context length
    let excessive_context = ConvertModel::new(
        config.clone(),
        temp_file.clone(),
        PathBuf::from("/tmp/output.onnx"),
        ModelFormat::Onnx,
        OptimizationLevel::Balanced,
        None,
        None,
        Some(50000), // Invalid - exceeds 32768 limit
        None,
        true,
        true,
    );

    match pipeline
        .execute(Box::new(excessive_context), &mut ctx_nonexistent.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation correctly caught excessive context length:");
            println!("  {}", e);
        }
    }

    // Cleanup
    let _ = std::fs::remove_file(&temp_file);

    println!("\n");

    // ========================================================================
    // Example 3: Model quantization
    // ========================================================================
    println!("Example 3: Model Quantization");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let quantize_cmd = QuantizeModel::new(");
    println!("      config,");
    println!("      PathBuf::from(\"model-f32.gguf\"),    // input");
    println!("      PathBuf::from(\"model-q4_0.gguf\"),   // output");
    println!("      QuantizationType::Q4_0,            // quantization");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  Quantizing model: model-f32.gguf -> model-q4_0.gguf");
    println!("  Quantization type: Q4_0");
    println!("  ✓ Quantization completed successfully!");
    println!("    Input size: 13542.78 MB");
    println!("    Output size: 3912.45 MB");
    println!("    Size reduction: 71.1%");
    println!("    Quantization time: 5m 12s");

    println!("\n");

    // ========================================================================
    // Example 4: Quantization types
    // ========================================================================
    println!("Example 4: Available Quantization Types");
    println!("{}", "─".repeat(80));
    println!("Quantization types (ordered by size vs. quality trade-off):");
    println!();
    println!("  Q4_0  - 4-bit, smallest size, fastest, lower quality");
    println!("  Q4_1  - 4-bit with offset, better quality than Q4_0");
    println!("  Q5_0  - 5-bit, balanced size/quality");
    println!("  Q5_1  - 5-bit with offset, better quality");
    println!("  Q8_0  - 8-bit, larger size, high quality");
    println!("  F16   - 16-bit float, very large, excellent quality");
    println!("  F32   - 32-bit float, original size, best quality");
    println!("  Int8  - 8-bit integer quantization");
    println!("  Int16 - 16-bit integer quantization");
    println!();
    println!("Typical size reductions:");
    println!("  F32 → Q4_0: ~75% reduction");
    println!("  F32 → Q5_0: ~65% reduction");
    println!("  F32 → Q8_0: ~50% reduction");
    println!("  F32 → F16:  ~50% reduction");

    println!("\n");

    // ========================================================================
    // Example 5: Model analysis
    // ========================================================================
    println!("Example 5: Model Analysis");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let analyze_cmd = AnalyzeModel::new(");
    println!("      config,");
    println!("      PathBuf::from(\"model.gguf\"),");
    println!("      false,  // detailed mode");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  Analyzing model: model.gguf");
    println!();
    println!("  === Model Information ===");
    println!("  Name: llama-2-7b-chat.Q4_0.gguf");
    println!("  Path: /models/llama-2-7b-chat.Q4_0.gguf");
    println!("  Size: 3825.47 MB");
    println!("  Modified: 2024-03-15 14:23:17 UTC");
    println!("  Backend: gguf");
    println!("  Checksum: a3f8b2c1d4e5...");
    println!();
    println!("  === Validation Results ===");
    println!("  Valid: true");
    println!("  File readable: true");
    println!("  Format valid: true");
    println!("  Size valid: true");
    println!("  Security valid: true");
    println!("  Metadata valid: true");
    println!("  Checksum valid: true");

    println!("\n");

    // ========================================================================
    // Example 6: Detailed analysis
    // ========================================================================
    println!("Example 6: Detailed Model Analysis");
    println!("{}", "─".repeat(80));
    println!("With detailed=true, additional metadata is shown:");
    println!();
    println!("For GGUF models:");
    println!("  Architecture: llama");
    println!("  Parameters: 6.7B");
    println!("  Quantization: Q4_0");
    println!("  Context length: 4096");
    println!();
    println!("For ONNX models:");
    println!("  ONNX version: 1.14.0");
    println!("  Producer: pytorch");
    println!("  Input count: 3");
    println!("  Output count: 1");
    println!();
    println!("If checksum not cached:");
    println!("  Computing checksum...");
    println!("  SHA256: a3f8b2c1d4e5f6a7b8c9d0e1f2a3b4c5...");

    println!("\n");

    // ========================================================================
    // Example 7: Optimization levels
    // ========================================================================
    println!("Example 7: Optimization Levels");
    println!("{}", "─".repeat(80));
    println!("Available optimization levels:");
    println!();
    println!("  None       - No optimization, fastest conversion");
    println!("  Basic      - Remove unused layers");
    println!("  Balanced   - Basic + merge operations (default)");
    println!("  Aggressive - Balanced + constant folding + dead code elimination");
    println!("  Maximum    - All optimizations + operator fusion");
    println!();
    println!("Trade-offs:");
    println!("  - Higher optimization = longer conversion time");
    println!("  - Higher optimization = smaller output size");
    println!("  - Higher optimization = faster inference");
    println!("  - Maximum may increase compatibility risk");

    println!("\n");

    // ========================================================================
    // Example 8: Precision options
    // ========================================================================
    println!("Example 8: Target Precision");
    println!("{}", "─".repeat(80));
    println!("Available precision options:");
    println!();
    println!("  Float32 - 32-bit floating point (highest precision)");
    println!("  Float16 - 16-bit floating point (good balance)");
    println!("  Int8    - 8-bit integer (smallest, requires calibration)");
    println!("  Int16   - 16-bit integer");
    println!("  Mixed   - Mixed precision (automatic selection)");
    println!();
    println!("When to use:");
    println!("  Float32: Maximum accuracy required");
    println!("  Float16: Good balance for GPU inference");
    println!("  Int8:    Edge devices, mobile deployment");
    println!("  Mixed:   Let converter optimize automatically");

    println!("\n");

    // ========================================================================
    // Example 9: Format support
    // ========================================================================
    println!("Example 9: Supported Formats");
    println!("{}", "─".repeat(80));
    println!("Input/Output formats:");
    println!();
    println!("  GGUF         - GGML Universal Format (llama.cpp)");
    println!("  ONNX         - Open Neural Network Exchange");
    println!("  SafeTensors  - Safe serialization format");
    println!("  PyTorch      - PyTorch native format (.pt, .pth)");
    println!("  TensorFlow   - TensorFlow SavedModel");
    println!();
    println!("Common conversion paths:");
    println!("  PyTorch → GGUF:        For llama.cpp inference");
    println!("  GGUF → ONNX:          For ONNX Runtime deployment");
    println!("  TensorFlow → ONNX:    For cross-platform deployment");
    println!("  PyTorch → SafeTensors: For secure model distribution");

    println!("\n");

    // ========================================================================
    // Example 10: JSON output
    // ========================================================================
    println!("Example 10: JSON Output Mode");
    println!("{}", "─".repeat(80));
    println!("With json_output=true, structured data is returned:");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "success": true,
            "input_path": "model.gguf",
            "output_path": "model.onnx",
            "input_size": 4256340000_u64,
            "output_size": 3847210000_u64,
            "compression_ratio": 1.11,
            "conversion_time_ms": 154000,
            "metadata_preserved": true,
            "warnings": []
        }))?
    );

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Convert Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Model format conversion (GGUF, ONNX, PyTorch, TensorFlow, SafeTensors)");
    println!("✓ Model quantization with multiple types (Q4_0 to F32)");
    println!("✓ Comprehensive model analysis and validation");
    println!("✓ Configurable optimization levels (None to Maximum)");
    println!("✓ Target precision control (Float32, Float16, Int8, Mixed)");
    println!("✓ Context length and batch size configuration");
    println!("✓ Metadata preservation option");
    println!("✓ Output verification");
    println!("✓ Input validation (paths, parameters, ranges)");
    println!("✓ Structured JSON output");
    println!("✓ Human-readable progress and results");
    println!("✓ Middleware support (logging, metrics)");
    println!();
    println!("Validation Checks:");
    println!("  - Input file existence and readability");
    println!("  - Output directory existence");
    println!("  - Context length range (1-32768)");
    println!("  - Batch size range (1-1024)");
    println!("  - File format validation");
    println!();
    println!("Use Cases:");
    println!("  - Convert models between inference frameworks");
    println!("  - Reduce model size through quantization");
    println!("  - Optimize models for production deployment");
    println!("  - Verify model integrity and format");
    println!("  - Analyze model characteristics and metadata");
    println!();
    println!("Note: This is a focused migration covering core conversion operations.");
    println!("Full conversion functionality (optimize, batch, benchmark) remains");
    println!("available through the original convert module.");

    Ok(())
}
