use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: u64,
    pub inference_metrics: InferenceMetrics,
    pub system_metrics: SystemMetrics,
    pub model_metrics: ModelMetrics,
    /// Custom counters from CLI commands and other sources
    #[serde(default)]
    pub custom_counters: HashMap<String, u64>,
    /// Custom gauges from CLI commands and other sources
    #[serde(default)]
    pub custom_gauges: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_tokens_generated: u64,
    pub total_inference_time_ms: u64,
    pub average_tokens_per_second: f64,
    pub average_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f32,
    pub gpu_memory_usage_bytes: Option<u64>,
    pub gpu_utilization_percent: Option<f32>,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub loaded_models: HashMap<String, ModelStats>,
    pub total_model_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStats {
    pub name: String,
    pub size_bytes: u64,
    pub load_time_ms: u64,
    pub inference_count: u64,
    pub total_inference_time_ms: u64,
    pub backend_type: String,
}

#[derive(Debug)]
pub struct InferenceEvent {
    pub model_name: String,
    pub input_length: u32,
    pub output_length: u32,
    pub duration: Duration,
    pub success: bool,
}

/// Handles async event processing for metrics collection.
/// This type owns the event receiver and is consumed when starting event processing.
/// Separated from MetricsCollector to maintain Send + Sync bounds on MetricsCollector.
#[derive(Debug)]
pub struct MetricsEventProcessor {
    receiver: mpsc::UnboundedReceiver<InferenceEvent>,
    counters: Arc<InferenceCounters>,
    model_stats: Arc<RwLock<HashMap<String, ModelStats>>>,
}

impl MetricsEventProcessor {
    /// Start processing metrics events. Consumes self and spawns a background task.
    pub fn start(mut self) {
        tokio::spawn(async move {
            while let Some(event) = self.receiver.recv().await {
                // Update global counters
                self.counters.total_requests.fetch_add(1, Ordering::Relaxed);

                if event.success {
                    self.counters
                        .successful_requests
                        .fetch_add(1, Ordering::Relaxed);
                    self.counters
                        .total_tokens_generated
                        .fetch_add(event.output_length as u64, Ordering::Relaxed);
                } else {
                    self.counters
                        .failed_requests
                        .fetch_add(1, Ordering::Relaxed);
                }

                self.counters
                    .total_inference_time_ms
                    .fetch_add(event.duration.as_millis() as u64, Ordering::Relaxed);

                // Update model-specific stats
                if let Ok(mut stats) = self.model_stats.write() {
                    let model_stat = stats.entry(event.model_name.clone()).or_insert_with(|| {
                        ModelStats {
                            name: event.model_name.clone(),
                            size_bytes: 0, // Will be updated when model is loaded
                            load_time_ms: 0,
                            inference_count: 0,
                            total_inference_time_ms: 0,
                            backend_type: "unknown".to_string(),
                        }
                    });

                    model_stat.inference_count += 1;
                    model_stat.total_inference_time_ms += event.duration.as_millis() as u64;
                }
            }
        });

        info!("Metrics event processing started");
    }
}

/// Thread-safe metrics collector for inference operations.
///
/// # Thread Safety
/// This type is Send + Sync because all fields are thread-safe:
/// - Instant is Send (not Sync, but that's okay for containing type)
/// - Arc<InferenceCounters> contains only atomics (Send + Sync)
/// - Arc<RwLock<HashMap>> is Send + Sync (standard pattern)
/// - mpsc::UnboundedSender is Send + Sync
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    start_time: Instant,
    inference_counters: Arc<InferenceCounters>,
    model_stats: Arc<RwLock<HashMap<String, ModelStats>>>,
    event_sender: mpsc::UnboundedSender<InferenceEvent>,
    /// Generic counters for custom metrics (e.g., CLI command counts)
    generic_counters: Arc<RwLock<HashMap<String, AtomicU64>>>,
    /// Generic gauges for custom metrics (e.g., duration measurements)
    generic_gauges: Arc<RwLock<HashMap<String, f64>>>,
}

#[derive(Debug)]
struct InferenceCounters {
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    total_tokens_generated: AtomicU64,
    total_inference_time_ms: AtomicU64,
}

impl Default for InferenceCounters {
    fn default() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_tokens_generated: AtomicU64::new(0),
            total_inference_time_ms: AtomicU64::new(0),
        }
    }
}

