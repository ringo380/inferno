//! Core command trait and metadata
//!
//! Defines the trait that all CLI commands must implement, providing
//! a unified interface for execution, validation, and metadata.

use super::{CommandContext, CommandOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Core trait for all CLI commands
///
/// Implement this trait to create a new command that integrates with
/// the command pipeline and middleware system.
///
/// # Example
///
/// ```ignore
/// use inferno::interfaces::cli::{Command, CommandContext, CommandOutput};
/// use async_trait::async_trait;
///
/// pub struct MyCommand {
///     arg1: String,
///     arg2: i32,
/// }
///
/// #[async_trait]
/// impl Command for MyCommand {
///     fn name(&self) -> &str {
///         "my-command"
///     }
///
///     fn description(&self) -> &str {
///         "Does something useful"
///     }
///
///     async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
///         // Command logic here
///         Ok(CommandOutput::success("Done!"))
///     }
/// }
/// ```
#[async_trait]
pub trait Command: Send + Sync {
    /// Unique command name for identification
    ///
    /// Used in logs, metrics, and command routing.
    fn name(&self) -> &str;

    /// Human-readable description
    ///
    /// Displayed in help text and documentation.
    fn description(&self) -> &str;

    /// Validate command arguments and context before execution
    ///
    /// Override this to add custom validation logic. Return an error
    /// to prevent execution.
    ///
    /// Default implementation does no validation.
    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        Ok(())
    }

    /// Execute the command
    ///
    /// This is where the main command logic lives. Use the context
    /// to access configuration, args, and shared state.
    ///
    /// Return CommandOutput to indicate success/failure and provide
    /// structured output.
    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput>;

    /// Get command metadata
    ///
    /// Override to provide additional metadata about the command.
    /// Default returns minimal metadata.
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: self.name().to_string(),
            description: self.description().to_string(),
            category: None,
            tags: Vec::new(),
            examples: Vec::new(),
            requires_config: true,
            is_dangerous: false,
        }
    }

    /// Check if command requires configuration
    ///
    /// Some commands might work without full config (like help).
    /// Default is true.
    fn requires_config(&self) -> bool {
        true
    }

    /// Check if command is considered dangerous
    ///
    /// Dangerous commands might require confirmation or additional
    /// validation. Examples: delete, reset, wipe.
    fn is_dangerous(&self) -> bool {
        false
    }
}

/// Metadata about a command
///
/// Provides additional information for help, documentation,
/// and command discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadata {
    /// Command name
    pub name: String,

    /// Short description
    pub description: String,

    /// Optional category (e.g., "Model Management", "Operations")
    pub category: Option<String>,

    /// Tags for discovery and filtering
    pub tags: Vec<String>,

    /// Usage examples
    pub examples: Vec<CommandExample>,

    /// Whether command requires configuration
    pub requires_config: bool,

    /// Whether command is considered dangerous
    pub is_dangerous: bool,
}

/// Example of command usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExample {
    /// Example description
    pub description: String,

    /// Example command line
    pub command: String,

    /// Expected output (optional)
    pub output: Option<String>,
}

impl CommandExample {
    /// Create a new command example
    pub fn new(description: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            command: command.into(),
            output: None,
        }
    }

    /// Create a new example with expected output
    pub fn with_output(
        description: impl Into<String>,
        command: impl Into<String>,
        output: impl Into<String>,
    ) -> Self {
        Self {
            description: description.into(),
            command: command.into(),
            output: Some(output.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCommand {
        name: String,
    }

    #[async_trait]
    impl Command for TestCommand {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "Test command"
        }

        async fn execute(&self, _ctx: &mut CommandContext) -> Result<CommandOutput> {
            Ok(CommandOutput::success("Test passed"))
        }
    }

    #[tokio::test]
    async fn test_command_trait() {
        let cmd = TestCommand {
            name: "test".to_string(),
        };

        assert_eq!(cmd.name(), "test");
        assert_eq!(cmd.description(), "Test command");
        assert!(cmd.requires_config());
        assert!(!cmd.is_dangerous());

        let mut ctx = CommandContext::mock();
        let result = cmd.execute(&mut ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validation() {
        struct ValidatingCommand;

        #[async_trait]
        impl Command for ValidatingCommand {
            fn name(&self) -> &str {
                "validator"
            }

            fn description(&self) -> &str {
                "Validates"
            }

            async fn validate(&self, ctx: &CommandContext) -> Result<()> {
                if ctx.get_arg("required").is_none() {
                    anyhow::bail!("Missing required argument");
                }
                Ok(())
            }

            async fn execute(&self, _ctx: &mut CommandContext) -> Result<CommandOutput> {
                Ok(CommandOutput::success("Executed"))
            }
        }

        let cmd = ValidatingCommand;
        let ctx = CommandContext::mock();

        // Should fail validation
        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_command_example() {
        let example = CommandExample::new("List models", "inferno models list");
        assert_eq!(example.description, "List models");
        assert_eq!(example.command, "inferno models list");
        assert!(example.output.is_none());

        let example_with_output = CommandExample::with_output(
            "Count models",
            "inferno models list --json",
            r#"{"count": 5}"#,
        );
        assert!(example_with_output.output.is_some());
    }
}