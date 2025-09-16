use crate::{
    config::Config,
    metrics::MetricsCollector,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};
use tokio::{
    sync::{Mutex, RwLock},
    time::interval,
};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub collection_interval_ms: u64,
    pub alert_evaluation_interval_ms: u64,
    pub metric_retention_hours: u64,
    pub performance_thresholds: PerformanceThresholds,
    pub alerting: AlertingConfig,
    pub dashboards: DashboardConfig,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_ms: 1000,
            alert_evaluation_interval_ms: 5000,
            metric_retention_hours: 24,
            performance_thresholds: PerformanceThresholds::default(),
            alerting: AlertingConfig::default(),
            dashboards: DashboardConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub max_response_time_ms: u64,
    pub min_throughput_rps: f64,
    pub max_error_rate_percent: f64,
    pub max_memory_usage_mb: u64,
    pub max_cpu_usage_percent: f64,
    pub max_queue_depth: usize,
    pub min_cache_hit_rate_percent: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_response_time_ms: 5000,
            min_throughput_rps: 1.0,
            max_error_rate_percent: 5.0,
            max_memory_usage_mb: 8192,
            max_cpu_usage_percent: 80.0,
            max_queue_depth: 100,
            min_cache_hit_rate_percent: 70.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    pub enabled: bool,
    pub webhooks: Vec<WebhookConfig>,
    pub email: Option<EmailConfig>,
    pub slack: Option<SlackConfig>,
    pub cooldown_minutes: u64,
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            webhooks: Vec::new(),
            email: None,
            slack: None,
            cooldown_minutes: 15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub name: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub to_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub webhook_url: String,
    pub channel: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub enabled: bool,
    pub bind_address: String,
    pub port: u16,
    pub update_interval_ms: u64,
    pub max_data_points: usize,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind_address: "127.0.0.1".to_string(),
            port: 3000,
            update_interval_ms: 1000,
            max_data_points: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub timestamp: SystemTime,
    pub model_id: String,
    pub response_time_ms: u64,
    pub throughput_rps: f64,
    pub error_rate_percent: f64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub queue_depth: usize,
    pub cache_hit_rate_percent: f64,
    pub active_connections: usize,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub model_id: Option<String>,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub timestamp: SystemTime,
    pub resolved: bool,
    pub resolved_at: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighResponseTime,
    LowThroughput,
    HighErrorRate,
    HighMemoryUsage,
    HighCpuUsage,
    HighQueueDepth,
    LowCacheHitRate,
    ModelUnavailable,
    SystemDown,
    DiskSpaceLow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug)]
