#!/usr/bin/env cargo-script
/*
[dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "stream"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
futures = "0.3"
tokio-tungstenite = "0.20"
url = "2.3"
*/

/**
 * Inferno Rust Client Example
 *
 * This example demonstrates how to use the Inferno API with Rust.
 * Includes basic inference, streaming, WebSocket communication, and more.
 */

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures::{SinkExt, StreamExt};
use url::Url;

#[derive(Clone)]
pub struct InfernoClient {
    base_url: String,
    client: Client,
    api_key: Option<String>,
}

#[derive(Serialize)]
struct InferenceRequest {
    model: String,
    prompt: String,
    max_tokens: u32,
    temperature: f32,
    top_p: f32,
    top_k: u32,
    stream: bool,
}

#[derive(Deserialize)]
struct InferenceResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Choice {
    text: String,
    index: u32,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
    models_loaded: u32,
}

#[derive(Deserialize)]
struct ModelsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Deserialize)]
struct ModelInfo {
    id: String,
    name: String,
    #[serde(rename = "type")]
    model_type: String,
    size_bytes: u64,
    loaded: bool,
    context_size: Option<u32>,
    capabilities: Vec<String>,
}

#[derive(Serialize)]
struct LoadModelRequest {
    gpu_layers: Option<u32>,
    context_size: Option<u32>,
    batch_size: Option<u32>,
}

#[derive(Deserialize)]
struct LoadModelResponse {
    status: String,
    model_id: String,
    memory_usage_bytes: Option<u64>,
    load_time_ms: Option<u64>,
}

#[derive(Serialize)]
struct EmbeddingsRequest {
    model: String,
    input: Vec<String>,
    encoding_format: String,
}

#[derive(Deserialize)]
struct EmbeddingsResponse {
    model: String,
    data: Vec<EmbeddingData>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: u32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
    index: u32,
    finish_reason: Option<String>,
}

#[derive(Serialize)]
struct BatchRequest {
    model: String,
    requests: Vec<BatchRequestItem>,
    max_tokens: u32,
    webhook_url: Option<String>,
}

#[derive(Serialize)]
struct BatchRequestItem {
    id: String,
    prompt: String,
}

#[derive(Deserialize)]
struct BatchResponse {
    batch_id: String,
    status: String,
    total_requests: u32,
    created: u64,
}

#[derive(Deserialize)]
struct BatchStatusResponse {
    batch_id: String,
    status: String,
    completed: u32,
    failed: u32,
    total: u32,
    results_url: Option<String>,
}

