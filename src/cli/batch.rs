use crate::{
    backends::{Backend, BackendType, InferenceParams},
    batch::{BatchConfig, BatchOutputFormat, BatchProcessor},
    config::Config,
    metrics::MetricsCollector,
    models::ModelManager,
};
use anyhow::Result;
use clap::{Args, ValueEnum};
use std::{path::PathBuf, sync::Arc};
use tracing::{info, warn};

#[derive(Args)]
pub struct BatchArgs {
    #[arg(short, long, help = "Model file path or name")]
    pub model: String,

    #[arg(short, long, help = "Input file path (JSON, JSONL, CSV, TSV, or text)")]
    pub input: PathBuf,

    #[arg(short, long, help = "Output file path")]
    pub output: Option<PathBuf>,

    #[arg(long, help = "Output format", value_enum, default_value = "json-lines")]
    pub output_format: OutputFormat,

    #[arg(long, help = "Maximum tokens to generate", default_value = "512")]
    pub max_tokens: u32,

    #[arg(long, help = "Temperature for text generation", default_value = "0.7")]
    pub temperature: f32,

    #[arg(long, help = "Top-K for text generation", default_value = "40")]
    pub top_k: u32,

    #[arg(long, help = "Top-p for text generation", default_value = "0.9")]
    pub top_p: f32,

    #[arg(long, help = "Number of concurrent requests", default_value = "4")]
    pub concurrency: usize,

    #[arg(long, help = "Timeout per request in seconds", default_value = "300")]
    pub timeout: u64,

    #[arg(long, help = "Number of retry attempts", default_value = "3")]
    pub retries: u32,

    #[arg(
        long,
        help = "Checkpoint interval (save progress every N items)",
        default_value = "100"
    )]
    pub checkpoint: u32,

    #[arg(long, help = "Continue processing on individual failures")]
    pub continue_on_error: bool,

    #[arg(long, help = "Shuffle input order for better load balancing")]
    pub shuffle: bool,

    #[arg(long, help = "Enable metrics collection")]
    pub metrics: bool,

    #[arg(long, help = "Resume from checkpoint file")]
    pub resume: Option<PathBuf>,

    #[arg(long, help = "Dry run - validate inputs without processing")]
    pub dry_run: bool,

    #[arg(long, help = "Backend to use", value_enum)]
    pub backend: Option<BackendType>,

    #[arg(short, long, help = "Verbose output")]
    pub verbose: bool,
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    #[value(name = "json")]
    Json,
    #[value(name = "json-lines")]
    JsonLines,
    #[value(name = "csv")]
    Csv,
    #[value(name = "tsv")]
    Tsv,
}

impl From<OutputFormat> for BatchOutputFormat {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Json => BatchOutputFormat::Json,
            OutputFormat::JsonLines => BatchOutputFormat::JsonLines,
            OutputFormat::Csv => BatchOutputFormat::Csv,
            OutputFormat::Tsv => BatchOutputFormat::Tsv,
        }
    }
}

/// Validate batch processing parameters
fn validate_parameters(args: &BatchArgs) -> Result<()> {
    // Validate model name
    if args.model.is_empty() {
        anyhow::bail!("Model name cannot be empty");
    }

    // Validate input file
    if !args.input.exists() {
        anyhow::bail!("Input file does not exist: {}", args.input.display());
    }

    // Validate output directory if specified
    if let Some(ref output) = args.output {
        if let Some(parent) = output.parent() {
            if !parent.exists() {
                anyhow::bail!("Output directory does not exist: {}", parent.display());
            }
        }
    }

    // Validate resume checkpoint if specified
    if let Some(ref resume) = args.resume {
        if !resume.exists() {
            anyhow::bail!("Checkpoint file does not exist: {}", resume.display());
        }
    }

    // Validate parameter ranges
    if args.max_tokens == 0 {
        anyhow::bail!("Max tokens must be greater than 0");
    }

    if args.max_tokens > 32768 {
        anyhow::bail!("Max tokens cannot exceed 32768");
    }

    if args.temperature < 0.0 || args.temperature > 2.0 {
        anyhow::bail!("Temperature must be between 0.0 and 2.0");
    }

    if args.top_p < 0.0 || args.top_p > 1.0 {
        anyhow::bail!("Top-p must be between 0.0 and 1.0");
    }

    if args.concurrency == 0 {
        anyhow::bail!("Concurrency must be at least 1");
    }

    if args.concurrency > 128 {
        anyhow::bail!("Concurrency cannot exceed 128");
    }

    if args.timeout == 0 {
        anyhow::bail!("Timeout must be at least 1 second");
    }

    if args.checkpoint == 0 {
        anyhow::bail!("Checkpoint interval must be at least 1");
    }

    Ok(())
}

