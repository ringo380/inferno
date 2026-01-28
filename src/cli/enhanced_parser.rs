#![allow(dead_code)]
use crate::cli::{fuzzy::FuzzyMatcher, help::HelpSystem};
use crate::config::Config;
use crate::interfaces::cli::{Command, CommandContext, CommandOutput};
use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use serde_json::json;
use std::env;
use tracing::info;

/// Enhanced CLI parser with fuzzy matching and helpful suggestions
pub struct EnhancedCliParser {
    fuzzy_matcher: FuzzyMatcher,
}

impl Default for EnhancedCliParser {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedCliParser {
    pub fn new() -> Self {
        Self {
            fuzzy_matcher: FuzzyMatcher::new(),
        }
    }

    /// Parse command line arguments with enhanced error handling
    pub fn parse() -> crate::cli::Cli {
        let args: Vec<String> = env::args().collect();

        // Check for typos and provide suggestions before clap tries to parse
        if args.len() > 1 {
            let parser = Self::new();
            parser.check_command_suggestions(&args);
        }

        // Let clap handle the actual parsing
        crate::cli::Cli::parse()
    }

    /// Try to parse arguments with custom error handling
    pub fn try_parse() -> Result<crate::cli::Cli, clap::Error> {
        let args: Vec<String> = env::args().collect();

        if args.len() > 1 {
            let parser = Self::new();
            parser.check_command_suggestions(&args);
        }

        crate::cli::Cli::try_parse()
    }

    fn check_command_suggestions(&self, args: &[String]) {
        if args.len() < 2 {
            return;
        }

        let command = &args[1];

        // Skip if it's a valid flag
        if command.starts_with('-') {
            return;
        }

        // Check for command suggestions
        match self.fuzzy_matcher.validate_command(command) {
            crate::cli::fuzzy::CommandValidation::Valid => {
                // Command is valid, check for subcommand suggestions if applicable
                if args.len() > 2 {
                    let subcommand = format!("{} {}", command, &args[2]);
                    if let Some(suggestion) = self.fuzzy_matcher.suggest_command(&subcommand) {
                        if suggestion != subcommand {
                            self.print_subcommand_suggestion(&subcommand, &suggestion);
                        }
                    }
                }
            }
            crate::cli::fuzzy::CommandValidation::Alias(correct_command) => {
                self.print_alias_suggestion(command, &correct_command);
            }
            crate::cli::fuzzy::CommandValidation::Suggestion(suggestion) => {
                self.print_typo_suggestion(command, &suggestion);
            }
            crate::cli::fuzzy::CommandValidation::Invalid => {
                self.print_invalid_command_help(command);
            }
        }
    }

    fn print_alias_suggestion(&self, input: &str, correct: &str) {
        eprintln!("💡 Note: '{}' is an alias for '{}'", input, correct);
    }

    fn print_typo_suggestion(&self, input: &str, suggestion: &str) {
        eprintln!("❓ Did you mean '{}'?", suggestion);
        eprintln!("   You typed: {}", input);

        // Provide additional context
        if suggestion.contains(" ") {
            let parts: Vec<&str> = suggestion.split_whitespace().collect();
            if parts.len() == 2 {
                eprintln!("💡 Try: inferno {} {}", parts[0], parts[1]);
            }
        } else {
            eprintln!("💡 Try: inferno {}", suggestion);
        }
    }

    fn print_subcommand_suggestion(&self, input: &str, suggestion: &str) {
        eprintln!("💡 Suggestion: '{}'", suggestion);
        eprintln!("   You typed: {}", input);
    }

    fn print_invalid_command_help(&self, input: &str) {
        eprintln!("❌ Unknown command: '{}'", input);

        // Get multiple suggestions
        let suggestions = self.fuzzy_matcher.suggest_multiple(input, 3);

        if !suggestions.is_empty() {
            eprintln!("💡 Did you mean one of these?");
            for suggestion in &suggestions {
                eprintln!("   • {}", suggestion);
            }
            eprintln!();
        }

        // Provide general help
        eprintln!("🔧 Common commands:");
        eprintln!("   • inferno install <model>     # Install a model");
        eprintln!("   • inferno search <query>      # Search for models");
        eprintln!("   • inferno list                # List installed models");
        eprintln!("   • inferno run <model>         # Run inference");
        eprintln!("   • inferno --help              # Show all commands");
        eprintln!();

        // Provide examples based on what they might have meant
        if input.contains("instal") || input.contains("add") || input.contains("get") {
            eprintln!("{}", HelpSystem::get_usage_examples("install"));
        } else if input.contains("search") || input.contains("find") {
            eprintln!("{}", HelpSystem::get_usage_examples("search"));
        } else if input.contains("list") || input.contains("show") {
            eprintln!("{}", HelpSystem::get_usage_examples("list"));
        } else {
            eprintln!("{}", HelpSystem::get_usage_examples("general"));
        }
    }
}

/// Enhanced command execution with prerequisite checking
pub async fn execute_with_prerequisites(
    command_name: &str,
    execution_fn: impl std::future::Future<Output = anyhow::Result<()>>,
) -> anyhow::Result<()> {
    // Check prerequisites before execution
    if let Some(prereq_message) = HelpSystem::check_prerequisites(command_name) {
        eprintln!("{}", prereq_message);

        // Ask user if they want to continue anyway
        if command_name == "install" || command_name == "search" {
            eprintln!("❓ Continue anyway? (y/N): ");

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok()
                && !input.trim().to_lowercase().starts_with('y')
            {
                eprintln!("Operation cancelled.");
                return Ok(());
            }
        }
    }

    // Execute the command
    let result = execution_fn.await;

    // Provide post-execution guidance on errors
    if let Err(ref e) = result {
        let error_msg = e.to_string().to_lowercase();

        // Provide specific guidance based on the command that failed
        match command_name {
            "install" => {
                if error_msg.contains("not found") {
                    eprintln!("\n💡 Try searching for the model first:");
                    eprintln!("   inferno search [partial-model-name]");
                    eprintln!("   inferno search [model] --repo huggingface");
                }
            }
            "run" => {
                if error_msg.contains("model") {
                    eprintln!("\n💡 Make sure you have models available:");
                    eprintln!("   inferno list                    # Check installed models");
                    eprintln!("   inferno models list             # Check all models");
                    eprintln!("   inferno install [model-name]    # Install a model");
                }
            }
            "serve" => {
                if error_msg.contains("model") || error_msg.contains("not found") {
                    eprintln!("\n💡 Server needs a model to serve:");
                    eprintln!("   inferno install microsoft/DialoGPT-medium");
                    eprintln!("   inferno serve --model DialoGPT-medium");
                }
            }
            _ => {}
        }
    }

    result
}

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
            crate::cli::fuzzy::CommandValidation::Valid => (
                true,
                format!("'{}' is a valid command", self.command_name),
                None,
            ),
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

    #[test]
    fn test_enhanced_parser_creation() {
        let parser = EnhancedCliParser::new();
        // Just test that it can be created without panicking
        assert!(!parser.fuzzy_matcher.suggest_command("test").is_none());
    }

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