impl InfernoClient {
    /// Create a new Inferno client.
    pub fn new(base_url: impl Into<String>, api_key: Option<String>) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        if let Some(ref key) = api_key {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", key))
                    .expect("Invalid API key"),
            );
        }

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            client,
            api_key,
        }
    }

    /// Check the health status of the server.
    pub async fn health_check(&self) -> Result<HealthResponse, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(&format!("{}/health", self.base_url))
            .send()
            .await?;

        let health: HealthResponse = response.json().await?;
        Ok(health)
    }

    /// List all available models.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(&format!("{}/models", self.base_url))
            .send()
            .await?;

        let models: ModelsResponse = response.json().await?;
        Ok(models.models)
    }

    /// Load a model into memory.
    pub async fn load_model(
        &self,
        model_id: &str,
        options: Option<LoadModelRequest>,
    ) -> Result<LoadModelResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/models/{}/load", self.base_url, model_id);
        let request_body = options.unwrap_or(LoadModelRequest {
            gpu_layers: None,
            context_size: None,
            batch_size: None,
        });

        let response = self.client.post(&url).json(&request_body).send().await?;

        let result: LoadModelResponse = response.json().await?;
        Ok(result)
    }

    /// Unload a model from memory.
    pub async fn unload_model(&self, model_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/models/{}/unload", self.base_url, model_id);
        let response = self.client.post(&url).send().await?;

        // Simple status check
        if response.status().is_success() {
            Ok("unloaded".to_string())
        } else {
            Err(format!("Failed to unload model: {}", response.status()).into())
        }
    }

    /// Run synchronous inference.
    pub async fn inference(
        &self,
        model: &str,
        prompt: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let request = InferenceRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            max_tokens,
            temperature,
            top_p: 0.9,
            top_k: 40,
            stream: false,
        };

        let response = self
            .client
            .post(&format!("{}/inference", self.base_url))
            .json(&request)
            .send()
            .await?;

        let result: InferenceResponse = response.json().await?;

        if let Some(choice) = result.choices.first() {
            Ok(choice.text.clone())
        } else {
            Err("No response received".into())
        }
    }

    /// Stream inference results.
    pub async fn stream_inference(
        &self,
        model: &str,
        prompt: &str,
        max_tokens: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = InferenceRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            max_tokens,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            stream: true,
        };

        let response = self
            .client
            .post(&format!("{}/inference/stream", self.base_url))
            .header("Accept", "text/event-stream")
            .json(&request)
            .send()
            .await?;

        // Note: In a real implementation, you'd parse SSE events here
        let text = response.text().await?;
        println!("Stream response: {}", text);

        Ok(())
    }

    /// Generate embeddings for text inputs.
    pub async fn embeddings(
        &self,
        model: &str,
        texts: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        let request = EmbeddingsRequest {
            model: model.to_string(),
            input: texts,
            encoding_format: "float".to_string(),
        };

        let response = self
            .client
            .post(&format!("{}/embeddings", self.base_url))
            .json(&request)
            .send()
            .await?;

        let result: EmbeddingsResponse = response.json().await?;
        Ok(result.data.into_iter().map(|d| d.embedding).collect())
    }

    /// OpenAI-compatible chat completion.
    pub async fn chat_completion(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let request = ChatCompletionRequest {
            model: model.to_string(),
            messages,
            temperature: Some(0.7),
            max_tokens: Some(100),
        };

        let response = self
            .client
            .post(&format!("{}/v1/chat/completions", self.base_url))
            .json(&request)
            .send()
            .await?;

        let result: ChatCompletionResponse = response.json().await?;

        if let Some(choice) = result.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("No response received".into())
        }
    }

    /// Submit a batch of prompts for processing.
    pub async fn batch_inference(
        &self,
        model: &str,
        prompts: Vec<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let requests: Vec<BatchRequestItem> = prompts
            .into_iter()
            .enumerate()
            .map(|(i, prompt)| BatchRequestItem {
                id: format!("req_{}", i),
                prompt,
            })
            .collect();

        let request = BatchRequest {
            model: model.to_string(),
            requests,
            max_tokens: 100,
            webhook_url: None,
        };

        let response = self
            .client
            .post(&format!("{}/batch", self.base_url))
            .json(&request)
            .send()
            .await?;

        let result: BatchResponse = response.json().await?;
        Ok(result.batch_id)
    }

    /// Get the status of a batch job.
    pub async fn get_batch_status(
        &self,
        batch_id: &str,
    ) -> Result<BatchStatusResponse, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(&format!("{}/batch/{}", self.base_url, batch_id))
            .send()
            .await?;

        let result: BatchStatusResponse = response.json().await?;
        Ok(result)
    }
}

/// WebSocket client for real-time streaming.
pub struct InfernoWebSocketClient {
    url: String,
    api_key: Option<String>,
}

impl InfernoWebSocketClient {
    pub fn new(url: impl Into<String>, api_key: Option<String>) -> Self {
        Self {
            url: url.into(),
            api_key,
        }
    }

