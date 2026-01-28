use crate::config::Config;
use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::info;

// ============================================================================
// Pre-execution Validation
// ============================================================================

/// Validate init command parameters before execution
fn validate_init(path: &Option<PathBuf>) -> Result<()> {
    if let Some(ref path) = path {
        // Check if path already exists
        if path.exists() {
            anyhow::bail!(
                "Configuration file already exists: {}. Remove it first or use a different path.",
                path.display()
            );
        }

        // Check if parent directory exists or can be created
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                // Test if we can create the directory (then clean up)
                if let Err(e) = std::fs::create_dir_all(parent) {
                    anyhow::bail!(
                        "Cannot create parent directory '{}': {}",
                        parent.display(),
                        e
                    );
                }
                // Clean up test directory
                let _ = std::fs::remove_dir_all(parent);
            }
        }
    }
    Ok(())
}

/// Validate validate command parameters before execution
fn validate_validate_command(path: &Option<PathBuf>) -> Result<()> {
    let config_path = path.clone().unwrap_or_else(Config::get_default_config_path);

    if !config_path.exists() {
        anyhow::bail!(
            "Configuration file not found: {}. Run 'inferno config init' to create one.",
            config_path.display()
        );
    }
    Ok(())
}

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
            // Pre-execution validation
            validate_init(&path)?;

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
            // Pre-execution validation
            validate_validate_command(&path)?;

            // Note: config_path is used for error reporting - the actual path being validated
            let _config_path = path.unwrap_or_else(Config::get_default_config_path);

            match Config::load() {
                Ok(config) => {
                    // Perform additional validation checks
                    let mut warnings = Vec::new();
                    let mut errors = Vec::new();

                    // Check models directory
                    if !config.models_dir.exists() {
                        warnings.push(format!(
                            "Models directory does not exist: {}",
                            config.models_dir.display()
                        ));
                    }

                    // Check backend config values
                    if config.backend_config.context_size == 0 {
                        errors.push("Backend context_size cannot be 0".to_string());
                    }

                    if config.backend_config.batch_size == 0 {
                        errors.push("Backend batch_size cannot be 0".to_string());
                    }

                    // Check server config
                    if config.server.port == 0 {
                        errors.push("Server port cannot be 0".to_string());
                    }

                    // Determine validation result
                    let is_valid = errors.is_empty();

                    // Display results
                    if is_valid {
                        println!("✓ Configuration is valid");
                    } else {
                        println!("✗ Configuration validation failed");
                    }

                    if !warnings.is_empty() {
                        println!("\nWarnings:");
                        for warning in &warnings {
                            println!("  ⚠ {}", warning);
                        }
                    }

                    if !errors.is_empty() {
                        println!("\nErrors:");
                        for error in &errors {
                            println!("  ✗ {}", error);
                        }
                    }

                    // Display config summary
                    println!("\nConfiguration Summary:");
                    println!("  Models directory: {}", config.models_dir.display());
                    println!(
                        "  Backend context_size: {}",
                        config.backend_config.context_size
                    );
                    println!("  Backend batch_size: {}", config.backend_config.batch_size);
                    println!(
                        "  Server: {}:{}",
                        config.server.bind_address, config.server.port
                    );
                }
                Err(e) => {
                    println!("✗ Configuration validation failed: {}", e);
                    println!("\nPossible causes:");
                    println!("  - Invalid TOML syntax");
                    println!("  - Missing required fields");
                    println!("  - Type mismatch in configuration values");
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_validate_init_with_existing_file() {
        // Create a temporary directory with a file
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, "test").unwrap();

        let result = validate_init(&Some(config_path.clone()));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("already exists"));
    }

    #[test]
    fn test_validate_init_with_new_path() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("new_dir").join("config.toml");

        let result = validate_init(&Some(config_path));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_init_with_none() {
        let result = validate_init(&None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_validate_command_missing_file() {
        let result = validate_validate_command(&Some(PathBuf::from("/nonexistent/config.toml")));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
        assert!(err_msg.contains("inferno config init"));
    }

    #[tokio::test]
    async fn test_handle_config_show() {
        // This test verifies that show command works without errors
        let args = ConfigArgs {
            action: ConfigAction::Show,
        };
        let result = handle_config_command(args).await;
        assert!(result.is_ok());
    }
}
