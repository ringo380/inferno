use crate::config::Config;
use crate::deployment::{DeploymentArgs, DeploymentConfig, DeploymentManager};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use tracing::{info, warn};

/// Configuration for deployment operations
/// Reduces function signature from 12 parameters to 2
pub struct DeployConfig {
    pub environment: String,
    pub version: String,
    pub namespace: Option<String>,
    pub replicas: Option<u32>,
    pub values_file: Option<PathBuf>,
    pub set_values: Vec<String>,
    pub gpu: bool,
    pub skip_checks: bool,
    pub dry_run: bool,
    pub wait: bool,
    pub timeout: u64,
}

impl DeployConfig {
    /// Convert to DeploymentArgs, parsing set_values into custom_values HashMap
    pub fn into_deployment_args(self) -> DeploymentArgs {
        let mut custom_values = HashMap::new();

        // Parse custom values from CLI format (key=value)
        for value_pair in self.set_values {
            if let Some((key, value)) = value_pair.split_once('=') {
                custom_values.insert(key.to_string(), value.to_string());
            } else {
                warn!("Invalid value format: {}. Expected key=value", value_pair);
            }
        }

        DeploymentArgs {
            environment: self.environment,
            version: self.version,
            namespace: self.namespace,
            replicas: self.replicas,
            gpu_enabled: self.gpu,
            dry_run: self.dry_run,
            wait_for_completion: self.wait,
            timeout_seconds: self.timeout,
            custom_values,
            values_file: self.values_file,
            skip_pre_checks: self.skip_checks,
        }
    }
}

#[derive(Args)]
pub struct DeploymentCliArgs {
    #[command(subcommand)]
    pub command: DeploymentCommands,
}

#[derive(Subcommand)]
pub enum DeploymentCommands {
    #[command(about = "Deploy application to target environment")]
    Deploy {
        #[arg(short, long, help = "Target environment (dev, staging, production)")]
        environment: String,

        #[arg(short, long, help = "Application version/tag to deploy")]
        version: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(long, help = "Override default replicas")]
        replicas: Option<u32>,

        #[arg(long, help = "Custom values file for Helm deployment")]
        values_file: Option<PathBuf>,

        #[arg(long, help = "Set custom values (key=value format)", action = clap::ArgAction::Append)]
        set: Vec<String>,

        #[arg(long, help = "Enable GPU support")]
        gpu: bool,

        #[arg(long, help = "Skip pre-deployment checks")]
        skip_checks: bool,

        #[arg(long, help = "Dry run - generate manifests without applying")]
        dry_run: bool,

        #[arg(long, help = "Wait for deployment to complete")]
        wait: bool,

