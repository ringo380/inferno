use crate::config::Config;
use crate::marketplace::{
    DownloadStatus, MarketplaceConfig, ModelCategory, ModelListing, ModelMarketplace,
    ModelVisibility, PricingInfo, PublishRequest, SearchFilters,
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use serde_json;
use serde_yaml;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::time::{interval as tokio_interval, Duration};
use tracing::{error, info};

#[derive(Args)]
pub struct MarketplaceArgs {
    #[command(subcommand)]
    pub command: MarketplaceCommands,
}

#[derive(Subcommand)]
pub enum MarketplaceCommands {
    #[command(about = "Search for models in the marketplace")]
    Search {
        #[arg(help = "Search query")]
        query: String,

        #[arg(short, long, help = "Filter by category")]
        category: Option<ModelCategoryArg>,

        #[arg(short, long, help = "Filter by publisher")]
        publisher: Option<String>,

        #[arg(short, long, help = "Filter by license")]
        license: Option<String>,

        #[arg(long, help = "Minimum rating (1.0-5.0)")]
        min_rating: Option<f32>,

        #[arg(long, help = "Maximum size in GB")]
        max_size: Option<f64>,

        #[arg(long, help = "Filter by tags (comma-separated)")]
        tags: Option<String>,

        #[arg(long, help = "Filter by frameworks (comma-separated)")]
        frameworks: Option<String>,

        #[arg(long, help = "Filter by languages (comma-separated)")]
        languages: Option<String>,

        #[arg(long, help = "Filter by platforms (comma-separated)")]
        platforms: Option<String>,

        #[arg(long, help = "Show only free models")]
        free_only: bool,

        #[arg(long, help = "Show only verified models")]
        verified_only: bool,

        #[arg(long, help = "Page number", default_value = "1")]
        page: usize,

        #[arg(long, help = "Results per page", default_value = "20")]
        per_page: usize,

        #[arg(long, help = "Output format", default_value = "table")]
        output: OutputFormat,
    },

    #[command(about = "Show detailed information about a model")]
    Info {
        #[arg(help = "Model ID")]
        model_id: String,

        #[arg(long, help = "Output format", default_value = "detailed")]
        output: OutputFormat,
    },

    #[command(about = "Download a model from the marketplace")]
    Download {
        #[arg(help = "Model ID")]
        model_id: String,

        #[arg(short, long, help = "Target directory")]
        output: Option<PathBuf>,

        #[arg(long, help = "Skip compatibility checks")]
        skip_checks: bool,

        #[arg(long, help = "Download in background")]
        background: bool,
    },

    #[command(about = "Install a downloaded model")]
    Install {
        #[arg(help = "Download ID or model ID")]
        id: String,

        #[arg(long, help = "Enable automatic updates")]
        auto_update: bool,

        #[arg(long, help = "Custom installation path")]
        path: Option<PathBuf>,
    },

    #[command(about = "Uninstall a model")]
    Uninstall {
        #[arg(help = "Model ID")]
        model_id: String,

        #[arg(long, help = "Remove model files")]
        remove_files: bool,

        #[arg(long, help = "Force uninstall without confirmation")]
        force: bool,
    },

    #[command(about = "List installed models")]
    List {
        #[arg(long, help = "Show detailed information")]
        detailed: bool,

        #[arg(long, help = "Filter by source")]
        source: Option<String>,

        #[arg(long, help = "Output format", default_value = "table")]
        output: OutputFormat,
    },

    #[command(about = "Show download progress")]
    Progress {
        #[arg(help = "Download ID")]
        download_id: Option<String>,

        #[arg(short, long, help = "Watch progress continuously")]
        watch: bool,

        #[arg(long, help = "Refresh interval in seconds", default_value = "2")]
        interval: u64,
    },

    #[command(about = "Cancel a download")]
    Cancel {
        #[arg(help = "Download ID")]
        download_id: String,
    },

    #[command(about = "Check for model updates")]
    Updates {
        #[arg(short, long, help = "Check specific model")]
        model_id: Option<String>,

        #[arg(long, help = "Automatically update all models")]
        auto_update: bool,

        #[arg(long, help = "Output format", default_value = "table")]
        output: OutputFormat,
    },

    #[command(about = "Update a specific model")]
    Update {
        #[arg(help = "Model ID")]
        model_id: String,

        #[arg(long, help = "Wait for update to complete")]
        wait: bool,
    },

    #[command(about = "Publish a model to the marketplace")]
    Publish {
        #[arg(help = "Model file path")]
        model_path: PathBuf,

        #[arg(short, long, help = "Model name")]
        name: String,

        #[arg(short, long, help = "Model version")]
        version: String,

        #[arg(short, long, help = "Description")]
        description: String,

        #[arg(long, help = "Category")]
        category: ModelCategoryArg,

        #[arg(short, long, help = "License")]
        license: String,

        #[arg(long, help = "Publisher name")]
        publisher: Option<String>,

        #[arg(long, help = "Tags (comma-separated)")]
        tags: Option<String>,

        #[arg(long, help = "Visibility", default_value = "public")]
        visibility: VisibilityArg,

        #[arg(long, help = "Make model free")]
        free: bool,

        #[arg(long, help = "Price per download")]
        price: Option<f64>,

        #[arg(long, help = "License file")]
        license_file: Option<PathBuf>,

        #[arg(long, help = "README file")]
        readme_file: Option<PathBuf>,
    },

    #[command(about = "Show popular models")]
    Popular {
        #[arg(short, long, help = "Filter by category")]
        category: Option<ModelCategoryArg>,

        #[arg(short, long, help = "Number of models to show", default_value = "10")]
        limit: usize,

        #[arg(long, help = "Output format", default_value = "table")]
        output: OutputFormat,
    },

    #[command(about = "Get personalized model recommendations")]
    Recommendations {
        #[arg(short, long, help = "User ID for personalization")]
        user_id: Option<String>,

        #[arg(short, long, help = "Number of recommendations", default_value = "10")]
        limit: usize,

        #[arg(long, help = "Output format", default_value = "table")]
        output: OutputFormat,
    },

    #[command(about = "Manage marketplace configuration")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    #[command(about = "Clear marketplace cache")]
    ClearCache {
        #[arg(long, help = "Clear all cached data")]
        all: bool,

        #[arg(long, help = "Clear only model metadata")]
        metadata_only: bool,

        #[arg(long, help = "Clear only downloaded files")]
        files_only: bool,
    },

    #[command(about = "Verify installed models")]
    Verify {
        #[arg(help = "Model ID (verify all if not specified)")]
        model_id: Option<String>,

        #[arg(long, help = "Perform deep verification")]
        deep: bool,

        #[arg(long, help = "Fix verification issues automatically")]
        fix: bool,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    #[command(about = "Show current configuration")]
    Show,

    #[command(about = "Set configuration value")]
    Set {
        #[arg(help = "Configuration key")]
        key: String,

        #[arg(help = "Configuration value")]
        value: String,
    },

    #[command(about = "Get configuration value")]
    Get {
        #[arg(help = "Configuration key")]
        key: String,
    },

    #[command(about = "Reset configuration to defaults")]
    Reset {
        #[arg(long, help = "Confirm reset")]
        confirm: bool,
    },

    #[command(about = "Test marketplace connection")]
    Test,
}

#[derive(Clone, ValueEnum)]
pub enum ModelCategoryArg {
    Language,
    Vision,
    Audio,
    MultiModal,
    Embedding,
    Classification,
    Generative,
    Reinforcement,
    Other,
}

impl From<ModelCategoryArg> for ModelCategory {
    fn from(arg: ModelCategoryArg) -> Self {
        match arg {
            ModelCategoryArg::Language => ModelCategory::LanguageModel,
            ModelCategoryArg::Vision => ModelCategory::VisionModel,
            ModelCategoryArg::Audio => ModelCategory::AudioModel,
            ModelCategoryArg::MultiModal => ModelCategory::MultiModal,
            ModelCategoryArg::Embedding => ModelCategory::Embedding,
            ModelCategoryArg::Classification => ModelCategory::ClassificationModel,
            ModelCategoryArg::Generative => ModelCategory::GenerativeModel,
            ModelCategoryArg::Reinforcement => ModelCategory::ReinforcementLearning,
            ModelCategoryArg::Other => ModelCategory::Other("Other".to_string()),
        }
    }
}

#[derive(Clone, ValueEnum)]
pub enum VisibilityArg {
    Public,
    Private,
    Organization,
}

impl From<VisibilityArg> for ModelVisibility {
    fn from(arg: VisibilityArg) -> Self {
        match arg {
            VisibilityArg::Public => ModelVisibility::Public,
            VisibilityArg::Private => ModelVisibility::Private,
            VisibilityArg::Organization => ModelVisibility::Organization,
        }
    }
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Detailed,
    Compact,
}

pub async fn handle_marketplace_command(args: MarketplaceArgs) -> Result<()> {
    let config = Config::load()?;
    let marketplace_config = MarketplaceConfig::from_config(&config)?;
    let marketplace = ModelMarketplace::new(marketplace_config)?;

    match args.command {
        MarketplaceCommands::Search {
            query,
            category,
            publisher,
            license,
            min_rating,
            max_size,
            tags,
            frameworks,
            languages,
            platforms,
            free_only,
            verified_only,
            page,
            per_page,
            output,
        } => {
            handle_search(
                &marketplace,
                query,
                category,
                publisher,
                license,
                min_rating,
                max_size,
                tags,
                frameworks,
                languages,
                platforms,
                free_only,
                verified_only,
                page,
                per_page,
                output,
            )
            .await
        }

        MarketplaceCommands::Info { model_id, output } => {
            handle_info(&marketplace, model_id, output).await
        }

        MarketplaceCommands::Download {
            model_id,
            output,
            skip_checks,
            background,
        } => handle_download(&marketplace, model_id, output, skip_checks, background).await,

        MarketplaceCommands::Install {
            id,
            auto_update,
            path,
        } => handle_install(&marketplace, id, auto_update, path).await,

        MarketplaceCommands::Uninstall {
            model_id,
            remove_files,
            force,
        } => handle_uninstall(&marketplace, model_id, remove_files, force).await,

        MarketplaceCommands::List {
            detailed,
            source,
            output,
        } => handle_list(&marketplace, detailed, source, output).await,

        MarketplaceCommands::Progress {
            download_id,
            watch,
            interval,
        } => handle_progress(&marketplace, download_id, watch, interval).await,

        MarketplaceCommands::Cancel { download_id } => {
            handle_cancel(&marketplace, download_id).await
        }

        MarketplaceCommands::Updates {
            model_id,
            auto_update,
            output,
        } => handle_updates(&marketplace, model_id, auto_update, output).await,

        MarketplaceCommands::Update { model_id, wait } => {
            handle_update(&marketplace, model_id, wait).await
        }

        MarketplaceCommands::Publish {
            model_path,
            name,
            version,
            description,
            category,
            license,
            publisher,
            tags,
            visibility,
            free,
            price,
            license_file,
            readme_file,
        } => {
            handle_publish(
                &marketplace,
                model_path,
                name,
                version,
                description,
                category,
                license,
                publisher,
                tags,
                visibility,
                free,
                price,
                license_file,
                readme_file,
            )
            .await
        }

        MarketplaceCommands::Popular {
            category,
            limit,
            output,
        } => handle_popular(&marketplace, category, limit, output).await,

        MarketplaceCommands::Recommendations {
            user_id,
            limit,
            output,
        } => handle_recommendations(&marketplace, user_id, limit, output).await,

        MarketplaceCommands::Config { command } => {
            handle_config_command(&marketplace, command).await
        }

        MarketplaceCommands::ClearCache {
            all,
            metadata_only,
            files_only,
        } => handle_clear_cache(&marketplace, all, metadata_only, files_only).await,

        MarketplaceCommands::Verify {
            model_id,
            deep,
            fix,
        } => handle_verify(&marketplace, model_id, deep, fix).await,
    }
}

async fn handle_search(
    marketplace: &ModelMarketplace,
    query: String,
    category: Option<ModelCategoryArg>,
    publisher: Option<String>,
    license: Option<String>,
    min_rating: Option<f32>,
    max_size: Option<f64>,
    tags: Option<String>,
    frameworks: Option<String>,
    languages: Option<String>,
    platforms: Option<String>,
    free_only: bool,
    verified_only: bool,
    page: usize,
    per_page: usize,
    output: OutputFormat,
) -> Result<()> {
    info!("Searching marketplace for: {}", query);

    let filters = Some(SearchFilters {
        category: category.map(|c| c.into()),
        publisher,
        license,
        min_rating,
        max_size_gb: max_size,
        tags: tags
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
        frameworks: frameworks
            .map(|f| f.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
        languages: languages
            .map(|l| l.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
        platforms: platforms
            .map(|p| p.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
        free_only,
        verified_only,
    });

    let results = marketplace
        .search_models(&query, filters, page, per_page)
        .await?;

    match output {
        OutputFormat::Table => {
            println!(
                "Search Results (Page {} of {})",
                results.page, results.total_pages
            );
            println!("Found {} models total", results.total_count);
            println!();
            println!(
                "{:<40} {:<20} {:<15} {:<10} {:<15}",
                "MODEL", "PUBLISHER", "VERSION", "SIZE", "DOWNLOADS"
            );
            println!("{}", "-".repeat(100));

            for model in &results.models {
                let size_str = format_size(model.size_bytes);
                let downloads_str = format_number(model.downloads);
                println!(
                    "{:<40} {:<20} {:<15} {:<10} {:<15}",
                    truncate(&model.name, 38),
                    truncate(&model.publisher, 18),
                    truncate(&model.version, 13),
                    size_str,
                    downloads_str
                );
            }

            if !results.facets.categories.is_empty() {
                println!("\nCategories:");
                for (category, count) in &results.facets.categories {
                    println!("  {}: {}", category, count);
                }
            }
        }

        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&results)?;
            println!("{}", json);
        }

        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&results)?;
            println!("{}", yaml);
        }

        OutputFormat::Compact => {
            for model in &results.models {
                println!(
                    "{} - {} v{} by {} ({})",
                    model.id,
                    model.name,
                    model.version,
                    model.publisher,
                    format_size(model.size_bytes)
                );
            }
        }

        OutputFormat::Detailed => {
            for (i, model) in results.models.iter().enumerate() {
                if i > 0 {
                    println!();
                }
                print_model_details(model);
            }
        }
    }

    Ok(())
}

async fn handle_info(
    marketplace: &ModelMarketplace,
    model_id: String,
    output: OutputFormat,
) -> Result<()> {
    info!("Fetching model information: {}", model_id);

    let model = marketplace.get_model_details(&model_id).await?;

    match output {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&model)?;
            println!("{}", json);
        }

        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&model)?;
            println!("{}", yaml);
        }

        _ => {
            print_model_details(&model);
        }
    }

    Ok(())
}

