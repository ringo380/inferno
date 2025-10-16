//! Dynamic Worker Pool Manager
//!
//! This module manages inference worker pools with automatic scaling based on queue pressure
//! and GPU memory availability.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Worker state
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum WorkerState {
    Idle,
    Active,
    Busy,
    Failed,
}

/// Individual worker metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerMetrics {
    pub worker_id: u32,
    pub state: WorkerState,
    pub active_requests: u32,
    pub total_processed: u64,
    pub total_failed: u64,
    pub gpu_memory_used_mb: u32,
    pub cpu_memory_used_mb: u32,
}

/// Worker pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPoolConfig {
    pub model_id: String,
    pub min_workers: usize,
    pub max_workers: usize,
    pub target_latency_ms: u32,
    pub estimated_gpu_memory_per_worker_mb: u32,
}

impl WorkerPoolConfig {
    /// Create default config for a model
    pub fn new(model_id: String) -> Self {
        Self {
            model_id,
            min_workers: 1,
            max_workers: 16,
            target_latency_ms: 250,
            estimated_gpu_memory_per_worker_mb: 4096, // ~4GB per worker estimate
        }
    }

    /// Set minimum workers
    pub fn with_min_workers(mut self, min: usize) -> Self {
        self.min_workers = min;
        self
    }

    /// Set maximum workers
    pub fn with_max_workers(mut self, max: usize) -> Self {
        self.max_workers = max;
        self
    }

    /// Set target latency
    pub fn with_target_latency_ms(mut self, latency: u32) -> Self {
        self.target_latency_ms = latency;
        self
    }

    /// Set GPU memory estimate
    pub fn with_gpu_memory_estimate_mb(mut self, memory: u32) -> Self {
        self.estimated_gpu_memory_per_worker_mb = memory;
        self
    }
}

/// Worker pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPoolStats {
    pub model_id: String,
    pub total_workers: usize,
    pub active_workers: usize,
    pub idle_workers: usize,
    pub failed_workers: usize,
    pub current_load: f32, // 0.0-1.0
    pub total_processed: u64,
    pub total_failed: u64,
    pub avg_request_duration_ms: f32,
    pub total_gpu_memory_used_mb: u32,
}

/// Dynamic worker pool manager
#[derive(Debug)]
pub struct WorkerPool {
    config: WorkerPoolConfig,
    workers: Vec<Arc<AtomicU32>>, // Simple worker IDs
    worker_metrics: HashMap<u32, WorkerMetrics>,
    next_worker_id: u32,
    current_load: f32,
    scale_up_threshold: f32,
    scale_down_threshold: f32,
    last_scale_change_secs: u64,
}

impl WorkerPool {
    /// Create a new worker pool
    pub fn new(config: WorkerPoolConfig) -> Self {
        let mut pool = Self {
            config,
            workers: Vec::new(),
            worker_metrics: HashMap::new(),
            next_worker_id: 0,
            current_load: 0.0,
            scale_up_threshold: 0.8,     // Scale up when 80% loaded
            scale_down_threshold: 0.2,   // Scale down when 20% loaded
            last_scale_change_secs: 0,
        };

        // Initialize with minimum workers
        for _ in 0..pool.config.min_workers {
            pool.create_worker();
        }

        pool
    }

    /// Create a new worker
    fn create_worker(&mut self) {
        let worker_id = self.next_worker_id;
        self.next_worker_id = self.next_worker_id.wrapping_add(1);

        self.workers.push(Arc::new(AtomicU32::new(worker_id)));

        self.worker_metrics.insert(
            worker_id,
            WorkerMetrics {
                worker_id,
                state: WorkerState::Idle,
                active_requests: 0,
                total_processed: 0,
                total_failed: 0,
                gpu_memory_used_mb: 0,
                cpu_memory_used_mb: 512, // Base memory
            },
        );
    }

    /// Get least loaded worker that can take a request
    pub fn get_least_loaded_worker(&mut self) -> Option<u32> {
        if self.workers.is_empty() {
            return None;
        }

        // Find worker with lowest load
        let mut best_worker_id = None;
        let mut best_load = f32::MAX;

        for (worker_id, metrics) in &self.worker_metrics {
            if metrics.state != WorkerState::Failed {
                let load = metrics.active_requests as f32 / 10.0; // Normalized
                if load < best_load {
                    best_load = load;
                    best_worker_id = Some(*worker_id);
                }
            }
        }

        best_worker_id
    }