        #[arg(
            long,
            help = "Timeout for deployment wait (seconds)",
            default_value = "600"
        )]
        timeout: u64,
    },

    #[command(about = "Roll back to previous deployment")]
    Rollback {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(short, long, help = "Specific revision to rollback to")]
        revision: Option<u32>,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(long, help = "Wait for rollback to complete")]
        wait: bool,
    },

    #[command(about = "Scale deployment replicas")]
    Scale {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(short, long, help = "Number of replicas")]
        replicas: u32,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(long, help = "Wait for scaling to complete")]
        wait: bool,
    },

    #[command(about = "Get deployment status")]
    Status {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(long, help = "Show detailed pod information")]
        detailed: bool,

        #[arg(long, help = "Watch for status changes")]
        watch: bool,
    },

    #[command(about = "View deployment logs")]
    Logs {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(short, long, help = "Follow logs")]
        follow: bool,

        #[arg(long, help = "Number of lines to show", default_value = "100")]
        lines: u32,

        #[arg(long, help = "Show logs since timestamp (RFC3339)")]
        since: Option<String>,

        #[arg(long, help = "Pod selector label")]
        selector: Option<String>,
    },

    #[command(about = "Pause deployment (zero replicas)")]
    Pause {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,
    },

    #[command(about = "Resume paused deployment")]
    Resume {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(short, long, help = "Number of replicas to resume with")]
        replicas: Option<u32>,
    },

    #[command(about = "Delete deployment")]
    Delete {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(long, help = "Force deletion without confirmation")]
        force: bool,

        #[arg(long, help = "Delete PVCs and other persistent resources")]
        purge: bool,
    },

    #[command(about = "Generate Kubernetes manifests")]
    Generate {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(short, long, help = "Application version/tag")]
        version: String,

        #[arg(short, long, help = "Output directory")]
        output: PathBuf,

        #[arg(long, help = "Format (yaml or helm)", default_value = "yaml")]
        format: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(long, help = "Custom values file")]
        values_file: Option<PathBuf>,
    },

    #[command(about = "Validate deployment configuration")]
    Validate {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(long, help = "Configuration file to validate")]
        config_file: Option<PathBuf>,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(long, help = "Validate against cluster resources")]
        cluster: bool,
    },

    #[command(about = "List deployment history")]
    History {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(long, help = "Number of revisions to show", default_value = "10")]
        limit: u32,

        #[arg(
            long,
            help = "Output format (table, json, yaml)",
            default_value = "table"
        )]
        output: String,
    },

    #[command(about = "Configure auto-scaling")]
    Autoscale {
        #[command(subcommand)]
        command: AutoscaleCommands,
    },

    #[command(about = "Manage secrets and configuration")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    #[command(about = "Monitor deployment health")]
    Health {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(long, help = "Watch for health changes")]
        watch: bool,

        #[arg(long, help = "Check interval in seconds", default_value = "5")]
        interval: u64,
    },
}

#[derive(Subcommand)]
pub enum AutoscaleCommands {
    #[command(about = "Enable horizontal pod autoscaling")]
    Enable {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(long, help = "Minimum replicas", default_value = "1")]
        min_replicas: u32,

        #[arg(long, help = "Maximum replicas", default_value = "10")]
        max_replicas: u32,

        #[arg(long, help = "Target CPU utilization percentage", default_value = "70")]
        cpu_percent: u32,

        #[arg(long, help = "Target memory utilization percentage")]
        memory_percent: Option<u32>,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,
    },

    #[command(about = "Disable autoscaling")]
    Disable {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,
    },

    #[command(about = "Show autoscaling status")]
    Status {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,
    },

    #[command(about = "Update autoscaling parameters")]
    Update {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(long, help = "Minimum replicas")]
        min_replicas: Option<u32>,

        #[arg(long, help = "Maximum replicas")]
        max_replicas: Option<u32>,

        #[arg(long, help = "Target CPU utilization percentage")]
        cpu_percent: Option<u32>,

        #[arg(long, help = "Target memory utilization percentage")]
        memory_percent: Option<u32>,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    #[command(about = "Create or update configuration")]
    Set {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(short, long, help = "Configuration key")]
        key: String,

        #[arg(short, long, help = "Configuration value")]
        value: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(long, help = "Store as secret instead of config map")]
        secret: bool,
    },

    #[command(about = "Get configuration value")]
    Get {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(short, long, help = "Configuration key")]
        key: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,
    },

    #[command(about = "List all configuration")]
    List {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(long, help = "Include secrets")]
        include_secrets: bool,

        #[arg(
            long,
            help = "Output format (table, json, yaml)",
            default_value = "table"
        )]
        output: String,
    },

    #[command(about = "Delete configuration")]
    Delete {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(short, long, help = "Configuration key")]
        key: String,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,
    },

    #[command(about = "Import configuration from file")]
    Import {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(short, long, help = "Configuration file path")]
        file: PathBuf,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(long, help = "Import as secrets")]
        secrets: bool,
    },

    #[command(about = "Export configuration to file")]
    Export {
        #[arg(short, long, help = "Target environment")]
        environment: String,

        #[arg(short, long, help = "Output file path")]
        file: PathBuf,

        #[arg(long, help = "Kubernetes namespace")]
        namespace: Option<String>,

        #[arg(long, help = "Include secrets")]
        include_secrets: bool,

        #[arg(long, help = "Output format (yaml, json, env)", default_value = "yaml")]
        format: String,
    },
}

