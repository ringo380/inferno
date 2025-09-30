//! Metrics Command - New Architecture
//!
//! This module demonstrates the migration of the metrics command to the new
//! CLI architecture. Focuses on core metrics export operations.
//!
//! Note: This is a focused migration covering the most commonly used subcommands.
//! Full metrics functionality (including HTTP server) remains available through the original module.

use crate::config::Config;
use crate::interfaces::cli::{Command, CommandContext, CommandOutput};
use crate::metrics::MetricsCollector;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use tracing::info;

// ============================================================================
// MetricsJson - Export metrics in JSON format
// ============================================================================

/// Export metrics in JSON format
pub struct MetricsJson {
    config: Config,
}

impl MetricsJson {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Command for MetricsJson {
    fn name(&self) -> &str {
        "metrics json"
    }

    fn description(&self) -> &str {
        "Export metrics in JSON format"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // No validation needed for JSON export
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Exporting metrics in JSON format");

        let mut collector = MetricsCollector::new();
        collector.start_event_processing().await?;

        let json_output = collector.export_metrics_json().await?;

        // Human-readable output (already JSON)
        if !ctx.json_output {
            println!("{}", json_output);
        }

        // Structured output - parse the JSON string back to Value
        let metrics_value: serde_json::Value = serde_json::from_str(&json_output)?;

        Ok(CommandOutput::success_with_data(
            "Metrics exported in JSON format",
            json!({
                "format": "json",
                "metrics": metrics_value,
            }),
        ))
    }
}

// ============================================================================
// MetricsPrometheus - Export metrics in Prometheus format
// ============================================================================

/// Export metrics in Prometheus format
pub struct MetricsPrometheus {
    config: Config,
}

impl MetricsPrometheus {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Command for MetricsPrometheus {
    fn name(&self) -> &str {
        "metrics prometheus"
    }

    fn description(&self) -> &str {
        "Export metrics in Prometheus format"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // No validation needed for Prometheus export
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Exporting metrics in Prometheus format");

        let mut collector = MetricsCollector::new();
        collector.start_event_processing().await?;

        let prometheus_output = collector.export_prometheus_format().await?;

        // Human-readable output (Prometheus text format)
        if !ctx.json_output {
            println!("{}", prometheus_output);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Metrics exported in Prometheus format",
            json!({
                "format": "prometheus",
                "content_type": "text/plain; version=0.0.4; charset=utf-8",
                "metrics": prometheus_output,
            }),
        ))
    }
}

// ============================================================================
// MetricsSnapshot - Show detailed metrics snapshot
// ============================================================================

/// Show detailed metrics snapshot
pub struct MetricsSnapshot {
    config: Config,
    pretty: bool,
}

impl MetricsSnapshot {
    pub fn new(config: Config, pretty: bool) -> Self {
        Self { config, pretty }
    }
}

#[async_trait]
impl Command for MetricsSnapshot {
    fn name(&self) -> &str {
        "metrics snapshot"
    }

    fn description(&self) -> &str {
        "Show detailed metrics snapshot"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // No validation needed for snapshot
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Creating metrics snapshot");

        let mut collector = MetricsCollector::new();
        collector.start_event_processing().await?;

        let snapshot = collector.get_snapshot().await?;

        // Human-readable output
        if !ctx.json_output {
            if self.pretty {
                println!("{}", serde_json::to_string_pretty(&snapshot)?);
            } else {
                println!("{}", serde_json::to_string(&snapshot)?);
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Metrics snapshot created",
            json!({
                "snapshot": snapshot,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "pretty": self.pretty,
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_json_validation() {
        let config = Config::default();
        let cmd = MetricsJson::new(config.clone());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_prometheus_validation() {
        let config = Config::default();
        let cmd = MetricsPrometheus::new(config.clone());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_snapshot_validation() {
        let config = Config::default();
        let cmd = MetricsSnapshot::new(config.clone(), false);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_snapshot_pretty() {
        let config = Config::default();
        let cmd = MetricsSnapshot::new(config.clone(), true);
        let mut ctx = CommandContext::new(config);

        // Should execute without errors
        let result = cmd.execute(&mut ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.data.is_some());

        let data = output.data.unwrap();
        assert_eq!(data["pretty"], true);
    }
}