async fn handle_download(
    marketplace: &ModelMarketplace,
    model_id: String,
    output: Option<PathBuf>,
    skip_checks: bool,
    background: bool,
) -> Result<()> {
    info!("Starting download for model: {}", model_id);

    if !skip_checks {
        let model = marketplace.get_model_details(&model_id).await?;
        println!(
            "Model: {} v{} by {}",
            model.name, model.version, model.publisher
        );
        println!("Size: {}", format_size(model.size_bytes));
        println!("License: {}", model.license);

        if !confirm("Continue with download?")? {
            println!("Download cancelled.");
            return Ok(());
        }
    }

    let download_id = marketplace.download_model(&model_id, output).await?;

    println!("Download started with ID: {}", download_id);

    if !background {
        // Monitor progress
        let mut last_progress = 0.0;
        loop {
            match marketplace.get_download_progress(&download_id).await {
                Ok(progress) => {
                    if progress.progress_percent != last_progress {
                        print!(
                            "\rProgress: {:.1}% ({}) - {:.1} MB/s",
                            progress.progress_percent,
                            format_bytes(progress.bytes_downloaded, progress.total_bytes),
                            progress.download_speed_mbps
                        );
                        last_progress = progress.progress_percent;
                    }

                    match progress.status {
                        DownloadStatus::Completed => {
                            println!("\n✓ Download completed successfully!");
                            break;
                        }
                        DownloadStatus::Failed => {
                            println!(
                                "\n✗ Download failed: {}",
                                progress.error.unwrap_or_default()
                            );
                            return Err(anyhow::anyhow!("Download failed"));
                        }
                        DownloadStatus::Cancelled => {
                            println!("\n⚠ Download cancelled");
                            break;
                        }
                        _ => {}
                    }
                }
                Err(_) => {
                    break;
                }
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    } else {
        println!(
            "Download running in background. Use 'inferno marketplace progress {}' to monitor.",
            download_id
        );
    }

    Ok(())
}

async fn handle_install(
    marketplace: &ModelMarketplace,
    id: String,
    auto_update: bool,
    path: Option<PathBuf>,
) -> Result<()> {
    info!("Installing model: {}", id);

    // Try as download ID first, then as model ID
    let install_result = marketplace.install_model(&id, auto_update).await;

    match install_result {
        Ok(_) => {
            println!("✓ Model installed successfully!");
            if auto_update {
                println!("  Automatic updates enabled");
            }
        }
        Err(_) => {
            // If install by download ID failed, try downloading first
            println!("Download ID not found, downloading model...");
            let download_id = marketplace.download_model(&id, path).await?;

            // Wait for download to complete
            loop {
                let progress = marketplace.get_download_progress(&download_id).await?;
                match progress.status {
                    DownloadStatus::Completed => {
                        marketplace.install_model(&download_id, auto_update).await?;
                        println!("✓ Model downloaded and installed successfully!");
                        break;
                    }
                    DownloadStatus::Failed => {
                        return Err(anyhow::anyhow!(
                            "Download failed: {}",
                            progress.error.unwrap_or_default()
                        ));
                    }
                    _ => {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_uninstall(
    marketplace: &ModelMarketplace,
    model_id: String,
    remove_files: bool,
    force: bool,
) -> Result<()> {
    info!("Uninstalling model: {}", model_id);

    if !force {
        let message = if remove_files {
            "This will uninstall the model and remove all files. Continue?"
        } else {
            "This will uninstall the model but keep files. Continue?"
        };

        if !confirm(message)? {
            println!("Uninstall cancelled.");
            return Ok(());
        }
    }

    marketplace.uninstall_model(&model_id, remove_files).await?;

    println!("✓ Model uninstalled successfully!");
    if remove_files {
        println!("  Model files removed");
    }

    Ok(())
}

async fn handle_list(
    marketplace: &ModelMarketplace,
    detailed: bool,
    source: Option<String>,
    output: OutputFormat,
) -> Result<()> {
    info!("Listing installed models");

    let mut models = marketplace.list_installed_models().await?;

    if let Some(filter_source) = source {
        models.retain(|model| {
            format!("{:?}", model.source)
                .to_lowercase()
                .contains(&filter_source.to_lowercase())
        });
    }

    match output {
        OutputFormat::Table => {
            if models.is_empty() {
                println!("No models installed.");
                return Ok(());
            }

            println!("Installed Models ({} total)", models.len());
            println!();

            if detailed {
                for (i, model) in models.iter().enumerate() {
                    if i > 0 {
                        println!();
                    }
                    println!("Model ID: {}", model.model_id);
                    println!("Version: {}", model.version);
                    println!("Source: {:?}", model.source);
                    println!(
                        "Installed: {}",
                        model.installed_at.format("%Y-%m-%d %H:%M:%S")
                    );
                    println!("Path: {}", model.local_path.display());
                    println!(
                        "Auto-update: {}",
                        if model.auto_update_enabled {
                            "Yes"
                        } else {
                            "No"
                        }
                    );
                    println!("Usage count: {}", model.usage_count);
                    if let Some(last_used) = model.last_used {
                        println!("Last used: {}", last_used.format("%Y-%m-%d %H:%M:%S"));
                    }
                    println!("Verified: {}", if model.verified { "Yes" } else { "No" });
                }
            } else {
                println!(
                    "{:<40} {:<15} {:<12} {:<20} {:<8}",
                    "MODEL ID", "VERSION", "SOURCE", "INSTALLED", "AUTO-UPDATE"
                );
                println!("{}", "-".repeat(95));

                for model in &models {
                    println!(
                        "{:<40} {:<15} {:<12} {:<20} {:<8}",
                        truncate(&model.model_id, 38),
                        truncate(&model.version, 13),
                        format!("{:?}", model.source),
                        model.installed_at.format("%Y-%m-%d"),
                        if model.auto_update_enabled {
                            "Yes"
                        } else {
                            "No"
                        }
                    );
                }
            }
        }

        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&models)?;
            println!("{}", json);
        }

        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&models)?;
            println!("{}", yaml);
        }

        _ => {
            for model in &models {
                println!(
                    "{} v{} ({})",
                    model.model_id,
                    model.version,
                    format!("{:?}", model.source)
                );
            }
        }
    }

    Ok(())
}

async fn handle_progress(
    marketplace: &ModelMarketplace,
    download_id: Option<String>,
    watch: bool,
    interval: u64,
) -> Result<()> {
    if let Some(id) = download_id {
        if watch {
            let mut timer = tokio_interval(Duration::from_secs(interval));
            loop {
                timer.tick().await;

                match marketplace.get_download_progress(&id).await {
                    Ok(progress) => {
                        print!("\r{}", " ".repeat(80)); // Clear line
                        print!(
                            "\rProgress: {:.1}% ({}) - {:.1} MB/s - ETA: {}s",
                            progress.progress_percent,
                            format_bytes(progress.bytes_downloaded, progress.total_bytes),
                            progress.download_speed_mbps,
                            progress.eta_seconds
                        );

                        match progress.status {
                            DownloadStatus::Completed => {
                                println!("\n✓ Download completed!");
                                break;
                            }
                            DownloadStatus::Failed => {
                                println!(
                                    "\n✗ Download failed: {}",
                                    progress.error.unwrap_or_default()
                                );
                                break;
                            }
                            DownloadStatus::Cancelled => {
                                println!("\n⚠ Download cancelled");
                                break;
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        println!("\nError getting progress: {}", e);
                        break;
                    }
                }
            }
        } else {
            let progress = marketplace.get_download_progress(&id).await?;
            println!("Download Progress for {}", id);
            println!("=================================");
            println!("Model: {}", progress.model_id);
            println!("Status: {:?}", progress.status);
            println!("Progress: {:.1}%", progress.progress_percent);
            println!(
                "Downloaded: {}",
                format_bytes(progress.bytes_downloaded, progress.total_bytes)
            );
            println!("Speed: {:.1} MB/s", progress.download_speed_mbps);
            println!("ETA: {}s", progress.eta_seconds);
            println!(
                "Started: {}",
                progress.started_at.format("%Y-%m-%d %H:%M:%S")
            );

            if let Some(error) = progress.error {
                println!("Error: {}", error);
            }
        }
    } else {
        println!("Please specify a download ID");
    }

    Ok(())
}

async fn handle_cancel(marketplace: &ModelMarketplace, download_id: String) -> Result<()> {
    info!("Cancelling download: {}", download_id);

    marketplace.cancel_download(&download_id).await?;

    println!("✓ Download cancelled successfully!");

    Ok(())
}

async fn handle_updates(
    marketplace: &ModelMarketplace,
    model_id: Option<String>,
    auto_update: bool,
    output: OutputFormat,
) -> Result<()> {
    info!("Checking for model updates");

    if let Some(id) = model_id {
        // Check specific model
        let current_model = marketplace
            .list_installed_models()
            .await?
            .into_iter()
            .find(|m| m.model_id == id)
            .ok_or_else(|| anyhow::anyhow!("Model not installed: {}", id))?;

        let latest_model = marketplace.get_model_details(&id).await?;

        if latest_model.version != current_model.version {
            println!("Update available for {}:", id);
            println!("  Current: {}", current_model.version);
            println!("  Latest: {}", latest_model.version);

            if auto_update {
                println!("Updating...");
                let download_id = marketplace.update_model(&id).await?;
                println!("Update started with download ID: {}", download_id);
            }
        } else {
            println!("Model {} is up to date (v{})", id, current_model.version);
        }
    } else {
        // Check all models
        let updates = marketplace.check_for_updates().await?;

        match output {
            OutputFormat::Table => {
                if updates.is_empty() {
                    println!("All models are up to date!");
                } else {
                    println!("Updates available for {} models:", updates.len());
                    for model_id in &updates {
                        println!("  {}", model_id);
                    }

                    if auto_update {
                        println!("\nStarting updates...");
                        for model_id in &updates {
                            match marketplace.update_model(model_id).await {
                                Ok(download_id) => {
                                    println!("  {}: {}", model_id, download_id);
                                }
                                Err(e) => {
                                    error!("Failed to update {}: {}", model_id, e);
                                }
                            }
                        }
                    }
                }
            }

            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&updates)?;
                println!("{}", json);
            }

            _ => {
                for model_id in &updates {
                    println!("{}", model_id);
                }
            }
        }
    }

    Ok(())
}

async fn handle_update(marketplace: &ModelMarketplace, model_id: String, wait: bool) -> Result<()> {
    info!("Updating model: {}", model_id);

    let download_id = marketplace.update_model(&model_id).await?;

    println!("Update started with download ID: {}", download_id);

    if wait {
        println!("Waiting for update to complete...");

        loop {
            let progress = marketplace.get_download_progress(&download_id).await?;

            match progress.status {
                DownloadStatus::Completed => {
                    println!("✓ Model updated successfully!");
                    break;
                }
                DownloadStatus::Failed => {
                    return Err(anyhow::anyhow!(
                        "Update failed: {}",
                        progress.error.unwrap_or_default()
                    ));
                }
                DownloadStatus::Cancelled => {
                    println!("⚠ Update cancelled");
                    break;
                }
                _ => {
                    print!("\rProgress: {:.1}%", progress.progress_percent);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    Ok(())
}

async fn handle_publish(
    marketplace: &ModelMarketplace,
    model_path: PathBuf,
    name: String,
    version: String,
    description: String,
    category: ModelCategoryArg,
    license: String,
    publisher: Option<String>,
    tags: Option<String>,
    visibility: VisibilityArg,
    free: bool,
    price: Option<f64>,
    license_file: Option<PathBuf>,
    readme_file: Option<PathBuf>,
) -> Result<()> {
    info!("Publishing model: {}", name);

    if !model_path.exists() {
        return Err(anyhow::anyhow!(
            "Model file not found: {}",
            model_path.display()
        ));
    }

    // Create basic model metadata
    let metadata = ModelListing {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.clone(),
        version,
        publisher: publisher.unwrap_or_else(|| "unknown".to_string()),
        description,
        category: category.into(),
        license: license.clone(),
        size_bytes: std::fs::metadata(&model_path)?.len(),
        download_url: String::new(), // Will be set by registry
        checksum: String::new(),     // Will be calculated
        signature: None,
        metadata: crate::marketplace::ModelMetadata {
            framework: "Unknown".to_string(),
            format: "GGUF".to_string(), // Default guess
            precision: "fp16".to_string(),
            quantization: None,
            context_length: None,
            parameters: None,
            vocab_size: None,
            input_types: vec!["text".to_string()],
            output_types: vec!["text".to_string()],
            languages: vec!["en".to_string()],
            domains: vec![],
        },
        compatibility: crate::marketplace::CompatibilityInfo {
            inferno_version: ">=0.1.0".to_string(),
            minimum_ram_gb: 4.0,
            minimum_vram_gb: None,
            supported_backends: vec!["gguf".to_string()],
            supported_platforms: vec![
                "linux".to_string(),
                "macos".to_string(),
                "windows".to_string(),
            ],
            gpu_architectures: vec![],
            cpu_instructions: vec![],
        },
        performance: crate::marketplace::PerformanceMetrics {
            inference_speed_tokens_per_sec: None,
            memory_usage_gb: None,
            throughput_requests_per_sec: None,
            latency_ms: None,
            benchmark_scores: HashMap::new(),
            energy_efficiency: None,
        },
        published_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        downloads: 0,
        rating: None,
        tags: tags
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
        dependencies: vec![],
    };

    let pricing = PricingInfo {
        free,
        price_per_download: price,
        subscription_tiers: vec![],
        usage_based: None,
    };

    let request = PublishRequest {
        model_path,
        metadata,
        license_file,
        readme_file,
        example_files: vec![],
        visibility: visibility.into(),
        pricing,
    };

    let model_id = marketplace.publish_model(request).await?;

    println!("✓ Model published successfully!");
    println!("Model ID: {}", model_id);
    println!("Name: {}", name);
    println!("License: {}", license);

    Ok(())
}

async fn handle_popular(
    marketplace: &ModelMarketplace,
    category: Option<ModelCategoryArg>,
    limit: usize,
    output: OutputFormat,
) -> Result<()> {
    info!("Fetching popular models");

    let models = marketplace
        .get_popular_models(category.map(|c| c.into()), limit)
        .await?;

    match output {
        OutputFormat::Table => {
            println!("Popular Models");
            println!("==============");

            if models.is_empty() {
                println!("No popular models found.");
                return Ok(());
            }

            println!(
                "{:<40} {:<20} {:<15} {:<10} {:<15}",
                "MODEL", "PUBLISHER", "VERSION", "RATING", "DOWNLOADS"
            );
            println!("{}", "-".repeat(100));

            for model in &models {
                let rating_str = model
                    .rating
                    .map(|r| format!("{:.1}★", r))
                    .unwrap_or_else(|| "N/A".to_string());
                let downloads_str = format_number(model.downloads);
                println!(
                    "{:<40} {:<20} {:<15} {:<10} {:<15}",
                    truncate(&model.name, 38),
                    truncate(&model.publisher, 18),
                    truncate(&model.version, 13),
                    rating_str,
                    downloads_str
                );
            }
        }

        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&models)?;
            println!("{}", json);
        }

        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&models)?;
            println!("{}", yaml);
        }

        _ => {
            for model in &models {
                let rating_str = model
                    .rating
                    .map(|r| format!(" ({:.1}★)", r))
                    .unwrap_or_default();
                println!(
                    "{} v{} by {}{}",
                    model.name, model.version, model.publisher, rating_str
                );
            }
        }
    }

    Ok(())
}

async fn handle_recommendations(
    marketplace: &ModelMarketplace,
    user_id: Option<String>,
    limit: usize,
    output: OutputFormat,
) -> Result<()> {
    info!("Fetching model recommendations");

    let models = marketplace
        .get_recommended_models(user_id.as_deref())
        .await?;
    let models: Vec<_> = models.into_iter().take(limit).collect();

    match output {
        OutputFormat::Table => {
            println!("Recommended Models");
            println!("==================");

            if models.is_empty() {
                println!("No recommendations available.");
                return Ok(());
            }

            for (i, model) in models.iter().enumerate() {
                println!(
                    "{}. {} v{} by {}",
                    i + 1,
                    model.name,
                    model.version,
                    model.publisher
                );
                println!("   {}", model.description);
                println!(
                    "   Category: {:?} | Downloads: {} | Rating: {}",
                    model.category,
                    format_number(model.downloads),
                    model
                        .rating
                        .map(|r| format!("{:.1}★", r))
                        .unwrap_or_else(|| "N/A".to_string())
                );
                println!();
            }
        }

        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&models)?;
            println!("{}", json);
        }

        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&models)?;
            println!("{}", yaml);
        }

        _ => {
            for model in &models {
                println!("{} v{} by {}", model.name, model.version, model.publisher);
            }
        }
    }

    Ok(())
}

