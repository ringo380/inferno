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

pub async fn execute(args: BatchArgs, config: &Config) -> Result<()> {
    info!("Starting batch processing with model: {}", args.model);

    // Validate inputs
    if !args.input.exists() {
        return Err(anyhow::anyhow!(
            "Input file does not exist: {}",
            args.input.display()
        ));
    }

    if args.dry_run {
        return validate_batch_inputs(&args).await;
    }

    // Set up metrics if requested
    let metrics = if args.metrics {
        let mut collector = MetricsCollector::new();
        collector.start_event_processing().await?;
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
        Err(e) => {
            Err(anyhow::anyhow!("Failed to parse batch inputs: {}", e))
        }
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
