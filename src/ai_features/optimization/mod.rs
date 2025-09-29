//! # Unified Optimization System
//!
//! This module consolidates all optimization functionality:
//! - General performance optimization
//! - Model-specific optimizations
//! - Performance baseline measurement
//! - Profiling and benchmarking
//!
//! Previously split across: optimization.rs, performance_optimization.rs, performance_baseline.rs

// Re-export existing optimization modules during transition
pub use crate::optimization::*;
pub use crate::performance_optimization;
pub use crate::performance_baseline;

// Future: Will consolidate into unified API
// pub mod performance;
// pub mod baseline;
// pub mod profiling;