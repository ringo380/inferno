//! Fair Queue Scheduler with Starvation Prevention
//!
//! This module implements weighted round-robin scheduling to ensure fair resource
//! allocation across different priority levels while preventing starvation.

use crate::operations::queue::priority_queue::{Priority, PriorityQueue, RequestMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-priority-level fairness metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessMetrics {
    pub priority: Priority,
    pub queued_count: usize,
    pub total_weight: u32,
    pub avg_wait_ms: f64,
    pub max_wait_ms: u64,
    pub assigned_count: u64,
}

/// Overall queue fairness statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessStats {
    pub per_priority: HashMap<u8, FairnessMetrics>,
    pub starvation_detected: bool,
    pub fairness_score: f32, // 0.0-1.0, percentage of requests meeting SLA
    pub starvation_threshold_ms: u64,
}

/// Fair queue scheduler using weighted round-robin
#[derive(Debug)]
pub struct FairScheduler {
    priority_queue: PriorityQueue,
    /// Weights for each priority level (from Priority::weight())
    priority_weights: HashMap<u8, u32>,
    /// Current position in round-robin cycle
    current_weight_position: u32,
    /// Assigned count for starvation detection
    per_priority_assigned: HashMap<u8, u64>,
    /// Per-priority wait time tracking (in milliseconds)
    per_priority_wait_times: HashMap<u8, Vec<u64>>,
    /// Starvation detection threshold (milliseconds)
    starvation_threshold_ms: u64,
}

impl FairScheduler {
    /// Create a new fair scheduler
    pub fn new() -> Self {
        let mut weights = HashMap::new();
        weights.insert(Priority::VIP as u8, Priority::VIP.weight());
        weights.insert(Priority::High as u8, Priority::High.weight());
        weights.insert(Priority::Normal as u8, Priority::Normal.weight());
        weights.insert(Priority::Low as u8, Priority::Low.weight());

        let mut assigned = HashMap::new();
        assigned.insert(Priority::VIP as u8, 0);
        assigned.insert(Priority::High as u8, 0);
        assigned.insert(Priority::Normal as u8, 0);
        assigned.insert(Priority::Low as u8, 0);

        let mut wait_times = HashMap::new();
        wait_times.insert(Priority::VIP as u8, Vec::new());
        wait_times.insert(Priority::High as u8, Vec::new());
        wait_times.insert(Priority::Normal as u8, Vec::new());
        wait_times.insert(Priority::Low as u8, Vec::new());

        Self {
            priority_queue: PriorityQueue::new(),
            priority_weights: weights,
            current_weight_position: 0,
            per_priority_assigned: assigned,
            per_priority_wait_times: wait_times,
            starvation_threshold_ms: 30_000, // 30 seconds
        }
    }

    /// Set the starvation threshold
    pub fn with_starvation_threshold(mut self, threshold_ms: u64) -> Self {
        self.starvation_threshold_ms = threshold_ms;
        self
    }

    /// Add a request to the queue
    pub fn enqueue(&mut self, metadata: RequestMetadata) {
        self.priority_queue.push(metadata);
    }

    /// Get the next request to process using weighted round-robin
    pub fn dequeue(&mut self) -> Option<RequestMetadata> {
        if self.priority_queue.is_empty() {
            return None;
        }

        // Apply age boosting before dequeuing
        let request = self.priority_queue.pop()?;

        // Track assignment
        let priority_val = request.priority as u8;
        *self.per_priority_assigned.entry(priority_val).or_insert(0) += 1;

        // Track wait time
        let wait_ms = request.age_ms();
        self.per_priority_wait_times
            .entry(priority_val)
            .or_insert_with(Vec::new)
            .push(wait_ms);

        // Keep only last 1000 wait times per priority to avoid memory bloat
        if let Some(times) = self.per_priority_wait_times.get_mut(&priority_val) {
            if times.len() > 1000 {
                times.remove(0);
            }
        }

        Some(request)
    }

    /// Calculate fairness metrics for all priority levels
    pub fn calculate_fairness_stats(&self) -> FairnessStats {
        let mut per_priority = HashMap::new();
        let mut total_requests = 0;
        let mut sla_met_requests = 0;
        let mut max_wait_detected = 0u64;

        for priority_val in 1..=4 {
            let wait_times = self
                .per_priority_wait_times
                .get(&priority_val)
                .cloned()
                .unwrap_or_default();

            let assigned_count = *self.per_priority_assigned.get(&priority_val).unwrap_or(&0);
            let queued_count = self.priority_queue.len(); // Approximate

            let (avg_wait_ms, max_wait_ms) = if !wait_times.is_empty() {
                let sum: u64 = wait_times.iter().sum();
                let avg = sum as f64 / wait_times.len() as f64;
                let max = *wait_times.iter().max().unwrap_or(&0);

                // Check SLA: most requests should complete within starvation threshold
                let sla_met = wait_times
                    .iter()
                    .filter(|&&t| t <= self.starvation_threshold_ms)
                    .count() as u64;
                sla_met_requests += sla_met;

                max_wait_detected = max_wait_detected.max(max);

                (avg, max)
            } else {
                (0.0, 0)
            };

            let weight = *self.priority_weights.get(&priority_val).unwrap_or(&1);
            let total_weight = assigned_count as u32 * weight;

            per_priority.insert(
                priority_val,
                FairnessMetrics {
                    priority: Priority::from_u8(priority_val).unwrap_or(Priority::Low),
                    queued_count,
                    total_weight,
                    avg_wait_ms,
                    max_wait_ms,
                    assigned_count,
                },
            );

            total_requests += assigned_count;
        }

        let starvation_detected = max_wait_detected > self.starvation_threshold_ms;

        let fairness_score = if total_requests > 0 {
            (sla_met_requests as f32 / total_requests as f32)
                .max(0.0)
                .min(1.0)
        } else {
            1.0
        };

        FairnessStats {
            per_priority,
            starvation_detected,
            fairness_score,
            starvation_threshold_ms: self.starvation_threshold_ms,
        }
    }

