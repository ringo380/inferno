use anyhow::Result;
use inferno::{
    cli::{enhanced_parser::EnhancedCliParser, help::HelpSystem, Commands},
    config::Config,
    setup_logging,
};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Use enhanced parser for better error handling and suggestions
    let cli = match EnhancedCliParser::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Handle clap errors with helpful suggestions
            eprintln!("{}", e);
            eprintln!("\n💡 For help with commands, try:");
            eprintln!("   inferno --help");
            eprintln!("   inferno [command] --help");
            std::process::exit(1);
        }
    };

    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config: {}", e);
        Config::default()
    });

    setup_logging(&config.log_level, &config.log_format)?;
    info!(
        "Starting Inferno AI/ML model runner v{}",
        std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string())
    );

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
        Commands::Optimization(args) => {
            inferno::cli::optimization::execute_optimization_command(args).await
        }
        Commands::MultiModal(args) => inferno::cli::multimodal::handle_multimodal_command(args)
            .await
            .map_err(|e| anyhow::anyhow!(e)),
        Commands::Deployment(args) => {
            inferno::cli::deployment::handle_deployment_command(args).await
        }
        Commands::Marketplace(args) => {
            inferno::cli::marketplace::handle_marketplace_command(args).await
        }
        Commands::Package(args) => inferno::cli::package::handle_package_command(args).await,

        // Simplified package manager aliases
        Commands::Install(args) => inferno::cli::package::handle_install_simple(args).await,
        Commands::Remove(args) => inferno::cli::package::handle_remove_simple(args).await,
        Commands::Search(args) => inferno::cli::package::handle_search_simple(args).await,
        Commands::List(args) => inferno::cli::package::handle_list_simple(args).await,
        Commands::Repo(args) => inferno::cli::repo::handle_repo_command(args).await,
        Commands::Federated(args) => inferno::cli::federated::handle_federated_command(args).await,
        Commands::Dashboard(args) => inferno::cli::dashboard::handle_dashboard_command(args).await,
        Commands::AdvancedMonitoring(args) => {
            inferno::cli::advanced_monitoring::execute(args, &config).await
        }
        Commands::ApiGateway(args) => inferno::cli::api_gateway::execute(args, &config).await,
        Commands::ModelVersioning(args) => {
            inferno::cli::model_versioning::execute(args, &config).await
        }
        Commands::DataPipeline(_args) => {
            println!("Data pipeline command is temporarily disabled due to implementation issues");
            Ok(())
        }
        Commands::BackupRecovery(args) => {
            inferno::cli::backup_recovery::execute(args, &config).await
        }
        Commands::LoggingAudit(args) => inferno::cli::logging_audit::execute(args, &config).await,
        Commands::PerformanceOptimization(args) => {
            inferno::cli::performance_optimization::execute(args, &config).await
        }
        Commands::MultiTenancy(args) => inferno::cli::multi_tenancy::execute(args, &config).await,
        Commands::AdvancedCache(args) => inferno::cli::advanced_cache::execute(args, &config).await,
        Commands::QAFramework(args) => inferno::cli::qa_framework::execute(args, &config).await,
        Commands::PerformanceBenchmark(args) => {
            inferno::cli::performance_benchmark::execute_performance_benchmark(args).await
        }
        Commands::Tui => inferno::tui::launch(&config).await,
    };

    if let Err(e) = result {
        // Provide user-friendly error handling
        let helpful_message = HelpSystem::handle_error(&e);
        eprintln!("{}", helpful_message);

        // Log the technical error for debugging
        error!("Command failed: {}", e);

        std::process::exit(1);
    }

    Ok(())
}
