//! Models Command - New Architecture
//!
//! This module demonstrates the migration of the models command to the new
//! CLI architecture with Command trait, pipeline, and middleware support.
//!
//! This is a v2 implementation that runs alongside the original models.rs
//! for gradual migration and testing.

use crate::config::Config;
use crate::interfaces::cli::{Command, CommandContext, CommandOutput};
use crate::models::ModelManager;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;

// ============================================================================
// ModelsList Command
// ============================================================================

/// List all available models
pub struct ModelsList {
    config: Config,
}

impl ModelsList {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Command for ModelsList {
    fn name(&self) -> &str {
        "models list"
    }

    fn description(&self) -> &str {
        "List all available models in the models directory"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if !self.config.models_dir.exists() {
            anyhow::bail!(
                "Models directory does not exist: {}",
                self.config.models_dir.display()
            );
        }
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        let model_manager = ModelManager::new(&self.config.models_dir);
        let models = model_manager.list_models().await?;

        if models.is_empty() {
            let message = format!(
                "No models found in: {}\nPlace GGUF (*.gguf) or ONNX (*.onnx) models in the models directory.",
                self.config.models_dir.display()
            );

            if ctx.json_output {
                return Ok(CommandOutput::success_with_data(
                    "No models found",
                    json!({
                        "models": [],
                        "models_dir": self.config.models_dir.display().to_string(),
                    }),
                ));
            } else {
                println!("{}", message);
                return Ok(CommandOutput::success("No models found"));
            }
        }

        // Format models for output
        let models_json: Vec<_> = models
            .iter()
            .map(|m| {
                json!({
                    "name": m.name,
                    "type": m.backend_type.to_string(),
                    "size": m.size,
                    "size_formatted": format_size(m.size),
                    "modified": m.modified.to_rfc3339(),
                    "path": m.path.display().to_string(),
                })
            })
            .collect();

        // Human-readable output
        if !ctx.json_output {
            println!("Available models:");
            println!(
                "{:<30} {:<15} {:<20} {:<15}",
                "Name", "Type", "Size", "Modified"
            );
            println!("{}", "─".repeat(80));

            for model in &models {
                let size_str = format_size(model.size);
                let modified = model.modified.format("%Y-%m-%d %H:%M").to_string();
                println!(
                    "{:<30} {:<15} {:<20} {:<15}",
                    model.name, model.backend_type, size_str, modified
                );
            }
        }

        Ok(CommandOutput::success_with_data(
            format!("Found {} models", models.len()),
            json!({
                "count": models.len(),
                "models": models_json,
                "models_dir": self.config.models_dir.display().to_string(),
            }),
        ))
    }
}

// ============================================================================
// ModelsInfo Command
// ============================================================================

/// Show detailed information about a specific model
pub struct ModelsInfo {
    config: Config,
    model: String,
}

impl ModelsInfo {
    pub fn new(config: Config, model: String) -> Self {
        Self { config, model }
    }
}

#[async_trait]
impl Command for ModelsInfo {
    fn name(&self) -> &str {
        "models info"
    }

    fn description(&self) -> &str {
        "Show detailed information about a model"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.model.is_empty() {
            anyhow::bail!("Model name or path cannot be empty");
        }
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        let model_manager = ModelManager::new(&self.config.models_dir);
        let model_info = model_manager.resolve_model(&self.model).await?;

        let mut metadata_json = json!({
            "name": model_info.name,
            "path": model_info.path.display().to_string(),
            "type": model_info.backend_type.to_string(),
            "size": model_info.size,
            "size_formatted": format_size(model_info.size),
            "modified": model_info.modified.to_rfc3339(),
        });

        // Add checksum if available
        if let Some(checksum) = &model_info.checksum {
            metadata_json["checksum"] = json!(checksum);
        }

        // Backend-specific information
        match model_info.backend_type.as_str() {
            "gguf" => {
                if let Ok(metadata) = model_manager.get_gguf_metadata(&model_info.path).await {
                    metadata_json["architecture"] = json!(metadata.architecture);
                    metadata_json["parameters"] = json!(metadata.parameter_count);
                    metadata_json["parameters_formatted"] =
                        json!(format_params(metadata.parameter_count));
                    metadata_json["quantization"] = json!(metadata.quantization);
                    metadata_json["context_length"] = json!(metadata.context_length);
                }
            }
            "onnx" => {
                if let Ok(metadata) = model_manager.get_onnx_metadata(&model_info.path).await {
                    metadata_json["onnx_version"] = json!(metadata.version);
                    metadata_json["producer"] = json!(metadata.producer);
                    metadata_json["input_count"] = json!(metadata.input_count);
                    metadata_json["output_count"] = json!(metadata.output_count);
                }
            }
            _ => {}
        }

        // Human-readable output
        if !ctx.json_output {
            println!("Model Information:");
            println!("  Name: {}", model_info.name);
            println!("  Path: {}", model_info.path.display());
            println!("  Type: {}", model_info.backend_type);
            println!("  Size: {}", format_size(model_info.size));
            println!(
                "  Modified: {}",
                model_info.modified.format("%Y-%m-%d %H:%M:%S")
            );

            if let Some(checksum) = &model_info.checksum {
                println!("  SHA256: {}", checksum);
            }

            // Backend-specific information
            match model_info.backend_type.as_str() {
                "gguf" => {
                    if let Ok(metadata) = model_manager.get_gguf_metadata(&model_info.path).await {
                        println!("  Architecture: {}", metadata.architecture);
                        println!("  Parameters: {}", format_params(metadata.parameter_count));
                        println!("  Quantization: {}", metadata.quantization);
                        println!("  Context Length: {}", metadata.context_length);
                    }
                }
                "onnx" => {
                    if let Ok(metadata) = model_manager.get_onnx_metadata(&model_info.path).await {
                        println!("  ONNX Version: {}", metadata.version);
                        println!("  Producer: {}", metadata.producer);
                        println!("  Inputs: {}", metadata.input_count);
                        println!("  Outputs: {}", metadata.output_count);
                    }
                }
                _ => {}
            }
        }

        Ok(CommandOutput::success_with_data(
            format!("Model info for: {}", model_info.name),
            metadata_json,
        ))
    }
}

// ============================================================================
// ModelsValidate Command
// ============================================================================

/// Validate a model file
pub struct ModelsValidate {
    path: PathBuf,
}

impl ModelsValidate {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait]
impl Command for ModelsValidate {
    fn name(&self) -> &str {
        "models validate"
    }

