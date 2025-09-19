use crate::{config::Config, InfernoError};
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, instrument, warn};

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Enable Prometheus metrics export
    pub prometheus_enabled: bool,
    /// Prometheus metrics endpoint
    pub prometheus_endpoint: String,
    /// Prometheus scrape interval in seconds
    pub prometheus_scrape_interval: u64,

    /// Enable OpenTelemetry tracing
    pub otel_enabled: bool,
    /// OpenTelemetry collector endpoint
    pub otel_endpoint: String,
    /// OpenTelemetry service name
    pub otel_service_name: String,
    /// OpenTelemetry sampling ratio (0.0 to 1.0)
    pub otel_sampling_ratio: f64,

    /// Enable Grafana integration
    pub grafana_enabled: bool,
    /// Grafana API endpoint
    pub grafana_endpoint: String,
    /// Grafana API key
    pub grafana_api_key: Option<String>,

    /// Custom metrics configuration
    pub custom_metrics_enabled: bool,
    /// Metrics retention period in hours
    pub metrics_retention_hours: u64,
    /// Enable histogram metrics
    pub histogram_enabled: bool,
    /// Histogram bucket configuration
    pub histogram_buckets: Vec<f64>,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            prometheus_enabled: true,
            prometheus_endpoint: "/metrics".to_string(),
            prometheus_scrape_interval: 15,

            otel_enabled: false,
            otel_endpoint: "http://localhost:4317".to_string(),
            otel_service_name: "inferno".to_string(),
            otel_sampling_ratio: 1.0,

            grafana_enabled: false,
            grafana_endpoint: "http://localhost:3000".to_string(),
            grafana_api_key: None,

            custom_metrics_enabled: true,
            metrics_retention_hours: 24,
            histogram_enabled: true,
            histogram_buckets: vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        }
    }
}

/// Metric types for Prometheus
#[derive(Debug, Clone, Copy)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Individual metric data
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub help: String,
    pub metric_type: MetricType,
    pub value: MetricValue,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// Metric value types
#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Summary {
        sum: f64,
        count: u64,
        quantiles: Vec<(f64, f64)>,
    },
}

/// Prometheus metrics collector
pub struct PrometheusCollector {
    metrics: Arc<RwLock<HashMap<String, Metric>>>,
    config: ObservabilityConfig,
}

