use anyhow::Result;
use chrono::Utc;
use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

use crate::config::Config;
use crate::data_pipeline::{
    AnomalyDetectionConfig, DataPipeline, DataPipelineConfig, DataPipelineSystem, ExecutionStatus,
    PipelineStatus, PipelineType, ValidationRule, ValidationRuleType,
};

// Mock structures for data pipeline metrics and quality reporting
#[derive(Debug, Clone, Serialize)]
struct PipelineMetrics {
    total_executions: u64,
    successful_executions: u64,
    failed_executions: u64,
    average_duration_secs: f64,
    total_records_processed: u64,
    total_data_volume_bytes: u64,
    average_throughput_per_sec: f64,
}

#[derive(Debug, Clone)]
struct DataQualityReport {
    overall_score: f32,
    total_rules: u32,
    passed_rules: u32,
    failed_rules: u32,
    rule_results: Vec<RuleResult>,
}

#[derive(Debug, Clone)]
struct RuleResult {
    rule_name: String,
    passed: bool,
    score: f32,
    message: Option<String>,
}

#[derive(Args)]
pub struct DataPipelineArgs {
    #[command(subcommand)]
    pub command: DataPipelineCommand,
}

#[derive(Subcommand)]
pub enum DataPipelineCommand {
    #[command(about = "Create a new data pipeline")]
    Create {
        #[arg(long, help = "Pipeline name")]
        name: String,

        #[arg(long, help = "Pipeline description")]
        description: Option<String>,

        #[arg(long, help = "Pipeline type (batch, streaming, hybrid)")]
        pipeline_type: Option<String>,

        #[arg(long, help = "Configuration file path")]
        config: Option<PathBuf>,

        #[arg(long, help = "Enable validation")]
        validate: bool,

        #[arg(long, help = "Auto-start after creation")]
        auto_start: bool,

        #[arg(long, help = "Template to use")]
        template: Option<String>,

        #[arg(long, help = "Tags for the pipeline")]
        tags: Vec<String>,
    },

    #[command(about = "List all pipelines")]
    List {
        #[arg(long, help = "Filter by status")]
        status: Option<String>,

        #[arg(long, help = "Filter by type")]
        pipeline_type: Option<String>,

        #[arg(long, help = "Output format (json, yaml, table)")]
        format: Option<String>,

        #[arg(long, help = "Show detailed information")]
        detailed: bool,

        #[arg(long, help = "Filter by tags")]
        tags: Vec<String>,

        #[arg(long, help = "Sort by field")]
        sort_by: Option<String>,

        #[arg(long, help = "Limit number of results")]
        limit: Option<usize>,
    },

    #[command(about = "Show pipeline details")]
    Show {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Output format (json, yaml, table)")]
        format: Option<String>,

        #[arg(long, help = "Show execution history")]
        history: bool,

        #[arg(long, help = "Show metrics")]
        metrics: bool,

        #[arg(long, help = "Show configuration")]
        config: bool,

        #[arg(long, help = "Show validation rules")]
        validation: bool,
    },

    #[command(about = "Start pipeline execution")]
    Start {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Execution parameters (JSON)")]
        params: Option<String>,

        #[arg(long, help = "Override schedule")]
        override_schedule: bool,

        #[arg(long, help = "Dry run mode")]
        dry_run: bool,

        #[arg(long, help = "Wait for completion")]
        wait: bool,

        #[arg(long, help = "Timeout in seconds")]
        timeout: Option<u64>,

        #[arg(long, help = "Force start even if already running")]
        force: bool,
    },

    #[command(about = "Stop pipeline execution")]
    Stop {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Execution ID to stop")]
        execution_id: Option<String>,

        #[arg(long, help = "Force stop")]
        force: bool,

        #[arg(long, help = "Grace period in seconds")]
        grace_period: Option<u64>,

        #[arg(long, help = "Stop all executions")]
        all: bool,
    },

    #[command(about = "Pause pipeline execution")]
    Pause {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Execution ID to pause")]
        execution_id: Option<String>,

        #[arg(long, help = "Pause reason")]
        reason: Option<String>,
    },

    #[command(about = "Resume pipeline execution")]
    Resume {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Execution ID to resume")]
        execution_id: Option<String>,

        #[arg(long, help = "Resume from checkpoint")]
        from_checkpoint: Option<String>,
    },

    #[command(about = "Delete a pipeline")]
    Delete {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Force deletion")]
        force: bool,

        #[arg(long, help = "Delete execution history")]
        with_history: bool,

        #[arg(long, help = "Delete associated data")]
        with_data: bool,

        #[arg(long, help = "Confirm deletion")]
        confirm: bool,
    },

    #[command(about = "Update pipeline configuration")]
    Update {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Configuration file path")]
        config: Option<PathBuf>,

        #[arg(long, help = "Update description")]
        description: Option<String>,

        #[arg(long, help = "Update tags")]
        tags: Vec<String>,

        #[arg(long, help = "Enable/disable pipeline")]
        enabled: Option<bool>,

        #[arg(long, help = "Update schedule")]
        schedule: Option<String>,

        #[arg(long, help = "Validate before updating")]
        validate: bool,
    },

    #[command(about = "Validate pipeline configuration")]
    Validate {
        #[arg(help = "Pipeline name or ID")]
        pipeline: Option<String>,

        #[arg(long, help = "Configuration file path")]
        config: Option<PathBuf>,

        #[arg(long, help = "Validation level (basic, full, strict)")]
        level: Option<String>,

        #[arg(long, help = "Show warnings")]
        warnings: bool,

        #[arg(long, help = "Output format")]
        format: Option<String>,
    },

    #[command(about = "Pipeline execution management")]
    Execution {
        #[command(subcommand)]
        command: ExecutionCommand,
    },

    #[command(about = "Pipeline stage management")]
    Stage {
        #[command(subcommand)]
        command: StageCommand,
    },

    #[command(about = "Pipeline monitoring and metrics")]
    Monitor {
        #[command(subcommand)]
        command: MonitorCommand,
    },

    #[command(about = "Data quality management")]
    Quality {
        #[command(subcommand)]
        command: QualityCommand,
    },

    #[command(about = "Pipeline scheduling")]
    Schedule {
        #[command(subcommand)]
        command: ScheduleCommand,
    },

    #[command(about = "Template management")]
    Template {
        #[command(subcommand)]
        command: TemplateCommand,
    },

    #[command(about = "Import/export pipelines")]
    Import {
        #[arg(long, help = "Import file path")]
        file: PathBuf,

        #[arg(long, help = "Import format (json, yaml)")]
        format: Option<String>,

        #[arg(long, help = "Overwrite existing")]
        overwrite: bool,

        #[arg(long, help = "Validate on import")]
        validate: bool,

        #[arg(long, help = "Dry run")]
        dry_run: bool,
    },

    #[command(about = "Export pipeline configuration")]
    Export {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Export file path")]
        file: Option<PathBuf>,

        #[arg(long, help = "Export format (json, yaml)")]
        format: Option<String>,

        #[arg(long, help = "Include execution history")]
        with_history: bool,

        #[arg(long, help = "Include metrics")]
        with_metrics: bool,
    },

    #[command(about = "Clone existing pipeline")]
    Clone {
        #[arg(help = "Source pipeline name or ID")]
        source: String,

        #[arg(help = "New pipeline name")]
        name: String,

        #[arg(long, help = "Clone configuration only")]
        config_only: bool,

        #[arg(long, help = "Update description")]
        description: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ExecutionCommand {
    #[command(about = "List pipeline executions")]
    List {
        #[arg(help = "Pipeline name or ID")]
        pipeline: Option<String>,

        #[arg(long, help = "Filter by status")]
        status: Option<String>,

        #[arg(long, help = "Limit results")]
        limit: Option<usize>,

        #[arg(long, help = "Show detailed information")]
        detailed: bool,

        #[arg(long, help = "Output format")]
        format: Option<String>,
    },

    #[command(about = "Show execution details")]
    Show {
        #[arg(help = "Execution ID")]
        execution_id: String,

        #[arg(long, help = "Show logs")]
        logs: bool,

        #[arg(long, help = "Show metrics")]
        metrics: bool,

        #[arg(long, help = "Show stages")]
        stages: bool,

        #[arg(long, help = "Output format")]
        format: Option<String>,
    },

    #[command(about = "Cancel execution")]
    Cancel {
        #[arg(help = "Execution ID")]
        execution_id: String,

        #[arg(long, help = "Force cancellation")]
        force: bool,

        #[arg(long, help = "Reason for cancellation")]
        reason: Option<String>,
    },

    #[command(about = "Retry failed execution")]
    Retry {
        #[arg(help = "Execution ID")]
        execution_id: String,

        #[arg(long, help = "Retry from specific stage")]
        from_stage: Option<String>,

        #[arg(long, help = "Override parameters")]
        params: Option<String>,
    },

    #[command(about = "Show execution logs")]
    Logs {
        #[arg(help = "Execution ID")]
        execution_id: String,

        #[arg(long, help = "Follow logs")]
        follow: bool,

        #[arg(long, help = "Show last N lines")]
        tail: Option<usize>,

        #[arg(long, help = "Filter by stage")]
        stage: Option<String>,

        #[arg(long, help = "Log level filter")]
        level: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum StageCommand {
    #[command(about = "List pipeline stages")]
    List {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Show configuration")]
        config: bool,

        #[arg(long, help = "Output format")]
        format: Option<String>,
    },

    #[command(about = "Add new stage")]
    Add {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(help = "Stage name")]
        name: String,

        #[arg(long, help = "Stage type")]
        stage_type: String,

        #[arg(long, help = "Configuration file")]
        config: Option<PathBuf>,

        #[arg(long, help = "Insert position")]
        position: Option<usize>,

        #[arg(long, help = "Dependencies")]
        depends_on: Vec<String>,
    },

    #[command(about = "Remove stage")]
    Remove {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(help = "Stage name")]
        stage: String,

        #[arg(long, help = "Force removal")]
        force: bool,
    },

    #[command(about = "Update stage configuration")]
    Update {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(help = "Stage name")]
        stage: String,

        #[arg(long, help = "Configuration file")]
        config: Option<PathBuf>,

        #[arg(long, help = "Update parameters")]
        params: Option<String>,
    },

    #[command(about = "Test stage execution")]
    Test {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(help = "Stage name")]
        stage: String,

        #[arg(long, help = "Test data path")]
        test_data: Option<PathBuf>,

        #[arg(long, help = "Dry run")]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum MonitorCommand {
    #[command(about = "Show real-time pipeline status")]
    Status {
        #[arg(help = "Pipeline name or ID")]
        pipeline: Option<String>,

        #[arg(long, help = "Refresh interval in seconds")]
        refresh: Option<u64>,

        #[arg(long, help = "Show metrics")]
        metrics: bool,

        #[arg(long, help = "Show alerts")]
        alerts: bool,
    },

    #[command(about = "Show pipeline metrics")]
    Metrics {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Time range (1h, 24h, 7d, 30d)")]
        range: Option<String>,

        #[arg(long, help = "Metric types")]
        metrics: Vec<String>,

        #[arg(long, help = "Output format")]
        format: Option<String>,

        #[arg(long, help = "Export to file")]
        export: Option<PathBuf>,
    },

    #[command(about = "Configure alerts")]
    Alerts {
        #[command(subcommand)]
        command: AlertCommand,
    },

    #[command(about = "Show dashboard")]
    Dashboard {
        #[arg(long, help = "Web interface port")]
        port: Option<u16>,

        #[arg(long, help = "Bind address")]
        bind: Option<String>,

        #[arg(long, help = "Open browser")]
        open: bool,
    },
}

#[derive(Subcommand)]
pub enum AlertCommand {
    #[command(about = "List alerts")]
    List {
        #[arg(long, help = "Filter by severity")]
        severity: Option<String>,

        #[arg(long, help = "Filter by status")]
        status: Option<String>,

        #[arg(long, help = "Limit results")]
        limit: Option<usize>,
    },

