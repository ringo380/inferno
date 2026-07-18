//! # Unified Monitoring System
//!
//! This module consolidates all monitoring functionality:
//! - Basic monitoring and metrics collection
//! - Alerting and notifications
//!
//! Previously split across: monitoring.rs

// Re-export existing monitoring modules during transition
pub use crate::monitoring::*;

// Future: Will consolidate into unified API
// pub mod metrics;
// pub mod alerts;
// pub mod advanced;
