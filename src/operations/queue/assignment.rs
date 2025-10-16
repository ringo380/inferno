//! Request Assignment & Load Balancing
//!
//! This module handles intelligent assignment of queued requests to available workers
//! and implements backpressure mechanisms to prevent overload.

use crate::operations::queue::priority_queue::RequestMetadata;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Assignment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentResult {
    pub request_id: String,
    pub assigned_worker_id: u32,
    pub estimated_duration_ms: u32,
}

/// Backpressure status
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum BackpressureStatus {
    /// Queue is healthy
    Healthy,
    /// Queue approaching capacity (warning)
    Elevated,
    /// Queue is full or GPU memory low
    Critical,
}

/// Assignment strategy for load balancing
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum AssignmentStrategy {
    /// Assign to least loaded worker
    LeastLoaded,
    /// Assign to worker with earliest completion time
    EarliestCompletion,
    /// Distribute evenly across workers
    RoundRobin,
}

/// Request grouping for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestGroup {
    pub requests: Vec<String>, // request IDs
    pub model_id: String,
    pub total_tokens: u32,
    pub batch_size: usize,
    pub priority: u8,
}

/// Load balancer for assigning requests to workers
#[derive(Debug)]
pub struct LoadBalancer {
    strategy: AssignmentStrategy,
    /// Per-worker: current active requests
    worker_load: HashMap<u32, u32>,
    /// Per-worker: estimated completion time (ms)
    worker_eta: HashMap<u32, u64>,
    /// Per-worker: GPU memory used (MB)
    worker_gpu_memory: HashMap<u32, u32>,
    /// Queue configuration
    max_queue_depth: usize,
    min_gpu_memory_free_mb: u32,
    /// Batch grouping window (ms)
    batch_grouping_window_ms: u32,
    /// Maximum batch size
    max_batch_size: usize,
    /// Request grouping
    pending_groups: VecDeque<RequestGroup>,
}

impl LoadBalancer {
    /// Create new load balancer
    pub fn new(strategy: AssignmentStrategy) -> Self {
        Self {
            strategy,
            worker_load: HashMap::new(),
            worker_eta: HashMap::new(),
            worker_gpu_memory: HashMap::new(),
            max_queue_depth: 10_000,
            min_gpu_memory_free_mb: 512,
            batch_grouping_window_ms: 50,
            max_batch_size: 32,
            pending_groups: VecDeque::new(),
        }
    }

    /// Set maximum queue depth
    pub fn with_max_queue_depth(mut self, depth: usize) -> Self {
        self.max_queue_depth = depth;
        self
    }

    /// Set minimum free GPU memory threshold
    pub fn with_min_gpu_memory_mb(mut self, memory: u32) -> Self {
        self.min_gpu_memory_free_mb = memory;
        self
    }

    /// Set batch grouping window
    pub fn with_batch_grouping_window_ms(mut self, window: u32) -> Self {
        self.batch_grouping_window_ms = window;
        self
    }

    /// Set maximum batch size
    pub fn with_max_batch_size(mut self, size: usize) -> Self {
        self.max_batch_size = size;
        self
    }

    /// Register a worker
    pub fn register_worker(&mut self, worker_id: u32) {
        self.worker_load.insert(worker_id, 0);
        self.worker_eta.insert(worker_id, 0);
        self.worker_gpu_memory.insert(worker_id, 0);
    }

    /// Unregister a worker
    pub fn unregister_worker(&mut self, worker_id: u32) {
        self.worker_load.remove(&worker_id);
        self.worker_eta.remove(&worker_id);
        self.worker_gpu_memory.remove(&worker_id);
    }

    /// Update worker metrics
    pub fn update_worker_metrics(
        &mut self,
        worker_id: u32,
        active_requests: u32,
        estimated_completion_ms: u64,
        gpu_memory_mb: u32,
    ) {
        self.worker_load.insert(worker_id, active_requests);
        self.worker_eta.insert(worker_id, estimated_completion_ms);
        self.worker_gpu_memory.insert(worker_id, gpu_memory_mb);
    }

