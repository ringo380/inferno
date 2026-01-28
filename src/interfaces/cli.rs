//! Modern CLI command architecture
//!
//! This module provides a unified command architecture with:
//! - **Command Trait**: Standard interface for all commands
//! - **Command Pipeline**: Orchestrates execution with middleware
//! - **Middleware System**: Cross-cutting concerns (logging, metrics, validation)
//! - **Command Context**: Shared state and configuration
//! - **Structured Output**: Type-safe command results
//!
//! # Architecture
//!
//! The CLI architecture follows a pipeline pattern:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    CommandPipeline                      │
//! ├─────────────────────────────────────────────────────────┤
//! │ 1. Run pre-execution middleware (logging, setup)        │
//! │ 2. Validate command arguments and context               │
//! │ 3. Execute command with shared context                  │
//! │ 4. Run post-execution middleware (metrics, cleanup)     │
//! │ 5. Handle errors and format output                      │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ## Creating a Command
//!
//! ```ignore
//! use inferno::interfaces::cli::{Command, CommandContext, CommandOutput};
//! use async_trait::async_trait;
//! use anyhow::Result;
//!
//! pub struct MyCommand {
//!     arg1: String,
//!     arg2: i32,
//! }
//!
//! #[async_trait]
//! impl Command for MyCommand {
//!     fn name(&self) -> &str {
//!         "my-command"
//!     }
//!
//!     fn description(&self) -> &str {
//!         "Does something useful"
//!     }
//!
//!     async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
//!         // Access config
//!         let models_dir = &ctx.config.models_dir;
//!
//!         // Use command args
//!         let arg1 = &self.arg1;
//!
//!         // Do work...
//!
//!         // Return structured output
//!         Ok(CommandOutput::success("Command completed successfully"))
//!     }
//! }
//! ```
//!
//! ## Executing a Command
//!
//! ```ignore
//! use inferno::interfaces::cli::CommandPipeline;
//!
//! let pipeline = CommandPipeline::builder()
//!     .with_logging()
//!     .with_metrics()
//!     .build();
//!
//! let command = MyCommand {
//!     arg1: "value".to_string(),
//!     arg2: 42,
//! };
//!
//! let mut ctx = CommandContext::new(config);
//! let output = pipeline.execute(&command, &mut ctx).await?;
//!
//! if output.success {
//!     println!("{}", output.to_display());
//! } else {
//!     std::process::exit(output.exit_code);
//! }
//! ```
//!
//! ## Creating Middleware
//!
//! ```ignore
//! use inferno::interfaces::cli::{Middleware, CommandContext, CommandOutput};
//! use async_trait::async_trait;
//! use anyhow::Result;
//!
//! pub struct LoggingMiddleware;
//!
//! #[async_trait]
//! impl Middleware for LoggingMiddleware {
//!     async fn before(&self, ctx: &mut CommandContext) -> Result<()> {
//!         tracing::info!("Starting command: {}", ctx.execution_id);
//!         Ok(())
//!     }
//!
//!     async fn after(&self, ctx: &mut CommandContext, result: &Result<CommandOutput>) -> Result<()> {
//!         tracing::info!("Finished command in {:?}", ctx.elapsed());
//!         Ok(())
//!     }
//!
//!     fn name(&self) -> &str {
//!         "logging"
//!     }
//! }
//! ```
//!
//! # Benefits
//!
//! - **Reduced Duplication**: Cross-cutting concerns handled once in middleware
//! - **Better Testing**: Commands can be tested in isolation with mock context
//! - **Consistent Behavior**: All commands get logging, metrics, error handling
//! - **Extensibility**: Easy to add new middleware or commands
//! - **Type Safety**: Compile-time guarantees for command structure
//!
//! # Migration
//!
//! The old command pattern is still supported for backward compatibility:
//!
//! ```ignore
//! // Old style (still works)
//! pub async fn execute(args: CommandArgs, config: &Config) -> Result<()> {
//!     // Command logic
//! }
//!
//! // New style (recommended)
//! pub struct CommandImpl { args: CommandArgs }
//!
//! #[async_trait]
//! impl Command for CommandImpl {
//!     async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
//!         // Command logic with context
//!     }
//! }
//! ```
//!
//! See `.claude/plans/phase2-cli-architecture.md` for migration guide.

pub mod command;
pub mod context;
pub mod middleware;
pub mod output;
pub mod pipeline;

// Re-export main types for convenience
pub use command::{Command, CommandExample, CommandMetadata};
pub use context::CommandContext;
pub use middleware::{LoggingMiddleware, MetricsMiddleware, Middleware, MiddlewareStack};
pub use output::{CommandOutput, OutputLevel};
pub use pipeline::{CommandPipeline, DefaultErrorHandler, ErrorHandler, PipelineBuilder};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_module_exports() {
        // Verify all main types are accessible
        let _: Option<Box<dyn Command>> = None;
        let _: Option<Box<dyn Middleware>> = None;
        let _: Option<Box<dyn ErrorHandler>> = None;
        let _ = CommandContext::mock();
        let _ = CommandOutput::success("test");
        let _ = MiddlewareStack::new();
        let _ = CommandPipeline::builder();
    }
}
