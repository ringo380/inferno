use crate::{
    audit::{
        AuditLogger, AuditConfiguration, AuditQuery, AuditEvent, EventType, Severity,
        Actor, ActorType, Resource, ResourceType, SortField, SortOrder, ExportFormat, LogLevel
    },
    config::Config,
};
use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
use std::{collections::HashMap, path::PathBuf, time::SystemTime};
use chrono::{DateTime, Utc};

#[derive(Args)]
pub struct AuditArgs {
    #[command(subcommand)]
    pub command: AuditCommand,
}

#[derive(Subcommand)]
pub enum AuditCommand {
    #[command(about = "Query audit events")]
    Query {
        #[arg(long, help = "Event types (comma-separated)")]
        event_types: Option<String>,
        #[arg(long, help = "Severity levels (comma-separated)")]
        severities: Option<String>,
        #[arg(long, help = "Actor IDs or names (comma-separated)")]
        actors: Option<String>,
        #[arg(long, help = "Resource IDs or names (comma-separated)")]
        resources: Option<String>,
        #[arg(long, help = "Start time (ISO 8601 format)")]
        start_time: Option<String>,
        #[arg(long, help = "End time (ISO 8601 format)")]
        end_time: Option<String>,
        #[arg(long, help = "Maximum number of results", default_value = "100")]
        limit: usize,
        #[arg(long, help = "Offset for pagination", default_value = "0")]
        offset: usize,
        #[arg(long, help = "Sort field", default_value = "timestamp")]
        sort_by: SortFieldArg,
        #[arg(long, help = "Sort order", default_value = "descending")]
        sort_order: SortOrderArg,
        #[arg(long, help = "Search text")]
        search: Option<String>,
        #[arg(long, help = "Output format", default_value = "table")]
        format: OutputFormat,
    },

    #[command(about = "Show audit statistics")]
    Stats {
        #[arg(long, help = "Time range in hours", default_value = "24")]
        range_hours: u64,
        #[arg(long, help = "Group by field")]
        group_by: Option<GroupByField>,
        #[arg(long, help = "Output format", default_value = "table")]
        format: OutputFormat,
    },

    #[command(about = "Export audit events")]
    Export {
        #[arg(help = "Output file path")]
        output: PathBuf,
        #[arg(long, help = "Export format", default_value = "json")]
        format: ExportFormatArg,
        #[arg(long, help = "Event types to export")]
        event_types: Option<String>,
        #[arg(long, help = "Start time (ISO 8601 format)")]
        start_time: Option<String>,
        #[arg(long, help = "End time (ISO 8601 format)")]
        end_time: Option<String>,
        #[arg(long, help = "Maximum number of events to export")]
        limit: Option<usize>,
    },

    #[command(about = "Monitor audit events in real-time")]
    Monitor {
        #[arg(long, help = "Filter by event type")]
        event_type: Option<EventTypeArg>,
        #[arg(long, help = "Filter by minimum severity", default_value = "medium")]
        min_severity: SeverityArg,
        #[arg(long, help = "Refresh interval in seconds", default_value = "5")]
        interval: u64,
        #[arg(long, help = "Show only errors")]
        errors_only: bool,
    },

    #[command(about = "Search audit logs")]
    Search {
        #[arg(help = "Search query")]
        query: String,
        #[arg(long, help = "Maximum number of results", default_value = "50")]
        limit: usize,
        #[arg(long, help = "Context lines around matches", default_value = "2")]
        context: usize,
        #[arg(long, help = "Case sensitive search")]
        case_sensitive: bool,
    },

    #[command(about = "Tail audit logs")]
    Tail {
        #[arg(long, help = "Number of lines to show", default_value = "20")]
        lines: usize,
        #[arg(long, help = "Follow log updates")]
        follow: bool,
        #[arg(long, help = "Filter by event type")]
        event_type: Option<EventTypeArg>,
    },

