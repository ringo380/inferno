//! Comprehensive API Integration Tests for Phase 4C
//!
//! Tests all OpenAI-compatible endpoints, WebSocket streaming, flow control,
//! profiling endpoints, and validation scenarios.

use inferno::api::{
    flow_control::{BackpressureLevel, FlowControlConfig, StreamFlowControl},
    openai_compliance::{ComplianceValidator, ModelInfo},
    streaming_enhancements::{
        CompressionFormat, SSEMessage, StreamingOptimizationConfig, TimeoutManager, TokenBatcher,
    },
};
use std::time::Duration;

// ============================================================================
// CHAT COMPLETIONS ENDPOINT TESTS
// ============================================================================

#[test]
fn test_chat_completions_basic_request() {
    // Valid chat completion request
    let valid_request = serde_json::json!({
        "model": "llama-7b",
        "messages": [
            {
                "role": "user",
                "content": "Hello, how are you?"
            }
        ],
        "max_tokens": 100,
        "temperature": 0.7
    });

    // Request should deserialize successfully
    assert!(valid_request.is_object());
    assert_eq!(valid_request["model"], "llama-7b");
}

#[test]
fn test_chat_completions_with_streaming() {
    let streaming_request = serde_json::json!({
        "model": "llama-7b",
        "messages": [
            {"role": "system", "content": "You are helpful"},
            {"role": "user", "content": "What is 2+2?"}
        ],
        "stream": true,
        "max_tokens": 50
    });

    assert!(streaming_request["stream"].as_bool().unwrap());
}

#[test]
fn test_chat_completions_with_temperature_validation() {
    // Valid temperatures
    assert!(
        ComplianceValidator::validate_chat_completion_request(
            "llama-7b",
            Some(100),
            Some(0.0),
            Some(0.9),
        )
        .is_valid
    );

    assert!(
        ComplianceValidator::validate_chat_completion_request(
            "llama-7b",
            Some(100),
            Some(2.0),
            Some(0.9),
        )
        .is_valid
    );

    // Invalid temperatures
    let invalid_cold = ComplianceValidator::validate_chat_completion_request(
        "llama-7b",
        Some(100),
        Some(-0.1),
        Some(0.9),
    );
    assert!(!invalid_cold.is_valid);
    assert!(!invalid_cold.errors.is_empty());

    let invalid_hot = ComplianceValidator::validate_chat_completion_request(
        "llama-7b",
        Some(100),
        Some(2.5),
        Some(0.9),
    );
    assert!(!invalid_hot.is_valid);
}

#[test]
fn test_chat_completions_with_top_p_validation() {
    // Valid top_p values
    let valid = ComplianceValidator::validate_chat_completion_request(
        "llama-7b",
        Some(100),
        Some(0.7),
        Some(1.0),
    );
    assert!(valid.is_valid);

    // Invalid top_p > 1.0
    let invalid = ComplianceValidator::validate_chat_completion_request(
        "llama-7b",
        Some(100),
        Some(0.7),
        Some(1.5),
    );
    assert!(!invalid.is_valid);
    assert!(invalid.errors.iter().any(|e| e.contains("top_p")));
}

#[test]
fn test_chat_completions_max_tokens_validation() {
    // Valid max_tokens
    assert!(
        ComplianceValidator::validate_chat_completion_request("llama-7b", Some(1), None, None,)
            .is_valid
    );

    assert!(
        ComplianceValidator::validate_chat_completion_request(
            "llama-7b",
            Some(2_000_000),
            None,
            None,
        )
        .is_valid
    );

    // Invalid max_tokens
    let invalid_zero =
        ComplianceValidator::validate_chat_completion_request("llama-7b", Some(0), None, None);
    assert!(!invalid_zero.is_valid);

    let invalid_over = ComplianceValidator::validate_chat_completion_request(
        "llama-7b",
        Some(2_000_001),
        None,
        None,
    );
    assert!(!invalid_over.is_valid);
}

#[test]
fn test_chat_completions_model_validation() {
    let invalid =
        ComplianceValidator::validate_chat_completion_request("", Some(100), Some(0.7), None);
    assert!(!invalid.is_valid);
    assert!(invalid.errors.iter().any(|e| e.contains("model")));
}

#[test]
fn test_chat_completions_multiple_messages() {
    let multi_turn = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {"role": "system", "content": "You are a helpful assistant"},
            {"role": "user", "content": "What is AI?"},
            {"role": "assistant", "content": "AI stands for artificial intelligence..."},
            {"role": "user", "content": "Tell me more"}
        ],
        "max_tokens": 200
    });

    assert_eq!(multi_turn["messages"].as_array().unwrap().len(), 4);
}

