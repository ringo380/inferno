//! Command execution pipeline
//!
//! Orchestrates command execution with middleware, validation,
//! and error handling.

use super::{Command, CommandContext, CommandOutput, MiddlewareStack};
use anyhow::Result;
use tracing::{debug, error, info};

/// Command execution pipeline
///
/// Orchestrates the full command execution lifecycle:
/// 1. Run pre-execution middleware
/// 2. Validate command
/// 3. Execute command
/// 4. Run post-execution middleware
/// 5. Handle errors
///
/// # Example
///
/// ```ignore
/// use inferno::interfaces::cli::{CommandPipeline, Command, CommandContext};
///
/// let pipeline = CommandPipeline::builder()
///     .with_logging()
///     .with_metrics()
///     .build();
///
/// let mut ctx = CommandContext::new(config);
/// let output = pipeline.execute(&command, &mut ctx).await?;
/// ```
pub struct CommandPipeline {
    middleware: MiddlewareStack,
    error_handler: Box<dyn ErrorHandler>,
}

impl CommandPipeline {
    /// Create a new pipeline builder
    pub fn builder() -> PipelineBuilder {
        PipelineBuilder::new()
    }

    /// Execute a command through the pipeline
    pub async fn execute(
        &self,
        command: &dyn Command,
        ctx: &mut CommandContext,
    ) -> Result<CommandOutput> {
        let command_name = command.name();

        debug!(
            command = command_name,
            execution_id = %ctx.execution_id,
            "Starting command execution"
        );

        // Run pre-execution middleware
        if let Err(e) = self.middleware.run_before(ctx).await {
            error!(
                command = command_name,
                error = %e,
                "Pre-execution middleware failed"
            );
            let output = self.error_handler.handle(e, ctx, command).await?;
            self.middleware.run_after(ctx, &Ok(output.clone())).await;
            return Ok(output);
        }

        // Validate command
        if let Err(e) = command.validate(ctx).await {
            error!(
                command = command_name,
                error = %e,
                "Command validation failed"
            );
            let output = self.error_handler.handle(e, ctx, command).await?;
            self.middleware.run_after(ctx, &Ok(output.clone())).await;
            return Ok(output);
        }

        // Execute command
        let result = command.execute(ctx).await;

        // Always run post-execution middleware
        self.middleware.run_after(ctx, &result).await;

        // Handle result
        match result {
            Ok(output) => {
                info!(
                    command = command_name,
                    execution_id = %ctx.execution_id,
                    duration_ms = ctx.elapsed().as_millis(),
                    success = output.success,
                    "Command execution completed"
                );
                Ok(output)
            }
            Err(e) => {
                error!(
                    command = command_name,
                    execution_id = %ctx.execution_id,
                    error = %e,
                    "Command execution failed"
                );
                self.error_handler.handle(e, ctx, command).await
            }
        }
    }
}

/// Error handler trait for customizing error responses
#[async_trait::async_trait]
pub trait ErrorHandler: Send + Sync {
    /// Handle an error and convert it to CommandOutput
    async fn handle(
        &self,
        error: anyhow::Error,
        ctx: &CommandContext,
        command: &dyn Command,
    ) -> Result<CommandOutput>;
}

/// Default error handler
pub struct DefaultErrorHandler;

#[async_trait::async_trait]
impl ErrorHandler for DefaultErrorHandler {
    async fn handle(
        &self,
        error: anyhow::Error,
        ctx: &CommandContext,
        command: &dyn Command,
    ) -> Result<CommandOutput> {
        let error_msg = if ctx.is_debug() {
            format!("{:?}", error) // Full error with backtrace
        } else {
            format!("{}", error) // Just the error message
        };

        error!(
            command = command.name(),
            execution_id = %ctx.execution_id,
            error = %error_msg,
            "Command failed"
        );

        Ok(CommandOutput::error(error_msg, 1))
    }
}

/// Builder for constructing command pipelines
pub struct PipelineBuilder {
    middleware: MiddlewareStack,
    error_handler: Option<Box<dyn ErrorHandler>>,
}

impl PipelineBuilder {
    /// Create a new pipeline builder
    pub fn new() -> Self {
        Self {
            middleware: MiddlewareStack::new(),
            error_handler: None,
        }
    }

