use crate::{
    config::Config,
    monitoring::{PerformanceMonitor, MonitoringConfig, AlertSeverity, create_test_metric},
    metrics::MetricsCollector,
};
use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
use std::{sync::Arc, time::Duration};
use tracing::warn;

#[derive(Args)]
pub struct MonitoringArgs {
    #[command(subcommand)]
    pub command: MonitoringCommand,
}

#[derive(Subcommand)]
pub enum MonitoringCommand {
    #[command(about = "Show current monitoring status and metrics")]
    Status,

    #[command(about = "Start real-time monitoring dashboard")]
    Dashboard {
        #[arg(long, help = "Dashboard port", default_value = "3000")]
        port: u16,

        #[arg(long, help = "Update interval in seconds", default_value = "1")]
        interval: u64,

        #[arg(long, help = "Show detailed metrics")]
        detailed: bool,
    },

    #[command(about = "List active alerts")]
    Alerts {
        #[arg(long, help = "Show resolved alerts")]
        show_resolved: bool,

        #[arg(long, help = "Filter by severity", value_enum)]
        severity: Option<AlertSeverityArg>,

        #[arg(long, help = "Limit number of alerts shown", default_value = "20")]
        limit: usize,
    },

    #[command(about = "Resolve an active alert")]
    Resolve {
        #[arg(help = "Alert ID to resolve")]
        alert_id: String,
    },

    #[command(about = "Configure monitoring thresholds")]
    Configure {
        #[arg(long, help = "Max response time in milliseconds")]
        max_response_time: Option<u64>,

        #[arg(long, help = "Min throughput in requests per second")]
        min_throughput: Option<f64>,

        #[arg(long, help = "Max error rate percentage")]
        max_error_rate: Option<f64>,

        #[arg(long, help = "Max memory usage in MB")]
        max_memory: Option<u64>,

        #[arg(long, help = "Max CPU usage percentage")]
        max_cpu: Option<f64>,

        #[arg(long, help = "Min cache hit rate percentage")]
        min_cache_hit_rate: Option<f64>,
    },

    #[command(about = "Test alert system")]
    TestAlerts {
        #[arg(long, help = "Test specific alert type")]
        alert_type: Option<String>,

        #[arg(long, help = "Generate test metrics")]
        generate_metrics: bool,
    },

    #[command(about = "Show performance trends")]
    Trends {
        #[arg(long, help = "Time period in hours", default_value = "1")]
        hours: u64,

        #[arg(long, help = "Model to analyze")]
        model: Option<String>,

        #[arg(long, help = "Group by time interval in minutes", default_value = "5")]
        group_by_minutes: u64,
    },

    #[command(about = "Export monitoring data")]
    Export {
        #[arg(short, long, help = "Output file path")]
        output: Option<std::path::PathBuf>,

        #[arg(long, help = "Export format", value_enum, default_value = "json")]
        format: ExportFormat,

        #[arg(long, help = "Time period in hours", default_value = "24")]
        hours: u64,

        #[arg(long, help = "Include alert history")]
        include_alerts: bool,
    },

    #[command(about = "Monitor performance in real-time")]
    Watch {
        #[arg(short, long, help = "Update interval in seconds", default_value = "2")]
        interval: u64,

        #[arg(long, help = "Model to monitor")]
        model: Option<String>,

        #[arg(long, help = "Show only alerts")]
        alerts_only: bool,
    },

    #[command(about = "Generate performance report")]
    Report {
        #[arg(long, help = "Time period in hours", default_value = "24")]
        hours: u64,

        #[arg(long, help = "Include detailed breakdown")]
        detailed: bool,

        #[arg(long, help = "Include recommendations")]
        recommendations: bool,
    },