    #[command(about = "Create alert rule")]
    Create {
        #[arg(help = "Alert name")]
        name: String,

        #[arg(long, help = "Condition expression")]
        condition: String,

        #[arg(long, help = "Severity level")]
        severity: String,

        #[arg(long, help = "Notification channels")]
        channels: Vec<String>,

        #[arg(long, help = "Configuration file")]
        config: Option<PathBuf>,
    },

    #[command(about = "Update alert rule")]
    Update {
        #[arg(help = "Alert name")]
        name: String,

        #[arg(long, help = "Configuration file")]
        config: Option<PathBuf>,

        #[arg(long, help = "Enable/disable")]
        enabled: Option<bool>,
    },

    #[command(about = "Delete alert rule")]
    Delete {
        #[arg(help = "Alert name")]
        name: String,

        #[arg(long, help = "Confirm deletion")]
        confirm: bool,
    },
}

#[derive(Subcommand)]
pub enum QualityCommand {
    #[command(about = "Run data quality checks")]
    Check {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Execution ID")]
        execution_id: Option<String>,

        #[arg(long, help = "Stage name")]
        stage: Option<String>,

        #[arg(long, help = "Quality rules to run")]
        rules: Vec<String>,

        #[arg(long, help = "Output format")]
        format: Option<String>,
    },

    #[command(about = "Show quality report")]
    Report {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Time range")]
        range: Option<String>,

        #[arg(long, help = "Report type")]
        report_type: Option<String>,

        #[arg(long, help = "Export to file")]
        export: Option<PathBuf>,
    },

    #[command(about = "Manage validation rules")]
    Rules {
        #[command(subcommand)]
        command: RulesCommand,
    },

    #[command(about = "Anomaly detection")]
    Anomaly {
        #[command(subcommand)]
        command: AnomalyCommand,
    },
}

#[derive(Subcommand)]
pub enum RulesCommand {
    #[command(about = "List validation rules")]
    List {
        #[arg(help = "Pipeline name or ID")]
        pipeline: Option<String>,

        #[arg(long, help = "Rule type")]
        rule_type: Option<String>,

        #[arg(long, help = "Show detailed information")]
        detailed: bool,
    },

    #[command(about = "Add validation rule")]
    Add {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(help = "Rule name")]
        name: String,

        #[arg(long, help = "Rule type")]
        rule_type: String,

        #[arg(long, help = "Rule expression")]
        expression: String,

        #[arg(long, help = "Severity level")]
        severity: Option<String>,

        #[arg(long, help = "Configuration file")]
        config: Option<PathBuf>,
    },

    #[command(about = "Update validation rule")]
    Update {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(help = "Rule name")]
        rule: String,

        #[arg(long, help = "Configuration file")]
        config: Option<PathBuf>,

        #[arg(long, help = "Enable/disable")]
        enabled: Option<bool>,
    },

    #[command(about = "Remove validation rule")]
    Remove {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(help = "Rule name")]
        rule: String,

        #[arg(long, help = "Confirm removal")]
        confirm: bool,
    },
}

#[derive(Subcommand)]
pub enum AnomalyCommand {
    #[command(about = "Detect anomalies")]
    Detect {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Detection method")]
        method: Option<String>,

        #[arg(long, help = "Time range")]
        range: Option<String>,

        #[arg(long, help = "Sensitivity level")]
        sensitivity: Option<f64>,
    },

    #[command(about = "Configure anomaly detection")]
    Configure {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Configuration file")]
        config: PathBuf,

        #[arg(long, help = "Enable/disable")]
        enabled: Option<bool>,
    },

    #[command(about = "Show anomaly history")]
    History {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Time range")]
        range: Option<String>,

        #[arg(long, help = "Severity filter")]
        severity: Option<String>,

        #[arg(long, help = "Limit results")]
        limit: Option<usize>,
    },
}

#[derive(Subcommand)]
pub enum ScheduleCommand {
    #[command(about = "Show pipeline schedule")]
    Show {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Show next executions")]
        next: Option<usize>,

        #[arg(long, help = "Output format")]
        format: Option<String>,
    },

    #[command(about = "Update pipeline schedule")]
    Update {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Cron expression")]
        cron: Option<String>,

        #[arg(long, help = "Interval in seconds")]
        interval: Option<u64>,

        #[arg(long, help = "Enable/disable schedule")]
        enabled: Option<bool>,

        #[arg(long, help = "Timezone")]
        timezone: Option<String>,
    },

    #[command(about = "Trigger scheduled execution")]
    Trigger {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(long, help = "Override parameters")]
        params: Option<String>,

