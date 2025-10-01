use crate::{
    config::Config,
    conversion::{
        ConversionConfig, ModelConverter, ModelFormat, OptimizationLevel, OptimizationOptions,
        Precision, QuantizationType,
    },
    models::ModelManager,
};
use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
use std::{path::PathBuf, sync::Arc};
use tracing::warn;

/// Configuration for model conversion operations
/// Reduces function signature from 11 parameters to 2
pub struct ConvertModelConfig {
    pub input: PathBuf,
    pub output: PathBuf,
    pub format: ModelFormatArg,
    pub optimization: OptimizationLevelArg,
    pub quantization: Option<QuantizationTypeArg>,
    pub precision: Option<PrecisionArg>,
    pub context_length: Option<u32>,
    pub batch_size: Option<u32>,
    pub preserve_metadata: bool,
    pub verify_output: bool,
}

impl ConvertModelConfig {
    /// Convert to internal ConversionConfig type
    fn into_conversion_config(self) -> ConversionConfig {
        ConversionConfig {
            output_format: self.format.into(),
            optimization_level: self.optimization.into(),
            quantization: self.quantization.map(Into::into),
            target_precision: self.precision.map(Into::into),
            context_length: self.context_length,
            batch_size: self.batch_size,
            preserve_metadata: self.preserve_metadata,
            verify_output: self.verify_output,
        }
    }
}

/// Configuration for model optimization operations
/// Reduces function signature from 11 parameters to 2
pub struct OptimizeModelConfig {
    pub input: PathBuf,
    pub output: PathBuf,
    pub remove_unused: bool,
    pub merge_ops: bool,
    pub constant_folding: bool,
    pub dead_code: bool,
    pub memory_opt: bool,
    pub inference_opt: bool,
    pub graph_simplify: bool,
    pub operator_fusion: bool,
}

impl OptimizeModelConfig {
    /// Convert to internal OptimizationOptions type
    fn into_optimization_options(self) -> OptimizationOptions {
        OptimizationOptions {
            remove_unused_layers: self.remove_unused,
            merge_consecutive_ops: self.merge_ops,
            constant_folding: self.constant_folding,
            dead_code_elimination: self.dead_code,
            memory_optimization: self.memory_opt,
            inference_optimization: self.inference_opt,
            graph_simplification: self.graph_simplify,
            operator_fusion: self.operator_fusion,
        }
    }
}

#[derive(Args)]
pub struct ConvertArgs {
    #[command(subcommand)]
    pub command: ConvertCommand,
}