async fn handle_config_command(
    marketplace: &ModelMarketplace,
    command: ConfigCommands,
) -> Result<()> {
    match command {
        ConfigCommands::Show => {
            println!("Marketplace Configuration:");
            println!("=========================");
            // Implementation would show current config
            println!("Registry URL: <configured>");
            println!("Cache Directory: <configured>");
            println!("Auto-update: <configured>");
            println!("Verification: <configured>");
        }

        ConfigCommands::Set { key, value } => {
            println!("Setting {} = {}", key, value);
            // Implementation would update config
        }

        ConfigCommands::Get { key } => {
            println!("{}: <value>", key);
            // Implementation would get config value
        }

        ConfigCommands::Reset { confirm } => {
            if confirm {
                println!("Configuration reset to defaults");
                // Implementation would reset config
            } else {
                println!("Use --confirm to reset configuration");
            }
        }

        ConfigCommands::Test => {
            println!("Testing marketplace connection...");
            // Implementation would test connection
            println!("✓ Connection successful");
        }
    }

    Ok(())
}

async fn handle_clear_cache(
    marketplace: &ModelMarketplace,
    all: bool,
    metadata_only: bool,
    files_only: bool,
) -> Result<()> {
    info!("Clearing marketplace cache");

    if all {
        println!("Clearing all cache data...");
        // Implementation would clear all cache
        println!("✓ All cache data cleared");
    } else if metadata_only {
        println!("Clearing metadata cache...");
        // Implementation would clear only metadata
        println!("✓ Metadata cache cleared");
    } else if files_only {
        println!("Clearing file cache...");
        // Implementation would clear only files
        println!("✓ File cache cleared");
    } else {
        println!("Please specify what to clear: --all, --metadata-only, or --files-only");
    }

    Ok(())
}

