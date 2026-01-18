//! OpenAI API Compliance Validation
//!
//! Ensures all API responses conform to OpenAI API specifications for drop-in replacement compatibility

use crate::InfernoError;
use serde::{Deserialize, Serialize};

/// Error response matching OpenAI format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIError {
    pub message: String,
    pub r#type: String,
    pub param: Option<String>,
    pub code: Option<String>,
}

/// Error wrapper matching OpenAI response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: OpenAIError,
}

impl ErrorResponse {
    /// Create from Inferno error
    pub fn from_inferno_error(error: &InfernoError) -> Self {
        let (message, code, r#type) = match error {
            InfernoError::ModelNotFound(msg) => (
                msg.clone(),
                Some("model_not_found"),
                "invalid_request_error",
            ),
            InfernoError::Config(_) => (
                "Invalid configuration".to_string(),
                Some("invalid_config"),
                "invalid_request_error",
            ),
            InfernoError::Backend(msg) => (msg.clone(), Some("backend_error"), "server_error"),
            InfernoError::Timeout(_) => (
                "Request timeout".to_string(),
                Some("timeout"),
                "server_error",
            ),
            InfernoError::Validation(msg) => {
                (msg.clone(), Some("invalid_value"), "invalid_request_error")
            }
            _ => (error.to_string(), None, "server_error"),
        };

        Self {
            error: OpenAIError {
                message,
                r#type: r#type.to_string(),
                param: None,
                code: code.map(|s| s.to_string()),
            },
        }
    }
}

/// Request validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    /// Create valid result
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    /// Create invalid result with errors
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }

    /// Add error message
    pub fn with_error(mut self, error: String) -> Self {
        self.errors.push(error);
        self.is_valid = false;
        self
    }
}

/// OpenAI request validators
pub struct ComplianceValidator;

impl ComplianceValidator {
    /// Validate chat completion request
    pub fn validate_chat_completion_request(
        model: &str,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
        top_p: Option<f32>,
    ) -> ValidationResult {
        let mut result = ValidationResult::valid();

        // Validate model
        if model.is_empty() {
            result = result.with_error("model is required".to_string());
        }

        // Validate temperature (0-2)
        if let Some(temp) = temperature {
            if temp < 0.0 || temp > 2.0 {
                result = result.with_error("temperature must be between 0 and 2".to_string());
            }
        }

        // Validate top_p (0-1)
        if let Some(p) = top_p {
            if p < 0.0 || p > 1.0 {
                result = result.with_error("top_p must be between 0 and 1".to_string());
            }
        }

        // Validate max_tokens
        if let Some(tokens) = max_tokens {
            if tokens <= 0 || tokens > 2_000_000 {
                result = result.with_error("max_tokens must be between 1 and 2000000".to_string());
            }
        }

        result
    }

    /// Validate embeddings request
    pub fn validate_embeddings_request(model: &str, input: &str) -> ValidationResult {
        let mut result = ValidationResult::valid();

        if model.is_empty() {
            result = result.with_error("model is required".to_string());
        }

        if input.is_empty() {
            result = result.with_error("input is required".to_string());
        }

        if input.len() > 8_000 {
            result = result.with_error("input length must not exceed 8000 characters".to_string());
        }

        result
    }

    /// Validate completion request
    pub fn validate_completion_request(model: &str, max_tokens: Option<i32>) -> ValidationResult {
        let mut result = ValidationResult::valid();

        if model.is_empty() {
            result = result.with_error("model is required".to_string());
        }

        if let Some(tokens) = max_tokens {
            if tokens <= 0 || tokens > 2_000_000 {
                result = result.with_error("max_tokens must be between 1 and 2000000".to_string());
            }
        }

        result
    }

    /// Map Inferno HTTP status code to OpenAI status code
    pub fn map_status_code(inferno_error: &InfernoError) -> (u16, &'static str) {
        match inferno_error {
            InfernoError::Validation(_) => (400, "Bad Request"),
            InfernoError::Auth(_) => (401, "Unauthorized"),
            InfernoError::SecurityValidation(_) => (403, "Forbidden"),
            InfernoError::ModelNotFound(_) => (404, "Not Found"),
            InfernoError::Timeout(_) => (504, "Gateway Timeout"),
            InfernoError::Resource(_) => (507, "Insufficient Storage"),
            _ => (500, "Internal Server Error"),
        }
    }
}

