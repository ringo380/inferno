use crate::cli::{fuzzy::FuzzyMatcher, help::HelpSystem};
use clap::Parser;
use std::env;

/// Enhanced CLI parser with fuzzy matching and helpful suggestions
pub struct EnhancedCliParser {
    fuzzy_matcher: FuzzyMatcher,
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
            if std::io::stdin().read_line(&mut input).is_ok() {
                if !input.trim().to_lowercase().starts_with('y') {
                    eprintln!("Operation cancelled.");
                    return Ok(());
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_parser_creation() {
        let parser = EnhancedCliParser::new();
        // Just test that it can be created without panicking
        assert!(!parser.fuzzy_matcher.suggest_command("test").is_none());
    }
}