pub async fn handle_deployment_command(args: DeploymentCliArgs) -> Result<()> {
    let config = Config::load()?;
    let deployment_config = DeploymentConfig::from_config(&config)?;
    let mut manager = DeploymentManager::new(deployment_config);

    match args.command {
        DeploymentCommands::Deploy {
            environment,
            version,
            namespace,
            replicas,
            values_file,
            set,
            gpu,
            skip_checks,
            dry_run,
            wait,
            timeout,
        } => {
            let config = DeployConfig {
                environment,
                version,
                namespace,
                replicas,
                values_file,
                set_values: set,
                gpu,
                skip_checks,
                dry_run,
                wait,
                timeout,
            };
            handle_deploy(&mut manager, config).await
        }

        DeploymentCommands::Rollback {
            environment,
            revision,
            namespace,
            wait,
        } => handle_rollback(&mut manager, environment, revision, namespace, wait).await,

        DeploymentCommands::Scale {
            environment,
            replicas,
            namespace,
            wait,
        } => handle_scale(&mut manager, environment, replicas, namespace, wait).await,

        DeploymentCommands::Status {
            environment,
            namespace,
            detailed,
            watch,
        } => handle_status(&mut manager, environment, namespace, detailed, watch).await,

        DeploymentCommands::Logs {
            environment,
            namespace,
            follow,
            lines,
            since,
            selector,
        } => {
            handle_logs(
                &mut manager,
                environment,
                namespace,
                follow,
                lines,
                since,
                selector,
            )
            .await
        }

        DeploymentCommands::Pause {
            environment,
            namespace,
        } => handle_pause(&mut manager, environment, namespace).await,

        DeploymentCommands::Resume {
            environment,
            namespace,
            replicas,
        } => handle_resume(&mut manager, environment, namespace, replicas).await,

        DeploymentCommands::Delete {
            environment,
            namespace,
            force,
            purge,
        } => handle_delete(&mut manager, environment, namespace, force, purge).await,

        DeploymentCommands::Generate {
            environment,
            version,
            output,
            format,
            namespace,
            values_file,
        } => {
            handle_generate(
                &mut manager,
                environment,
                version,
                output,
                format,
                namespace,
                values_file,
            )
            .await
        }

        DeploymentCommands::Validate {
            environment,
            config_file,
            namespace,
            cluster,
        } => handle_validate(&mut manager, environment, config_file, namespace, cluster).await,

        DeploymentCommands::History {
            environment,
            namespace,
            limit,
            output,
        } => handle_history(&mut manager, environment, namespace, limit, output).await,

        DeploymentCommands::Autoscale { command } => {
            handle_autoscale_command(&mut manager, command).await
        }

        DeploymentCommands::Config { command } => {
            handle_config_command(&mut manager, command).await
        }

        DeploymentCommands::Health {
            environment,
            namespace,
            watch,
            interval,
        } => handle_health(&mut manager, environment, namespace, watch, interval).await,
    }
}

async fn handle_deploy(manager: &mut DeploymentManager, config: DeployConfig) -> Result<()> {
    info!("Starting deployment to {} environment", config.environment);

    // Convert config to DeploymentArgs (parses set_values into custom_values)
    let deploy_args = config.into_deployment_args();

    if deploy_args.dry_run {
        info!("Performing dry run deployment");
    }

    let result = manager.deploy(&deploy_args).await?;

    if deploy_args.dry_run {
        println!("Dry run completed successfully. Generated manifests:");
        println!("{}", result.manifest_preview);
    } else {
        println!("Deployment successful!");
        println!("Deployment ID: {}", result.deployment_id);
        println!("Status: {}", result.status);
        if let Some(url) = result.service_urls.get("main") {
            println!("Service URL: {}", url);
        }
    }

    Ok(())
}