        #[arg(long, help = "Skip if already running")]
        skip_if_running: bool,
    },

    #[command(about = "List all scheduled pipelines")]
    List {
        #[arg(long, help = "Show disabled schedules")]
        show_disabled: bool,

        #[arg(long, help = "Show next execution times")]
        show_next: bool,

        #[arg(long, help = "Output format")]
        format: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum TemplateCommand {
    #[command(about = "List available templates")]
    List {
        #[arg(long, help = "Template category")]
        category: Option<String>,

        #[arg(long, help = "Show detailed information")]
        detailed: bool,

        #[arg(long, help = "Output format")]
        format: Option<String>,
    },

    #[command(about = "Show template details")]
    Show {
        #[arg(help = "Template name")]
        template: String,

        #[arg(long, help = "Show configuration")]
        config: bool,

        #[arg(long, help = "Output format")]
        format: Option<String>,
    },

    #[command(about = "Create template from pipeline")]
    Create {
        #[arg(help = "Pipeline name or ID")]
        pipeline: String,

        #[arg(help = "Template name")]
        name: String,

        #[arg(long, help = "Template description")]
        description: Option<String>,

        #[arg(long, help = "Template category")]
        category: Option<String>,

        #[arg(long, help = "Template tags")]
        tags: Vec<String>,
    },

    #[command(about = "Update template")]
    Update {
        #[arg(help = "Template name")]
        template: String,

        #[arg(long, help = "Configuration file")]
        config: Option<PathBuf>,

        #[arg(long, help = "Update description")]
        description: Option<String>,

        #[arg(long, help = "Update tags")]
        tags: Vec<String>,
    },

    #[command(about = "Delete template")]
    Delete {
        #[arg(help = "Template name")]
        template: String,

        #[arg(long, help = "Confirm deletion")]
        confirm: bool,
    },

    #[command(about = "Apply template to create pipeline")]
    Apply {
        #[arg(help = "Template name")]
        template: String,

        #[arg(help = "New pipeline name")]
        name: String,

        #[arg(long, help = "Template parameters")]
        params: Option<String>,

        #[arg(long, help = "Configuration overrides")]
        config: Option<PathBuf>,
    },
}

pub async fn execute(args: DataPipelineArgs, config: &Config) -> Result<()> {
    let system = DataPipelineSystem::new_simple(config.data_pipeline.clone()).await?;

    match args.command {
        DataPipelineCommand::Create {
            name,
            description,
            pipeline_type,
            config: config_file,
            validate,
            auto_start,
            template,
            tags,
        } => {
            println!("Creating data pipeline: {}", name);

            let pipeline_type = pipeline_type.as_deref().unwrap_or("batch");
            let pipeline_type = match pipeline_type {
                "batch" => PipelineType::Batch,
                "streaming" => PipelineType::Streaming,
                "hybrid" => PipelineType::Hybrid,
                _ => return Err(anyhow::anyhow!("Invalid pipeline type: {}", pipeline_type)),
            };

            let mut pipeline_config = if let Some(config_file) = config_file {
                let config_content = std::fs::read_to_string(config_file)?;
                serde_json::from_str(&config_content)?
            } else if let Some(template_name) = template {
                get_template_config(&template_name)?
            } else {
                create_default_pipeline_config(pipeline_type)?
            };

            pipeline_config.name = name.clone();
            if let Some(desc) = description {
                pipeline_config.description = Some(desc);
            }
            pipeline_config.tags = tags;

            if validate {
                println!("Validating pipeline configuration...");
                validate_pipeline_config(&pipeline_config)?;
                println!("✓ Configuration is valid");
            }

            let pipeline_id = system.create_pipeline_from_config(pipeline_config).await?;
            println!("✓ Pipeline created with ID: {}", pipeline_id);

            if auto_start {
                println!("Starting pipeline...");
                let execution_id = system.start_pipeline(&pipeline_id, None).await?;
                println!("✓ Pipeline started with execution ID: {}", execution_id);
            }
        }

        DataPipelineCommand::List {
            status,
            pipeline_type,
            format,
            detailed,
            tags,
            sort_by,
            limit,
        } => {
            let pipelines = system.list_pipelines_with_ids().await?;

            let mut filtered_pipelines: Vec<_> = pipelines.into_iter().collect();

            if let Some(status_filter) = status {
                let status_enum = parse_pipeline_status(&status_filter)?;
                filtered_pipelines.retain(|(_, pipeline)| pipeline.status == status_enum);
            }

            if let Some(type_filter) = pipeline_type {
                let type_enum = parse_pipeline_type(&type_filter)?;
                filtered_pipelines
                    .retain(|(_, pipeline)| pipeline.config.pipeline_type == type_enum);
            }

            if !tags.is_empty() {
                filtered_pipelines.retain(|(_, pipeline)| {
                    tags.iter().any(|tag| pipeline.config.tags.contains(tag))
                });
            }

            if let Some(sort_field) = sort_by {
                match sort_field.as_str() {
                    "name" => filtered_pipelines
                        .sort_by(|(_, a), (_, b)| a.config.name.cmp(&b.config.name)),
                    "created" => {
                        filtered_pipelines.sort_by(|(_, a), (_, b)| a.created_at.cmp(&b.created_at))
                    }
                    "status" => filtered_pipelines.sort_by(|(_, a), (_, b)| {
                        format!("{:?}", a.status).cmp(&format!("{:?}", b.status))
                    }),
                    _ => return Err(anyhow::anyhow!("Invalid sort field: {}", sort_field)),
                }
            }

            if let Some(limit_count) = limit {
                filtered_pipelines.truncate(limit_count);
            }

            match format.as_deref().unwrap_or("table") {
                "json" => {
                    let output: Value = filtered_pipelines
                        .into_iter()
                        .map(|(id, pipeline)| {
                            json!({
                                "id": id,
                                "name": pipeline.config.name,
                                "type": format!("{:?}", pipeline.config.pipeline_type),
                                "status": format!("{:?}", pipeline.status),
                                "created_at": pipeline.created_at,
                                "description": pipeline.config.description,
                                "tags": pipeline.config.tags,
                            })
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                "yaml" => {
                    let output: Value = filtered_pipelines
                        .into_iter()
                        .map(|(id, pipeline)| {
                            json!({
                                "id": id,
                                "name": pipeline.config.name,
                                "type": format!("{:?}", pipeline.config.pipeline_type),
                                "status": format!("{:?}", pipeline.status),
                                "created_at": pipeline.created_at,
                                "description": pipeline.config.description,
                                "tags": pipeline.config.tags,
                            })
                        })
                        .collect();
                    println!("{}", serde_yaml::to_string(&output)?);
                }
                "table" | _ => {
                    if detailed {
                        println!(
                            "{:<36} {:<30} {:<12} {:<12} {:<20} {:<50}",
                            "ID", "NAME", "TYPE", "STATUS", "CREATED", "DESCRIPTION"
                        );
                        println!("{}", "-".repeat(150));
                        for (id, pipeline) in filtered_pipelines {
                            println!(
                                "{:<36} {:<30} {:<12} {:<12} {:<20} {:<50}",
                                id,
                                truncate_string(&pipeline.config.name, 30),
                                format!("{:?}", pipeline.config.pipeline_type),
                                format!("{:?}", pipeline.status),
                                pipeline.created_at.format("%Y-%m-%d %H:%M:%S"),
                                truncate_string(
                                    &pipeline.config.description.unwrap_or_default(),
                                    50
                                )
                            );
                        }
                    } else {
                        println!(
                            "{:<36} {:<30} {:<12} {:<12}",
                            "ID", "NAME", "TYPE", "STATUS"
                        );
                        println!("{}", "-".repeat(90));
                        for (id, pipeline) in filtered_pipelines {
                            println!(
                                "{:<36} {:<30} {:<12} {:<12}",
                                id,
                                truncate_string(&pipeline.config.name, 30),
                                format!("{:?}", pipeline.config.pipeline_type),
                                format!("{:?}", pipeline.status)
                            );
                        }
                    }
                }
            }
        }

        DataPipelineCommand::Show {
            pipeline,
            format,
            history,
            metrics,
            config: show_config,
            validation,
        } => {
            let pipeline_data = system.get_pipeline(&pipeline).await?;

            match format.as_deref().unwrap_or("table") {
                "json" => {
                    let mut output = json!({
                        "id": pipeline,
                        "name": pipeline_data.config.name,
                        "type": format!("{:?}", pipeline_data.config.pipeline_type),
                        "status": format!("{:?}", pipeline_data.status),
                        "created_at": pipeline_data.created_at,
                        "updated_at": pipeline_data.updated_at,
                        "description": pipeline_data.config.description,
                        "tags": pipeline_data.config.tags,
                    });

                    if show_config {
                        output["config"] = serde_json::to_value(&pipeline_data.config)?;
                    }

                    if history {
                        let executions = system.get_execution_history(&pipeline).await?;
                        output["executions"] = serde_json::to_value(executions)?;
                    }

                    if metrics {
                        let pipeline_metrics = system.get_pipeline_metrics(&pipeline).await?;
                        output["metrics"] = serde_json::to_value(pipeline_metrics)?;
                    }

                    if validation {
                        let validation_rules = system.get_validation_rules(&pipeline).await?;
                        output["validation_rules"] = serde_json::to_value(validation_rules)?;
                    }

                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                "yaml" => {
                    let mut output = json!({
                        "id": pipeline,
                        "name": pipeline_data.config.name,
                        "type": format!("{:?}", pipeline_data.config.pipeline_type),
                        "status": format!("{:?}", pipeline_data.status),
                        "created_at": pipeline_data.created_at,
                        "updated_at": pipeline_data.updated_at,
                        "description": pipeline_data.config.description,
                        "tags": pipeline_data.config.tags,
                    });

                    if show_config {
                        output["config"] = serde_json::to_value(&pipeline_data.config)?;
                    }

                    if history {
                        let executions = system.get_execution_history(&pipeline).await?;
                        output["executions"] = serde_json::to_value(executions)?;
                    }

                    if metrics {
                        let pipeline_metrics = system.get_pipeline_metrics(&pipeline).await?;
                        output["metrics"] = serde_json::to_value(pipeline_metrics)?;
                    }

                    if validation {
                        let validation_rules = system.get_validation_rules(&pipeline).await?;
                        output["validation_rules"] = serde_json::to_value(validation_rules)?;
                    }

                    println!("{}", serde_yaml::to_string(&output)?);
                }
                "table" | _ => {
                    println!("Pipeline Details");
                    println!("================");
                    println!("ID: {}", pipeline);
                    println!("Name: {}", pipeline_data.config.name);
                    println!("Type: {:?}", pipeline_data.config.pipeline_type);
                    println!("Status: {:?}", pipeline_data.status);
                    println!("Created: {}", pipeline_data.created_at);
                    println!("Updated: {}", pipeline_data.updated_at);
                    if let Some(desc) = &pipeline_data.config.description {
                        println!("Description: {}", desc);
                    }
                    if !pipeline_data.config.tags.is_empty() {
                        println!("Tags: {}", pipeline_data.config.tags.join(", "));
                    }

                    if show_config {
                        println!("\nConfiguration:");
                        println!("=============");
                        println!("{}", serde_yaml::to_string(&pipeline_data.config)?);
                    }

                    if history {
                        let executions = system.get_execution_history(&pipeline).await?;
                        println!("\nExecution History:");
                        println!("==================");
                        for execution in executions.iter().take(10) {
                            println!(
                                "{} - {} - {:?}",
                                execution.execution_id, execution.start_time, execution.status
                            );
                        }
                    }

                    if metrics {
                        let pipeline_metrics = system.get_pipeline_metrics(&pipeline).await?;
                        println!("\nMetrics:");
                        println!("========");
                        println!("Duration: {}s", pipeline_metrics.duration_seconds);
                        println!("Records Processed: {}", pipeline_metrics.records_processed);
                        println!("Data Size: {} bytes", pipeline_metrics.data_size_bytes);
                        println!("CPU Usage: {:.2}%", pipeline_metrics.cpu_usage_percent);
                    }

                    if validation {
                        let validation_rules = system.get_validation_rules(&pipeline).await?;
                        println!("\nValidation Rules:");
                        println!("=================");
                        for rule in validation_rules {
                            println!("- {} ({:?}): {}", rule.name, rule.rule_type, rule.config);
                        }
                    }
                }
            }
        }

        DataPipelineCommand::Start {
            pipeline,
            params,
            override_schedule,
            dry_run,
            wait,
            timeout,
            force,
        } => {
            if dry_run {
                println!("Dry run mode - validating pipeline execution...");
                let validation_result = system
                    .validate_pipeline_execution(&pipeline, params.as_deref())
                    .await?;
                println!("✓ Pipeline execution validation successful");
                println!(
                    "Estimated duration: {:.2}s",
                    validation_result.estimated_duration_secs
                );
                return Ok(());
            }

            println!("Starting pipeline: {}", pipeline);

            let execution_params = if let Some(params_str) = params {
                let params_map = serde_json::from_str::<HashMap<String, Value>>(&params_str)?;
                Some(serde_json::to_value(params_map)?)
            } else {
                None
            };

            let execution_id = if force {
                system
                    .force_start_pipeline(&pipeline, execution_params)
                    .await?
            } else {
                system.start_pipeline(&pipeline, execution_params).await?
            };

            println!("✓ Pipeline started with execution ID: {}", execution_id);

            if wait {
                println!("Waiting for pipeline completion...");
                let result = system.wait_for_completion(&execution_id).await?;
                // TODO: Implement timeout handling if needed

                match result.status {
                    ExecutionStatus::Success => {
                        println!("✓ Pipeline completed successfully");
                        println!("Duration: {:.2}s", result.duration_secs.unwrap_or(0.0));
                    }
                    ExecutionStatus::Failed => {
                        println!("✗ Pipeline failed");
                        if let Some(error) = result.error {
                            println!("Error: {}", error);
                        }
                        std::process::exit(1);
                    }
                    _ => {
                        println!("Pipeline execution status: {:?}", result.status);
                    }
                }
            }
        }

        DataPipelineCommand::Stop {
            pipeline,
            execution_id,
            force,
            grace_period,
            all,
        } => {
            if all {
                println!("Stopping all executions for pipeline: {}", pipeline);
                system.stop_all_executions().await?;
                println!("✓ All executions stopped");
            } else if let Some(exec_id) = execution_id {
                println!("Stopping execution: {}", exec_id);
                system.stop_execution(&exec_id).await?;
                println!("✓ Execution stopped");
            } else {
                println!("Stopping current execution for pipeline: {}", pipeline);
                system.stop_pipeline(&pipeline, force, grace_period).await?;
                println!("✓ Pipeline stopped");
            }
        }

        DataPipelineCommand::Pause {
            pipeline,
            execution_id,
            reason,
        } => {
            if let Some(exec_id) = execution_id {
                println!("Pausing execution: {}", exec_id);
                system.pause_execution(&exec_id, reason.as_deref()).await?;
                println!("✓ Execution paused");
            } else {
                println!("Pausing pipeline: {}", pipeline);
                system.pause_pipeline(&pipeline, reason.as_deref()).await?;
                println!("✓ Pipeline paused");
            }
        }

        DataPipelineCommand::Resume {
            pipeline,
            execution_id,
            from_checkpoint,
        } => {
            if let Some(exec_id) = execution_id {
                println!("Resuming execution: {}", exec_id);
                system
                    .resume_execution(&exec_id, from_checkpoint.as_deref())
                    .await?;
                println!("✓ Execution resumed");
            } else {
                println!("Resuming pipeline: {}", pipeline);
                system
                    .resume_pipeline(&pipeline, from_checkpoint.as_deref())
                    .await?;
                println!("✓ Pipeline resumed");
            }
        }

        DataPipelineCommand::Delete {
            pipeline,
            force,
            with_history,
            with_data,
            confirm,
        } => {
            if !confirm && !force {
                println!("This will permanently delete the pipeline. Use --confirm to proceed.");
                return Ok(());
            }

            println!("Deleting pipeline: {}", pipeline);
            system
                .delete_pipeline(&pipeline, with_history, with_data)
                .await?;
            println!("✓ Pipeline deleted");
        }

        DataPipelineCommand::Update {
            pipeline,
            config: config_file,
            description,
            tags,
            enabled,
            schedule,
            validate,
        } => {
            println!("Updating pipeline: {}", pipeline);

            let mut updates = HashMap::new();

            if let Some(config_file) = config_file {
                let config_content = std::fs::read_to_string(config_file)?;
                let config: DataPipelineConfig = serde_json::from_str(&config_content)?;

                if validate {
                    validate_pipeline_config(&config)?;
                }

                updates.insert("config".to_string(), serde_json::to_value(config)?);
            }

            if let Some(desc) = description {
                updates.insert("description".to_string(), Value::String(desc));
            }

            if !tags.is_empty() {
                updates.insert("tags".to_string(), serde_json::to_value(tags)?);
            }

            if let Some(enabled_flag) = enabled {
                updates.insert("enabled".to_string(), Value::Bool(enabled_flag));
            }

            if let Some(schedule_expr) = schedule {
                updates.insert("schedule".to_string(), Value::String(schedule_expr));
            }

            system.update_pipeline(&pipeline, updates).await?;
            println!("✓ Pipeline updated");
        }

        DataPipelineCommand::Validate {
            pipeline,
            config: config_file,
            level,
            warnings,
            format,
        } => {
            let validation_level = level.as_deref().unwrap_or("basic");

            if let Some(pipeline_id) = pipeline {
                println!("Validating pipeline: {}", pipeline_id);
                let result = system
                    .validate_pipeline(&pipeline_id, Some(validation_level), warnings)
                    .await?;
                // Convert bool result to ValidationResult for display
                let validation_result = ValidationResult {
                    valid: result,
                    warnings: Vec::new(),
                    errors: Vec::new(),
                    info: Vec::new(),
                };
                display_validation_result(&validation_result, format.as_deref())?;
            } else if let Some(config_file) = config_file {
                println!("Validating configuration file: {}", config_file.display());
                let config_content = std::fs::read_to_string(config_file)?;
                let config: DataPipelineConfig = serde_json::from_str(&config_content)?;
                let result =
                    validate_pipeline_config_detailed(&config, validation_level, warnings)?;
                display_validation_result(&result, format.as_deref())?;
            } else {
                return Err(anyhow::anyhow!(
                    "Either --pipeline or --config must be specified"
                ));
            }
        }

        DataPipelineCommand::Execution { command } => {
            handle_execution_command(command, &system).await?;
        }

        DataPipelineCommand::Stage { command } => {
            handle_stage_command(command, &system).await?;
        }

        DataPipelineCommand::Monitor { command } => {
            handle_monitor_command(command, &system).await?;
        }

        DataPipelineCommand::Quality { command } => {
            handle_quality_command(command, &system).await?;
        }

        DataPipelineCommand::Schedule { command } => {
            handle_schedule_command(command, &system).await?;
        }

        DataPipelineCommand::Template { command } => {
            handle_template_command(command, &system).await?;
        }

        DataPipelineCommand::Import {
            file,
            format,
            overwrite,
            validate,
            dry_run,
        } => {
            println!("Importing pipelines from: {}", file.display());

            let import_format = format.as_deref().unwrap_or("json");
            let result = system
                .import_pipelines(
                    file.to_str().unwrap_or(""),
                    import_format,
                    overwrite,
                    validate,
                    dry_run,
                )
                .await?;

            if dry_run {
                println!(
                    "Dry run completed. {} pipelines would be imported",
                    result.len()
                );
            } else {
                println!("✓ Imported {} pipelines", result.len());
            }

            for name in result {
                println!("  {} - imported", name);
            }
        }

        DataPipelineCommand::Export {
            pipeline,
            file,
            format,
            with_history,
            with_metrics,
        } => {
            println!("Exporting pipeline: {}", pipeline);

            let export_format = format.as_deref().unwrap_or("json");
            let output_file =
                file.unwrap_or_else(|| PathBuf::from(format!("{}.{}", pipeline, export_format)));

            system
                .export_pipeline(
                    &pipeline,
                    output_file.to_str().unwrap_or(""),
                    export_format,
                    with_history,
                    with_metrics,
                )
                .await?;
            println!("✓ Pipeline exported to: {}", output_file.display());
        }

        DataPipelineCommand::Clone {
            source,
            name,
            config_only,
            description,
        } => {
            println!("Cloning pipeline {} to {}", source, name);

            let new_pipeline_id = system
                .clone_pipeline(&source, &name, config_only, description.as_deref())
                .await?;
            println!("✓ Pipeline cloned with ID: {}", new_pipeline_id);
        }
    }

    Ok(())
}

async fn handle_execution_command(
    command: ExecutionCommand,
    system: &DataPipelineSystem,
) -> Result<()> {
    match command {
        ExecutionCommand::List {
            pipeline,
            status,
            limit,
            detailed,
            format,
        } => {
            let executions = if let Some(pipeline_id) = pipeline {
                system.get_execution_history(&pipeline_id).await?
            } else {
                system.list_all_executions().await?
            };

            let mut filtered_executions = executions;

            if let Some(status_filter) = status {
                let status_enum = parse_execution_status(&status_filter)?;
                filtered_executions.retain(|exec| exec.status == status_enum);
            }

            if let Some(limit_count) = limit {
                filtered_executions.truncate(limit_count);
            }

            match format.as_deref().unwrap_or("table") {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&filtered_executions)?);
                }
                "yaml" => {
                    println!("{}", serde_yaml::to_string(&filtered_executions)?);
                }
                "table" | _ => {
                    if detailed {
                        println!(
                            "{:<36} {:<30} {:<12} {:<20} {:<20} {:<10}",
                            "EXECUTION_ID", "PIPELINE", "STATUS", "STARTED", "DURATION", "STAGES"
                        );
                        println!("{}", "-".repeat(140));
                        for execution in filtered_executions {
                            let duration = execution
                                .end_time
                                .map(|end| {
                                    let duration = end - execution.start_time;
                                    format!("{:.2}s", duration.num_seconds() as f64)
                                })
                                .unwrap_or("-".to_string());
                            println!(
                                "{:<36} {:<30} {:<12} {:<20} {:<20} {:<10}",
                                execution.execution_id,
                                truncate_string(&execution.pipeline_id, 30),
                                format!("{:?}", execution.status),
                                execution.start_time.format("%Y-%m-%d %H:%M:%S"),
                                duration,
                                execution.task_executions.len()
                            );
                        }
                    } else {
                        println!(
                            "{:<36} {:<30} {:<12} {:<20}",
                            "EXECUTION_ID", "PIPELINE", "STATUS", "STARTED"
                        );
                        println!("{}", "-".repeat(100));
                        for execution in filtered_executions {
                            println!(
                                "{:<36} {:<30} {:<12} {:<20}",
                                execution.execution_id,
                                truncate_string(&execution.pipeline_id, 30),
                                format!("{:?}", execution.status),
                                execution.start_time.format("%Y-%m-%d %H:%M:%S")
                            );
                        }
                    }
                }
            }
        }

        ExecutionCommand::Show {
            execution_id,
            logs,
            metrics,
            stages,
            format,
        } => {
            let execution = system.get_execution(&execution_id).await?;

            match format.as_deref().unwrap_or("table") {
                "json" => {
                    let mut output = serde_json::to_value(&execution)?;

                    if logs {
                        let execution_logs = system.get_execution_logs(&execution_id).await?;
                        output["logs"] = serde_json::to_value(execution_logs)?;
                    }

                    if metrics {
                        let execution_metrics = system.get_execution_metrics(&execution_id).await?;
                        output["metrics"] = serde_json::to_value(execution_metrics)?;
                    }

                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                "yaml" => {
                    let mut output = serde_json::to_value(&execution)?;

                    if logs {
                        let execution_logs = system.get_execution_logs(&execution_id).await?;
                        output["logs"] = serde_json::to_value(execution_logs)?;
                    }

                    if metrics {
                        let execution_metrics = system.get_execution_metrics(&execution_id).await?;
                        output["metrics"] = serde_json::to_value(execution_metrics)?;
                    }

                    println!("{}", serde_yaml::to_string(&output)?);
                }
                "table" | _ => {
                    println!("Execution Details");
                    println!("=================");
                    println!("ID: {}", execution.id);
                    println!("Pipeline: {}", execution.pipeline_id);
                    println!("Status: {:?}", execution.status);
                    println!("Started: {}", execution.started_at);
                    if let Some(finished) = execution.finished_at {
                        println!("Finished: {}", finished);
                    }
                    if let Some(duration) = execution.duration_secs {
                        println!("Duration: {:.2}s", duration);
                    }
                    if let Some(error) = &execution.error {
                        println!("Error: {}", error);
                    }

                    if stages {
                        println!("\nStages:");
                        println!("=======");
                        for stage in &execution.stages {
                            println!("- {} ({:?})", stage.task_id, stage.status);
                            if let Some(error) = &stage.error_message {
                                println!("  Error: {}", error);
                            }
                        }
                    }

                    if logs {
                        let execution_logs = system.get_execution_logs(&execution_id).await?;
                        println!("\nLogs:");
                        println!("=====");
                        for log in execution_logs.iter().take(50) {
                            println!("[{}] {}: {}", log.timestamp, log.level, log.message);
                        }
                    }

                    if metrics {
                        let execution_metrics = system.get_execution_metrics(&execution_id).await?;
                        println!("\nMetrics:");
                        println!("========");
                        println!("Records Processed: {}", execution_metrics.records_processed);
                        println!("Data Volume: {} bytes", execution_metrics.data_size_bytes);
                        println!("CPU Usage: {:.2}%", execution_metrics.cpu_usage_percent);
                    }
                }
            }
        }

        ExecutionCommand::Cancel {
            execution_id,
            force,
            reason,
        } => {
            println!("Cancelling execution: {}", execution_id);
            system
                .cancel_execution(&execution_id, force, reason.as_deref())
                .await?;
            println!("✓ Execution cancelled");
        }

        ExecutionCommand::Retry {
            execution_id,
            from_stage,
            params,
        } => {
            println!("Retrying execution: {}", execution_id);

            let retry_params = if let Some(params_str) = params {
                Some(serde_json::from_str::<serde_json::Value>(&params_str)?)
            } else {
                None
            };

            let new_execution_id = system
                .retry_execution(&execution_id, from_stage.as_deref(), retry_params)
                .await?;
            println!("✓ Retry started with execution ID: {}", new_execution_id);
        }

        ExecutionCommand::Logs {
            execution_id,
            follow,
            tail,
            stage,
            level,
        } => {
            if follow {
                println!("Following logs for execution: {}", execution_id);
                system
                    .follow_execution_logs(&execution_id, stage.as_deref(), level.as_deref())
                    .await?;
            } else {
                let logs = system.get_execution_logs(&execution_id).await?;

                let mut filtered_logs = logs;

                if let Some(stage_filter) = stage {
                    filtered_logs.retain(|log| log.task_id.as_ref() == Some(&stage_filter));
                }

                if let Some(level_filter) = level {
                    filtered_logs
                        .retain(|log| log.level.to_lowercase() == level_filter.to_lowercase());
                }

                if let Some(tail_count) = tail {
                    let start_index = filtered_logs.len().saturating_sub(tail_count);
                    filtered_logs = filtered_logs.into_iter().skip(start_index).collect();
                }

                for log in filtered_logs {
                    let stage_info = log
                        .task_id
                        .as_ref()
                        .map(|s| format!("[{}] ", s))
                        .unwrap_or_default();
                    println!(
                        "[{}] {}{}: {}",
                        log.timestamp, stage_info, log.level, log.message
                    );
                }
            }
        }
    }

    Ok(())
}

async fn handle_stage_command(command: StageCommand, system: &DataPipelineSystem) -> Result<()> {
    match command {
        StageCommand::List {
            pipeline,
            config,
            format,
        } => {
            let pipeline_data = system.get_pipeline(&pipeline).await?;

            match format.as_deref().unwrap_or("table") {
                "json" => {
                    let output: Value = pipeline_data
                        .tasks
                        .into_iter()
                        .map(|task| {
                            let mut task_json = json!({
                                "name": task.name,
                                "task_type": format!("{:?}", task.task_type),
                                "dependencies": task.dependencies,
                            });

                            if config {
                                task_json["config"] =
                                    serde_json::to_value(&task.config).unwrap_or_default();
                            }

                            task_json
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                "yaml" => {
                    let output: Value = pipeline_data
                        .tasks
                        .into_iter()
                        .map(|task| {
                            let mut task_json = json!({
                                "name": task.name,
                                "task_type": format!("{:?}", task.task_type),
                                "dependencies": task.dependencies,
                            });

                            if config {
                                task_json["config"] =
                                    serde_json::to_value(&task.config).unwrap_or_default();
                            }

                            task_json
                        })
                        .collect();
                    println!("{}", serde_yaml::to_string(&output)?);
                }
                "table" | _ => {
                    println!(
                        "{:<30} {:<15} {:<10} {:<30}",
                        "NAME", "TYPE", "ENABLED", "DEPENDS_ON"
                    );
                    println!("{}", "-".repeat(85));
                    for stage in pipeline_data.config.stages {
                        let depends_on = if stage.dependencies.is_empty() {
                            "-".to_string()
                        } else {
                            stage.dependencies.join(", ")
                        };
                        println!(
                            "{:<30} {:<15} {:<10} {:<30}",
                            stage.name,
                            format!("{:?}", stage.task_type),
                            "true", // PipelineTask doesn't have enabled field, default to true
                            truncate_string(&depends_on, 30)
                        );
                    }
                }
            }
        }

        StageCommand::Add {
            pipeline,
            name,
            stage_type,
            config,
            position,
            depends_on,
        } => {
            println!("Adding stage '{}' to pipeline '{}'", name, pipeline);

            let stage_config = if let Some(config_file) = config {
                let config_content = std::fs::read_to_string(config_file)?;
                Some(serde_json::from_str(&config_content)?)
            } else {
                create_default_stage_config(&stage_type)?
            };

            system
                .add_pipeline_stage(
                    &pipeline,
                    &name,
                    &stage_type,
                    stage_config,
                    position,
                    depends_on,
                )
                .await?;
            println!("✓ Stage added successfully");
        }

        StageCommand::Remove {
            pipeline,
            stage,
            force,
        } => {
            if !force {
                println!(
                    "This will remove the stage '{}' from pipeline '{}'. Use --force to confirm.",
                    stage, pipeline
                );
                return Ok(());
            }

            println!("Removing stage '{}' from pipeline '{}'", stage, pipeline);
            system.remove_pipeline_stage(&pipeline, &stage).await?;
            println!("✓ Stage removed successfully");
        }

        StageCommand::Update {
            pipeline,
            stage,
            config,
            params,
        } => {
            println!("Updating stage '{}' in pipeline '{}'", stage, pipeline);

            let mut updates = HashMap::new();

            if let Some(config_file) = config {
                let config_content = std::fs::read_to_string(config_file)?;
                let stage_config: DataPipelineConfig = serde_json::from_str(&config_content)?;
                updates.insert("config".to_string(), serde_json::to_value(stage_config)?);
            }

            if let Some(params_str) = params {
                let params_value: Value = serde_json::from_str(&params_str)?;
                updates.insert("params".to_string(), params_value);
            }

            system
                .update_pipeline_stage(&pipeline, &stage, updates)
                .await?;
            println!("✓ Stage updated successfully");
        }

        StageCommand::Test {
            pipeline,
            stage,
            test_data,
            dry_run,
        } => {
            println!("Testing stage '{}' in pipeline '{}'", stage, pipeline);

            let test_result = system
                .test_pipeline_stage(&pipeline, &stage, test_data.as_deref(), dry_run)
                .await?;

            if dry_run {
                println!("✓ Stage test validation successful");
            } else {
                println!("✓ Stage test completed");
                println!("Status: {:?}", test_result.status);
                if let Some(duration) = test_result.duration_secs {
                    println!("Duration: {:.2}s", duration);
                }
                if let Some(records) = test_result.records_processed {
                    println!("Records Processed: {}", records);
                }
            }
        }
    }

    Ok(())
}

async fn handle_monitor_command(
    command: MonitorCommand,
    system: &DataPipelineSystem,
) -> Result<()> {
    match command {
        MonitorCommand::Status {
            pipeline,
            refresh,
            metrics,
            alerts,
        } => {
            if let Some(pipeline_id) = pipeline.as_deref() {
                if let Some(refresh_interval) = refresh {
                    println!(
                        "Monitoring pipeline status (refresh every {}s, press Ctrl+C to exit)...",
                        refresh_interval
                    );
                    system
                        .monitor_pipeline_status(pipeline_id, refresh_interval, metrics, alerts)
                        .await?;
                } else {
                    let status = system.get_pipeline_status(pipeline_id).await?;
                    display_pipeline_status(&status, metrics, alerts)?;
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Pipeline ID is required for status monitoring"
                ));
            }
        }

        MonitorCommand::Metrics {
            pipeline,
            range,
            metrics,
            format,
            export,
        } => {
            let time_range = range.as_deref().unwrap_or("24h");
            let metrics_filter = if metrics.is_empty() {
                None
            } else {
                Some(metrics)
            };
            let pipeline_metrics = system
                .get_pipeline_metrics_range(&pipeline, time_range, metrics_filter)
                .await?;

            match format.as_deref().unwrap_or("table") {
                "json" => {
                    let output = serde_json::to_string_pretty(&pipeline_metrics)?;
                    if let Some(export_file) = &export {
                        std::fs::write(export_file, &output)?;
                        println!("Metrics exported to: {}", export_file.display());
                    } else {
                        println!("{}", output);
                    }
                }
                "yaml" => {
                    let output = serde_yaml::to_string(&pipeline_metrics)?;
                    if let Some(export_file) = &export {
                        std::fs::write(export_file, &output)?;
                        println!("Metrics exported to: {}", export_file.display());
                    } else {
                        println!("{}", output);
                    }
                }
                "table" | _ => {
                    let converted_metrics =
                        convert_execution_metrics_to_pipeline_metrics(&pipeline_metrics);
                    display_pipeline_metrics(&converted_metrics)?;
                    if let Some(export_file) = &export {
                        let output = serde_json::to_string_pretty(&converted_metrics)?;
                        std::fs::write(export_file, output)?;
                        println!("Metrics exported to: {}", export_file.display());
                    }
                }
            }
        }

        MonitorCommand::Alerts { command } => {
            handle_alert_command(command, system).await?;
        }

        MonitorCommand::Dashboard { port, bind, open } => {
            let port = port.unwrap_or(8080);
            let bind_addr = bind.as_deref().unwrap_or("127.0.0.1");

            println!(
                "Starting pipeline dashboard at http://{}:{}",
                bind_addr, port
            );
            system.start_dashboard(bind_addr, port, open).await?;
        }
    }

    Ok(())
}

async fn handle_alert_command(command: AlertCommand, system: &DataPipelineSystem) -> Result<()> {
    match command {
        AlertCommand::List {
            severity,
            status,
            limit,
        } => {
            let alerts = system
                .list_alerts(severity.as_deref(), status.as_deref(), limit)
                .await?;

            println!(
                "{:<36} {:<30} {:<10} {:<10} {:<20}",
                "ID", "NAME", "SEVERITY", "STATUS", "CREATED"
            );
            println!("{}", "-".repeat(116));
            for alert in alerts {
                println!(
                    "{:<36} {:<30} {:<10} {:<10} {:<20}",
                    alert.id,
                    truncate_string(&alert.name, 30),
                    format!("{:?}", alert.severity),
                    format!("{:?}", alert.status),
                    alert.created_at.format("%Y-%m-%d %H:%M:%S")
                );
            }
        }

        AlertCommand::Create {
            name,
            condition,
            severity,
            channels,
            config,
        } => {
            println!("Creating alert rule: {}", name);

            let alert_config = if let Some(config_file) = config {
                let config_content = std::fs::read_to_string(config_file)?;
                serde_json::from_str(&config_content)?
            } else {
                let default_config =
                    create_default_alert_config(&name, &condition, &severity, &channels)?;
                // Convert AlertingConfig to HashMap<String, Value>
                let config_value = serde_json::to_value(default_config)?;
                serde_json::from_value(config_value)?
            };

            let alert_id = system.create_alert_rule(alert_config).await?;
            println!("✓ Alert rule created with ID: {}", alert_id);
        }

        AlertCommand::Update {
            name,
            config,
            enabled,
        } => {
            println!("Updating alert rule: {}", name);

            let mut updates = HashMap::new();

            if let Some(config_file) = config {
                let config_content = std::fs::read_to_string(config_file)?;
                let alert_config: Value = serde_json::from_str(&config_content)?;
                updates.insert("config".to_string(), alert_config);
            }

            if let Some(enabled_flag) = enabled {
                updates.insert("enabled".to_string(), Value::Bool(enabled_flag));
            }

            system.update_alert_rule(&name, updates).await?;
            println!("✓ Alert rule updated");
        }

        AlertCommand::Delete { name, confirm } => {
            if !confirm {
                println!(
                    "This will delete the alert rule '{}'. Use --confirm to proceed.",
                    name
                );
                return Ok(());
            }

            println!("Deleting alert rule: {}", name);
            system.delete_alert_rule(&name).await?;
            println!("✓ Alert rule deleted");
        }
    }

    Ok(())
}

async fn handle_quality_command(
    command: QualityCommand,
    system: &DataPipelineSystem,
) -> Result<()> {
    match command {
        QualityCommand::Check {
            pipeline,
            execution_id,
            stage,
            rules,
            format,
        } => {
            println!("Running data quality checks for pipeline: {}", pipeline);

            let quality_report = system
                .run_quality_checks(
                    &pipeline,
                    vec![],           // check_types - empty for now
                    stage.as_deref(), // severity parameter
                    rules,
                )
                .await?;

            match format.as_deref().unwrap_or("table") {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&quality_report)?);
                }
                "yaml" => {
                    println!("{}", serde_yaml::to_string(&quality_report)?);
                }
                "table" | _ => {
                    display_quality_report(&quality_report)?;
                }
            }
        }

        QualityCommand::Report {
            pipeline,
            range,
            report_type,
            export,
        } => {
            let time_range = range.as_deref().unwrap_or("7d");
            let report_type_enum = report_type.as_deref().unwrap_or("summary");

            let quality_report = system
                .generate_quality_report(&pipeline, Some(time_range), Some(report_type_enum))
                .await?;

            if let Some(export_file) = &export {
                let output = serde_json::to_string_pretty(&quality_report)?;
                std::fs::write(export_file, output)?;
                println!("Quality report exported to: {}", export_file.display());
            } else {
                display_quality_report(&quality_report)?;
            }
        }

        QualityCommand::Rules { command } => {
            handle_rules_command(command, system).await?;
        }

        QualityCommand::Anomaly { command } => {
            handle_anomaly_command(command, system).await?;
        }
    }

    Ok(())
}

async fn handle_rules_command(command: RulesCommand, system: &DataPipelineSystem) -> Result<()> {
    match command {
        RulesCommand::List {
            pipeline,
            rule_type,
            detailed,
        } => {
            let rules = if let Some(pipeline_id) = pipeline {
                system.get_validation_rules(&pipeline_id).await?
            } else {
                system.list_all_validation_rules().await?
            };

            let mut filtered_rules = rules;

            if let Some(type_filter) = rule_type {
                let type_enum = parse_validation_rule_type(&type_filter)?;
                filtered_rules.retain(|rule| rule.rule_type == type_enum);
            }

            if detailed {
                println!(
                    "{:<30} {:<15} {:<50} {:<10} {:<10}",
                    "NAME", "TYPE", "EXPRESSION", "SEVERITY", "ENABLED"
                );
                println!("{}", "-".repeat(125));
                for rule in filtered_rules {
                    println!(
                        "{:<30} {:<15} {:<50} {:<10} {:<10}",
                        truncate_string(&rule.name, 30),
                        format!("{:?}", rule.rule_type),
                        truncate_string(&rule.expression, 50),
                        format!("{:?}", rule.severity),
                        rule.enabled
                    );
                }
            } else {
                println!("{:<30} {:<15} {:<10}", "NAME", "TYPE", "ENABLED");
                println!("{}", "-".repeat(55));
                for rule in filtered_rules {
                    println!(
                        "{:<30} {:<15} {:<10}",
                        truncate_string(&rule.name, 30),
                        format!("{:?}", rule.rule_type),
                        rule.enabled
                    );
                }
            }
        }

        RulesCommand::Add {
            pipeline,
            name,
            rule_type,
            expression,
            severity,
            config,
        } => {
            println!(
                "Adding validation rule '{}' to pipeline '{}'",
                name, pipeline
            );

            let rule_config = if let Some(config_file) = config {
                let config_content = std::fs::read_to_string(config_file)?;
                serde_json::from_str(&config_content)?
            } else {
                create_default_validation_rule(&name, &rule_type, &expression, severity.as_deref())?
            };

            system.add_validation_rule(&pipeline, rule_config).await?;
            println!("✓ Validation rule added successfully");
        }

        RulesCommand::Update {
            pipeline,
            rule,
            config,
            enabled,
        } => {
            println!(
                "Updating validation rule '{}' in pipeline '{}'",
                rule, pipeline
            );

            let mut updates = HashMap::new();

            if let Some(config_file) = config {
                let config_content = std::fs::read_to_string(config_file)?;
                let rule_config: Value = serde_json::from_str(&config_content)?;
                updates.insert("config".to_string(), rule_config);
            }

            if let Some(enabled_flag) = enabled {
                updates.insert("enabled".to_string(), Value::Bool(enabled_flag));
            }

            system
                .update_validation_rule(&pipeline, &rule, updates)
                .await?;
            println!("✓ Validation rule updated successfully");
        }

        RulesCommand::Remove {
            pipeline,
            rule,
            confirm,
        } => {
            if !confirm {
                println!("This will remove the validation rule '{}' from pipeline '{}'. Use --confirm to proceed.", rule, pipeline);
                return Ok(());
            }

            println!(
                "Removing validation rule '{}' from pipeline '{}'",
                rule, pipeline
            );
            system.remove_validation_rule(&pipeline, &rule).await?;
            println!("✓ Validation rule removed successfully");
        }
    }

    Ok(())
}

async fn handle_anomaly_command(
    command: AnomalyCommand,
    system: &DataPipelineSystem,
) -> Result<()> {
    match command {
        AnomalyCommand::Detect {
            pipeline,
            method,
            range,
            sensitivity,
        } => {
            let detection_method = method.as_deref().unwrap_or("statistical");
            let time_range = range.as_deref().unwrap_or("24h");
            let sensitivity_level = sensitivity.unwrap_or(0.95);

            println!(
                "Detecting anomalies in pipeline '{}' using {} method",
                pipeline, detection_method
            );

            let anomalies = system
                .detect_anomalies(&pipeline, detection_method, time_range, sensitivity_level)
                .await?;

            if anomalies.is_empty() {
                println!("✓ No anomalies detected");
            } else {
                println!("Found {} anomalies:", anomalies.len());
                println!(
                    "{:<20} {:<15} {:<30} {:<10}",
                    "TIMESTAMP", "TYPE", "DESCRIPTION", "SEVERITY"
                );
                println!("{}", "-".repeat(75));
                for (_i, anomaly) in anomalies.iter().enumerate() {
                    println!(
                        "{:<20} {:<15} {:<30} {:<10}",
                        anomaly.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        format!("{:?}", anomaly.anomaly_type),
                        truncate_string(&anomaly.description, 30),
                        format!("{:?}", anomaly.severity)
                    );
                }
            }
        }

        AnomalyCommand::Configure {
            pipeline,
            config,
            enabled,
        } => {
            println!("Configuring anomaly detection for pipeline: {}", pipeline);

            let config_content = std::fs::read_to_string(config)?;
            let anomaly_config: AnomalyDetectionConfig = serde_json::from_str(&config_content)?;

            system
                .configure_anomaly_detection(&pipeline, anomaly_config, enabled.unwrap_or(true))
                .await?;
            println!("✓ Anomaly detection configured successfully");
        }

        AnomalyCommand::History {
            pipeline,
            range,
            severity,
            limit,
        } => {
            let time_range = range.as_deref().unwrap_or("7d");

            let anomaly_history = system
                .get_anomaly_history(&pipeline, time_range, severity.as_deref(), limit)
                .await?;

            println!(
                "{:<20} {:<15} {:<30} {:<10} {:<20}",
                "TIMESTAMP", "TYPE", "DESCRIPTION", "SEVERITY", "STATUS"
            );
            println!("{}", "-".repeat(95));
            for anomaly in anomaly_history {
                println!(
                    "{:<20} {:<15} {:<30} {:<10} {:<20}",
                    anomaly.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    format!("{:?}", anomaly.anomaly_type),
                    truncate_string(&anomaly.description, 30),
                    format!("{:?}", anomaly.severity),
                    format!("{:?}", anomaly.status)
                );
            }
        }
    }

    Ok(())
}

async fn handle_schedule_command(
    command: ScheduleCommand,
    system: &DataPipelineSystem,
) -> Result<()> {
    match command {
        ScheduleCommand::Show {
            pipeline,
            next,
            format,
        } => {
            let schedule = system.get_pipeline_schedule(&pipeline).await?;

            match format.as_deref().unwrap_or("table") {
                "json" => {
                    let mut output = serde_json::to_value(&schedule)?;

                    if let Some(next_count) = next {
                        let next_executions = system.get_next_executions(next_count).await?;
                        output["next_executions"] = serde_json::to_value(next_executions)?;
                    }

                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                "yaml" => {
                    let mut output = serde_json::to_value(&schedule)?;

                    if let Some(next_count) = next {
                        let next_executions = system.get_next_executions(next_count).await?;
                        output["next_executions"] = serde_json::to_value(next_executions)?;
                    }

                    println!("{}", serde_yaml::to_string(&output)?);
                }
                "table" | _ => {
                    println!("Schedule for pipeline: {}", pipeline);
                    println!("=======================");
                    if let Some(cron) = &schedule.cron_expression {
                        println!("Cron: {}", cron);
                    }
                    if let Some(interval) = schedule.interval_seconds {
                        println!("Interval: {}s", interval);
                    }
                    println!("Enabled: {}", schedule.enabled);
                    println!("Timezone: {}", schedule.timezone);

                    if let Some(next_count) = next {
                        let next_executions = system.get_next_executions(next_count).await?;
                        println!("\nNext {} executions:", next_count);
                        for (i, execution_time) in next_executions.iter().enumerate() {
                            println!(
                                "{}. {}",
                                i + 1,
                                execution_time.1.format("%Y-%m-%d %H:%M:%S")
                            );
                        }
                    }
                }
            }
        }

        ScheduleCommand::Update {
            pipeline,
            cron,
            interval,
            enabled,
            timezone,
        } => {
            println!("Updating schedule for pipeline: {}", pipeline);

            let mut updates = HashMap::new();

            if let Some(cron_expr) = cron {
                updates.insert("cron_expression".to_string(), Value::String(cron_expr));
            }

            if let Some(interval_secs) = interval {
                updates.insert(
                    "interval_seconds".to_string(),
                    Value::Number(interval_secs.into()),
                );
            }

            if let Some(enabled_flag) = enabled {
                updates.insert("enabled".to_string(), Value::Bool(enabled_flag));
            }

            if let Some(tz) = timezone {
                updates.insert("timezone".to_string(), Value::String(tz));
            }

            system.update_pipeline_schedule(&pipeline, updates).await?;
            println!("✓ Schedule updated successfully");
        }

        ScheduleCommand::Trigger {
            pipeline,
            params,
            skip_if_running,
        } => {
            println!("Triggering scheduled execution for pipeline: {}", pipeline);

            let execution_params = if let Some(params_str) = params {
                let params_map: HashMap<String, Value> = serde_json::from_str(&params_str)?;
                Some(serde_json::to_value(params_map)?)
            } else {
                None
            };

            let execution_id = system
                .trigger_scheduled_execution(&pipeline, execution_params, skip_if_running)
                .await?;
            println!("✓ Execution triggered with ID: {}", execution_id);
        }

        ScheduleCommand::List {
            show_disabled,
            show_next,
            format,
        } => {
            let scheduled_pipelines = system.list_scheduled_pipelines(show_disabled).await?;

            match format.as_deref().unwrap_or("table") {
                "json" => {
                    let output: Value = scheduled_pipelines
                        .into_iter()
                        .map(|(pipeline_id, schedule)| {
                            let mut schedule_json = json!({
                                "pipeline_id": pipeline_id,
                                "enabled": schedule.enabled,
                                "cron_expression": schedule.cron_expression,
                                "interval_seconds": schedule.interval_seconds,
                                "timezone": schedule.timezone,
                            });

                            if show_next {
                                // This would need to be implemented in the system
                                schedule_json["next_execution"] = Value::Null;
                            }

                            schedule_json
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                "yaml" => {
                    let output: Value = scheduled_pipelines
                        .into_iter()
                        .map(|(pipeline_id, schedule)| {
                            let mut schedule_json = json!({
                                "pipeline_id": pipeline_id,
                                "enabled": schedule.enabled,
                                "cron_expression": schedule.cron_expression,
                                "interval_seconds": schedule.interval_seconds,
                                "timezone": schedule.timezone,
                            });

                            if show_next {
                                schedule_json["next_execution"] = Value::Null;
                            }

                            schedule_json
                        })
                        .collect();
                    println!("{}", serde_yaml::to_string(&output)?);
                }
                "table" | _ => {
                    if show_next {
                        println!(
                            "{:<36} {:<10} {:<30} {:<20} {:<20}",
                            "PIPELINE_ID", "ENABLED", "SCHEDULE", "TIMEZONE", "NEXT_EXECUTION"
                        );
                        println!("{}", "-".repeat(116));
                    } else {
                        println!(
                            "{:<36} {:<10} {:<30} {:<20}",
                            "PIPELINE_ID", "ENABLED", "SCHEDULE", "TIMEZONE"
                        );
                        println!("{}", "-".repeat(96));
                    }

                    for (pipeline_id, schedule) in scheduled_pipelines {
                        let schedule_str = if let Some(cron) = &schedule.cron_expression {
                            format!("cron: {}", cron)
                        } else if let Some(interval) = schedule.interval_seconds {
                            format!("interval: {}s", interval)
                        } else {
                            "none".to_string()
                        };

                        let timezone = &schedule.timezone;

                        if show_next {
                            println!(
                                "{:<36} {:<10} {:<30} {:<20} {:<20}",
                                pipeline_id,
                                schedule.enabled,
                                truncate_string(&schedule_str, 30),
                                truncate_string(&timezone, 20),
                                "TBD"
                            );
                        } else {
                            println!(
                                "{:<36} {:<10} {:<30} {:<20}",
                                pipeline_id,
                                schedule.enabled,
                                truncate_string(&schedule_str, 30),
                                truncate_string(&timezone, 20)
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_template_command(
    command: TemplateCommand,
    system: &DataPipelineSystem,
) -> Result<()> {
    match command {
        TemplateCommand::List {
            category,
            detailed,
            format,
        } => {
            let templates = system.list_templates(category.as_deref()).await?;

            match format.as_deref().unwrap_or("table") {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&templates)?);
                }
                "yaml" => {
                    println!("{}", serde_yaml::to_string(&templates)?);
                }
                "table" | _ => {
                    if detailed {
                        println!(
                            "{:<30} {:<15} {:<50} {:<20}",
                            "NAME", "CATEGORY", "DESCRIPTION", "TAGS"
                        );
                        println!("{}", "-".repeat(115));
                        for template in templates {
                            let tags = template.tags.join(", ");
                            println!(
                                "{:<30} {:<15} {:<50} {:<20}",
                                truncate_string(&template.name, 30),
                                truncate_string(&template.category.unwrap_or_default(), 15),
                                truncate_string(&template.description.unwrap_or_default(), 50),
                                truncate_string(&tags, 20)
                            );
                        }
                    } else {
                        println!("{:<30} {:<15} {:<50}", "NAME", "CATEGORY", "DESCRIPTION");
                        println!("{}", "-".repeat(95));
                        for template in templates {
                            println!(
                                "{:<30} {:<15} {:<50}",
                                truncate_string(&template.name, 30),
                                truncate_string(&template.category.unwrap_or_default(), 15),
                                truncate_string(&template.description.unwrap_or_default(), 50)
                            );
                        }
                    }
                }
            }
        }

        TemplateCommand::Show {
            template,
            config,
            format,
        } => {
            let template_data = system.get_template(&template).await?;

            match format.as_deref().unwrap_or("table") {
                "json" => {
                    let mut output = serde_json::to_value(&template_data)?;

                    if config {
                        let template_config = system.get_template_config(&template).await?;
                        output["config"] = serde_json::to_value(template_config)?;
                    }

                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                "yaml" => {
                    let mut output = serde_json::to_value(&template_data)?;

                    if config {
                        let template_config = system.get_template_config(&template).await?;
                        output["config"] = serde_json::to_value(template_config)?;
                    }

                    println!("{}", serde_yaml::to_string(&output)?);
                }
                "table" | _ => {
                    println!("Template Details");
                    println!("================");
                    println!("Name: {}", template_data.name);
                    if let Some(category) = &template_data.category {
                        println!("Category: {}", category);
                    }
                    if let Some(description) = &template_data.description {
                        println!("Description: {}", description);
                    }
                    if !template_data.tags.is_empty() {
                        println!("Tags: {}", template_data.tags.join(", "));
                    }
                    println!("Created: {}", template_data.created_at);

                    if config {
                        let template_config = system.get_template_config(&template).await?;
                        println!("\nConfiguration:");
                        println!("=============");
                        println!("{}", serde_yaml::to_string(&template_config)?);
                    }
                }
            }
        }

        TemplateCommand::Create {
            pipeline,
            name,
            description,
            category,
            tags,
        } => {
            println!("Creating template '{}' from pipeline '{}'", name, pipeline);

            let template_id = system
                .create_template(
                    &pipeline,
                    &name,
                    description.as_deref(),
                    category.as_deref(),
                    tags,
                )
                .await?;
            println!("✓ Template created with ID: {}", template_id);
        }

        TemplateCommand::Update {
            template,
            config,
            description,
            tags,
        } => {
            println!("Updating template: {}", template);

            let mut updates = HashMap::new();

            if let Some(config_file) = config {
                let config_content = std::fs::read_to_string(config_file)?;
                let template_config: Value = serde_json::from_str(&config_content)?;
                updates.insert("config".to_string(), template_config);
            }

            if let Some(desc) = description {
                updates.insert("description".to_string(), Value::String(desc));
            }

            if !tags.is_empty() {
                updates.insert("tags".to_string(), serde_json::to_value(tags)?);
            }

            system.update_template(&template, updates).await?;
            println!("✓ Template updated successfully");
        }

        TemplateCommand::Delete { template, confirm } => {
            if !confirm {
                println!(
                    "This will delete the template '{}'. Use --confirm to proceed.",
                    template
                );
                return Ok(());
            }

            println!("Deleting template: {}", template);
            system.delete_template(&template).await?;
            println!("✓ Template deleted successfully");
        }

        TemplateCommand::Apply {
            template,
            name,
            params,
            config,
        } => {
            println!(
                "Applying template '{}' to create pipeline '{}'",
                template, name
            );

            let template_params = if let Some(params_str) = params {
                let params_map: HashMap<String, Value> = serde_json::from_str(&params_str)?;
                Some(serde_json::to_value(params_map)?)
            } else {
                None
            };

            let config_overrides = if let Some(config_file) = config {
                let config_content = std::fs::read_to_string(config_file)?;
                let config_map: HashMap<String, Value> = serde_json::from_str(&config_content)?;
                Some(serde_json::to_value(config_map)?)
            } else {
                None
            };

            let pipeline_id = system
                .apply_template(&template, &name, template_params, config_overrides)
                .await?;
            println!("✓ Pipeline created from template with ID: {}", pipeline_id);
        }
    }

    Ok(())
}

// Helper functions

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn parse_pipeline_status(status: &str) -> Result<PipelineStatus> {
    match status.to_lowercase().as_str() {
        "active" => Ok(PipelineStatus::Active),
        "inactive" => Ok(PipelineStatus::Inactive),
        "error" => Ok(PipelineStatus::Error),
        "archived" => Ok(PipelineStatus::Archived),
        _ => Err(anyhow::anyhow!("Invalid pipeline status: {}", status)),
    }
}

fn parse_pipeline_type(pipeline_type: &str) -> Result<PipelineType> {
    match pipeline_type.to_lowercase().as_str() {
        "batch" => Ok(PipelineType::Batch),
        "streaming" => Ok(PipelineType::Streaming),
        "hybrid" => Ok(PipelineType::Hybrid),
        _ => Err(anyhow::anyhow!("Invalid pipeline type: {}", pipeline_type)),
    }
}

fn parse_execution_status(status: &str) -> Result<ExecutionStatus> {
    match status.to_lowercase().as_str() {
        "pending" => Ok(ExecutionStatus::Pending),
        "running" => Ok(ExecutionStatus::Running),
        "success" => Ok(ExecutionStatus::Success),
        "failed" => Ok(ExecutionStatus::Failed),
        "cancelled" => Ok(ExecutionStatus::Cancelled),
        "paused" => Ok(ExecutionStatus::Paused),
        _ => Err(anyhow::anyhow!("Invalid execution status: {}", status)),
    }
}

fn parse_validation_rule_type(rule_type: &str) -> Result<ValidationRuleType> {
    match rule_type.to_lowercase().as_str() {
        "schema" => Ok(ValidationRuleType::Schema),
        "range" => Ok(ValidationRuleType::Range),
        "pattern" => Ok(ValidationRuleType::Pattern),
        "uniqueness" => Ok(ValidationRuleType::Uniqueness),
        "completeness" => Ok(ValidationRuleType::Completeness),
        "consistency" => Ok(ValidationRuleType::Consistency),
        "custom" => Ok(ValidationRuleType::Custom),
        _ => Err(anyhow::anyhow!(
            "Invalid validation rule type: {}",
            rule_type
        )),
    }
}

fn get_template_config(template_name: &str) -> Result<DataPipelineConfig> {
    // This would load a template configuration
    // For now, return a default configuration
    create_default_pipeline_config(PipelineType::Batch)
}

fn create_default_pipeline_config(pipeline_type: PipelineType) -> Result<DataPipelineConfig> {
    let mut config = DataPipelineConfig::default();
    config.pipeline_type = pipeline_type;
    config.name = "new-pipeline".to_string();
    config.description = Some("A new data pipeline".to_string());
    config.tags = Vec::new();
    Ok(config)
}

fn create_default_stage_config(stage_type: &str) -> Result<Option<HashMap<String, Value>>> {
    let mut config = HashMap::new();

    match stage_type.to_lowercase().as_str() {
        "extract" => {
            config.insert("type".to_string(), Value::String("extract".to_string()));
            config.insert("source_type".to_string(), Value::String("file".to_string()));
            config.insert("source_path".to_string(), Value::String("/data/input".to_string()));
            config.insert("format".to_string(), Value::String("json".to_string()));
            config.insert("batch_size".to_string(), Value::Number(serde_json::Number::from(1000)));
            config.insert("parallel_workers".to_string(), Value::Number(serde_json::Number::from(4)));
            config.insert("retry_attempts".to_string(), Value::Number(serde_json::Number::from(3)));
            config.insert("timeout_seconds".to_string(), Value::Number(serde_json::Number::from(300)));
        },
        "transform" => {
            config.insert("type".to_string(), Value::String("transform".to_string()));
            config.insert("transformation_type".to_string(), Value::String("map".to_string()));
            config.insert("operations".to_string(), Value::Array(vec![
                Value::Object({
                    let mut op = serde_json::Map::new();
                    op.insert("name".to_string(), Value::String("normalize".to_string()));
                    op.insert("enabled".to_string(), Value::Bool(true));
                    op
                })
            ]));
            config.insert("error_handling".to_string(), Value::String("skip".to_string()));
            config.insert("validation_enabled".to_string(), Value::Bool(true));
            config.insert("memory_limit_mb".to_string(), Value::Number(serde_json::Number::from(512)));
        },
        "load" => {
            config.insert("type".to_string(), Value::String("load".to_string()));
            config.insert("destination_type".to_string(), Value::String("database".to_string()));
            config.insert("destination_path".to_string(), Value::String("/data/output".to_string()));
            config.insert("table_name".to_string(), Value::String("processed_data".to_string()));
            config.insert("write_mode".to_string(), Value::String("append".to_string()));
            config.insert("batch_commit_size".to_string(), Value::Number(serde_json::Number::from(1000)));
            config.insert("create_indexes".to_string(), Value::Bool(true));
            config.insert("backup_enabled".to_string(), Value::Bool(true));
        },
        "validate" => {
            config.insert("type".to_string(), Value::String("validate".to_string()));
            config.insert("validation_rules".to_string(), Value::Array(vec![
                Value::Object({
                    let mut rule = serde_json::Map::new();
                    rule.insert("field".to_string(), Value::String("id".to_string()));
                    rule.insert("rule_type".to_string(), Value::String("required".to_string()));
                    rule.insert("enabled".to_string(), Value::Bool(true));
                    rule
                })
            ]));
            config.insert("strict_mode".to_string(), Value::Bool(false));
            config.insert("error_threshold_percent".to_string(), Value::Number(serde_json::Number::from(5)));
            config.insert("report_validation_errors".to_string(), Value::Bool(true));
        },
        "filter" => {
            config.insert("type".to_string(), Value::String("filter".to_string()));
            config.insert("filter_conditions".to_string(), Value::Array(vec![
                Value::Object({
                    let mut condition = serde_json::Map::new();
                    condition.insert("field".to_string(), Value::String("status".to_string()));
                    condition.insert("operator".to_string(), Value::String("equals".to_string()));
                    condition.insert("value".to_string(), Value::String("active".to_string()));
                    condition
                })
            ]));
            config.insert("filter_mode".to_string(), Value::String("include".to_string()));
            config.insert("case_sensitive".to_string(), Value::Bool(false));
        },
        "aggregate" => {
            config.insert("type".to_string(), Value::String("aggregate".to_string()));
            config.insert("group_by_fields".to_string(), Value::Array(vec![
                Value::String("category".to_string())
            ]));
            config.insert("aggregations".to_string(), Value::Array(vec![
                Value::Object({
                    let mut agg = serde_json::Map::new();
                    agg.insert("field".to_string(), Value::String("amount".to_string()));
                    agg.insert("function".to_string(), Value::String("sum".to_string()));
                    agg.insert("alias".to_string(), Value::String("total_amount".to_string()));
                    agg
                })
            ]));
            config.insert("window_size".to_string(), Value::Number(serde_json::Number::from(3600)));
            config.insert("emit_partial_results".to_string(), Value::Bool(false));
        },
        "enrich" => {
            config.insert("type".to_string(), Value::String("enrich".to_string()));
            config.insert("enrichment_source".to_string(), Value::String("lookup_table".to_string()));
            config.insert("lookup_key".to_string(), Value::String("id".to_string()));
            config.insert("enrichment_fields".to_string(), Value::Array(vec![
                Value::String("description".to_string()),
                Value::String("metadata".to_string())
            ]));
            config.insert("cache_enabled".to_string(), Value::Bool(true));
            config.insert("cache_ttl_seconds".to_string(), Value::Number(serde_json::Number::from(3600)));
            config.insert("missing_value_strategy".to_string(), Value::String("null".to_string()));
        },
        "split" => {
            config.insert("type".to_string(), Value::String("split".to_string()));
            config.insert("split_strategy".to_string(), Value::String("round_robin".to_string()));
            config.insert("output_branches".to_string(), Value::Number(serde_json::Number::from(2)));
            config.insert("split_condition".to_string(), Value::String("${record.type}".to_string()));
            config.insert("preserve_order".to_string(), Value::Bool(true));
        },
        "deduplicate" => {
            config.insert("type".to_string(), Value::String("deduplicate".to_string()));
            config.insert("dedup_keys".to_string(), Value::Array(vec![
                Value::String("id".to_string())
            ]));
            config.insert("dedup_strategy".to_string(), Value::String("first".to_string()));
            config.insert("window_size".to_string(), Value::Number(serde_json::Number::from(10000)));
            config.insert("memory_efficient".to_string(), Value::Bool(true));
        },
        _ => {
            // Default generic stage configuration
            config.insert("type".to_string(), Value::String(stage_type.to_string()));
            config.insert("enabled".to_string(), Value::Bool(true));
            config.insert("description".to_string(), Value::String(format!("Default {} stage", stage_type)));
            config.insert("timeout_seconds".to_string(), Value::Number(serde_json::Number::from(300)));
            config.insert("retry_attempts".to_string(), Value::Number(serde_json::Number::from(3)));
            config.insert("parallel_processing".to_string(), Value::Bool(false));
        }
    }

    // Add common configuration for all stages
    config.insert("stage_name".to_string(), Value::String(stage_type.to_string()));
    config.insert("created_at".to_string(), Value::String(Utc::now().to_rfc3339()));
    config.insert("version".to_string(), Value::String("1.0".to_string()));
    config.insert("metadata".to_string(), Value::Object({
        let mut meta = serde_json::Map::new();
        meta.insert("generated".to_string(), Value::Bool(true));
        meta.insert("stage_type".to_string(), Value::String(stage_type.to_string()));
        meta
    }));

    Ok(Some(config))
}

fn validate_pipeline_config(config: &DataPipelineConfig) -> Result<()> {
    // Validate pipeline name
    if config.name.trim().is_empty() {
        return Err(anyhow::anyhow!("Pipeline name cannot be empty"));
    }

    // Validate pipeline name format (alphanumeric, hyphens, underscores only)
    if !config.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(anyhow::anyhow!(
            "Pipeline name '{}' contains invalid characters. Only alphanumeric, hyphens, and underscores are allowed",
            config.name
        ));
    }

    // Validate pipeline name length
    if config.name.len() > 64 {
        return Err(anyhow::anyhow!(
            "Pipeline name '{}' is too long. Maximum length is 64 characters",
            config.name
        ));
    }

    // Validate description length if provided
    if let Some(ref description) = config.description {
        if description.len() > 500 {
            return Err(anyhow::anyhow!(
                "Pipeline description is too long. Maximum length is 500 characters"
            ));
        }
    }

    // Validate pipeline type
    match config.pipeline_type {
        PipelineType::Batch | PipelineType::Streaming | PipelineType::Hybrid => {},
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid pipeline type: {:?}. Must be Batch, Streaming, or Hybrid",
                config.pipeline_type
            ));
        }
    }

    // Validate tags
    for tag in &config.tags {
        if tag.trim().is_empty() {
            return Err(anyhow::anyhow!("Tags cannot be empty"));
        }
        if tag.len() > 32 {
            return Err(anyhow::anyhow!(
                "Tag '{}' is too long. Maximum length is 32 characters",
                tag
            ));
        }
    }

    // Validate stages if present
    if !config.stages.is_empty() {
        // Check for duplicate stage names
        let mut stage_names = std::collections::HashSet::new();
        for stage in &config.stages {
            if !stage_names.insert(&stage.name) {
                return Err(anyhow::anyhow!(
                    "Duplicate stage name: '{}'",
                    stage.name
                ));
            }

            // Validate individual stage (convert PipelineTask to basic validation)
            validate_pipeline_task_config(stage)?;
        }

        // Validate stage dependencies
        for stage in &config.stages {
            for dependency in &stage.dependencies {
                if !stage_names.contains(dependency) {
                    return Err(anyhow::anyhow!(
                        "Stage '{}' depends on non-existent stage '{}'",
                        stage.name,
                        dependency
                    ));
                }
            }
        }

        // Check for circular dependencies
        validate_no_circular_dependencies_tasks(&config.stages)?;
    }

    // Validate orchestration schedule for batch pipelines
    if config.pipeline_type == PipelineType::Batch {
        if let Some(ref cron_expr) = config.orchestration.scheduler.cron_expression {
            validate_cron_expression(cron_expr)?;
        }
    }

    Ok(())
}

fn validate_stage_config(stage: &crate::data_pipeline::Stage) -> Result<()> {
    // Validate stage name
    if stage.name.trim().is_empty() {
        return Err(anyhow::anyhow!("Stage name cannot be empty"));
    }

    if !stage.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(anyhow::anyhow!(
            "Stage name '{}' contains invalid characters. Only alphanumeric, hyphens, and underscores are allowed",
            stage.name
        ));
    }

    if stage.name.len() > 64 {
        return Err(anyhow::anyhow!(
            "Stage name '{}' is too long. Maximum length is 64 characters",
            stage.name
        ));
    }

    // Validate stage type (stage_type is an enum, no need to check if empty)
    match stage.stage_type {
        crate::data_pipeline::StageType::Source => {},
        crate::data_pipeline::StageType::Transform => {},
        crate::data_pipeline::StageType::Filter => {},
        crate::data_pipeline::StageType::Aggregate => {},
        crate::data_pipeline::StageType::Join => {},
        crate::data_pipeline::StageType::Sink => {},
        crate::data_pipeline::StageType::Custom => {},
    }

    Ok(())
}

fn validate_no_circular_dependencies(stages: &[crate::data_pipeline::Stage]) -> Result<()> {
    use std::collections::{HashMap, HashSet};

    let mut graph: HashMap<&String, Vec<&String>> = HashMap::new();

    // Build dependency graph
    for stage in stages {
        let deps: Vec<&String> = stage.dependencies.iter().collect();
        graph.insert(&stage.name, deps);
    }

    // Check for cycles using DFS
    fn has_cycle(
        node: &String,
        graph: &HashMap<&String, Vec<&String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.clone());
        rec_stack.insert(node.clone());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(*neighbor) {
                    if has_cycle(neighbor, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(*neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    let mut visited = HashSet::new();
    for stage in stages {
        if !visited.contains(&stage.name) {
            let mut rec_stack = HashSet::new();
            if has_cycle(&stage.name, &graph, &mut visited, &mut rec_stack) {
                return Err(anyhow::anyhow!(
                    "Circular dependency detected in pipeline stages"
                ));
            }
        }
    }

    Ok(())
}

fn validate_cron_expression(cron_expr: &str) -> Result<()> {
    // Basic cron validation - split into parts and check format
    let parts: Vec<&str> = cron_expr.trim().split_whitespace().collect();

    if parts.len() != 5 && parts.len() != 6 {
        return Err(anyhow::anyhow!(
            "Invalid cron expression '{}'. Must have 5 or 6 fields",
            cron_expr
        ));
    }

    // Basic field validation
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid cron expression '{}'. Field {} is empty",
                cron_expr, i + 1
            ));
        }

        // Check for valid characters (numbers, *, -, /, ,)
        if !part.chars().all(|c| c.is_ascii_digit() || "*/,-".contains(c)) {
            return Err(anyhow::anyhow!(
                "Invalid cron expression '{}'. Field {} contains invalid characters",
                cron_expr, i + 1
            ));
        }
    }

    Ok(())
}

fn validate_pipeline_config_detailed(
    config: &DataPipelineConfig,
    _level: &str,
    warnings: bool,
) -> Result<ValidationResult> {
    // This would perform detailed validation and return a comprehensive result
    validate_pipeline_config(config)?;

    Ok(ValidationResult {
        valid: true,
        errors: vec![],
        warnings: if warnings {
            vec!["Example warning".to_string()]
        } else {
            vec![]
        },
        info: vec!["Configuration is valid".to_string()],
    })
}

fn display_validation_result(result: &ValidationResult, format: Option<&str>) -> Result<()> {
    match format.unwrap_or("table") {
        "json" => {
            println!("{}", serde_json::to_string_pretty(result)?);
        }
        "yaml" => {
            println!("{}", serde_yaml::to_string(result)?);
        }
        "table" | _ => {
            if result.valid {
                println!("✓ Validation successful");
            } else {
                println!("✗ Validation failed");
            }

            if !result.errors.is_empty() {
                println!("\nErrors:");
                for error in &result.errors {
                    println!("  ✗ {}", error);
                }
            }

            if !result.warnings.is_empty() {
                println!("\nWarnings:");
                for warning in &result.warnings {
                    println!("  ⚠ {}", warning);
                }
            }

            if !result.info.is_empty() {
                println!("\nInfo:");
                for info in &result.info {
                    println!("  ℹ {}", info);
                }
            }
        }
    }

    Ok(())
}

fn display_pipeline_status(status: &DataPipeline, _metrics: bool, _alerts: bool) -> Result<()> {
    println!("Pipeline Status");
    println!("===============");
    // This would display comprehensive pipeline status information
    Ok(())
}

fn convert_execution_metrics_to_pipeline_metrics(
    exec_metrics: &[crate::data_pipeline::ExecutionMetrics],
) -> PipelineMetrics {
    let total_executions = exec_metrics.len() as u64;
    let total_records_processed: u64 = exec_metrics.iter().map(|m| m.records_processed).sum();
    let total_data_volume_bytes: u64 = exec_metrics.iter().map(|m| m.data_size_bytes).sum();
    let average_duration_secs = if !exec_metrics.is_empty() {
        exec_metrics
            .iter()
            .map(|m| m.duration_seconds as f64)
            .sum::<f64>()
            / total_executions as f64
    } else {
        0.0
    };
    let average_throughput_per_sec = if average_duration_secs > 0.0 {
        total_records_processed as f64 / average_duration_secs
    } else {
        0.0
    };

    PipelineMetrics {
        total_executions,
        successful_executions: total_executions, // Assume all are successful for now
        failed_executions: 0,
        average_duration_secs,
        total_records_processed,
        total_data_volume_bytes,
        average_throughput_per_sec,
    }
}

fn display_pipeline_metrics(metrics: &PipelineMetrics) -> Result<()> {
    println!("Pipeline Metrics");
    println!("================");
    println!("Total Executions: {}", metrics.total_executions);
    println!("Successful Executions: {}", metrics.successful_executions);
    println!("Failed Executions: {}", metrics.failed_executions);
    println!("Average Duration: {:.2}s", metrics.average_duration_secs);
    println!("Records Processed: {}", metrics.total_records_processed);
    println!("Data Volume: {} bytes", metrics.total_data_volume_bytes);
    println!(
        "Average Throughput: {:.2} records/sec",
        metrics.average_throughput_per_sec
    );

    Ok(())
}

fn display_quality_report(report: &crate::data_pipeline::DataQualityReport) -> Result<()> {
    println!("Data Quality Report");
    println!("===================");
    println!("Overall Score: {:.2}%", report.overall_score * 100.0);
    println!("Pipeline ID: {}", report.pipeline_id);

    if !report.checks.is_empty() {
        println!("\nQuality Checks:");
        println!("===============");
        for check in &report.checks {
            let status = if check.passed { "✓" } else { "✗" };
            println!("{} {}", status, check.name);
            if !check.details.is_empty() {
                println!("  {}", check.details);
            }
        }
    }

    Ok(())
}

fn create_default_alert_config(
    name: &str,
    condition: &str,
    severity: &str,
    channels: &[String],
) -> Result<crate::monitoring::AlertingConfig> {
    // This would create a default alert configuration
    Ok(crate::monitoring::AlertingConfig {
        enabled: true,
        webhooks: vec![],
        email: None,
        slack: None,
        cooldown_minutes: 5,
    })
}

fn create_default_validation_rule(
    name: &str,
    rule_type: &str,
    expression: &str,
    severity: Option<&str>,
) -> Result<ValidationRule> {
    let rule_type_enum = parse_validation_rule_type(rule_type)?;

    Ok(ValidationRule {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        rule_type: rule_type_enum,
        expression: expression.to_string(),
        config: serde_json::Value::Object(serde_json::Map::new()),
        severity: match severity {
            Some("high") => crate::data_pipeline::SeverityLevel::High,
            Some("medium") => crate::data_pipeline::SeverityLevel::Medium,
            Some("low") | _ => crate::data_pipeline::SeverityLevel::Low,
        },
        error_handling: crate::data_pipeline::ValidationErrorHandling::Warn,
        enabled: true,
        description: None,
        parameters: HashMap::new(),
    })
}

// Placeholder structs for compilation
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ValidationResult {
    valid: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
    info: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PipelineStatusInfo {
    // Status information fields
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ValidationSeverity;

impl ValidationSeverity {
    const HIGH: Self = Self;
    const MEDIUM: Self = Self;
    const LOW: Self = Self;
}

fn validate_pipeline_task_config(task: &crate::data_pipeline::PipelineTask) -> Result<()> {
    // Validate task name
    if task.name.trim().is_empty() {
        return Err(anyhow::anyhow!("Task name cannot be empty"));
    }

    if task.name.len() > 64 {
        return Err(anyhow::anyhow!(
            "Task name '{}' is too long. Maximum length is 64 characters",
            task.name
        ));
    }

    // Validate task ID
    if task.id.trim().is_empty() {
        return Err(anyhow::anyhow!("Task ID cannot be empty"));
    }

    Ok(())
}

fn validate_no_circular_dependencies_tasks(tasks: &[crate::data_pipeline::PipelineTask]) -> Result<()> {
    use std::collections::{HashMap, HashSet};

    let mut graph: HashMap<&String, Vec<&String>> = HashMap::new();

    // Build dependency graph
    for task in tasks {
        let deps: Vec<&String> = task.dependencies.iter().collect();
        graph.insert(&task.name, deps);
    }

    // Check for cycles using DFS
    fn has_cycle(
        node: &String,
        graph: &HashMap<&String, Vec<&String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.clone());
        rec_stack.insert(node.clone());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(*neighbor) {
                    if has_cycle(neighbor, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(*neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    for task in tasks {
        if !visited.contains(&task.name) {
            if has_cycle(&task.name, &graph, &mut visited, &mut rec_stack) {
                return Err(anyhow::anyhow!("Circular dependency detected in pipeline tasks"));
            }
        }
    }

    Ok(())
}
