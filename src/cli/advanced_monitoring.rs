use crate::{advanced_monitoring::AdvancedMonitoringSystem, config::Config};
use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tokio::signal;
use tracing::{error, info, warn};

#[derive(Debug, Args)]
pub struct AdvancedMonitoringArgs {
    #[command(subcommand)]
    pub command: AdvancedMonitoringCommand,
}

#[derive(Debug, Subcommand)]
pub enum AdvancedMonitoringCommand {
    #[command(about = "Start the advanced monitoring system")]
    Start {
        #[arg(long, help = "Override default config file")]
        config: Option<PathBuf>,

        #[arg(long, help = "Run in daemon mode")]
        daemon: bool,

        #[arg(long, help = "Override bind address")]
        bind_address: Option<String>,

        #[arg(long, help = "Override metrics port")]
        metrics_port: Option<u16>,

        #[arg(long, help = "Override dashboard port")]
        dashboard_port: Option<u16>,

        #[arg(long, help = "Enable debug logging")]
        debug: bool,
    },

    #[command(about = "Stop the monitoring system")]
    Stop {
        #[arg(long, help = "Force stop without graceful shutdown")]
        force: bool,

        #[arg(long, help = "Timeout for graceful shutdown in seconds")]
        timeout: Option<u64>,
    },

    #[command(about = "Get monitoring system status")]
    Status {
        #[arg(long, help = "Show detailed component status")]
        detailed: bool,

        #[arg(long, help = "Show recent alerts")]
        alerts: bool,

        #[arg(long, help = "Show metrics summary")]
        metrics: bool,

        #[arg(long, help = "Output format")]
        format: Option<OutputFormat>,
    },

    #[command(about = "Manage Prometheus configuration")]
    Prometheus {
        #[command(subcommand)]
        action: PrometheusAction,
    },

    #[command(about = "Manage Alertmanager configuration")]
    Alertmanager {
        #[command(subcommand)]
        action: AlertmanagerAction,
    },

    #[command(about = "Manage Grafana dashboards")]
    Dashboard {
        #[command(subcommand)]
        action: DashboardAction,
    },

    #[command(about = "Manage monitoring targets")]
    Targets {
        #[command(subcommand)]
        action: TargetAction,
    },

    #[command(about = "Manage alerts and alert rules")]
    Alerts {
        #[command(subcommand)]
        action: AlertAction,
    },

    #[command(about = "Export monitoring data")]
    Export {
        #[command(subcommand)]
        action: ExportAction,
    },

    #[command(about = "Health check and diagnostics")]
    Health {
        #[arg(long, help = "Run comprehensive health check")]
        comprehensive: bool,

        #[arg(long, help = "Check specific component")]
        component: Option<String>,

        #[arg(long, help = "Output format")]
        format: Option<OutputFormat>,
    },

    #[command(about = "Manage data retention and cleanup")]
    Retention {
        #[command(subcommand)]
        action: RetentionAction,
    },