#[derive(Subcommand)]
pub enum ConvertCommand {
    #[command(about = "Convert model between different formats")]
    Model {
        #[arg(help = "Input model path")]
        input: PathBuf,

        #[arg(help = "Output model path")]
        output: PathBuf,

        #[arg(long, help = "Target format", value_enum)]
        format: ModelFormatArg,

        #[arg(
            long,
            help = "Optimization level",
            value_enum,
            default_value = "balanced"
        )]
        optimization: OptimizationLevelArg,

        #[arg(long, help = "Quantization type", value_enum)]
        quantization: Option<QuantizationTypeArg>,

        #[arg(long, help = "Target precision", value_enum)]
        precision: Option<PrecisionArg>,

        #[arg(long, help = "Context length")]
        context_length: Option<u32>,

        #[arg(long, help = "Batch size")]
        batch_size: Option<u32>,

        #[arg(long, help = "Preserve metadata")]
        preserve_metadata: bool,

        #[arg(long, help = "Skip output verification")]
        no_verify: bool,
    },

    #[command(about = "Optimize model for better performance")]
    Optimize {
        #[arg(help = "Input model path")]
        input: PathBuf,

        #[arg(help = "Output model path")]
        output: PathBuf,

        #[arg(long, help = "Remove unused layers")]
        remove_unused: bool,

        #[arg(long, help = "Merge consecutive operations")]
        merge_ops: bool,

        #[arg(long, help = "Apply constant folding")]
        constant_folding: bool,

        #[arg(long, help = "Dead code elimination")]
        dead_code: bool,

        #[arg(long, help = "Memory optimization")]
        memory_opt: bool,

        #[arg(long, help = "Inference optimization")]
        inference_opt: bool,

        #[arg(long, help = "Graph simplification")]
        graph_simplify: bool,

        #[arg(long, help = "Operator fusion")]
        operator_fusion: bool,
    },

    #[command(about = "Quantize model to reduce size")]
    Quantize {
        #[arg(help = "Input model path")]
        input: PathBuf,

        #[arg(help = "Output model path")]
        output: PathBuf,

        #[arg(long, help = "Quantization type", value_enum)]
        quantization: QuantizationTypeArg,
    },

    #[command(about = "Batch convert models in a directory")]
    Batch {
        #[arg(help = "Input directory")]
        input_dir: PathBuf,

        #[arg(help = "Output directory")]
        output_dir: PathBuf,

        #[arg(long, help = "Target format", value_enum)]
        format: ModelFormatArg,

        #[arg(long, help = "File pattern filter")]
        pattern: Option<String>,

        #[arg(
            long,
            help = "Optimization level",
            value_enum,
            default_value = "balanced"
        )]
        optimization: OptimizationLevelArg,

        #[arg(long, help = "Quantization type", value_enum)]
        quantization: Option<QuantizationTypeArg>,

        #[arg(long, help = "Maximum concurrent conversions", default_value = "2")]
        concurrent: usize,
    },

    #[command(about = "Analyze and report model information")]
    Analyze {
        #[arg(help = "Model path")]
        path: PathBuf,

        #[arg(long, help = "Show detailed analysis")]
        detailed: bool,

        #[arg(long, help = "Export analysis to file")]
        export: Option<PathBuf>,

        #[arg(long, help = "Export format", value_enum, default_value = "json")]
        export_format: ExportFormat,
    },

    #[command(about = "Benchmark conversion and optimization performance")]
    Benchmark {
        #[arg(help = "Model path")]
        model: PathBuf,

        #[arg(long, help = "Number of iterations", default_value = "3")]
        iterations: usize,

        #[arg(long, help = "Test all optimization levels")]
        all_optimizations: bool,

        #[arg(long, help = "Test all quantization types")]
        all_quantizations: bool,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ModelFormatArg {
    Gguf,
    Onnx,
    SafeTensors,
    Pytorch,
    TensorFlow,
}

impl From<ModelFormatArg> for ModelFormat {
    fn from(arg: ModelFormatArg) -> Self {
        match arg {
            ModelFormatArg::Gguf => ModelFormat::Gguf,
            ModelFormatArg::Onnx => ModelFormat::Onnx,
            ModelFormatArg::SafeTensors => ModelFormat::SafeTensors,
            ModelFormatArg::Pytorch => ModelFormat::Pytorch,
            ModelFormatArg::TensorFlow => ModelFormat::TensorFlow,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OptimizationLevelArg {
    None,
    Basic,
    Balanced,
    Aggressive,
    Maximum,
}

impl From<OptimizationLevelArg> for OptimizationLevel {
    fn from(arg: OptimizationLevelArg) -> Self {
        match arg {
            OptimizationLevelArg::None => OptimizationLevel::None,
            OptimizationLevelArg::Basic => OptimizationLevel::Basic,
            OptimizationLevelArg::Balanced => OptimizationLevel::Balanced,
            OptimizationLevelArg::Aggressive => OptimizationLevel::Aggressive,
            OptimizationLevelArg::Maximum => OptimizationLevel::Maximum,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum QuantizationTypeArg {
    Q4_0,
    Q4_1,
    Q5_0,
    Q5_1,
    Q8_0,
    F16,
    F32,
    Int8,
    Int16,
}

impl From<QuantizationTypeArg> for QuantizationType {
    fn from(arg: QuantizationTypeArg) -> Self {
        match arg {
            QuantizationTypeArg::Q4_0 => QuantizationType::Q4_0,
            QuantizationTypeArg::Q4_1 => QuantizationType::Q4_1,
            QuantizationTypeArg::Q5_0 => QuantizationType::Q5_0,
            QuantizationTypeArg::Q5_1 => QuantizationType::Q5_1,
            QuantizationTypeArg::Q8_0 => QuantizationType::Q8_0,
            QuantizationTypeArg::F16 => QuantizationType::F16,
            QuantizationTypeArg::F32 => QuantizationType::F32,
            QuantizationTypeArg::Int8 => QuantizationType::Int8,
            QuantizationTypeArg::Int16 => QuantizationType::Int16,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum PrecisionArg {
    Float32,
    Float16,
    Int8,
    Int16,
    Mixed,
}

impl From<PrecisionArg> for Precision {
    fn from(arg: PrecisionArg) -> Self {
        match arg {
            PrecisionArg::Float32 => Precision::Float32,
            PrecisionArg::Float16 => Precision::Float16,
            PrecisionArg::Int8 => Precision::Int8,
            PrecisionArg::Int16 => Precision::Int16,
            PrecisionArg::Mixed => Precision::Mixed,
        }
    }
}

#[derive(Clone, ValueEnum)]
pub enum ExportFormat {
    Json,
    Yaml,
    Toml,
}

pub async fn execute(args: ConvertArgs, config: &Config) -> Result<()> {
    let model_manager = Arc::new(ModelManager::new(&config.models_dir));
    let converter = ModelConverter::new(model_manager.clone(), config.clone());

    match args.command {
        ConvertCommand::Model {
            input,
            output,
            format,
            optimization,
            quantization,
            precision,
            context_length,
            batch_size,
            preserve_metadata,
            no_verify,
        } => {
            let config = ConvertModelConfig {
                input,
                output,
                format,
                optimization,
                quantization,
                precision,
                context_length,
                batch_size,
                preserve_metadata,
                verify_output: !no_verify,
            };
            convert_model(&converter, config).await
        }

        ConvertCommand::Optimize {
            input,
            output,
            remove_unused,
            merge_ops,
            constant_folding,
            dead_code,
            memory_opt,
            inference_opt,
            graph_simplify,
            operator_fusion,
        } => {
            let config = OptimizeModelConfig {
                input,
                output,
                remove_unused,
                merge_ops,
                constant_folding,
                dead_code,
                memory_opt,
                inference_opt,
                graph_simplify,
                operator_fusion,
            };
            optimize_model(&converter, config).await
        }

        ConvertCommand::Quantize {
            input,
            output,
            quantization,
        } => quantize_model(&converter, input, output, quantization).await,

        ConvertCommand::Batch {
            input_dir,
            output_dir,
            format,
            pattern,
            optimization,
            quantization,
            concurrent: _,
        } => {
            batch_convert_models(
                &converter,
                input_dir,
                output_dir,
                format,
                pattern,
                optimization,
                quantization,
            )
            .await
        }

        ConvertCommand::Analyze {
            path,
            detailed,
            export,
            export_format,
        } => analyze_model(&model_manager, path, detailed, export, export_format).await,

        ConvertCommand::Benchmark {
            model,
            iterations,
            all_optimizations,
            all_quantizations,
        } => {
            benchmark_conversion(
                &converter,
                model,
                iterations,
                all_optimizations,
                all_quantizations,
            )
            .await
        }
    }
}

async fn convert_model(
    converter: &ModelConverter,
    config: ConvertModelConfig,
) -> Result<()> {
    println!(
        "Converting model: {} -> {}",
        config.input.display(),
        config.output.display()
    );
    println!("Target format: {:?}", config.format);
    println!("Optimization: {:?}", config.optimization);

    if let Some(ref quant) = config.quantization {
        println!("Quantization: {:?}", quant);
    }

    // Store paths before moving config
    let input_path = config.input.clone();
    let output_path = config.output.clone();

    let conversion_config = config.into_conversion_config();

    let result = converter
        .convert_model(&input_path, &output_path, &conversion_config)
        .await?;

    if result.success {
        println!("✓ Conversion completed successfully!");
        println!(
            "  Input size: {:.2} MB",
            result.input_size as f64 / (1024.0 * 1024.0)
        );
        println!(
            "  Output size: {:.2} MB",
            result.output_size as f64 / (1024.0 * 1024.0)
        );
        println!("  Compression ratio: {:.2}x", result.compression_ratio);
        println!("  Conversion time: {:?}", result.conversion_time);
        println!("  Metadata preserved: {}", result.metadata_preserved);

        if !result.warnings.is_empty() {
            println!("  Warnings:");
            for warning in &result.warnings {
                println!("    - {}", warning);
            }
        }
    } else {
        println!("✗ Conversion failed!");
        for error in &result.errors {
            println!("  Error: {}", error);
        }
    }

    Ok(())
}

async fn optimize_model(
    converter: &ModelConverter,
    config: OptimizeModelConfig,
) -> Result<()> {
    println!(
        "Optimizing model: {} -> {}",
        config.input.display(),
        config.output.display()
    );

    // Store paths before moving config
    let input_path = config.input.clone();
    let output_path = config.output.clone();

    let optimization_options = config.into_optimization_options();

    println!("Optimization options:");
    println!(
        "  Remove unused layers: {}",
        optimization_options.remove_unused_layers
    );
    println!(
        "  Merge consecutive ops: {}",
        optimization_options.merge_consecutive_ops
    );
    println!(
        "  Constant folding: {}",
        optimization_options.constant_folding
    );
    println!(
        "  Dead code elimination: {}",
        optimization_options.dead_code_elimination
    );
    println!(
        "  Memory optimization: {}",
        optimization_options.memory_optimization
    );
    println!(
        "  Inference optimization: {}",
        optimization_options.inference_optimization
    );
    println!(
        "  Graph simplification: {}",
        optimization_options.graph_simplification
    );
    println!(
        "  Operator fusion: {}",
        optimization_options.operator_fusion
    );

    let result = converter
        .optimize_model(&input_path, &output_path, &optimization_options)
        .await?;

    if result.success {
        println!("✓ Optimization completed successfully!");
        println!(
            "  Input size: {:.2} MB",
            result.input_size as f64 / (1024.0 * 1024.0)
        );
        println!(
            "  Output size: {:.2} MB",
            result.output_size as f64 / (1024.0 * 1024.0)
        );
        println!(
            "  Size reduction: {:.2}%",
            (1.0 - result.compression_ratio) * 100.0
        );
        println!("  Optimization time: {:?}", result.conversion_time);

        if !result.warnings.is_empty() {
            println!("  Optimizations applied:");
            for warning in &result.warnings {
                println!("    - {}", warning);
            }
        }
    } else {
        println!("✗ Optimization failed!");
        for error in &result.errors {
            println!("  Error: {}", error);
        }
    }

    Ok(())
}

async fn quantize_model(
    converter: &ModelConverter,
    input: PathBuf,
    output: PathBuf,
    quantization: QuantizationTypeArg,
) -> Result<()> {
    println!(
        "Quantizing model: {} -> {}",
        input.display(),
        output.display()
    );
    println!("Quantization type: {:?}", quantization);

    let result = converter
        .quantize_model(&input, &output, quantization.into())
        .await?;

    if result.success {
        println!("✓ Quantization completed successfully!");
        println!(
            "  Input size: {:.2} MB",
            result.input_size as f64 / (1024.0 * 1024.0)
        );
        println!(
            "  Output size: {:.2} MB",
            result.output_size as f64 / (1024.0 * 1024.0)
        );
        println!(
            "  Size reduction: {:.1}%",
            (1.0 - result.compression_ratio) * 100.0
        );
        println!("  Quantization time: {:?}", result.conversion_time);

        if !result.warnings.is_empty() {
            println!("  Notes:");
            for warning in &result.warnings {
                println!("    - {}", warning);
            }
        }
    } else {
        println!("✗ Quantization failed!");
        for error in &result.errors {
            println!("  Error: {}", error);
        }
    }

    Ok(())
}

async fn batch_convert_models(
    converter: &ModelConverter,
    input_dir: PathBuf,
    output_dir: PathBuf,
    format: ModelFormatArg,
    pattern: Option<String>,
    optimization: OptimizationLevelArg,
    quantization: Option<QuantizationTypeArg>,
) -> Result<()> {
    println!(
        "Batch converting models from {} to {}",
        input_dir.display(),
        output_dir.display()
    );
    println!("Target format: {:?}", format);
    if let Some(ref pat) = pattern {
        println!("File pattern filter: {}", pat);
    }

    let conversion_config = ConversionConfig {
        output_format: format.into(),
        optimization_level: optimization.into(),
        quantization: quantization.map(Into::into),
        target_precision: None,
        context_length: None,
        batch_size: None,
        preserve_metadata: true,
        verify_output: true,
    };

    let results = converter
        .batch_convert_models(
            &input_dir,
            &output_dir,
            &conversion_config,
            pattern.as_deref(),
        )
        .await?;

    let successful = results.iter().filter(|r| r.success).count();
    let total_input_size: u64 = results.iter().map(|r| r.input_size).sum();
    let total_output_size: u64 = results.iter().map(|r| r.output_size).sum();
    let average_compression = if total_output_size > 0 {
        total_input_size as f64 / total_output_size as f64
    } else {
        0.0
    };

    println!("\n=== Batch Conversion Results ===");
    println!(
        "Successfully converted: {}/{} models",
        successful,
        results.len()
    );
    println!(
        "Total input size: {:.2} MB",
        total_input_size as f64 / (1024.0 * 1024.0)
    );
    println!(
        "Total output size: {:.2} MB",
        total_output_size as f64 / (1024.0 * 1024.0)
    );
    println!("Average compression: {:.2}x", average_compression);

    if successful < results.len() {
        println!("\nFailed conversions:");
        for result in &results {
            if !result.success {
                println!(
                    "  ✗ {}: {:?}",
                    result.input_path.file_name().unwrap().to_string_lossy(),
                    result.errors
                );
            }
        }
    }

    Ok(())
}

async fn analyze_model(
    model_manager: &Arc<ModelManager>,
    path: PathBuf,
    detailed: bool,
    export: Option<PathBuf>,
    export_format: ExportFormat,
) -> Result<()> {
    println!("Analyzing model: {}", path.display());

    if !path.exists() {
        return Err(anyhow::anyhow!(
            "Model file does not exist: {}",
            path.display()
        ));
    }

    let model_info = model_manager.resolve_model(&path.to_string_lossy()).await?;
    let validation_result = model_manager
        .validate_model_comprehensive(&path, None)
        .await?;

    println!("\n=== Model Information ===");
    println!("Name: {}", model_info.name);
    println!("Path: {}", model_info.path.display());
    println!("Size: {:.2} MB", model_info.size as f64 / (1024.0 * 1024.0));
    println!(
        "Modified: {}",
        model_info.modified.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("Backend: {}", model_info.backend_type);

    if let Some(checksum) = &model_info.checksum {
        println!("Checksum: {}", checksum);
    }

    println!("\n=== Validation Results ===");
    println!("Valid: {}", validation_result.is_valid);
    println!("File readable: {}", validation_result.file_readable);
    println!("Format valid: {}", validation_result.format_valid);
    println!("Size valid: {}", validation_result.size_valid);
    println!("Security valid: {}", validation_result.security_valid);
    println!("Metadata valid: {}", validation_result.metadata_valid);

    if let Some(checksum_valid) = validation_result.checksum_valid {
        println!("Checksum valid: {}", checksum_valid);
    }

    if !validation_result.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &validation_result.warnings {
            println!("  - {}", warning);
        }
    }

    if !validation_result.errors.is_empty() {
        println!("\nErrors:");
        for error in &validation_result.errors {
            println!("  - {}", error);
        }
    }

    if detailed {
        println!("\n=== Detailed Analysis ===");

        match model_info.backend_type.as_str() {
            "gguf" => match model_manager.get_gguf_metadata(&path).await {
                Ok(metadata) => {
                    println!("Architecture: {}", metadata.architecture);
                    println!(
                        "Parameters: {:.1}B",
                        metadata.parameter_count as f64 / 1_000_000_000.0
                    );
                    println!("Quantization: {}", metadata.quantization);
                    println!("Context length: {}", metadata.context_length);
                }
                Err(e) => {
                    warn!("Failed to read GGUF metadata: {}", e);
                }
            },
            "onnx" => match model_manager.get_onnx_metadata(&path).await {
                Ok(metadata) => {
                    println!("ONNX version: {}", metadata.version);
                    println!("Producer: {}", metadata.producer);
                    println!("Input count: {}", metadata.input_count);
                    println!("Output count: {}", metadata.output_count);
                }
                Err(e) => {
                    warn!("Failed to read ONNX metadata: {}", e);
                }
            },
            _ => {
                println!(
                    "Detailed analysis not available for {} format",
                    model_info.backend_type
                );
            }
        }

        // Compute checksum if not already available
        if model_info.checksum.is_none() {
            println!("\nComputing checksum...");
            match model_manager.compute_checksum(&path).await {
                Ok(checksum) => println!("SHA256: {}", checksum),
                Err(e) => warn!("Failed to compute checksum: {}", e),
            }
        }
    }

    // Export analysis if requested
    if let Some(export_path) = export {
        let analysis_data = serde_json::json!({
            "model_info": {
                "name": model_info.name,
                "path": model_info.path,
                "size": model_info.size,
                "modified": model_info.modified,
                "backend_type": model_info.backend_type,
                "checksum": model_info.checksum
            },
            "validation": {
                "is_valid": validation_result.is_valid,
                "file_readable": validation_result.file_readable,
                "format_valid": validation_result.format_valid,
                "size_valid": validation_result.size_valid,
                "security_valid": validation_result.security_valid,
                "metadata_valid": validation_result.metadata_valid,
                "checksum_valid": validation_result.checksum_valid,
                "warnings": validation_result.warnings,
                "errors": validation_result.errors
            }
        });

        let output_content = match export_format {
            ExportFormat::Json => serde_json::to_string_pretty(&analysis_data)?,
            ExportFormat::Yaml => serde_yaml::to_string(&analysis_data)
                .map_err(|e| anyhow::anyhow!("YAML serialization failed: {}", e))?,
            ExportFormat::Toml => toml::to_string_pretty(&analysis_data)?,
        };

        tokio::fs::write(&export_path, output_content).await?;
        println!("\nAnalysis exported to: {}", export_path.display());
    }

    Ok(())
}

async fn benchmark_conversion(
    converter: &ModelConverter,
    model: PathBuf,
    iterations: usize,
    all_optimizations: bool,
    all_quantizations: bool,
) -> Result<()> {
    println!(
        "Benchmarking conversion performance for: {}",
        model.display()
    );
    println!("Iterations: {}", iterations);

    if !model.exists() {
        return Err(anyhow::anyhow!(
            "Model file does not exist: {}",
            model.display()
        ));
    }

    let temp_dir = std::env::temp_dir().join("inferno_benchmark");
    tokio::fs::create_dir_all(&temp_dir).await?;

    println!("\n=== Baseline Conversion Benchmark ===");

    // Baseline conversion
    let mut baseline_times = Vec::new();
    for i in 0..iterations {
        let output_path = temp_dir.join(format!("baseline_{}.gguf", i));
        let config = ConversionConfig::default();

        let start = std::time::Instant::now();
        let result = converter
            .convert_model(&model, &output_path, &config)
            .await?;
        let duration = start.elapsed();

        if result.success {
            baseline_times.push(duration);
            println!("  Iteration {}: {:?}", i + 1, duration);
        } else {
            warn!("  Iteration {} failed: {:?}", i + 1, result.errors);
        }

        // Clean up
        let _ = tokio::fs::remove_file(&output_path).await;
    }

    if !baseline_times.is_empty() {
        let avg_time =
            baseline_times.iter().sum::<std::time::Duration>() / baseline_times.len() as u32;
        println!("  Average time: {:?}", avg_time);
    }

    if all_optimizations {
        println!("\n=== Optimization Level Benchmark ===");

        let optimization_levels = [
            OptimizationLevel::None,
            OptimizationLevel::Basic,
            OptimizationLevel::Balanced,
            OptimizationLevel::Aggressive,
            OptimizationLevel::Maximum,
        ];

        for opt_level in &optimization_levels {
            let mut times = Vec::new();
            println!("Testing {:?} optimization:", opt_level);

            for i in 0..iterations {
                let output_path = temp_dir.join(format!("opt_{:?}_{}.gguf", opt_level, i));
                let config = ConversionConfig {
                    optimization_level: opt_level.clone(),
                    ..Default::default()
                };

                let start = std::time::Instant::now();
                let result = converter
                    .convert_model(&model, &output_path, &config)
                    .await?;
                let duration = start.elapsed();

                if result.success {
                    times.push(duration);
                }

                let _ = tokio::fs::remove_file(&output_path).await;
            }

            if !times.is_empty() {
                let avg_time = times.iter().sum::<std::time::Duration>() / times.len() as u32;
                println!("  Average time: {:?}", avg_time);
            }
        }
    }

    if all_quantizations {
        println!("\n=== Quantization Benchmark ===");

        let quantization_types = [
            QuantizationType::Q4_0,
            QuantizationType::Q4_1,
            QuantizationType::Q5_0,
            QuantizationType::Q8_0,
            QuantizationType::F16,
        ];

        for quant_type in &quantization_types {
            let mut times = Vec::new();
            println!("Testing {:?} quantization:", quant_type);

            for i in 0..iterations {
                let output_path = temp_dir.join(format!("quant_{:?}_{}.gguf", quant_type, i));

                let start = std::time::Instant::now();
                let result = converter
                    .quantize_model(&model, &output_path, quant_type.clone())
                    .await?;
                let duration = start.elapsed();

                if result.success {
                    times.push(duration);
                    println!(
                        "    Iteration {}: {:?} (compression: {:.2}x)",
                        i + 1,
                        duration,
                        result.compression_ratio
                    );
                }

                let _ = tokio::fs::remove_file(&output_path).await;
            }

            if !times.is_empty() {
                let avg_time = times.iter().sum::<std::time::Duration>() / times.len() as u32;
                println!("  Average time: {:?}", avg_time);
            }
        }
    }

    // Clean up temp directory
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;

    println!("\nBenchmark completed!");

    Ok(())
}