pub struct PerformanceMonitor {
    config: MonitoringConfig,
    metrics: Arc<RwLock<VecDeque<PerformanceMetric>>>,
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    alert_history: Arc<RwLock<VecDeque<Alert>>>,
    last_alert_times: Arc<RwLock<HashMap<String, SystemTime>>>,
    metrics_collector: Option<Arc<MetricsCollector>>,
    background_tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl PerformanceMonitor {
    pub async fn new(
        config: MonitoringConfig,
        metrics_collector: Option<Arc<MetricsCollector>>,
    ) -> Result<Self> {
        let metrics = Arc::new(RwLock::new(VecDeque::new()));
        let active_alerts = Arc::new(RwLock::new(HashMap::new()));
        let alert_history = Arc::new(RwLock::new(VecDeque::new()));
        let last_alert_times = Arc::new(RwLock::new(HashMap::new()));

        let mut monitor = Self {
            config,
            metrics,
            active_alerts,
            alert_history,
            last_alert_times,
            metrics_collector,
            background_tasks: Vec::new(),
        };

        if monitor.config.enabled {
            monitor.start_background_monitoring().await?;
        }

        Ok(monitor)
    }

    pub async fn record_metric(&self, metric: PerformanceMetric) -> Result<()> {
        let mut metrics = self.metrics.write().await;

        // Add new metric
        metrics.push_back(metric.clone());

        // Trim old metrics based on retention policy
        let retention_duration = Duration::from_secs(self.config.metric_retention_hours * 3600);
        let cutoff_time = SystemTime::now() - retention_duration;

        while let Some(front_metric) = metrics.front() {
            if front_metric.timestamp < cutoff_time {
                metrics.pop_front();
            } else {
                break;
            }
        }

        // Limit total metrics to prevent excessive memory usage
        if metrics.len() > 100000 {
            metrics.pop_front();
        }

        // Evaluate alerts for this metric
        self.evaluate_alerts(&metric).await?;

        debug!("Recorded performance metric for model: {}", metric.model_id);
        Ok(())
    }

    pub async fn get_current_metrics(&self) -> Vec<PerformanceMetric> {
        let metrics = self.metrics.read().await;
        metrics.iter().rev().take(100).cloned().collect()
    }

    pub async fn get_metrics_for_model(&self, model_id: &str) -> Vec<PerformanceMetric> {
        let metrics = self.metrics.read().await;
        metrics
            .iter()
            .filter(|m| m.model_id == model_id)
            .rev()
            .take(100)
            .cloned()
            .collect()
    }

    pub async fn get_aggregated_metrics(&self, duration: Duration) -> Option<AggregatedMetrics> {
        let metrics = self.metrics.read().await;
        let cutoff_time = SystemTime::now() - duration;

        let recent_metrics: Vec<_> = metrics
            .iter()
            .filter(|m| m.timestamp >= cutoff_time)
            .collect();

        if recent_metrics.is_empty() {
            return None;
        }

        let total_metrics = recent_metrics.len() as f64;
        let avg_response_time = recent_metrics.iter().map(|m| m.response_time_ms as f64).sum::<f64>() / total_metrics;
        let avg_throughput = recent_metrics.iter().map(|m| m.throughput_rps).sum::<f64>() / total_metrics;
        let avg_error_rate = recent_metrics.iter().map(|m| m.error_rate_percent).sum::<f64>() / total_metrics;
        let avg_memory_usage = recent_metrics.iter().map(|m| m.memory_usage_mb as f64).sum::<f64>() / total_metrics;
        let avg_cpu_usage = recent_metrics.iter().map(|m| m.cpu_usage_percent).sum::<f64>() / total_metrics;
        let avg_cache_hit_rate = recent_metrics.iter().map(|m| m.cache_hit_rate_percent).sum::<f64>() / total_metrics;

        let total_requests = recent_metrics.iter().map(|m| m.total_requests).sum();
        let successful_requests = recent_metrics.iter().map(|m| m.successful_requests).sum();
        let failed_requests = recent_metrics.iter().map(|m| m.failed_requests).sum();

        Some(AggregatedMetrics {
            duration,
            avg_response_time_ms: avg_response_time as u64,
            avg_throughput_rps: avg_throughput,
            avg_error_rate_percent: avg_error_rate,
            avg_memory_usage_mb: avg_memory_usage as u64,
            avg_cpu_usage_percent: avg_cpu_usage,
            avg_cache_hit_rate_percent: avg_cache_hit_rate,
            total_requests,
            successful_requests,
            failed_requests,
            uptime_percent: if total_requests > 0 {
                (successful_requests as f64 / total_requests as f64) * 100.0
            } else {
                100.0
            },
        })
    }

    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.active_alerts.read().await;
        alerts.values().cloned().collect()
    }

    pub async fn get_alert_history(&self, limit: Option<usize>) -> Vec<Alert> {
        let history = self.alert_history.read().await;
        match limit {
            Some(n) => history.iter().rev().take(n).cloned().collect(),
            None => history.iter().rev().cloned().collect(),
        }
    }