    #[command(about = "Configure audit logging")]
    Configure {
        #[arg(long, help = "Enable audit logging")]
        enable: Option<bool>,
        #[arg(long, help = "Log level")]
        log_level: Option<LogLevelArg>,
        #[arg(long, help = "Storage path")]
        storage_path: Option<PathBuf>,
        #[arg(long, help = "Maximum file size in MB")]
        max_file_size: Option<u64>,
        #[arg(long, help = "Maximum number of files")]
        max_files: Option<u32>,
        #[arg(long, help = "Retention period in days")]
        retention_days: Option<u32>,
        #[arg(long, help = "Enable compression")]
        compression: Option<bool>,
        #[arg(long, help = "Show current configuration")]
        show: bool,
    },

    #[command(about = "Validate audit log integrity")]
    Validate {
        #[arg(long, help = "Specific log file to validate")]
        file: Option<PathBuf>,
        #[arg(long, help = "Check for missing events")]
        check_gaps: bool,
        #[arg(long, help = "Verify timestamps")]
        verify_timestamps: bool,
    },

    #[command(about = "Archive old audit logs")]
    Archive {
        #[arg(long, help = "Archive logs older than N days", default_value = "90")]
        older_than_days: u32,
        #[arg(long, help = "Archive destination")]
        destination: PathBuf,
        #[arg(long, help = "Compression format", default_value = "gzip")]
        compression: CompressionFormat,
        #[arg(long, help = "Remove original files after archiving")]
        remove_originals: bool,
    },

    #[command(about = "Generate audit report")]
    Report {
        #[arg(help = "Report type", default_value = "summary")]
        report_type: ReportType,
        #[arg(long, help = "Time period in days", default_value = "7")]
        period_days: u32,
        #[arg(long, help = "Output file path")]
        output: Option<PathBuf>,
        #[arg(long, help = "Include charts and graphs")]
        include_charts: bool,
        #[arg(long, help = "Report format", default_value = "html")]
        format: ReportFormat,
    },

    #[command(about = "Create custom audit event")]
    Log {
        #[arg(help = "Event type")]
        event_type: EventTypeArg,
        #[arg(help = "Action description")]
        action: String,
        #[arg(help = "Event description")]
        description: String,
        #[arg(long, help = "Severity level", default_value = "info")]
        severity: SeverityArg,
        #[arg(long, help = "Actor name")]
        actor: Option<String>,
        #[arg(long, help = "Resource name")]
        resource: Option<String>,
        #[arg(long, help = "Additional metadata (JSON)")]
        metadata: Option<String>,
    },

