/// Mock implementations for testing

use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use std::{
    collections::HashMap,
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// Mock backend for testing without real model loading
pub struct MockBackend {
    pub backend_type: inferno::backends::BackendType,
    pub config: inferno::backends::BackendConfig,
    pub is_loaded: bool,
    pub model_info: Option<inferno::models::ModelInfo>,
    pub inference_delay_ms: u64,
    pub failure_rate: f64, // 0.0 = never fail, 1.0 = always fail
    pub metrics: inferno::backends::InferenceMetrics,
    pub call_count: AtomicU64,
}

impl MockBackend {
    pub fn new(backend_type: inferno::backends::BackendType, config: inferno::backends::BackendConfig) -> Self {
        Self {
            backend_type,
            config,
            is_loaded: false,
            model_info: None,
            inference_delay_ms: 100,
            failure_rate: 0.0,
            metrics: inferno::backends::InferenceMetrics {
                total_tokens: 0,
                prompt_tokens: 0,
                completion_tokens: 0,
                total_time_ms: 0,
                tokens_per_second: 0.0,
                prompt_time_ms: 0,
                completion_time_ms: 0,
            },
            call_count: AtomicU64::new(0),
        }
    }

    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.failure_rate = rate;
        self
    }

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.inference_delay_ms = delay_ms;
        self
    }

    fn should_fail(&self) -> bool {
        use rand::Rng;
        rand::thread_rng().gen::<f64>() < self.failure_rate
    }

    fn simulate_inference(&mut self, input: &str) -> Result<String> {
        self.call_count.fetch_add(1, Ordering::Relaxed);

        if self.should_fail() {
            return Err(anyhow::anyhow!("Mock backend simulated failure"));
        }

        let response = format!(
            "Mock response for input: '{}' (length: {})",
            input.chars().take(50).collect::<String>(),
            input.len()
        );

        // Update metrics
        self.metrics.total_tokens += input.len() as u32 + response.len() as u32;
        self.metrics.prompt_tokens += input.len() as u32;
        self.metrics.completion_tokens += response.len() as u32;
        self.metrics.total_time_ms += self.inference_delay_ms;

        Ok(response)
    }
}

#[async_trait]
impl inferno::backends::InferenceBackend for MockBackend {
    async fn load_model(&mut self, model_info: &inferno::models::ModelInfo) -> Result<()> {
        if self.should_fail() {
            return Err(anyhow::anyhow!("Mock backend failed to load model"));
        }

        tokio::time::sleep(Duration::from_millis(50)).await; // Simulate load time
        self.model_info = Some(model_info.clone());
        self.is_loaded = true;
        Ok(())
    }

    async fn unload_model(&mut self) -> Result<()> {
        self.model_info = None;
        self.is_loaded = false;
        Ok(())
    }

    async fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    async fn get_model_info(&self) -> Option<inferno::models::ModelInfo> {
        self.model_info.clone()
    }

    async fn infer(&mut self, input: &str, _params: &inferno::backends::InferenceParams) -> Result<String> {
        if !self.is_loaded {
            return Err(anyhow::anyhow!("No model loaded"));
        }

        tokio::time::sleep(Duration::from_millis(self.inference_delay_ms)).await;
        self.simulate_inference(input)
    }

    async fn infer_stream(
        &mut self,
        input: &str,
        params: &inferno::backends::InferenceParams,
    ) -> Result<inferno::backends::TokenStream> {
        if !self.is_loaded {
            return Err(anyhow::anyhow!("No model loaded"));
        }

        let response = self.simulate_inference(input)?;
        let tokens: Vec<String> = response
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let delay_per_token = Duration::from_millis(self.inference_delay_ms / tokens.len().max(1) as u64);

        let stream = async_stream::stream! {
            for token in tokens {
                tokio::time::sleep(delay_per_token).await;
                yield Ok(token);
            }
        };

        Ok(Box::pin(stream))
    }