    #[command(about = "Test monitoring configuration")]
    Test {
        #[command(subcommand)]
        action: TestAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum PrometheusAction {
    #[command(about = "Show Prometheus configuration")]
    Config {
        #[arg(long, help = "Show raw configuration")]
        raw: bool,

        #[arg(long, help = "Validate configuration")]
        validate: bool,
    },

    #[command(about = "Reload Prometheus configuration")]
    Reload {
        #[arg(long, help = "Force reload even if validation fails")]
        force: bool,
    },

    #[command(about = "Query Prometheus metrics")]
    Query {
        #[arg(help = "PromQL query")]
        query: String,

        #[arg(long, help = "Query evaluation time")]
        time: Option<String>,

        #[arg(long, help = "Query timeout in seconds")]
        timeout: Option<u64>,

        #[arg(long, help = "Output format")]
        format: Option<OutputFormat>,
    },

    #[command(about = "Query range data")]
    QueryRange {
        #[arg(help = "PromQL query")]
        query: String,

        #[arg(long, help = "Start time")]
        start: String,

        #[arg(long, help = "End time")]
        end: String,

        #[arg(long, help = "Step duration")]
        step: Option<String>,

        #[arg(long, help = "Output format")]
        format: Option<OutputFormat>,
    },

    #[command(about = "Manage recording rules")]
    Rules {
        #[command(subcommand)]
        action: RulesAction,
    },

    #[command(about = "Manage remote write configuration")]
    RemoteWrite {
        #[command(subcommand)]
        action: RemoteWriteAction,
    },

    #[command(about = "Show Prometheus targets")]
    Targets {
        #[arg(long, help = "Show only active targets")]
        active: bool,

        #[arg(long, help = "Show only unhealthy targets")]
        unhealthy: bool,

        #[arg(long, help = "Filter by job name")]
        job: Option<String>,
    },

    #[command(about = "Show Prometheus flags and build info")]
    Info,
}

#[derive(Debug, Subcommand)]
pub enum AlertmanagerAction {
    #[command(about = "Show Alertmanager configuration")]
    Config {
        #[arg(long, help = "Show raw configuration")]
        raw: bool,

        #[arg(long, help = "Validate configuration")]
        validate: bool,
    },

    #[command(about = "Reload Alertmanager configuration")]
    Reload {
        #[arg(long, help = "Force reload even if validation fails")]
        force: bool,
    },

    #[command(about = "Show active alerts")]
    Alerts {
        #[arg(long, help = "Filter by alert state")]
        state: Option<AlertState>,

        #[arg(long, help = "Filter by receiver")]
        receiver: Option<String>,

        #[arg(long, help = "Filter by label")]
        label: Vec<String>,

        #[arg(long, help = "Output format")]
        format: Option<OutputFormat>,
    },

    #[command(about = "Silence alerts")]
    Silence {
        #[command(subcommand)]
        action: SilenceAction,
    },

    #[command(about = "Test notification channels")]
    Test {
        #[arg(help = "Receiver name to test")]
        receiver: String,

        #[arg(long, help = "Test alert labels")]
        labels: Vec<String>,

        #[arg(long, help = "Test alert annotations")]
        annotations: Vec<String>,
    },

    #[command(about = "Show Alertmanager status")]
    Status,
}

#[derive(Debug, Subcommand)]
pub enum DashboardAction {
    #[command(about = "List available dashboards")]
    List {
        #[arg(long, help = "Filter by tag")]
        tag: Vec<String>,

        #[arg(long, help = "Show only imported dashboards")]
        imported: bool,

        #[arg(long, help = "Output format")]
        format: Option<OutputFormat>,
    },

    #[command(about = "Import dashboard")]
    Import {
        #[arg(help = "Dashboard file path or URL")]
        source: String,

        #[arg(long, help = "Dashboard name override")]
        name: Option<String>,

        #[arg(long, help = "Dashboard folder")]
        folder: Option<String>,

        #[arg(long, help = "Overwrite existing dashboard")]
        overwrite: bool,
    },

    #[command(about = "Export dashboard")]
    Export {
        #[arg(help = "Dashboard name or ID")]
        dashboard: String,

        #[arg(help = "Output file path")]
        output: PathBuf,

        #[arg(long, help = "Include template variables")]
        include_variables: bool,
    },

    #[command(about = "Update dashboard")]
    Update {
        #[arg(help = "Dashboard name or ID")]
        dashboard: String,

        #[arg(help = "Updated dashboard file")]
        file: PathBuf,

        #[arg(long, help = "Update message")]
        message: Option<String>,
    },

    #[command(about = "Delete dashboard")]
    Delete {
        #[arg(help = "Dashboard name or ID")]
        dashboard: String,

        #[arg(long, help = "Force deletion without confirmation")]
        force: bool,
    },

    #[command(about = "Create dashboard snapshot")]
    Snapshot {
        #[arg(help = "Dashboard name or ID")]
        dashboard: String,

        #[arg(long, help = "Snapshot name")]
        name: Option<String>,

        #[arg(long, help = "Snapshot expiration in hours")]
        expires: Option<u64>,
    },

    #[command(about = "Provision dashboards from directory")]
    Provision {
        #[arg(help = "Directory containing dashboard files")]
        directory: PathBuf,

        #[arg(long, help = "Watch directory for changes")]
        watch: bool,

        #[arg(long, help = "Dashboard folder")]
        folder: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum TargetAction {
    #[command(about = "List monitoring targets")]
    List {
        #[arg(long, help = "Filter by target type")]
        target_type: Option<String>,

        #[arg(long, help = "Show only healthy targets")]
        healthy: bool,

        #[arg(long, help = "Show only unhealthy targets")]
        unhealthy: bool,

        #[arg(long, help = "Output format")]
        format: Option<OutputFormat>,
    },

    #[command(about = "Add monitoring target")]
    Add {
        #[arg(help = "Target address")]
        address: String,

        #[arg(help = "Target type")]
        target_type: String,

        #[arg(long, help = "Target labels")]
        labels: Vec<String>,

        #[arg(long, help = "Scrape interval")]
        interval: Option<String>,

        #[arg(long, help = "Scrape timeout")]
        timeout: Option<String>,
    },

    #[command(about = "Remove monitoring target")]
    Remove {
        #[arg(help = "Target ID or address")]
        target: String,

        #[arg(long, help = "Force removal without confirmation")]
        force: bool,
    },

    #[command(about = "Update target configuration")]
    Update {
        #[arg(help = "Target ID or address")]
        target: String,

        #[arg(long, help = "New target labels")]
        labels: Vec<String>,

        #[arg(long, help = "New scrape interval")]
        interval: Option<String>,

        #[arg(long, help = "New scrape timeout")]
        timeout: Option<String>,
    },

    #[command(about = "Test target connectivity")]
    Test {
        #[arg(help = "Target address")]
        address: String,

        #[arg(long, help = "Connection timeout")]
        timeout: Option<u64>,
    },

    #[command(about = "Discover targets automatically")]
    Discover {
        #[arg(long, help = "Discovery method")]
        method: Option<DiscoveryMethod>,

        #[arg(long, help = "Discovery configuration")]
        config: Option<PathBuf>,

        #[arg(long, help = "Auto-add discovered targets")]
        auto_add: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum AlertAction {
    #[command(about = "List alert rules")]
    Rules {
        #[arg(long, help = "Filter by group")]
        group: Option<String>,

        #[arg(long, help = "Show only firing rules")]
        firing: bool,

        #[arg(long, help = "Output format")]
        format: Option<OutputFormat>,
    },

    #[command(about = "Add alert rule")]
    Add {
        #[arg(help = "Rule file path")]
        file: PathBuf,

        #[arg(long, help = "Rule group")]
        group: Option<String>,

        #[arg(long, help = "Validate rule before adding")]
        validate: bool,
    },

    #[command(about = "Remove alert rule")]
    Remove {
        #[arg(help = "Rule name")]
        name: String,

        #[arg(long, help = "Rule group")]
        group: Option<String>,

        #[arg(long, help = "Force removal without confirmation")]
        force: bool,
    },

    #[command(about = "Test alert rule")]
    Test {
        #[arg(help = "Rule name or file")]
        rule: String,

        #[arg(long, help = "Test data file")]
        data: Option<PathBuf>,

        #[arg(long, help = "Test duration")]
        duration: Option<String>,
    },

    #[command(about = "Show active alerts")]
    Active {
        #[arg(long, help = "Filter by severity")]
        severity: Option<String>,

        #[arg(long, help = "Filter by label")]
        label: Vec<String>,

        #[arg(long, help = "Output format")]
        format: Option<OutputFormat>,
    },

    #[command(about = "Show alert history")]
    History {
        #[arg(long, help = "Start time")]
        start: Option<String>,

        #[arg(long, help = "End time")]
        end: Option<String>,

        #[arg(long, help = "Filter by rule")]
        rule: Option<String>,

        #[arg(long, help = "Limit results")]
        limit: Option<usize>,
    },

    #[command(about = "Acknowledge alert")]
    Ack {
        #[arg(help = "Alert fingerprint or label matcher")]
        alert: String,

        #[arg(long, help = "Acknowledgment comment")]
        comment: Option<String>,

        #[arg(long, help = "Acknowledgment expiry")]
        expires: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ExportAction {
    #[command(about = "Export metrics data")]
    Metrics {
        #[arg(help = "Output file path")]
        output: PathBuf,

        #[arg(long, help = "Start time")]
        start: Option<String>,

        #[arg(long, help = "End time")]
        end: Option<String>,

        #[arg(long, help = "Metric names to export")]
        metrics: Vec<String>,

        #[arg(long, help = "Export format")]
        format: Option<ExportFormat>,

        #[arg(long, help = "Compression")]
        compress: bool,
    },

    #[command(about = "Export alerts data")]
    Alerts {
        #[arg(help = "Output file path")]
        output: PathBuf,

        #[arg(long, help = "Start time")]
        start: Option<String>,

        #[arg(long, help = "End time")]
        end: Option<String>,

        #[arg(long, help = "Export format")]
        format: Option<ExportFormat>,
    },

    #[command(about = "Export configuration")]
    Config {
        #[arg(help = "Output directory")]
        output: PathBuf,

        #[arg(long, help = "Include sensitive data")]
        include_secrets: bool,

        #[arg(long, help = "Export format")]
        format: Option<ExportFormat>,
    },

    #[command(about = "Export dashboards")]
    Dashboards {
        #[arg(help = "Output directory")]
        output: PathBuf,

        #[arg(long, help = "Dashboard names to export")]
        dashboards: Vec<String>,

        #[arg(long, help = "Export format")]
        format: Option<ExportFormat>,
    },
}

#[derive(Debug, Subcommand)]
pub enum RetentionAction {
    #[command(about = "Show retention policies")]
    Show {
        #[arg(long, help = "Show policy details")]
        detailed: bool,

        #[arg(long, help = "Output format")]
        format: Option<OutputFormat>,
    },

    #[command(about = "Update retention policy")]
    Update {
        #[arg(long, help = "Metrics retention duration")]
        metrics: Option<String>,

        #[arg(long, help = "Alerts retention duration")]
        alerts: Option<String>,

        #[arg(long, help = "Logs retention duration")]
        logs: Option<String>,

        #[arg(long, help = "Enable automatic cleanup")]
        auto_cleanup: Option<bool>,
    },

    #[command(about = "Run cleanup manually")]
    Cleanup {
        #[arg(long, help = "Cleanup type")]
        cleanup_type: Option<CleanupType>,

        #[arg(long, help = "Cleanup older than")]
        older_than: Option<String>,

        #[arg(long, help = "Dry run - show what would be cleaned")]
        dry_run: bool,

        #[arg(long, help = "Force cleanup without confirmation")]
        force: bool,
    },

    #[command(about = "Compact data")]
    Compact {
        #[arg(long, help = "Compaction level")]
        level: Option<u32>,

        #[arg(long, help = "Start time")]
        start: Option<String>,

        #[arg(long, help = "End time")]
        end: Option<String>,

        #[arg(long, help = "Force compaction")]
        force: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum TestAction {
    #[command(about = "Test configuration validity")]
    Config {
        #[arg(help = "Configuration file path")]
        config: Option<PathBuf>,

        #[arg(long, help = "Test specific component")]
        component: Option<String>,
    },

    #[command(about = "Test connectivity")]
    Connectivity {
        #[arg(long, help = "Test Prometheus connectivity")]
        prometheus: bool,

        #[arg(long, help = "Test Alertmanager connectivity")]
        alertmanager: bool,

        #[arg(long, help = "Test Grafana connectivity")]
        grafana: bool,

        #[arg(long, help = "Test all connections")]
        all: bool,
    },

    #[command(about = "Test alert rules")]
    AlertRules {
        #[arg(help = "Alert rules file")]
        file: PathBuf,

        #[arg(long, help = "Test data file")]
        data: Option<PathBuf>,
    },

    #[command(about = "Test notification channels")]
    Notifications {
        #[arg(help = "Receiver name")]
        receiver: String,

        #[arg(long, help = "Test message")]
        message: Option<String>,
    },

    #[command(about = "Load testing")]
    Load {
        #[arg(long, help = "Number of concurrent requests")]
        concurrency: Option<u32>,

        #[arg(long, help = "Test duration in seconds")]
        duration: Option<u64>,

        #[arg(long, help = "Requests per second")]
        rate: Option<f64>,
    },
}

#[derive(Debug, Subcommand)]
pub enum RulesAction {
    #[command(about = "List recording rules")]
    List {
        #[arg(long, help = "Filter by group")]
        group: Option<String>,

        #[arg(long, help = "Output format")]
        format: Option<OutputFormat>,
    },

    #[command(about = "Add recording rule")]
    Add {
        #[arg(help = "Rule file path")]
        file: PathBuf,

        #[arg(long, help = "Rule group")]
        group: Option<String>,
    },

    #[command(about = "Remove recording rule")]
    Remove {
        #[arg(help = "Rule name")]
        name: String,

        #[arg(long, help = "Rule group")]
        group: Option<String>,
    },

    #[command(about = "Test recording rule")]
    Test {
        #[arg(help = "Rule file")]
        file: PathBuf,

        #[arg(long, help = "Test duration")]
        duration: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum RemoteWriteAction {
    #[command(about = "List remote write configurations")]
    List {
        #[arg(long, help = "Output format")]
        format: Option<OutputFormat>,
    },

    #[command(about = "Add remote write endpoint")]
    Add {
        #[arg(help = "Remote write URL")]
        url: String,

        #[arg(long, help = "Remote write name")]
        name: Option<String>,

        #[arg(long, help = "Authentication header")]
        auth: Option<String>,

        #[arg(long, help = "Queue configuration")]
        queue_config: Option<PathBuf>,
    },

    #[command(about = "Remove remote write endpoint")]
    Remove {
        #[arg(help = "Remote write name or URL")]
        endpoint: String,
    },

    #[command(about = "Test remote write endpoint")]
    Test {
        #[arg(help = "Remote write name or URL")]
        endpoint: String,

        #[arg(long, help = "Test timeout")]
        timeout: Option<u64>,
    },
}

#[derive(Debug, Subcommand)]
pub enum SilenceAction {
    #[command(about = "List active silences")]
    List {
        #[arg(long, help = "Show expired silences")]
        expired: bool,

        #[arg(long, help = "Output format")]
        format: Option<OutputFormat>,
    },

    #[command(about = "Create silence")]
    Create {
        #[arg(help = "Alert matcher")]
        matcher: String,

        #[arg(long, help = "Silence duration")]
        duration: String,

        #[arg(long, help = "Silence comment")]
        comment: Option<String>,

        #[arg(long, help = "Created by")]
        created_by: Option<String>,
    },

    #[command(about = "Remove silence")]
    Remove {
        #[arg(help = "Silence ID")]
        id: String,
    },

    #[command(about = "Extend silence")]
    Extend {
        #[arg(help = "Silence ID")]
        id: String,

        #[arg(help = "Extension duration")]
        duration: String,
    },
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Json,
    Yaml,
    Csv,
    Table,
    Plain,
}

#[derive(Debug, Clone)]
pub enum AlertState {
    Firing,
    Pending,
    Inactive,
}

#[derive(Debug, Clone)]
pub enum DiscoveryMethod {
    Kubernetes,
    Dns,
    Consul,
    File,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Yaml,
    Csv,
    Prometheus,
    Parquet,
}

#[derive(Debug, Clone)]
pub enum CleanupType {
    Metrics,
    Alerts,
    Logs,
    All,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "yaml" => Ok(OutputFormat::Yaml),
            "table" => Ok(OutputFormat::Table),
            "plain" => Ok(OutputFormat::Plain),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

impl std::str::FromStr for AlertState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "firing" => Ok(AlertState::Firing),
            "pending" => Ok(AlertState::Pending),
            "inactive" => Ok(AlertState::Inactive),
            _ => Err(format!("Invalid alert state: {}", s)),
        }
    }
}

impl std::str::FromStr for DiscoveryMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "kubernetes" | "k8s" => Ok(DiscoveryMethod::Kubernetes),
            "dns" => Ok(DiscoveryMethod::Dns),
            "consul" => Ok(DiscoveryMethod::Consul),
            "file" => Ok(DiscoveryMethod::File),
            _ => Err(format!("Invalid discovery method: {}", s)),
        }
    }
}

impl std::str::FromStr for ExportFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            "prometheus" => Ok(ExportFormat::Prometheus),
            "parquet" => Ok(ExportFormat::Parquet),
            _ => Err(format!("Invalid export format: {}", s)),
        }
    }
}

impl std::str::FromStr for CleanupType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "metrics" => Ok(CleanupType::Metrics),
            "alerts" => Ok(CleanupType::Alerts),
            "logs" => Ok(CleanupType::Logs),
            "all" => Ok(CleanupType::All),
            _ => Err(format!("Invalid cleanup type: {}", s)),
        }
    }
}

pub async fn execute(args: AdvancedMonitoringArgs, config: &Config) -> Result<()> {
    match args.command {
        AdvancedMonitoringCommand::Start {
            config: config_override,
            daemon,
            bind_address,
            metrics_port,
            dashboard_port,
            debug,
        } => {
            handle_start_command(
                config,
                config_override,
                daemon,
                bind_address,
                metrics_port,
                dashboard_port,
                debug,
            )
            .await
        }

        AdvancedMonitoringCommand::Stop { force, timeout } => {
            handle_stop_command(force, timeout).await
        }

        AdvancedMonitoringCommand::Status {
            detailed,
            alerts,
            metrics,
            format,
        } => handle_status_command(config, detailed, alerts, metrics, format).await,

        AdvancedMonitoringCommand::Prometheus { action } => {
            handle_prometheus_command(config, action).await
        }

        AdvancedMonitoringCommand::Alertmanager { action } => {
            handle_alertmanager_command(config, action).await
        }

        AdvancedMonitoringCommand::Dashboard { action } => {
            handle_dashboard_command(config, action).await
        }

        AdvancedMonitoringCommand::Targets { action } => {
            handle_targets_command(config, action).await
        }

        AdvancedMonitoringCommand::Alerts { action } => handle_alerts_command(config, action).await,

        AdvancedMonitoringCommand::Export { action } => handle_export_command(config, action).await,

        AdvancedMonitoringCommand::Health {
            comprehensive,
            component,
            format,
        } => handle_health_command(config, comprehensive, component, format).await,

        AdvancedMonitoringCommand::Retention { action } => {
            handle_retention_command(config, action).await
        }

        AdvancedMonitoringCommand::Test { action } => handle_test_command(config, action).await,
    }
}

async fn handle_start_command(
    config: &Config,
    config_override: Option<PathBuf>,
    daemon: bool,
    bind_address: Option<String>,
    metrics_port: Option<u16>,
    dashboard_port: Option<u16>,
    debug: bool,
) -> Result<()> {
    info!("Starting advanced monitoring system");

    // Load configuration with overrides
    let mut monitoring_config = config.monitoring.clone();

    if let Some(addr) = bind_address {
        monitoring_config.prometheus.global.external_url = format!(
            "http://{}:{}",
            addr, monitoring_config.prometheus.global.scrape_interval_seconds
        );
    }

    if let Some(port) = metrics_port {
        // Update metrics port in config
    }

    if let Some(port) = dashboard_port {
        // Update dashboard port in config
    }

    // Initialize monitoring system
    let monitoring_system = AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

    // Start the system
    monitoring_system.start().await?;

    if daemon {
        info!("Running in daemon mode");
        // In daemon mode, we would typically detach from the terminal
        // For now, we'll just run in the background
    }

    info!("Advanced monitoring system started successfully");
    info!(
        "Metrics endpoint: http://{}:{}/metrics",
        monitoring_config.prometheus.global.external_url,
        monitoring_config.prometheus.global.scrape_interval_seconds
    );

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutdown signal received, stopping monitoring system");

    monitoring_system.stop().await?;
    info!("Advanced monitoring system stopped");

    Ok(())
}

async fn handle_stop_command(force: bool, timeout: Option<u64>) -> Result<()> {
    info!("Stopping advanced monitoring system");

    if force {
        warn!("Force stop requested - performing immediate shutdown");
        // Implement force stop logic
        return Ok(());
    }

    let timeout_duration = timeout.unwrap_or(30);
    info!("Graceful shutdown with {} second timeout", timeout_duration);

    // Implement graceful shutdown logic
    tokio::time::timeout(std::time::Duration::from_secs(timeout_duration), async {
        // Shutdown monitoring system gracefully
        info!("Monitoring system stopped gracefully");
    })
    .await
    .map_err(|_| anyhow::anyhow!("Shutdown timeout exceeded"))?;

    Ok(())
}

async fn handle_status_command(
    config: &Config,
    detailed: bool,
    alerts: bool,
    metrics: bool,
    format: Option<OutputFormat>,
) -> Result<()> {
    info!("Getting monitoring system status");

    let monitoring_system = AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

    let status = monitoring_system.get_status().await?;

    let output_format = format.unwrap_or(OutputFormat::Table);

    match output_format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&status)?);
        }
        OutputFormat::Csv => {
            // CSV output for status
            println!("component,status,healthy,message");
            for (name, component_status) in &status.components {
                println!(
                    "{},{},{},\"{}\"",
                    name,
                    if component_status.healthy {
                        "OK"
                    } else {
                        "ERROR"
                    },
                    component_status.healthy,
                    component_status.message.replace("\"", "\"\"")
                );
            }
        }
        OutputFormat::Table => {
            println!("Advanced Monitoring System Status");
            println!("=================================");
            println!(
                "Status: {}",
                if status.healthy {
                    "Healthy"
                } else {
                    "Unhealthy"
                }
            );
            println!("Uptime: {} seconds", status.uptime.as_secs());
            println!("Components:");

            for (name, component_status) in status.components {
                println!(
                    "  {}: {}",
                    name,
                    if component_status.healthy {
                        "OK"
                    } else {
                        "ERROR"
                    }
                );
                if detailed && !component_status.message.is_empty() {
                    println!("    Message: {}", component_status.message);
                }
            }

            if alerts {
                println!("\nActive Alerts: {}", status.active_alerts);
            }

            if metrics {
                println!("Metrics Collected: {}", status.metrics_collected);
                println!("Last Collection: {:?}", status.last_collection);
            }
        }
        OutputFormat::Plain => {
            println!("Status: {}", if status.healthy { "OK" } else { "ERROR" });
            if detailed {
                println!("Uptime: {}s", status.uptime.as_secs());
                println!("Components: {}", status.components.len());
                println!("Active Alerts: {}", status.active_alerts);
                println!("Metrics: {}", status.metrics_collected);
            }
        }
    }

    Ok(())
}