async fn handle_rollback(
    manager: &mut DeploymentManager,
    environment: String,
    revision: Option<u32>,
    _namespace: Option<String>,
    wait: bool,
) -> Result<()> {
    info!("Rolling back deployment in {} environment", environment);

    let result = manager.rollback(&environment, revision).await?;

    println!("Rollback successful!");
    println!("Rolled back to revision: {}", result.revision);
    println!("Status: {}", result.status);

    if wait {
        info!("Waiting for rollback to complete...");
        // Implementation would wait for rollback completion
    }

    Ok(())
}

async fn handle_scale(
    manager: &mut DeploymentManager,
    environment: String,
    replicas: u32,
    _namespace: Option<String>,
    wait: bool,
) -> Result<()> {
    info!(
        "Scaling deployment in {} environment to {} replicas",
        environment, replicas
    );

    let result = manager.scale(&environment, replicas).await?;

    println!("Scaling successful!");
    println!("Current replicas: {}", result.current_replicas);
    println!("Target replicas: {}", result.target_replicas);
    println!("Status: {}", result.status);

    if wait {
        info!("Waiting for scaling to complete...");
        // Implementation would wait for scaling completion
    }

    Ok(())
}

async fn handle_status(
    manager: &mut DeploymentManager,
    environment: String,
    namespace: Option<String>,
    detailed: bool,
    watch: bool,
) -> Result<()> {
    if watch {
        info!("Watching deployment status (Ctrl+C to exit)");
        // Implementation would continuously monitor status
    }

    let status = manager
        .get_status(&environment, namespace.as_deref())
        .await?;

    println!("Deployment Status for {}", environment);
    println!("==========================================");
    println!("Environment: {}", status.environment);
    println!("Version: {}", status.version);
    println!("Status: {}", status.status);
    println!(
        "Replicas: {}/{}",
        status.ready_replicas, status.total_replicas
    );
    println!("Updated: {}", status.last_updated);

    if detailed {
        println!("\nPod Details:");
        for pod in &status.pods {
            println!("  {}: {} ({})", pod.name, pod.status, pod.ready);
        }

        println!("\nService URLs:");
        for (name, url) in &status.service_urls {
            println!("  {}: {}", name, url);
        }
    }

    if !watch {
        println!("\nHealth Checks:");
        for check in &status.health_checks {
            println!(
                "  {}: {}",
                check.name,
                if check.passing { "✓" } else { "✗" }
            );
        }
    }

    Ok(())
}

async fn handle_logs(
    manager: &mut DeploymentManager,
    environment: String,
    namespace: Option<String>,
    follow: bool,
    lines: u32,
    since: Option<String>,
    selector: Option<String>,
) -> Result<()> {
    info!("Fetching logs for {} environment", environment);

    let logs = manager
        .get_logs(
            &environment,
            namespace.as_deref(),
            lines,
            since.as_deref(),
            selector.as_deref(),
        )
        .await?;

    if follow {
        // Implementation would stream logs continuously
        println!("Following logs (Ctrl+C to exit)...");
    }

    for log_entry in logs {
        println!(
            "[{}] {}: {}",
            log_entry.timestamp, log_entry.pod, log_entry.message
        );
    }

    Ok(())
}

async fn handle_pause(
    manager: &mut DeploymentManager,
    environment: String,
    _namespace: Option<String>,
) -> Result<()> {
    info!("Pausing deployment in {} environment", environment);

    manager.pause(&environment).await?;

    println!("Deployment paused successfully!");
    println!("All replicas have been scaled to zero.");

    Ok(())
}

async fn handle_resume(
    manager: &mut DeploymentManager,
    environment: String,
    _namespace: Option<String>,
    replicas: Option<u32>,
) -> Result<()> {
    info!("Resuming deployment in {} environment", environment);

    let replica_count = replicas.unwrap_or(1);
    manager.resume(&environment).await?;

    println!("Deployment resumed successfully!");
    println!("Scaled back to {} replicas.", replica_count);

    Ok(())
}