    /// Add custom middleware to the pipeline
    pub fn with_middleware(mut self, middleware: Box<dyn super::Middleware>) -> Self {
        self.middleware = self.middleware.push(middleware);
        self
    }

    /// Set custom error handler
    pub fn with_error_handler(mut self, handler: Box<dyn ErrorHandler>) -> Self {
        self.error_handler = Some(handler);
        self
    }

    /// Build the pipeline
    pub fn build(self) -> CommandPipeline {
        CommandPipeline {
            middleware: self.middleware,
            error_handler: self
                .error_handler
                .unwrap_or_else(|| Box::new(DefaultErrorHandler)),
        }
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interfaces::cli::{Command, CommandContext};
    use async_trait::async_trait;

    struct SuccessCommand;

    #[async_trait]
    impl Command for SuccessCommand {
        fn name(&self) -> &str {
            "success"
        }

        fn description(&self) -> &str {
            "Always succeeds"
        }

        async fn execute(&self, _ctx: &mut CommandContext) -> Result<CommandOutput> {
            Ok(CommandOutput::success("Success!"))
        }
    }

    struct FailureCommand;

    #[async_trait]
    impl Command for FailureCommand {
        fn name(&self) -> &str {
            "failure"
        }

        fn description(&self) -> &str {
            "Always fails"
        }

        async fn execute(&self, _ctx: &mut CommandContext) -> Result<CommandOutput> {
            anyhow::bail!("Intentional failure")
        }
    }

    struct ValidationFailureCommand;

    #[async_trait]
    impl Command for ValidationFailureCommand {
        fn name(&self) -> &str {
            "validation-fail"
        }

        fn description(&self) -> &str {
            "Fails validation"
        }

        async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
            anyhow::bail!("Validation failed")
        }

        async fn execute(&self, _ctx: &mut CommandContext) -> Result<CommandOutput> {
            Ok(CommandOutput::success("Should not reach here"))
        }
    }

    #[tokio::test]
    async fn test_successful_execution() {
        let pipeline = CommandPipeline::builder().build();
        let command = SuccessCommand;
        let mut ctx = CommandContext::mock();

        let output = pipeline.execute(&command, &mut ctx).await.unwrap();
        assert!(output.success);
        assert_eq!(output.message, Some("Success!".to_string()));
    }

    #[tokio::test]
    async fn test_failed_execution() {
        let pipeline = CommandPipeline::builder().build();
        let command = FailureCommand;
        let mut ctx = CommandContext::mock();

        let output = pipeline.execute(&command, &mut ctx).await.unwrap();
        assert!(!output.success);
        assert!(output.message.unwrap().contains("Intentional failure"));
    }

    #[tokio::test]
    async fn test_validation_failure() {
        let pipeline = CommandPipeline::builder().build();
        let command = ValidationFailureCommand;
        let mut ctx = CommandContext::mock();

        let output = pipeline.execute(&command, &mut ctx).await.unwrap();
        assert!(!output.success);
        assert!(output.message.unwrap().contains("Validation failed"));
    }

    struct TestMiddleware {
        before_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        after_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl super::super::Middleware for TestMiddleware {
        async fn before(&self, _ctx: &mut CommandContext) -> Result<()> {
            self.before_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn after(
            &self,
            _ctx: &mut CommandContext,
            _result: &Result<CommandOutput>,
        ) -> Result<()> {
            self.after_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_middleware_execution() {
        let before_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let after_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let middleware = TestMiddleware {
            before_count: before_count.clone(),
            after_count: after_count.clone(),
        };

        let pipeline = CommandPipeline::builder()
            .with_middleware(Box::new(middleware))
            .build();

        let command = SuccessCommand;
        let mut ctx = CommandContext::mock();

        pipeline.execute(&command, &mut ctx).await.unwrap();

        assert_eq!(before_count.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(after_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_middleware_on_failure() {
        let before_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let after_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let middleware = TestMiddleware {
            before_count: before_count.clone(),
            after_count: after_count.clone(),
        };

        let pipeline = CommandPipeline::builder()
            .with_middleware(Box::new(middleware))
            .build();

        let command = FailureCommand;
        let mut ctx = CommandContext::mock();

        pipeline.execute(&command, &mut ctx).await.unwrap();

        // Middleware should still run on failure
        assert_eq!(before_count.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(after_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}