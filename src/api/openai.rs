use crate::{
    backends::{BackendHandle, InferenceParams, BackendType},
    cli::serve::ServerState,
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// OpenAI API compatible types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    #[serde(default)]
    pub n: Option<u32>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub stop: Option<Vec<String>>,
    #[serde(default)]
    pub presence_penalty: Option<f32>,
    #[serde(default)]
    pub frequency_penalty: Option<f32>,
    #[serde(default)]
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub prompt: StringOrArray,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    #[serde(default)]
    pub n: Option<u32>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub logprobs: Option<u32>,
    #[serde(default)]
    pub echo: bool,
    #[serde(default)]
    pub stop: Option<Vec<String>>,
    #[serde(default)]
    pub presence_penalty: Option<f32>,
    #[serde(default)]
    pub frequency_penalty: Option<f32>,
    #[serde(default)]
    pub best_of: Option<u32>,
    #[serde(default)]
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrArray {
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<CompletionChoice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoice {
    pub text: String,
    pub index: u32,
    pub logprobs: Option<serde_json::Value>,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: StringOrArray,
    #[serde(default)]
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: EmbeddingUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelObject {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
    pub permission: Vec<serde_json::Value>,
    pub root: String,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelListResponse {
    pub object: String,
    pub data: Vec<ModelObject>,
}

// Streaming response types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChunkChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunkChoice {
    pub index: u32,
    pub delta: ChatDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

// Default values

fn default_max_tokens() -> u32 {
    512
}

fn default_temperature() -> f32 {
    0.7
}

fn default_top_p() -> f32 {
    0.9
}

// API State

// Note: We use the ServerState from cli::serve module

// API Handlers

pub async fn chat_completions(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    // Convert chat messages to a single prompt
    let prompt = format_chat_messages(&request.messages);

    // Get or load the backend
    let backend = match get_or_load_backend(&state, &request.model).await {
        Ok(backend) => backend,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": {
                        "message": format!("Failed to load model: {}", e),
                        "type": "invalid_request_error",
                        "param": "model",
                        "code": null
                    }
                })),
            ).into_response();
        }
    };

    let inference_params = InferenceParams {
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        stream: request.stream,
    };

    if request.stream {
        // Handle streaming response
        handle_streaming_chat(&request, backend, prompt, inference_params).await.into_response()
    } else {
        // Handle non-streaming response
        handle_non_streaming_chat(&request, backend, prompt, inference_params).await.into_response()
    }
}

pub async fn completions(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<CompletionRequest>,
) -> impl IntoResponse {
    // Extract prompt
    let prompt = match &request.prompt {
        StringOrArray::String(s) => s.clone(),
        StringOrArray::Array(arr) => arr.join("\n"),
    };

    // Get or load the backend
    let backend = match get_or_load_backend(&state, &request.model).await {
        Ok(backend) => backend,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": {
                        "message": format!("Failed to load model: {}", e),
                        "type": "invalid_request_error",
                        "param": "model",
                        "code": null
                    }
                })),
            ).into_response();
        }
    };

    let inference_params = InferenceParams {
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        stream: request.stream,
    };

    if request.stream {
        // Handle streaming response
        handle_streaming_completion(&request, backend, prompt, inference_params).await.into_response()
    } else {
        // Handle non-streaming response
        handle_non_streaming_completion(&request, backend, prompt, inference_params).await.into_response()
    }
}

pub async fn embeddings(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<EmbeddingRequest>,
) -> impl IntoResponse {
    // Extract input
    let inputs = match request.input {
        StringOrArray::String(s) => vec![s],
        StringOrArray::Array(arr) => arr,
    };

    // Get or load the backend
    let backend = match get_or_load_backend(&state, &request.model).await {
        Ok(backend) => backend,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": {
                        "message": format!("Failed to load model: {}", e),
                        "type": "invalid_request_error",
                        "param": "model",
                        "code": null
                    }
                })),
            ).into_response();
        }
    };

    let mut embeddings_data = Vec::new();
    let mut total_tokens = 0u32;

    for (index, input) in inputs.iter().enumerate() {
        // BackendHandle already provides async methods, no need for explicit locking
        match backend.get_embeddings(input).await {
            Ok(embedding) => {
                embeddings_data.push(EmbeddingData {
                    object: "embedding".to_string(),
                    embedding,
                    index: index as u32,
                });
                total_tokens += estimate_tokens(input);
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": {
                            "message": format!("Failed to generate embeddings: {}", e),
                            "type": "internal_error",
                            "param": null,
                            "code": null
                        }
                    })),
                ).into_response();
            }
        }
    }

    let response = EmbeddingResponse {
        object: "list".to_string(),
        data: embeddings_data,
        model: request.model,
        usage: EmbeddingUsage {
            prompt_tokens: total_tokens,
            total_tokens,
        },
    };

    Json(response).into_response()
}