async fn handle_prometheus_command(config: &Config, action: PrometheusAction) -> Result<()> {
    match action {
        PrometheusAction::Config { raw, validate } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let prometheus_config = monitoring_system.get_prometheus_config().await?;

            if validate {
                match monitoring_system.validate_prometheus_config().await {
                    Ok(_) => println!("Prometheus configuration is valid"),
                    Err(e) => {
                        error!("Prometheus configuration validation failed: {}", e);
                        return Err(e);
                    }
                }
            }

            if raw {
                println!("{}", serde_yaml::to_string(&prometheus_config)?);
            } else {
                println!("Prometheus Configuration Summary");
                println!("==============================");
                println!(
                    "Global scrape interval: {}s",
                    prometheus_config.global.scrape_interval_seconds
                );
                println!(
                    "Global evaluation interval: {}s",
                    prometheus_config.global.evaluation_interval_seconds
                );
                println!("Scrape jobs: {}", prometheus_config.scrape_configs.len());
                println!("Recording rules: {}", prometheus_config.rule_files.len());
                println!(
                    "Remote write endpoints: {}",
                    prometheus_config.remote_write.len()
                );
                println!(
                    "Remote read endpoints: {}",
                    prometheus_config.remote_read.len()
                );
            }
        }

        PrometheusAction::Reload { force } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            if !force {
                if let Err(e) = monitoring_system.validate_prometheus_config().await {
                    error!("Configuration validation failed: {}", e);
                    return Err(anyhow::anyhow!("Use --force to reload anyway"));
                }
            }

            monitoring_system.reload_prometheus_config().await?;
            info!("Prometheus configuration reloaded successfully");
        }

        PrometheusAction::Query {
            query,
            time,
            timeout,
            format,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let query_time = time.as_deref().unwrap_or("now");
            let query_timeout = timeout.unwrap_or(30);

            let result = monitoring_system.query_prometheus(&query, query_time, query_timeout)?;

            let output_format = format.unwrap_or(OutputFormat::Json);
            match output_format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&result)?),
                OutputFormat::Table => {
                    // Format as table
                    println!("Query: {}", query);
                    println!("Result: {}", serde_json::to_string(&result)?);
                }
                _ => println!("{}", serde_json::to_string(&result)?),
            }
        }

        PrometheusAction::QueryRange {
            query,
            start,
            end,
            step,
            format,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let step_duration = step.as_deref().unwrap_or("1m");

            let result =
                monitoring_system.query_range_prometheus(&query, &start, &end, step_duration)?;

            let output_format = format.unwrap_or(OutputFormat::Json);
            match output_format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&result)?),
                OutputFormat::Csv => {
                    // Convert to CSV format
                    println!("timestamp,value");
                    // Implementation would depend on the result structure
                }
                _ => println!("{}", serde_json::to_string(&result)?),
            }
        }

        PrometheusAction::Rules { action } => {
            handle_rules_action(config, action).await?;
        }

        PrometheusAction::RemoteWrite { action } => {
            handle_remote_write_action(config, action).await?;
        }

        PrometheusAction::Targets {
            active,
            unhealthy,
            job,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let targets = monitoring_system.get_prometheus_targets().await?;

            println!("Prometheus Targets");
            println!("=================");

            for target in targets {
                let should_show = match (active, unhealthy, &job) {
                    (true, _, _) if !target.health.as_str().eq_ignore_ascii_case("up") => continue,
                    (_, true, _) if target.health.as_str().eq_ignore_ascii_case("up") => continue,
                    (_, _, Some(job_filter)) if !target.job.contains(job_filter) => continue,
                    _ => true,
                };

                if should_show {
                    println!(
                        "Job: {} | Target: {} | Health: {} | Last Scrape: {}",
                        target.job, target.instance, target.health, target.last_scrape
                    );
                }
            }
        }

        PrometheusAction::Info => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let info = monitoring_system.get_prometheus_info().await?;

            println!("Prometheus Build Information");
            println!("===========================");
            println!("Version: {}", info.version);
            println!("Revision: {}", info.revision);
            println!("Branch: {}", info.branch);
            println!("Build User: {}", info.build_user);
            println!("Build Date: {}", info.build_date);
            println!("Go Version: {}", info.go_version);
        }
    }

    Ok(())
}