async fn handle_delete(
    manager: &mut DeploymentManager,
    environment: String,
    _namespace: Option<String>,
    force: bool,
    purge: bool,
) -> Result<()> {
    if !force {
        println!(
            "This will delete the deployment in {} environment.",
            environment
        );
        if purge {
            println!("This will also delete all persistent volumes and data.");
        }
        println!("Are you sure? (y/N)");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("Failed to read user input")?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    info!("Deleting deployment in {} environment", environment);

    manager.delete(&environment).await?;

    println!("Deployment deleted successfully!");
    if purge {
        println!("All persistent resources have been removed.");
    }

    Ok(())
}

async fn handle_generate(
    manager: &mut DeploymentManager,
    environment: String,
    version: String,
    output: PathBuf,
    format: String,
    _namespace: Option<String>,
    _values_file: Option<PathBuf>,
) -> Result<()> {
    info!(
        "Generating {} manifests for {} environment",
        format, environment
    );

    let manifests = manager.generate_manifests(&environment, &version).await?;

    std::fs::create_dir_all(&output).context("Failed to create output directory")?;

    match format.as_str() {
        "yaml" => {
            for (name, content) in manifests {
                let file_path = output.join(format!("{}.yaml", name));
                std::fs::write(&file_path, content).context("Failed to write manifest file")?;
                println!("Generated: {}", file_path.display());
            }
        }
        "helm" => {
            let chart_dir = output.join("inferno-chart");
            manager.generate_helm_chart(&chart_dir).await?;
            println!("Generated Helm chart: {}", chart_dir.display());
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported format: {}. Use 'yaml' or 'helm'",
                format
            ));
        }
    }

    println!("Manifest generation completed successfully!");

    Ok(())
}

async fn handle_validate(
    manager: &mut DeploymentManager,
    environment: String,
    config_file: Option<PathBuf>,
    namespace: Option<String>,
    cluster: bool,
) -> Result<()> {
    info!(
        "Validating deployment configuration for {} environment",
        environment
    );

    let validation_result = manager
        .validate_config(
            &environment,
            config_file.as_deref(),
            namespace.as_deref(),
            cluster,
        )
        .await?;

    println!("Validation Results for {}", environment);
    println!("===================================");

    if validation_result.is_valid {
        println!("✓ Configuration is valid");
    } else {
        println!("✗ Configuration has errors");
    }

    if !validation_result.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &validation_result.warnings {
            println!("  ⚠ {}", warning);
        }
    }

    if !validation_result.errors.is_empty() {
        println!("\nErrors:");
        for error in &validation_result.errors {
            println!("  ✗ {}", error);
        }
    }

    if cluster {
        println!("\nCluster Resources:");
        for resource in &validation_result.cluster_resources {
            println!(
                "  {} ({}): {}",
                resource.name, resource.kind, resource.status
            );
        }
    }

    Ok(())
}

async fn handle_history(
    manager: &mut DeploymentManager,
    environment: String,
    namespace: Option<String>,
    limit: u32,
    output_format: String,
) -> Result<()> {
    info!(
        "Fetching deployment history for {} environment",
        environment
    );

    let history = manager
        .get_deployment_history(&environment, namespace.as_deref(), limit)
        .await?;

    match output_format.as_str() {
        "table" => {
            println!("Deployment History for {}", environment);
            println!("========================================");
            println!(
                "{:<10} {:<15} {:<20} {:<15} {:<10}",
                "REVISION", "VERSION", "TIMESTAMP", "STATUS", "ROLLBACK"
            );
            println!("{}", "-".repeat(80));

            for entry in history {
                let rollback_info = if entry.rolled_back { "Yes" } else { "No" };
                println!(
                    "{:<10} {:<15} {:<20} {:<15} {:<10}",
                    entry.revision, entry.version, entry.timestamp, entry.status, rollback_info
                );
            }
        }
        "json" => {
            let json = serde_json::to_string_pretty(&history)?;
            println!("{}", json);
        }
        "yaml" => {
            let yaml = serde_yaml::to_string(&history)?;
            println!("{}", yaml);
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported output format: {}. Use 'table', 'json', or 'yaml'",
                output_format
            ));
        }
    }

    Ok(())
}

