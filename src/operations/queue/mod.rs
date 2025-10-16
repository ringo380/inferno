//! Request Queueing & Scheduling Module
//!
//! This module implements advanced request management with:
//! - Priority-based queuing
//! - Fair scheduling with starvation prevention
//! - Dynamic worker pool allocation
//! - Intelligent load balancing

pub mod priority_queue;
pub mod fair_scheduler;
pub mod metrics;
pub mod worker_pool;

pub use priority_queue::{Priority, PriorityQueue, RequestMetadata, QueueStats};
pub use fair_scheduler::{FairScheduler, FairnessMetrics, FairnessStats};
pub use metrics::{QueueMetricsCollector, QueueMetricsSnapshot, PriorityMetrics, RequestMetrics};
pub use worker_pool::{
    WorkerPool, WorkerPoolConfig, WorkerPoolRegistry, WorkerPoolStats, WorkerMetrics, WorkerState,
};