async fn handle_alertmanager_command(config: &Config, action: AlertmanagerAction) -> Result<()> {
    match action {
        AlertmanagerAction::Config { raw, validate } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let alertmanager_config = monitoring_system.get_alertmanager_config().await?;

            if validate {
                match monitoring_system.validate_alertmanager_config().await {
                    Ok(_) => println!("Alertmanager configuration is valid"),
                    Err(e) => {
                        error!("Alertmanager configuration validation failed: {}", e);
                        return Err(e);
                    }
                }
            }

            if raw {
                println!("{}", serde_yaml::to_string(&alertmanager_config)?);
            } else {
                println!("Alertmanager Configuration Summary");
                println!("=================================");
                println!(
                    "Global resolve timeout: {}s",
                    alertmanager_config.global.resolve_timeout_seconds
                );
                println!("Routes: {}", alertmanager_config.routes.len());
                println!("Receivers: {}", alertmanager_config.receivers.len());
                println!("Inhibit rules: {}", alertmanager_config.inhibit_rules.len());
            }
        }

        AlertmanagerAction::Reload { force } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            if !force {
                if let Err(e) = monitoring_system.validate_alertmanager_config().await {
                    error!("Configuration validation failed: {}", e);
                    return Err(anyhow::anyhow!("Use --force to reload anyway"));
                }
            }

            monitoring_system.reload_alertmanager_config().await?;
            info!("Alertmanager configuration reloaded successfully");
        }

        AlertmanagerAction::Alerts {
            state,
            receiver,
            label,
            format,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let alerts = monitoring_system.get_alertmanager_alerts(
                state.as_ref(),
                receiver.as_deref(),
                &label,
            )?;

            let output_format = format.unwrap_or(OutputFormat::Table);
            match output_format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&alerts)?);
                }
                OutputFormat::Table => {
                    println!("Active Alerts");
                    println!("=============");
                    for alert in alerts {
                        println!(
                            "Alert: {} | State: {} | Started: {} | Receiver: {}",
                            alert.name, alert.state, alert.started_at, alert.receiver
                        );
                        for (key, value) in alert.labels {
                            println!("  {}: {}", key, value);
                        }
                    }
                }
                _ => println!("{}", serde_json::to_string(&alerts)?),
            }
        }

        AlertmanagerAction::Silence { action } => {
            handle_silence_action(config, action).await?;
        }

        AlertmanagerAction::Test {
            receiver,
            labels,
            annotations,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let test_result =
                monitoring_system.test_alertmanager_receiver(&receiver, &labels, &annotations)?;

            if test_result.success {
                info!(
                    "Test notification sent successfully to receiver: {}",
                    receiver
                );
            } else {
                error!(
                    "Test notification failed: {}",
                    test_result.error.unwrap_or_default()
                );
            }
        }

        AlertmanagerAction::Status => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let status = monitoring_system.get_alertmanager_status().await?;

            println!("Alertmanager Status");
            println!("==================");
            println!("Version: {}", status.version);
            println!("Uptime: {}s", status.uptime.as_secs());
            println!("Active Alerts: {}", status.active_alerts);
            println!("Silences: {}", status.silences);
            println!("Cluster Size: {}", status.cluster_size);
        }
    }

    Ok(())
}

