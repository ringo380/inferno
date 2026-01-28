//! Command execution context
//!
//! Provides a shared context for command execution that includes
//! configuration, state, metrics, and other shared resources.

use crate::config::Config;
use crate::core::config::CoreConfig;
use crate::metrics::MetricsCollector;
use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// Context passed to all commands during execution
///
/// Contains configuration, shared state, metrics, and execution metadata.
/// Can be extended with command-specific state via the `state` HashMap.
pub struct CommandContext {
    /// Legacy configuration (for backward compatibility)
    pub config: Arc<Config>,

    /// New core configuration (builder-based)
    pub core_config: Option<Arc<CoreConfig>>,

    /// Command-specific arguments (flexible JSON values)
    pub args: HashMap<String, serde_json::Value>,

    /// Shared state across middleware and command execution
    /// Use TypeId as key via state.insert::<T>(value)
    state: HashMap<String, Box<dyn Any + Send + Sync>>,

    /// Unique execution ID for this command run
    pub execution_id: Uuid,

    /// When command execution started
    pub start_time: Instant,

    /// Metrics collector for recording command metrics
    pub metrics: Arc<MetricsCollector>,

    /// Whether to output in JSON format
    pub json_output: bool,

    /// Verbosity level (0 = normal, 1 = verbose, 2+ = debug)
    pub verbosity: u8,
}

impl CommandContext {
    /// Create new command context with legacy config
    pub fn new(config: Config) -> Self {
        let (metrics_collector, processor) = MetricsCollector::new();
        processor.start();

        Self {
            config: Arc::new(config),
            core_config: None,
            args: HashMap::new(),
            state: HashMap::new(),
            execution_id: Uuid::new_v4(),
            start_time: Instant::now(),
            metrics: Arc::new(metrics_collector),
            json_output: false,
            verbosity: 0,
        }
    }

    /// Create new command context with both configs
    pub fn with_configs(config: Config, core_config: CoreConfig) -> Self {
        let (metrics_collector, processor) = MetricsCollector::new();
        processor.start();

        Self {
            config: Arc::new(config),
            core_config: Some(Arc::new(core_config)),
            args: HashMap::new(),
            state: HashMap::new(),
            execution_id: Uuid::new_v4(),
            start_time: Instant::now(),
            metrics: Arc::new(metrics_collector),
            json_output: false,
            verbosity: 0,
        }
    }

    /// Set an argument value
    pub fn set_arg(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.args.insert(key.into(), value);
    }

    /// Get an argument value
    pub fn get_arg(&self, key: &str) -> Option<&serde_json::Value> {
        self.args.get(key)
    }

    /// Get an argument as a specific type
    pub fn get_arg_as<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T> {
        let value = self
            .args
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("Argument '{}' not found", key))?;

        serde_json::from_value(value.clone())
            .map_err(|e| anyhow::anyhow!("Failed to deserialize argument '{}': {}", key, e))
    }

    /// Store typed state value
    pub fn set_state<T: Any + Send + Sync>(&mut self, key: impl Into<String>, value: T) {
        self.state.insert(key.into(), Box::new(value));
    }

    /// Get typed state value
    pub fn get_state<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.state.get(key).and_then(|v| v.downcast_ref::<T>())
    }

    /// Get mutable typed state value
    pub fn get_state_mut<T: Any + Send + Sync>(&mut self, key: &str) -> Option<&mut T> {
        self.state.get_mut(key).and_then(|v| v.downcast_mut::<T>())
    }

    /// Get elapsed time since command started
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Enable JSON output mode
    pub fn set_json_output(&mut self, enabled: bool) {
        self.json_output = enabled;
    }

    /// Set verbosity level
    pub fn set_verbosity(&mut self, level: u8) {
        self.verbosity = level;
    }

    /// Check if verbose mode is enabled
    pub fn is_verbose(&self) -> bool {
        self.verbosity >= 1
    }

    /// Check if debug mode is enabled
    pub fn is_debug(&self) -> bool {
        self.verbosity >= 2
    }
}

#[cfg(test)]
impl CommandContext {
    /// Create a mock context for testing
    pub fn mock() -> Self {
        let (metrics_collector, processor) = MetricsCollector::new();
        processor.start();

        Self {
            config: Arc::new(Config::default()),
            core_config: None,
            args: HashMap::new(),
            state: HashMap::new(),
            execution_id: Uuid::new_v4(),
            start_time: Instant::now(),
            metrics: Arc::new(metrics_collector),
            json_output: false,
            verbosity: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_context_creation() {
        let ctx = CommandContext::mock();
        assert!(!ctx.json_output);
        assert_eq!(ctx.verbosity, 0);
    }

    #[tokio::test]
    async fn test_arg_storage() {
        let mut ctx = CommandContext::mock();
        ctx.set_arg("model", json!("llama-2-7b"));
        ctx.set_arg("count", json!(5));

        assert_eq!(ctx.get_arg("model"), Some(&json!("llama-2-7b")));
        assert_eq!(ctx.get_arg("count"), Some(&json!(5)));
        assert_eq!(ctx.get_arg("missing"), None);
    }

    #[tokio::test]
    async fn test_typed_arg_retrieval() {
        let mut ctx = CommandContext::mock();
        ctx.set_arg("count", json!(42));
        ctx.set_arg("name", json!("test"));

        let count: i32 = ctx.get_arg_as("count").unwrap();
        assert_eq!(count, 42);

        let name: String = ctx.get_arg_as("name").unwrap();
        assert_eq!(name, "test");
    }

    #[tokio::test]
    async fn test_state_storage() {
        let mut ctx = CommandContext::mock();

        ctx.set_state("counter", 42_i32);
        ctx.set_state("name", "test".to_string());

        assert_eq!(ctx.get_state::<i32>("counter"), Some(&42));
        assert_eq!(ctx.get_state::<String>("name"), Some(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_verbosity() {
        let mut ctx = CommandContext::mock();
        assert!(!ctx.is_verbose());
        assert!(!ctx.is_debug());

        ctx.set_verbosity(1);
        assert!(ctx.is_verbose());
        assert!(!ctx.is_debug());

        ctx.set_verbosity(2);
        assert!(ctx.is_verbose());
        assert!(ctx.is_debug());
    }

    #[tokio::test]
    async fn test_elapsed_time() {
        let ctx = CommandContext::mock();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let elapsed = ctx.elapsed();
        assert!(elapsed.as_millis() >= 10);
    }
}
