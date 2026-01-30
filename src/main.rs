use anyhow::Result;
use inferno::{
    cli::{Commands, enhanced_parser::EnhancedCliParser, help::HelpSystem},
    config::Config,
    upgrade::{
        ApplicationVersion, background_service::BackgroundUpdateService, init_upgrade_system,
    },
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<()> {
    // Use enhanced parser for better error handling and suggestions
    let cli = match EnhancedCliParser::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Let clap handle output routing:
            // - Help/version → stdout, exit 0
            // - Parse errors → stderr, exit non-zero
            if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion
            {
                e.exit();
            }
            // For actual parsing errors, add helpful suggestions
            eprintln!("{}", e);
            eprintln!("\n💡 For help with commands, try:");
            eprintln!("   inferno --help");
            eprintln!("   inferno [command] --help");
            std::process::exit(e.exit_code());
        }
    };

    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config: {}", e);
        Config::default()
    });

    setup_logging();
    info!(
        "Starting Inferno AI/ML model runner v{}",
        std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string())
    );

    // Initialize background update service for long-running commands
    let background_service = if should_start_background_service(&cli.command) {
        match init_background_update_service(&config).await {
            Ok(service) => {
                info!("Background update service initialized");
                Some(service)
            }
            Err(e) => {
                warn!("Failed to initialize background update service: {}", e);
                None
            }
        }
    } else {
        None
    };

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
        Commands::Dashboard(args) => inferno::cli::dashboard::handle_dashboard_command(args).await,
        Commands::AdvancedMonitoring(args) => {
            inferno::cli::advanced_monitoring::execute(args, &config).await
        }
        Commands::ModelVersioning(args) => {
            inferno::cli::model_versioning::execute(args, &config).await
        }
        Commands::BackupRecovery(args) => {
            inferno::cli::backup_recovery::execute(args, &config).await
        }
        Commands::LoggingAudit(args) => inferno::cli::logging_audit::execute(args, &config).await,
        Commands::PerformanceOptimization(args) => {
            inferno::cli::performance_optimization::execute(args, &config).await
        }
        Commands::AdvancedCache(args) => inferno::cli::advanced_cache::execute(args, &config).await,
        Commands::PerformanceBenchmark(args) => {
            inferno::cli::performance_benchmark::execute_performance_benchmark(args).await
        }
        Commands::Upgrade(args) => inferno::cli::upgrade::execute(args, &config).await,
        Commands::Tui => inferno::tui::launch(&config).await,
    };

    if let Err(e) = result {
        // Stop background service if it was started
        if let Some(service) = background_service {
            if let Err(stop_err) = service.stop().await {
                warn!("Failed to stop background service: {}", stop_err);
            }
        }

        // Provide user-friendly error handling
        let helpful_message = HelpSystem::handle_error(&e);
        eprintln!("{}", helpful_message);

        // Log the technical error for debugging
        error!("Command failed: {}", e);

        std::process::exit(1);
    }

    // Background service will be stopped automatically when the process exits for serve/tui
    // For other commands, it's not started so no cleanup needed

    Ok(())
}

/// Determine if the background update service should be started for this command
fn should_start_background_service(command: &Commands) -> bool {
    matches!(
        command,
        Commands::Serve(_) |     // API server runs continuously
        Commands::Tui |          // TUI runs continuously
        Commands::Dashboard(_) // Dashboard runs continuously
    )
}

/// Initialize the background update service
async fn init_background_update_service(config: &Config) -> Result<BackgroundUpdateService> {
    // Initialize upgrade manager
    let upgrade_manager = match init_upgrade_system(config).await {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            warn!("Failed to initialize upgrade system: {}", e);
            return Err(e);
        }
    };

    // Create event broadcast channel for upgrade notifications
    let (event_sender, _) = broadcast::channel(1000);

    // Get upgrade config from main config
    let upgrade_config = match inferno::upgrade::config::UpgradeConfig::from_config(config) {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load upgrade config, using defaults: {}", e);
            inferno::upgrade::config::UpgradeConfig::default()
        }
    };

    // Create and start the background service
    let service = BackgroundUpdateService::new(upgrade_manager, upgrade_config, event_sender);

    // Start the service in a background task
    let service_handle = service.clone();
    tokio::spawn(async move {
        if let Err(e) = service_handle.start().await {
            error!("Background update service failed: {}", e);
        }
    });

    info!(
        "Background update service started for {}",
        ApplicationVersion::current().to_string()
    );

    Ok(service)
}

/// Set up comprehensive logging and tracing
fn setup_logging() {
    // Create a subscriber with environment filter support
    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("inferno=info".parse().unwrap())
                .add_directive("warn".parse().unwrap()),
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to initialize tracing subscriber");
}