async fn handle_dashboard_command(config: &Config, action: DashboardAction) -> Result<()> {
    match action {
        DashboardAction::List {
            tag,
            imported,
            format,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let dashboards = monitoring_system.list_dashboards(&tag, imported).await?;

            let output_format = format.unwrap_or(OutputFormat::Table);
            match output_format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&dashboards)?);
                }
                OutputFormat::Table => {
                    println!("Available Dashboards");
                    println!("===================");
                    for dashboard in dashboards {
                        println!(
                            "ID: {} | Name: {} | Folder: {} | Tags: {:?}",
                            dashboard.id, dashboard.name, dashboard.folder, dashboard.tags
                        );
                    }
                }
                _ => println!("{}", serde_json::to_string(&dashboards)?),
            }
        }

        DashboardAction::Import {
            source,
            name,
            folder,
            overwrite,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let dashboard_id = monitoring_system.import_dashboard(
                &source,
                name.as_deref(),
                folder.as_deref(),
                overwrite,
            )?;

            info!("Dashboard imported successfully with ID: {}", dashboard_id);
        }

        DashboardAction::Export {
            dashboard,
            output,
            include_variables,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            monitoring_system.export_dashboard(&dashboard, &output, include_variables)?;

            info!("Dashboard exported to: {}", output.display());
        }

        DashboardAction::Update {
            dashboard,
            file,
            message,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            monitoring_system.update_dashboard(&dashboard, &file, message.as_deref())?;

            info!("Dashboard updated successfully");
        }

        DashboardAction::Delete { dashboard, force } => {
            if !force {
                println!(
                    "Are you sure you want to delete dashboard '{}'? (y/N)",
                    dashboard
                );
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    info!("Dashboard deletion cancelled");
                    return Ok(());
                }
            }

            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            monitoring_system.delete_dashboard(&dashboard).await?;
            info!("Dashboard deleted successfully");
        }

        DashboardAction::Snapshot {
            dashboard,
            name,
            expires,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let snapshot_url = monitoring_system.create_dashboard_snapshot(
                &dashboard,
                name.as_deref(),
                expires,
            )?;

            info!("Dashboard snapshot created: {}", snapshot_url);
        }

        DashboardAction::Provision {
            directory,
            watch,
            folder,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            if watch {
                info!(
                    "Watching directory for dashboard changes: {}",
                    directory.display()
                );
                monitoring_system.watch_and_provision_dashboards(&directory, folder.as_deref())?;
            } else {
                let count =
                    monitoring_system.provision_dashboards(&directory, folder.as_deref())?;
                info!(
                    "Provisioned {} dashboards from {}",
                    count,
                    directory.display()
                );
            }
        }
    }

    Ok(())
}

