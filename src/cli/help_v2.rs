#![allow(dead_code, unused_imports, unused_variables)]
//! Help Command - New Architecture
//!
//! This module provides user-friendly help and guidance commands.
//! Wraps the HelpSystem utility module as Commands.

use crate::{
    cli::help::HelpSystem,
    config::Config,
    interfaces::cli::{Command, CommandContext, CommandOutput},
};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use tracing::info;

// ============================================================================
// HandleError - Provide helpful error guidance
// ============================================================================

/// Provide helpful error messages and setup guidance
pub struct HandleError {
    config: Config,
    error_message: String,
}

impl HandleError {
    pub fn new(config: Config, error_message: String) -> Self {
        Self {
            config,
            error_message,
        }
    }
}

#[async_trait]
impl Command for HandleError {
    fn name(&self) -> &str {
        "help error"
    }

    fn description(&self) -> &str {
        "Provide helpful error messages and setup guidance"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.error_message.is_empty() {
            anyhow::bail!("Error message cannot be empty");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Providing help for error: {}", self.error_message);

        // Create a mock error to pass to HelpSystem
        let error = anyhow::anyhow!("{}", self.error_message);
        let guidance = HelpSystem::handle_error(&error);

        // Human-readable output
        if !ctx.json_output {
            print!("{}", guidance);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Error guidance provided",
            json!({
                "error": self.error_message,
                "guidance": guidance,
            }),
        ))
    }
}

// ============================================================================
// CheckPrerequisites - Check command prerequisites
// ============================================================================

/// Check prerequisites for a command
pub struct CheckPrerequisites {
    config: Config,
    command_name: String,
}

impl CheckPrerequisites {
    pub fn new(config: Config, command_name: String) -> Self {
        Self {
            config,
            command_name,
        }
    }
}

#[async_trait]
impl Command for CheckPrerequisites {
    fn name(&self) -> &str {
        "help prerequisites"
    }

    fn description(&self) -> &str {
        "Check prerequisites for a command"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.command_name.is_empty() {
            anyhow::bail!("Command name cannot be empty");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Checking prerequisites for: {}", self.command_name);

        let prereq_check = HelpSystem::check_prerequisites(&self.command_name);

        let (has_prereqs, message) = match prereq_check {
            Some(msg) => (true, msg),
            None => (
                false,
                format!("✓ No prerequisites for '{}'", self.command_name),
            ),
        };

        // Human-readable output
        if !ctx.json_output {
            if has_prereqs {
                println!("⚠️  Prerequisites for '{}':", self.command_name);
                print!("{}", message);
            } else {
                println!("{}", message);
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            if has_prereqs {
                "Prerequisites found"
            } else {
                "No prerequisites"
            },
            json!({
                "command": self.command_name,
                "has_prerequisites": has_prereqs,
                "message": message,
            }),
        ))
    }
}

// ============================================================================
// GetExamples - Get usage examples for a command
// ============================================================================

/// Get usage examples for a command
pub struct GetExamples {
    config: Config,
    command_name: String,
}

impl GetExamples {
    pub fn new(config: Config, command_name: String) -> Self {
        Self {
            config,
            command_name,
        }
    }
}

#[async_trait]
impl Command for GetExamples {
    fn name(&self) -> &str {
        "help examples"
    }

    fn description(&self) -> &str {
        "Get usage examples for a command"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.command_name.is_empty() {
            anyhow::bail!("Command name cannot be empty");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Getting examples for: {}", self.command_name);

        let examples = HelpSystem::get_usage_examples(&self.command_name);

        // Human-readable output
        if !ctx.json_output {
            print!("{}", examples);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!("Examples for '{}'", self.command_name),
            json!({
                "command": self.command_name,
                "examples": examples,
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_error_validation_empty() {
        let config = Config::default();
        let cmd = HandleError::new(config.clone(), String::new());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Error message cannot be empty"));
    }

    #[tokio::test]
    async fn test_handle_error_validation_valid() {
        let config = Config::default();
        let cmd = HandleError::new(config.clone(), "no such file or directory".to_string());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_prerequisites_validation_empty() {
        let config = Config::default();
        let cmd = CheckPrerequisites::new(config.clone(), String::new());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Command name cannot be empty"));
    }

    #[tokio::test]
    async fn test_check_prerequisites_validation_valid() {
        let config = Config::default();
        let cmd = CheckPrerequisites::new(config.clone(), "install".to_string());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_examples_validation_empty() {
        let config = Config::default();
        let cmd = GetExamples::new(config.clone(), String::new());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Command name cannot be empty"));
    }

    #[tokio::test]
    async fn test_get_examples_validation_valid() {
        let config = Config::default();
        let cmd = GetExamples::new(config.clone(), "install".to_string());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }
}
