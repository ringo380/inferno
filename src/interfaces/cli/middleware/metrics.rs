//! Metrics middleware for command execution
//!
//! Automatically collects and records metrics about command execution.

use super::super::{CommandContext, CommandOutput};
use super::base::Middleware;
use anyhow::Result;
use async_trait::async_trait;
use std::time::Instant;
use tracing::debug;

/// Metrics middleware
///
/// Automatically records command execution metrics including:
/// - Execution duration
/// - Success/failure counts
/// - Error rates
/// - Command usage statistics
pub struct MetricsMiddleware {
    /// Prefix for metric names
    metric_prefix: String,
}

impl MetricsMiddleware {
    /// Create new metrics middleware with default prefix
    pub fn new() -> Self {
        Self {
            metric_prefix: "inferno.command".to_string(),
        }
    }

    /// Create metrics middleware with custom prefix
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            metric_prefix: prefix.into(),
        }
    }

    /// Record a metric (placeholder - integrate with actual metrics system)
    fn record_metric(&self, name: &str, value: f64, ctx: &CommandContext) {
        debug!(
            metric = format!("{}.{}", self.metric_prefix, name),
            value = value,
            execution_id = %ctx.execution_id,
            "Recording metric"
        );

        // TODO: Integrate with MetricsCollector
        // ctx.metrics.record_gauge(name, value);
    }

    /// Increment a counter (placeholder)
    fn increment_counter(&self, name: &str, ctx: &CommandContext) {
        debug!(
            counter = format!("{}.{}", self.metric_prefix, name),
            execution_id = %ctx.execution_id,
            "Incrementing counter"
        );

        // TODO: Integrate with MetricsCollector
        // ctx.metrics.increment_counter(name);
    }
}

impl Default for MetricsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for MetricsMiddleware {
    async fn before(&self, ctx: &mut CommandContext) -> Result<()> {
        // Store start time for duration calculation
        ctx.set_state("metrics_start", Instant::now());

        // Increment total command counter
        self.increment_counter("total", ctx);

        Ok(())
    }

    async fn after(&self, ctx: &mut CommandContext, result: &Result<CommandOutput>) -> Result<()> {
        // Calculate duration
        if let Some(start) = ctx.get_state::<Instant>("metrics_start") {
            let duration = start.elapsed();
            self.record_metric("duration_ms", duration.as_millis() as f64, ctx);
        }

        // Record success/failure
        match result {
            Ok(output) => {
                if output.success {
                    self.increment_counter("success", ctx);
                    self.record_metric("exit_code", output.exit_code as f64, ctx);
                } else {
                    self.increment_counter("failure", ctx);
                    self.record_metric("exit_code", output.exit_code as f64, ctx);
                }
            }
            Err(_) => {
                self.increment_counter("error", ctx);
                self.record_metric("exit_code", 1.0, ctx);
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "metrics"
    }

    fn is_enabled(&self, ctx: &CommandContext) -> bool {
        // Enable unless explicitly disabled
        ctx.get_state::<bool>("disable_metrics")
            .map(|v| !v)
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_middleware_success() {
        let middleware = MetricsMiddleware::new();
        let mut ctx = CommandContext::mock();

        middleware.before(&mut ctx).await.unwrap();
        assert!(ctx.get_state::<Instant>("metrics_start").is_some());

        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let output = CommandOutput::success("Test");
        middleware.after(&mut ctx, &Ok(output)).await.unwrap();
    }

    #[tokio::test]
    async fn test_metrics_middleware_failure() {
        let middleware = MetricsMiddleware::new();
        let mut ctx = CommandContext::mock();

        middleware.before(&mut ctx).await.unwrap();

        let err = anyhow::anyhow!("Test error");
        middleware.after(&mut ctx, &Err(err)).await.unwrap();
    }

    #[tokio::test]
    async fn test_custom_prefix() {
        let middleware = MetricsMiddleware::with_prefix("custom.prefix");
        assert_eq!(middleware.metric_prefix, "custom.prefix");
    }

    #[tokio::test]
    async fn test_disabled_metrics() {
        let middleware = MetricsMiddleware::new();
        let mut ctx = CommandContext::mock();

        ctx.set_state("disable_metrics", true);
        assert!(!middleware.is_enabled(&ctx));
    }
}