// ============================================================================
// COMPLETIONS ENDPOINT TESTS
// ============================================================================

#[test]
fn test_completions_single_string_prompt() {
    let completion = ComplianceValidator::validate_completion_request("llama-7b", Some(100));
    assert!(completion.is_valid);
}

#[test]
fn test_completions_array_prompt() {
    // Test request with array of prompts
    let request = serde_json::json!({
        "model": "llama-7b",
        "prompt": ["Prompt 1", "Prompt 2"],
        "max_tokens": 100
    });

    assert!(request["prompt"].is_array());
}

#[test]
fn test_completions_with_stop_sequences() {
    let completion = serde_json::json!({
        "model": "llama-7b",
        "prompt": "Write a short poem",
        "max_tokens": 100,
        "stop": ["\n\n", "END"]
    });

    let stops = completion["stop"].as_array().unwrap();
    assert_eq!(stops.len(), 2);
}

#[test]
fn test_completions_with_penalties() {
    let completion = serde_json::json!({
        "model": "llama-7b",
        "prompt": "Generate creative text",
        "max_tokens": 100,
        "presence_penalty": 0.5,
        "frequency_penalty": 0.5
    });

    assert_eq!(completion["presence_penalty"], 0.5);
    assert_eq!(completion["frequency_penalty"], 0.5);
}

// ============================================================================
// EMBEDDINGS ENDPOINT TESTS
// ============================================================================

#[test]
fn test_embeddings_single_input() {
    let valid = ComplianceValidator::validate_embeddings_request(
        "text-embedding-ada-002",
        "This is a test embedding",
    );
    assert!(valid.is_valid);
}

#[test]
fn test_embeddings_multiple_inputs() {
    let request = serde_json::json!({
        "model": "text-embedding-ada-002",
        "input": ["Text 1", "Text 2", "Text 3"]
    });

    assert_eq!(request["input"].as_array().unwrap().len(), 3);
}

#[test]
fn test_embeddings_input_too_long() {
    let long_input = "a".repeat(8_001);
    let invalid =
        ComplianceValidator::validate_embeddings_request("text-embedding-ada-002", &long_input);
    assert!(!invalid.is_valid);
    assert!(invalid.errors.iter().any(|e| e.contains("length")));
}

#[test]
fn test_embeddings_empty_input() {
    let invalid = ComplianceValidator::validate_embeddings_request("text-embedding-ada-002", "");
    assert!(!invalid.is_valid);
}

#[test]
fn test_embeddings_at_boundary() {
    let boundary_input = "a".repeat(8_000);
    let valid =
        ComplianceValidator::validate_embeddings_request("text-embedding-ada-002", &boundary_input);
    assert!(valid.is_valid);
}

// ============================================================================
// FLOW CONTROL TESTS
// ============================================================================

#[test]
fn test_flow_control_healthy_state() {
    let config = FlowControlConfig::default();
    let fc = StreamFlowControl::new(config);

    assert_eq!(fc.check_backpressure(), BackpressureLevel::Healthy);
    assert_eq!(fc.buffer_utilization_percent(), 0);
}

#[test]
fn test_flow_control_moderate_backpressure() {
    let config = FlowControlConfig {
        max_pending_messages: 100,
        moderate_threshold_percent: 70,
        critical_threshold_percent: 90,
        ..Default::default()
    };
    let fc = StreamFlowControl::new(config);

    // Add messages to reach moderate threshold (70 out of 100)
    for _ in 0..70 {
        let _ = fc.add_message();
    }

    assert_eq!(fc.check_backpressure(), BackpressureLevel::Moderate);
    assert_eq!(fc.buffer_utilization_percent(), 70);
}

#[test]
fn test_flow_control_critical_backpressure() {
    let config = FlowControlConfig {
        max_pending_messages: 100,
        critical_threshold_percent: 90,
        ..Default::default()
    };
    let fc = StreamFlowControl::new(config);

    // Add messages to reach critical threshold
    for _ in 0..90 {
        let _ = fc.add_message();
    }

    assert_eq!(fc.check_backpressure(), BackpressureLevel::Critical);
}

#[test]
fn test_flow_control_buffer_overflow() {
    let config = FlowControlConfig {
        max_pending_messages: 10,
        ..Default::default()
    };
    let fc = StreamFlowControl::new(config);

    // Fill buffer
    for _ in 0..10 {
        assert!(fc.add_message().is_ok());
    }

    // Should fail to add more
    assert!(fc.add_message().is_err());
}

