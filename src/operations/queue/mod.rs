//! Request Queueing & Scheduling Module
//!
//! This module implements advanced request management with:
//! - Priority-based queuing
//! - Fair scheduling with starvation prevention
//! - Dynamic worker pool allocation
//! - Intelligent load balancing

pub mod assignment;
pub mod fair_scheduler;
pub mod metrics;
pub mod persistence;
pub mod priority_queue;
pub mod worker_pool;

pub use assignment::{
    AssignmentResult, AssignmentStrategy, BackpressureStatus, LoadBalancer, LoadStats, RequestGroup,
};
pub use fair_scheduler::{FairScheduler, FairnessMetrics, FairnessStats};
pub use metrics::{PriorityMetrics, QueueMetricsCollector, QueueMetricsSnapshot, RequestMetrics};
pub use persistence::{
    HealthStatus, PersistenceConfig, QueueHealthStatus, QueuePersistence, QueueStateSnapshot,
    ShutdownCoordinator,
};
pub use priority_queue::{Priority, PriorityQueue, QueueStats, RequestMetadata};
pub use worker_pool::{
    WorkerMetrics, WorkerPool, WorkerPoolConfig, WorkerPoolRegistry, WorkerPoolStats, WorkerState,
};