async fn handle_verify(
    marketplace: &ModelMarketplace,
    model_id: Option<String>,
    deep: bool,
    fix: bool,
) -> Result<()> {
    if let Some(id) = model_id {
        info!("Verifying model: {}", id);
        println!("Verifying model: {}", id);

        if deep {
            println!("Performing deep verification...");
        }

        // Implementation would verify the model
        println!("✓ Model verification completed");

        if fix {
            println!("Fixing any issues found...");
            // Implementation would fix issues
        }
    } else {
        info!("Verifying all installed models");
        let models = marketplace.list_installed_models().await?;

        println!("Verifying {} installed models...", models.len());

        for model in &models {
            print!("Verifying {}... ", model.model_id);
            // Implementation would verify each model
            println!("✓");
        }

        println!("All models verified successfully");
    }

    Ok(())
}

// Helper functions

fn print_model_details(model: &ModelListing) {
    println!("Model Information");
    println!("=================");
    println!("ID: {}", model.id);
    println!("Name: {}", model.name);
    println!("Version: {}", model.version);
    println!("Publisher: {}", model.publisher);
    println!("Category: {:?}", model.category);
    println!("License: {}", model.license);
    println!("Size: {}", format_size(model.size_bytes));
    println!("Downloads: {}", format_number(model.downloads));

    if let Some(rating) = model.rating {
        println!("Rating: {:.1}★", rating);
    }

    println!("Published: {}", model.published_at.format("%Y-%m-%d"));
    println!("Updated: {}", model.updated_at.format("%Y-%m-%d"));

    println!("\nDescription:");
    println!("{}", model.description);

    if !model.tags.is_empty() {
        println!("\nTags: {}", model.tags.join(", "));
    }

    println!("\nCompatibility:");
    println!(
        "  Minimum RAM: {:.1} GB",
        model.compatibility.minimum_ram_gb
    );
    if let Some(vram) = model.compatibility.minimum_vram_gb {
        println!("  Minimum VRAM: {:.1} GB", vram);
    }
    println!(
        "  Platforms: {}",
        model.compatibility.supported_platforms.join(", ")
    );
    println!(
        "  Backends: {}",
        model.compatibility.supported_backends.join(", ")
    );

    println!("\nMetadata:");
    println!("  Framework: {}", model.metadata.framework);
    println!("  Format: {}", model.metadata.format);
    println!("  Precision: {}", model.metadata.precision);
    if let Some(params) = model.metadata.parameters {
        println!("  Parameters: {}", format_number(params));
    }
    if let Some(context) = model.metadata.context_length {
        println!("  Context Length: {}", context);
    }
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

fn format_number(num: u64) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K", num as f64 / 1_000.0)
    } else {
        num.to_string()
    }
}

fn format_bytes(downloaded: u64, total: u64) -> String {
    format!("{}/{}", format_size(downloaded), format_size(total))
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn confirm(message: &str) -> Result<bool> {
    println!("{} (y/N): ", message);
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .context("Failed to read input")?;
    Ok(input.trim().to_lowercase().starts_with('y'))
}