impl MetricsCollector {
    /// Create a new metrics collector and event processor.
    ///
    /// Returns a tuple of (MetricsCollector, MetricsEventProcessor).
    /// The MetricsCollector can be cloned and shared across threads.
    /// The MetricsEventProcessor should have `.start()` called to begin processing events.
    ///
    /// # Example
    /// ```no_run
    /// use inferno::metrics::MetricsCollector;
    /// let (collector, processor) = MetricsCollector::new();
    /// processor.start(); // Start background event processing
    /// // Use collector.record_inference(...) from any thread
    /// ```
    pub fn new() -> (Self, MetricsEventProcessor) {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let inference_counters = Arc::new(InferenceCounters::default());
        let model_stats = Arc::new(RwLock::new(HashMap::new()));

        let collector = Self {
            start_time: Instant::now(),
            inference_counters: Arc::clone(&inference_counters),
            model_stats: Arc::clone(&model_stats),
            event_sender,
            generic_counters: Arc::new(RwLock::new(HashMap::new())),
            generic_gauges: Arc::new(RwLock::new(HashMap::new())),
        };

        let processor = MetricsEventProcessor {
            receiver: event_receiver,
            counters: inference_counters,
            model_stats,
        };

        (collector, processor)
    }

    pub fn get_event_sender(&self) -> mpsc::UnboundedSender<InferenceEvent> {
        self.event_sender.clone()
    }

    pub fn record_model_loaded(
        &self,
        name: String,
        size_bytes: u64,
        load_time: Duration,
        backend_type: String,
    ) {
        if let Ok(mut stats) = self.model_stats.write() {
            stats.insert(
                name.clone(),
                ModelStats {
                    name,
                    size_bytes,
                    load_time_ms: load_time.as_millis() as u64,
                    inference_count: 0,
                    total_inference_time_ms: 0,
                    backend_type,
                },
            );
        }
    }

    pub fn record_inference(&self, event: InferenceEvent) {
        if self.event_sender.send(event).is_err() {
            tracing::warn!("Failed to send inference event - metrics collector may be shutdown");
        }
    }

