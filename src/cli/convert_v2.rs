//! Convert Command - New Architecture
//!
//! This module demonstrates the migration of the convert command to the new
//! CLI architecture. Focuses on core model conversion operations.
//!
//! Note: This is a focused migration covering the most commonly used subcommands.
//! Full conversion functionality remains available through the original module.

use crate::config::Config;
use crate::conversion::{
    ConversionConfig, ModelConverter, ModelFormat, OptimizationLevel, Precision, QuantizationType,
};
use crate::interfaces::cli::{Command, CommandContext, CommandOutput};
use crate::models::ModelManager;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

// ============================================================================
// ConvertModel - Convert model between formats
// ============================================================================

/// Convert a model between different formats
pub struct ConvertModel {
    config: Config,
    input: PathBuf,
    output: PathBuf,
    format: ModelFormat,
    optimization: OptimizationLevel,
    quantization: Option<QuantizationType>,
    precision: Option<Precision>,
    context_length: Option<u32>,
    batch_size: Option<u32>,
    preserve_metadata: bool,
    verify_output: bool,
}

impl ConvertModel {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: Config,
        input: PathBuf,
        output: PathBuf,
        format: ModelFormat,
        optimization: OptimizationLevel,
        quantization: Option<QuantizationType>,
        precision: Option<Precision>,
        context_length: Option<u32>,
        batch_size: Option<u32>,
        preserve_metadata: bool,
        verify_output: bool,
    ) -> Self {
        Self {
            config,
            input,
            output,
            format,
            optimization,
            quantization,
            precision,
            context_length,
            batch_size,
            preserve_metadata,
            verify_output,
        }
    }
}

#[async_trait]
impl Command for ConvertModel {
    fn name(&self) -> &str {
        "convert model"
    }

    fn description(&self) -> &str {
        "Convert model between different formats"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // Validate input exists
        if !self.input.exists() {
            anyhow::bail!("Input model does not exist: {}", self.input.display());
        }

        if !self.input.is_file() {
            anyhow::bail!("Input path is not a file: {}", self.input.display());
        }

        // Validate output directory exists or can be created
        if let Some(parent) = self.output.parent() {
            if !parent.exists() {
                anyhow::bail!("Output directory does not exist: {}", parent.display());
            }
        }

        // Validate context length if specified
        if let Some(ctx_len) = self.context_length {
            if ctx_len == 0 {
                anyhow::bail!("Context length cannot be 0");
            }
            if ctx_len > 32768 {
                anyhow::bail!("Context length cannot exceed 32768");
            }
        }

        // Validate batch size if specified
        if let Some(batch) = self.batch_size {
            if batch == 0 {
                anyhow::bail!("Batch size cannot be 0");
            }
            if batch > 1024 {
                anyhow::bail!("Batch size cannot exceed 1024");
            }
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!(
            "Converting model: {} -> {}",
            self.input.display(),
            self.output.display()
        );

        let model_manager = Arc::new(ModelManager::new(&self.config.models_dir));
        let converter = ModelConverter::new(model_manager, self.config.clone());

        let conversion_config = ConversionConfig {
            output_format: self.format.clone(),
            optimization_level: self.optimization.clone(),
            quantization: self.quantization.clone(),
            target_precision: self.precision.clone(),
            context_length: self.context_length,
            batch_size: self.batch_size,
            preserve_metadata: self.preserve_metadata,
            verify_output: self.verify_output,
        };

        // Human-readable output
        if !ctx.json_output {
            println!(
                "Converting model: {} -> {}",
                self.input.display(),
                self.output.display()
            );
            println!("Target format: {:?}", self.format);
            println!("Optimization: {:?}", self.optimization);

            if let Some(ref quant) = self.quantization {
                println!("Quantization: {:?}", quant);
            }
        }

        let result = converter
            .convert_model(&self.input, &self.output, &conversion_config)
            .await?;

        // Human-readable output
        if !ctx.json_output {
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
        }

        // Structured output
        if result.success {
            Ok(CommandOutput::success_with_data(
                format!(
                    "Successfully converted {} to {}",
                    self.input.display(),
                    self.output.display()
                ),
                json!({
                    "success": true,
                    "input_path": self.input.display().to_string(),
                    "output_path": self.output.display().to_string(),
                    "input_size": result.input_size,
                    "output_size": result.output_size,
                    "compression_ratio": result.compression_ratio,
                    "conversion_time_ms": result.conversion_time.as_millis(),
                    "metadata_preserved": result.metadata_preserved,
                    "warnings": result.warnings,
                }),
            ))
        } else {
            Ok(CommandOutput::error_with_data(
                "Conversion failed",
                json!({
                    "success": false,
                    "errors": result.errors,
                    "warnings": result.warnings,
                }),
                1,
            ))
        }
    }
}

// ============================================================================
// QuantizeModel - Quantize model to reduce size
// ============================================================================

/// Quantize a model to reduce its size
pub struct QuantizeModel {
    config: Config,
    input: PathBuf,
    output: PathBuf,
    quantization: QuantizationType,
}

impl QuantizeModel {
    pub fn new(
        config: Config,
        input: PathBuf,
        output: PathBuf,
        quantization: QuantizationType,
    ) -> Self {
        Self {
            config,
            input,
            output,
            quantization,
        }
    }
}

#[async_trait]
impl Command for QuantizeModel {
    fn name(&self) -> &str {
        "convert quantize"
    }