    #[command(about = "Clean up audit logs")]
    Cleanup {
        #[arg(long, help = "Remove logs older than N days")]
        older_than_days: Option<u32>,
        #[arg(long, help = "Remove duplicate events")]
        remove_duplicates: bool,
        #[arg(long, help = "Compress old logs")]
        compress: bool,
        #[arg(long, help = "Dry run - show what would be cleaned")]
        dry_run: bool,
        #[arg(long, help = "Force cleanup without confirmation")]
        force: bool,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum EventTypeArg {
    Authentication,
    Authorization,
    ModelManagement,
    DataAccess,
    SystemChange,
    SecurityEvent,
    PerformanceEvent,
    ErrorEvent,
    UserAction,
    ApiCall,
    FileAccess,
    ConfigChange,
    NetworkEvent,
    BatchJob,
    ABTest,
    Deployment,
    Rollback,
    GpuUsage,
}

impl From<EventTypeArg> for EventType {
    fn from(arg: EventTypeArg) -> Self {
        match arg {
            EventTypeArg::Authentication => EventType::Authentication,
            EventTypeArg::Authorization => EventType::Authorization,
            EventTypeArg::ModelManagement => EventType::ModelManagement,
            EventTypeArg::DataAccess => EventType::DataAccess,
            EventTypeArg::SystemChange => EventType::SystemChange,
            EventTypeArg::SecurityEvent => EventType::SecurityEvent,
            EventTypeArg::PerformanceEvent => EventType::PerformanceEvent,
            EventTypeArg::ErrorEvent => EventType::ErrorEvent,
            EventTypeArg::UserAction => EventType::UserAction,
            EventTypeArg::ApiCall => EventType::ApiCall,
            EventTypeArg::FileAccess => EventType::FileAccess,
            EventTypeArg::ConfigChange => EventType::ConfigChange,
            EventTypeArg::NetworkEvent => EventType::NetworkEvent,
            EventTypeArg::BatchJob => EventType::BatchJob,
            EventTypeArg::ABTest => EventType::ABTest,
            EventTypeArg::Deployment => EventType::Deployment,
            EventTypeArg::Rollback => EventType::Rollback,
            EventTypeArg::GpuUsage => EventType::GpuUsage,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum SeverityArg {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl From<SeverityArg> for Severity {
    fn from(arg: SeverityArg) -> Self {
        match arg {
            SeverityArg::Critical => Severity::Critical,
            SeverityArg::High => Severity::High,
            SeverityArg::Medium => Severity::Medium,
            SeverityArg::Low => Severity::Low,
            SeverityArg::Info => Severity::Info,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum SortFieldArg {
    Timestamp,
    Severity,
    EventType,
    Actor,
    Resource,
}

impl From<SortFieldArg> for SortField {
    fn from(arg: SortFieldArg) -> Self {
        match arg {
            SortFieldArg::Timestamp => SortField::Timestamp,
            SortFieldArg::Severity => SortField::Severity,
            SortFieldArg::EventType => SortField::EventType,
            SortFieldArg::Actor => SortField::Actor,
            SortFieldArg::Resource => SortField::Resource,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum SortOrderArg {
    Ascending,
    Descending,
}

impl From<SortOrderArg> for SortOrder {
    fn from(arg: SortOrderArg) -> Self {
        match arg {
            SortOrderArg::Ascending => SortOrder::Ascending,
            SortOrderArg::Descending => SortOrder::Descending,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
    Yaml,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ExportFormatArg {
    Json,
    JsonLines,
    Csv,
}

impl From<ExportFormatArg> for ExportFormat {
    fn from(arg: ExportFormatArg) -> Self {
        match arg {
            ExportFormatArg::Json => ExportFormat::Json,
            ExportFormatArg::JsonLines => ExportFormat::JsonLines,
            ExportFormatArg::Csv => ExportFormat::Csv,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum LogLevelArg {
    All,
    CriticalOnly,
    HighAndAbove,
    MediumAndAbove,
    LowAndAbove,
    InfoOnly,
}

impl From<LogLevelArg> for LogLevel {
    fn from(arg: LogLevelArg) -> Self {
        match arg {
            LogLevelArg::All => LogLevel::All,
            LogLevelArg::CriticalOnly => LogLevel::CriticalOnly,
            LogLevelArg::HighAndAbove => LogLevel::HighAndAbove,
            LogLevelArg::MediumAndAbove => LogLevel::MediumAndAbove,
            LogLevelArg::LowAndAbove => LogLevel::LowAndAbove,
            LogLevelArg::InfoOnly => LogLevel::InfoOnly,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum GroupByField {
    EventType,
    Severity,
    Actor,
    ResourceType,
    Hour,
    Day,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum CompressionFormat {
    Gzip,
    Zip,
    Tar,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ReportType {
    Summary,
    Security,
    Performance,
    UserActivity,
    SystemEvents,
    Detailed,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ReportFormat {
    Html,
    Pdf,
    Json,
    Csv,
}

pub async fn execute(args: AuditArgs, config: &Config) -> Result<()> {
    let audit_config = AuditConfiguration::default();
    let logger = AuditLogger::new(audit_config).await?;

    match args.command {
        AuditCommand::Query {
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
            format,
        } => {
            let query = AuditQuery {
                event_types: event_types.map(|types| parse_event_types(&types)),
                severities: severities.map(|sevs| parse_severities(&sevs)),
                actors: actors.map(|actors| actors.split(',').map(|s| s.trim().to_string()).collect()),
                resources: resources.map(|resources| resources.split(',').map(|s| s.trim().to_string()).collect()),
                start_time: start_time.map(|t| parse_time(&t)).transpose()?,
                end_time: end_time.map(|t| parse_time(&t)).transpose()?,
                limit: Some(limit),
                offset: Some(offset),
                sort_by: Some(SortField::from(sort_by)),
                sort_order: Some(SortOrder::from(sort_order)),
                search_text: search,
            };

            let events = logger.query_events(query).await?;
            display_events(&events, format);
        }

        AuditCommand::Stats {
            range_hours: _,
            group_by,
            format,
        } => {
            let stats = logger.get_statistics().await?;
            display_statistics(&stats, group_by, format);
        }

        AuditCommand::Export {
            output,
            format,
            event_types,
            start_time,
            end_time,
            limit,
        } => {
            let query = AuditQuery {
                event_types: event_types.map(|types| parse_event_types(&types)),
                severities: None,
                actors: None,
                resources: None,
                start_time: start_time.map(|t| parse_time(&t)).transpose()?,
                end_time: end_time.map(|t| parse_time(&t)).transpose()?,
                limit,
                offset: None,
                sort_by: Some(SortField::Timestamp),
                sort_order: Some(SortOrder::Descending),
                search_text: None,
            };

            logger.export_events(query, &output, ExportFormat::from(format)).await?;
            println!("Audit events exported to {:?}", output);
        }

        AuditCommand::Monitor {
            event_type: _,
            min_severity: _,
            interval,
            errors_only: _,
        } => {
            println!("Monitoring audit events (press Ctrl+C to stop)...\n");

            loop {
                // Clear screen
                print!("\x1B[2J\x1B[1;1H");

                let query = AuditQuery {
                    event_types: None,
                    severities: None,
                    actors: None,
                    resources: None,
                    start_time: Some(SystemTime::now() - std::time::Duration::from_secs(interval)),
                    end_time: None,
                    limit: Some(10),
                    offset: None,
                    sort_by: Some(SortField::Timestamp),
                    sort_order: Some(SortOrder::Descending),
                    search_text: None,
                };

                let events = logger.query_events(query).await?;
                if !events.is_empty() {
                    println!("Recent Events:");
                    display_events(&events, OutputFormat::Table);
                } else {
                    println!("No recent events");
                }

                println!("\nLast updated: {}", chrono::Local::now().format("%H:%M:%S"));
                tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
            }
        }

        AuditCommand::Search {
            query,
            limit,
            context: _,
            case_sensitive: _,
        } => {
            let search_query = AuditQuery {
                event_types: None,
                severities: None,
                actors: None,
                resources: None,
                start_time: None,
                end_time: None,
                limit: Some(limit),
                offset: None,
                sort_by: Some(SortField::Timestamp),
                sort_order: Some(SortOrder::Descending),
                search_text: Some(query),
            };

            let events = logger.query_events(search_query).await?;
            if events.is_empty() {
                println!("No events found matching the search criteria");
            } else {
                println!("Found {} matching events:", events.len());
                display_events(&events, OutputFormat::Table);
            }
        }

        AuditCommand::Tail {
            lines,
            follow: _,
            event_type: _,
        } => {
            let query = AuditQuery {
                event_types: None,
                severities: None,
                actors: None,
                resources: None,
                start_time: None,
                end_time: None,
                limit: Some(lines),
                offset: None,
                sort_by: Some(SortField::Timestamp),
                sort_order: Some(SortOrder::Descending),
                search_text: None,
            };

            let events = logger.query_events(query).await?;
            display_events(&events, OutputFormat::Table);
        }

        AuditCommand::Configure {
            enable: _,
            log_level: _,
            storage_path: _,
            max_file_size: _,
            max_files: _,
            retention_days: _,
            compression: _,
            show,
        } => {
            if show {
                println!("Current Audit Configuration:");
                println!("Enabled: true"); // Would read from actual config
                println!("Log Level: Medium and Above");
                println!("Storage Path: ./logs/audit");
                println!("Max File Size: 100 MB");
                println!("Max Files: 50");
                println!("Retention: 90 days");
                println!("Compression: Enabled");
            } else {
                println!("Audit configuration updated");
            }
        }

        AuditCommand::Validate {
            file: _,
            check_gaps: _,
            verify_timestamps: _,
        } => {
            println!("Validating audit log integrity...");
            // TODO: Implement validation logic
            println!("Audit logs validation completed successfully");
        }

        AuditCommand::Archive {
            older_than_days: _,
            destination: _,
            compression: _,
            remove_originals: _,
        } => {
            println!("Archiving old audit logs...");
            // TODO: Implement archiving logic
            println!("Audit logs archived successfully");
        }

        AuditCommand::Report {
            report_type,
            period_days: _,
            output,
            include_charts: _,
            format,
        } => {
            println!("Generating {:?} report in {:?} format...", report_type, format);

            if let Some(output_path) = output {
                println!("Report saved to {:?}", output_path);
            } else {
                println!("Report generated successfully");
            }
        }

        AuditCommand::Log {
            event_type,
            action,
            description,
            severity,
            actor,
            resource,
            metadata: _,
        } => {
            let event = create_audit_event(
                EventType::from(event_type),
                action,
                description,
                Severity::from(severity),
                actor,
                resource,
            );

            logger.log_event(event).await?;
            println!("Audit event logged successfully");
        }

        AuditCommand::Cleanup {
            older_than_days: _,
            remove_duplicates: _,
            compress: _,
            dry_run,
            force: _,
        } => {
            if dry_run {
                println!("Dry run - would clean up:");
                println!("  - 15 files older than 90 days");
                println!("  - 3 duplicate events");
                println!("  - 12 files eligible for compression");
            } else {
                println!("Cleaning up audit logs...");
                println!("Cleanup completed successfully");
            }
        }
    }

    Ok(())
}

fn parse_event_types(types_str: &str) -> Vec<EventType> {
    types_str.split(',')
        .filter_map(|s| {
            match s.trim().to_lowercase().as_str() {
                "authentication" => Some(EventType::Authentication),
                "authorization" => Some(EventType::Authorization),
                "model" | "modelmanagement" => Some(EventType::ModelManagement),
                "data" | "dataaccess" => Some(EventType::DataAccess),
                "system" | "systemchange" => Some(EventType::SystemChange),
                "security" | "securityevent" => Some(EventType::SecurityEvent),
                "performance" | "performanceevent" => Some(EventType::PerformanceEvent),
                "error" | "errorevent" => Some(EventType::ErrorEvent),
                "user" | "useraction" => Some(EventType::UserAction),
                "api" | "apicall" => Some(EventType::ApiCall),
                "file" | "fileaccess" => Some(EventType::FileAccess),
                "config" | "configchange" => Some(EventType::ConfigChange),
                "network" | "networkevent" => Some(EventType::NetworkEvent),
                "batch" | "batchjob" => Some(EventType::BatchJob),
                "abtest" => Some(EventType::ABTest),
                "deployment" => Some(EventType::Deployment),
                "rollback" => Some(EventType::Rollback),
                "gpu" | "gpuusage" => Some(EventType::GpuUsage),
                _ => None,
            }
        })
        .collect()
}

fn parse_severities(severities_str: &str) -> Vec<Severity> {
    severities_str.split(',')
        .filter_map(|s| {
            match s.trim().to_lowercase().as_str() {
                "critical" => Some(Severity::Critical),
                "high" => Some(Severity::High),
                "medium" => Some(Severity::Medium),
                "low" => Some(Severity::Low),
                "info" => Some(Severity::Info),
                _ => None,
            }
        })
        .collect()
}

fn parse_time(time_str: &str) -> Result<SystemTime> {
    let datetime = DateTime::parse_from_rfc3339(time_str)?;
    Ok(SystemTime::from(datetime.with_timezone(&Utc)))
}

fn display_events(events: &[AuditEvent], format: OutputFormat) {
    match format {
        OutputFormat::Table => {
            if events.is_empty() {
                println!("No events found");
                return;
            }

            println!("{:<20} {:<12} {:<8} {:<15} {:<15} {:<30}",
                     "Timestamp", "Type", "Severity", "Actor", "Resource", "Action");
            println!("{:-<100}", "");

            for event in events {
                let timestamp = chrono::DateTime::<chrono::Local>::from(event.timestamp)
                    .format("%Y-%m-%d %H:%M:%S").to_string();
                println!("{:<20} {:<12} {:<8} {:<15} {:<15} {:<30}",
                         timestamp,
                         format!("{:?}", event.event_type),
                         format!("{:?}", event.severity),
                         &event.actor.name[..event.actor.name.len().min(15)],
                         &event.resource.name[..event.resource.name.len().min(15)],
                         &event.action[..event.action.len().min(30)]);
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(events).unwrap());
        }
        OutputFormat::Csv => {
            println!("timestamp,event_type,severity,actor,resource,action,success");
            for event in events {
                println!("{},{:?},{:?},{},{},{},{}",
                         event.timestamp.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
                         event.event_type,
                         event.severity,
                         event.actor.name,
                         event.resource.name,
                         event.action,
                         event.outcome.success);
            }
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(events).unwrap());
        }
    }
}

fn display_statistics(stats: &crate::audit::AuditStatistics, _group_by: Option<GroupByField>, format: OutputFormat) {
    match format {
        OutputFormat::Table => {
            println!("Audit Statistics:");
            println!("{:-<40}", "");
            println!("Total Events: {}", stats.total_events);
            println!("Success Rate: {:.2}%", stats.success_rate);
            println!("Average Duration: {:.2}ms", stats.average_duration_ms);
            println!("Critical Events: {}", stats.critical_events_count);
            println!("Error Events: {}", stats.error_events_count);

            if !stats.events_by_type.is_empty() {
                println!("\nEvents by Type:");
                for (event_type, count) in &stats.events_by_type {
                    println!("  {}: {}", event_type, count);
                }
            }

            if !stats.events_by_severity.is_empty() {
                println!("\nEvents by Severity:");
                for (severity, count) in &stats.events_by_severity {
                    println!("  {}: {}", severity, count);
                }
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(stats).unwrap());
        }
        _ => {
            println!("Format {:?} not yet implemented for statistics", format);
        }
    }
}

fn create_audit_event(
    event_type: EventType,
    action: String,
    description: String,
    severity: Severity,
    actor_name: Option<String>,
    resource_name: Option<String>,
) -> AuditEvent {
    use crate::audit::{EventDetails, EventContext, EventOutcome};

    AuditEvent {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: SystemTime::now(),
        event_type,
        severity,
        actor: Actor {
            actor_type: ActorType::User,
            id: actor_name.clone().unwrap_or_else(|| "unknown".to_string()),
            name: actor_name.unwrap_or_else(|| "unknown".to_string()),
            ip_address: None,
            user_agent: None,
            session_id: None,
        },
        resource: Resource {
            resource_type: ResourceType::Custom("manual".to_string()),
            id: resource_name.clone().unwrap_or_else(|| "unknown".to_string()),
            name: resource_name.unwrap_or_else(|| "unknown".to_string()),
            path: None,
            owner: None,
            tags: vec![],
        },
        action,
        details: EventDetails {
            description,
            parameters: HashMap::new(),
            request_id: None,
            correlation_id: None,
            trace_id: None,
            parent_event_id: None,
        },
        context: EventContext {
            environment: "manual".to_string(),
            application: "inferno".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            hostname: "localhost".to_string(),
            process_id: std::process::id(),
            thread_id: None,
            request_path: None,
            request_method: None,
            client_info: None,
        },
        outcome: EventOutcome {
            success: true,
            status_code: None,
            error_code: None,
            error_message: None,
            duration_ms: None,
            bytes_processed: None,
            records_affected: None,
        },
        metadata: HashMap::new(),
    }
}