//! Queue Metrics & Monitoring
//!
//! This module provides comprehensive metrics and monitoring for the queue system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-request metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub request_id: String,
    pub queued_duration_ms: u64,
    pub processing_duration_ms: u64,
    pub total_duration_ms: u64,
    pub queue_position_when_added: usize,
}

/// Per-priority metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityMetrics {
    pub priority_level: u8,
    pub total_queued: u64,
    pub total_processed: u64,
    pub avg_wait_time_ms: f64,
    pub p50_wait_time_ms: f64,
    pub p95_wait_time_ms: f64,
    pub p99_wait_time_ms: f64,
}

/// Overall queue metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMetricsSnapshot {
    pub timestamp_ms: u64,
    pub total_queued: u64,
    pub total_processed: u64,
    pub current_queue_depth: usize,
    pub avg_queue_depth: f64,
    pub max_queue_depth: usize,
    pub per_priority: HashMap<u8, PriorityMetrics>,
    pub throughput_requests_per_sec: f32,
    pub avg_latency_ms: f64,
}

/// Queue metrics collector
#[derive(Debug)]
pub struct QueueMetricsCollector {
    total_queued: u64,
    total_processed: u64,
    queue_depth_history: Vec<usize>,
    max_queue_depth: usize,
    per_priority_metrics: HashMap<u8, Vec<u64>>, // wait times per priority
    start_time_ms: u64,
}

impl QueueMetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            total_queued: 0,
            total_processed: 0,
            queue_depth_history: Vec::new(),
            max_queue_depth: 0,
            per_priority_metrics: HashMap::new(),
            start_time_ms: Self::current_timestamp(),
        }
    }

    /// Record a request being queued
    pub fn record_queued(&mut self, priority: u8) {
        self.total_queued += 1;
        self.per_priority_metrics
            .entry(priority)
            .or_default();
    }

    /// Record a request being processed
    pub fn record_processed(&mut self, priority: u8, wait_time_ms: u64) {
        self.total_processed += 1;
        self.per_priority_metrics
            .entry(priority)
            .or_default()
            .push(wait_time_ms);
    }

    /// Record current queue depth
    pub fn record_queue_depth(&mut self, depth: usize) {
        self.queue_depth_history.push(depth);
        self.max_queue_depth = self.max_queue_depth.max(depth);

        // Keep only last 10000 readings to avoid memory bloat
        if self.queue_depth_history.len() > 10000 {
            self.queue_depth_history.remove(0);
        }
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self, current_queue_depth: usize) -> QueueMetricsSnapshot {
        let elapsed_ms = Self::current_timestamp().saturating_sub(self.start_time_ms);
        let elapsed_secs = (elapsed_ms as f32 / 1000.0).max(1.0);

        let avg_queue_depth = if self.queue_depth_history.is_empty() {
            current_queue_depth as f64
        } else {
            let sum: usize = self.queue_depth_history.iter().sum();
            sum as f64 / self.queue_depth_history.len() as f64
        };

        let avg_latency_ms = if self.total_processed > 0 {
            let total_wait: u64 = self
                .per_priority_metrics
                .values()
                .flat_map(|v| v.iter())
                .sum();
            total_wait as f64 / self.total_processed as f64
        } else {
            0.0
        };

        let throughput = self.total_processed as f32 / elapsed_secs;

        let mut per_priority = HashMap::new();
        for (priority, wait_times) in &self.per_priority_metrics {
            if !wait_times.is_empty() {
                let mut sorted = wait_times.clone();
                sorted.sort_unstable();

                let sum: u64 = sorted.iter().sum();
                let avg = sum as f64 / sorted.len() as f64;
                let p50 = sorted[sorted.len() / 2] as f64;
                let p95_idx = (sorted.len() as f64 * 0.95) as usize;
                let p95 = sorted[p95_idx.min(sorted.len() - 1)] as f64;
                let p99_idx = (sorted.len() as f64 * 0.99) as usize;
                let p99 = sorted[p99_idx.min(sorted.len() - 1)] as f64;

                per_priority.insert(
                    *priority,
                    PriorityMetrics {
                        priority_level: *priority,
                        total_queued: self.total_queued,
                        total_processed: self.total_processed,
                        avg_wait_time_ms: avg,
                        p50_wait_time_ms: p50,
                        p95_wait_time_ms: p95,
                        p99_wait_time_ms: p99,
                    },
                );
            }
        }

        QueueMetricsSnapshot {
            timestamp_ms: Self::current_timestamp(),
            total_queued: self.total_queued,
            total_processed: self.total_processed,
            current_queue_depth,
            avg_queue_depth,
            max_queue_depth: self.max_queue_depth,
            per_priority,
            throughput_requests_per_sec: throughput,
            avg_latency_ms,
        }
    }

    /// Get current Unix timestamp in milliseconds
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.total_queued = 0;
        self.total_processed = 0;
        self.queue_depth_history.clear();
        self.max_queue_depth = 0;
        self.per_priority_metrics.clear();
        self.start_time_ms = Self::current_timestamp();
    }
}

impl Default for QueueMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collection() {
        let mut collector = QueueMetricsCollector::new();

        // Record some activity
        for i in 0..10 {
            let priority = ((i % 4) + 1) as u8;
            collector.record_queued(priority);
            collector.record_queue_depth(i);
            collector.record_processed(priority, (i as u64 + 1) * 10);
        }

        let snapshot = collector.snapshot(5);
        assert_eq!(snapshot.total_queued, 10);
        assert_eq!(snapshot.total_processed, 10);
        assert!(snapshot.throughput_requests_per_sec > 0.0);
        assert!(snapshot.avg_latency_ms > 0.0);
    }

    #[test]
    fn test_percentile_calculation() {
        let mut collector = QueueMetricsCollector::new();

        // Record wait times: 10, 20, 30, ..., 100
        for i in 1..=10 {
            collector.record_queued(2); // Normal priority
            collector.record_processed(2, i as u64 * 10);
        }

        let snapshot = collector.snapshot(0);
        let metrics = snapshot.per_priority.get(&2).unwrap();

        assert!(metrics.avg_wait_time_ms > 0.0);
        assert!(metrics.p50_wait_time_ms > 0.0);
        assert!(metrics.p95_wait_time_ms >= metrics.p50_wait_time_ms);
        assert!(metrics.p99_wait_time_ms >= metrics.p95_wait_time_ms);
    }

    #[test]
    fn test_max_queue_depth_tracking() {
        let mut collector = QueueMetricsCollector::new();

        collector.record_queue_depth(5);
        collector.record_queue_depth(10);
        collector.record_queue_depth(3);
        collector.record_queue_depth(8);

        let snapshot = collector.snapshot(8);
        assert_eq!(snapshot.max_queue_depth, 10);
    }

    #[test]
    fn test_reset() {
        let mut collector = QueueMetricsCollector::new();

        collector.record_queued(2);
        collector.record_processed(2, 10);

        assert_eq!(collector.total_queued, 1);
        assert_eq!(collector.total_processed, 1);

        collector.reset();

        assert_eq!(collector.total_queued, 0);
        assert_eq!(collector.total_processed, 0);
    }
}
