//! Validate Command - New Architecture
//!
//! This module demonstrates the migration of the validate command to the new
//! CLI architecture with Command trait, pipeline, and middleware support.
//!
//! Validates model files, config files, and directories with optional deep validation.

use crate::backends::{Backend, BackendType, InferenceParams};
use crate::config::Config;
use crate::interfaces::cli::{Command, CommandContext, CommandOutput};
use crate::models::{ModelInfo, ModelManager, ValidationResult};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use tracing::info;

// ============================================================================
// ValidateCommand - File and model validation
// ============================================================================

/// Validate model files, config files, or directories
pub struct ValidateCommand {
    config: Config,
    path: PathBuf,
    checksum: bool,
    deep: bool,
}

impl ValidateCommand {
    pub fn new(config: Config, path: PathBuf, checksum: bool, deep: bool) -> Self {
        Self {
            config,
            path,
            checksum,
            deep,
        }
    }
}

#[async_trait]
impl Command for ValidateCommand {
    fn name(&self) -> &str {
        "validate"
    }

    fn description(&self) -> &str {
        "Validate model files, config files, or directories"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // Validate path exists
        if !self.path.exists() {
            anyhow::bail!("Path does not exist: {}", self.path.display());
        }

        // Validate it's either a file or directory
        if !self.path.is_file() && !self.path.is_dir() {
            anyhow::bail!("Path must be a file or directory: {}", self.path.display());
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Validating: {}", self.path.display());

        let validation_result = if self.path.is_file() {
            self.validate_file(ctx).await?
        } else {
            self.validate_directory(ctx).await?
        };

        // Determine success
        let success = validation_result.passed;
        let exit_code = if success { 0 } else { 1 };

        // Human-readable output
        if !ctx.json_output {
            if success {
                println!("✓ All validations passed");
            } else {
                println!("✗ Some validations failed");
            }

            if ctx.is_verbose() {
                print_validation_summary(&validation_result);
            }
        }

        // Structured output
        let result_json = json!({
            "path": self.path.display().to_string(),
            "path_type": if self.path.is_file() { "file" } else { "directory" },
            "passed": validation_result.passed,
            "checksum_validation": self.checksum,
            "deep_validation": self.deep,
            "details": validation_result.details,
            "errors": validation_result.errors,
            "warnings": validation_result.warnings,
        });

        if success {
            Ok(CommandOutput::success_with_data(
                format!("Validation passed for: {}", self.path.display()),
                result_json,
            ))
        } else {
            Ok(CommandOutput::error_with_data(
                format!("Validation failed for: {}", self.path.display()),
                result_json,
                exit_code,
            ))
        }
    }
}

impl ValidateCommand {
    /// Validate a single file
    async fn validate_file(&self, ctx: &CommandContext) -> Result<FileValidationResult> {
        let mut result = FileValidationResult::default();

        // Determine file type
        if let Some(ext) = self.path.extension() {
            match ext.to_str().unwrap_or("") {
                "gguf" | "onnx" => {
                    result.merge(self.validate_model_file(ctx).await?);
                }
                "toml" => {
                    result.merge(self.validate_config_file(ctx).await?);
                }
                _ => {
                    if ctx.is_verbose() {
                        println!("ℹ Unknown file type, performing basic validation");
                    }
                    result.merge(self.validate_basic_file(ctx).await?);
                }
            }
        } else {
            result.merge(self.validate_basic_file(ctx).await?);
        }

        Ok(result)
    }

    /// Validate a directory
    async fn validate_directory(&self, ctx: &CommandContext) -> Result<FileValidationResult> {
        let mut result = FileValidationResult::default();
        let mut model_count = 0;

        if !ctx.json_output {
            println!("Validating directory: {}", self.path.display());
        }

        let mut entries = tokio::fs::read_dir(&self.path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    if matches!(ext.to_str().unwrap_or(""), "gguf" | "onnx") {
                        model_count += 1;
                        if ctx.is_verbose() {
                            println!("  Validating model: {}", entry_path.display());
                        }

                        // Create temporary command for this file
                        let file_cmd = ValidateCommand::new(
                            self.config.clone(),
                            entry_path,
                            self.checksum,
                            self.deep,
                        );
                        result.merge(file_cmd.validate_model_file(ctx).await?);
                    }
                }
            }
        }

        if model_count == 0 {
            result
                .warnings
                .push("No model files found in directory".to_string());
            if !ctx.json_output {
                println!("ℹ No model files found in directory");
            }
        } else {
            result
                .details
                .insert("model_count".to_string(), model_count.into());
            if !ctx.json_output {
                println!("✓ Validated {} model files", model_count);
            }
        }

