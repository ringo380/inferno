use anyhow::Result;
use clap::Parser;
use inferno::{
    cli::{Cli, Commands},
    config::Config,
    setup_logging,
};
use std::env;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config: {}", e);
        Config::default()
    });

    setup_logging(&config.log_level, &config.log_format)?;
    info!("Starting Inferno AI/ML model runner v{}", env!("CARGO_PKG_VERSION"));

    let result = match cli.command {
        Commands::Run(args) => inferno::cli::run::execute(args, &config).await,
        Commands::Batch(args) => inferno::cli::batch::execute(args, &config).await,
        Commands::Serve(args) => inferno::cli::serve::execute(args, &config).await,
        Commands::Models(args) => inferno::cli::models::execute(args, &config).await,
        Commands::Metrics(args) => inferno::cli::metrics::execute(args, &config).await,
        Commands::Bench(args) => inferno::cli::bench::execute(args, &config).await,
        Commands::Validate(args) => inferno::cli::validate::execute(args, &config).await,
        Commands::Config(args) => inferno::cli::config::handle_config_command(args).await,
        Commands::Cache(args) => inferno::cli::cache::execute(args, &config).await,
        Commands::Convert(args) => inferno::cli::convert::execute(args, &config).await,
        Commands::ResponseCache(args) => inferno::cli::response_cache::execute(args, &config).await,
        Commands::Monitor(args) => inferno::cli::monitoring::execute(args, &config).await,
        Commands::Distributed(args) => inferno::cli::distributed::execute(args, &config).await,
        Commands::ABTest(args) => inferno::cli::ab_testing::execute(args, &config).await,
        Commands::Audit(args) => inferno::cli::audit::execute(args, &config).await,
        Commands::Queue(args) => inferno::cli::batch_queue::execute(args, &config).await,
        Commands::Version(args) => inferno::cli::versioning::execute(args, &config).await,
        Commands::Gpu(args) => inferno::cli::gpu::execute(args, &config).await,
        Commands::Resilience(args) => inferno::cli::resilience::execute(args, &config).await,
        Commands::Streaming(args) => inferno::cli::streaming::execute(args, &config).await,
        Commands::Security(args) => inferno::cli::security::execute(args, &config).await,
        Commands::Observability(args) => inferno::cli::observability::execute(args, &config).await,
        Commands::Optimization(args) => inferno::cli::optimization::handle_optimization_command(args).await.map_err(|e| anyhow::anyhow!(e)),
        Commands::MultiModal(args) => inferno::cli::multimodal::handle_multimodal_command(args).await.map_err(|e| anyhow::anyhow!(e)),
        Commands::Deployment(args) => inferno::cli::deployment::handle_deployment_command(args).await,
        Commands::Marketplace(args) => inferno::cli::marketplace::handle_marketplace_command(args).await,
        Commands::Federated(args) => inferno::cli::federated::handle_federated_command(args).await,
        Commands::Dashboard(args) => inferno::cli::dashboard::handle_dashboard_command(args).await,
        Commands::AdvancedMonitoring(args) => inferno::cli::advanced_monitoring::execute(args, &config).await,
        Commands::ApiGateway(args) => inferno::cli::api_gateway::execute(args, &config).await,
        Commands::ModelVersioning(args) => inferno::cli::model_versioning::execute(args, &config).await,
        Commands::DataPipeline(args) => inferno::cli::data_pipeline::execute(args, &config).await,
        Commands::BackupRecovery(args) => inferno::cli::backup_recovery::execute(args, &config).await,
        Commands::LoggingAudit(args) => inferno::cli::logging_audit::execute(args, &config).await,
        Commands::PerformanceOptimization(args) => inferno::cli::performance_optimization::execute(args, &config).await,
        Commands::MultiTenancy(args) => inferno::cli::multi_tenancy::execute(args, &config).await,
        Commands::AdvancedCache(args) => inferno::cli::advanced_cache::execute(args, &config).await,
        Commands::QAFramework(args) => inferno::cli::qa_framework::execute(args, &config).await,
        Commands::Tui => inferno::tui::launch(&config).await,
    };

    if let Err(e) = result {
        error!("Command failed: {}", e);
        std::process::exit(1);
    }

    Ok(())
}