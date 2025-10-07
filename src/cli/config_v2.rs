#![allow(dead_code, unused_imports, unused_variables)]
//! Config Command - New Architecture
//!
//! This module demonstrates the migration of the config command to the new
//! CLI architecture with Command trait, pipeline, and middleware support.
//!
//! Manages configuration files with show, init, and validate subcommands.

use crate::config::Config;
use crate::interfaces::cli::{Command, CommandContext, CommandOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use tracing::info;

// ============================================================================
// ConfigShow - Display current configuration
// ============================================================================

/// Show current configuration
pub struct ConfigShow {
    config: Config,
}

impl ConfigShow {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Command for ConfigShow {
    fn name(&self) -> &str {
        "config show"
    }

    fn description(&self) -> &str {
        "Show current configuration"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // No validation needed for show
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Showing current configuration");

        // Serialize config to TOML
        let config_toml = toml::to_string_pretty(&self.config)?;

        // Human-readable output
        if !ctx.json_output {
            println!("Current Configuration:");
            println!("===================");
            println!("{}", config_toml);
        }

        // Structured output
        let config_json = serde_json::to_value(&self.config)?;

        Ok(CommandOutput::success_with_data(
            "Configuration displayed successfully",
            json!({
                "format": "toml",
                "config": config_json,
            }),
        ))
    }
}

// ============================================================================
// ConfigInit - Generate default configuration file
// ============================================================================

/// Generate default configuration file
pub struct ConfigInit {
    path: Option<PathBuf>,
}

impl ConfigInit {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self { path }
    }
}

#[async_trait]
impl Command for ConfigInit {
    fn name(&self) -> &str {
        "config init"
    }

    fn description(&self) -> &str {
        "Generate default configuration file"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // Validate path if provided
        if let Some(ref path) = self.path {
            // Check if path already exists
            if path.exists() {
                anyhow::bail!(
                    "Configuration file already exists: {}. Remove it first or use a different path.",
                    path.display()
                );
            }

            // Check if parent directory exists or can be created
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    // We'll create it in execute(), just check we have permission
                    std::fs::create_dir_all(parent)?;
                    std::fs::remove_dir_all(parent)?; // Clean up test directory
                }
            }
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        let config = Config::default();
        let config_path = self
            .path
            .clone()
            .unwrap_or_else(Config::get_default_config_path);

        info!("Creating configuration file at: {}", config_path.display());

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Save configuration
        config.save(Some(&config_path))?;

        // Human-readable output
        if !ctx.json_output {
            println!("✓ Configuration file created at: {}", config_path.display());
            println!("Edit this file to customize your settings.");
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!("Configuration file created at: {}", config_path.display()),
            json!({
                "path": config_path.display().to_string(),
                "default": true,
            }),
        ))
    }
}

// ============================================================================
// ConfigValidate - Validate configuration file
// ============================================================================

/// Validate configuration file
pub struct ConfigValidate {
    path: Option<PathBuf>,
}

impl ConfigValidate {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self { path }
    }
}

#[async_trait]
impl Command for ConfigValidate {
    fn name(&self) -> &str {
        "config validate"
    }

    fn description(&self) -> &str {
        "Validate configuration file"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        let config_path = self
            .path
            .clone()
            .unwrap_or_else(Config::get_default_config_path);

        // Validate path exists
        if !config_path.exists() {
            anyhow::bail!("Configuration file not found: {}", config_path.display());
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        let config_path = self
            .path
            .clone()
            .unwrap_or_else(Config::get_default_config_path);

        info!("Validating configuration file: {}", config_path.display());

        // Attempt to load config
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
                let exit_code = if is_valid { 0 } else { 1 };

                // Human-readable output
                if !ctx.json_output {
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
                }

                // Structured output
                let result_json = json!({
                    "valid": is_valid,
                    "path": config_path.display().to_string(),
                    "warnings": warnings,
                    "errors": errors,
                    "config_summary": {
                        "models_dir": config.models_dir.display().to_string(),
                        "backend": {
                            "context_size": config.backend_config.context_size,
                            "batch_size": config.backend_config.batch_size,
                        },
                        "server": {
                            "bind_address": config.server.bind_address.clone(),
                            "port": config.server.port,
                        },
                    },
                });

                if is_valid {
                    Ok(CommandOutput::success_with_data(
                        "Configuration is valid",
                        result_json,
                    ))
                } else {
                    Ok(CommandOutput::error_with_data(
                        "Configuration validation failed",
                        result_json,
                        exit_code,
                    ))
                }
            }
            Err(e) => {
                // Failed to load config - syntax error or invalid TOML
                if !ctx.json_output {
                    println!("✗ Configuration validation failed: {}", e);
                }

                Ok(CommandOutput::error_with_data(
                    format!("Configuration validation failed: {}", e),
                    json!({
                        "valid": false,
                        "path": config_path.display().to_string(),
                        "parse_error": e.to_string(),
                    }),
                    1,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_show() {
        let config = Config::default();
        let cmd = ConfigShow::new(config);
        let mut ctx = CommandContext::new(Config::default());

        let result = cmd.execute(&mut ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.data.is_some());
    }

    #[tokio::test]
    async fn test_config_validate_missing_file() {
        let cmd = ConfigValidate::new(Some(PathBuf::from("/nonexistent/config.toml")));
        let ctx = CommandContext::new(Config::default());

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
    }
}