impl PrometheusCollector {
    pub fn new(config: ObservabilityConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Register a new metric
    pub async fn register_metric(
        &self,
        name: String,
        help: String,
        metric_type: MetricType,
        labels: HashMap<String, String>,
    ) -> Result<()> {
        let metric = Metric {
            name: name.clone(),
            help,
            metric_type,
            value: match metric_type {
                MetricType::Counter => MetricValue::Counter(0),
                MetricType::Gauge => MetricValue::Gauge(0.0),
                MetricType::Histogram => MetricValue::Histogram(vec![]),
                MetricType::Summary => MetricValue::Summary {
                    sum: 0.0,
                    count: 0,
                    quantiles: vec![(0.5, 0.0), (0.9, 0.0), (0.99, 0.0)],
                },
            },
            labels,
            timestamp: Utc::now(),
        };

        let mut metrics = self.metrics.write().await;
        metrics.insert(name, metric);
        Ok(())
    }

    /// Increment a counter metric
    pub async fn increment_counter(&self, name: &str, increment: u64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(name) {
            if let MetricValue::Counter(ref mut value) = metric.value {
                *value += increment;
                metric.timestamp = Utc::now();
            }
        }
        Ok(())
    }

    /// Set a gauge metric
    pub async fn set_gauge(&self, name: &str, value: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(name) {
            if let MetricValue::Gauge(ref mut v) = metric.value {
                *v = value;
                metric.timestamp = Utc::now();
            }
        }
        Ok(())
    }

    /// Observe a value for histogram
    pub async fn observe_histogram(&self, name: &str, value: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(name) {
            if let MetricValue::Histogram(ref mut values) = metric.value {
                values.push(value);
                metric.timestamp = Utc::now();

                // Keep only recent values to prevent unbounded growth
                if values.len() > 10000 {
                    values.drain(0..1000);
                }
            }
        }
        Ok(())
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus_format(&self) -> String {
        let metrics = self.metrics.read().await;
        let mut output = String::new();

        for metric in metrics.values() {
            // Write help text
            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));

            // Write type
            let type_str = match metric.metric_type {
                MetricType::Counter => "counter",
                MetricType::Gauge => "gauge",
                MetricType::Histogram => "histogram",
                MetricType::Summary => "summary",
            };
            output.push_str(&format!("# TYPE {} {}\n", metric.name, type_str));

            // Write metric value with labels
            let labels_str = if metric.labels.is_empty() {
                String::new()
            } else {
                let labels: Vec<String> = metric.labels
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();
                format!("{{{}}}", labels.join(","))
            };

            match &metric.value {
                MetricValue::Counter(value) => {
                    output.push_str(&format!("{}{} {}\n", metric.name, labels_str, value));
                }
                MetricValue::Gauge(value) => {
                    output.push_str(&format!("{}{} {}\n", metric.name, labels_str, value));
                }
                MetricValue::Histogram(values) => {
                    if !values.is_empty() {
                        // Calculate histogram buckets
                        for bucket in &self.config.histogram_buckets {
                            let count = values.iter().filter(|&&v| v <= *bucket).count();
                            output.push_str(&format!(
                                "{}_bucket{{le=\"{}\"{}}} {}\n",
                                metric.name,
                                bucket,
                                if labels_str.is_empty() { String::new() } else { format!(",{}", &labels_str[1..labels_str.len()-1]) },
                                count
                            ));
                        }
                        output.push_str(&format!(
                            "{}_bucket{{le=\"+Inf\"{}}} {}\n",
                            metric.name,
                            if labels_str.is_empty() { String::new() } else { format!(",{}", &labels_str[1..labels_str.len()-1]) },
                            values.len()
                        ));

                        let sum: f64 = values.iter().sum();
                        output.push_str(&format!("{}_sum{} {}\n", metric.name, labels_str, sum));
                        output.push_str(&format!("{}_count{} {}\n", metric.name, labels_str, values.len()));
                    }
                }
                MetricValue::Summary { sum, count, quantiles } => {
                    for (quantile, value) in quantiles {
                        output.push_str(&format!(
                            "{}{{quantile=\"{}\"{}}} {}\n",
                            metric.name,
                            quantile,
                            if labels_str.is_empty() { String::new() } else { format!(",{}", &labels_str[1..labels_str.len()-1]) },
                            value
                        ));
                    }
                    output.push_str(&format!("{}_sum{} {}\n", metric.name, labels_str, sum));
                    output.push_str(&format!("{}_count{} {}\n", metric.name, labels_str, count));
                }
            }
        }

        output
    }
}

/// OpenTelemetry trace span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<f64>,
    pub status: SpanStatus,
    pub attributes: HashMap<String, String>,
    pub events: Vec<SpanEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanStatus {
    Unset,
    Ok,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub attributes: HashMap<String, String>,
}

/// OpenTelemetry tracer
pub struct OpenTelemetryTracer {
    spans: Arc<Mutex<Vec<TraceSpan>>>,
    config: ObservabilityConfig,
}