    pub async fn resolve_alert(&self, alert_id: &str) -> Result<bool> {
        let mut active_alerts = self.active_alerts.write().await;

        if let Some(mut alert) = active_alerts.remove(alert_id) {
            alert.resolved = true;
            alert.resolved_at = Some(SystemTime::now());

            // Add to history
            let mut history = self.alert_history.write().await;
            history.push_back(alert);

            // Limit history size
            if history.len() > 1000 {
                history.pop_front();
            }

            info!("Resolved alert: {}", alert_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn evaluate_alerts(&self, metric: &PerformanceMetric) -> Result<()> {
        let thresholds = &self.config.performance_thresholds;
        let mut new_alerts = Vec::new();

        // Check response time
        if metric.response_time_ms > thresholds.max_response_time_ms {
            new_alerts.push((
                AlertType::HighResponseTime,
                AlertSeverity::Warning,
                format!("Response time {}ms exceeds threshold {}ms for model {}",
                        metric.response_time_ms, thresholds.max_response_time_ms, metric.model_id),
                metric.response_time_ms as f64,
                thresholds.max_response_time_ms as f64,
            ));
        }

        // Check throughput
        if metric.throughput_rps < thresholds.min_throughput_rps {
            new_alerts.push((
                AlertType::LowThroughput,
                AlertSeverity::Warning,
                format!("Throughput {:.2} RPS below threshold {:.2} RPS for model {}",
                        metric.throughput_rps, thresholds.min_throughput_rps, metric.model_id),
                metric.throughput_rps,
                thresholds.min_throughput_rps,
            ));
        }

        // Check error rate
        if metric.error_rate_percent > thresholds.max_error_rate_percent {
            new_alerts.push((
                AlertType::HighErrorRate,
                AlertSeverity::Critical,
                format!("Error rate {:.2}% exceeds threshold {:.2}% for model {}",
                        metric.error_rate_percent, thresholds.max_error_rate_percent, metric.model_id),
                metric.error_rate_percent,
                thresholds.max_error_rate_percent,
            ));
        }

        // Check memory usage
        if metric.memory_usage_mb > thresholds.max_memory_usage_mb {
            new_alerts.push((
                AlertType::HighMemoryUsage,
                AlertSeverity::Warning,
                format!("Memory usage {}MB exceeds threshold {}MB",
                        metric.memory_usage_mb, thresholds.max_memory_usage_mb),
                metric.memory_usage_mb as f64,
                thresholds.max_memory_usage_mb as f64,
            ));
        }

        // Check CPU usage
        if metric.cpu_usage_percent > thresholds.max_cpu_usage_percent {
            new_alerts.push((
                AlertType::HighCpuUsage,
                AlertSeverity::Warning,
                format!("CPU usage {:.2}% exceeds threshold {:.2}%",
                        metric.cpu_usage_percent, thresholds.max_cpu_usage_percent),
                metric.cpu_usage_percent,
                thresholds.max_cpu_usage_percent,
            ));
        }

        // Check queue depth
        if metric.queue_depth > thresholds.max_queue_depth {
            new_alerts.push((
                AlertType::HighQueueDepth,
                AlertSeverity::Warning,
                format!("Queue depth {} exceeds threshold {} for model {}",
                        metric.queue_depth, thresholds.max_queue_depth, metric.model_id),
                metric.queue_depth as f64,
                thresholds.max_queue_depth as f64,
            ));
        }

        // Check cache hit rate
        if metric.cache_hit_rate_percent < thresholds.min_cache_hit_rate_percent {
            new_alerts.push((
                AlertType::LowCacheHitRate,
                AlertSeverity::Info,
                format!("Cache hit rate {:.2}% below threshold {:.2}% for model {}",
                        metric.cache_hit_rate_percent, thresholds.min_cache_hit_rate_percent, metric.model_id),
                metric.cache_hit_rate_percent,
                thresholds.min_cache_hit_rate_percent,
            ));
        }

        // Process new alerts
        for (alert_type, severity, message, metric_value, threshold_value) in new_alerts {
            let alert_key = format!("{:?}_{}", alert_type, metric.model_id);

            // Check cooldown
            if self.is_in_cooldown(&alert_key).await {
                continue;
            }

            let alert_id = format!("{}_{}", alert_key, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs());

            let alert = Alert {
                id: alert_id.clone(),
                alert_type,
                severity,
                message: message.clone(),
                model_id: Some(metric.model_id.clone()),
                metric_value,
                threshold_value,
                timestamp: SystemTime::now(),
                resolved: false,
                resolved_at: None,
            };

            // Add to active alerts
            let mut active_alerts = self.active_alerts.write().await;
            active_alerts.insert(alert_id.clone(), alert.clone());

            // Update cooldown
            let mut last_alert_times = self.last_alert_times.write().await;
            last_alert_times.insert(alert_key, SystemTime::now());

            // Send notifications
            if self.config.alerting.enabled {
                self.send_alert_notifications(&alert).await?;
            }

            warn!("Alert triggered: {} - {}", alert_id, message);
        }

        Ok(())
    }

    async fn is_in_cooldown(&self, alert_key: &str) -> bool {
        let last_alert_times = self.last_alert_times.read().await;
        if let Some(last_time) = last_alert_times.get(alert_key) {
            let cooldown_duration = Duration::from_secs(self.config.alerting.cooldown_minutes * 60);
            SystemTime::now().duration_since(*last_time).unwrap_or(Duration::ZERO) < cooldown_duration
        } else {
            false
        }
    }

    async fn send_alert_notifications(&self, alert: &Alert) -> Result<()> {
        // Send webhook notifications
        for webhook in &self.config.alerting.webhooks {
            if let Err(e) = self.send_webhook_alert(webhook, alert).await {
                error!("Failed to send webhook alert to {}: {}", webhook.url, e);
            }
        }

        // Send email notifications
        if let Some(ref email_config) = self.config.alerting.email {
            if let Err(e) = self.send_email_alert(email_config, alert).await {
                error!("Failed to send email alert: {}", e);
            }
        }

        // Send Slack notifications
        if let Some(ref slack_config) = self.config.alerting.slack {
            if let Err(e) = self.send_slack_alert(slack_config, alert).await {
                error!("Failed to send Slack alert: {}", e);
            }
        }

        Ok(())
    }

    async fn send_webhook_alert(&self, webhook: &WebhookConfig, alert: &Alert) -> Result<()> {
        let payload = serde_json::json!({
            "alert_id": alert.id,
            "alert_type": alert.alert_type,
            "severity": alert.severity,
            "message": alert.message,
            "model_id": alert.model_id,
            "metric_value": alert.metric_value,
            "threshold_value": alert.threshold_value,
            "timestamp": alert.timestamp.duration_since(SystemTime::UNIX_EPOCH)?.as_secs()
        });

        info!("Sending webhook alert to {} for alert: {}", webhook.url, alert.id);
        // Note: In a real implementation, you would use reqwest or similar to send HTTP request
        debug!("Webhook payload: {}", payload);

        Ok(())
    }

    async fn send_email_alert(&self, _email_config: &EmailConfig, alert: &Alert) -> Result<()> {
        info!("Sending email alert for: {}", alert.id);
        // Note: In a real implementation, you would use lettre or similar crate
        debug!("Email alert: {}", alert.message);
        Ok(())
    }

    async fn send_slack_alert(&self, _slack_config: &SlackConfig, alert: &Alert) -> Result<()> {
        info!("Sending Slack alert for: {}", alert.id);
        // Note: In a real implementation, you would send to Slack webhook
        debug!("Slack alert: {}", alert.message);
        Ok(())
    }

    async fn start_background_monitoring(&mut self) -> Result<()> {
        // Start metrics collection task
        let metrics_handle = self.start_metrics_collection_task().await;
        self.background_tasks.push(metrics_handle);

        // Start alert evaluation task
        let alert_handle = self.start_alert_evaluation_task().await;
        self.background_tasks.push(alert_handle);

        // Start dashboard server if enabled
        if self.config.dashboards.enabled {
            let dashboard_handle = self.start_dashboard_server().await?;
            self.background_tasks.push(dashboard_handle);
        }

        info!("Started background monitoring tasks");
        Ok(())
    }

    async fn start_metrics_collection_task(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let metrics = Arc::clone(&self.metrics);
        let metrics_collector = self.metrics_collector.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(config.collection_interval_ms));

            loop {
                interval.tick().await;

                // Collect system metrics
                let metric = PerformanceMetric {
                    timestamp: SystemTime::now(),
                    model_id: "system".to_string(),
                    response_time_ms: 0, // Will be updated by actual inference calls
                    throughput_rps: 0.0, // Will be calculated
                    error_rate_percent: 0.0, // Will be calculated
                    memory_usage_mb: Self::get_memory_usage(),
                    cpu_usage_percent: Self::get_cpu_usage(),
                    queue_depth: 0, // Will be updated by queue monitoring
                    cache_hit_rate_percent: 0.0, // Will be updated by cache monitoring
                    active_connections: 0, // Will be updated by connection monitoring
                    total_requests: 0,
                    successful_requests: 0,
                    failed_requests: 0,
                };

                let mut metrics_guard = metrics.write().await;
                metrics_guard.push_back(metric);

                // Trim old metrics
                if metrics_guard.len() > 10000 {
                    metrics_guard.pop_front();
                }

                if let Some(ref collector) = metrics_collector {
                    // Record metrics in the collector as well
                    debug!("Collected system metrics");
                }
            }
        })
    }