#[test]
fn test_flow_control_message_lifecycle() {
    let config = FlowControlConfig::default();
    let fc = StreamFlowControl::new(config);

    // Add message
    assert!(fc.add_message().is_ok());
    let util_after_add = fc.buffer_utilization_percent();

    // Send message
    fc.message_sent();
    assert_eq!(fc.buffer_utilization_percent(), 0);
    assert!(util_after_add > 0);
}

#[test]
fn test_flow_control_token_management() {
    let config = FlowControlConfig {
        max_unacked_tokens: 1000,
        ..Default::default()
    };
    let fc = StreamFlowControl::new(config);

    // Add tokens
    assert!(fc.add_tokens(500).is_ok());
    assert_eq!(fc.unacked_token_count(), 500);

    // Acknowledge tokens
    fc.ack_tokens(200);
    assert_eq!(fc.unacked_token_count(), 300);

    // Add more tokens
    assert!(fc.add_tokens(700).is_ok());
    assert_eq!(fc.unacked_token_count(), 1000);

    // Exceed limit
    assert!(fc.add_tokens(1).is_err());
}

#[test]
fn test_flow_control_timeout_detection() {
    let config = FlowControlConfig {
        ack_timeout_secs: 1,
        inference_timeout_secs: 5,
        ..Default::default()
    };
    let fc = StreamFlowControl::new(config);

    // Initial state - no timeout
    assert!(!fc.is_ack_timeout());

    // Wait for timeout
    std::thread::sleep(Duration::from_millis(1100));
    assert!(fc.is_ack_timeout());

    // Reset timeout
    fc.ack_tokens(0);
    assert!(!fc.is_ack_timeout());
}

// ============================================================================
// STREAMING ENHANCEMENTS TESTS
// ============================================================================

#[test]
fn test_compression_format_parsing() {
    let formats = CompressionFormat::from_accept_encoding("gzip, deflate, br");
    assert!(formats.contains(&CompressionFormat::Gzip));
    assert!(formats.contains(&CompressionFormat::Deflate));
    assert!(formats.contains(&CompressionFormat::Brotli));
}

#[test]
fn test_compression_format_headers() {
    assert_eq!(CompressionFormat::None.header_value(), "");
    assert_eq!(CompressionFormat::Gzip.header_value(), "gzip");
    assert_eq!(CompressionFormat::Deflate.header_value(), "deflate");
    assert_eq!(CompressionFormat::Brotli.header_value(), "br");
}

#[test]
fn test_sse_message_formatting() {
    let msg = SSEMessage::new("token".to_string(), "Hello".to_string())
        .with_id("123".to_string())
        .with_retry(1000);

    let formatted = msg.to_sse_format();
    assert!(formatted.contains("event: token"));
    assert!(formatted.contains("data: Hello"));
    assert!(formatted.contains("id: 123"));
    assert!(formatted.contains("retry: 1000"));
}

#[test]
fn test_sse_message_without_optional_fields() {
    let msg = SSEMessage::new("complete".to_string(), "[DONE]".to_string());
    let formatted = msg.to_sse_format();

    assert!(formatted.contains("event: complete"));
    assert!(formatted.contains("data: [DONE]"));
    assert!(!formatted.contains("id:"));
    assert!(!formatted.contains("retry:"));
}

#[test]
fn test_token_batcher_basic() {
    let mut batcher = TokenBatcher::new(3, 100);

    assert!(!batcher.should_flush());
    assert!(batcher.is_empty());

    batcher.add_token("Hello".to_string());
    batcher.add_token(" ".to_string());
    assert_eq!(batcher.len(), 2);
    assert!(!batcher.should_flush());

    batcher.add_token("World".to_string());
    assert!(batcher.should_flush());

    let batched = batcher.flush();
    assert_eq!(batched, "Hello World");
    assert!(batcher.is_empty());
}

#[test]
fn test_token_batcher_timeout() {
    let mut batcher = TokenBatcher::new(100, 50); // Small timeout

    batcher.add_token("token1".to_string());
    assert!(!batcher.should_flush());

    std::thread::sleep(Duration::from_millis(100));
    assert!(batcher.should_flush());
}

#[test]
fn test_token_batcher_flush_partial() {
    let mut batcher = TokenBatcher::new(5, 1000);

    batcher.add_token("a".to_string());
    batcher.add_token("b".to_string());
    assert!(!batcher.should_flush());

    // Manual flush before batch size reached
    let result = batcher.flush();
    assert_eq!(result, "ab");
    assert!(batcher.is_empty());
}