    /// Increment a named counter by 1
    ///
    /// Counters are useful for tracking totals like command executions,
    /// errors, or other discrete events.
    pub fn increment_counter(&self, name: &str) {
        // First try to read - most common case is counter already exists
        if let Ok(counters) = self.generic_counters.read() {
            if let Some(counter) = counters.get(name) {
                counter.fetch_add(1, Ordering::Relaxed);
                return;
            }
        }
        // Counter doesn't exist, need write lock to create it
        if let Ok(mut counters) = self.generic_counters.write() {
            counters
                .entry(name.to_string())
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Set a gauge value
    ///
    /// Gauges are useful for point-in-time measurements like durations,
    /// resource usage, or other continuous values.
    pub fn record_gauge(&self, name: &str, value: f64) {
        if let Ok(mut gauges) = self.generic_gauges.write() {
            gauges.insert(name.to_string(), value);
        }
    }

    /// Get all custom counters (for snapshot/export)
    pub fn get_counters(&self) -> HashMap<String, u64> {
        if let Ok(counters) = self.generic_counters.read() {
            counters
                .iter()
                .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
                .collect()
        } else {
            HashMap::new()
        }
    }

    /// Get all custom gauges (for snapshot/export)
    pub fn get_gauges(&self) -> HashMap<String, f64> {
        if let Ok(gauges) = self.generic_gauges.read() {
            gauges.clone()
        } else {
            HashMap::new()
        }
    }

    pub async fn get_snapshot(&self) -> Result<MetricsSnapshot> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let inference_metrics = self.get_inference_metrics().await;
        let system_metrics = self.get_system_metrics().await?;
        let model_metrics = self.get_model_metrics().await;
        let custom_counters = self.get_counters();
        let custom_gauges = self.get_gauges();

        Ok(MetricsSnapshot {
            timestamp,
            inference_metrics,
            system_metrics,
            model_metrics,
            custom_counters,
            custom_gauges,
        })
    }

    async fn get_inference_metrics(&self) -> InferenceMetrics {
        let total_requests = self
            .inference_counters
            .total_requests
            .load(Ordering::Relaxed);
        let successful_requests = self
            .inference_counters
            .successful_requests
            .load(Ordering::Relaxed);
        let failed_requests = self
            .inference_counters
            .failed_requests
            .load(Ordering::Relaxed);
        let total_tokens = self
            .inference_counters
            .total_tokens_generated
            .load(Ordering::Relaxed);
        let total_time_ms = self
            .inference_counters
            .total_inference_time_ms
            .load(Ordering::Relaxed);

        let average_tokens_per_second = if total_time_ms > 0 {
            (total_tokens as f64 * 1000.0) / total_time_ms as f64
        } else {
            0.0
        };

        let average_latency_ms = if successful_requests > 0 {
            total_time_ms as f64 / successful_requests as f64
        } else {
            0.0
        };

        InferenceMetrics {
            total_requests,
            successful_requests,
            failed_requests,
            total_tokens_generated: total_tokens,
            total_inference_time_ms: total_time_ms,
            average_tokens_per_second,
            average_latency_ms,
        }
    }

    async fn get_system_metrics(&self) -> Result<SystemMetrics> {
        use sysinfo::{CpuExt, System, SystemExt};

        let mut system = System::new_all();
        system.refresh_all();

        let memory_usage_bytes = system.used_memory() * 1024; // sysinfo returns KB
        let cpu_usage_percent = system.global_cpu_info().cpu_usage();
        let uptime_seconds = self.start_time.elapsed().as_secs();

        // GPU metrics would require platform-specific code
        let gpu_memory_usage_bytes = None;
        let gpu_utilization_percent = None;

        Ok(SystemMetrics {
            memory_usage_bytes,
            cpu_usage_percent,
            gpu_memory_usage_bytes,
            gpu_utilization_percent,
            uptime_seconds,
        })
    }

    async fn get_model_metrics(&self) -> ModelMetrics {
        let loaded_models = if let Ok(stats) = self.model_stats.read() {
            stats.clone()
        } else {
            HashMap::new()
        };

        let total_model_size_bytes = loaded_models.values().map(|stats| stats.size_bytes).sum();

        ModelMetrics {
            loaded_models,
            total_model_size_bytes,
        }
    }

    pub async fn get_model_stats(&self, model_name: &str) -> Option<ModelStats> {
        if let Ok(stats) = self.model_stats.read() {
            stats.get(model_name).cloned()
        } else {
            None
        }
    }

    pub async fn export_metrics_json(&self) -> Result<String> {
        let snapshot = self.get_snapshot().await?;
        Ok(serde_json::to_string_pretty(&snapshot)?)
    }

    pub async fn export_prometheus_format(&self) -> Result<String> {
        let snapshot = self.get_snapshot().await?;
        let mut output = String::new();

        // Inference metrics
        output.push_str(
            "# HELP inferno_inference_requests_total Total number of inference requests\n",
        );
        output.push_str("# TYPE inferno_inference_requests_total counter\n");
        output.push_str(&format!(
            "inferno_inference_requests_total {}\n",
            snapshot.inference_metrics.total_requests
        ));

        output.push_str("# HELP inferno_inference_requests_successful_total Total number of successful inference requests\n");
        output.push_str("# TYPE inferno_inference_requests_successful_total counter\n");
        output.push_str(&format!(
            "inferno_inference_requests_successful_total {}\n",
            snapshot.inference_metrics.successful_requests
        ));

        output.push_str("# HELP inferno_inference_requests_failed_total Total number of failed inference requests\n");
        output.push_str("# TYPE inferno_inference_requests_failed_total counter\n");
        output.push_str(&format!(
            "inferno_inference_requests_failed_total {}\n",
            snapshot.inference_metrics.failed_requests
        ));

        output.push_str("# HELP inferno_inference_tokens_total Total number of tokens generated\n");
        output.push_str("# TYPE inferno_inference_tokens_total counter\n");
        output.push_str(&format!(
            "inferno_inference_tokens_total {}\n",
            snapshot.inference_metrics.total_tokens_generated
        ));

        output.push_str("# HELP inferno_inference_duration_ms_total Total time spent on inference in milliseconds\n");
        output.push_str("# TYPE inferno_inference_duration_ms_total counter\n");
        output.push_str(&format!(
            "inferno_inference_duration_ms_total {}\n",
            snapshot.inference_metrics.total_inference_time_ms
        ));

        output.push_str("# HELP inferno_tokens_per_second Average tokens generated per second\n");
        output.push_str("# TYPE inferno_tokens_per_second gauge\n");
        output.push_str(&format!(
            "inferno_tokens_per_second {}\n",
            snapshot.inference_metrics.average_tokens_per_second
        ));

        output.push_str("# HELP inferno_latency_ms Average latency in milliseconds\n");
        output.push_str("# TYPE inferno_latency_ms gauge\n");
        output.push_str(&format!(
            "inferno_latency_ms {}\n",
            snapshot.inference_metrics.average_latency_ms
        ));

        // System metrics
        output.push_str("# HELP inferno_memory_usage_bytes Memory usage in bytes\n");
        output.push_str("# TYPE inferno_memory_usage_bytes gauge\n");
        output.push_str(&format!(
            "inferno_memory_usage_bytes {}\n",
            snapshot.system_metrics.memory_usage_bytes
        ));

        output.push_str("# HELP inferno_cpu_usage_percent CPU usage percentage\n");
        output.push_str("# TYPE inferno_cpu_usage_percent gauge\n");
        output.push_str(&format!(
            "inferno_cpu_usage_percent {}\n",
            snapshot.system_metrics.cpu_usage_percent
        ));

        output.push_str("# HELP inferno_uptime_seconds Server uptime in seconds\n");
        output.push_str("# TYPE inferno_uptime_seconds counter\n");
        output.push_str(&format!(
            "inferno_uptime_seconds {}\n",
            snapshot.system_metrics.uptime_seconds
        ));

        // GPU metrics (if available)
        if let Some(gpu_memory) = snapshot.system_metrics.gpu_memory_usage_bytes {
            output.push_str("# HELP inferno_gpu_memory_usage_bytes GPU memory usage in bytes\n");
            output.push_str("# TYPE inferno_gpu_memory_usage_bytes gauge\n");
            output.push_str(&format!("inferno_gpu_memory_usage_bytes {}\n", gpu_memory));
        }

        if let Some(gpu_util) = snapshot.system_metrics.gpu_utilization_percent {
            output.push_str("# HELP inferno_gpu_utilization_percent GPU utilization percentage\n");
            output.push_str("# TYPE inferno_gpu_utilization_percent gauge\n");
            output.push_str(&format!("inferno_gpu_utilization_percent {}\n", gpu_util));
        }

        // Model metrics
        output.push_str("# HELP inferno_loaded_models_count Number of currently loaded models\n");
        output.push_str("# TYPE inferno_loaded_models_count gauge\n");
        output.push_str(&format!(
            "inferno_loaded_models_count {}\n",
            snapshot.model_metrics.loaded_models.len()
        ));

        output.push_str(
            "# HELP inferno_models_size_bytes_total Total size of all loaded models in bytes\n",
        );
        output.push_str("# TYPE inferno_models_size_bytes_total gauge\n");
        output.push_str(&format!(
            "inferno_models_size_bytes_total {}\n",
            snapshot.model_metrics.total_model_size_bytes
        ));

        // Per-model metrics
        for (model_name, stats) in &snapshot.model_metrics.loaded_models {
            let safe_model_name = model_name.replace("\"", "\\\"");

            output
                .push_str("# HELP inferno_model_inference_count Number of inferences per model\n");
            output.push_str("# TYPE inferno_model_inference_count counter\n");
            output.push_str(&format!(
                "inferno_model_inference_count{{model=\"{}\",backend=\"{}\"}} {}\n",
                safe_model_name, stats.backend_type, stats.inference_count
            ));

            output.push_str("# HELP inferno_model_size_bytes Model size in bytes\n");
            output.push_str("# TYPE inferno_model_size_bytes gauge\n");
            output.push_str(&format!(
                "inferno_model_size_bytes{{model=\"{}\",backend=\"{}\"}} {}\n",
                safe_model_name, stats.backend_type, stats.size_bytes
            ));

            output.push_str("# HELP inferno_model_load_time_ms Model load time in milliseconds\n");
            output.push_str("# TYPE inferno_model_load_time_ms gauge\n");
            output.push_str(&format!(
                "inferno_model_load_time_ms{{model=\"{}\",backend=\"{}\"}} {}\n",
                safe_model_name, stats.backend_type, stats.load_time_ms
            ));

            output.push_str("# HELP inferno_model_inference_duration_ms_total Total inference time per model in milliseconds\n");
            output.push_str("# TYPE inferno_model_inference_duration_ms_total counter\n");
            output.push_str(&format!(
                "inferno_model_inference_duration_ms_total{{model=\"{}\",backend=\"{}\"}} {}\n",
                safe_model_name, stats.backend_type, stats.total_inference_time_ms
            ));
        }

        // Custom counters
        if !snapshot.custom_counters.is_empty() {
            output.push_str("\n# Custom counters\n");
            for (name, value) in &snapshot.custom_counters {
                // Sanitize metric name for Prometheus (replace . and - with _)
                let safe_name = name.replace(['.', '-'], "_");
                output.push_str(&format!("# HELP {} Custom counter metric\n", safe_name));
                output.push_str(&format!("# TYPE {} counter\n", safe_name));
                output.push_str(&format!("{} {}\n", safe_name, value));
            }
        }

        // Custom gauges
        if !snapshot.custom_gauges.is_empty() {
            output.push_str("\n# Custom gauges\n");
            for (name, value) in &snapshot.custom_gauges {
                // Sanitize metric name for Prometheus (replace . and - with _)
                let safe_name = name.replace(['.', '-'], "_");
                output.push_str(&format!("# HELP {} Custom gauge metric\n", safe_name));
                output.push_str(&format!("# TYPE {} gauge\n", safe_name));
                output.push_str(&format!("{} {}\n", safe_name, value));
            }
        }

        Ok(output)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        let (collector, processor) = Self::new();
        processor.start();
        collector
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_metrics_collector() {
        let (collector, processor) = MetricsCollector::new();
        processor.start();

        // Record a model load
        collector.record_model_loaded(
            "test_model".to_string(),
            1024 * 1024, // 1MB
            Duration::from_millis(100),
            "gguf".to_string(),
        );

        // Record an inference event
        let event = InferenceEvent {
            model_name: "test_model".to_string(),
            input_length: 50,
            output_length: 100,
            duration: Duration::from_millis(500),
            success: true,
        };
        collector.record_inference(event);

        // Give some time for event processing
        sleep(Duration::from_millis(10)).await;

        let snapshot = collector.get_snapshot().await.unwrap();
        assert_eq!(snapshot.inference_metrics.total_requests, 1);
        assert_eq!(snapshot.inference_metrics.successful_requests, 1);
        assert_eq!(snapshot.inference_metrics.total_tokens_generated, 100);
        assert_eq!(snapshot.model_metrics.loaded_models.len(), 1);
    }

    #[tokio::test]
    async fn test_metrics_export() {
        let (collector, processor) = MetricsCollector::new();
        processor.start();

        let json_export = collector.export_metrics_json().await.unwrap();
        assert!(json_export.contains("inference_metrics"));
        assert!(json_export.contains("system_metrics"));

        let prometheus_export = collector.export_prometheus_format().await.unwrap();
        assert!(prometheus_export.contains("inferno_inference_requests_total"));
        assert!(prometheus_export.contains("# HELP"));
        assert!(prometheus_export.contains("# TYPE"));
    }

    #[tokio::test]
    async fn test_generic_counters() {
        let (collector, processor) = MetricsCollector::new();
        processor.start();

        // Increment a new counter
        collector.increment_counter("test.command.total");
        collector.increment_counter("test.command.total");
        collector.increment_counter("test.command.success");

        let counters = collector.get_counters();
        assert_eq!(counters.get("test.command.total"), Some(&2));
        assert_eq!(counters.get("test.command.success"), Some(&1));

        // Verify snapshot includes custom counters
        let snapshot = collector.get_snapshot().await.unwrap();
        assert_eq!(snapshot.custom_counters.get("test.command.total"), Some(&2));
    }

    #[tokio::test]
    async fn test_generic_gauges() {
        let (collector, processor) = MetricsCollector::new();
        processor.start();

        // Record gauge values
        collector.record_gauge("test.duration_ms", 150.5);
        collector.record_gauge("test.exit_code", 0.0);

        let gauges = collector.get_gauges();
        assert_eq!(gauges.get("test.duration_ms"), Some(&150.5));
        assert_eq!(gauges.get("test.exit_code"), Some(&0.0));

        // Update gauge value
        collector.record_gauge("test.duration_ms", 200.0);
        let gauges = collector.get_gauges();
        assert_eq!(gauges.get("test.duration_ms"), Some(&200.0));

        // Verify snapshot includes custom gauges
        let snapshot = collector.get_snapshot().await.unwrap();
        assert_eq!(snapshot.custom_gauges.get("test.duration_ms"), Some(&200.0));
    }

    #[tokio::test]
    async fn test_custom_metrics_prometheus_export() {
        let (collector, processor) = MetricsCollector::new();
        processor.start();

        // Add some custom metrics
        collector.increment_counter("inferno.command.total");
        collector.record_gauge("inferno.command.duration_ms", 42.5);

        let prometheus_export = collector.export_prometheus_format().await.unwrap();

        // Custom counters should be exported with sanitized names
        assert!(prometheus_export.contains("inferno_command_total"));
        assert!(prometheus_export.contains("# TYPE inferno_command_total counter"));

        // Custom gauges should be exported with sanitized names
        assert!(prometheus_export.contains("inferno_command_duration_ms"));
        assert!(prometheus_export.contains("# TYPE inferno_command_duration_ms gauge"));
    }
}
