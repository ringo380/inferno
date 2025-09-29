//! Middleware system for command execution
//!
//! Provides both the base middleware trait and built-in middleware implementations.

// Base middleware trait and stack
pub mod base;
// Built-in middleware implementations
pub mod logging;
pub mod metrics;

// Re-export base types
pub use base::{Middleware, MiddlewareStack};

// Re-export built-in middleware
pub use logging::LoggingMiddleware;
pub use metrics::MetricsMiddleware;