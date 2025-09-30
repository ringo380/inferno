//! Fuzzy Matching Command - New Architecture
//!
//! This module provides fuzzy matching operations for CLI commands.
//! Wrapper around the fuzzy matching utility module.

use crate::{
    cli::fuzzy::FuzzyMatcher,
    config::Config,
    interfaces::cli::{Command, CommandContext, CommandOutput},
};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use tracing::info;

// ============================================================================
// FuzzyMatch - Find best fuzzy match for input
// ============================================================================

/// Find best fuzzy match for input string
pub struct FuzzyMatch {
    config: Config,
    input: String,
}

impl FuzzyMatch {
    pub fn new(config: Config, input: String) -> Self {
        Self { config, input }
    }
}

#[async_trait]
impl Command for FuzzyMatch {
    fn name(&self) -> &str {
        "fuzzy match"
    }

    fn description(&self) -> &str {
        "Find best fuzzy match for input"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.input.is_empty() {
            anyhow::bail!("Input cannot be empty");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Finding fuzzy match for: {}", self.input);

        let matcher = FuzzyMatcher::new();
        let suggestion = matcher.suggest_command(&self.input);

        // Human-readable output
        if !ctx.json_output {
            match &suggestion {
                Some(sug) => {
                    println!("Best match for '{}':", self.input);
                    println!("  → {}", sug);
                }
                None => {
                    println!("No match found for '{}'", self.input);
                }
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            match &suggestion {
                Some(sug) => format!("Found match: {}", sug),
                None => "No match found".to_string(),
            },
            json!({
                "input": self.input,
                "match": suggestion,
                "found": suggestion.is_some(),
            }),
        ))
    }
}

// ============================================================================
// FuzzyMultiMatch - Find multiple fuzzy matches
// ============================================================================

/// Find multiple fuzzy matches for input
pub struct FuzzyMultiMatch {
    config: Config,
    input: String,
    limit: usize,
}

impl FuzzyMultiMatch {
    pub fn new(config: Config, input: String, limit: usize) -> Self {
        Self {
            config,
            input,
            limit,
        }
    }
}

#[async_trait]
impl Command for FuzzyMultiMatch {
    fn name(&self) -> &str {
        "fuzzy multimatch"
    }

    fn description(&self) -> &str {
        "Find multiple fuzzy matches for input"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.input.is_empty() {
            anyhow::bail!("Input cannot be empty");
        }

        if self.limit == 0 {
            anyhow::bail!("Limit must be at least 1");
        }

        if self.limit > 50 {
            anyhow::bail!("Limit cannot exceed 50");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Finding {} fuzzy matches for: {}", self.limit, self.input);

        let matcher = FuzzyMatcher::new();
        let suggestions = matcher.suggest_multiple(&self.input, self.limit);

        // Human-readable output
        if !ctx.json_output {
            if suggestions.is_empty() {
                println!("No matches found for '{}'", self.input);
            } else {
                println!("Top {} matches for '{}':", suggestions.len(), self.input);
                for (i, suggestion) in suggestions.iter().enumerate() {
                    println!("  {}. {}", i + 1, suggestion);
                }
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!("Found {} matches", suggestions.len()),
            json!({
                "input": self.input,
                "matches": suggestions,
                "count": suggestions.len(),
                "limit": self.limit,
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fuzzy_match_validation_empty() {
        let config = Config::default();
        let cmd = FuzzyMatch::new(config.clone(), String::new());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Input cannot be empty"));
    }

    #[tokio::test]
    async fn test_fuzzy_match_validation_valid() {
        let config = Config::default();
        let cmd = FuzzyMatch::new(config.clone(), "instal".to_string());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fuzzy_multimatch_validation_zero_limit() {
        let config = Config::default();
        let cmd = FuzzyMultiMatch::new(config.clone(), "test".to_string(), 0);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Limit must be at least 1"));
    }

    #[tokio::test]
    async fn test_fuzzy_multimatch_validation_excessive_limit() {
        let config = Config::default();
        let cmd = FuzzyMultiMatch::new(config.clone(), "test".to_string(), 60);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Limit cannot exceed 50"));
    }

    #[tokio::test]
    async fn test_fuzzy_multimatch_validation_valid() {
        let config = Config::default();
        let cmd = FuzzyMultiMatch::new(config.clone(), "instal".to_string(), 5);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }
}
