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

    #[command(about = "Search HuggingFace for models")]
    Search {
        #[arg(help = "Search query (e.g. 'llama' or 'mistral instruct')")]
        query: String,

        #[arg(long, help = "Filter by task (e.g. text-generation, fill-mask)")]
        task: Option<String>,

        #[arg(long, default_value = "10", help = "Max results to return")]
        limit: u32,
    },

    #[command(about = "Install a model from HuggingFace or a direct URL")]
    Install {
        #[arg(help = "HuggingFace repo ID (e.g. TheBloke/Llama-2-7B-GGUF) or direct HTTPS URL")]
        model: String,

        #[arg(long, help = "Specific filename to download from a HF repo")]
        file: Option<String>,

        #[arg(long, help = "Override the local filename")]
        name: Option<String>,
    },

    #[command(about = "Add tags to a local model")]
    Tag {
        #[arg(help = "Model name or path")]
        model: String,

        #[arg(help = "Tags to add", required = true)]
        tags: Vec<String>,
    },

    #[command(about = "Show usage statistics for local models")]
    Stats,
}

fn validate_command(command: &ModelsCommand, config: &Config) -> Result<()> {
    match command {
        ModelsCommand::List | ModelsCommand::Stats => {
            if !config.models_dir.exists() {
                anyhow::bail!(
                    "Models directory does not exist: {}\nCreate it or configure models_dir.",
                    config.models_dir.display()
                );
            }
        }
        ModelsCommand::Info { model } | ModelsCommand::Quant { model } => {
            if model.is_empty() {
                anyhow::bail!("Model name or path cannot be empty.");
            }
        }
        ModelsCommand::Validate { path } => {
            if !path.exists() {
                anyhow::bail!("Model file does not exist: {}", path.display());
            }
            if !path.is_file() {
                anyhow::bail!("Path is not a file: {}", path.display());
            }
        }
        ModelsCommand::Search { query, .. } => {
            if query.is_empty() {
                anyhow::bail!("Search query cannot be empty.");
            }
        }
        ModelsCommand::Install { model, .. } => {
            if model.is_empty() {
                anyhow::bail!("Model identifier cannot be empty.");
            }
        }
        ModelsCommand::Tag { model, tags } => {
            if model.is_empty() {
                anyhow::bail!("Model name or path cannot be empty.");
            }
            if tags.is_empty() {
                anyhow::bail!("Provide at least one tag.");
            }
        }
    }
    Ok(())
}