    /// Assign request to best worker using configured strategy
    pub fn assign_request(
        &self,
        request: &RequestMetadata,
        available_gpu_memory_mb: u32,
    ) -> Option<AssignmentResult> {
        // Check backpressure conditions
        if available_gpu_memory_mb < self.min_gpu_memory_free_mb {
            return None; // GPU memory insufficient
        }

        if self.worker_load.is_empty() {
            return None; // No workers available
        }

        let assigned_worker = match self.strategy {
            AssignmentStrategy::LeastLoaded => self.find_least_loaded_worker(),
            AssignmentStrategy::EarliestCompletion => self.find_earliest_completion_worker(),
            AssignmentStrategy::RoundRobin => self.find_next_worker(),
        }?;

        // Estimate processing time based on tokens and worker load
        let tokens = request.estimated_tokens;
        let estimated_tokens_per_sec = 50; // Average
        let estimated_duration_ms = ((tokens as u32 / estimated_tokens_per_sec) * 1000) as u32;

        Some(AssignmentResult {
            request_id: request.request_id.clone(),
            assigned_worker_id: assigned_worker,
            estimated_duration_ms,
        })
    }

    /// Find least loaded worker
    fn find_least_loaded_worker(&self) -> Option<u32> {
        self.worker_load
            .iter()
            .min_by_key(|(_, &load)| load)
            .map(|(&id, _)| id)
    }

    /// Find worker with earliest completion time
    fn find_earliest_completion_worker(&self) -> Option<u32> {
        self.worker_eta
            .iter()
            .min_by_key(|(_, &eta)| eta)
            .map(|(&id, _)| id)
    }

    /// Round-robin worker selection (simple)
    fn find_next_worker(&self) -> Option<u32> {
        self.worker_load.keys().next().copied()
    }

    /// Group requests for batch processing
    pub fn group_requests(
        &mut self,
        requests: Vec<RequestMetadata>,
        model_id: &str,
    ) -> Vec<RequestGroup> {
        let mut groups: HashMap<u8, Vec<String>> = HashMap::new();

        for request in requests {
            groups
                .entry(request.priority as u8)
                .or_insert_with(Vec::new)
                .push(request.request_id);
        }

        let mut result = Vec::new();

        for (priority, mut request_ids) in groups {
            // Split into chunks of max_batch_size
            while !request_ids.is_empty() {
                let chunk_size = request_ids.len().min(self.max_batch_size);
                let chunk: Vec<String> = request_ids.drain(0..chunk_size).collect();

                // Estimate total tokens
                let total_tokens = chunk.len() as u32 * 256; // Approximate

                result.push(RequestGroup {
                    requests: chunk.clone(),
                    model_id: model_id.to_string(),
                    total_tokens,
                    batch_size: chunk.len(),
                    priority,
                });
            }
        }

        result
    }

    /// Check backpressure status
    pub fn check_backpressure(
        &self,
        current_queue_depth: usize,
        available_gpu_memory_mb: u32,
    ) -> BackpressureStatus {
        let queue_utilization = current_queue_depth as f32 / self.max_queue_depth as f32;

        match (queue_utilization, available_gpu_memory_mb < self.min_gpu_memory_free_mb) {
            (util, true) if util > 0.8 => BackpressureStatus::Critical,
            (util, true) => BackpressureStatus::Elevated,
            (util, _) if util > 0.9 => BackpressureStatus::Critical,
            (util, _) if util > 0.7 => BackpressureStatus::Elevated,
            _ => BackpressureStatus::Healthy,
        }
    }

