use inferno::conversion::{ModelConverter, ConversionConfig, ModelFormat, QuantizationType, OptimizationLevel};
use inferno::models::ModelManager;
use inferno::config::Config;
use std::sync::Arc;
use std::path::Path;
use tempfile::tempdir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Inferno Model Conversion Test");

    // Create a temporary directory for test files
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path();

    // Create mock model files for testing
    let gguf_path = temp_path.join("test_model.gguf");
    let safetensors_path = temp_path.join("test_model.safetensors");
    let output_path = temp_path.join("converted_model.onnx");

    // Create a mock GGUF file
    create_mock_gguf_file(&gguf_path).await?;

    // Create a mock SafeTensors file
    create_mock_safetensors_file(&safetensors_path).await?;

    // Initialize the conversion system
    let config = Config::default();
    let model_manager = Arc::new(ModelManager::new(config.clone()));
    let converter = ModelConverter::new(model_manager, config);

    // Test conversion configuration
    let conversion_config = ConversionConfig {
        output_format: ModelFormat::Onnx,
        optimization_level: OptimizationLevel::Balanced,
        quantization: Some(QuantizationType::F16),
        target_precision: None,
        context_length: None,
        batch_size: None,
        preserve_metadata: true,
        verify_output: true,
    };

    println!("Testing GGUF to ONNX conversion...");

    // Test the conversion
    match converter.convert_model(&gguf_path, &output_path, &conversion_config).await {
        Ok(result) => {
            println!("Conversion completed!");
            println!("Success: {}", result.success);
            println!("Input size: {} bytes", result.input_size);
            println!("Output size: {} bytes", result.output_size);
            println!("Compression ratio: {:.2}", result.compression_ratio);
            println!("Conversion time: {:?}", result.conversion_time);

            if !result.warnings.is_empty() {
                println!("Warnings:");
                for warning in &result.warnings {
                    println!("  - {}", warning);
                }
            }

            if !result.errors.is_empty() {
                println!("Errors:");
                for error in &result.errors {
                    println!("  - {}", error);
                }
            }
        }
        Err(e) => {
            eprintln!("Conversion failed: {}", e);
        }
    }

    println!("\nTesting SafeTensors to GGUF conversion...");

    let safetensors_to_gguf_config = ConversionConfig {
        output_format: ModelFormat::Gguf,
        optimization_level: OptimizationLevel::Basic,
        quantization: None,
        target_precision: None,
        context_length: None,
        batch_size: None,
        preserve_metadata: true,
        verify_output: true,
    };

    let gguf_output_path = temp_path.join("converted_model.gguf");

    match converter.convert_model(&safetensors_path, &gguf_output_path, &safetensors_to_gguf_config).await {
        Ok(result) => {
            println!("SafeTensors conversion completed!");
            println!("Success: {}", result.success);
            println!("Input size: {} bytes", result.input_size);
            println!("Output size: {} bytes", result.output_size);
            println!("Compression ratio: {:.2}", result.compression_ratio);

            if !result.warnings.is_empty() {
                println!("Warnings:");
                for warning in &result.warnings {
                    println!("  - {}", warning);
                }
            }
        }
        Err(e) => {
            eprintln!("SafeTensors conversion failed: {}", e);
        }
    }

    println!("\nConversion test completed!");

    Ok(())
}

async fn create_mock_gguf_file(path: &Path) -> anyhow::Result<()> {
    use tokio::fs;
    use byteorder::{LittleEndian, WriteBytesExt};

    let mut data = Vec::new();

    // Write GGUF magic number
    data.extend_from_slice(b"GGUF");

    // Write version (3)
    data.write_u32::<LittleEndian>(3)?;

    // Write tensor count (1)
    data.write_u64::<LittleEndian>(1)?;

    // Write metadata count (2)
    data.write_u64::<LittleEndian>(2)?;

    // Write metadata: general.architecture
    let arch_key = "general.architecture";
    data.write_u64::<LittleEndian>(arch_key.len() as u64)?;
    data.extend_from_slice(arch_key.as_bytes());
    data.write_u32::<LittleEndian>(8)?; // String type
    let arch_value = "llama";
    data.write_u64::<LittleEndian>(arch_value.len() as u64)?;
    data.extend_from_slice(arch_value.as_bytes());

    // Write metadata: llama.vocab_size
    let vocab_key = "llama.vocab_size";
    data.write_u64::<LittleEndian>(vocab_key.len() as u64)?;
    data.extend_from_slice(vocab_key.as_bytes());
    data.write_u32::<LittleEndian>(10)?; // Uint64 type
    data.write_u64::<LittleEndian>(32000)?; // Vocab size value

    // Write tensor info: test_tensor
    let tensor_name = "test_tensor";
    data.write_u64::<LittleEndian>(tensor_name.len() as u64)?;
    data.extend_from_slice(tensor_name.as_bytes());
    data.write_u32::<LittleEndian>(2)?; // 2 dimensions
    data.write_u64::<LittleEndian>(10)?; // Dimension 1
    data.write_u64::<LittleEndian>(20)?; // Dimension 2
    data.write_u32::<LittleEndian>(0)?; // F32 type
    data.write_u64::<LittleEndian>(0)?; // Offset

    // Align to 32-byte boundary
    let aligned_size = (data.len() + 31) & !31;
    data.resize(aligned_size, 0);

    // Add some mock tensor data (10 * 20 * 4 bytes = 800 bytes of F32 data)
    for i in 0..(10 * 20) {
        data.write_f32::<LittleEndian>(i as f32 / 100.0)?;
    }

    fs::write(path, data).await?;
    println!("Created mock GGUF file: {}", path.display());

    Ok(())
}

async fn create_mock_safetensors_file(path: &Path) -> anyhow::Result<()> {
    use tokio::fs;
    use safetensors::{serialize, Dtype};
    use std::collections::HashMap;

    // Create mock tensor data
    let mut tensors = HashMap::new();

    // Create a simple 2x3 F32 tensor
    let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
    let bytes = data.iter()
        .flat_map(|f| f.to_le_bytes())
        .collect::<Vec<u8>>();

    tensors.insert("test_tensor".to_string(), (Dtype::F32, vec![2, 3], bytes));

    // Serialize to SafeTensors format
    let serialized = serialize(&tensors, &None)?;

    fs::write(path, serialized).await?;
    println!("Created mock SafeTensors file: {}", path.display());

    Ok(())
}