impl OpenTelemetryTracer {
    pub fn new(config: ObservabilityConfig) -> Self {
        Self {
            spans: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    /// Start a new trace span
    pub async fn start_span(&self, operation_name: String) -> String {
        let span = TraceSpan {
            trace_id: uuid::Uuid::new_v4().to_string(),
            span_id: uuid::Uuid::new_v4().to_string(),
            parent_span_id: None,
            operation_name,
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            status: SpanStatus::Unset,
            attributes: HashMap::new(),
            events: Vec::new(),
        };

        let span_id = span.span_id.clone();
        let mut spans = self.spans.lock().await;
        spans.push(span);

        // Keep only recent spans
        if spans.len() > 1000 {
            spans.drain(0..100);
        }

        span_id
    }

    /// End a trace span
    pub async fn end_span(&self, span_id: &str, status: SpanStatus) -> Result<()> {
        let mut spans = self.spans.lock().await;
        if let Some(span) = spans.iter_mut().find(|s| s.span_id == span_id) {
            span.end_time = Some(Utc::now());
            span.duration_ms = Some(
                (span.end_time.expect("End time should be set just above") - span.start_time).num_milliseconds() as f64
            );
            span.status = status;
        }
        Ok(())
    }

    /// Add attributes to a span
    pub async fn add_span_attributes(
        &self,
        span_id: &str,
        attributes: HashMap<String, String>,
    ) -> Result<()> {
        let mut spans = self.spans.lock().await;
        if let Some(span) = spans.iter_mut().find(|s| s.span_id == span_id) {
            span.attributes.extend(attributes);
        }
        Ok(())
    }

    /// Add event to a span
    pub async fn add_span_event(
        &self,
        span_id: &str,
        event_name: String,
        attributes: HashMap<String, String>,
    ) -> Result<()> {
        let mut spans = self.spans.lock().await;
        if let Some(span) = spans.iter_mut().find(|s| s.span_id == span_id) {
            span.events.push(SpanEvent {
                name: event_name,
                timestamp: Utc::now(),
                attributes,
            });
        }
        Ok(())
    }

    /// Export spans in OTLP format (simplified)
    pub async fn export_otlp_format(&self) -> Vec<TraceSpan> {
        let spans = self.spans.lock().await;
        spans.clone()
    }
}

/// Grafana dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaDashboard {
    pub id: String,
    pub title: String,
    pub panels: Vec<DashboardPanel>,
    pub refresh_interval: String,
    pub time_range: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPanel {
    pub id: u32,
    pub title: String,
    pub panel_type: String, // graph, stat, gauge, table
    pub datasource: String,
    pub query: String,
    pub grid_pos: GridPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridPosition {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Observability manager combining all observability features
pub struct ObservabilityManager {
    config: ObservabilityConfig,
    prometheus: Arc<PrometheusCollector>,
    tracer: Arc<OpenTelemetryTracer>,
    dashboards: Arc<RwLock<Vec<GrafanaDashboard>>>,
}

impl ObservabilityManager {
    pub fn new(config: ObservabilityConfig) -> Self {
        let prometheus = Arc::new(PrometheusCollector::new(config.clone()));
        let tracer = Arc::new(OpenTelemetryTracer::new(config.clone()));

        Self {
            config,
            prometheus,
            tracer,
            dashboards: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Initialize default metrics
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing observability stack");

        // Register default Prometheus metrics
        if self.config.prometheus_enabled {
            // System metrics
            self.prometheus.register_metric(
                "inferno_up".to_string(),
                "Whether the Inferno service is up".to_string(),
                MetricType::Gauge,
                HashMap::new(),
            ).await?;

            // Inference metrics
            self.prometheus.register_metric(
                "inferno_inference_requests_total".to_string(),
                "Total number of inference requests".to_string(),
                MetricType::Counter,
                HashMap::from([("model".to_string(), "all".to_string())]),
            ).await?;

            self.prometheus.register_metric(
                "inferno_inference_duration_seconds".to_string(),
                "Inference request duration in seconds".to_string(),
                MetricType::Histogram,
                HashMap::new(),
            ).await?;

            // Model metrics
            self.prometheus.register_metric(
                "inferno_models_loaded".to_string(),
                "Number of models currently loaded".to_string(),
                MetricType::Gauge,
                HashMap::new(),
            ).await?;

            self.prometheus.register_metric(
                "inferno_model_memory_bytes".to_string(),
                "Memory usage per model in bytes".to_string(),
                MetricType::Gauge,
                HashMap::new(),
            ).await?;

            // Error metrics
            self.prometheus.register_metric(
                "inferno_errors_total".to_string(),
                "Total number of errors".to_string(),
                MetricType::Counter,
                HashMap::from([("type".to_string(), "all".to_string())]),
            ).await?;

            // Set service up
            self.prometheus.set_gauge("inferno_up", 1.0).await?;
        }

        // Create default Grafana dashboard
        if self.config.grafana_enabled {
            self.create_default_dashboard().await?;
        }

        Ok(())
    }

    /// Create default Grafana dashboard
    async fn create_default_dashboard(&self) -> Result<()> {
        let dashboard = GrafanaDashboard {
            id: "inferno-default".to_string(),
            title: "Inferno AI/ML Model Runner".to_string(),
            refresh_interval: "5s".to_string(),
            time_range: "now-1h".to_string(),
            panels: vec![
                DashboardPanel {
                    id: 1,
                    title: "Service Status".to_string(),
                    panel_type: "stat".to_string(),
                    datasource: "Prometheus".to_string(),
                    query: "inferno_up".to_string(),
                    grid_pos: GridPosition { x: 0, y: 0, w: 6, h: 4 },
                },
                DashboardPanel {
                    id: 2,
                    title: "Request Rate".to_string(),
                    panel_type: "graph".to_string(),
                    datasource: "Prometheus".to_string(),
                    query: "rate(inferno_inference_requests_total[5m])".to_string(),
                    grid_pos: GridPosition { x: 6, y: 0, w: 12, h: 8 },
                },
                DashboardPanel {
                    id: 3,
                    title: "Response Time".to_string(),
                    panel_type: "graph".to_string(),
                    datasource: "Prometheus".to_string(),
                    query: "histogram_quantile(0.95, rate(inferno_inference_duration_seconds_bucket[5m]))".to_string(),
                    grid_pos: GridPosition { x: 18, y: 0, w: 6, h: 8 },
                },
                DashboardPanel {
                    id: 4,
                    title: "Models Loaded".to_string(),
                    panel_type: "gauge".to_string(),
                    datasource: "Prometheus".to_string(),
                    query: "inferno_models_loaded".to_string(),
                    grid_pos: GridPosition { x: 0, y: 4, w: 6, h: 4 },
                },
                DashboardPanel {
                    id: 5,
                    title: "Error Rate".to_string(),
                    panel_type: "graph".to_string(),
                    datasource: "Prometheus".to_string(),
                    query: "rate(inferno_errors_total[5m])".to_string(),
                    grid_pos: GridPosition { x: 6, y: 8, w: 12, h: 8 },
                },
                DashboardPanel {
                    id: 6,
                    title: "Memory Usage".to_string(),
                    panel_type: "graph".to_string(),
                    datasource: "Prometheus".to_string(),
                    query: "sum(inferno_model_memory_bytes) / 1024 / 1024 / 1024".to_string(),
                    grid_pos: GridPosition { x: 18, y: 8, w: 6, h: 8 },
                },
            ],
        };

        let mut dashboards = self.dashboards.write().await;
        dashboards.push(dashboard);

        Ok(())
    }

    /// Record an inference request
    #[instrument(skip(self))]
    pub async fn record_inference(
        &self,
        model: &str,
        duration: Duration,
        success: bool,
    ) -> Result<()> {
        if self.config.prometheus_enabled {
            // Increment request counter
            self.prometheus.increment_counter("inferno_inference_requests_total", 1).await?;

            // Record duration histogram
            self.prometheus.observe_histogram(
                "inferno_inference_duration_seconds",
                duration.as_secs_f64(),
            ).await?;

            // Record error if failed
            if !success {
                self.prometheus.increment_counter("inferno_errors_total", 1).await?;
            }
        }

        if self.config.otel_enabled {
            // Create trace span
            let span_id = self.tracer.start_span(format!("inference.{}", model)).await;

            // Add attributes
            let mut attributes = HashMap::new();
            attributes.insert("model".to_string(), model.to_string());
            attributes.insert("duration_ms".to_string(), duration.as_millis().to_string());
            attributes.insert("success".to_string(), success.to_string());

            self.tracer.add_span_attributes(&span_id, attributes).await?;

            // End span
            let status = if success {
                SpanStatus::Ok
            } else {
                SpanStatus::Error("Inference failed".to_string())
            };
            self.tracer.end_span(&span_id, status).await?;
        }

        Ok(())
    }

    /// Get Prometheus metrics
    pub async fn get_prometheus_metrics(&self) -> String {
        self.prometheus.export_prometheus_format().await
    }

    /// Get OpenTelemetry traces
    pub async fn get_traces(&self) -> Vec<TraceSpan> {
        self.tracer.export_otlp_format().await
    }

    /// Get Grafana dashboards
    pub async fn get_dashboards(&self) -> Vec<GrafanaDashboard> {
        let dashboards = self.dashboards.read().await;
        dashboards.clone()
    }

    /// Export dashboard as JSON
    pub async fn export_dashboard_json(&self, dashboard_id: &str) -> Result<String> {
        let dashboards = self.dashboards.read().await;

        if let Some(dashboard) = dashboards.iter().find(|d| d.id == dashboard_id) {
            Ok(serde_json::to_string_pretty(dashboard)?)
        } else {
            Err(InfernoError::ModelNotFound(format!("Dashboard {} not found", dashboard_id)).into())
        }
    }
}

/// Create observability routes for Axum server
pub fn create_observability_routes(manager: Arc<ObservabilityManager>) -> Router {
    Router::new()
        .route("/metrics", get(prometheus_metrics_handler))
        .route("/traces", get(traces_handler))
        .route("/dashboards", get(dashboards_handler))
        .route("/dashboards/:id", get(dashboard_handler))
        .with_state(manager)
}

/// Prometheus metrics endpoint handler
async fn prometheus_metrics_handler(
    State(manager): State<Arc<ObservabilityManager>>,
) -> impl IntoResponse {
    let metrics = manager.get_prometheus_metrics().await;
    (StatusCode::OK, metrics)
}

/// OpenTelemetry traces endpoint handler
async fn traces_handler(
    State(manager): State<Arc<ObservabilityManager>>,
) -> impl IntoResponse {
    let traces = manager.get_traces().await;
    (StatusCode::OK, serde_json::to_string(&traces).unwrap_or_default())
}

/// Grafana dashboards list handler
async fn dashboards_handler(
    State(manager): State<Arc<ObservabilityManager>>,
) -> impl IntoResponse {
    let dashboards = manager.get_dashboards().await;
    (StatusCode::OK, serde_json::to_string(&dashboards).unwrap_or_default())
}

/// Individual dashboard handler
async fn dashboard_handler(
    State(manager): State<Arc<ObservabilityManager>>,
    axum::extract::Path(dashboard_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match manager.export_dashboard_json(&dashboard_id).await {
        Ok(json) => (StatusCode::OK, json),
        Err(_) => (StatusCode::NOT_FOUND, "Dashboard not found".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prometheus_collector() {
        let config = ObservabilityConfig::default();
        let collector = PrometheusCollector::new(config);

        // Register metrics
        collector.register_metric(
            "test_counter".to_string(),
            "Test counter metric".to_string(),
            MetricType::Counter,
            HashMap::new(),
        ).await.expect("Failed to register metric in test");

        collector.register_metric(
            "test_gauge".to_string(),
            "Test gauge metric".to_string(),
            MetricType::Gauge,
            HashMap::new(),
        ).await.expect("Failed to register metric in test");

        // Update metrics
        collector.increment_counter("test_counter", 5).await.expect("Failed to increment counter in test");
        collector.set_gauge("test_gauge", 42.5).await.expect("Failed to set gauge in test");

        // Export
        let output = collector.export_prometheus_format().await;
        assert!(output.contains("test_counter 5"));
        assert!(output.contains("test_gauge 42.5"));
    }

    #[tokio::test]
    async fn test_opentelemetry_tracer() {
        let config = ObservabilityConfig::default();
        let tracer = OpenTelemetryTracer::new(config);

        // Start span
        let span_id = tracer.start_span("test_operation".to_string()).await;

        // Add attributes
        let mut attributes = HashMap::new();
        attributes.insert("test_key".to_string(), "test_value".to_string());
        tracer.add_span_attributes(&span_id, attributes).await.expect("Failed to add span attributes in test");

        // Add event
        tracer.add_span_event(
            &span_id,
            "test_event".to_string(),
            HashMap::new(),
        ).await.expect("Failed to register metric in test");

        // End span
        tracer.end_span(&span_id, SpanStatus::Ok).await.expect("Failed to end span in test");

        // Export
        let spans = tracer.export_otlp_format().await;
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].operation_name, "test_operation");
    }

    #[tokio::test]
    async fn test_observability_manager() {
        let config = ObservabilityConfig::default();
        let manager = ObservabilityManager::new(config);

        // Initialize
        manager.initialize().await.expect("Failed to initialize observability manager in test");

        // Record inference
        manager.record_inference(
            "test_model",
            Duration::from_millis(100),
            true,
        ).await.expect("Failed to register metric in test");

        // Get metrics
        let metrics = manager.get_prometheus_metrics().await;
        assert!(metrics.contains("inferno_up 1"));
        assert!(metrics.contains("inferno_inference_requests_total"));
    }
}