    async fn get_embeddings(&mut self, input: &str) -> Result<Vec<f32>> {
        if !self.is_loaded {
            return Err(anyhow::anyhow!("No model loaded"));
        }

        if self.should_fail() {
            return Err(anyhow::anyhow!("Mock backend failed to get embeddings"));
        }

        tokio::time::sleep(Duration::from_millis(self.inference_delay_ms / 2)).await;

        // Generate mock embeddings based on input
        let embedding_size = 768; // Common embedding size
        let mut embeddings = Vec::with_capacity(embedding_size);

        for i in 0..embedding_size {
            let value = (input.len() as f32 * i as f32 * 0.001) % 1.0;
            embeddings.push(value - 0.5); // Center around 0
        }

        Ok(embeddings)
    }

    fn get_backend_type(&self) -> inferno::backends::BackendType {
        self.backend_type
    }

    fn get_metrics(&self) -> Option<inferno::backends::InferenceMetrics> {
        Some(self.metrics.clone())
    }
}

/// Mock model manager for testing
pub struct MockModelManager {
    pub models: Vec<inferno::models::ModelInfo>,
    pub discovery_delay_ms: u64,
    pub failure_rate: f64,
}

impl MockModelManager {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            discovery_delay_ms: 50,
            failure_rate: 0.0,
        }
    }

    pub fn with_models(mut self, models: Vec<inferno::models::ModelInfo>) -> Self {
        self.models = models;
        self
    }

    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.failure_rate = rate;
        self
    }

    pub async fn discover_models(&self) -> Result<Vec<inferno::models::ModelInfo>> {
        tokio::time::sleep(Duration::from_millis(self.discovery_delay_ms)).await;

        if self.failure_rate > 0.0 {
            use rand::Rng;
            if rand::thread_rng().gen::<f64>() < self.failure_rate {
                return Err(anyhow::anyhow!("Mock model discovery failed"));
            }
        }

        Ok(self.models.clone())
    }
}

/// Mock metrics collector for testing
pub struct MockMetricsCollector {
    pub metrics: Arc<RwLock<HashMap<String, f64>>>,
    pub events: Arc<RwLock<Vec<String>>>,
}

impl MockMetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn record_metric(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.insert(name.to_string(), value);
    }

    pub async fn record_event(&self, event: &str) {
        let mut events = self.events.write().await;
        events.push(event.to_string());
    }

    pub async fn get_metric(&self, name: &str) -> Option<f64> {
        let metrics = self.metrics.read().await;
        metrics.get(name).copied()
    }

    pub async fn get_events(&self) -> Vec<String> {
        let events = self.events.read().await;
        events.clone()
    }

    pub async fn clear(&self) {
        self.metrics.write().await.clear();
        self.events.write().await.clear();
    }
}

/// Mock audit system for testing
pub struct MockAuditSystem {
    pub events: Arc<RwLock<Vec<inferno::audit::AuditEvent>>>,
    pub enabled: bool,
    pub failure_rate: f64,
}

impl MockAuditSystem {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            enabled: true,
            failure_rate: 0.0,
        }
    }

    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.failure_rate = rate;
        self
    }

    pub async fn log_event(&self, event: inferno::audit::AuditEvent) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        if self.failure_rate > 0.0 {
            use rand::Rng;
            if rand::thread_rng().gen::<f64>() < self.failure_rate {
                return Err(anyhow::anyhow!("Mock audit system failed"));
            }
        }

        let mut events = self.events.write().await;
        events.push(event);
        Ok(())
    }

    pub async fn get_events(&self) -> Vec<inferno::audit::AuditEvent> {
        let events = self.events.read().await;
        events.clone()
    }

    pub async fn clear_events(&self) {
        let mut events = self.events.write().await;
        events.clear();
    }

    pub async fn count_events(&self) -> usize {
        let events = self.events.read().await;
        events.len()
    }

    pub async fn count_events_by_type(&self, event_type: &inferno::audit::EventType) -> usize {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| std::mem::discriminant(&e.event_type) == std::mem::discriminant(event_type))
            .count()
    }
}

/// Mock batch processor for testing
pub struct MockBatchProcessor {
    pub processed_jobs: Arc<RwLock<Vec<String>>>,
    pub processing_delay_ms: u64,
    pub failure_rate: f64,
    pub is_running: Arc<RwLock<bool>>,
}

impl MockBatchProcessor {
    pub fn new() -> Self {
        Self {
            processed_jobs: Arc::new(RwLock::new(Vec::new())),
            processing_delay_ms: 100,
            failure_rate: 0.0,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.failure_rate = rate;
        self
    }

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.processing_delay_ms = delay_ms;
        self
    }