pub async fn execute(args: ModelsArgs, config: &Config) -> Result<()> {
    validate_command(&args.command, config)?;

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

            let registry = model_manager.load_registry().await.unwrap_or_default();

            println!("Available models:");
            println!(
                "{:<35} {:<8} {:<12} {:<20} {:<10} Tags",
                "Name", "Type", "Size", "Modified", "Uses"
            );
            println!("{}", "─".repeat(100));

            for model in models {
                let size_str = format_size(model.size);
                let modified = model.modified.format("%Y-%m-%d %H:%M").to_string();
                let key = model.path.to_string_lossy().to_string();
                let (uses, tags) = if let Some(entry) = registry.entries.get(&key) {
                    (entry.use_count.to_string(), entry.tags.join(", "))
                } else {
                    ("0".to_string(), String::new())
                };
                println!(
                    "{:<35} {:<8} {:<12} {:<20} {:<10} {}",
                    truncate(&model.name, 34),
                    model.backend_type,
                    size_str,
                    modified,
                    uses,
                    tags
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
            println!(
                "  Modified: {}",
                model_info.modified.format("%Y-%m-%d %H:%M:%S")
            );

            if let Some(checksum) = &model_info.checksum {
                println!("  SHA256: {}", checksum);
            }

            // Registry info
            let registry = model_manager.load_registry().await.unwrap_or_default();
            let key = model_info.path.to_string_lossy().to_string();
            if let Some(entry) = registry.entries.get(&key) {
                if !entry.tags.is_empty() {
                    println!("  Tags: {}", entry.tags.join(", "));
                }
                println!("  Uses: {}", entry.use_count);
                if let Some(last) = entry.last_used {
                    println!("  Last used: {}", last.format("%Y-%m-%d %H:%M:%S"));
                }
            }

            // Compatibility
            let compat = model_manager.check_compatibility(&model_info);
            println!(
                "  Est. RAM: {:.1} GB  (available: {:.1} GB)  {}",
                compat.estimated_ram_gb,
                compat.available_ram_gb,
                if compat.is_compatible {
                    "✓"
                } else {
                    "✗ may not fit"
                }
            );
            if let Some(warn) = compat.warning {
                println!("  ⚠  {}", warn);
            }

            // Backend-specific metadata
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

        ModelsCommand::Search { query, task, limit } => {
            println!("Searching HuggingFace for '{}'...", query);
            match search_huggingface(&query, task.as_deref(), limit).await {
                Ok(results) if results.is_empty() => {
                    println!("No results found.");
                }
                Ok(results) => {
                    println!(
                        "\n{:<45} {:<20} {:<12} {:<10} Tags",
                        "ID", "Author", "Downloads", "Likes"
                    );
                    println!("{}", "─".repeat(110));
                    for r in &results {
                        let tags = r
                            .tags
                            .iter()
                            .take(3)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ");
                        println!(
                            "{:<45} {:<20} {:<12} {:<10} {}",
                            truncate(&r.id, 44),
                            truncate(&r.author, 19),
                            r.downloads,
                            r.likes,
                            tags,
                        );
                    }
                    println!("\nRun `inferno models install <ID>` to install a model.");
                }
                Err(e) => {
                    anyhow::bail!("Search failed: {}", e);
                }
            }
        }

        ModelsCommand::Install { model, file, name } => {
            if !config.models_dir.exists() {
                async_std_create_dir(&config.models_dir).await?;
            }

            if model.starts_with("http://") || model.starts_with("https://") {
                // Direct URL download
                let filename = name
                    .clone()
                    .or_else(|| {
                        model
                            .rsplit('/')
                            .next()
                            .map(|s| s.split('?').next().unwrap_or(s).to_string())
                    })
                    .unwrap_or_else(|| "model.gguf".to_string());
                if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
                    anyhow::bail!("Invalid filename: path traversal not allowed");
                }
                let dest = config.models_dir.join(&filename);
                println!("Downloading {} → {}...", model, dest.display());
                download_to_file(&model, &dest).await?;
                post_install(&model_manager, &dest).await?;
            } else {
                // HuggingFace repo ID
                println!("Looking up '{}' on HuggingFace...", model);
                let files = list_hf_gguf_files(&model).await?;
                if files.is_empty() {
                    anyhow::bail!("No GGUF files found in repo '{}'.", model);
                }

                let chosen = if let Some(ref wanted) = file {
                    files
                        .iter()
                        .find(|(fname, _)| fname == wanted)
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "File '{}' not found in repo. Available:\n{}",
                                wanted,
                                files
                                    .iter()
                                    .map(|(f, _)| format!("  {}", f))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            )
                        })?
                } else if files.len() == 1 {
                    &files[0]
                } else {
                    println!("Available GGUF files in '{}':", model);
                    for (i, (fname, size)) in files.iter().enumerate() {
                        let size_str = size.map(format_size).unwrap_or_default();
                        println!("  [{}] {}  {}", i + 1, fname, size_str);
                    }
                    // Default to first file
                    println!("Selecting first file. Use --file <name> to choose.");
                    &files[0]
                };

                let (fname, _) = chosen;
                let out_name = name.clone().unwrap_or_else(|| fname.clone());
                if out_name.contains("..") || out_name.contains('/') || out_name.contains('\\') {
                    anyhow::bail!("Invalid filename: path traversal not allowed");
                }
                let dest = config.models_dir.join(&out_name);
                let url = format!("https://huggingface.co/{}/resolve/main/{}", model, fname);
                println!("Downloading {}...", fname);
                download_to_file(&url, &dest).await?;
                post_install(&model_manager, &dest).await?;
            }
        }

        ModelsCommand::Tag { model, tags } => {
            let model_info = model_manager.resolve_model(&model).await?;
            model_manager.tag_model(&model_info.path, &tags).await?;
            println!("Tagged '{}' with: {}", model_info.name, tags.join(", "));
        }

        ModelsCommand::Stats => {
            let registry = model_manager.load_registry().await.unwrap_or_default();
            if registry.entries.is_empty() {
                println!("No usage data recorded yet.");
                println!("Models are tracked after first inference run.");
                return Ok(());
            }

            let mut entries: Vec<_> = registry.entries.values().collect();
            entries.sort_by(|a, b| b.use_count.cmp(&a.use_count));

            println!("{:<35} {:<10} {:<25} Tags", "Model", "Uses", "Last Used");
            println!("{}", "─".repeat(90));
            for entry in entries {
                let last_used = entry
                    .last_used
                    .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "never".to_string());
                println!(
                    "{:<35} {:<10} {:<25} {}",
                    truncate(&entry.name, 34),
                    entry.use_count,
                    last_used,
                    entry.tags.join(", ")
                );
            }
        }
    }

    Ok(())
}