    async fn start_alert_evaluation_task(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let active_alerts = Arc::clone(&self.active_alerts);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(config.alert_evaluation_interval_ms));

            loop {
                interval.tick().await;

                // Auto-resolve alerts that are no longer triggered
                let active_alerts_guard = active_alerts.read().await;
                let alert_count = active_alerts_guard.len();
                drop(active_alerts_guard);

                if alert_count > 0 {
                    debug!("Evaluating {} active alerts for auto-resolution", alert_count);
                }
            }
        })
    }

    async fn start_dashboard_server(&self) -> Result<tokio::task::JoinHandle<()>> {
        let config = self.config.dashboards.clone();
        let metrics = Arc::clone(&self.metrics);
        let active_alerts = Arc::clone(&self.active_alerts);

        let handle = tokio::spawn(async move {
            info!("Starting monitoring dashboard on {}:{}", config.bind_address, config.port);

            // Note: In a real implementation, you would start an HTTP server here
            // using axum, warp, or similar framework to serve the dashboard

            let mut interval = interval(Duration::from_millis(config.update_interval_ms));

            loop {
                interval.tick().await;

                let metrics_guard = metrics.read().await;
                let alerts_guard = active_alerts.read().await;

                debug!("Dashboard update: {} metrics, {} active alerts",
                       metrics_guard.len(), alerts_guard.len());
            }
        });

        Ok(handle)
    }

    fn get_memory_usage() -> u64 {
        // Note: In a real implementation, you would use sysinfo or similar
        // to get actual system memory usage
        use std::process;
        let pid = process::id();
        debug!("Getting memory usage for PID: {}", pid);
        1024 // Placeholder: 1GB
    }

    fn get_cpu_usage() -> f64 {
        // Note: In a real implementation, you would use sysinfo or similar
        // to get actual CPU usage
        25.0 // Placeholder: 25% CPU usage
    }

    pub async fn shutdown(&mut self) {
        info!("Shutting down performance monitor");

        for handle in &self.background_tasks {
            handle.abort();
        }

        self.background_tasks.clear();
    }
}

