//! # Unified Caching System
//!
//! This module consolidates all caching functionality:
//! - Model caching (loading and warm-up)
//! - Response caching (deduplication)
//!
//! Previously split across: cache.rs, response_cache.rs

// Re-export existing cache modules during transition
pub use crate::cache::*;
pub use crate::response_cache;

// Future: Will consolidate into unified API
// pub mod model_cache;
// pub mod response_cache;
// pub mod advanced;
