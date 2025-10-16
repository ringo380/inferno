//! # AI/ML Specialized Features
//!
//! This module contains AI/ML-specific features:
//! - Model format conversion
//! - Performance optimization
//! - Multimodal support (vision, audio)
//! - Real-time streaming
//! - GPU management
//!
//! AI features provide specialized capabilities for model execution and optimization.

// Re-export from existing locations for now
pub use crate::conversion;
pub use crate::gpu;
pub use crate::multimodal;
pub use crate::streaming;

// Optimization consolidation
pub mod optimization;

// Sampling strategies and configuration
pub mod sampling;