    /// Assign request to a worker
    pub fn assign_request(&mut self, worker_id: u32) -> bool {
        if let Some(metrics) = self.worker_metrics.get_mut(&worker_id) {
            if metrics.state != WorkerState::Failed {
                metrics.active_requests += 1;
                metrics.state = WorkerState::Active;
                self.update_load();
                return true;
            }
        }
        false
    }

    /// Complete a request on a worker
    pub fn complete_request(&mut self, worker_id: u32, success: bool) {
        if let Some(metrics) = self.worker_metrics.get_mut(&worker_id) {
            metrics.active_requests = metrics.active_requests.saturating_sub(1);

            if success {
                metrics.total_processed += 1;
                metrics.state = if metrics.active_requests > 0 {
                    WorkerState::Busy
                } else {
                    WorkerState::Idle
                };
            } else {
                metrics.total_failed += 1;
            }

            self.update_load();
        }
    }

    /// Auto-scale workers based on load and queue depth
    pub fn auto_scale(
        &mut self,
        queue_depth: usize,
        avg_latency_ms: f32,
        available_gpu_memory_mb: u32,
    ) {
        let current_workers = self.workers.len();

        // Scale up conditions
        let should_scale_up = queue_depth > current_workers * 10
            || (avg_latency_ms > self.config.target_latency_ms as f32
                && current_workers < self.config.max_workers);

        if should_scale_up && current_workers < self.config.max_workers {
            // Check if we have enough GPU memory
            let required_memory = self.config.estimated_gpu_memory_per_worker_mb;
            if available_gpu_memory_mb > required_memory {
                self.create_worker();
            }
        }

        // Scale down conditions
        let idle_workers = self
            .worker_metrics
            .values()
            .filter(|m| m.state == WorkerState::Idle && m.active_requests == 0)
            .count();

        if idle_workers > 0
            && current_workers > self.config.min_workers
            && self.current_load < self.scale_down_threshold
        {
            // Remove one idle worker
            self.remove_idle_worker();
        }
    }

    /// Remove an idle worker
    fn remove_idle_worker(&mut self) {
        // Find first idle worker
        if let Some(pos) = self.workers.iter().position(|_| {
            // Find idle worker in metrics
            if let Some(metrics) = self.worker_metrics.values().find(|m| m.state == WorkerState::Idle) {
                return metrics.active_requests == 0;
            }
            false
        }) {
            self.workers.remove(pos);
        }
    }

    /// Update current load calculation
    fn update_load(&mut self) {
        let total_capacity = self.workers.len() * 10; // Each worker can handle ~10 requests
        let total_active: u32 = self
            .worker_metrics
            .values()
            .map(|m| m.active_requests)
            .sum();

        self.current_load = if total_capacity > 0 {
            (total_active as f32 / total_capacity as f32).min(1.0)
        } else {
            0.0
        };
    }

    /// Get pool statistics
    pub fn stats(&self) -> WorkerPoolStats {
        let active_workers = self
            .worker_metrics
            .values()
            .filter(|m| m.state == WorkerState::Active || m.state == WorkerState::Busy)
            .count();

        let idle_workers = self
            .worker_metrics
            .values()
            .filter(|m| m.state == WorkerState::Idle)
            .count();

        let failed_workers = self
            .worker_metrics
            .values()
            .filter(|m| m.state == WorkerState::Failed)
            .count();

        let total_processed: u64 = self.worker_metrics.values().map(|m| m.total_processed).sum();
        let total_failed: u64 = self.worker_metrics.values().map(|m| m.total_failed).sum();
        let total_gpu_memory: u32 = self
            .worker_metrics
            .values()
            .map(|m| m.gpu_memory_used_mb)
            .sum();

        WorkerPoolStats {
            model_id: self.config.model_id.clone(),
            total_workers: self.workers.len(),
            active_workers,
            idle_workers,
            failed_workers,
            current_load: self.current_load,
            total_processed,
            total_failed,
            avg_request_duration_ms: (total_processed as f32).max(1.0) / 100.0, // Placeholder
            total_gpu_memory_used_mb: total_gpu_memory,
        }
    }

