use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
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

#[derive(Debug)]
pub struct MetricsCollector {
    start_time: Instant,
    inference_counters: Arc<InferenceCounters>,
    model_stats: Arc<RwLock<HashMap<String, ModelStats>>>,
    event_sender: mpsc::UnboundedSender<InferenceEvent>,
    event_receiver: Option<mpsc::UnboundedReceiver<InferenceEvent>>,
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        // We can't clone the receiver, so create a new one that won't be used
        let (_tx, _rx) = mpsc::unbounded_channel::<InferenceEvent>();
        Self {
            start_time: self.start_time,
            inference_counters: self.inference_counters.clone(),
            model_stats: self.model_stats.clone(),
            event_sender: self.event_sender.clone(),
            event_receiver: None, // Can't clone receiver
        }
    }
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
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            start_time: Instant::now(),
            inference_counters: Arc::new(InferenceCounters::default()),
            model_stats: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_receiver: Some(event_receiver),
        }
    }

    pub fn get_event_sender(&self) -> mpsc::UnboundedSender<InferenceEvent> {
        self.event_sender.clone()
    }

    pub async fn start_event_processing(&mut self) -> Result<()> {
        if let Some(mut receiver) = self.event_receiver.take() {
            let counters = Arc::clone(&self.inference_counters);
            let model_stats = Arc::clone(&self.model_stats);

            tokio::spawn(async move {
                while let Some(event) = receiver.recv().await {
                    // Update global counters
                    counters.total_requests.fetch_add(1, Ordering::Relaxed);

                    if event.success {
                        counters.successful_requests.fetch_add(1, Ordering::Relaxed);
                        counters
                            .total_tokens_generated
                            .fetch_add(event.output_length as u64, Ordering::Relaxed);
                    } else {
                        counters.failed_requests.fetch_add(1, Ordering::Relaxed);
                    }

                    counters
                        .total_inference_time_ms
                        .fetch_add(event.duration.as_millis() as u64, Ordering::Relaxed);

                    // Update model-specific stats
                    if let Ok(mut stats) = model_stats.write() {
                        let model_stat =
                            stats
                                .entry(event.model_name.clone())
                                .or_insert_with(|| ModelStats {
                                    name: event.model_name.clone(),
                                    size_bytes: 0, // Will be updated when model is loaded
                                    load_time_ms: 0,
                                    inference_count: 0,
                                    total_inference_time_ms: 0,
                                    backend_type: "unknown".to_string(),
                                });

                        model_stat.inference_count += 1;
                        model_stat.total_inference_time_ms += event.duration.as_millis() as u64;
                    }
                }
            });

            info!("Metrics event processing started");
        }

        Ok(())
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

    pub async fn get_snapshot(&self) -> Result<MetricsSnapshot> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let inference_metrics = self.get_inference_metrics().await;
        let system_metrics = self.get_system_metrics().await?;
        let model_metrics = self.get_model_metrics().await;

        Ok(MetricsSnapshot {
            timestamp,
            inference_metrics,
            system_metrics,
            model_metrics,
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

        Ok(output)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();
        collector.start_event_processing().await.unwrap();

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
        let mut collector = MetricsCollector::new();
        collector.start_event_processing().await.unwrap();

        let json_export = collector.export_metrics_json().await.unwrap();
        assert!(json_export.contains("inference_metrics"));
        assert!(json_export.contains("system_metrics"));

        let prometheus_export = collector.export_prometheus_format().await.unwrap();
        assert!(prometheus_export.contains("inferno_inference_requests_total"));
        assert!(prometheus_export.contains("# HELP"));
        assert!(prometheus_export.contains("# TYPE"));
    }
}
