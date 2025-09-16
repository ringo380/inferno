use crate::config::Config;
use crate::models::ModelManager;
use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::info;

#[derive(Args)]
pub struct ModelsArgs {
    #[command(subcommand)]
    pub command: ModelsCommand,
}

#[derive(Subcommand)]
pub enum ModelsCommand {
    #[command(about = "List all available models")]
    List,

    #[command(about = "Show detailed information about a model")]
    Info {
        #[arg(help = "Model name or path")]
        model: String,
    },

    #[command(about = "Validate a model file")]
    Validate {
        #[arg(help = "Model file path")]
        path: PathBuf,
    },

    #[command(about = "Show model quantization information")]
    Quant {
        #[arg(help = "Model name or path")]
        model: String,
    },
}

pub async fn execute(args: ModelsArgs, config: &Config) -> Result<()> {
    let model_manager = ModelManager::new(&config.models_dir);

    match args.command {
        ModelsCommand::List => {
            info!("Scanning for models in: {}", config.models_dir.display());
            let models = model_manager.list_models().await?;

            if models.is_empty() {
                println!("No models found in: {}", config.models_dir.display());
                println!("Place GGUF (*.gguf) or ONNX (*.onnx) models in the models directory.");
                return Ok(());
            }

            println!("Available models:");
            println!("{:<30} {:<15} {:<20} {:<15}", "Name", "Type", "Size", "Modified");
            println!("{}", "─".repeat(80));

            for model in models {
                let size_str = format_size(model.size);
                let modified = model.modified.format("%Y-%m-%d %H:%M").to_string();
                println!(
                    "{:<30} {:<15} {:<20} {:<15}",
                    model.name, model.backend_type, size_str, modified
                );
            }
        }

        ModelsCommand::Info { model } => {
            let model_info = model_manager.resolve_model(&model).await?;
            println!("Model Information:");
            println!("  Name: {}", model_info.name);
            println!("  Path: {}", model_info.path.display());
            println!("  Type: {}", model_info.backend_type);
            println!("  Size: {}", format_size(model_info.size));
            println!("  Modified: {}", model_info.modified.format("%Y-%m-%d %H:%M:%S"));

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

        ModelsCommand::Validate { path } => {
            info!("Validating model: {}", path.display());
            let is_valid = model_manager.validate_model(&path).await?;

            if is_valid {
                println!("✓ Model is valid: {}", path.display());
            } else {
                println!("✗ Model validation failed: {}", path.display());
                std::process::exit(1);
            }
        }

        ModelsCommand::Quant { model } => {
            let model_info = model_manager.resolve_model(&model).await?;

            if model_info.backend_type == "gguf" {
                if let Ok(metadata) = model_manager.get_gguf_metadata(&model_info.path).await {
                    println!("Quantization Information:");
                    println!("  Method: {}", metadata.quantization);
                    println!("  Parameters: {}", format_params(metadata.parameter_count));
                    println!("  Estimated VRAM: {}", estimate_vram_usage(&metadata));
                }
            } else {
                println!("Quantization information only available for GGUF models");
            }
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

fn format_params(count: u64) -> String {
    if count >= 1_000_000_000 {
        format!("{:.1}B", count as f64 / 1_000_000_000.0)
    } else if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

fn estimate_vram_usage(metadata: &crate::models::GgufMetadata) -> String {
    // Rough estimation based on quantization and parameter count
    let base_gb = match metadata.quantization.as_str() {
        "Q4_0" | "Q4_1" => metadata.parameter_count as f64 * 0.5 / 1_000_000_000.0,
        "Q5_0" | "Q5_1" => metadata.parameter_count as f64 * 0.625 / 1_000_000_000.0,
        "Q8_0" => metadata.parameter_count as f64 * 1.0 / 1_000_000_000.0,
        _ => metadata.parameter_count as f64 * 2.0 / 1_000_000_000.0, // F16
    };

    format!("{:.1} GB", base_gb * 1.2) // Add 20% overhead
}