//! Logging middleware for command execution
//!
//! Provides automatic logging of command execution lifecycle events.

use super::super::{CommandContext, CommandOutput};
use super::base::Middleware;
use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, error, info, warn};

/// Logging middleware
///
/// Automatically logs command execution start, completion, and errors.
/// Log level depends on context verbosity and command result.
pub struct LoggingMiddleware {
    /// Whether to log execution details
    log_details: bool,
}

impl LoggingMiddleware {
    /// Create new logging middleware with default settings
    pub fn new() -> Self {
        Self { log_details: true }
    }

    /// Create logging middleware without detailed logging
    pub fn minimal() -> Self {
        Self { log_details: false }
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn before(&self, ctx: &mut CommandContext) -> Result<()> {
        if self.log_details {
            if ctx.is_debug() {
                debug!(
                    execution_id = %ctx.execution_id,
                    json_output = ctx.json_output,
                    verbosity = ctx.verbosity,
                    "Starting command execution"
                );
            } else if ctx.is_verbose() {
                info!(
                    execution_id = %ctx.execution_id,
                    "Starting command"
                );
            }
        }

        // Store start time in context for later use
        ctx.set_state("logging_start", std::time::Instant::now());

        Ok(())
    }

    async fn after(
        &self,
        ctx: &mut CommandContext,
        result: &Result<CommandOutput>,
    ) -> Result<()> {
        let duration = ctx.elapsed();

        match result {
            Ok(output) => {
                if output.success {
                    if self.log_details {
                        info!(
                            execution_id = %ctx.execution_id,
                            duration_ms = duration.as_millis(),
                            exit_code = output.exit_code,
                            "Command completed successfully"
                        );
                    }
                } else {
                    warn!(
                        execution_id = %ctx.execution_id,
                        duration_ms = duration.as_millis(),
                        exit_code = output.exit_code,
                        message = ?output.message,
                        "Command completed with warnings"
                    );
                }
            }
            Err(e) => {
                error!(
                    execution_id = %ctx.execution_id,
                    duration_ms = duration.as_millis(),
                    error = %e,
                    "Command execution failed"
                );
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "logging"
    }

    fn is_enabled(&self, ctx: &CommandContext) -> bool {
        // Enable unless explicitly disabled via state
        ctx.get_state::<bool>("disable_logging")
            .map(|v| !v)
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_logging_middleware_success() {
        let middleware = LoggingMiddleware::new();
        let mut ctx = CommandContext::mock();

        // Before hook
        middleware.before(&mut ctx).await.unwrap();
        assert!(ctx.get_state::<std::time::Instant>("logging_start").is_some());

        // After hook with success
        let output = CommandOutput::success("Test passed");
        middleware.after(&mut ctx, &Ok(output)).await.unwrap();
    }

    #[tokio::test]
    async fn test_logging_middleware_failure() {
        let middleware = LoggingMiddleware::new();
        let mut ctx = CommandContext::mock();

        middleware.before(&mut ctx).await.unwrap();

        // After hook with error
        let err = anyhow::anyhow!("Test error");
        middleware.after(&mut ctx, &Err(err)).await.unwrap();
    }

    #[tokio::test]
    async fn test_disabled_logging() {
        let middleware = LoggingMiddleware::new();
        let mut ctx = CommandContext::mock();

        // Disable logging
        ctx.set_state("disable_logging", true);

        assert!(!middleware.is_enabled(&ctx));
    }

    #[tokio::test]
    async fn test_minimal_logging() {
        let middleware = LoggingMiddleware::minimal();
        assert!(!middleware.log_details);

        let mut ctx = CommandContext::mock();
        middleware.before(&mut ctx).await.unwrap();
    }
}