    fn description(&self) -> &str {
        "Quantize model to reduce size"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // Validate input exists
        if !self.input.exists() {
            anyhow::bail!("Input model does not exist: {}", self.input.display());
        }

        if !self.input.is_file() {
            anyhow::bail!("Input path is not a file: {}", self.input.display());
        }

        // Validate output directory exists
        if let Some(parent) = self.output.parent() {
            if !parent.exists() {
                anyhow::bail!("Output directory does not exist: {}", parent.display());
            }
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!(
            "Quantizing model: {} -> {}",
            self.input.display(),
            self.output.display()
        );

        let model_manager = Arc::new(ModelManager::new(&self.config.models_dir));
        let converter = ModelConverter::new(model_manager, self.config.clone());

        // Human-readable output
        if !ctx.json_output {
            println!(
                "Quantizing model: {} -> {}",
                self.input.display(),
                self.output.display()
            );
            println!("Quantization type: {:?}", self.quantization);
        }

        let result = converter
            .quantize_model(&self.input, &self.output, self.quantization.clone())
            .await?;

        // Human-readable output
        if !ctx.json_output {
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
        }

        // Structured output
        if result.success {
            Ok(CommandOutput::success_with_data(
                format!(
                    "Successfully quantized {} to {}",
                    self.input.display(),
                    self.output.display()
                ),
                json!({
                    "success": true,
                    "input_path": self.input.display().to_string(),
                    "output_path": self.output.display().to_string(),
                    "quantization_type": format!("{:?}", self.quantization),
                    "input_size": result.input_size,
                    "output_size": result.output_size,
                    "size_reduction_percent": (1.0 - result.compression_ratio) * 100.0,
                    "quantization_time_ms": result.conversion_time.as_millis(),
                    "warnings": result.warnings,
                }),
            ))
        } else {
            Ok(CommandOutput::error_with_data(
                "Quantization failed",
                json!({
                    "success": false,
                    "errors": result.errors,
                    "warnings": result.warnings,
                }),
                1,
            ))
        }
    }
}

// ============================================================================
// AnalyzeModel - Analyze and report model information
// ============================================================================

/// Analyze and report detailed model information
pub struct AnalyzeModel {
    config: Config,
    path: PathBuf,
    detailed: bool,
}

impl AnalyzeModel {
    pub fn new(config: Config, path: PathBuf, detailed: bool) -> Self {
        Self {
            config,
            path,
            detailed,
        }
    }
}

#[async_trait]
impl Command for AnalyzeModel {
    fn name(&self) -> &str {
        "convert analyze"
    }

    fn description(&self) -> &str {
        "Analyze and report model information"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if !self.path.exists() {
            anyhow::bail!("Model file does not exist: {}", self.path.display());
        }

        if !self.path.is_file() {
            anyhow::bail!("Path is not a file: {}", self.path.display());
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Analyzing model: {}", self.path.display());

        let model_manager = Arc::new(ModelManager::new(&self.config.models_dir));

        let model_info = model_manager
            .resolve_model(&self.path.to_string_lossy())
            .await?;
        let validation_result = model_manager
            .validate_model_comprehensive(&self.path, None)
            .await?;

        // Human-readable output
        if !ctx.json_output {
            println!("Analyzing model: {}", self.path.display());
            println!();
            println!("=== Model Information ===");
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

            println!();
            println!("=== Validation Results ===");
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
                println!();
                println!("Warnings:");
                for warning in &validation_result.warnings {
                    println!("  - {}", warning);
                }
            }

            if !validation_result.errors.is_empty() {
                println!();
                println!("Errors:");
                for error in &validation_result.errors {
                    println!("  - {}", error);
                }
            }

            if self.detailed {
                println!();
                println!("=== Detailed Analysis ===");

                match model_info.backend_type.as_str() {
                    "gguf" => match model_manager.get_gguf_metadata(&self.path).await {
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
                            println!("Failed to read GGUF metadata: {}", e);
                        }
                    },
                    "onnx" => match model_manager.get_onnx_metadata(&self.path).await {
                        Ok(metadata) => {
                            println!("ONNX version: {}", metadata.version);
                            println!("Producer: {}", metadata.producer);
                            println!("Input count: {}", metadata.input_count);
                            println!("Output count: {}", metadata.output_count);
                        }
                        Err(e) => {
                            println!("Failed to read ONNX metadata: {}", e);
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
                    println!();
                    println!("Computing checksum...");
                    match model_manager.compute_checksum(&self.path).await {
                        Ok(checksum) => println!("SHA256: {}", checksum),
                        Err(e) => println!("Failed to compute checksum: {}", e),
                    }
                }
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!("Analysis complete for {}", self.path.display()),
            json!({
                "model_info": {
                    "name": model_info.name,
                    "path": model_info.path.display().to_string(),
                    "size": model_info.size,
                    "size_mb": model_info.size as f64 / (1024.0 * 1024.0),
                    "modified": model_info.modified.to_rfc3339(),
                    "backend_type": model_info.backend_type,
                    "checksum": model_info.checksum,
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
                    "errors": validation_result.errors,
                },
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_convert_validation() {
        let config = Config::default();
        let cmd = ConvertModel::new(
            config.clone(),
            PathBuf::from("/nonexistent/input.gguf"),
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
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[tokio::test]
    async fn test_convert_invalid_context_length() {
        let config = Config::default();
        let temp_file = std::env::temp_dir().join("test_model.gguf");
        std::fs::write(&temp_file, b"test").unwrap();

        let cmd = ConvertModel::new(
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
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be 0"));

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_quantize_validation() {
        let config = Config::default();
        let cmd = QuantizeModel::new(
            config.clone(),
            PathBuf::from("/nonexistent/model.gguf"),
            PathBuf::from("/tmp/output.gguf"),
            QuantizationType::Q4_0,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[tokio::test]
    async fn test_analyze_validation() {
        let config = Config::default();
        let cmd = AnalyzeModel::new(
            config.clone(),
            PathBuf::from("/nonexistent/model.gguf"),
            false,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
}