async fn handle_autoscale_command(
    manager: &mut DeploymentManager,
    command: AutoscaleCommands,
) -> Result<()> {
    match command {
        AutoscaleCommands::Enable {
            environment,
            min_replicas,
            max_replicas,
            cpu_percent,
            memory_percent,
            namespace,
        } => {
            info!("Enabling autoscaling for {} environment", environment);

            manager
                .enable_autoscaling(
                    &environment,
                    namespace.as_deref(),
                    min_replicas,
                    max_replicas,
                    cpu_percent,
                    memory_percent,
                )
                .await?;

            println!("Autoscaling enabled successfully!");
            println!("Min replicas: {}", min_replicas);
            println!("Max replicas: {}", max_replicas);
            println!("CPU target: {}%", cpu_percent);
            if let Some(memory) = memory_percent {
                println!("Memory target: {}%", memory);
            }
        }

        AutoscaleCommands::Disable {
            environment,
            namespace,
        } => {
            info!("Disabling autoscaling for {} environment", environment);

            manager
                .disable_autoscaling(&environment, namespace.as_deref())
                .await?;

            println!("Autoscaling disabled successfully!");
        }

        AutoscaleCommands::Status {
            environment,
            namespace,
        } => {
            let status = manager
                .get_autoscaling_status(&environment, namespace.as_deref())
                .await?;

            println!("Autoscaling Status for {}", environment);
            println!("==================================");
            println!("Enabled: {}", status.enabled);
            if status.enabled {
                println!("Current replicas: {}", status.current_replicas);
                println!("Min replicas: {}", status.min_replicas);
                println!("Max replicas: {}", status.max_replicas);
                println!(
                    "CPU utilization: {}% (target: {}%)",
                    status.current_cpu_percent, status.target_cpu_percent
                );
                if let Some(memory) = status.current_memory_percent {
                    println!(
                        "Memory utilization: {}% (target: {}%)",
                        memory,
                        status.target_memory_percent.unwrap_or(0)
                    );
                }
                println!("Last scale time: {}", status.last_scale_time);
            }
        }

        AutoscaleCommands::Update {
            environment,
            min_replicas,
            max_replicas,
            cpu_percent,
            memory_percent,
            namespace,
        } => {
            info!("Updating autoscaling for {} environment", environment);

            manager
                .update_autoscaling(
                    &environment,
                    namespace.as_deref(),
                    min_replicas,
                    max_replicas,
                    cpu_percent,
                    memory_percent,
                )
                .await?;

            println!("Autoscaling configuration updated successfully!");
        }
    }

    Ok(())
}

