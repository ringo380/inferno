use crate::config::Config;
use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::info;

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    #[command(about = "Show current configuration")]
    Show,
    #[command(about = "Generate default configuration file")]
    Init {
        #[arg(short, long, help = "Configuration file path")]
        path: Option<PathBuf>,
    },
    #[command(about = "Validate configuration")]
    Validate {
        #[arg(short, long, help = "Configuration file path")]
        path: Option<PathBuf>,
    },
}

pub async fn handle_config_command(args: ConfigArgs) -> Result<()> {
    match args.action {
        ConfigAction::Show => {
            let config = Config::load()?;
            let config_toml = toml::to_string_pretty(&config)?;
            println!("Current Configuration:");
            println!("===================");
            println!("{}", config_toml);
        }
        ConfigAction::Init { path } => {
            let config = Config::default();
            let config_path = path.unwrap_or_else(Config::get_default_config_path);

            // Create parent directory if it doesn't exist
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            config.save(Some(&config_path))?;
            info!("Configuration file created at: {}", config_path.display());
            println!("✓ Configuration file created at: {}", config_path.display());
            println!("Edit this file to customize your settings.");
        }
        ConfigAction::Validate { path } => {
            let config_path = path.unwrap_or_else(Config::get_default_config_path);

            if !config_path.exists() {
                println!("❌ Configuration file not found: {}", config_path.display());
                return Ok(());
            }

            match Config::load() {
                Ok(_) => {
                    println!("✓ Configuration is valid");
                }
                Err(e) => {
                    println!("❌ Configuration validation failed: {}", e);
                }
            }
        }
    }

    Ok(())
}
