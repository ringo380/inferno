//! # Operations & DevOps
//!
//! This module contains operational features:
//! - Batch processing and job queuing
//! - Deployment automation
//! - Backup and recovery
//! - Auto-update system
//! - Resilience patterns
//! - Version management
//!
//! Operations modules handle production deployment and maintenance tasks.

// New queue module for Phase 4A
pub mod queue;

// Re-export from existing locations for now
pub use crate::backup_recovery;
pub use crate::batch;
pub use crate::deployment;
pub use crate::resilience;
pub use crate::upgrade;

// Versioning consolidation
pub mod versioning;