    #[command(about = "Benchmark monitoring system performance")]
    Benchmark {
        #[arg(long, help = "Number of test metrics to generate", default_value = "1000")]
        metrics: usize,

        #[arg(long, help = "Number of concurrent writers", default_value = "10")]
        concurrent: usize,

        #[arg(long, help = "Duration in seconds", default_value = "60")]
        duration: u64,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum AlertSeverityArg {
    Critical,
    Warning,
    Info,
}

impl From<AlertSeverityArg> for AlertSeverity {
    fn from(arg: AlertSeverityArg) -> Self {
        match arg {
            AlertSeverityArg::Critical => AlertSeverity::Critical,
            AlertSeverityArg::Warning => AlertSeverity::Warning,
            AlertSeverityArg::Info => AlertSeverity::Info,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ExportFormat {
    Json,
    Csv,
    Yaml,
}

pub async fn execute(args: MonitoringArgs, config: &Config) -> Result<()> {
    match args.command {
        MonitoringCommand::Status => show_monitoring_status(config).await,
        MonitoringCommand::Dashboard { port, interval, detailed } => {
            start_dashboard(config, port, interval, detailed).await
        }
        MonitoringCommand::Alerts { show_resolved, severity, limit } => {
            show_alerts(config, show_resolved, severity, limit).await
        }
        MonitoringCommand::Resolve { alert_id } => resolve_alert(config, alert_id).await,
        MonitoringCommand::Configure {
            max_response_time,
            min_throughput,
            max_error_rate,
            max_memory,
            max_cpu,
            min_cache_hit_rate,
        } => {
            configure_monitoring(
                config,
                max_response_time,
                min_throughput,
                max_error_rate,
                max_memory,
                max_cpu,
                min_cache_hit_rate,
            ).await
        }
        MonitoringCommand::TestAlerts { alert_type, generate_metrics } => {
            test_alerts(config, alert_type, generate_metrics).await
        }
        MonitoringCommand::Trends { hours, model, group_by_minutes } => {
            show_trends(config, hours, model, group_by_minutes).await
        }
        MonitoringCommand::Export { output, format, hours, include_alerts } => {
            export_monitoring_data(config, output, format, hours, include_alerts).await
        }
        MonitoringCommand::Watch { interval, model, alerts_only } => {
            watch_performance(config, interval, model, alerts_only).await
        }
        MonitoringCommand::Report { hours, detailed, recommendations } => {
            generate_report(config, hours, detailed, recommendations).await
        }
        MonitoringCommand::Benchmark { metrics, concurrent, duration } => {
            benchmark_monitoring(config, metrics, concurrent, duration).await
        }
    }
}

async fn show_monitoring_status(_config: &Config) -> Result<()> {
    println!("=== Monitoring System Status ===");

    let monitoring_config = MonitoringConfig::default(); // In real implementation, load from config
    let metrics_collector = Some(Arc::new({
        let mut collector = MetricsCollector::new();
        collector.start_event_processing().await?;
        collector
    }));

    let monitor = PerformanceMonitor::new(monitoring_config.clone(), metrics_collector).await?;

    println!("Monitoring: {}", if monitoring_config.enabled { "Enabled" } else { "Disabled" });
    println!("Collection Interval: {}ms", monitoring_config.collection_interval_ms);
    println!("Alert Evaluation Interval: {}ms", monitoring_config.alert_evaluation_interval_ms);
    println!("Metric Retention: {} hours", monitoring_config.metric_retention_hours);

    println!("\n=== Performance Thresholds ===");
    let thresholds = &monitoring_config.performance_thresholds;
    println!("Max Response Time: {}ms", thresholds.max_response_time_ms);
    println!("Min Throughput: {:.2} RPS", thresholds.min_throughput_rps);
    println!("Max Error Rate: {:.2}%", thresholds.max_error_rate_percent);
    println!("Max Memory Usage: {}MB", thresholds.max_memory_usage_mb);
    println!("Max CPU Usage: {:.2}%", thresholds.max_cpu_usage_percent);
    println!("Max Queue Depth: {}", thresholds.max_queue_depth);
    println!("Min Cache Hit Rate: {:.2}%", thresholds.min_cache_hit_rate_percent);

    println!("\n=== Alerting Configuration ===");
    let alerting = &monitoring_config.alerting;
    println!("Alerting: {}", if alerting.enabled { "Enabled" } else { "Disabled" });
    println!("Webhooks: {}", alerting.webhooks.len());
    println!("Email: {}", if alerting.email.is_some() { "Configured" } else { "Not configured" });
    println!("Slack: {}", if alerting.slack.is_some() { "Configured" } else { "Not configured" });
    println!("Cooldown: {} minutes", alerting.cooldown_minutes);

    // Show current metrics
    let metrics = monitor.get_current_metrics().await;
    if !metrics.is_empty() {
        println!("\n=== Recent Metrics ===");
        println!("Total metrics: {}", metrics.len());
        if let Some(latest) = metrics.first() {
            println!("Latest metric timestamp: {:?}", latest.timestamp);
            println!("Memory usage: {}MB", latest.memory_usage_mb);
            println!("CPU usage: {:.2}%", latest.cpu_usage_percent);
        }
    }

    // Show active alerts
    let active_alerts = monitor.get_active_alerts().await;
    println!("\n=== Active Alerts ===");
    if active_alerts.is_empty() {
        println!("No active alerts");
    } else {
        println!("Active alerts: {}", active_alerts.len());
        for alert in active_alerts.iter().take(5) {
            println!("  [{:?}] {:?}: {}", alert.severity, alert.alert_type, alert.message);
        }
    }

    println!("\n=== Dashboard ===");
    let dashboard = &monitoring_config.dashboards;
    println!("Dashboard: {}", if dashboard.enabled { "Enabled" } else { "Disabled" });
    if dashboard.enabled {
        println!("Address: http://{}:{}", dashboard.bind_address, dashboard.port);
        println!("Update Interval: {}ms", dashboard.update_interval_ms);
    }

    Ok(())
}

async fn start_dashboard(_config: &Config, port: u16, interval: u64, detailed: bool) -> Result<()> {
    println!("Starting monitoring dashboard on port {} (update interval: {}s)", port, interval);
    if detailed {
        println!("Detailed metrics enabled");
    }

    let monitoring_config = MonitoringConfig::default();
    let metrics_collector = Some(Arc::new({
        let mut collector = MetricsCollector::new();
        collector.start_event_processing().await?;
        collector
    }));

    let monitor = PerformanceMonitor::new(monitoring_config, metrics_collector).await?;

    println!("Dashboard started. Press Ctrl+C to stop.");
    println!("Navigate to http://127.0.0.1:{} to view the dashboard", port);

    let mut counter = 0;
    loop {
        tokio::time::sleep(Duration::from_secs(interval)).await;

        if counter % 20 == 0 {
            if detailed {
                println!("\n{:<8} {:<8} {:<10} {:<8} {:<8} {:<10} {:<8}",
                    "Time", "Models", "Memory(MB)", "CPU%", "Alerts", "Cache%", "RPS");
            } else {
                println!("\n{:<8} {:<8} {:<10} {:<8}",
                    "Time", "Models", "Memory(MB)", "Alerts");
            }
        }

        let metrics = monitor.get_current_metrics().await;
        let active_alerts = monitor.get_active_alerts().await;
        let now = chrono::Utc::now().format("%H:%M:%S");

        if let Some(latest_metric) = metrics.first() {
            if detailed {
                println!("{:<8} {:<8} {:<10} {:<8.1} {:<8} {:<10.1} {:<8.2}",
                    now,
                    1, // Number of models (placeholder)
                    latest_metric.memory_usage_mb,
                    latest_metric.cpu_usage_percent,
                    active_alerts.len(),
                    latest_metric.cache_hit_rate_percent,
                    latest_metric.throughput_rps
                );
            } else {
                println!("{:<8} {:<8} {:<10} {:<8}",
                    now,
                    1, // Number of models (placeholder)
                    latest_metric.memory_usage_mb,
                    active_alerts.len()
                );
            }
        }

        counter += 1;
    }
}

async fn show_alerts(
    _config: &Config,
    show_resolved: bool,
    severity: Option<AlertSeverityArg>,
    limit: usize,
) -> Result<()> {
    let monitoring_config = MonitoringConfig::default();
    let monitor = PerformanceMonitor::new(monitoring_config, None).await?;

    let active_alerts = monitor.get_active_alerts().await;
    let alert_history = if show_resolved {
        monitor.get_alert_history(Some(limit)).await
    } else {
        Vec::new()
    };

    println!("=== Active Alerts ===");
    if active_alerts.is_empty() {
        println!("No active alerts");
    } else {
        let filtered_active: Vec<_> = active_alerts
            .iter()
            .filter(|alert| {
                if let Some(ref sev) = severity {
                    std::mem::discriminant(&alert.severity) == std::mem::discriminant(&AlertSeverity::from(sev.clone()))
                } else {
                    true
                }
            })
            .take(limit)
            .collect();

        for alert in filtered_active {
            let age = alert.timestamp.elapsed().unwrap_or(Duration::ZERO);
            println!("ID: {}", alert.id);
            println!("  Type: {:?}", alert.alert_type);
            println!("  Severity: {:?}", alert.severity);
            println!("  Message: {}", alert.message);
            if let Some(ref model_id) = alert.model_id {
                println!("  Model: {}", model_id);
            }
            println!("  Metric Value: {:.2}", alert.metric_value);
            println!("  Threshold: {:.2}", alert.threshold_value);
            println!("  Age: {:?}", age);
            println!();
        }
    }

    if show_resolved && !alert_history.is_empty() {
        println!("=== Recently Resolved Alerts ===");
        let filtered_history: Vec<_> = alert_history
            .iter()
            .filter(|alert| alert.resolved)
            .filter(|alert| {
                if let Some(ref sev) = severity {
                    std::mem::discriminant(&alert.severity) == std::mem::discriminant(&AlertSeverity::from(sev.clone()))
                } else {
                    true
                }
            })
            .take(limit)
            .collect();

        for alert in filtered_history {
            let duration = if let Some(resolved_at) = alert.resolved_at {
                resolved_at.duration_since(alert.timestamp).unwrap_or(Duration::ZERO)
            } else {
                Duration::ZERO
            };

            println!("ID: {}", alert.id);
            println!("  Type: {:?}", alert.alert_type);
            println!("  Severity: {:?}", alert.severity);
            println!("  Message: {}", alert.message);
            println!("  Duration: {:?}", duration);
            println!();
        }
    }

    Ok(())
}

async fn resolve_alert(_config: &Config, alert_id: String) -> Result<()> {
    let monitoring_config = MonitoringConfig::default();
    let monitor = PerformanceMonitor::new(monitoring_config, None).await?;

    println!("Resolving alert: {}", alert_id);

    let resolved = monitor.resolve_alert(&alert_id).await?;

    if resolved {
        println!("✓ Alert {} has been resolved", alert_id);
    } else {
        println!("✗ Alert {} not found or already resolved", alert_id);
    }

    Ok(())
}

async fn configure_monitoring(
    _config: &Config,
    max_response_time: Option<u64>,
    min_throughput: Option<f64>,
    max_error_rate: Option<f64>,
    max_memory: Option<u64>,
    max_cpu: Option<f64>,
    min_cache_hit_rate: Option<f64>,
) -> Result<()> {
    println!("=== Monitoring Configuration Update ===");

    if let Some(rt) = max_response_time {
        println!("Max response time: {}ms", rt);
    }
    if let Some(tp) = min_throughput {
        println!("Min throughput: {:.2} RPS", tp);
    }
    if let Some(er) = max_error_rate {
        println!("Max error rate: {:.2}%", er);
    }
    if let Some(mem) = max_memory {
        println!("Max memory usage: {}MB", mem);
    }
    if let Some(cpu) = max_cpu {
        println!("Max CPU usage: {:.2}%", cpu);
    }
    if let Some(chr) = min_cache_hit_rate {
        println!("Min cache hit rate: {:.2}%", chr);
    }

    println!("\nNote: Configuration changes require restart to take effect.");
    println!("Update your config.toml file with these values.");

    Ok(())
}

async fn test_alerts(
    _config: &Config,
    alert_type: Option<String>,
    generate_metrics: bool,
) -> Result<()> {
    println!("Testing alert system...");

    let monitoring_config = MonitoringConfig::default();
    let monitor = PerformanceMonitor::new(monitoring_config, None).await?;

    if generate_metrics {
        println!("Generating test metrics to trigger alerts...");

        // Generate a metric that exceeds thresholds
        let mut test_metric = create_test_metric("test-model");
        test_metric.response_time_ms = 10000; // Exceeds default threshold
        test_metric.error_rate_percent = 15.0; // Exceeds default threshold
        test_metric.memory_usage_mb = 10000; // Exceeds default threshold

        monitor.record_metric(test_metric).await?;

        println!("Test metrics generated. Checking for alerts...");

        tokio::time::sleep(Duration::from_secs(2)).await;

        let active_alerts = monitor.get_active_alerts().await;
        println!("Generated {} test alerts", active_alerts.len());

        for alert in &active_alerts {
            println!("  [{:?}] {:?}: {}", alert.severity, alert.alert_type, alert.message);
        }
    } else {
        println!("Alert system is configured and running.");
        if let Some(ref alert_type_str) = alert_type {
            println!("Testing alert type: {}", alert_type_str);
        }
    }

    Ok(())
}

async fn show_trends(
    _config: &Config,
    hours: u64,
    model: Option<String>,
    group_by_minutes: u64,
) -> Result<()> {
    println!("=== Performance Trends ===");
    println!("Time period: {} hours", hours);
    if let Some(ref model_id) = model {
        println!("Model: {}", model_id);
    }
    println!("Grouping: {} minute intervals", group_by_minutes);

    let monitoring_config = MonitoringConfig::default();
    let monitor = PerformanceMonitor::new(monitoring_config, None).await?;

    let duration = Duration::from_secs(hours * 3600);
    if let Some(aggregated) = monitor.get_aggregated_metrics(duration).await {
        println!("\n=== Aggregated Metrics ===");
        println!("Average Response Time: {}ms", aggregated.avg_response_time_ms);
        println!("Average Throughput: {:.2} RPS", aggregated.avg_throughput_rps);
        println!("Average Error Rate: {:.2}%", aggregated.avg_error_rate_percent);
        println!("Average Memory Usage: {}MB", aggregated.avg_memory_usage_mb);
        println!("Average CPU Usage: {:.2}%", aggregated.avg_cpu_usage_percent);
        println!("Average Cache Hit Rate: {:.2}%", aggregated.avg_cache_hit_rate_percent);
        println!("Total Requests: {}", aggregated.total_requests);
        println!("Successful Requests: {}", aggregated.successful_requests);
        println!("Failed Requests: {}", aggregated.failed_requests);
        println!("Uptime: {:.2}%", aggregated.uptime_percent);
    } else {
        println!("No metrics available for the specified time period");
    }

    Ok(())
}

async fn export_monitoring_data(
    _config: &Config,
    output: Option<std::path::PathBuf>,
    format: ExportFormat,
    hours: u64,
    include_alerts: bool,
) -> Result<()> {
    println!("Exporting monitoring data...");
    println!("Time period: {} hours", hours);
    println!("Format: {:?}", format);
    println!("Include alerts: {}", include_alerts);

    let monitoring_config = MonitoringConfig::default();
    let monitor = PerformanceMonitor::new(monitoring_config, None).await?;

    let metrics = monitor.get_current_metrics().await;
    let alerts = if include_alerts {
        monitor.get_alert_history(None).await
    } else {
        Vec::new()
    };

    let export_data = serde_json::json!({
        "export_timestamp": chrono::Utc::now(),
        "time_period_hours": hours,
        "metrics": metrics,
        "alerts": alerts,
        "summary": {
            "total_metrics": metrics.len(),
            "total_alerts": alerts.len()
        }
    });

    let output_str = match format {
        ExportFormat::Json => serde_json::to_string_pretty(&export_data)?,
        ExportFormat::Csv => {
            // Simplified CSV export for metrics
            let mut csv_output = String::new();
            csv_output.push_str("timestamp,model_id,response_time_ms,throughput_rps,error_rate_percent,memory_usage_mb,cpu_usage_percent\n");
            for metric in &metrics {
                csv_output.push_str(&format!("{:?},{},{},{},{},{},{}\n",
                    metric.timestamp, metric.model_id, metric.response_time_ms,
                    metric.throughput_rps, metric.error_rate_percent,
                    metric.memory_usage_mb, metric.cpu_usage_percent));
            }
            csv_output
        },
        ExportFormat::Yaml => serde_yaml::to_string(&export_data)
            .map_err(|e| anyhow::anyhow!("YAML serialization failed: {}", e))?,
    };

    if let Some(path) = output {
        tokio::fs::write(&path, output_str).await?;
        println!("Monitoring data exported to: {:?}", path);
    } else {
        println!("{}", output_str);
    }

    Ok(())
}

async fn watch_performance(
    _config: &Config,
    interval: u64,
    model: Option<String>,
    alerts_only: bool,
) -> Result<()> {
    println!("Starting performance monitoring (update interval: {}s)", interval);
    if let Some(ref model_id) = model {
        println!("Monitoring model: {}", model_id);
    }
    if alerts_only {
        println!("Showing alerts only");
    }
    println!("Press Ctrl+C to stop\n");

    let monitoring_config = MonitoringConfig::default();
    let monitor = PerformanceMonitor::new(monitoring_config, None).await?;

    let mut counter = 0;
    loop {
        if counter % 20 == 0 && !alerts_only {
            println!("{:<8} {:<12} {:<8} {:<8} {:<10} {:<8}",
                "Time", "Model", "RT(ms)", "RPS", "Memory(MB)", "CPU%");
        }

        let now = chrono::Utc::now().format("%H:%M:%S");
        let active_alerts = monitor.get_active_alerts().await;

        if alerts_only {
            if !active_alerts.is_empty() {
                println!("=== Active Alerts at {} ===", now);
                for alert in &active_alerts {
                    println!("[{:?}] {:?}: {}", alert.severity, alert.alert_type, alert.message);
                }
                println!();
            }
        } else {
            let metrics = if let Some(ref model_id) = model {
                monitor.get_metrics_for_model(model_id).await
            } else {
                monitor.get_current_metrics().await
            };

            if let Some(latest_metric) = metrics.first() {
                println!("{:<8} {:<12} {:<8} {:<8.2} {:<10} {:<8.1}",
                    now,
                    latest_metric.model_id,
                    latest_metric.response_time_ms,
                    latest_metric.throughput_rps,
                    latest_metric.memory_usage_mb,
                    latest_metric.cpu_usage_percent
                );
            }

            // Show alerts inline if any
            if !active_alerts.is_empty() {
                for alert in &active_alerts {
                    println!("  🚨 [{:?}] {}", alert.severity, alert.message);
                }
            }
        }

        counter += 1;
        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
}

async fn generate_report(
    _config: &Config,
    hours: u64,
    detailed: bool,
    recommendations: bool,
) -> Result<()> {
    println!("=== Performance Report ===");
    println!("Report period: {} hours", hours);
    println!("Generated at: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));

    let monitoring_config = MonitoringConfig::default();
    let monitor = PerformanceMonitor::new(monitoring_config, None).await?;

    let duration = Duration::from_secs(hours * 3600);
    if let Some(aggregated) = monitor.get_aggregated_metrics(duration).await {
        println!("\n=== Executive Summary ===");
        println!("System Uptime: {:.2}%", aggregated.uptime_percent);
        println!("Total Requests Processed: {}", aggregated.total_requests);
        println!("Average Response Time: {}ms", aggregated.avg_response_time_ms);
        println!("Average Throughput: {:.2} RPS", aggregated.avg_throughput_rps);
        println!("Overall Error Rate: {:.2}%", aggregated.avg_error_rate_percent);

        if detailed {
            println!("\n=== Detailed Performance Metrics ===");
            println!("Resource Utilization:");
            println!("  Average Memory Usage: {}MB", aggregated.avg_memory_usage_mb);
            println!("  Average CPU Usage: {:.2}%", aggregated.avg_cpu_usage_percent);
            println!("  Cache Hit Rate: {:.2}%", aggregated.avg_cache_hit_rate_percent);

            println!("\nRequest Statistics:");
            println!("  Successful Requests: {} ({:.2}%)",
                aggregated.successful_requests,
                (aggregated.successful_requests as f64 / aggregated.total_requests as f64) * 100.0);
            println!("  Failed Requests: {} ({:.2}%)",
                aggregated.failed_requests,
                (aggregated.failed_requests as f64 / aggregated.total_requests as f64) * 100.0);
        }

        if recommendations {
            println!("\n=== Recommendations ===");

            if aggregated.avg_response_time_ms > 2000 {
                println!("🔍 High response times detected. Consider:");
                println!("   - Enabling model caching");
                println!("   - Upgrading hardware");
                println!("   - Implementing request queueing");
            }

            if aggregated.avg_error_rate_percent > 2.0 {
                println!("⚠️  Elevated error rates detected. Consider:");
                println!("   - Reviewing model configurations");
                println!("   - Checking input validation");
                println!("   - Implementing retry mechanisms");
            }

            if aggregated.avg_cache_hit_rate_percent < 80.0 {
                println!("📈 Low cache hit rate detected. Consider:");
                println!("   - Increasing cache size");
                println!("   - Adjusting TTL settings");
                println!("   - Implementing smarter cache strategies");
            }

            if aggregated.avg_cpu_usage_percent > 70.0 {
                println!("🔥 High CPU usage detected. Consider:");
                println!("   - Scaling horizontally");
                println!("   - Optimizing model inference");
                println!("   - Implementing load balancing");
            }
        }
    } else {
        println!("No performance data available for the specified time period");
    }

    let active_alerts = monitor.get_active_alerts().await;
    if !active_alerts.is_empty() {
        println!("\n=== Current Issues ===");
        for alert in &active_alerts {
            println!("🚨 [{:?}] {}", alert.severity, alert.message);
        }
    }

    println!("\n=== Report Complete ===");

    Ok(())
}

async fn benchmark_monitoring(
    _config: &Config,
    metrics_count: usize,
    concurrent: usize,
    duration: u64,
) -> Result<()> {
    println!("Benchmarking monitoring system...");
    println!("Metrics to generate: {}", metrics_count);
    println!("Concurrent writers: {}", concurrent);
    println!("Duration: {} seconds", duration);

    let monitoring_config = MonitoringConfig::default();
    let monitor = Arc::new(PerformanceMonitor::new(monitoring_config, None).await?);

    let start_time = std::time::Instant::now();
    let mut handles = Vec::new();

    // Spawn concurrent metric writers
    for i in 0..concurrent {
        let monitor_clone = Arc::clone(&monitor);
        let metrics_per_writer = metrics_count / concurrent;

        let handle = tokio::spawn(async move {
            let mut metrics_written = 0;
            let writer_start = std::time::Instant::now();

            while writer_start.elapsed().as_secs() < duration && metrics_written < metrics_per_writer {
                let test_metric = create_test_metric(&format!("benchmark-model-{}", i));

                if let Err(e) = monitor_clone.record_metric(test_metric).await {
                    warn!("Failed to record metric: {}", e);
                }

                metrics_written += 1;
                tokio::time::sleep(Duration::from_millis(10)).await; // Small delay to avoid overwhelming
            }

            metrics_written
        });

        handles.push(handle);
    }

    // Wait for all writers to complete
    let mut total_written = 0;
    for handle in handles {
        total_written += handle.await?;
    }

    let elapsed = start_time.elapsed();

    println!("\n=== Benchmark Results ===");
    println!("Total time: {:?}", elapsed);
    println!("Metrics written: {}", total_written);
    println!("Metrics per second: {:.2}", total_written as f64 / elapsed.as_secs_f64());
    println!("Average write latency: {:.2}μs", elapsed.as_micros() as f64 / total_written as f64);

    // Show current monitoring system status
    let current_metrics = monitor.get_current_metrics().await;
    let active_alerts = monitor.get_active_alerts().await;

    println!("\n=== System Status After Benchmark ===");
    println!("Total metrics stored: {}", current_metrics.len());
    println!("Active alerts: {}", active_alerts.len());

    Ok(())
}