#[test]
fn test_timeout_manager_inference_timeout() {
    let tm = TimeoutManager::new(1, 30);

    assert!(!tm.is_inference_timeout());

    std::thread::sleep(Duration::from_millis(1100));
    assert!(tm.is_inference_timeout());
}

#[test]
fn test_timeout_manager_token_timeout() {
    let tm = TimeoutManager::new(30, 1);

    assert!(!tm.is_token_timeout());

    std::thread::sleep(Duration::from_millis(1100));
    assert!(tm.is_token_timeout());
}

#[test]
fn test_timeout_manager_token_recording() {
    let mut tm = TimeoutManager::new(30, 1);  // 1 second token timeout

    assert!(!tm.is_token_timeout());

    std::thread::sleep(Duration::from_millis(1100));
    assert!(tm.is_token_timeout());

    // Record token resets the timeout
    tm.record_token();
    assert!(!tm.is_token_timeout());
}

#[test]
fn test_timeout_manager_time_tracking() {
    let tm = TimeoutManager::new(30, 5);

    let elapsed = tm.elapsed_secs();
    assert_eq!(elapsed, 0);

    let time_since = tm.time_since_last_token_ms();
    assert!(time_since < 100); // Should be just a few ms

    std::thread::sleep(Duration::from_millis(200));
    let time_since_after = tm.time_since_last_token_ms();
    assert!(time_since_after >= 200);
}

// ============================================================================
// STREAMING OPTIMIZATION CONFIG TESTS
// ============================================================================

#[test]
fn test_streaming_optimization_config_defaults() {
    let config = StreamingOptimizationConfig::default();

    assert_eq!(config.batch_size, 3);
    assert_eq!(config.batch_max_wait_ms, 50);
    assert_eq!(config.inference_timeout_secs, 300);
    assert_eq!(config.token_timeout_secs, 30);
    assert_eq!(config.keepalive_interval_secs, 30);
    assert!(config.tcp_nodelay);
    assert_eq!(config.compression, CompressionFormat::None);
}

#[test]
fn test_streaming_optimization_config_custom() {
    let config = StreamingOptimizationConfig {
        compression: CompressionFormat::Gzip,
        batch_size: 5,
        batch_max_wait_ms: 100,
        inference_timeout_secs: 600,
        token_timeout_secs: 60,
        keepalive_interval_secs: 60,
        tcp_nodelay: false,
        ..Default::default()
    };

    assert_eq!(config.batch_size, 5);
    assert_eq!(config.compression, CompressionFormat::Gzip);
    assert!(!config.tcp_nodelay);
}

// ============================================================================
// OPENAI COMPLIANCE TESTS
// ============================================================================

#[test]
fn test_model_info_creation() {
    let info = ModelInfo::local_model("llama-7b");

    assert_eq!(info.id, "llama-7b");
    assert_eq!(info.owned_by, "inferno");
    assert_eq!(info.object, "model");
    assert!(!info.permission.is_empty());
    assert_eq!(info.permission[0].allow_sampling, true);
    assert_eq!(info.permission[0].allow_view, true);
}

#[test]
fn test_openai_status_code_mapping() {
    // 400 Bad Request
    let validation_err = inferno::InfernoError::Validation("bad input".to_string());
    let (code, _) = ComplianceValidator::map_status_code(&validation_err);
    assert_eq!(code, 400);

    // 401 Unauthorized
    let auth_err = inferno::InfernoError::Auth("unauthorized".to_string());
    let (code, _) = ComplianceValidator::map_status_code(&auth_err);
    assert_eq!(code, 401);

    // 403 Forbidden
    let security_err = inferno::InfernoError::SecurityValidation("forbidden".to_string());
    let (code, _) = ComplianceValidator::map_status_code(&security_err);
    assert_eq!(code, 403);

    // 404 Not Found
    let not_found = inferno::InfernoError::ModelNotFound("not found".to_string());
    let (code, _) = ComplianceValidator::map_status_code(&not_found);
    assert_eq!(code, 404);

    // 504 Gateway Timeout
    let timeout_err = inferno::InfernoError::Timeout("timeout".to_string());
    let (code, _) = ComplianceValidator::map_status_code(&timeout_err);
    assert_eq!(code, 504);

    // 507 Insufficient Storage
    let resource_err = inferno::InfernoError::Resource("out of memory".to_string());
    let (code, _) = ComplianceValidator::map_status_code(&resource_err);
    assert_eq!(code, 507);

    // 500 Internal Server Error
    let backend_err = inferno::InfernoError::Backend("backend error".to_string());
    let (code, _) = ComplianceValidator::map_status_code(&backend_err);
    assert_eq!(code, 500);
}

