//! Audit Command - New Architecture
//!
//! This module demonstrates the migration of the audit command to the new
//! CLI architecture. Focuses on querying, statistics, export, and configuration.
//!
//! Note: This is a focused migration covering the most commonly used subcommands.
//! Full audit functionality (monitor, tail, search) remains available through the original module.

use crate::{
    audit::{
        AuditConfiguration, AuditEvent, AuditLogger, AuditQuery, EventType, ExportFormat, Severity,
        SortField, SortOrder,
    },
    config::Config,
    interfaces::cli::{Command, CommandContext, CommandOutput},
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use std::{path::PathBuf, time::SystemTime};
use tracing::info;

// ============================================================================
// AuditQueryCmd - Query audit events
// ============================================================================

/// Query audit events
pub struct AuditQueryCmd {
    config: Config,
    event_types: Option<Vec<EventType>>,
    severities: Option<Vec<Severity>>,
    actors: Option<Vec<String>>,
    resources: Option<Vec<String>>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    limit: usize,
    offset: usize,
    sort_by: SortField,
    sort_order: SortOrder,
    search: Option<String>,
}

impl AuditQueryCmd {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: Config,
        event_types: Option<Vec<EventType>>,
        severities: Option<Vec<Severity>>,
        actors: Option<Vec<String>>,
        resources: Option<Vec<String>>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: usize,
        offset: usize,
        sort_by: SortField,
        sort_order: SortOrder,
        search: Option<String>,
    ) -> Self {
        Self {
            config,
            event_types,
            severities,
            actors,
            resources,
            start_time,
            end_time,
            limit,
            offset,
            sort_by,
            sort_order,
            search,
        }
    }
}

#[async_trait]
impl Command for AuditQueryCmd {
    fn name(&self) -> &str {
        "audit query"
    }

