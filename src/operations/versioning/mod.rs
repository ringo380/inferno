//! # Unified Versioning System
//!
//! This module consolidates all versioning functionality:
//! - Application versioning
//! - Model versioning
//! - Version compatibility checks
//!
//! Previously split across: versioning.rs, model_versioning.rs

// Re-export existing versioning modules during transition
pub use crate::model_versioning;
pub use crate::versioning::*;

// Future: Will consolidate into unified API
// pub mod app_version;
// pub mod model_version;