pub async fn execute(args: BatchArgs, config: &Config) -> Result<()> {
    info!("Starting batch processing with model: {}", args.model);

    // Validate all parameters
    validate_parameters(&args)?;

    if args.dry_run {
        return validate_batch_inputs(&args).await;
    }

    // Set up metrics if requested
    let metrics = if args.metrics {
        let (collector, processor) = MetricsCollector::new();
        processor.start();
        Some(Arc::new(collector))
    } else {
        None
    };

    // Create batch configuration
    let batch_config = BatchConfig {
        concurrency: args.concurrency,
        timeout_seconds: args.timeout,
        retry_attempts: args.retries,
        checkpoint_interval: args.checkpoint,
        output_format: args.output_format.clone().into(),
        continue_on_error: args.continue_on_error,
        shuffle_inputs: args.shuffle,
    };

    // Load and validate model
    let model_manager = ModelManager::new(&config.models_dir);
    let model_info = model_manager.resolve_model(&args.model).await?;

    info!("Validating model: {}", model_info.name);
    let validation_result = model_manager
        .validate_model_comprehensive(&model_info.path, Some(config))
        .await?;
    if !validation_result.is_valid {
        warn!("Model validation issues:");
        for error in &validation_result.errors {
            warn!("  - {}", error);
        }
        if !args.continue_on_error {
            return Err(anyhow::anyhow!("Model validation failed"));
        }
    }

    let backend_type = args
        .backend
        .or_else(|| BackendType::from_model_path(&model_info.path))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No suitable backend found for model: {}",
                model_info.path.display()
            )
        })?;

    let mut backend = Backend::new(backend_type, &config.backend_config)?;

    info!("Loading model...");
    let load_start = std::time::Instant::now();
    backend.load_model(&model_info).await?;
    let load_duration = load_start.elapsed();
    info!("Model loaded in {:?}", load_duration);

    // Record model load metrics
    if let Some(ref metrics) = metrics {
        metrics.record_model_loaded(
            model_info.name.clone(),
            model_info.size,
            load_duration,
            backend_type.to_string(),
        );
    }

    // Create inference parameters
    let inference_params = InferenceParams {
        max_tokens: args.max_tokens,
        temperature: args.temperature,
        top_k: args.top_k,
        top_p: args.top_p,
        stream: false, // Batch processing uses non-streaming
        stop_sequences: vec![],
        seed: None,
    };

    // Estimate total items for progress tracking
    let total_items = estimate_batch_size(&args.input).await?;
    info!("Estimated {} items to process", total_items);

    // Create batch processor
    let mut processor = BatchProcessor::new(batch_config, total_items);
    if let Some(metrics) = metrics {
        processor = processor.with_metrics(metrics);
    }

    // Determine output path
    let default_output = args.input.with_extension("batch.jsonl");
    let output_path = args.output.as_deref().unwrap_or(default_output.as_path());

    info!("Output will be saved to: {}", output_path.display());

    // Process the batch
    let progress = processor
        .process_file(
            &mut backend,
            &args.input,
            Some(output_path),
            &inference_params,
        )
        .await?;

    // Print summary
    print_batch_summary(&progress, &args);

    Ok(())
}