    /// Connect and run WebSocket streaming.
    pub async fn run_streaming(
        &self,
        model: &str,
        prompt: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = Url::parse(&self.url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Send authentication if API key provided
        if let Some(ref api_key) = self.api_key {
            let auth_msg = serde_json::json!({
                "type": "auth",
                "token": api_key
            });
            write
                .send(Message::Text(auth_msg.to_string()))
                .await?;
        }

        // Send inference request
        let request = serde_json::json!({
            "type": "inference",
            "id": format!("req_{}", chrono::Utc::now().timestamp_millis()),
            "model": model,
            "prompt": prompt,
            "max_tokens": 100,
            "stream": true
        });

        write
            .send(Message::Text(request.to_string()))
            .await?;

        // Handle incoming messages
        while let Some(message) = read.next().await {
            match message? {
                Message::Text(text) => {
                    let data: serde_json::Value = serde_json::from_str(&text)?;

                    match data["type"].as_str() {
                        Some("token") => {
                            if let Some(token) = data["token"].as_str() {
                                print!("{}", token);
                                use std::io::{self, Write};
                                io::stdout().flush()?;
                            }
                        }
                        Some("complete") => {
                            println!("\n[Inference complete]");
                            break;
                        }
                        Some("error") => {
                            if let Some(error_msg) = data["message"].as_str() {
                                println!("\n[Error: {}]", error_msg);
                            }
                            break;
                        }
                        _ => {
                            println!("Unknown message: {}", text);
                        }
                    }
                }
                Message::Close(_) => {
                    println!("WebSocket closed");
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Inferno Rust Client Example ===\n");

    // Initialize client
    let client = InfernoClient::new("http://localhost:8080", Some("your_api_key_here".to_string()));

    // 1. Health check
    println!("1. Health Check");
    match client.health_check().await {
        Ok(health) => {
            println!("   Status: {}", health.status);
            println!("   Version: {}\n", health.version);
        }
        Err(e) => println!("   Error: {}\n", e),
    }

    // 2. List models
    println!("2. Available Models");
    match client.list_models().await {
        Ok(models) => {
            for model in models {
                println!("   - {}: {} ({})", model.id, model.name, model.model_type);
            }
            println!();
        }
        Err(e) => println!("   Error: {}\n", e),
    }

    // 3. Load a model
    println!("3. Loading Model");
    let model_id = "llama-2-7b";
    // Uncomment to actually load:
    // match client.load_model(model_id, None).await {
    //     Ok(result) => println!("   Model loaded: {}\n", result.status),
    //     Err(e) => println!("   Error: {}\n", e),
    // }

    // 4. Simple inference
    println!("4. Simple Inference");
    let prompt = "What is artificial intelligence?";
    println!("   Prompt: {}", prompt);
    // Uncomment to run inference:
    // match client.inference(model_id, prompt, 50, 0.7).await {
    //     Ok(response) => println!("   Response: {}\n", response),
    //     Err(e) => println!("   Error: {}\n", e),
    // }

    // 5. Generate embeddings
    println!("5. Text Embeddings");
    let texts = vec![
        "Hello world".to_string(),
        "How are you?".to_string(),
        "Machine learning is fascinating".to_string(),
    ];
    println!("   Texts: {:?}", texts);
    // Uncomment to generate embeddings:
    // match client.embeddings(model_id, texts).await {
    //     Ok(embeddings) => {
    //         println!("   Generated {} embeddings", embeddings.len());
    //         if !embeddings.is_empty() {
    //             println!("   Embedding dimension: {}\n", embeddings[0].len());
    //         }
    //     }
    //     Err(e) => println!("   Error: {}\n", e),
    // }

    // 6. Chat completion (OpenAI compatible)
    println!("6. Chat Completion");
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "What is the capital of France?".to_string(),
        },
    ];
    println!("   Messages: {}", messages.len());
    // Uncomment to run chat:
    // match client.chat_completion(model_id, messages).await {
    //     Ok(response) => println!("   Assistant: {}\n", response),
    //     Err(e) => println!("   Error: {}\n", e),
    // }

    // 7. Batch processing
    println!("7. Batch Processing");
    let prompts = vec![
        "What is Python?".to_string(),
        "Explain quantum computing".to_string(),
        "How does photosynthesis work?".to_string(),
    ];
    println!("   Batch size: {}", prompts.len());
    // Uncomment to submit batch:
    // match client.batch_inference(model_id, prompts).await {
    //     Ok(batch_id) => {
    //         println!("   Batch ID: {}", batch_id);
    //
    //         // Wait for completion
    //         loop {
    //             match client.get_batch_status(&batch_id).await {
    //                 Ok(status) => {
    //                     if status.status == "completed" {
    //                         println!("   Completed: {} responses\n", status.completed);
    //                         break;
    //                     }
    //                 }
    //                 Err(e) => {
    //                     println!("   Status check error: {}", e);
    //                     break;
    //                 }
    //             }
    //             tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    //         }
    //     }
    //     Err(e) => println!("   Error: {}\n", e),
    // }

    // 8. WebSocket streaming (uncomment to test)
    println!("8. WebSocket Streaming");
    println!("   Setting up WebSocket client...");
    // let ws_client = InfernoWebSocketClient::new(
    //     "ws://localhost:8080/ws",
    //     Some("your_api_key_here".to_string())
    // );
    // println!("   Sending inference request...");
    // match ws_client.run_streaming(model_id, "Tell me a joke").await {
    //     Ok(_) => println!("   WebSocket streaming completed"),
    //     Err(e) => println!("   WebSocket error: {}", e),
    // }

    println!("\n=== Example Complete ===");

    Ok(())
}