async fn handle_targets_command(config: &Config, action: TargetAction) -> Result<()> {
    match action {
        TargetAction::List {
            target_type,
            healthy,
            unhealthy,
            format,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let targets = monitoring_system.list_monitoring_targets(
                target_type.as_deref(),
                healthy,
                unhealthy,
            )?;

            let output_format = format.unwrap_or(OutputFormat::Table);
            match output_format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&targets)?);
                }
                OutputFormat::Table => {
                    println!("Monitoring Targets");
                    println!("=================");
                    for target in targets {
                        println!(
                            "ID: {} | Address: {} | Type: {} | Status: {} | Last Check: {}",
                            target.id,
                            target.address,
                            target.target_type,
                            target.status,
                            target.last_check
                        );
                    }
                }
                _ => println!("{}", serde_json::to_string(&targets)?),
            }
        }

        TargetAction::Add {
            address,
            target_type,
            labels,
            interval,
            timeout,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let target_id = monitoring_system.add_monitoring_target(
                &address,
                &target_type,
                &labels,
                interval.as_deref(),
                timeout.as_deref(),
            )?;

            info!("Monitoring target added with ID: {}", target_id);
        }

        TargetAction::Remove { target, force } => {
            if !force {
                println!("Are you sure you want to remove target '{}'? (y/N)", target);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    info!("Target removal cancelled");
                    return Ok(());
                }
            }

            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            monitoring_system.remove_monitoring_target(&target).await?;
            info!("Monitoring target removed successfully");
        }

        TargetAction::Update {
            target,
            labels,
            interval,
            timeout,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            monitoring_system.update_monitoring_target(
                &target,
                &labels,
                interval.as_deref(),
                timeout.as_deref(),
            )?;

            info!("Monitoring target updated successfully");
        }

        TargetAction::Test { address, timeout } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let test_timeout = timeout.unwrap_or(10);
            let result = monitoring_system.test_target_connectivity(&address, test_timeout)?;

            if result.success {
                info!(
                    "Target connectivity test successful - Response time: {}ms",
                    result.response_time.unwrap_or_default()
                );
            } else {
                error!(
                    "Target connectivity test failed: {}",
                    result.error.unwrap_or_default()
                );
            }
        }

        TargetAction::Discover {
            method,
            config: config_file,
            auto_add,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let discovered_targets =
                monitoring_system.discover_targets(method.as_ref(), config_file.as_deref())?;

            info!("Discovered {} targets", discovered_targets.len());

            for target in &discovered_targets {
                println!("Discovered: {} ({})", target.address, target.target_type);
            }

            if auto_add {
                let added_count =
                    monitoring_system.auto_add_discovered_targets(&discovered_targets)?;
                info!("Automatically added {} targets", added_count);
            }
        }
    }

    Ok(())
}

async fn handle_alerts_command(config: &Config, action: AlertAction) -> Result<()> {
    match action {
        AlertAction::Rules {
            group,
            firing,
            format,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let rules = monitoring_system.list_alert_rules(group.as_deref(), firing)?;

            let output_format = format.unwrap_or(OutputFormat::Table);
            match output_format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&rules)?);
                }
                OutputFormat::Table => {
                    println!("Alert Rules");
                    println!("===========");
                    for rule in rules {
                        println!(
                            "Rule: {} | Group: {} | State: {} | Severity: {}",
                            rule.name, rule.group, rule.state, rule.severity
                        );
                        if firing && rule.state == "firing" {
                            println!(
                                "  Firing for: {} | Labels: {:?}",
                                rule.firing_duration.unwrap_or_default(),
                                rule.labels
                            );
                        }
                    }
                }
                _ => println!("{}", serde_json::to_string(&rules)?),
            }
        }

        AlertAction::Add {
            file,
            group,
            validate,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            if validate {
                monitoring_system.validate_alert_rules(&file).await?;
                info!("Alert rules validation passed");
            }

            let rule_id = monitoring_system.add_alert_rule(&file, group.as_deref())?;

            info!("Alert rule added with ID: {}", rule_id);
        }

        AlertAction::Remove { name, group, force } => {
            if !force {
                println!(
                    "Are you sure you want to remove alert rule '{}'? (y/N)",
                    name
                );
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    info!("Alert rule removal cancelled");
                    return Ok(());
                }
            }

            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            monitoring_system
                .remove_alert_rule(&name, group.as_deref())
                .await?;
            info!("Alert rule removed successfully");
        }

        AlertAction::Test {
            rule,
            data,
            duration,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let test_result =
                monitoring_system.test_alert_rule(&rule, data.as_deref(), duration.as_deref())?;

            if test_result.success {
                info!("Alert rule test passed");
                if let Some(alerts) = test_result.triggered_alerts {
                    info!("Would trigger {} alerts", alerts.len());
                    for alert in alerts {
                        println!("  Alert: {} | Severity: {}", alert.name, alert.severity);
                    }
                }
            } else {
                error!(
                    "Alert rule test failed: {}",
                    test_result.error.unwrap_or_default()
                );
            }
        }

        AlertAction::Active {
            severity,
            label,
            format,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let active_alerts = monitoring_system.get_active_alerts(severity.as_deref(), &label)?;

            let output_format = format.unwrap_or(OutputFormat::Table);
            match output_format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&active_alerts)?);
                }
                OutputFormat::Table => {
                    println!("Active Alerts");
                    println!("=============");
                    for alert in active_alerts {
                        println!(
                            "Alert: {} | Severity: {} | Started: {} | Duration: {}",
                            alert.name, alert.severity, alert.started_at, alert.duration
                        );
                        for (key, value) in alert.labels {
                            println!("  {}: {}", key, value);
                        }
                    }
                }
                _ => println!("{}", serde_json::to_string(&active_alerts)?),
            }
        }

        AlertAction::History {
            start,
            end,
            rule,
            limit,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let alert_history = monitoring_system.get_alert_history(
                start.as_deref(),
                end.as_deref(),
                rule.as_deref(),
                limit,
            )?;

            println!("Alert History");
            println!("=============");
            for entry in alert_history {
                println!(
                    "Alert: {} | State: {} | Time: {} | Duration: {}",
                    entry.name, entry.state, entry.timestamp, entry.duration
                );
            }
        }

        AlertAction::Ack {
            alert,
            comment,
            expires,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            monitoring_system.acknowledge_alert(&alert, comment.as_deref(), expires.as_deref())?;

            info!("Alert acknowledged: {}", alert);
        }
    }

    Ok(())
}