async fn validate_batch_inputs(args: &BatchArgs) -> Result<()> {
    info!("Validating batch inputs (dry run mode)");

    let batch_config = BatchConfig::default();
    let processor = BatchProcessor::new(batch_config, 0);

    match processor.load_inputs(&args.input).await {
        Ok(inputs) => {
            info!(
                "✓ Successfully parsed {} inputs from {}",
                inputs.len(),
                args.input.display()
            );

            if args.verbose {
                info!("Sample inputs:");
                for input in inputs.iter().take(3) {
                    info!(
                        "  {}: {} ({})",
                        input.id,
                        input.content.chars().take(50).collect::<String>(),
                        if input.content.len() > 50 { "..." } else { "" }
                    );
                }
                if inputs.len() > 3 {
                    info!("  ... and {} more", inputs.len() - 3);
                }
            }

            // Validate output path
            if let Some(output_path) = &args.output {
                if let Some(parent) = output_path.parent() {
                    if !parent.exists() {
                        warn!("Output directory does not exist: {}", parent.display());
                    }
                }
            }

            info!("✓ Batch validation complete - ready for processing");
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("Failed to parse batch inputs: {}", e)),
    }
}

async fn estimate_batch_size(input_path: &std::path::Path) -> Result<usize> {
    let content = tokio::fs::read_to_string(input_path).await?;
    let extension = input_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let count = match extension.to_lowercase().as_str() {
        "json" => {
            let value: serde_json::Value = serde_json::from_str(&content)?;
            match value {
                serde_json::Value::Array(ref items) => items.len(),
                _ => 1,
            }
        }
        "jsonl" | "ndjson" => content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count(),
        "csv" | "tsv" => {
            let delimiter = if extension == "tsv" { b'\t' } else { b',' };
            let mut rdr = csv::ReaderBuilder::new()
                .delimiter(delimiter)
                .from_reader(content.as_bytes());
            rdr.records().count()
        }
        _ => content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count(),
    };

    Ok(count)
}

