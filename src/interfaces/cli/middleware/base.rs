//! Middleware system for command execution
//!
//! Provides a middleware pattern for cross-cutting concerns like
//! logging, metrics, validation, and error handling.

use anyhow::Result;
use async_trait::async_trait;

// Import from parent modules
use super::super::{CommandContext, CommandOutput};

/// Middleware trait for command execution pipeline
///
/// Middleware can run code before and after command execution,
/// enabling cross-cutting concerns like logging, metrics, and validation.
///
/// # Example
///
/// ```ignore
/// use inferno::interfaces::cli::{Middleware, CommandContext, CommandOutput};
/// use async_trait::async_trait;
///
/// pub struct LoggingMiddleware;
///
/// #[async_trait]
/// impl Middleware for LoggingMiddleware {
///     async fn before(&self, ctx: &mut CommandContext) -> Result<()> {
///         println!("Starting command: {}", ctx.execution_id);
///         Ok(())
///     }
///
///     async fn after(&self, ctx: &mut CommandContext, result: &Result<CommandOutput>) -> Result<()> {
///         println!("Finished command: {} in {:?}", ctx.execution_id, ctx.elapsed());
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Run before command execution
    ///
    /// Can modify the context, add state, or return an error to prevent
    /// command execution.
    async fn before(&self, ctx: &mut CommandContext) -> Result<()>;

    /// Run after command execution
    ///
    /// Runs even if the command returned an error. The result parameter
    /// contains the command's result.
    ///
    /// Middleware errors are logged but don't affect the command result.
    async fn after(&self, ctx: &mut CommandContext, result: &Result<CommandOutput>) -> Result<()>;

    /// Middleware name for identification
    fn name(&self) -> &str {
        "unnamed-middleware"
    }

    /// Whether this middleware is enabled
    ///
    /// Can be overridden to conditionally enable/disable middleware
    /// based on context or configuration.
    fn is_enabled(&self, _ctx: &CommandContext) -> bool {
        true
    }
}

/// Middleware stack for composing multiple middleware
pub struct MiddlewareStack {
    middleware: Vec<Box<dyn Middleware>>,
}

impl MiddlewareStack {
    /// Create a new empty middleware stack
    pub fn new() -> Self {
        Self {
            middleware: Vec::new(),
        }
    }

    /// Add middleware to the stack
    pub fn push(mut self, middleware: Box<dyn Middleware>) -> Self {
        self.middleware.push(middleware);
        self
    }

    /// Run all "before" hooks in order
    pub async fn run_before(&self, ctx: &mut CommandContext) -> Result<()> {
        for mw in &self.middleware {
            if mw.is_enabled(ctx) {
                mw.before(ctx).await?;
            }
        }
        Ok(())
    }

    /// Run all "after" hooks in reverse order
    pub async fn run_after(&self, ctx: &mut CommandContext, result: &Result<CommandOutput>) {
        for mw in self.middleware.iter().rev() {
            if mw.is_enabled(ctx) {
                if let Err(e) = mw.after(ctx, result).await {
                    tracing::warn!(
                        middleware = mw.name(),
                        error = %e,
                        "Middleware after hook failed"
                    );
                }
            }
        }
    }

    /// Get number of middleware in stack
    pub fn len(&self) -> usize {
        self.middleware.len()
    }

    /// Check if stack is empty
    pub fn is_empty(&self) -> bool {
        self.middleware.is_empty()
    }
}

impl Default for MiddlewareStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestMiddleware {
        name: String,
        before_called: std::sync::Arc<std::sync::atomic::AtomicBool>,
        after_called: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    impl TestMiddleware {
        fn new(name: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                before_called: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                after_called: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            }
        }
    }

    #[async_trait]
    impl Middleware for TestMiddleware {
        async fn before(&self, _ctx: &mut CommandContext) -> Result<()> {
            self.before_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn after(
            &self,
            _ctx: &mut CommandContext,
            _result: &Result<CommandOutput>,
        ) -> Result<()> {
            self.after_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_middleware_execution() {
        let mw = TestMiddleware::new("test");
        let before_called = mw.before_called.clone();
        let after_called = mw.after_called.clone();

        let mut ctx = CommandContext::mock();

        // Before hook
        mw.before(&mut ctx).await.unwrap();
        assert!(before_called.load(std::sync::atomic::Ordering::SeqCst));

        // After hook
        let result = Ok(CommandOutput::success("test"));
        mw.after(&mut ctx, &result).await.unwrap();
        assert!(after_called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_middleware_stack() {
        let mw1 = TestMiddleware::new("first");
        let mw2 = TestMiddleware::new("second");

        let before1 = mw1.before_called.clone();
        let before2 = mw2.before_called.clone();
        let after1 = mw1.after_called.clone();
        let after2 = mw2.after_called.clone();

        let stack = MiddlewareStack::new()
            .push(Box::new(mw1))
            .push(Box::new(mw2));

        assert_eq!(stack.len(), 2);

        let mut ctx = CommandContext::mock();

        // Run before hooks
        stack.run_before(&mut ctx).await.unwrap();
        assert!(before1.load(std::sync::atomic::Ordering::SeqCst));
        assert!(before2.load(std::sync::atomic::Ordering::SeqCst));

        // Run after hooks
        let result = Ok(CommandOutput::success("test"));
        stack.run_after(&mut ctx, &result).await;
        assert!(after1.load(std::sync::atomic::Ordering::SeqCst));
        assert!(after2.load(std::sync::atomic::Ordering::SeqCst));
    }

    struct ConditionalMiddleware {
        enabled: bool,
    }

    #[async_trait]
    impl Middleware for ConditionalMiddleware {
        async fn before(&self, ctx: &mut CommandContext) -> Result<()> {
            ctx.set_state("conditional_ran", true);
            Ok(())
        }

        async fn after(
            &self,
            _ctx: &mut CommandContext,
            _result: &Result<CommandOutput>,
        ) -> Result<()> {
            Ok(())
        }

        fn is_enabled(&self, _ctx: &CommandContext) -> bool {
            self.enabled
        }
    }

    #[tokio::test]
    async fn test_conditional_middleware() {
        let enabled_mw = ConditionalMiddleware { enabled: true };
        let disabled_mw = ConditionalMiddleware { enabled: false };

        let mut ctx1 = CommandContext::mock();
        let mut ctx2 = CommandContext::mock();

        // Enabled middleware should run
        assert!(enabled_mw.is_enabled(&ctx1));
        enabled_mw.before(&mut ctx1).await.unwrap();
        assert_eq!(ctx1.get_state::<bool>("conditional_ran"), Some(&true));

        // Disabled middleware should not run
        assert!(!disabled_mw.is_enabled(&ctx2));
        // Don't call before() since is_enabled() is false
        assert_eq!(ctx2.get_state::<bool>("conditional_ran"), None);
    }
}