async fn handle_export_command(config: &Config, action: ExportAction) -> Result<()> {
    match action {
        ExportAction::Metrics {
            output,
            start,
            end,
            metrics,
            format,
            compress,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let export_format = format.unwrap_or(ExportFormat::Json);

            monitoring_system.export_metrics(
                &output,
                start.as_deref(),
                end.as_deref(),
                &metrics,
                &export_format,
                compress,
            )?;

            info!("Metrics exported to: {}", output.display());
        }

        ExportAction::Alerts {
            output,
            start,
            end,
            format,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let export_format = format.unwrap_or(ExportFormat::Json);

            monitoring_system.export_alerts(
                &output,
                start.as_deref(),
                end.as_deref(),
                &export_format,
            )?;

            info!("Alerts exported to: {}", output.display());
        }

        ExportAction::Config {
            output,
            include_secrets,
            format,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let export_format = format.unwrap_or(ExportFormat::Yaml);

            monitoring_system.export_configuration(&output, include_secrets, &export_format)?;

            info!("Configuration exported to: {}", output.display());
        }

        ExportAction::Dashboards {
            output,
            dashboards,
            format,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let export_format = format.unwrap_or(ExportFormat::Json);

            monitoring_system.export_dashboards(&output, &dashboards, &export_format)?;

            info!("Dashboards exported to: {}", output.display());
        }
    }

    Ok(())
}

async fn handle_health_command(
    config: &Config,
    comprehensive: bool,
    component: Option<String>,
    format: Option<OutputFormat>,
) -> Result<()> {
    let monitoring_system = AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

    let health_check = if comprehensive {
        monitoring_system.comprehensive_health_check().await?
    } else if let Some(comp) = component {
        monitoring_system.component_health_check(&comp).await?
    } else {
        monitoring_system.basic_health_check().await?
    };

    let output_format = format.unwrap_or(OutputFormat::Table);
    match output_format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&health_check)?);
        }
        OutputFormat::Table => {
            println!("Health Check Results");
            println!("===================");
            println!(
                "Overall Status: {}",
                if health_check.healthy {
                    "HEALTHY"
                } else {
                    "UNHEALTHY"
                }
            );
            println!("Check Time: {}", health_check.timestamp);

            for (component, status) in health_check.components {
                println!(
                    "Component: {} | Status: {} | Response Time: {}ms",
                    component,
                    if status.healthy { "OK" } else { "ERROR" },
                    status.response_time.unwrap_or_default()
                );
                if !status.message.is_empty() {
                    println!("  Message: {}", status.message);
                }
            }

            if comprehensive {
                println!("\nDetailed Metrics:");
                println!(
                    "Memory Usage: {}MB",
                    health_check.memory_usage.unwrap_or_default()
                );
                println!("CPU Usage: {}%", health_check.cpu_usage.unwrap_or_default());
                println!(
                    "Disk Usage: {}%",
                    health_check.disk_usage.unwrap_or_default()
                );
                println!(
                    "Network Latency: {}ms",
                    health_check.network_latency.unwrap_or_default()
                );
            }
        }
        _ => println!("{}", serde_json::to_string(&health_check)?),
    }

    Ok(())
}

async fn handle_retention_command(config: &Config, action: RetentionAction) -> Result<()> {
    match action {
        RetentionAction::Show { detailed, format } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let retention_policies = monitoring_system.get_retention_policies().await?;

            let output_format = format.unwrap_or(OutputFormat::Table);
            match output_format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&retention_policies)?);
                }
                OutputFormat::Table => {
                    println!("Retention Policies");
                    println!("=================");
                    for policy in retention_policies {
                        println!(
                            "Type: {} | Retention: {} | Auto Cleanup: {}",
                            policy.data_type, policy.retention_period, policy.auto_cleanup
                        );
                        if detailed {
                            println!(
                                "  Last Cleanup: {}",
                                policy.last_cleanup.unwrap_or_default()
                            );
                            println!("  Data Size: {}MB", policy.current_size.unwrap_or_default());
                        }
                    }
                }
                _ => println!("{}", serde_json::to_string(&retention_policies)?),
            }
        }

        RetentionAction::Update {
            metrics,
            alerts,
            logs,
            auto_cleanup,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            monitoring_system.update_retention_policies(
                metrics.as_deref(),
                alerts.as_deref(),
                logs.as_deref(),
                auto_cleanup,
            )?;

            info!("Retention policies updated successfully");
        }

        RetentionAction::Cleanup {
            cleanup_type,
            older_than,
            dry_run,
            force,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            if dry_run {
                let cleanup_preview = monitoring_system
                    .preview_cleanup(cleanup_type.as_ref(), older_than.as_deref())?;

                println!("Cleanup Preview");
                println!("===============");
                for item in cleanup_preview {
                    println!("Would delete: {} ({} MB)", item.path, item.size_mb);
                }
                return Ok(());
            }

            if !force {
                println!(
                    "Are you sure you want to perform cleanup? This action cannot be undone. (y/N)"
                );
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    info!("Cleanup cancelled");
                    return Ok(());
                }
            }

            let cleanup_result =
                monitoring_system.perform_cleanup(cleanup_type.as_ref(), older_than.as_deref())?;

            info!(
                "Cleanup completed - Deleted {} items, freed {} MB",
                cleanup_result.deleted_count, cleanup_result.freed_space_mb
            );
        }

        RetentionAction::Compact {
            level,
            start,
            end,
            force,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            if !force {
                println!(
                    "Data compaction may take a long time and impact performance. Continue? (y/N)"
                );
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    info!("Compaction cancelled");
                    return Ok(());
                }
            }

            let compaction_result =
                monitoring_system.compact_data(level, start.as_deref(), end.as_deref())?;

            info!(
                "Data compaction completed - Processed {} blocks, saved {} MB",
                compaction_result.processed_blocks, compaction_result.space_saved_mb
            );
        }
    }

    Ok(())
}