    fn description(&self) -> &str {
        "Query audit events"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.limit == 0 {
            anyhow::bail!("Limit must be at least 1");
        }

        if self.limit > 10000 {
            anyhow::bail!("Limit cannot exceed 10000");
        }

        if let (Some(start), Some(end)) = (&self.start_time, &self.end_time) {
            if end < start {
                anyhow::bail!("End time must be after start time");
            }
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Querying audit events");

        let audit_config = AuditConfiguration::default();
        let logger = AuditLogger::new(audit_config).await?;

        // Convert DateTime<Utc> to SystemTime for query
        let start_time = self.start_time.map(SystemTime::from);
        let end_time = self.end_time.map(SystemTime::from);

        let query = AuditQuery {
            event_types: self.event_types.clone(),
            severities: self.severities.clone(),
            actors: self.actors.clone(),
            resources: self.resources.clone(),
            start_time,
            end_time,
            limit: Some(self.limit),
            offset: Some(self.offset),
            sort_by: Some(self.sort_by.clone()),
            sort_order: Some(self.sort_order.clone()),
            search_text: self.search.clone(),
            ..Default::default()
        };

        let events = logger.query_events(query).await?;

        // Human-readable output
        if !ctx.json_output {
            println!("Found {} audit events", events.len());
            println!();

            if !events.is_empty() {
                println!(
                    "{:<20} {:<15} {:<10} {:<20} {:<30}",
                    "TIMESTAMP", "EVENT_TYPE", "SEVERITY", "ACTOR", "ACTION"
                );
                println!("{}", "-".repeat(95));

                for event in &events {
                    // Format SystemTime for display
                    let timestamp = event
                        .timestamp
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map(|d| {
                            chrono::DateTime::<Utc>::from_timestamp(d.as_secs() as i64, 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                .unwrap_or_else(|| "Unknown".to_string())
                        })
                        .unwrap_or_else(|_| "Invalid".to_string());

                    println!(
                        "{:<20} {:<15} {:<10} {:<20} {:<30}",
                        timestamp,
                        format!("{:?}", event.event_type),
                        format!("{:?}", event.severity),
                        &event.actor.id[..20.min(event.actor.id.len())],
                        &event.action[..30.min(event.action.len())],
                    );
                }
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!("Found {} audit events", events.len()),
            json!({
                "events": events.iter().map(|e| {
                    let timestamp_secs = e.timestamp
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    json!({
                        "id": e.id,
                        "timestamp_secs": timestamp_secs,
                        "event_type": format!("{:?}", e.event_type),
                        "severity": format!("{:?}", e.severity),
                        "actor": {
                            "id": e.actor.id,
                            "type": format!("{:?}", e.actor.actor_type),
                            "name": e.actor.name,
                        },
                        "resource": {
                            "id": e.resource.id,
                            "type": format!("{:?}", e.resource.resource_type),
                            "name": e.resource.name,
                        },
                        "action": e.action,
                    })
                }).collect::<Vec<_>>(),
                "total": events.len(),
                "limit": self.limit,
                "offset": self.offset,
            }),
        ))
    }
}

// ============================================================================
// AuditStats - Show audit statistics
// ============================================================================

/// Show audit statistics
pub struct AuditStats {
    config: Config,
    range_hours: u64,
}

impl AuditStats {
    pub fn new(config: Config, range_hours: u64) -> Self {
        Self {
            config,
            range_hours,
        }
    }
}

#[async_trait]
impl Command for AuditStats {
    fn name(&self) -> &str {
        "audit stats"
    }

    fn description(&self) -> &str {
        "Show audit statistics"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.range_hours == 0 {
            anyhow::bail!("Time range must be at least 1 hour");
        }

        if self.range_hours > 8760 {
            anyhow::bail!("Time range cannot exceed 8760 hours (1 year)");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!(
            "Generating audit statistics for last {} hours",
            self.range_hours
        );

        let audit_config = AuditConfiguration::default();
        let logger = AuditLogger::new(audit_config).await?;

        let start_dt = Utc::now() - Duration::hours(self.range_hours as i64);
        let start_time = SystemTime::from(start_dt);

        let query = AuditQuery {
            start_time: Some(start_time),
            limit: Some(100000), // High limit for stats
            ..Default::default()
        };

        let events = logger.query_events(query).await?;

        // Calculate statistics
        let total_events = events.len();
        let event_type_counts = count_by_event_type(&events);
        let severity_counts = count_by_severity(&events);
        let hourly_rate = if self.range_hours > 0 {
            total_events as f64 / self.range_hours as f64
        } else {
            0.0
        };

        // Human-readable output
        if !ctx.json_output {
            println!("=== Audit Statistics ===");
            println!("Time range: last {} hours", self.range_hours);
            println!("Total events: {}", total_events);
            println!("Average events/hour: {:.1}", hourly_rate);
            println!();

            println!("Events by type:");
            for (event_type, count) in &event_type_counts {
                println!("  {}: {}", event_type, count);
            }
            println!();

            println!("Events by severity:");
            for (severity, count) in &severity_counts {
                println!("  {}: {}", severity, count);
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!("Statistics for {} events", total_events),
            json!({
                "time_range_hours": self.range_hours,
                "total_events": total_events,
                "average_events_per_hour": hourly_rate,
                "event_types": event_type_counts.iter().map(|(k, v)| {
                    json!({ "type": k, "count": v })
                }).collect::<Vec<_>>(),
                "severities": severity_counts.iter().map(|(k, v)| {
                    json!({ "severity": k, "count": v })
                }).collect::<Vec<_>>(),
            }),
        ))
    }
}

// ============================================================================
// AuditExport - Export audit events
// ============================================================================

/// Export audit events
pub struct AuditExport {
    config: Config,
    output: PathBuf,
    format: ExportFormat,
    event_types: Option<Vec<EventType>>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    limit: Option<usize>,
}

impl AuditExport {
    pub fn new(
        config: Config,
        output: PathBuf,
        format: ExportFormat,
        event_types: Option<Vec<EventType>>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Self {
        Self {
            config,
            output,
            format,
            event_types,
            start_time,
            end_time,
            limit,
        }
    }
}

#[async_trait]
impl Command for AuditExport {
    fn name(&self) -> &str {
        "audit export"
    }

    fn description(&self) -> &str {
        "Export audit events"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if let Some(parent) = self.output.parent() {
            if !parent.exists() {
                anyhow::bail!("Output directory does not exist: {}", parent.display());
            }
        }

        if let Some(limit) = self.limit {
            if limit == 0 {
                anyhow::bail!("Limit must be at least 1");
            }
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Exporting audit events to {}", self.output.display());

        let audit_config = AuditConfiguration::default();
        let logger = AuditLogger::new(audit_config).await?;

        // Convert DateTime<Utc> to SystemTime for query
        let start_time = self.start_time.map(SystemTime::from);
        let end_time = self.end_time.map(SystemTime::from);

        let query = AuditQuery {
            event_types: self.event_types.clone(),
            start_time,
            end_time,
            limit: Some(self.limit.unwrap_or(100000)),
            ..Default::default()
        };

        logger
            .export_events(query, &self.output, self.format.clone())
            .await?;

        // Re-query to get count for output
        let count_query = AuditQuery {
            event_types: self.event_types.clone(),
            start_time,
            end_time,
            limit: Some(self.limit.unwrap_or(100000)),
            ..Default::default()
        };
        let events = logger.query_events(count_query).await?;

        // Human-readable output
        if !ctx.json_output {
            println!(
                "✓ Exported {} events to {}",
                events.len(),
                self.output.display()
            );
            println!("Format: {:?}", self.format);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!("Exported {} events", events.len()),
            json!({
                "output_file": self.output.display().to_string(),
                "format": format!("{:?}", self.format),
                "event_count": events.len(),
            }),
        ))
    }
}

// ============================================================================
// AuditConfigure - Show audit configuration
// ============================================================================

/// Show audit configuration
pub struct AuditConfigure {
    config: Config,
}

impl AuditConfigure {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Command for AuditConfigure {
    fn name(&self) -> &str {
        "audit configure"
    }

    fn description(&self) -> &str {
        "Show audit configuration"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Showing audit configuration");

        let audit_config = AuditConfiguration::default();

        // Human-readable output
        if !ctx.json_output {
            println!("=== Audit Configuration ===");
            println!("Enabled: {}", audit_config.enabled);
            println!("Storage path: {}", audit_config.storage_path.display());
            println!("Max file size: {} MB", audit_config.max_file_size_mb);
            println!("Max files: {}", audit_config.max_files);
            println!("Retention days: {}", audit_config.retention_days);
            println!("Compression: {}", audit_config.compression_enabled);
            println!("Encryption: {}", audit_config.encryption_enabled);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Audit configuration",
            json!({
                "enabled": audit_config.enabled,
                "storage_path": audit_config.storage_path.display().to_string(),
                "max_file_size_mb": audit_config.max_file_size_mb,
                "max_files": audit_config.max_files,
                "retention_days": audit_config.retention_days,
                "compression_enabled": audit_config.compression_enabled,
                "encryption_enabled": audit_config.encryption_enabled,
            }),
        ))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn count_by_event_type(events: &[AuditEvent]) -> Vec<(String, usize)> {
    use std::collections::HashMap;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for event in events {
        let key = format!("{:?}", event.event_type);
        *counts.entry(key).or_insert(0) += 1;
    }
    let mut result: Vec<_> = counts.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1));
    result
}

fn count_by_severity(events: &[AuditEvent]) -> Vec<(String, usize)> {
    use std::collections::HashMap;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for event in events {
        let key = format!("{:?}", event.severity);
        *counts.entry(key).or_insert(0) += 1;
    }
    let mut result: Vec<_> = counts.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_query_validation_zero_limit() {
        let config = Config::default();
        let cmd = AuditQueryCmd::new(
            config.clone(),
            None,
            None,
            None,
            None,
            None,
            None,
            0, // Invalid
            0,
            SortField::Timestamp,
            SortOrder::Descending,
            None,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_audit_stats_validation_zero_range() {
        let config = Config::default();
        let cmd = AuditStats::new(config.clone(), 0);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_audit_export_validation() {
        let config = Config::default();
        let cmd = AuditExport::new(
            config.clone(),
            PathBuf::from("/tmp/audit.json"),
            ExportFormat::Json,
            None,
            None,
            None,
            None,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_audit_configure_validation() {
        let config = Config::default();
        let cmd = AuditConfigure::new(config.clone());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }
}