        Ok(result)
    }

    /// Validate a model file (GGUF or ONNX)
    async fn validate_model_file(&self, ctx: &CommandContext) -> Result<FileValidationResult> {
        let mut result = FileValidationResult::default();
        let model_manager = ModelManager::new(&self.config.models_dir);

        if ctx.is_verbose() && !ctx.json_output {
            println!("Validating model file: {}", self.path.display());
        }

        // Basic file checks
        let metadata = tokio::fs::metadata(&self.path).await?;
        if metadata.len() == 0 {
            result.passed = false;
            result.errors.push("Model file is empty".to_string());
            return Ok(result);
        }

        // Check readability
        match tokio::fs::File::open(&self.path).await {
            Ok(_) => {
                if ctx.is_verbose() && !ctx.json_output {
                    println!("  ✓ File is readable");
                }
                result.details.insert("readable".to_string(), true.into());
            }
            Err(e) => {
                result.passed = false;
                result.errors.push(format!("Cannot read file: {}", e));
                return Ok(result);
            }
        }

        // Format-specific validation
        if let Some(ext) = self.path.extension() {
            match ext.to_str().unwrap_or("") {
                "gguf" => {
                    result.merge(self.validate_gguf_file(&model_manager, ctx).await?);
                }
                "onnx" => {
                    result.merge(self.validate_onnx_file(&model_manager, ctx).await?);
                }
                _ => {}
            }
        }

        // Checksum validation
        if self.checksum {
            if ctx.is_verbose() && !ctx.json_output {
                println!("  Computing SHA256 checksum...");
            }
            let checksum = model_manager.compute_checksum(&self.path).await?;
            result
                .details
                .insert("checksum".to_string(), checksum.clone().into());
            if ctx.is_verbose() && !ctx.json_output {
                println!("  ✓ SHA256: {}", checksum);
            }
        }

        // Comprehensive validation
        let validation = model_manager
            .validate_model_comprehensive(&self.path, Some(&self.config))
            .await?;

        result.merge_validation_result(&validation);

        if validation.is_valid {
            if !ctx.json_output && !ctx.is_verbose() {
                println!("✓ Model is valid: {}", self.path.display());
            }
            if ctx.is_verbose() && !ctx.json_output {
                print_validation_details(&validation, true);
            }
        } else {
            result.passed = false;
            if !ctx.json_output {
                println!("✗ Model validation failed: {}", self.path.display());
                print_validation_details(&validation, ctx.is_verbose());
            }
        }

        // Deep validation (actually load the model)
        if self.deep && validation.is_valid {
            if ctx.is_verbose() && !ctx.json_output {
                println!("  Performing deep validation...");
            }
            result.merge(self.deep_validate_model(ctx).await?);
        }

        Ok(result)
    }

    /// Validate GGUF file format
    async fn validate_gguf_file(
        &self,
        model_manager: &ModelManager,
        ctx: &CommandContext,
    ) -> Result<FileValidationResult> {
        let mut result = FileValidationResult::default();

        match model_manager.get_gguf_metadata(&self.path).await {
            Ok(metadata) => {
                if ctx.is_verbose() && !ctx.json_output {
                    println!("  ✓ Valid GGUF file");
                    println!("    Architecture: {}", metadata.architecture);
                    println!("    Parameters: {}", metadata.parameter_count);
                    println!("    Quantization: {}", metadata.quantization);
                }

                result.details.insert("format".to_string(), "gguf".into());
                result
                    .details
                    .insert("architecture".to_string(), metadata.architecture.into());
                result
                    .details
                    .insert("parameters".to_string(), metadata.parameter_count.into());
                result
                    .details
                    .insert("quantization".to_string(), metadata.quantization.into());
            }
            Err(e) => {
                result.passed = false;
                result.errors.push(format!("Invalid GGUF file: {}", e));
            }
        }

        Ok(result)
    }

    /// Validate ONNX file format
    async fn validate_onnx_file(
        &self,
        model_manager: &ModelManager,
        ctx: &CommandContext,
    ) -> Result<FileValidationResult> {
        let mut result = FileValidationResult::default();

        match model_manager.get_onnx_metadata(&self.path).await {
            Ok(metadata) => {
                if ctx.is_verbose() && !ctx.json_output {
                    println!("  ✓ Valid ONNX file");
                    println!("    Version: {}", metadata.version);
                    println!("    Producer: {}", metadata.producer);
                    println!("    Inputs: {}", metadata.input_count);
                    println!("    Outputs: {}", metadata.output_count);
                }

                result.details.insert("format".to_string(), "onnx".into());
                result
                    .details
                    .insert("version".to_string(), metadata.version.into());
                result
                    .details
                    .insert("producer".to_string(), metadata.producer.into());
                result
                    .details
                    .insert("input_count".to_string(), metadata.input_count.into());
                result
                    .details
                    .insert("output_count".to_string(), metadata.output_count.into());
            }
            Err(e) => {
                result.passed = false;
                result.errors.push(format!("Invalid ONNX file: {}", e));
            }
        }

        Ok(result)
    }

    /// Validate config file (TOML)
    async fn validate_config_file(&self, ctx: &CommandContext) -> Result<FileValidationResult> {
        let mut result = FileValidationResult::default();

        if ctx.is_verbose() && !ctx.json_output {
            println!("Validating config file: {}", self.path.display());
        }

        let content = tokio::fs::read_to_string(&self.path).await?;
        match toml::from_str::<toml::Value>(&content) {
            Ok(_) => {
                if ctx.is_verbose() && !ctx.json_output {
                    println!("  ✓ Valid TOML syntax");
                }
                result.details.insert("format".to_string(), "toml".into());
                result
                    .details
                    .insert("syntax_valid".to_string(), true.into());
            }
            Err(e) => {
                result.passed = false;
                result.errors.push(format!("Invalid TOML file: {}", e));
            }
        }

        Ok(result)
    }

    /// Basic file validation
    async fn validate_basic_file(&self, ctx: &CommandContext) -> Result<FileValidationResult> {
        let mut result = FileValidationResult::default();

        if ctx.is_verbose() && !ctx.json_output {
            println!("Validating file: {}", self.path.display());
        }

        let metadata = tokio::fs::metadata(&self.path).await?;

        if ctx.is_verbose() && !ctx.json_output {
            println!("  ✓ File exists");
            println!("  ✓ Size: {} bytes", metadata.len());
            println!("  ✓ Modified: {:?}", metadata.modified()?);
        }

        result
            .details
            .insert("size".to_string(), metadata.len().into());
        result.details.insert("exists".to_string(), true.into());

        Ok(result)
    }

    /// Deep validation - actually load and test the model
    async fn deep_validate_model(&self, ctx: &CommandContext) -> Result<FileValidationResult> {
        let mut result = FileValidationResult::default();

        let backend_type = BackendType::from_model_path(&self.path).ok_or_else(|| {
            anyhow::anyhow!(
                "No suitable backend found for model: {}",
                self.path.display()
            )
        })?;

        let mut backend = Backend::new(backend_type.clone(), &self.config.backend_config)?;

        let metadata = tokio::fs::metadata(&self.path).await?;
        let model_info = ModelInfo {
            name: self.path.file_name().unwrap().to_string_lossy().to_string(),
            path: self.path.clone(),
            file_path: self.path.clone(),
            size: metadata.len(),
            size_bytes: metadata.len(),
            modified: chrono::DateTime::from(metadata.modified()?),
            backend_type: backend_type.to_string(),
            format: self
                .path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("unknown")
                .to_string(),
            checksum: None,
            metadata: std::collections::HashMap::new(),
        };

        match backend.load_model(&model_info).await {
            Ok(_) => {
                if !ctx.json_output {
                    println!("  ✓ Model loads successfully");
                }

                // Test a simple inference
                let test_input = "Hello";
                let inference_params = InferenceParams {
                    max_tokens: 10,
                    temperature: 0.7,
                    top_p: 0.9,
                    stream: false,
                    stop_sequences: vec![],
                    seed: None,
                };

                match backend.infer(test_input, &inference_params).await {
                    Ok(_) => {
                        if !ctx.json_output {
                            println!("  ✓ Model inference works");
                        }
                        result
                            .details
                            .insert("deep_validation".to_string(), true.into());
                        result
                            .details
                            .insert("inference_test".to_string(), "passed".into());
                    }
                    Err(e) => {
                        result.passed = false;
                        result.errors.push(format!("Model inference failed: {}", e));
                        if !ctx.json_output {
                            println!("  ✗ Model inference failed: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                result.passed = false;
                result.errors.push(format!("Model failed to load: {}", e));
                if !ctx.json_output {
                    println!("  ✗ Model failed to load: {}", e);
                }
            }
        }

        Ok(result)
    }
}

// ============================================================================
// Helper Types and Functions
// ============================================================================

#[derive(Default)]
struct FileValidationResult {
    passed: bool,
    details: std::collections::HashMap<String, serde_json::Value>,
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl FileValidationResult {
    fn merge(&mut self, other: FileValidationResult) {
        self.passed = self.passed && other.passed;
        self.details.extend(other.details);
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }

    fn merge_validation_result(&mut self, validation: &ValidationResult) {
        if !validation.is_valid {
            self.passed = false;
        }

        self.details
            .insert("file_readable".to_string(), validation.file_readable.into());
        self.details
            .insert("format_valid".to_string(), validation.format_valid.into());
        self.details
            .insert("size_valid".to_string(), validation.size_valid.into());
        self.details.insert(
            "security_valid".to_string(),
            validation.security_valid.into(),
        );
        self.details.insert(
            "metadata_valid".to_string(),
            validation.metadata_valid.into(),
        );

        if let Some(checksum_valid) = validation.checksum_valid {
            self.details
                .insert("checksum_valid".to_string(), checksum_valid.into());
        }

        self.errors.extend(validation.errors.clone());
        self.warnings.extend(validation.warnings.clone());
    }
}

fn print_validation_details(result: &ValidationResult, verbose: bool) {
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

fn print_validation_summary(result: &FileValidationResult) {
    println!("\nValidation Summary:");
    println!(
        "  Status: {}",
        if result.passed {
            "✓ Passed"
        } else {
            "✗ Failed"
        }
    );

    if !result.errors.is_empty() {
        println!("  Errors: {}", result.errors.len());
        for error in &result.errors {
            println!("    - {}", error);
        }
    }

    if !result.warnings.is_empty() {
        println!("  Warnings: {}", result.warnings.len());
        for warning in &result.warnings {
            println!("    - {}", warning);
        }
    }
}