    /// Get current load statistics
    pub fn load_stats(&self) -> LoadStats {
        let total_load: u32 = self.worker_load.values().sum();
        let worker_count = self.worker_load.len();
        let avg_load = if worker_count > 0 {
            total_load as f32 / worker_count as f32
        } else {
            0.0
        };

        let total_gpu_memory: u32 = self.worker_gpu_memory.values().sum();

        LoadStats {
            total_load,
            worker_count,
            avg_load_per_worker: avg_load,
            total_gpu_memory_used_mb: total_gpu_memory,
        }
    }
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new(AssignmentStrategy::LeastLoaded)
    }
}

/// Load statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadStats {
    pub total_load: u32,
    pub worker_count: usize,
    pub avg_load_per_worker: f32,
    pub total_gpu_memory_used_mb: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::queue::Priority;

    #[test]
    fn test_load_balancer_creation() {
        let lb = LoadBalancer::new(AssignmentStrategy::LeastLoaded);
        assert_eq!(lb.max_queue_depth, 10_000);
    }

    #[test]
    fn test_worker_registration() {
        let mut lb = LoadBalancer::new(AssignmentStrategy::LeastLoaded);

        lb.register_worker(1);
        lb.register_worker(2);
        lb.register_worker(3);

        assert!(lb.worker_load.contains_key(&1));
        assert!(lb.worker_load.contains_key(&2));
        assert!(lb.worker_load.contains_key(&3));
    }

    #[test]
    fn test_least_loaded_assignment() {
        let mut lb = LoadBalancer::new(AssignmentStrategy::LeastLoaded);

        lb.register_worker(1);
        lb.register_worker(2);
        lb.register_worker(3);

        lb.update_worker_metrics(1, 10, 1000, 4096);
        lb.update_worker_metrics(2, 5, 500, 4096);
        lb.update_worker_metrics(3, 15, 2000, 4096);

        let least_loaded = lb.find_least_loaded_worker();
        assert_eq!(least_loaded, Some(2)); // Worker 2 has load of 5
    }

    #[test]
    fn test_earliest_completion_assignment() {
        let mut lb = LoadBalancer::new(AssignmentStrategy::EarliestCompletion);

        lb.register_worker(1);
        lb.register_worker(2);
        lb.register_worker(3);

        lb.update_worker_metrics(1, 5, 2000, 4096);
        lb.update_worker_metrics(2, 10, 500, 4096);
        lb.update_worker_metrics(3, 8, 1500, 4096);

        let earliest = lb.find_earliest_completion_worker();
        assert_eq!(earliest, Some(2)); // Earliest ETA is 500ms
    }

    #[test]
    fn test_backpressure_detection() {
        let lb = LoadBalancer::new(AssignmentStrategy::LeastLoaded);

        let status = lb.check_backpressure(100, 1024);
        assert_eq!(status, BackpressureStatus::Healthy);

        let status = lb.check_backpressure(7000, 1024);
        assert_eq!(status, BackpressureStatus::Elevated);

        let status = lb.check_backpressure(9500, 100);
        assert_eq!(status, BackpressureStatus::Critical);
    }

    #[test]
    fn test_request_grouping() {
        let mut lb = LoadBalancer::new(AssignmentStrategy::LeastLoaded);

        let mut requests = Vec::new();
        for i in 0..5 {
            requests.push(RequestMetadata::new(
                format!("req_{}", i),
                "user".to_string(),
                Priority::Normal,
                "model".to_string(),
            ));
        }

        let groups = lb.group_requests(requests, "model");
        assert!(!groups.is_empty());
        assert_eq!(groups[0].model_id, "model");
    }

    #[test]
    fn test_load_stats() {
        let mut lb = LoadBalancer::new(AssignmentStrategy::LeastLoaded);

        lb.register_worker(1);
        lb.register_worker(2);

        lb.update_worker_metrics(1, 5, 1000, 4096);
        lb.update_worker_metrics(2, 10, 2000, 4096);

        let stats = lb.load_stats();
        assert_eq!(stats.total_load, 15);
        assert_eq!(stats.worker_count, 2);
        assert_eq!(stats.avg_load_per_worker, 7.5);
    }
}