    pub async fn start_processing(&self) -> Result<()> {
        *self.is_running.write().await = true;
        Ok(())
    }

    pub async fn stop_processing(&self) -> Result<()> {
        *self.is_running.write().await = false;
        Ok(())
    }

    pub async fn process_job(&self, job_id: &str) -> Result<()> {
        if !*self.is_running.read().await {
            return Err(anyhow::anyhow!("Processor not running"));
        }

        tokio::time::sleep(Duration::from_millis(self.processing_delay_ms)).await;

        if self.failure_rate > 0.0 {
            use rand::Rng;
            if rand::thread_rng().gen::<f64>() < self.failure_rate {
                return Err(anyhow::anyhow!("Mock processor failed to process job"));
            }
        }

        let mut processed = self.processed_jobs.write().await;
        processed.push(job_id.to_string());
        Ok(())
    }

    pub async fn get_processed_jobs(&self) -> Vec<String> {
        let processed = self.processed_jobs.read().await;
        processed.clone()
    }

    pub async fn clear_processed_jobs(&self) {
        let mut processed = self.processed_jobs.write().await;
        processed.clear();
    }
}

/// Mock response cache for testing
pub struct MockResponseCache {
    pub cache: Arc<RwLock<HashMap<String, (String, Instant)>>>,
    pub ttl: Duration,
    pub failure_rate: f64,
    pub hit_count: AtomicU64,
    pub miss_count: AtomicU64,
}

impl MockResponseCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(3600),
            failure_rate: 0.0,
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.failure_rate = rate;
        self
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        if self.failure_rate > 0.0 {
            use rand::Rng;
            if rand::thread_rng().gen::<f64>() < self.failure_rate {
                return Err(anyhow::anyhow!("Mock cache get failed"));
            }
        }

        let cache = self.cache.read().await;
        if let Some((value, timestamp)) = cache.get(key) {
            if timestamp.elapsed() < self.ttl {
                self.hit_count.fetch_add(1, Ordering::Relaxed);
                return Ok(Some(value.clone()));
            }
        }

        self.miss_count.fetch_add(1, Ordering::Relaxed);
        Ok(None)
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        if self.failure_rate > 0.0 {
            use rand::Rng;
            if rand::thread_rng().gen::<f64>() < self.failure_rate {
                return Err(anyhow::anyhow!("Mock cache set failed"));
            }
        }

        let mut cache = self.cache.write().await;
        cache.insert(key.to_string(), (value.to_string(), Instant::now()));
        Ok(())
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> (u64, u64) {
        (
            self.hit_count.load(Ordering::Relaxed),
            self.miss_count.load(Ordering::Relaxed),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::fixtures::{ConfigFixtures, DataFixtures};

    #[tokio::test]
    async fn test_mock_backend() -> Result<()> {
        let config = ConfigFixtures::backend_config();
        let mut backend = MockBackend::new(inferno::backends::BackendType::Gguf, config);

        assert!(!backend.is_loaded().await);

        let model_info = DataFixtures::model_info(
            "test_id",
            "Test Model",
            std::path::PathBuf::from("/test/model.gguf"),
            "gguf",
        );

        backend.load_model(&model_info).await?;
        assert!(backend.is_loaded().await);

        let params = DataFixtures::inference_params();
        let result = backend.infer("test input", &params).await?;
        assert!(result.contains("Mock response"));

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_response_cache() -> Result<()> {
        let cache = MockResponseCache::new();

        // Test miss
        let result = cache.get("nonexistent").await?;
        assert!(result.is_none());

        // Test set and hit
        cache.set("test_key", "test_value").await?;
        let result = cache.get("test_key").await?;
        assert_eq!(result, Some("test_value".to_string()));

        let (hits, misses) = cache.get_stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_audit_system() -> Result<()> {
        let audit = MockAuditSystem::new();

        let event = DataFixtures::audit_event(
            inferno::audit::EventType::ModelManagement,
            inferno::audit::Severity::Info,
        );

        audit.log_event(event).await?;

        assert_eq!(audit.count_events().await, 1);
        assert_eq!(
            audit.count_events_by_type(&inferno::audit::EventType::ModelManagement).await,
            1
        );

        Ok(())
    }
}