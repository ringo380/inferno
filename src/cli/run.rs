#![allow(dead_code, unused_imports, unused_variables)]
use crate::backends::{Backend, BackendType};
use crate::config::Config;
use crate::io::{InputFormat, OutputFormat};
use crate::models::ModelManager;
use anyhow::Result;
use clap::Args;
use futures::StreamExt;
use std::path::PathBuf;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tracing::{info, warn};

#[derive(Args)]
pub struct RunArgs {
    #[arg(short, long, help = "Model file path or name")]
    pub model: String,

    #[arg(
        short = 't',
        long,
        help = "Input type",
        value_enum,
        default_value = "text"
    )]
    pub input_type: InputFormat,

    #[arg(
        short = 'o',
        long,
        help = "Output format",
        value_enum,
        default_value = "text"
    )]
    pub output_format: OutputFormat,

    #[arg(
        short,
        long,
        help = "Input file path (if not provided, reads from stdin)"
    )]
    pub input: Option<PathBuf>,

    #[arg(long, help = "Output file path (if not provided, writes to stdout)")]
    pub output: Option<PathBuf>,

    #[arg(short, long, help = "Prompt text for text generation")]
    pub prompt: Option<String>,

    #[arg(long, help = "Maximum tokens to generate", default_value = "512")]
    pub max_tokens: u32,

    #[arg(long, help = "Temperature for text generation", default_value = "0.7")]
    pub temperature: f32,

    #[arg(long, help = "Top-k for text generation", default_value = "40")]
    pub top_k: u32,

    #[arg(long, help = "Top-p for text generation", default_value = "0.9")]
    pub top_p: f32,

    #[arg(long, help = "Enable streaming output")]
    pub stream: bool,

    #[arg(short, long, help = "Process input in batch mode")]
    pub batch: bool,

    #[arg(long, help = "Backend to use", value_enum)]
    pub backend: Option<BackendType>,
}

pub async fn execute(args: RunArgs, config: &Config) -> Result<()> {
    // Validate parameters
    if args.model.is_empty() {
        anyhow::bail!("Model name cannot be empty");
    }
    if args.max_tokens == 0 {
        anyhow::bail!("max_tokens must be greater than 0");
    }
    if !(0.0..=2.0).contains(&args.temperature) {
        anyhow::bail!("temperature must be between 0.0 and 2.0");
    }
    if !(0.0..=1.0).contains(&args.top_p) {
        anyhow::bail!("top_p must be between 0.0 and 1.0");
    }

    info!("Running inference with model: {}", args.model);

    let model_manager = ModelManager::new(&config.models_dir);
    let model_info = model_manager.resolve_model(&args.model).await?;

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
    backend.load_model(&model_info).await?;

    if args.batch {
        // Use enhanced batch processing
        use crate::batch::{BatchConfig, BatchProcessor};

        let batch_config = BatchConfig {
            concurrency: 1, // Keep single-threaded for run command compatibility
            timeout_seconds: 300,
            retry_attempts: 3,
            checkpoint_interval: 50,
            output_format: crate::batch::BatchOutputFormat::JsonLines,
            continue_on_error: true,
            shuffle_inputs: false,
        };

        let input_path = args
            .input
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Batch mode requires input file"))?;

        let total_items = estimate_batch_size(input_path).await?;
        let processor = BatchProcessor::new(batch_config, total_items);

        let inference_params = crate::backends::InferenceParams {
            max_tokens: args.max_tokens,
            temperature: args.temperature,
            top_k: args.top_k,
            top_p: args.top_p,
            stream: false,
            stop_sequences: vec![],
            seed: None,
        };

        let progress = processor
            .process_file(
                &mut backend,
                input_path,
                args.output.as_deref(),
                &inference_params,
            )
            .await?;

        info!(
            "Batch processing completed: {}/{} items processed",
            progress.completed_items, progress.total_items
        );
    } else {
        process_single(&mut backend, &args, config).await?;
    }

    Ok(())
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

async fn process_single(backend: &mut Backend, args: &RunArgs, _config: &Config) -> Result<()> {
    let input = if let Some(prompt) = &args.prompt {
        prompt.clone()
    } else if let Some(input_path) = &args.input {
        tokio::fs::read_to_string(input_path).await?
    } else {
        // Read all lines from stdin
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();
        let mut input = String::new();

        while let Some(line) = lines.next_line().await? {
            input.push_str(&line);
            input.push('\n');
        }

        input
    };

    if input.is_empty() {
        warn!("No input provided");
        return Ok(());
    }

    let inference_params = crate::backends::InferenceParams {
        max_tokens: args.max_tokens,
        temperature: args.temperature,
        top_k: args.top_k,
        top_p: args.top_p,
        stream: args.stream,
        stop_sequences: vec![],
        seed: None,
    };

    let start = std::time::Instant::now();

    if args.stream {
        let mut stream = backend.infer_stream(&input, &inference_params).await?;
        while let Some(token) = stream.next().await {
            match token {
                Ok(t) => {
                    print!("{}", t);
                    use std::io::Write;
                    std::io::stdout().flush()?;
                }
                Err(e) => {
                    eprintln!("Stream error: {}", e);
                    break;
                }
            }
        }
        println!();
    } else {
        let result = backend.infer(&input, &inference_params).await?;
        if let Some(output_path) = &args.output {
            tokio::fs::write(output_path, &result).await?;
            info!("Output written to: {}", output_path.display());
        } else {
            println!("{}", result);
        }
    }

    let elapsed = start.elapsed();
    info!("Inference completed in {:.2}s", elapsed.as_secs_f64());

    Ok(())
}

async fn process_batch(backend: &mut Backend, args: &RunArgs, _config: &Config) -> Result<()> {
    let input_path = args
        .input
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Batch mode requires input file"))?;

    let content = tokio::fs::read_to_string(input_path).await?;
    let lines: Vec<&str> = content.lines().collect();

    info!("Processing {} inputs in batch mode", lines.len());

    let inference_params = crate::backends::InferenceParams {
        max_tokens: args.max_tokens,
        temperature: args.temperature,
        top_k: args.top_k,
        top_p: args.top_p,
        stream: false, // No streaming in batch mode
        stop_sequences: vec![],
        seed: None,
    };

    let mut results = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        info!("Processing batch item {}/{}", i + 1, lines.len());
        let result = backend.infer(line.trim(), &inference_params).await?;
        results.push(serde_json::json!({
            "input": line.trim(),
            "output": result,
            "index": i
        }));
    }

    let output_json = serde_json::to_string_pretty(&results)?;

    if let Some(output_path) = &args.output {
        tokio::fs::write(output_path, &output_json).await?;
        info!("Batch results written to: {}", output_path.display());
    } else {
        println!("{}", output_json);
    }

    Ok(())
}