    /// Get pool size
    pub fn len(&self) -> usize {
        self.workers.len()
    }

    /// Check if pool is empty
    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }

    /// Get worker metrics
    pub fn worker_metrics(&self) -> Vec<WorkerMetrics> {
        self.worker_metrics.values().cloned().collect()
    }

    /// Check if pool has capacity for new work
    pub fn has_capacity(&self) -> bool {
        self.current_load < 0.95
    }
}

/// Worker pool registry for managing multiple pools per model
#[derive(Debug)]
pub struct WorkerPoolRegistry {
    pools: HashMap<String, WorkerPool>,
}

impl WorkerPoolRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
        }
    }

    /// Get or create pool for a model
    pub fn get_or_create(&mut self, config: WorkerPoolConfig) -> &mut WorkerPool {
        let model_id = config.model_id.clone();
        self.pools.entry(model_id).or_insert_with(|| WorkerPool::new(config))
    }

    /// Get existing pool
    pub fn get(&mut self, model_id: &str) -> Option<&mut WorkerPool> {
        self.pools.get_mut(model_id)
    }

    /// Remove pool
    pub fn remove(&mut self, model_id: &str) -> Option<WorkerPool> {
        self.pools.remove(model_id)
    }

    /// Get all pools
    pub fn all(&self) -> Vec<&WorkerPool> {
        self.pools.values().collect()
    }

    /// Get registry statistics
    pub fn stats(&self) -> HashMap<String, WorkerPoolStats> {
        self.pools
            .iter()
            .map(|(model_id, pool)| (model_id.clone(), pool.stats()))
            .collect()
    }
}

impl Default for WorkerPoolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_pool_creation() {
        let config = WorkerPoolConfig::new("llama-2-7b".to_string()).with_min_workers(2);
        let pool = WorkerPool::new(config);

        assert_eq!(pool.len(), 2);
        assert!(!pool.is_empty());
    }

    #[test]
    fn test_least_loaded_worker() {
        let config = WorkerPoolConfig::new("llama-2-7b".to_string()).with_min_workers(3);
        let mut pool = WorkerPool::new(config);

        let worker1 = pool.get_least_loaded_worker().unwrap();
        pool.assign_request(worker1);
        pool.assign_request(worker1);

        let worker2 = pool.get_least_loaded_worker().unwrap();
        assert_ne!(worker1, worker2); // Should pick different worker

        pool.assign_request(worker2);

        let worker3 = pool.get_least_loaded_worker().unwrap();
        assert_ne!(worker3, worker1);
        assert_ne!(worker3, worker2);
    }

    #[test]
    fn test_auto_scaling() {
        let config = WorkerPoolConfig::new("llama-2-7b".to_string())
            .with_min_workers(1)
            .with_max_workers(5)
            .with_target_latency_ms(200);

        let mut pool = WorkerPool::new(config);
        assert_eq!(pool.len(), 1);

        // High load and latency should trigger scale up
        pool.auto_scale(50, 300.0, 10_000); // 50 queued, high latency, lots of memory
        assert!(pool.len() >= 2);
    }

    #[test]
    fn test_pool_statistics() {
        let config = WorkerPoolConfig::new("llama-2-7b".to_string());
        let mut pool = WorkerPool::new(config);

        if let Some(worker_id) = pool.get_least_loaded_worker() {
            pool.assign_request(worker_id);
            pool.complete_request(worker_id, true);
        }

        let stats = pool.stats();
        assert!(stats.total_processed > 0);
        assert_eq!(stats.total_failed, 0);
    }

    #[test]
    fn test_registry() {
        let mut registry = WorkerPoolRegistry::new();

        let config1 = WorkerPoolConfig::new("model1".to_string());
        let config2 = WorkerPoolConfig::new("model2".to_string());

        registry.get_or_create(config1);
        registry.get_or_create(config2);

        assert!(registry.get("model1").is_some());
        assert!(registry.get("model2").is_some());
        assert!(registry.get("model3").is_none());
    }
}