pub async fn list_models(
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    match state.model_manager.list_models().await {
        Ok(models) => {
            let model_objects: Vec<ModelObject> = models
                .into_iter()
                .map(|model| ModelObject {
                    id: model.name.clone(),
                    object: "model".to_string(),
                    created: model.modified.timestamp(),
                    owned_by: "inferno".to_string(),
                    permission: vec![],
                    root: model.name,
                    parent: None,
                })
                .collect();

            let response = ModelListResponse {
                object: "list".to_string(),
                data: model_objects,
            };

            Json(response).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": {
                        "message": format!("Failed to list models: {}", e),
                        "type": "internal_error",
                        "param": null,
                        "code": null
                    }
                })),
            ).into_response()
        }
    }
}

// Helper functions

async fn get_or_load_backend(
    state: &Arc<ServerState>,
    model_name: &str,
) -> anyhow::Result<BackendHandle> {
    // If distributed inference is available, we don't need a direct backend
    // This function should not be called when using distributed inference
    if state.distributed.is_some() {
        return Err(anyhow::anyhow!("Cannot load direct backend when using distributed inference"));
    }

    // Check if we have a loaded backend and if it matches the requested model
    if let Some(ref loaded_model) = state.loaded_model {
        if loaded_model == model_name {
            if let Some(ref backend) = state.backend {
                return Ok(backend.clone());
            }
        }
    }

    // For now, if the model doesn't match, we load a new one
    // In a more sophisticated implementation, we'd cache multiple backends
    let model_info = state.model_manager.resolve_model(model_name).await?;
    let backend_type = BackendType::from_model_path(&model_info.path);
    let backend_handle = BackendHandle::new_shared(backend_type, &state.config.backend_config)?;
    backend_handle.load_model(&model_info).await?;

    Ok(backend_handle)
}

fn format_chat_messages(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .map(|msg| format!("{}: {}", msg.role, msg.content))
        .collect::<Vec<_>>()
        .join("\n")
}

fn estimate_tokens(text: &str) -> u32 {
    (text.len() as f32 / 4.0).ceil() as u32
}

async fn handle_non_streaming_chat(
    request: &ChatCompletionRequest,
    backend: BackendHandle,
    prompt: String,
    params: InferenceParams,
) -> impl IntoResponse {
    // BackendHandle already provides async methods, no need for explicit locking

    match backend.infer(&prompt, &params).await {
        Ok(output) => {
            let response = ChatCompletionResponse {
                id: format!("chatcmpl-{}", Uuid::new_v4()),
                object: "chat.completion".to_string(),
                created: chrono::Utc::now().timestamp(),
                model: request.model.clone(),
                choices: vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: "assistant".to_string(),
                        content: output.clone(),
                        name: None,
                    },
                    finish_reason: "stop".to_string(),
                }],
                usage: Usage {
                    prompt_tokens: estimate_tokens(&prompt),
                    completion_tokens: estimate_tokens(&output),
                    total_tokens: estimate_tokens(&prompt) + estimate_tokens(&output),
                },
            };

            Json(response).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": {
                        "message": format!("Inference failed: {}", e),
                        "type": "internal_error",
                        "param": null,
                        "code": null
                    }
                })),
            ).into_response()
        }
    }
}