impl Drop for PerformanceMonitor {
    fn drop(&mut self) {
        for handle in &self.background_tasks {
            handle.abort();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub duration: Duration,
    pub avg_response_time_ms: u64,
    pub avg_throughput_rps: f64,
    pub avg_error_rate_percent: f64,
    pub avg_memory_usage_mb: u64,
    pub avg_cpu_usage_percent: f64,
    pub avg_cache_hit_rate_percent: f64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub uptime_percent: f64,
}

// Utility function to create test metrics for monitoring
pub fn create_test_metric(model_id: &str) -> PerformanceMetric {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    PerformanceMetric {
        timestamp: SystemTime::now(),
        model_id: model_id.to_string(),
        response_time_ms: rng.gen_range(100..2000),
        throughput_rps: rng.gen_range(1.0..10.0),
        error_rate_percent: rng.gen_range(0.0..5.0),
        memory_usage_mb: rng.gen_range(512..4096),
        cpu_usage_percent: rng.gen_range(10.0..80.0),
        queue_depth: rng.gen_range(0..50),
        cache_hit_rate_percent: rng.gen_range(60.0..95.0),
        active_connections: rng.gen_range(1..20),
        total_requests: rng.gen_range(100..1000),
        successful_requests: rng.gen_range(90..950),
        failed_requests: rng.gen_range(0..50),
    }
}