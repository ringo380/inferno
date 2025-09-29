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

// Re-export from existing locations for now
pub use crate::batch;
pub use crate::deployment;
pub use crate::backup_recovery;
pub use crate::upgrade;
pub use crate::resilience;

// Versioning consolidation
pub mod versioning;