async fn handle_streaming_chat(
    request: &ChatCompletionRequest,
    backend: BackendHandle,
    prompt: String,
    params: InferenceParams,
) -> impl IntoResponse {
    use axum::response::sse::{Event, Sse};
    use futures::stream::StreamExt;

    let model = request.model.clone();
    let request_id = format!("chatcmpl-{}", Uuid::new_v4());

    let stream = async_stream::stream! {
        // BackendHandle already provides async methods, no need for explicit locking

        match backend.infer_stream(&prompt, &params).await {
            Ok(mut token_stream) => {
                // Send initial chunk with role
                let initial_chunk = ChatCompletionChunk {
                    id: request_id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp(),
                    model: model.clone(),
                    choices: vec![ChatChunkChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: Some("assistant".to_string()),
                            content: None,
                        },
                        finish_reason: None,
                    }],
                };

                yield Ok::<axum::response::sse::Event, axum::Error>(Event::default().data(serde_json::to_string(&initial_chunk).unwrap()));

                // Stream tokens
                while let Some(token_result) = token_stream.next().await {
                    match token_result {
                        Ok(token) => {
                            let chunk = ChatCompletionChunk {
                                id: request_id.clone(),
                                object: "chat.completion.chunk".to_string(),
                                created: chrono::Utc::now().timestamp(),
                                model: model.clone(),
                                choices: vec![ChatChunkChoice {
                                    index: 0,
                                    delta: ChatDelta {
                                        role: None,
                                        content: Some(token),
                                    },
                                    finish_reason: None,
                                }],
                            };

                            yield Ok(Event::default().data(serde_json::to_string(&chunk).unwrap()));
                        }
                        Err(e) => {
                            tracing::error!("Stream error: {}", e);
                            break;
                        }
                    }
                }

                // Send final chunk
                let final_chunk = ChatCompletionChunk {
                    id: request_id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp(),
                    model: model.clone(),
                    choices: vec![ChatChunkChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: None,
                            content: None,
                        },
                        finish_reason: Some("stop".to_string()),
                    }],
                };

                yield Ok(Event::default().data(serde_json::to_string(&final_chunk).unwrap()));
                yield Ok(Event::default().data("[DONE]"));
            }
            Err(e) => {
                let error_msg = serde_json::json!({
                    "error": {
                        "message": format!("Stream failed: {}", e),
                        "type": "internal_error"
                    }
                });
                yield Ok(Event::default().data(serde_json::to_string(&error_msg).unwrap()));
            }
        }
    };

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
        .into_response()
}

async fn handle_non_streaming_completion(
    request: &CompletionRequest,
    backend: BackendHandle,
    prompt: String,
    params: InferenceParams,
) -> impl IntoResponse {
    // BackendHandle already provides async methods, no need for explicit locking

    match backend.infer(&prompt, &params).await {
        Ok(output) => {
            let response = CompletionResponse {
                id: format!("cmpl-{}", Uuid::new_v4()),
                object: "text_completion".to_string(),
                created: chrono::Utc::now().timestamp(),
                model: request.model.clone(),
                choices: vec![CompletionChoice {
                    text: output.clone(),
                    index: 0,
                    logprobs: None,
                    finish_reason: "stop".to_string(),
                }],
                usage: Usage {
                    prompt_tokens: estimate_tokens(&prompt),
                    completion_tokens: estimate_tokens(&output),
                    total_tokens: estimate_tokens(&prompt) + estimate_tokens(&output),
                },
            };

            Json(response).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": {
                        "message": format!("Inference failed: {}", e),
                        "type": "internal_error",
                        "param": null,
                        "code": null
                    }
                })),
            ).into_response()
        }
    }
}

async fn handle_streaming_completion(
    request: &CompletionRequest,
    backend: BackendHandle,
    prompt: String,
    params: InferenceParams,
) -> impl IntoResponse {
    use axum::response::sse::{Event, Sse};
    use futures::stream::StreamExt;

    let model = request.model.clone();
    let request_id = format!("cmpl-{}", Uuid::new_v4());

    let stream = async_stream::stream! {
        // BackendHandle already provides async methods, no need for explicit locking

        match backend.infer_stream(&prompt, &params).await {
            Ok(mut token_stream) => {
                while let Some(token_result) = token_stream.next().await {
                    match token_result {
                        Ok(token) => {
                            let response = CompletionResponse {
                                id: request_id.clone(),
                                object: "text_completion".to_string(),
                                created: chrono::Utc::now().timestamp(),
                                model: model.clone(),
                                choices: vec![CompletionChoice {
                                    text: token,
                                    index: 0,
                                    logprobs: None,
                                    finish_reason: "".to_string(),
                                }],
                                usage: Usage {
                                    prompt_tokens: 0,
                                    completion_tokens: 1,
                                    total_tokens: 1,
                                },
                            };

                            yield Ok::<axum::response::sse::Event, axum::Error>(Event::default().data(serde_json::to_string(&response).unwrap()));
                        }
                        Err(e) => {
                            tracing::error!("Stream error: {}", e);
                            break;
                        }
                    }
                }

                yield Ok(Event::default().data("[DONE]"));
            }
            Err(e) => {
                let error_msg = serde_json::json!({
                    "error": {
                        "message": format!("Stream failed: {}", e),
                        "type": "internal_error"
                    }
                });
                yield Ok(Event::default().data(serde_json::to_string(&error_msg).unwrap()));
            }
        }
    };

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
        .into_response()
}