// ── HuggingFace helpers ───────────────────────────────────────────────────────

#[derive(Debug)]
struct HfModelResult {
    id: String,
    author: String,
    downloads: u64,
    likes: u64,
    tags: Vec<String>,
}

async fn search_huggingface(
    query: &str,
    task: Option<&str>,
    limit: u32,
) -> Result<Vec<HfModelResult>> {
    let client = reqwest::Client::builder()
        .user_agent("inferno/1.0")
        .build()?;

    let mut req = client
        .get("https://huggingface.co/api/models")
        .query(&[
            ("search", query),
            ("sort", "downloads"),
            ("direction", "-1"),
        ])
        .query(&[("limit", &limit.to_string())]);

    if let Some(t) = task {
        req = req.query(&[("pipeline_tag", t)]);
    }

    let resp = req.send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("HuggingFace API returned {}", resp.status());
    }

    let raw: serde_json::Value = resp.json().await?;
    let arr = raw
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Unexpected API response format"))?;

    let mut results = Vec::new();
    for item in arr {
        let id = item["id"].as_str().unwrap_or("").to_string();
        if id.is_empty() {
            continue;
        }
        let author = item["author"].as_str().unwrap_or("").to_string();
        let downloads = item["downloads"].as_u64().unwrap_or(0);
        let likes = item["likes"].as_u64().unwrap_or(0);
        let tags = item["tags"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        results.push(HfModelResult {
            id,
            author,
            downloads,
            likes,
            tags,
        });
    }
    Ok(results)
}

/// List `.gguf` files in a HuggingFace repo, returning (filename, optional_size).
async fn list_hf_gguf_files(repo_id: &str) -> Result<Vec<(String, Option<u64>)>> {
    let client = reqwest::Client::builder()
        .user_agent("inferno/1.0")
        .build()?;
    let url = format!("https://huggingface.co/api/models/{}", repo_id);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("Cannot fetch repo '{}': HTTP {}", repo_id, resp.status());
    }

    let raw: serde_json::Value = resp.json().await?;
    let siblings = raw["siblings"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No 'siblings' in repo response"))?;

    let files = siblings
        .iter()
        .filter_map(|s| {
            let name = s["rfilename"].as_str()?;
            if name.ends_with(".gguf") {
                let size = s["size"].as_u64();
                Some((name.to_string(), size))
            } else {
                None
            }
        })
        .collect();

    Ok(files)
}