async fn handle_test_command(config: &Config, action: TestAction) -> Result<()> {
    match action {
        TestAction::Config {
            config: config_file,
            component,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let test_result = if let Some(comp) = component {
                monitoring_system
                    .test_component_config(&comp, config_file.as_deref())
                    .await?
            } else {
                monitoring_system
                    .test_full_config(config_file.as_deref())
                    .await?
            };

            if test_result.success {
                info!("Configuration test passed");
            } else {
                error!(
                    "Configuration test failed: {}",
                    test_result.error.unwrap_or_default()
                );
                for warning in test_result.warnings {
                    warn!("Warning: {}", warning);
                }
            }
        }

        TestAction::Connectivity {
            prometheus,
            alertmanager,
            grafana,
            all,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            if all || prometheus {
                match monitoring_system.test_prometheus_connectivity().await {
                    Ok(_) => info!("Prometheus connectivity: OK"),
                    Err(e) => error!("Prometheus connectivity: FAILED - {}", e),
                }
            }

            if all || alertmanager {
                match monitoring_system.test_alertmanager_connectivity().await {
                    Ok(_) => info!("Alertmanager connectivity: OK"),
                    Err(e) => error!("Alertmanager connectivity: FAILED - {}", e),
                }
            }

            if all || grafana {
                match monitoring_system.test_grafana_connectivity().await {
                    Ok(_) => info!("Grafana connectivity: OK"),
                    Err(e) => error!("Grafana connectivity: FAILED - {}", e),
                }
            }
        }

        TestAction::AlertRules { file, data } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let test_result =
                monitoring_system.test_alert_rules_file(file.as_path(), data.as_deref())?;

            if test_result.success {
                info!(
                    "Alert rules test passed - {} rules validated",
                    test_result.rules_count
                );
            } else {
                error!(
                    "Alert rules test failed: {}",
                    test_result.error.unwrap_or_default()
                );
            }
        }

        TestAction::Notifications { receiver, message } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let test_result =
                monitoring_system.test_notification_channel(&receiver, message.as_deref())?;

            if test_result.success {
                info!(
                    "Notification test successful - Delivery time: {}ms",
                    test_result.delivery_time.unwrap_or_default()
                );
            } else {
                error!(
                    "Notification test failed: {}",
                    test_result.error.unwrap_or_default()
                );
            }
        }

        TestAction::Load {
            concurrency,
            duration,
            rate,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let test_concurrency = concurrency.unwrap_or(10);
            let test_duration = duration.unwrap_or(60);
            let test_rate = rate.unwrap_or(1.0);

            info!(
                "Starting load test - Concurrency: {}, Duration: {}s, Rate: {} req/s",
                test_concurrency, test_duration, test_rate
            );

            let load_test_result =
                monitoring_system.run_load_test(test_concurrency, test_duration, test_rate)?;

            println!("Load Test Results");
            println!("================");
            println!("Total Requests: {}", load_test_result.total_requests);
            println!(
                "Successful Requests: {}",
                load_test_result.successful_requests
            );
            println!("Failed Requests: {}", load_test_result.failed_requests);
            println!(
                "Average Response Time: {}ms",
                load_test_result.avg_response_time
            );
            println!("95th Percentile: {}ms", load_test_result.p95_response_time);
            println!("99th Percentile: {}ms", load_test_result.p99_response_time);
            println!("Throughput: {} req/s", load_test_result.throughput);
        }
    }

    Ok(())
}

async fn handle_rules_action(config: &Config, action: RulesAction) -> Result<()> {
    match action {
        RulesAction::List { group, format } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let rules = monitoring_system
                .list_recording_rules(group.as_deref())
                .await?;

            let output_format = format.unwrap_or(OutputFormat::Table);
            match output_format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&rules)?);
                }
                OutputFormat::Table => {
                    println!("Recording Rules");
                    println!("==============");
                    for rule in rules {
                        println!(
                            "Rule: {} | Group: {} | Interval: {}s",
                            rule.name, rule.group, rule.interval
                        );
                    }
                }
                _ => println!("{}", serde_json::to_string(&rules)?),
            }
        }

        RulesAction::Add { file, group } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let rule_id = monitoring_system
                .add_recording_rule(&file, group.as_deref())
                .await?;
            info!("Recording rule added with ID: {}", rule_id);
        }

        RulesAction::Remove { name, group } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            monitoring_system
                .remove_recording_rule(&name, group.as_deref())
                .await?;
            info!("Recording rule removed successfully");
        }

        RulesAction::Test { file, duration } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let test_result = monitoring_system.test_recording_rule(&file, duration.as_deref())?;

            if test_result.success {
                info!("Recording rule test passed");
            } else {
                error!(
                    "Recording rule test failed: {}",
                    test_result.error.unwrap_or_default()
                );
            }
        }
    }

    Ok(())
}

async fn handle_remote_write_action(config: &Config, action: RemoteWriteAction) -> Result<()> {
    match action {
        RemoteWriteAction::List { format } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let endpoints = monitoring_system.list_remote_write_endpoints().await?;

            let output_format = format.unwrap_or(OutputFormat::Table);
            match output_format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&endpoints)?);
                }
                OutputFormat::Table => {
                    println!("Remote Write Endpoints");
                    println!("=====================");
                    for endpoint in endpoints {
                        println!(
                            "Name: {} | URL: {} | Status: {}",
                            endpoint.name, endpoint.url, endpoint.status
                        );
                    }
                }
                _ => println!("{}", serde_json::to_string(&endpoints)?),
            }
        }

        RemoteWriteAction::Add {
            url,
            name,
            auth,
            queue_config,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let endpoint_id = monitoring_system.add_remote_write_endpoint(
                &url,
                name.as_deref(),
                auth.as_deref(),
                queue_config.as_deref(),
            )?;

            info!("Remote write endpoint added with ID: {}", endpoint_id);
        }

        RemoteWriteAction::Remove { endpoint } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            monitoring_system
                .remove_remote_write_endpoint(&endpoint)
                .await?;
            info!("Remote write endpoint removed successfully");
        }

        RemoteWriteAction::Test { endpoint, timeout } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let test_timeout = timeout.unwrap_or(30);
            let test_result =
                monitoring_system.test_remote_write_endpoint(&endpoint, test_timeout)?;

            if test_result.success {
                info!(
                    "Remote write endpoint test successful - Response time: {}ms",
                    test_result.response_time.unwrap_or_default()
                );
            } else {
                error!(
                    "Remote write endpoint test failed: {}",
                    test_result.error.unwrap_or_default()
                );
            }
        }
    }

    Ok(())
}

async fn handle_silence_action(config: &Config, action: SilenceAction) -> Result<()> {
    match action {
        SilenceAction::List { expired, format } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let silences = monitoring_system.list_silences(expired).await?;

            let output_format = format.unwrap_or(OutputFormat::Table);
            match output_format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&silences)?);
                }
                OutputFormat::Table => {
                    println!("Alert Silences");
                    println!("==============");
                    for silence in silences {
                        println!(
                            "ID: {} | Matcher: {} | Expires: {} | Created By: {}",
                            silence.id, silence.matcher, silence.expires_at, silence.created_by
                        );
                        if !silence.comment.is_empty() {
                            println!("  Comment: {}", silence.comment);
                        }
                    }
                }
                _ => println!("{}", serde_json::to_string(&silences)?),
            }
        }

        SilenceAction::Create {
            matcher,
            duration,
            comment,
            created_by,
        } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            let silence_id = monitoring_system.create_silence(
                &matcher,
                &duration,
                comment.as_deref(),
                created_by.as_deref(),
            )?;

            info!("Alert silence created with ID: {}", silence_id);
        }

        SilenceAction::Remove { id } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            monitoring_system.remove_silence(&id).await?;
            info!("Alert silence removed successfully");
        }

        SilenceAction::Extend { id, duration } => {
            let monitoring_system =
                AdvancedMonitoringSystem::new(config.monitoring.clone().into())?;

            monitoring_system.extend_silence(&id, &duration).await?;
            info!("Alert silence extended successfully");
        }
    }

    Ok(())
}