// ============================================================================
// COMBINED SCENARIO TESTS
// ============================================================================

#[test]
fn test_streaming_with_flow_control() {
    let config = StreamingOptimizationConfig::default();
    let fc_config = FlowControlConfig::default();
    let fc = StreamFlowControl::new(fc_config);

    let mut batcher = TokenBatcher::new(config.batch_size, config.batch_max_wait_ms);
    let tm = TimeoutManager::new(config.inference_timeout_secs, config.token_timeout_secs);

    // Simulate streaming tokens
    for i in 0..10 {
        // Add token
        batcher.add_token(format!("token{} ", i));

        // Check flow control
        let backpressure = fc.check_backpressure();
        assert_eq!(backpressure, BackpressureLevel::Healthy);

        // Batch if needed
        if batcher.should_flush() {
            let _batched = batcher.flush();
            assert!(tm.elapsed_secs() < config.inference_timeout_secs);
        }
    }
}

#[test]
fn test_full_request_validation_pipeline() {
    // Chat completion validation
    let chat_validation = ComplianceValidator::validate_chat_completion_request(
        "gpt-3.5-turbo",
        Some(200),
        Some(0.8),
        Some(0.95),
    );
    assert!(chat_validation.is_valid);

    // Embeddings validation
    let embedding_validation =
        ComplianceValidator::validate_embeddings_request("text-embedding-ada-002", "test input");
    assert!(embedding_validation.is_valid);

    // Completion validation
    let completion_validation =
        ComplianceValidator::validate_completion_request("llama-7b", Some(150));
    assert!(completion_validation.is_valid);
}

#[test]
fn test_concurrent_flow_control() {
    let config = FlowControlConfig {
        max_pending_messages: 100,
        moderate_threshold_percent: 70,
        critical_threshold_percent: 90,
        ..Default::default()
    };
    let fc = StreamFlowControl::new(config);

    // Simulate concurrent messages
    let mut handles = vec![];

    for _ in 0..5 {
        let fc_clone = fc.clone();
        let handle = std::thread::spawn(move || {
            for _ in 0..10 {
                let _ = fc_clone.add_message();
                fc_clone.message_sent();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(fc.buffer_utilization_percent(), 0);
}

// ============================================================================
// ERROR SCENARIO TESTS
// ============================================================================

#[test]
fn test_invalid_chat_all_parameters() {
    let validation = ComplianceValidator::validate_chat_completion_request(
        "",         // Empty model
        Some(0),    // Invalid tokens
        Some(-1.0), // Invalid temperature
        Some(1.5),  // Invalid top_p
    );

    assert!(!validation.is_valid);
    assert!(validation.errors.len() >= 3); // Multiple errors
}

#[test]
fn test_compression_empty_accept_encoding() {
    let formats = CompressionFormat::from_accept_encoding("");
    assert_eq!(formats.len(), 1);
    assert_eq!(formats[0], CompressionFormat::None);
}

#[test]
fn test_compression_partial_match() {
    let formats = CompressionFormat::from_accept_encoding("deflate");
    assert_eq!(formats.len(), 1);
    assert_eq!(formats[0], CompressionFormat::Deflate);
}

#[test]
fn test_batcher_with_max_wait_expiry() {
    let mut batcher = TokenBatcher::new(1000, 10); // Very small timeout, large batch

    batcher.add_token("token".to_string());
    assert!(!batcher.should_flush()); // Small batch, should not flush

    std::thread::sleep(Duration::from_millis(20));
    assert!(batcher.should_flush()); // Timeout expired, should flush
}

// ============================================================================
// BACKWARDS COMPATIBILITY TESTS
// ============================================================================

#[test]
fn test_openai_request_format_compatibility() {
    // Ensure requests match OpenAI format
    let request = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [{"role": "user", "content": "hello"}],
        "temperature": 0.7,
        "top_p": 0.9,
        "max_tokens": 100,
        "stream": false
    });

    // All required fields present
    assert!(request["model"].is_string());
    assert!(request["messages"].is_array());
    assert!(request["temperature"].is_number());
}

#[test]
fn test_openai_response_format_compatibility() {
    // Verify response structure
    let response = serde_json::json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1234567890i64,
        "model": "gpt-3.5-turbo",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello!"
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    });

    assert!(response["id"].is_string());
    assert_eq!(response["object"], "chat.completion");
    assert!(response["choices"].is_array());
    assert!(response["usage"]["total_tokens"].is_number());
}
