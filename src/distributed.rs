use crate::{
    backends::{BackendHandle, BackendConfig, BackendType, InferenceParams},
    cache::ModelCache,
    models::{ModelInfo, ModelManager},
    metrics::MetricsCollector,
    InfernoError,
};
use anyhow::{anyhow, Result};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::Path,
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{mpsc, oneshot, Mutex, RwLock, Semaphore},
    task::JoinHandle,
    time::timeout,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Configuration for the distributed inference system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedConfig {
    /// Number of worker processes/threads to spawn
    pub worker_count: usize,
    /// Maximum number of concurrent requests per worker
    pub max_concurrent_per_worker: usize,
    /// Request timeout in seconds
    pub request_timeout_seconds: u64,
    /// Whether to enable load balancing across workers
    pub load_balancing: bool,
    /// Worker pool strategy
    pub pool_strategy: PoolStrategy,
    /// Enable automatic model preloading
    pub preload_models: bool,
    /// Maximum models to keep loaded per worker
    pub max_models_per_worker: usize,
}

impl Default for DistributedConfig {
    fn default() -> Self {
        Self {
            worker_count: num_cpus::get().max(1),
            max_concurrent_per_worker: 4,
            request_timeout_seconds: 300,
            load_balancing: true,
            pool_strategy: PoolStrategy::RoundRobin,
            preload_models: false,
            max_models_per_worker: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoolStrategy {
    RoundRobin,
    LeastLoaded,
    Sticky, // Same model always goes to the same worker
}

/// Request sent to workers
#[derive(Debug)]
pub struct InferenceRequest {
    pub id: Uuid,
    pub model_name: String,
    pub input: String,
    pub params: InferenceParams,
    pub response_tx: oneshot::Sender<Result<InferenceResponse>>,
}

/// Response from workers
#[derive(Debug, Clone)]
pub struct InferenceResponse {
    pub id: Uuid,
    pub output: String,
    pub tokens_generated: u32,
    pub duration: Duration,
    pub worker_id: usize,
}

/// Streaming request sent to workers
#[derive(Debug)]
pub struct StreamingInferenceRequest {
    pub id: Uuid,
    pub model_name: String,
    pub input: String,
    pub params: InferenceParams,
    pub response_tx: mpsc::UnboundedSender<Result<String>>,
}

/// Worker statistics
#[derive(Debug, Clone, Default)]
pub struct WorkerStats {
    pub worker_id: usize,
    pub active_requests: usize,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub loaded_models: Vec<String>,
    pub memory_usage: u64,
    pub last_activity: Option<Instant>,
}

/// Main distributed inference coordinator
pub struct DistributedInference {
    config: DistributedConfig,
    backend_config: BackendConfig,
    model_manager: Arc<ModelManager>,
    metrics: Option<Arc<MetricsCollector>>,
    workers: Vec<WorkerHandle>,
    next_worker: Arc<AtomicUsize>,
    stats: Arc<RwLock<HashMap<usize, WorkerStats>>>,
    shutdown_tx: Option<mpsc::UnboundedSender<()>>,
}

/// Handle to a worker thread
struct WorkerHandle {
    worker_id: usize,
    request_tx: mpsc::UnboundedSender<WorkerMessage>,
    streaming_tx: mpsc::UnboundedSender<StreamingInferenceRequest>,
    join_handle: JoinHandle<()>,
    semaphore: Arc<Semaphore>,
}

/// Messages sent to workers
#[derive(Debug)]
enum WorkerMessage {
    InferenceRequest(InferenceRequest),
    PreloadModel { model_name: String },
    UnloadModel { model_name: String },
    GetStats { response_tx: oneshot::Sender<WorkerStats> },
    Shutdown,
}

/// Internal worker state
struct Worker {
    worker_id: usize,
    backends: HashMap<String, BackendHandle>,
    backend_config: BackendConfig,
    model_manager: Arc<ModelManager>,
    metrics: Option<Arc<MetricsCollector>>,
    stats: WorkerStats,
    max_models: usize,
}

impl DistributedInference {
    /// Create a new distributed inference system
    pub async fn new(
        config: DistributedConfig,
        backend_config: BackendConfig,
        model_manager: Arc<ModelManager>,
        metrics: Option<Arc<MetricsCollector>>,
    ) -> Result<Self> {
        info!("Initializing distributed inference with {} workers", config.worker_count);

        let stats = Arc::new(RwLock::new(HashMap::new()));
        let next_worker = Arc::new(AtomicUsize::new(0));
        let mut workers = Vec::with_capacity(config.worker_count);
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();

        // Spawn workers
        for worker_id in 0..config.worker_count {
            let worker_handle = Self::spawn_worker(
                worker_id,
                backend_config.clone(),
                model_manager.clone(),
                metrics.clone(),
                config.max_concurrent_per_worker,
                config.max_models_per_worker,
                stats.clone(),
            ).await?;

            workers.push(worker_handle);
        }

        info!("Successfully spawned {} workers", workers.len());

        let mut distributed = Self {
            config,
            backend_config,
            model_manager,
            metrics,
            workers,
            next_worker,
            stats,
            shutdown_tx: Some(shutdown_tx),
        };

        // Preload models if enabled
        if distributed.config.preload_models {
            distributed.preload_common_models().await?;
        }

        Ok(distributed)
    }

    /// Spawn a new worker
    async fn spawn_worker(
        worker_id: usize,
        backend_config: BackendConfig,
        model_manager: Arc<ModelManager>,
        metrics: Option<Arc<MetricsCollector>>,
        max_concurrent: usize,
        max_models: usize,
        stats: Arc<RwLock<HashMap<usize, WorkerStats>>>,
    ) -> Result<WorkerHandle> {
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let (streaming_tx, streaming_rx) = mpsc::unbounded_channel();
        let semaphore = Arc::new(Semaphore::new(max_concurrent));

        let worker = Worker {
            worker_id,
            backends: HashMap::new(),
            backend_config,
            model_manager,
            metrics,
            stats: WorkerStats {
                worker_id,
                ..Default::default()
            },
            max_models,
        };

        let join_handle = tokio::spawn(
            Self::worker_loop(worker, request_rx, streaming_rx, stats)
        );

        Ok(WorkerHandle {
            worker_id,
            request_tx,
            streaming_tx,
            join_handle,
            semaphore,
        })
    }

    /// Main worker loop
    async fn worker_loop(
        mut worker: Worker,
        mut request_rx: mpsc::UnboundedReceiver<WorkerMessage>,
        mut streaming_rx: mpsc::UnboundedReceiver<StreamingInferenceRequest>,
        stats: Arc<RwLock<HashMap<usize, WorkerStats>>>,
    ) {
        debug!("Worker {} started", worker.worker_id);

        loop {
            tokio::select! {
                // Handle regular requests
                msg = request_rx.recv() => {
                    match msg {
                        Some(WorkerMessage::InferenceRequest(req)) => {
                            worker.handle_inference_request(req).await;
                        }
                        Some(WorkerMessage::PreloadModel { model_name }) => {
                            worker.preload_model(&model_name).await;
                        }
                        Some(WorkerMessage::UnloadModel { model_name }) => {
                            worker.unload_model(&model_name).await;
                        }
                        Some(WorkerMessage::GetStats { response_tx }) => {
                            let _ = response_tx.send(worker.stats.clone());
                        }
                        Some(WorkerMessage::Shutdown) | None => {
                            info!("Worker {} shutting down", worker.worker_id);
                            break;
                        }
                    }
                }

                // Handle streaming requests
                streaming_req = streaming_rx.recv() => {
                    if let Some(req) = streaming_req {
                        worker.handle_streaming_request(req).await;
                    }
                }
            }

            // Update stats
            worker.stats.last_activity = Some(Instant::now());
            let mut stats_guard = stats.write().await;
            stats_guard.insert(worker.worker_id, worker.stats.clone());
        }

        debug!("Worker {} finished", worker.worker_id);
    }

    /// Submit an inference request
    pub async fn infer(
        &self,
        model_name: &str,
        input: &str,
        params: &InferenceParams,
    ) -> Result<InferenceResponse> {
        let worker_id = self.select_worker(model_name).await?;
        let worker = &self.workers[worker_id];

        // Acquire semaphore permit
        let _permit = timeout(
            Duration::from_secs(self.config.request_timeout_seconds),
            worker.semaphore.acquire()
        ).await
        .map_err(|_| anyhow!("Request timed out waiting for worker availability"))?
        .map_err(|_| anyhow!("Worker semaphore closed"))?;

        let (response_tx, response_rx) = oneshot::channel();
        let request = InferenceRequest {
            id: Uuid::new_v4(),
            model_name: model_name.to_string(),
            input: input.to_string(),
            params: params.clone(),
            response_tx,
        };

        worker.request_tx.send(WorkerMessage::InferenceRequest(request))
            .map_err(|_| anyhow!("Failed to send request to worker"))?;

        let response = timeout(
            Duration::from_secs(self.config.request_timeout_seconds),
            response_rx
        ).await
        .map_err(|_| anyhow!("Request timed out"))?
        .map_err(|_| anyhow!("Worker response channel closed"))??;

        Ok(response)
    }

    /// Submit a streaming inference request
    pub async fn infer_stream(
        &self,
        model_name: &str,
        input: &str,
        params: &InferenceParams,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let worker_id = self.select_worker(model_name).await?;
        let worker = &self.workers[worker_id];

        let (response_tx, response_rx) = mpsc::unbounded_channel();
        let request = StreamingInferenceRequest {
            id: Uuid::new_v4(),
            model_name: model_name.to_string(),
            input: input.to_string(),
            params: params.clone(),
            response_tx,
        };

        worker.streaming_tx.send(request)
            .map_err(|_| anyhow!("Failed to send streaming request to worker"))?;

        let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(response_rx);
        Ok(Box::pin(stream))
    }

    /// Select the best worker for a request
    async fn select_worker(&self, model_name: &str) -> Result<usize> {
        match self.config.pool_strategy {
            PoolStrategy::RoundRobin => {
                let worker_id = self.next_worker.fetch_add(1, Ordering::Relaxed) % self.workers.len();
                Ok(worker_id)
            }
            PoolStrategy::LeastLoaded => {
                self.select_least_loaded_worker().await
            }
            PoolStrategy::Sticky => {
                // Use consistent hashing to assign model to worker
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut hasher = DefaultHasher::new();
                model_name.hash(&mut hasher);
                let worker_id = (hasher.finish() as usize) % self.workers.len();
                Ok(worker_id)
            }
        }
    }

    /// Find the worker with the least active requests
    async fn select_least_loaded_worker(&self) -> Result<usize> {
        let stats = self.stats.read().await;
        let mut best_worker = 0;
        let mut min_load = usize::MAX;

        for (worker_id, worker_stats) in stats.iter() {
            if worker_stats.active_requests < min_load {
                min_load = worker_stats.active_requests;
                best_worker = *worker_id;
            }
        }

        Ok(best_worker)
    }

    /// Preload commonly used models
    async fn preload_common_models(&self) -> Result<()> {
        info!("Preloading common models across workers...");

        // For now, just distribute models evenly across workers
        // In a production system, this could be based on usage patterns
        let models = self.model_manager.list_models().await?;

        for (i, model) in models.iter().enumerate() {
            let worker_id = i % self.workers.len();
            let worker = &self.workers[worker_id];

            let _ = worker.request_tx.send(WorkerMessage::PreloadModel {
                model_name: model.name.clone(),
            });
        }

        info!("Model preloading initiated");
        Ok(())
    }

    /// Get system-wide statistics
    pub async fn get_stats(&self) -> HashMap<usize, WorkerStats> {
        self.stats.read().await.clone()
    }

    /// Get detailed worker statistics
    pub async fn get_detailed_stats(&self) -> Result<HashMap<usize, WorkerStats>> {
        let mut detailed_stats = HashMap::new();

        for worker in &self.workers {
            let (tx, rx) = oneshot::channel();
            if worker.request_tx.send(WorkerMessage::GetStats { response_tx: tx }).is_ok() {
                if let Ok(stats) = timeout(Duration::from_secs(5), rx).await {
                    if let Ok(stats) = stats {
                        detailed_stats.insert(worker.worker_id, stats);
                    }
                }
            }
        }

        Ok(detailed_stats)
    }

    /// Graceful shutdown
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down distributed inference system");

        // Send shutdown signal to all workers
        for worker in &self.workers {
            let _ = worker.request_tx.send(WorkerMessage::Shutdown);
        }

        // Wait for all workers to finish
        let handles = std::mem::take(&mut self.workers);
        for worker in handles {
            if let Err(e) = worker.join_handle.await {
                warn!("Worker {} failed to shutdown cleanly: {}", worker.worker_id, e);
            }
        }

        info!("All workers shut down successfully");
        Ok(())
    }
}

impl Worker {
    /// Handle a single inference request
    async fn handle_inference_request(&mut self, request: InferenceRequest) {
        let start_time = Instant::now();
        self.stats.active_requests += 1;
        self.stats.total_requests += 1;

        let result = self.process_inference(&request.model_name, &request.input, &request.params).await;

        let duration = start_time.elapsed();
        self.stats.active_requests = self.stats.active_requests.saturating_sub(1);

        match result {
            Ok(output) => {
                self.stats.successful_requests += 1;
                self.update_average_response_time(duration);

                let response = InferenceResponse {
                    id: request.id,
                    output: output.clone(),
                    tokens_generated: (output.len() / 4) as u32, // Rough estimate
                    duration,
                    worker_id: self.worker_id,
                };

                if let Some(ref metrics) = self.metrics {
                    use crate::metrics::InferenceEvent;
                    metrics.record_inference(InferenceEvent {
                        model_name: request.model_name.clone(),
                        input_length: request.input.len() as u32,
                        output_length: response.tokens_generated,
                        duration,
                        success: true,
                    });
                }

                let _ = request.response_tx.send(Ok(response));
            }
            Err(e) => {
                self.stats.failed_requests += 1;
                error!("Inference failed on worker {}: {}", self.worker_id, e);
                let _ = request.response_tx.send(Err(e));
            }
        }
    }

    /// Handle a streaming inference request
    async fn handle_streaming_request(&mut self, request: StreamingInferenceRequest) {
        self.stats.active_requests += 1;
        self.stats.total_requests += 1;

        let result = self.process_streaming_inference(
            &request.model_name,
            &request.input,
            &request.params,
            request.response_tx.clone(),
        ).await;

        self.stats.active_requests = self.stats.active_requests.saturating_sub(1);

        match result {
            Ok(_) => self.stats.successful_requests += 1,
            Err(e) => {
                self.stats.failed_requests += 1;
                error!("Streaming inference failed on worker {}: {}", self.worker_id, e);
                let _ = request.response_tx.send(Err(e));
            }
        }
    }

    /// Process a single inference request
    async fn process_inference(
        &mut self,
        model_name: &str,
        input: &str,
        params: &InferenceParams,
    ) -> Result<String> {
        let backend = self.get_or_load_backend(model_name).await?;
        backend.infer(input, params).await
    }

    /// Process a streaming inference request
    async fn process_streaming_inference(
        &mut self,
        model_name: &str,
        input: &str,
        params: &InferenceParams,
        response_tx: mpsc::UnboundedSender<Result<String>>,
    ) -> Result<()> {
        let backend = self.get_or_load_backend(model_name).await?;
        let mut stream = backend.infer_stream(input, params).await?;

        while let Some(token_result) = stream.next().await {
            let converted_result = token_result.map_err(|e| anyhow::anyhow!("{}", e));
            if response_tx.send(converted_result).is_err() {
                break; // Client disconnected
            }
        }

        Ok(())
    }

    /// Get or load a backend for the specified model
    async fn get_or_load_backend(&mut self, model_name: &str) -> Result<&BackendHandle> {
        if !self.backends.contains_key(model_name) {
            // Check if we need to evict models due to memory constraints
            if self.backends.len() >= self.max_models {
                self.evict_least_used_model().await;
            }

            let model_info = self.model_manager.resolve_model(model_name).await?;
            let backend_type = BackendType::from_model_path(&model_info.path);
            let backend_handle = BackendHandle::new_shared(backend_type, &self.backend_config)?;
            backend_handle.load_model(&model_info).await?;

            self.backends.insert(model_name.to_string(), backend_handle);
            self.stats.loaded_models.push(model_name.to_string());

            info!("Loaded model {} on worker {}", model_name, self.worker_id);
        }

        Ok(self.backends.get(model_name).unwrap())
    }

    /// Preload a model
    async fn preload_model(&mut self, model_name: &str) {
        if let Err(e) = self.get_or_load_backend(model_name).await {
            warn!("Failed to preload model {} on worker {}: {}",
                  model_name, self.worker_id, e);
        }
    }

    /// Unload a model
    async fn unload_model(&mut self, model_name: &str) {
        if self.backends.remove(model_name).is_some() {
            self.stats.loaded_models.retain(|m| m != model_name);
            info!("Unloaded model {} from worker {}", model_name, self.worker_id);
        }
    }

    /// Evict the least recently used model
    async fn evict_least_used_model(&mut self) {
        // Simple LRU - in practice, you'd want better tracking
        if let Some(model_name) = self.stats.loaded_models.first().cloned() {
            self.unload_model(&model_name).await;
        }
    }

    /// Update average response time
    fn update_average_response_time(&mut self, new_duration: Duration) {
        let current_avg = self.stats.average_response_time.as_millis() as f64;
        let new_duration_ms = new_duration.as_millis() as f64;
        let requests = self.stats.successful_requests as f64;

        if requests > 0.0 {
            let new_avg = ((current_avg * (requests - 1.0)) + new_duration_ms) / requests;
            self.stats.average_response_time = Duration::from_millis(new_avg as u64);
        } else {
            self.stats.average_response_time = new_duration;
        }
    }
}

impl Drop for DistributedInference {
    fn drop(&mut self) {
        if !self.workers.is_empty() {
            warn!("DistributedInference dropped without explicit shutdown");
        }
    }
}