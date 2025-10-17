//! # Infrastructure & Observability
//!
//! This module contains infrastructure-level features:
//! - Caching (model cache, response cache, advanced caching)
//! - Monitoring and alerting
//! - Observability (tracing, telemetry)
//! - Metrics collection and export
//! - Audit logging and compliance
//!
//! Infrastructure modules provide cross-cutting concerns for the platform.

// Re-export from existing locations for now
pub use crate::observability;

// Submodules for consolidated features
pub mod audit;
pub mod cache;
pub mod monitoring;
pub mod profiling;

// System monitoring and performance profiling
pub mod sys_monitor;

// Keep direct access to metrics for now
pub use crate::metrics;