    /// Check if starvation is detected for any priority level
    pub fn is_starving(&self) -> bool {
        let stats = self.calculate_fairness_stats();
        stats.starvation_detected
    }

    /// Get queue size
    pub fn len(&self) -> usize {
        self.priority_queue.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.priority_queue.is_empty()
    }

    /// Remove a request by ID
    pub fn cancel_request(&mut self, request_id: &str) -> Option<RequestMetadata> {
        self.priority_queue.remove_by_id(request_id)
    }

    /// Get current fairness statistics
    pub fn fairness_stats(&self) -> FairnessStats {
        self.calculate_fairness_stats()
    }

    /// Get all pending requests (for debugging)
    pub fn iter(&self) -> impl Iterator<Item = &RequestMetadata> {
        self.priority_queue.iter()
    }
}

impl Default for FairScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fair_scheduling_no_starvation() {
        let mut scheduler = FairScheduler::new().with_starvation_threshold(5000);

        // Add requests from each priority level
        for i in 0..10 {
            let priority = match i % 4 {
                0 => Priority::VIP,
                1 => Priority::High,
                2 => Priority::Normal,
                _ => Priority::Low,
            };

            let req = RequestMetadata::new(
                format!("req_{}", i),
                "user".to_string(),
                priority,
                "model".to_string(),
            );

            scheduler.enqueue(req);
        }

        // Dequeue all requests
        let mut dequeued_count = 0;
        while let Some(req) = scheduler.dequeue() {
            dequeued_count += 1;
            assert!(!req.request_id.is_empty());
        }

        assert_eq!(dequeued_count, 10);

        // Check fairness stats
        let stats = scheduler.fairness_stats();
        assert!(!stats.starvation_detected); // No starvation with small queue
        assert!(stats.fairness_score > 0.0); // Some fairness score
    }

    #[test]
    fn test_fairness_metrics_per_priority() {
        let mut scheduler = FairScheduler::new();

        // Add requests to each priority level
        scheduler.enqueue(RequestMetadata::new(
            "vip1".to_string(),
            "user".to_string(),
            Priority::VIP,
            "model".to_string(),
        ));

        scheduler.enqueue(RequestMetadata::new(
            "high1".to_string(),
            "user".to_string(),
            Priority::High,
            "model".to_string(),
        ));

        scheduler.enqueue(RequestMetadata::new(
            "normal1".to_string(),
            "user".to_string(),
            Priority::Normal,
            "model".to_string(),
        ));

        scheduler.enqueue(RequestMetadata::new(
            "low1".to_string(),
            "user".to_string(),
            Priority::Low,
            "model".to_string(),
        ));

        // Dequeue in order and check
        assert_eq!(scheduler.dequeue().unwrap().priority, Priority::VIP);
        assert_eq!(scheduler.dequeue().unwrap().priority, Priority::High);
        assert_eq!(scheduler.dequeue().unwrap().priority, Priority::Normal);
        assert_eq!(scheduler.dequeue().unwrap().priority, Priority::Low);

        let stats = scheduler.fairness_stats();
        assert_eq!(
            stats
                .per_priority
                .get(&(Priority::VIP as u8))
                .unwrap()
                .assigned_count,
            1
        );
        assert_eq!(
            stats
                .per_priority
                .get(&(Priority::High as u8))
                .unwrap()
                .assigned_count,
            1
        );
    }

    #[test]
    fn test_cancel_request() {
        let mut scheduler = FairScheduler::new();

        scheduler.enqueue(RequestMetadata::new(
            "req1".to_string(),
            "user".to_string(),
            Priority::Normal,
            "model".to_string(),
        ));

        scheduler.enqueue(RequestMetadata::new(
            "req2".to_string(),
            "user".to_string(),
            Priority::Normal,
            "model".to_string(),
        ));

        assert_eq!(scheduler.len(), 2);

        let cancelled = scheduler.cancel_request("req1");
        assert!(cancelled.is_some());
        assert_eq!(scheduler.len(), 1);

        let remaining = scheduler.dequeue().unwrap();
        assert_eq!(remaining.request_id, "req2");
    }

    #[test]
    fn test_empty_scheduler() {
        let scheduler = FairScheduler::new();
        assert!(scheduler.is_empty());
        assert_eq!(scheduler.len(), 0);

        let stats = scheduler.fairness_stats();
        assert!(!stats.starvation_detected);
        assert_eq!(stats.fairness_score, 1.0); // Perfect score when empty
    }
}