fn print_batch_summary(progress: &crate::batch::BatchProgress, args: &BatchArgs) {
    println!("\n=== Batch Processing Summary ===");
    println!("Input file: {}", args.input.display());
    println!("Model: {}", args.model);
    println!("Total items: {}", progress.total_items);
    println!("Completed: {}", progress.completed_items);
    println!("Failed: {}", progress.failed_items);
    println!("Skipped: {}", progress.skipped_items);

    let success_rate = if progress.total_items > 0 {
        (progress.completed_items as f64 / progress.total_items as f64) * 100.0
    } else {
        0.0
    };
    println!("Success rate: {:.1}%", success_rate);

    if let Some(completion_time) = progress.estimated_completion {
        let duration = completion_time - progress.start_time;
        println!(
            "Processing time: {}",
            humantime::format_duration(duration.to_std().unwrap_or(std::time::Duration::ZERO))
        );
    }

    println!("Average rate: {:.2} items/second", progress.current_rate);

    if args.output.is_some() {
        println!(
            "Output saved to: {}",
            args.output.as_ref().unwrap().display()
        );
    }

    if progress.failed_items > 0 {
        println!("\n⚠️  {} items failed processing", progress.failed_items);
        if args.continue_on_error {
            println!("Failed items are included in output with error details");
        }
    }

    if progress.completed_items > 0 {
        println!("\n✅ Batch processing completed successfully!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Create default BatchArgs for testing with a valid temp file
    fn create_test_args_with_file(temp_file: &NamedTempFile) -> BatchArgs {
        BatchArgs {
            model: "test-model".to_string(),
            input: temp_file.path().to_path_buf(),
            output: None,
            output_format: OutputFormat::JsonLines,
            max_tokens: 512,
            temperature: 0.7,
            top_k: 40,
            top_p: 0.9,
            concurrency: 4,
            timeout: 300,
            retries: 3,
            checkpoint: 100,
            continue_on_error: false,
            shuffle: false,
            metrics: false,
            resume: None,
            dry_run: false,
            backend: None,
            verbose: false,
        }
    }

    #[test]
    fn test_validate_empty_model() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);
        args.model = String::new();

        let result = validate_parameters(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Model name cannot be empty"));
    }

    #[test]
    fn test_validate_max_tokens_zero() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);
        args.max_tokens = 0;

        let result = validate_parameters(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Max tokens must be greater than 0"));
    }

    #[test]
    fn test_validate_max_tokens_exceeds_limit() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);
        args.max_tokens = 32769;

        let result = validate_parameters(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Max tokens cannot exceed 32768"));
    }

    #[test]
    fn test_validate_max_tokens_boundary_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);

        // Test lower boundary
        args.max_tokens = 1;
        assert!(validate_parameters(&args).is_ok());

        // Test upper boundary
        args.max_tokens = 32768;
        assert!(validate_parameters(&args).is_ok());
    }

    #[test]
    fn test_validate_temperature_too_low() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);
        args.temperature = -0.1;

        let result = validate_parameters(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Temperature must be between 0.0 and 2.0"));
    }

    #[test]
    fn test_validate_temperature_too_high() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);
        args.temperature = 2.1;

        let result = validate_parameters(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Temperature must be between 0.0 and 2.0"));
    }

    #[test]
    fn test_validate_temperature_boundary_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);

        // Test lower boundary
        args.temperature = 0.0;
        assert!(validate_parameters(&args).is_ok());

        // Test upper boundary
        args.temperature = 2.0;
        assert!(validate_parameters(&args).is_ok());
    }

    #[test]
    fn test_validate_top_p_too_low() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);
        args.top_p = -0.1;

        let result = validate_parameters(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Top-p must be between 0.0 and 1.0"));
    }

    #[test]
    fn test_validate_top_p_too_high() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);
        args.top_p = 1.1;

        let result = validate_parameters(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Top-p must be between 0.0 and 1.0"));
    }

    #[test]
    fn test_validate_top_p_boundary_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);

        // Test lower boundary
        args.top_p = 0.0;
        assert!(validate_parameters(&args).is_ok());

        // Test upper boundary
        args.top_p = 1.0;
        assert!(validate_parameters(&args).is_ok());
    }

    #[test]
    fn test_validate_concurrency_zero() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);
        args.concurrency = 0;

        let result = validate_parameters(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Concurrency must be at least 1"));
    }

    #[test]
    fn test_validate_concurrency_exceeds_limit() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);
        args.concurrency = 129;

        let result = validate_parameters(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Concurrency cannot exceed 128"));
    }

    #[test]
    fn test_validate_concurrency_boundary_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);

        // Test lower boundary
        args.concurrency = 1;
        assert!(validate_parameters(&args).is_ok());

        // Test upper boundary
        args.concurrency = 128;
        assert!(validate_parameters(&args).is_ok());
    }

    #[test]
    fn test_validate_timeout_zero() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);
        args.timeout = 0;

        let result = validate_parameters(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Timeout must be at least 1 second"));
    }

    #[test]
    fn test_validate_timeout_boundary_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);

        // Test lower boundary
        args.timeout = 1;
        assert!(validate_parameters(&args).is_ok());
    }

    #[test]
    fn test_validate_checkpoint_zero() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);
        args.checkpoint = 0;

        let result = validate_parameters(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Checkpoint interval must be at least 1"));
    }

    #[test]
    fn test_validate_checkpoint_boundary_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let mut args = create_test_args_with_file(&temp_file);

        // Test lower boundary
        args.checkpoint = 1;
        assert!(validate_parameters(&args).is_ok());
    }

    #[test]
    fn test_validate_input_file_not_exists() {
        let args = BatchArgs {
            model: "test-model".to_string(),
            input: PathBuf::from("/nonexistent/path/to/file.json"),
            output: None,
            output_format: OutputFormat::JsonLines,
            max_tokens: 512,
            temperature: 0.7,
            top_k: 40,
            top_p: 0.9,
            concurrency: 4,
            timeout: 300,
            retries: 3,
            checkpoint: 100,
            continue_on_error: false,
            shuffle: false,
            metrics: false,
            resume: None,
            dry_run: false,
            backend: None,
            verbose: false,
        };

        let result = validate_parameters(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Input file does not exist"));
    }

    #[test]
    fn test_validate_all_parameters_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        let args = create_test_args_with_file(&temp_file);

        let result = validate_parameters(&args);
        assert!(result.is_ok());
    }
}