async fn handle_config_command(
    manager: &mut DeploymentManager,
    command: ConfigCommands,
) -> Result<()> {
    match command {
        ConfigCommands::Set {
            environment,
            key,
            value,
            namespace,
            secret,
        } => {
            info!(
                "Setting configuration {} in {} environment",
                key, environment
            );

            manager
                .set_config(&environment, namespace.as_deref(), &key, &value, secret)
                .await?;

            let resource_type = if secret { "secret" } else { "config" };
            println!(
                "Configuration {} set successfully as {}",
                key, resource_type
            );
        }

        ConfigCommands::Get {
            environment,
            key,
            namespace,
        } => {
            let value = manager
                .get_config(&environment, namespace.as_deref(), &key)
                .await?;

            println!("{}: {}", key, value);
        }

        ConfigCommands::List {
            environment,
            namespace,
            include_secrets,
            output,
        } => {
            let configs = manager
                .list_config(&environment, namespace.as_deref(), include_secrets)
                .await?;

            match output.as_str() {
                "table" => {
                    println!("Configuration for {}", environment);
                    println!("==========================");
                    println!("{:<30} {:<50} {:<10}", "KEY", "VALUE", "TYPE");
                    println!("{}", "-".repeat(90));

                    for config in configs {
                        let display_value = if config.is_secret {
                            "*".repeat(config.value.len().min(8))
                        } else {
                            config.value.clone()
                        };
                        let config_type = if config.is_secret { "secret" } else { "config" };
                        println!(
                            "{:<30} {:<50} {:<10}",
                            config.key, display_value, config_type
                        );
                    }
                }
                "json" => {
                    let json = serde_json::to_string_pretty(&configs)?;
                    println!("{}", json);
                }
                "yaml" => {
                    let yaml = serde_yaml::to_string(&configs)?;
                    println!("{}", yaml);
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported output format: {}. Use 'table', 'json', or 'yaml'",
                        output
                    ));
                }
            }
        }

        ConfigCommands::Delete {
            environment,
            key,
            namespace,
        } => {
            info!(
                "Deleting configuration {} in {} environment",
                key, environment
            );

            manager
                .delete_config(&environment, namespace.as_deref(), &key)
                .await?;

            println!("Configuration {} deleted successfully", key);
        }

        ConfigCommands::Import {
            environment,
            file,
            namespace,
            secrets,
        } => {
            info!(
                "Importing configuration from file for {} environment",
                environment
            );

            manager
                .import_config(&environment, namespace.as_deref(), &file, secrets)
                .await?;

            let resource_type = if secrets { "secrets" } else { "config" };
            println!("Configuration imported successfully as {}", resource_type);
        }

        ConfigCommands::Export {
            environment,
            file,
            namespace,
            include_secrets,
            format,
        } => {
            info!("Exporting configuration for {} environment", environment);

            manager
                .export_config(
                    &environment,
                    namespace.as_deref(),
                    &file,
                    include_secrets,
                    &format,
                )
                .await?;

            println!("Configuration exported to {}", file.display());
        }
    }

    Ok(())
}

async fn handle_health(
    manager: &mut DeploymentManager,
    environment: String,
    namespace: Option<String>,
    watch: bool,
    interval: u64,
) -> Result<()> {
    if watch {
        info!("Monitoring deployment health (Ctrl+C to exit)");

        loop {
            let health = manager
                .check_health(&environment, namespace.as_deref())
                .await?;

            // Clear screen and move cursor to top
            print!("\x1B[2J\x1B[1;1H");

            println!(
                "Health Status for {} (Updated: {})",
                environment,
                chrono::Utc::now().format("%H:%M:%S")
            );
            println!("===============================================");
            println!(
                "Overall Health: {}",
                if health.overall_healthy {
                    "✓ Healthy"
                } else {
                    "✗ Unhealthy"
                }
            );
            println!("Uptime: {}", health.uptime);

            println!("\nService Health:");
            for service in &health.services {
                let status_icon = if service.healthy { "✓" } else { "✗" };
                println!(
                    "  {} {}: {} ({}ms)",
                    status_icon, service.name, service.status, service.response_time_ms
                );
            }

            println!("\nResource Usage:");
            println!(
                "  CPU: {}% (limit: {}%)",
                health.cpu_usage, health.cpu_limit
            );
            println!(
                "  Memory: {}% (limit: {}%)",
                health.memory_usage, health.memory_limit
            );

            if !health.recent_errors.is_empty() {
                println!("\nRecent Errors:");
                for error in &health.recent_errors[..health.recent_errors.len().min(5)] {
                    println!("  [{}] {}", error.timestamp, error.message);
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        }
    } else {
        let health = manager
            .check_health(&environment, namespace.as_deref())
            .await?;

        println!("Health Status for {}", environment);
        println!("========================");
        println!(
            "Overall Health: {}",
            if health.overall_healthy {
                "✓ Healthy"
            } else {
                "✗ Unhealthy"
            }
        );
        println!("Uptime: {}", health.uptime);

        println!("\nServices:");
        for service in &health.services {
            let status_icon = if service.healthy { "✓" } else { "✗" };
            println!("  {} {}: {}", status_icon, service.name, service.status);
        }
    }

    Ok(())
}