    fn description(&self) -> &str {
        "Validate a model file format and integrity"
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
        let model_manager =
            ModelManager::new(&self.path.parent().unwrap_or(std::path::Path::new(".")));

        // Perform comprehensive validation
        let validation_result = model_manager
            .validate_model_comprehensive(&self.path, None)
            .await?;

        let validation_json = json!({
            "path": self.path.display().to_string(),
            "valid": validation_result.is_valid,
            "file_readable": validation_result.file_readable,
            "format_valid": validation_result.format_valid,
            "size_valid": validation_result.size_valid,
            "checksum_valid": validation_result.checksum_valid,
            "security_valid": validation_result.security_valid,
            "metadata_valid": validation_result.metadata_valid,
            "errors": validation_result.errors,
            "warnings": validation_result.warnings,
        });

        // Human-readable output
        if !ctx.json_output {
            println!("Model Validation Results:");
            println!("  Path: {}", self.path.display());
            println!(
                "  Valid: {}",
                if validation_result.is_valid {
                    "✓"
                } else {
                    "✗"
                }
            );
            println!(
                "  File Readable: {}",
                if validation_result.file_readable {
                    "✓"
                } else {
                    "✗"
                }
            );
            println!(
                "  Format Valid: {}",
                if validation_result.format_valid {
                    "✓"
                } else {
                    "✗"
                }
            );
            println!(
                "  Size Valid: {}",
                if validation_result.size_valid {
                    "✓"
                } else {
                    "✗"
                }
            );
            if let Some(checksum_valid) = validation_result.checksum_valid {
                println!(
                    "  Checksum Valid: {}",
                    if checksum_valid { "✓" } else { "✗" }
                );
            }
            println!(
                "  Security Valid: {}",
                if validation_result.security_valid {
                    "✓"
                } else {
                    "✗"
                }
            );
            println!(
                "  Metadata Valid: {}",
                if validation_result.metadata_valid {
                    "✓"
                } else {
                    "✗"
                }
            );

            if !validation_result.errors.is_empty() {
                println!("\n  Errors:");
                for error in &validation_result.errors {
                    println!("    • {}", error);
                }
            }

            if !validation_result.warnings.is_empty() {
                println!("\n  Warnings:");
                for warning in &validation_result.warnings {
                    println!("    • {}", warning);
                }
            }
        }

        if validation_result.is_valid {
            Ok(CommandOutput::success_with_data(
                "Model validation passed",
                validation_json,
            ))
        } else {
            Ok(CommandOutput::error_with_data(
                "Model validation failed",
                validation_json,
                1, // Exit code for validation failure
            ))
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

fn format_params(count: u64) -> String {
    const MILLION: u64 = 1_000_000;
    const BILLION: u64 = 1_000_000_000;

    if count >= BILLION {
        format!("{:.1}B", count as f64 / BILLION as f64)
    } else if count >= MILLION {
        format!("{:.0}M", count as f64 / MILLION as f64)
    } else {
        format!("{}", count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1_572_864), "1.50 MB");
        assert_eq!(format_size(1_610_612_736), "1.50 GB");
    }

    #[test]
    fn test_format_params() {
        assert_eq!(format_params(500_000), "500000");
        assert_eq!(format_params(7_000_000), "7M");
        assert_eq!(format_params(13_000_000_000), "13.0B");
    }
}
