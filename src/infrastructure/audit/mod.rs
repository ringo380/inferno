//! # Unified Audit System
//!
//! This module consolidates all audit functionality:
//! - Audit event logging
//! - Compliance tracking
//! - Retention policies
//! - Log encryption
//!
//! Previously split across: audit.rs, logging_audit.rs

// Re-export existing audit modules during transition
pub use crate::audit::*;
pub use crate::logging_audit;

// Future: Will consolidate into unified API
// pub mod logger;
// pub mod compliance;
// pub mod retention;
