use crate::{
    api::openai::{ChatCompletionRequest, ChatMessage, ChatDelta, ChatCompletionChunk, ChatChunkChoice},
    backends::{Backend, InferenceParams},
    cli::serve::ServerState,
    streaming::{StreamingConfig, StreamingManager, StreamingToken},
    InfernoError,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// WebSocket message types for streaming inference
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WSMessage {
    #[serde(rename = "chat_request")]
    ChatRequest {
        id: String,
        data: ChatCompletionRequest,
    },
    #[serde(rename = "chat_chunk")]
    ChatChunk {
        id: String,
        data: ChatCompletionChunk,
    },
    #[serde(rename = "error")]
    Error {
        id: Option<String>,
        message: String,
        code: String,
    },
    #[serde(rename = "heartbeat")]
    Heartbeat {
        timestamp: chrono::DateTime<chrono::Utc>,
        active_streams: usize,
    },
    #[serde(rename = "stream_metrics")]
    StreamMetrics {
        active_streams: usize,
        total_tokens: u64,
        average_latency: f32,
    },
    #[serde(rename = "connection_info")]
    ConnectionInfo {
        connection_id: String,
        server_version: String,
        capabilities: Vec<String>,
    },
}

/// WebSocket streaming handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> Response {
    info!("New WebSocket connection for streaming");

    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

/// Handle individual WebSocket connection
async fn handle_websocket(socket: WebSocket, state: Arc<ServerState>) {
    let connection_id = Uuid::new_v4().to_string();
    info!("WebSocket connection established: {}", connection_id);

    // Initialize streaming manager for this connection
    let streaming_config = StreamingConfig {
        max_concurrent_streams: 5, // Limit per connection
        enable_metrics: true,
        heartbeat_interval_ms: 30000, // 30 second heartbeat
        ..Default::default()
    };

    let streaming_manager = Arc::new(StreamingManager::new(streaming_config));
    if let Err(e) = streaming_manager.start().await {
        error!("Failed to start streaming manager: {}", e);
        return;
    }

    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    // Send connection info
    let connection_info = WSMessage::ConnectionInfo {
        connection_id: connection_id.clone(),
        server_version: env!("CARGO_PKG_VERSION").to_string(),
        capabilities: vec![
            "streaming_chat".to_string(),
            "real_time_metrics".to_string(),
            "heartbeat".to_string(),
        ],
    };

    if let Ok(msg) = serde_json::to_string(&connection_info) {
        if sender.lock().await.send(Message::Text(msg)).await.is_err() {
            return;
        }
    }

    // Start heartbeat task
    let heartbeat_sender = sender.clone();
    let heartbeat_manager = streaming_manager.clone();
    let heartbeat_connection_id = connection_id.clone();

    let heartbeat_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

        loop {
            interval.tick().await;

            let metrics = heartbeat_manager.get_metrics();
            let heartbeat = WSMessage::Heartbeat {
                timestamp: chrono::Utc::now(),
                active_streams: metrics.active_streams,
            };

            if let Ok(msg) = serde_json::to_string(&heartbeat) {
                if heartbeat_sender.lock().await.send(Message::Text(msg)).await.is_err() {
                    debug!("Heartbeat failed for connection: {}", heartbeat_connection_id);
                    break;
                }
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<WSMessage>(&text) {
                    Ok(ws_message) => {
                        if let Err(e) = handle_ws_message(
                            ws_message,
                            &state,
                            &streaming_manager,
                            &sender,
                            &connection_id,
                        ).await {
                            error!("Error handling WebSocket message: {}", e);

                            let error_msg = WSMessage::Error {
                                id: None,
                                message: format!("Message handling failed: {}", e),
                                code: "INTERNAL_ERROR".to_string(),
                            };

                            if let Ok(error_json) = serde_json::to_string(&error_msg) {
                                let _ = sender.lock().await.send(Message::Text(error_json)).await;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Invalid WebSocket message format: {}", e);

                        let error_msg = WSMessage::Error {
                            id: None,
                            message: format!("Invalid message format: {}", e),
                            code: "INVALID_FORMAT".to_string(),
                        };

                        if let Ok(error_json) = serde_json::to_string(&error_msg) {
                            let _ = sender.lock().await.send(Message::Text(error_json)).await;
                        }
                    }
                }
            }
            Ok(Message::Binary(_)) => {
                warn!("Binary messages not supported in streaming WebSocket");
            }
            Ok(Message::Ping(data)) => {
                debug!("Received ping, sending pong");
                let _ = sender.lock().await.send(Message::Pong(data)).await;
            }
            Ok(Message::Pong(_)) => {
                debug!("Received pong");
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed: {}", connection_id);
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Cleanup
    heartbeat_handle.abort();
    info!("WebSocket connection handler finished: {}", connection_id);
}

/// Handle specific WebSocket message types
async fn handle_ws_message(
    message: WSMessage,
    state: &Arc<ServerState>,
    streaming_manager: &Arc<StreamingManager>,
    sender: &Arc<Mutex<futures::stream::SplitSink<WebSocket, Message>>>,
    connection_id: &str,
) -> Result<(), InfernoError> {
    match message {
        WSMessage::ChatRequest { id, data } => {
            info!("Processing chat request {} for connection {}", id, connection_id);

            // Get or load backend
            let backend = get_or_load_backend_for_ws(state, &data.model).await?;

            // Convert chat messages to prompt
            let prompt = format_chat_messages(&data.messages);

            let inference_params = InferenceParams {
                max_tokens: data.max_tokens,
                temperature: data.temperature,
                top_p: data.top_p,
                stream: true, // Always stream for WebSocket
            };

            // Create streaming session
            let mut stream = streaming_manager.create_enhanced_stream(
                &mut *backend.lock().await,
                &prompt,
                &inference_params,
            ).await.map_err(|e| InfernoError::WebSocket(format!("Stream creation failed: {}", e)))?;

            let sender_clone = sender.clone();
            let request_id = id.clone();
            let model_name = data.model.clone();

            // Spawn streaming task
            tokio::spawn(async move {
                // Send initial chunk with role
                let initial_chunk = ChatCompletionChunk {
                    id: request_id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp(),
                    model: model_name.clone(),
                    choices: vec![ChatChunkChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: Some("assistant".to_string()),
                            content: None,
                        },
                        finish_reason: None,
                    }],
                };

                let initial_ws_msg = WSMessage::ChatChunk {
                    id: request_id.clone(),
                    data: initial_chunk,
                };

                if let Ok(msg_json) = serde_json::to_string(&initial_ws_msg) {
                    let _ = sender_clone.lock().await.send(Message::Text(msg_json)).await;
                }

                // Stream tokens
                while let Some(token_result) = stream.next().await {
                    match token_result {
                        Ok(streaming_token) => {
                            if !streaming_token.is_heartbeat() {
                                let chunk = ChatCompletionChunk {
                                    id: request_id.clone(),
                                    object: "chat.completion.chunk".to_string(),
                                    created: chrono::Utc::now().timestamp(),
                                    model: model_name.clone(),
                                    choices: vec![ChatChunkChoice {
                                        index: 0,
                                        delta: ChatDelta {
                                            role: None,
                                            content: Some(streaming_token.content),
                                        },
                                        finish_reason: None,
                                    }],
                                };

                                let ws_msg = WSMessage::ChatChunk {
                                    id: request_id.clone(),
                                    data: chunk,
                                };

                                if let Ok(msg_json) = serde_json::to_string(&ws_msg) {
                                    if sender_clone.lock().await.send(Message::Text(msg_json)).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Streaming error: {}", e);

                            let error_msg = WSMessage::Error {
                                id: Some(request_id.clone()),
                                message: format!("Streaming failed: {}", e),
                                code: "STREAM_ERROR".to_string(),
                            };

                            if let Ok(error_json) = serde_json::to_string(&error_msg) {
                                let _ = sender_clone.lock().await.send(Message::Text(error_json)).await;
                            }
                            break;
                        }
                    }
                }

                // Send final chunk
                let final_chunk = ChatCompletionChunk {
                    id: request_id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp(),
                    model: model_name,
                    choices: vec![ChatChunkChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: None,
                            content: None,
                        },
                        finish_reason: Some("stop".to_string()),
                    }],
                };

                let final_ws_msg = WSMessage::ChatChunk {
                    id: request_id,
                    data: final_chunk,
                };

                if let Ok(msg_json) = serde_json::to_string(&final_ws_msg) {
                    let _ = sender_clone.lock().await.send(Message::Text(msg_json)).await;
                }
            });

            Ok(())
        }
        _ => {
            warn!("Unsupported WebSocket message type");
            Err(InfernoError::WebSocket("Unsupported message type".to_string()))
        }
    }
}

/// Get or load backend for WebSocket connection
async fn get_or_load_backend_for_ws(
    state: &Arc<ServerState>,
    model_name: &str,
) -> Result<Arc<tokio::sync::Mutex<Backend>>, InfernoError> {
    // Similar to the HTTP API version but optimized for WebSocket
    if let Some(ref distributed) = state.distributed {
        return Err(InfernoError::WebSocket(
            "WebSocket streaming not supported with distributed inference yet".to_string()
        ));
    }

    // Check if we have a loaded backend matching the model
    if let Some(ref loaded_model) = state.loaded_model {
        if loaded_model == model_name {
            if let Some(ref backend) = state.backend {
                return Ok(backend.clone());
            }
        }
    }

    // Load new backend for this model
    let model_info = state.model_manager.resolve_model(model_name).await
        .map_err(|e| InfernoError::WebSocket(format!("Model resolution failed: {}", e)))?;

    let backend_type = crate::backends::BackendType::from_model_path(&model_info.path);
    let mut backend = Backend::new(backend_type, &state.config.backend_config)
        .map_err(|e| InfernoError::WebSocket(format!("Backend creation failed: {}", e)))?;

    backend.load_model(&model_info).await
        .map_err(|e| InfernoError::WebSocket(format!("Model loading failed: {}", e)))?;

    Ok(Arc::new(tokio::sync::Mutex::new(backend)))
}

/// Format chat messages into a single prompt
fn format_chat_messages(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .map(|msg| format!("{}: {}", msg.role, msg.content))
        .collect::<Vec<_>>()
        .join("\n")
}