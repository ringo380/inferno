//! # Unified Caching System
//!
//! This module consolidates all caching functionality:
//! - Model caching (loading and warm-up)
//! - Response caching (deduplication)
//! - Advanced caching (hierarchy, compression, persistence)
//!
//! Previously split across: cache.rs, response_cache.rs, advanced_cache.rs

// Re-export existing cache modules during transition
pub use crate::cache::*;
pub use crate::response_cache;
pub use crate::advanced_cache;

// Future: Will consolidate into unified API
// pub mod model_cache;
// pub mod response_cache;
// pub mod advanced;