/// Stream-download a URL to a local file with progress reporting.
/// Removes the partial file if any error occurs mid-download.
async fn download_to_file(url: &str, dest: &PathBuf) -> Result<()> {
    let result = async {
        let client = reqwest::Client::builder()
            .user_agent("inferno/1.0")
            .build()?;
        let resp = client.get(url).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("Download failed: HTTP {}", resp.status());
        }

        let total = resp.content_length();
        let mut downloaded: u64 = 0;

        let mut file = tokio::fs::File::create(dest).await?;
        let mut stream = resp.bytes_stream();
        use futures::StreamExt;
        use tokio::io::AsyncWriteExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            if let Some(total) = total {
                let pct = downloaded * 100 / total;
                print!(
                    "\r  {:.1} MB / {:.1} MB  ({}%)",
                    mb(downloaded),
                    mb(total),
                    pct
                );
            } else {
                print!("\r  {:.1} MB downloaded", mb(downloaded));
            }
        }
        println!(); // newline after progress
        file.flush().await?;
        Ok(())
    }
    .await;

    if result.is_err() {
        let _ = tokio::fs::remove_file(dest).await;
    }
    result
}

async fn async_std_create_dir(path: &PathBuf) -> Result<()> {
    tokio::fs::create_dir_all(path).await?;
    Ok(())
}

/// Validate newly installed model and register it.
async fn post_install(manager: &ModelManager, path: &PathBuf) -> Result<()> {
    print!("Validating...");
    let valid = manager.validate_model(path).await?;
    if valid {
        println!(" ✓");
        manager.register_model(path).await?;
        println!("Installed: {}", path.display());
    } else {
        println!(" ✗");
        tokio::fs::remove_file(path).await.ok();
        anyhow::bail!("Downloaded file failed validation — removed.");
    }
    Ok(())
}

fn mb(bytes: u64) -> f64 {
    bytes as f64 / 1_048_576.0
}

// ── Formatting helpers ────────────────────────────────────────────────────────

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
    let base_gb = match metadata.quantization.as_str() {
        "Q4_0" | "Q4_1" | "Q4_K_S" | "Q4_K_M" => {
            metadata.parameter_count as f64 * 0.5 / 1_000_000_000.0
        }
        "Q5_0" | "Q5_1" | "Q5_K_S" | "Q5_K_M" => {
            metadata.parameter_count as f64 * 0.625 / 1_000_000_000.0
        }
        "Q6_K" => metadata.parameter_count as f64 * 0.75 / 1_000_000_000.0,
        "Q8_0" => metadata.parameter_count as f64 * 1.0 / 1_000_000_000.0,
        _ => metadata.parameter_count as f64 * 2.0 / 1_000_000_000.0,
    };
    format!("{:.1} GB", base_gb * 1.2)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0.0 B");
        assert_eq!(format_size(512), "512.0 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1_048_576), "1.0 MB");
        assert_eq!(format_size(1_572_864), "1.5 MB");
        assert_eq!(format_size(1_073_741_824), "1.0 GB");
        assert_eq!(format_size(1_610_612_736), "1.5 GB");
    }

    #[test]
    fn test_format_params() {
        assert_eq!(format_params(500), "500");
        assert_eq!(format_params(1_500), "1.5K");
        assert_eq!(format_params(7_000_000), "7.0M");
        assert_eq!(format_params(13_000_000_000), "13.0B");
    }

    #[test]
    fn test_format_size_edge_cases() {
        assert_eq!(format_size(1023), "1023.0 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024 - 1), "1024.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn test_format_params_edge_cases() {
        assert_eq!(format_params(999), "999");
        assert_eq!(format_params(1_000), "1.0K");
        assert_eq!(format_params(999_999), "1000.0K");
        assert_eq!(format_params(1_000_000), "1.0M");
        assert_eq!(format_params(999_999_999), "1000.0M");
        assert_eq!(format_params(1_000_000_000), "1.0B");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 5), "hell…");
    }
}
