#![allow(dead_code)]
use crate::config::Config;
use crate::deployment::{DeploymentConfig, DeploymentManager};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::info;

// ============================================================================
// Validation Helpers
// ============================================================================

/// Valid environments for deployment operations
const VALID_ENVIRONMENTS: &[&str] = &["dev", "staging", "prod", "production"];

/// Validate that the environment is one of the allowed values
fn validate_environment(environment: &str) -> Result<()> {
    if !VALID_ENVIRONMENTS.contains(&environment) {
        anyhow::bail!(
            "Invalid environment '{}'. Must be one of: dev, staging, prod, production",
            environment
        );
    }
    Ok(())
}

/// Validate manifest format
fn validate_manifest_format(format: &str) -> Result<()> {
    if !["yaml", "helm"].contains(&format) {
        anyhow::bail!(
            "Invalid manifest format '{}'. Must be one of: yaml, helm",
            format
        );
    }
    Ok(())
}

#[derive(Args)]
pub struct DeploymentCliArgs {
    #[command(subcommand)]
    pub command: DeploymentCommands,
}

#[derive(Subcommand)]
pub enum DeploymentCommands {
    #[command(about = "Generate Kubernetes manifests or a Helm chart")]
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
}

pub async fn handle_deployment_command(args: DeploymentCliArgs) -> Result<()> {
    let config = Config::load()?;
    let deployment_config = DeploymentConfig::from_config(&config)?;
    let mut manager = DeploymentManager::new(deployment_config);

    match args.command {
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
    }
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
    // Validate inputs
    validate_environment(&environment)?;
    validate_manifest_format(&format)?;

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
        // Other formats already rejected by validate_manifest_format()
        _ => unreachable!("Format already validated"),
    }

    println!("Manifest generation completed successfully!");

    Ok(())
}