/// OpenAI-compatible model info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
    pub permission: Vec<ModelPermission>,
    pub root: String,
    pub parent: Option<String>,
}

/// Model permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPermission {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub allow_create_engine: bool,
    pub allow_sampling: bool,
    pub allow_logprobs: bool,
    pub allow_search_indices: bool,
    pub allow_view: bool,
    pub allow_fine_tuning: bool,
    pub organization: String,
    pub group_id: Option<String>,
    pub is_blocking: bool,
}

impl ModelInfo {
    /// Create model info for a local model
    pub fn local_model(model_id: &str) -> Self {
        Self {
            id: model_id.to_string(),
            object: "model".to_string(),
            created: chrono::Utc::now().timestamp(),
            owned_by: "inferno".to_string(),
            permission: vec![ModelPermission {
                id: format!("modelperm-{}", uuid::Uuid::new_v4()),
                object: "model_permission".to_string(),
                created: chrono::Utc::now().timestamp(),
                allow_create_engine: false,
                allow_sampling: true,
                allow_logprobs: false,
                allow_search_indices: false,
                allow_view: true,
                allow_fine_tuning: false,
                organization: "*".to_string(),
                group_id: None,
                is_blocking: false,
            }],
            root: model_id.to_string(),
            parent: None,
        }
    }
}

/// OpenAI API version
pub const OPENAI_API_VERSION: &str = "2023-06-01";

/// Inferno version header for OpenAI compatibility
pub const INFERNO_VERSION_HEADER: &str = "Inferno/0.8.0 (OpenAI-compatible)";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_completion_validation() {
        let result = ComplianceValidator::validate_chat_completion_request(
            "gpt-3.5-turbo",
            Some(100),
            Some(0.7),
            Some(0.9),
        );

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_invalid_temperature() {
        let result = ComplianceValidator::validate_chat_completion_request(
            "gpt-3.5-turbo",
            Some(100),
            Some(2.5), // Invalid: > 2.0
            None,
        );

        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_invalid_top_p() {
        let result = ComplianceValidator::validate_chat_completion_request(
            "gpt-3.5-turbo",
            Some(100),
            None,
            Some(1.5), // Invalid: > 1.0
        );

        assert!(!result.is_valid);
    }

    #[test]
    fn test_embeddings_validation() {
        let result = ComplianceValidator::validate_embeddings_request(
            "text-embedding-ada-002",
            "This is a test embedding",
        );

        assert!(result.is_valid);
    }

    #[test]
    fn test_embeddings_too_long() {
        let long_input = "a".repeat(10_000);
        let result =
            ComplianceValidator::validate_embeddings_request("text-embedding-ada-002", &long_input);

        assert!(!result.is_valid);
    }

    #[test]
    fn test_model_info_creation() {
        let info = ModelInfo::local_model("llama-2-7b");
        assert_eq!(info.id, "llama-2-7b");
        assert_eq!(info.owned_by, "inferno");
        assert_eq!(info.object, "model");
        assert!(!info.permission.is_empty());
    }

    #[test]
    fn test_error_response_creation() {
        let inferno_err = InfernoError::ModelNotFound("model not found".to_string());
        let err_response = ErrorResponse::from_inferno_error(&inferno_err);

        assert_eq!(err_response.error.code, Some("model_not_found".to_string()));
        assert_eq!(err_response.error.r#type, "invalid_request_error");
    }

    #[test]
    fn test_status_code_mapping() {
        let validation_err = InfernoError::Validation("bad input".to_string());
        let (code, _) = ComplianceValidator::map_status_code(&validation_err);
        assert_eq!(code, 400);

        let auth_err = InfernoError::Auth("unauthorized".to_string());
        let (code, _) = ComplianceValidator::map_status_code(&auth_err);
        assert_eq!(code, 401);
    }
}
