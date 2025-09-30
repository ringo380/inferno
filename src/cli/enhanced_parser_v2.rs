//! Enhanced Parser Command - New Architecture
//!
//! This module provides CLI parser utilities and command validation.
//! Exposes diagnostic and validation functionality as Commands.

use crate::{
    cli::{fuzzy::FuzzyMatcher, help::HelpSystem},
    config::Config,
    interfaces::cli::{Command, CommandContext, CommandOutput},
};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use tracing::info;

// ============================================================================
// ValidateCommand - Validate command names and provide suggestions
// ============================================================================

/// Validate command name and get suggestions
pub struct ValidateCommand {
    config: Config,
    command_name: String,
}

impl ValidateCommand {
    pub fn new(config: Config, command_name: String) -> Self {
        Self {
            config,
            command_name,
        }
    }
}

#[async_trait]
impl Command for ValidateCommand {
    fn name(&self) -> &str {
        "parser validate"
    }

    fn description(&self) -> &str {
        "Validate command name and get suggestions"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.command_name.is_empty() {
            anyhow::bail!("Command name cannot be empty");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Validating command: {}", self.command_name);

        let fuzzy_matcher = FuzzyMatcher::new();
        let validation = fuzzy_matcher.validate_command(&self.command_name);

        let (is_valid, message, suggestion) = match validation {
            crate::cli::fuzzy::CommandValidation::Valid => {
                (true, format!("'{}' is a valid command", self.command_name), None)
            }
            crate::cli::fuzzy::CommandValidation::Alias(correct) => (
                true,
                format!("'{}' is an alias for '{}'", self.command_name, correct),
                Some(correct),
            ),
            crate::cli::fuzzy::CommandValidation::Suggestion(suggested) => (
                false,
                format!("'{}' is not valid", self.command_name),
                Some(suggested),
            ),
            crate::cli::fuzzy::CommandValidation::Invalid => {
                let suggestions = fuzzy_matcher.suggest_multiple(&self.command_name, 3);
                (
                    false,
                    format!("'{}' is unknown", self.command_name),
                    suggestions.first().cloned(),
                )
            }
        };

        // Human-readable output
        if !ctx.json_output {
            if is_valid {
                println!("✓ {}", message);
                if let Some(ref sug) = suggestion {
                    println!("  Canonical form: {}", sug);
                }
            } else {
                println!("✗ {}", message);
                if let Some(ref sug) = suggestion {
                    println!("  Did you mean: {}", sug);
                }
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            message,
            json!({
                "command": self.command_name,
                "valid": is_valid,
                "suggestion": suggestion,
            }),
        ))
    }
}

// ============================================================================
// GetSuggestions - Get multiple command suggestions
// ============================================================================

/// Get multiple command suggestions for a query
pub struct GetSuggestions {
    config: Config,
    query: String,
    limit: usize,
}

impl GetSuggestions {
    pub fn new(config: Config, query: String, limit: usize) -> Self {
        Self {
            config,
            query,
            limit,
        }
    }
}

#[async_trait]
impl Command for GetSuggestions {
    fn name(&self) -> &str {
        "parser suggestions"
    }

    fn description(&self) -> &str {
        "Get command suggestions for a query"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.query.is_empty() {
            anyhow::bail!("Query cannot be empty");
        }

        if self.limit == 0 {
            anyhow::bail!("Limit must be at least 1");
        }

        if self.limit > 20 {
            anyhow::bail!("Limit cannot exceed 20");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Getting suggestions for: {}", self.query);

        let fuzzy_matcher = FuzzyMatcher::new();
        let suggestions = fuzzy_matcher.suggest_multiple(&self.query, self.limit);

        // Human-readable output
        if !ctx.json_output {
            if suggestions.is_empty() {
                println!("No suggestions found for '{}'", self.query);
            } else {
                println!("Suggestions for '{}':", self.query);
                for (i, suggestion) in suggestions.iter().enumerate() {
                    println!("  {}. {}", i + 1, suggestion);
                }
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!("Found {} suggestions", suggestions.len()),
            json!({
                "query": self.query,
                "suggestions": suggestions,
                "count": suggestions.len(),
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
        "parser prerequisites"
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
                format!("No prerequisites for '{}'", self.command_name),
            ),
        };

        // Human-readable output
        if !ctx.json_output {
            if has_prereqs {
                println!("⚠️  Prerequisites for '{}':", self.command_name);
                println!("{}", message);
            } else {
                println!("✓ {}", message);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_command_empty() {
        let config = Config::default();
        let cmd = ValidateCommand::new(config.clone(), String::new());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Command name cannot be empty"));
    }

    #[tokio::test]
    async fn test_validate_command_valid() {
        let config = Config::default();
        let cmd = ValidateCommand::new(config.clone(), "run".to_string());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_suggestions_validation_zero_limit() {
        let config = Config::default();
        let cmd = GetSuggestions::new(config.clone(), "test".to_string(), 0);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Limit must be at least 1"));
    }

    #[tokio::test]
    async fn test_suggestions_validation_excessive_limit() {
        let config = Config::default();
        let cmd = GetSuggestions::new(config.clone(), "test".to_string(), 25);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Limit cannot exceed 20"));
    }

    #[tokio::test]
    async fn test_prerequisites_validation() {
        let config = Config::default();
        let cmd = CheckPrerequisites::new(config.clone(), "install".to_string());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }
}