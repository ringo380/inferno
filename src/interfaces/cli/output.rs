//! Command output types and formatting
//!
//! Provides structured output for CLI commands with support for
//! JSON serialization, exit codes, and human-readable messages.

use serde::{Deserialize, Serialize};

/// Output from command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    /// Whether the command succeeded
    pub success: bool,

    /// Optional structured data (for JSON output, piping, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Optional human-readable message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Exit code (0 = success, non-zero = error)
    pub exit_code: i32,

    /// Output level (info, warning, error)
    #[serde(default)]
    pub level: OutputLevel,
}

/// Output severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputLevel {
    /// Informational output
    Info,

    /// Warning (non-fatal)
    Warning,

    /// Error (fatal)
    Error,
}

impl Default for OutputLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl CommandOutput {
    /// Create a successful output with just a message
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: None,
            message: Some(message.into()),
            exit_code: 0,
            level: OutputLevel::Info,
        }
    }

    /// Create a successful output with data and message
    pub fn success_with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message.into()),
            exit_code: 0,
            level: OutputLevel::Info,
        }
    }

    /// Create a successful output with only data (no message)
    pub fn data(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            exit_code: 0,
            level: OutputLevel::Info,
        }
    }

    /// Create a warning output (success but with concerns)
    pub fn warning(message: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self {
            success: true,
            data,
            message: Some(message.into()),
            exit_code: 0,
            level: OutputLevel::Warning,
        }
    }

    /// Create an error output
    pub fn error(message: impl Into<String>, exit_code: i32) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message.into()),
            exit_code,
            level: OutputLevel::Error,
        }
    }

    /// Create an error output with data
    pub fn error_with_data(
        message: impl Into<String>,
        data: serde_json::Value,
        exit_code: i32,
    ) -> Self {
        Self {
            success: false,
            data: Some(data),
            message: Some(message.into()),
            exit_code,
            level: OutputLevel::Error,
        }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Convert to compact JSON string
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Format for human display
    pub fn to_display(&self) -> String {
        match (&self.message, &self.data) {
            (Some(msg), Some(data)) => {
                format!(
                    "{}\n{}",
                    msg,
                    serde_json::to_string_pretty(data).unwrap_or_default()
                )
            }
            (Some(msg), None) => msg.clone(),
            (None, Some(data)) => serde_json::to_string_pretty(data).unwrap_or_default(),
            (None, None) => String::from("Command completed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_success_output() {
        let output = CommandOutput::success("Operation completed");
        assert!(output.success);
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.message, Some("Operation completed".to_string()));
    }

    #[test]
    fn test_success_with_data() {
        let data = json!({"count": 5, "items": ["a", "b", "c"]});
        let output = CommandOutput::success_with_data("Found items", data.clone());

        assert!(output.success);
        assert_eq!(output.data, Some(data));
    }

    #[test]
    fn test_warning_output() {
        let output = CommandOutput::warning("No models found", None);
        assert!(output.success); // Warnings are still successful
        assert_eq!(output.level, OutputLevel::Warning);
    }

    #[test]
    fn test_error_output() {
        let output = CommandOutput::error("File not found", 1);
        assert!(!output.success);
        assert_eq!(output.exit_code, 1);
        assert_eq!(output.level, OutputLevel::Error);
    }

    #[test]
    fn test_json_serialization() {
        let output = CommandOutput::success_with_data("Test", json!({"key": "value"}));

        let json = output.to_json().unwrap();
        assert!(json.contains("\"success\": true"));
        assert!(json.contains("\"key\": \"value\""));
    }
}
