use crate::config::Config;
use crate::models::ModelManager;
use anyhow::Result;
use clap::Args;
use std::path::PathBuf;
use tracing::info;

#[derive(Args)]
pub struct ValidateArgs {
    #[arg(help = "Path to validate (model file, config file, or directory)")]
    pub path: PathBuf,

    #[arg(long, help = "Validate checksums")]
    pub checksum: bool,

    #[arg(long, help = "Deep validation (load and test model)")]
    pub deep: bool,

    #[arg(short, long, help = "Verbose output")]
    pub verbose: bool,
}

pub async fn execute(args: ValidateArgs, config: &Config) -> Result<()> {
    info!("Validating: {}", args.path.display());

    if !args.path.exists() {
        println!("✗ Path does not exist: {}", args.path.display());
        std::process::exit(1);
    }

    let mut validation_passed = true;

    if args.path.is_file() {
        validation_passed &= validate_file(&args.path, &args, config).await?;
    } else if args.path.is_dir() {
        validation_passed &= validate_directory(&args.path, &args, config).await?;
    }

    if validation_passed {
        println!("✓ All validations passed");
        Ok(())
    } else {
        println!("✗ Some validations failed");
        std::process::exit(1);
    }
}

async fn validate_file(path: &PathBuf, args: &ValidateArgs, config: &Config) -> Result<bool> {
    let mut passed = true;

    // Determine file type
    if let Some(ext) = path.extension() {
        match ext.to_str().unwrap_or("") {
            "gguf" | "onnx" => {
                passed &= validate_model_file(path, args, config).await?;
            }
            "toml" => {
                passed &= validate_config_file(path, args).await?;
            }
            _ => {
                if args.verbose {
                    println!("ℹ Unknown file type, performing basic validation");
                }
                passed &= validate_basic_file(path, args).await?;
            }
        }
    } else {
        passed &= validate_basic_file(path, args).await?;
    }

    Ok(passed)
}

async fn validate_directory(path: &PathBuf, args: &ValidateArgs, config: &Config) -> Result<bool> {
    let mut passed = true;
    let mut model_count = 0;

    println!("Validating directory: {}", path.display());

    let mut entries = tokio::fs::read_dir(path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        if entry_path.is_file() {
            if let Some(ext) = entry_path.extension() {
                if matches!(ext.to_str().unwrap_or(""), "gguf" | "onnx") {
                    model_count += 1;
                    if args.verbose {
                        println!("  Validating model: {}", entry_path.display());
                    }
                    passed &= validate_model_file(&entry_path, args, config).await?;
                }
            }
        }
    }

    if model_count == 0 {
        println!("ℹ No model files found in directory");
    } else {
        println!("✓ Validated {} model files", model_count);
    }

    Ok(passed)
}

async fn validate_model_file(path: &PathBuf, args: &ValidateArgs, config: &Config) -> Result<bool> {
    let mut passed = true;
    let model_manager = ModelManager::new(&config.models_dir);

    if args.verbose {
        println!("Validating model file: {}", path.display());
    }

    // Basic file validation
    let metadata = tokio::fs::metadata(path).await?;
    if metadata.len() == 0 {
        println!("✗ Model file is empty: {}", path.display());
        return Ok(false);
    }

    // Check if file is readable
    match tokio::fs::File::open(path).await {
        Ok(_) => {
            if args.verbose {
                println!("  ✓ File is readable");
            }
        }
        Err(e) => {
            println!("✗ Cannot read file: {} ({})", path.display(), e);
            return Ok(false);
        }
    }

    // Format-specific validation
    if let Some(ext) = path.extension() {
        match ext.to_str().unwrap_or("") {
            "gguf" => {
                passed &= validate_gguf_file(path, &model_manager, args).await?;
            }
            "onnx" => {
                passed &= validate_onnx_file(path, &model_manager, args).await?;
            }
            _ => {}
        }
    }

    // Checksum validation
    if args.checksum {
        if args.verbose {
            println!("  Computing SHA256 checksum...");
        }
        let checksum = model_manager.compute_checksum(path).await?;
        if args.verbose {
            println!("  ✓ SHA256: {}", checksum);
        }
    }

    // Use comprehensive validation instead of the old method
    let validation_result = model_manager
        .validate_model_comprehensive(path, Some(config))
        .await?;

    // Print validation results
    if validation_result.is_valid {
        if !args.verbose {
            println!("✓ Model is valid: {}", path.display());
        }
        if args.verbose {
            print_validation_details(&validation_result, true);
        }
    } else {
        println!("✗ Model validation failed: {}", path.display());
        print_validation_details(&validation_result, args.verbose);
        passed = false;
    }

    // Deep validation (actually load the model)
    if args.deep && validation_result.is_valid {
        if args.verbose {
            println!("  Performing deep validation...");
        }
        passed &= deep_validate_model(path, config).await?;
    }

    Ok(passed)
}

fn print_validation_details(result: &crate::models::ValidationResult, verbose: bool) {
    if verbose {
        println!("  Validation Details:");
        println!(
            "    File readable: {}",
            if result.file_readable { "✓" } else { "✗" }
        );
        println!(
            "    Format valid: {}",
            if result.format_valid { "✓" } else { "✗" }
        );
        println!(
            "    Size valid: {}",
            if result.size_valid { "✓" } else { "✗" }
        );
        println!(
            "    Security valid: {}",
            if result.security_valid { "✓" } else { "✗" }
        );
        println!(
            "    Metadata valid: {}",
            if result.metadata_valid { "✓" } else { "✗" }
        );
        if let Some(checksum_valid) = result.checksum_valid {
            println!(
                "    Checksum valid: {}",
                if checksum_valid { "✓" } else { "✗" }
            );
        }
    }

    for error in &result.errors {
        println!("    ✗ Error: {}", error);
    }

    for warning in &result.warnings {
        println!("    ⚠ Warning: {}", warning);
    }
}

async fn validate_gguf_file(
    path: &PathBuf,
    model_manager: &ModelManager,
    args: &ValidateArgs,
) -> Result<bool> {
    match model_manager.get_gguf_metadata(path).await {
        Ok(metadata) => {
            if args.verbose {
                println!("  ✓ Valid GGUF file");
                println!("    Architecture: {}", metadata.architecture);
                println!("    Parameters: {}", metadata.parameter_count);
                println!("    Quantization: {}", metadata.quantization);
            }
            Ok(true)
        }
        Err(e) => {
            println!("✗ Invalid GGUF file: {} ({})", path.display(), e);
            Ok(false)
        }
    }
}

async fn validate_onnx_file(
    path: &PathBuf,
    model_manager: &ModelManager,
    args: &ValidateArgs,
) -> Result<bool> {
    match model_manager.get_onnx_metadata(path).await {
        Ok(metadata) => {
            if args.verbose {
                println!("  ✓ Valid ONNX file");
                println!("    Version: {}", metadata.version);
                println!("    Producer: {}", metadata.producer);
                println!("    Inputs: {}", metadata.input_count);
                println!("    Outputs: {}", metadata.output_count);
            }
            Ok(true)
        }
        Err(e) => {
            println!("✗ Invalid ONNX file: {} ({})", path.display(), e);
            Ok(false)
        }
    }
}

async fn validate_config_file(path: &PathBuf, args: &ValidateArgs) -> Result<bool> {
    if args.verbose {
        println!("Validating config file: {}", path.display());
    }

    let content = tokio::fs::read_to_string(path).await?;
    match toml::from_str::<toml::Value>(&content) {
        Ok(_) => {
            if args.verbose {
                println!("  ✓ Valid TOML syntax");
            }
            Ok(true)
        }
        Err(e) => {
            println!("✗ Invalid TOML file: {} ({})", path.display(), e);
            Ok(false)
        }
    }
}

async fn validate_basic_file(path: &PathBuf, args: &ValidateArgs) -> Result<bool> {
    if args.verbose {
        println!("Validating file: {}", path.display());
    }

    let metadata = tokio::fs::metadata(path).await?;

    if args.verbose {
        println!("  ✓ File exists");
        println!("  ✓ Size: {} bytes", metadata.len());
        println!("  ✓ Modified: {:?}", metadata.modified()?);
    }

    Ok(true)
}

async fn deep_validate_model(path: &PathBuf, config: &Config) -> Result<bool> {
    use crate::backends::{Backend, BackendType};
    use crate::models::ModelInfo;

    let backend_type = BackendType::from_model_path(path).ok_or_else(|| {
        anyhow::anyhow!("No suitable backend found for model: {}", path.display())
    })?;
    let mut backend = Backend::new(backend_type, &config.backend_config)?;

    let model_info = ModelInfo {
        name: path.file_name().unwrap().to_string_lossy().to_string(),
        path: path.clone(),
        file_path: path.clone(),
        size: tokio::fs::metadata(path).await?.len(),
        size_bytes: tokio::fs::metadata(path).await?.len(),
        modified: chrono::DateTime::from(tokio::fs::metadata(path).await?.modified()?),
        backend_type: backend_type.to_string(),
        format: path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_string(),
        checksum: None,
        metadata: std::collections::HashMap::new(),
    };

    match backend.load_model(&model_info).await {
        Ok(_) => {
            println!("  ✓ Model loads successfully");

            // Test a simple inference
            let test_input = "Hello";
            let inference_params = crate::backends::InferenceParams {
                max_tokens: 10,
                temperature: 0.7,
                top_k: 40,
                top_p: 0.9,
                stream: false,
                stop_sequences: vec![],
                seed: None,
            };

            match backend.infer(test_input, &inference_params).await {
                Ok(_) => {
                    println!("  ✓ Model inference works");
                    Ok(true)
                }
                Err(e) => {
                    println!("  ✗ Model inference failed: {}", e);
                    Ok(false)
                }
            }
        }
        Err(e) => {
            println!("  ✗ Model failed to load: {}", e);
            Ok(false)
        }
    }
}
