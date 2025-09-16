# Inferno API Documentation

## Table of Contents
- [Overview](#overview)
- [Authentication](#authentication)
- [REST API Endpoints](#rest-api-endpoints)
- [WebSocket API](#websocket-api)
- [OpenAI-Compatible API](#openai-compatible-api)
- [Metrics & Monitoring](#metrics--monitoring)
- [Error Handling](#error-handling)
- [Rate Limiting](#rate-limiting)
- [Examples](#examples)

## Overview

Inferno provides multiple API interfaces for AI/ML model inference:

- **REST API**: Standard HTTP endpoints for synchronous inference
- **WebSocket API**: Real-time bidirectional streaming
- **OpenAI-Compatible API**: Drop-in replacement for OpenAI API
- **Metrics API**: Prometheus-compatible metrics endpoint

### Base URL
```
http://localhost:8080
```

### Content Types
- Request: `application/json`
- Response: `application/json`
- Streaming: `text/event-stream` (SSE) or WebSocket

## Authentication

Inferno supports multiple authentication methods:

### API Key Authentication
Include your API key in the `Authorization` header:
```http
Authorization: Bearer YOUR_API_KEY
```

### JWT Token Authentication
For session-based authentication:
```http
Authorization: Bearer YOUR_JWT_TOKEN
```

### Obtaining Credentials

#### Generate API Key
```bash
inferno security api-key create --user USER_ID --name "My API Key"
```

#### Login for JWT Token
```bash
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "user", "password": "pass"}'
```

## REST API Endpoints

### Health Check
Check service health status.

```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "models_loaded": 2
}
```

### List Models
Get available models.

```http
GET /models
```

**Response:**
```json
{
  "models": [
    {
      "id": "llama-2-7b",
      "name": "Llama 2 7B",
      "type": "gguf",
      "size_bytes": 7516192768,
      "loaded": true,
      "context_size": 4096,
      "capabilities": ["text-generation", "embeddings"]
    }
  ]
}
```

### Load Model
Load a model into memory.

```http
POST /models/{model_id}/load
```

**Request:**
```json
{
  "gpu_layers": 32,
  "context_size": 2048,
  "batch_size": 512
}
```

**Response:**
```json
{
  "status": "loaded",
  "model_id": "llama-2-7b",
  "memory_usage_bytes": 8589934592,
  "load_time_ms": 5432
}
```

### Unload Model
Unload a model from memory.

```http
POST /models/{model_id}/unload
```

**Response:**
```json
{
  "status": "unloaded",
  "model_id": "llama-2-7b"
}
```

### Inference
Run inference on a loaded model.

```http
POST /inference
```

**Request:**
```json
{
  "model": "llama-2-7b",
  "prompt": "What is the capital of France?",
  "max_tokens": 100,
  "temperature": 0.7,
  "top_p": 0.9,
  "top_k": 40,
  "repeat_penalty": 1.1,
  "stop": ["\n", "###"],
  "stream": false
}
```

**Response:**
```json
{
  "id": "inf_123456",
  "model": "llama-2-7b",
  "choices": [
    {
      "text": "The capital of France is Paris.",
      "index": 0,
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 8,
    "completion_tokens": 7,
    "total_tokens": 15
  },
  "created": 1704067200,
  "processing_time_ms": 234
}
```

### Streaming Inference
Stream inference results using Server-Sent Events.

```http
POST /inference/stream
```

**Request:** Same as regular inference with `"stream": true`

**Response (SSE):**
```
data: {"token": "The", "index": 0}
data: {"token": " capital", "index": 1}
data: {"token": " of", "index": 2}
data: {"token": " France", "index": 3}
data: {"token": " is", "index": 4}
data: {"token": " Paris", "index": 5}
data: {"token": ".", "index": 6}
data: {"done": true, "finish_reason": "stop"}
```

### Embeddings
Generate text embeddings.

```http
POST /embeddings
```

**Request:**
```json
{
  "model": "llama-2-7b",
  "input": ["Hello world", "How are you?"],
  "encoding_format": "float"
}
```

**Response:**
```json
{
  "model": "llama-2-7b",
  "data": [
    {
      "embedding": [0.023, -0.445, 0.192, ...],
      "index": 0
    },
    {
      "embedding": [0.011, -0.234, 0.567, ...],
      "index": 1
    }
  ],
  "usage": {
    "prompt_tokens": 5,
    "total_tokens": 5
  }
}
```

### Batch Processing
Submit batch inference jobs.

```http
POST /batch
```

**Request:**
```json
{
  "model": "llama-2-7b",
  "requests": [
    {"id": "req1", "prompt": "What is AI?"},
    {"id": "req2", "prompt": "Explain quantum computing"}
  ],
  "max_tokens": 100,
  "webhook_url": "https://example.com/webhook"
}
```

**Response:**
```json
{
  "batch_id": "batch_789",
  "status": "processing",
  "total_requests": 2,
  "created": 1704067200
}
```

### Get Batch Status
Check batch job status.

```http
GET /batch/{batch_id}
```

**Response:**
```json
{
  "batch_id": "batch_789",
  "status": "completed",
  "completed": 2,
  "failed": 0,
  "total": 2,
  "results_url": "/batch/batch_789/results"
}
```

## WebSocket API

Connect to the WebSocket endpoint for real-time streaming:

```
ws://localhost:8080/ws
```

### Connection
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
ws.onopen = () => {
  ws.send(JSON.stringify({
    type: 'auth',
    token: 'YOUR_API_KEY'
  }));
};
```

### Request Format
```json
{
  "type": "inference",
  "id": "req_123",
  "model": "llama-2-7b",
  "prompt": "Tell me a story",
  "max_tokens": 200,
  "stream": true
}
```

### Response Format
```json
{
  "type": "token",
  "id": "req_123",
  "token": "Once",
  "index": 0
}
```

### Message Types
- `auth`: Authentication
- `inference`: Inference request
- `cancel`: Cancel ongoing inference
- `ping`/`pong`: Keep-alive
- `error`: Error message
- `token`: Streaming token
- `complete`: Inference complete

## OpenAI-Compatible API

Inferno provides OpenAI API compatibility for easy migration.

### Chat Completions
```http
POST /v1/chat/completions
```

**Request:**
```json
{
  "model": "llama-2-7b",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "What is the weather like?"}
  ],
  "temperature": 0.7,
  "max_tokens": 100,
  "stream": false
}
```

**Response:**
```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1704067200,
  "model": "llama-2-7b",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "I don't have access to real-time weather data..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 20,
    "completion_tokens": 15,
    "total_tokens": 35
  }
}
```

### Completions (Legacy)
```http
POST /v1/completions
```

**Request:**
```json
{
  "model": "llama-2-7b",
  "prompt": "Once upon a time",
  "max_tokens": 50,
  "temperature": 0.8
}
```

### Models List
```http
GET /v1/models
```

**Response:**
```json
{
  "object": "list",
  "data": [
    {
      "id": "llama-2-7b",
      "object": "model",
      "created": 1704067200,
      "owned_by": "local"
    }
  ]
}
```

## Metrics & Monitoring

### Prometheus Metrics
```http
GET /metrics
```

**Response (Prometheus format):**
```
# HELP inferno_inference_requests_total Total inference requests
# TYPE inferno_inference_requests_total counter
inferno_inference_requests_total{model="llama-2-7b"} 1234

# HELP inferno_inference_duration_seconds Inference duration
# TYPE inferno_inference_duration_seconds histogram
inferno_inference_duration_seconds_bucket{le="0.1"} 100
inferno_inference_duration_seconds_bucket{le="0.5"} 450
inferno_inference_duration_seconds_bucket{le="1.0"} 890
```

### OpenTelemetry Traces
```http
GET /traces
```

**Response:**
```json
{
  "traces": [
    {
      "trace_id": "abc123",
      "span_id": "def456",
      "operation_name": "inference.llama-2-7b",
      "start_time": "2024-01-01T12:00:00Z",
      "duration_ms": 234,
      "status": "ok"
    }
  ]
}
```

### Custom Metrics
```http
POST /metrics/custom
```

**Request:**
```json
{
  "name": "custom_metric",
  "value": 42.5,
  "type": "gauge",
  "labels": {
    "environment": "production"
  }
}
```

## Error Handling

All API errors follow a consistent format:

```json
{
  "error": {
    "code": "MODEL_NOT_FOUND",
    "message": "Model 'gpt-5' not found",
    "details": {
      "available_models": ["llama-2-7b", "mistral-7b"]
    }
  },
  "request_id": "req_abc123",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### Error Codes
- `INVALID_REQUEST`: Malformed request
- `AUTHENTICATION_FAILED`: Invalid credentials
- `AUTHORIZATION_FAILED`: Insufficient permissions
- `MODEL_NOT_FOUND`: Model doesn't exist
- `MODEL_NOT_LOADED`: Model not in memory
- `RATE_LIMIT_EXCEEDED`: Too many requests
- `CONTEXT_LENGTH_EXCEEDED`: Input too long
- `INFERENCE_FAILED`: Processing error
- `TIMEOUT`: Request timeout
- `INTERNAL_ERROR`: Server error

### HTTP Status Codes
- `200 OK`: Success
- `400 Bad Request`: Invalid request
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Access denied
- `404 Not Found`: Resource not found
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error
- `503 Service Unavailable`: Service overloaded

## Rate Limiting

Rate limits are enforced per API key or IP address:

### Default Limits
- **Requests per minute**: 60
- **Requests per hour**: 1000
- **Tokens per minute**: 10000
- **Concurrent requests**: 10

### Rate Limit Headers
```http
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1704067260
X-RateLimit-Reset-After: 30
```

### Rate Limit Response
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Please retry after 30 seconds.",
    "retry_after": 30
  }
}
```

## Examples

### Python Example
```python
import requests
import json

# Configuration
API_KEY = "your_api_key"
BASE_URL = "http://localhost:8080"

headers = {
    "Authorization": f"Bearer {API_KEY}",
    "Content-Type": "application/json"
}

# Simple inference
response = requests.post(
    f"{BASE_URL}/inference",
    headers=headers,
    json={
        "model": "llama-2-7b",
        "prompt": "What is machine learning?",
        "max_tokens": 100,
        "temperature": 0.7
    }
)

result = response.json()
print(result["choices"][0]["text"])

# Streaming inference with SSE
import sseclient

response = requests.post(
    f"{BASE_URL}/inference/stream",
    headers=headers,
    json={
        "model": "llama-2-7b",
        "prompt": "Explain quantum physics",
        "max_tokens": 200,
        "stream": True
    },
    stream=True
)

client = sseclient.SSEClient(response)
for event in client.events():
    data = json.loads(event.data)
    if "token" in data:
        print(data["token"], end="", flush=True)
    elif "done" in data:
        break
```

### JavaScript/TypeScript Example
```typescript
// Configuration
const API_KEY = 'your_api_key';
const BASE_URL = 'http://localhost:8080';

// Simple inference
async function runInference(prompt: string): Promise<string> {
  const response = await fetch(`${BASE_URL}/inference`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${API_KEY}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      model: 'llama-2-7b',
      prompt: prompt,
      max_tokens: 100,
      temperature: 0.7
    })
  });

  const result = await response.json();
  return result.choices[0].text;
}

// WebSocket streaming
function streamInference(prompt: string) {
  const ws = new WebSocket(`ws://localhost:8080/ws`);

  ws.onopen = () => {
    // Authenticate
    ws.send(JSON.stringify({
      type: 'auth',
      token: API_KEY
    }));

    // Send inference request
    ws.send(JSON.stringify({
      type: 'inference',
      id: 'req_' + Date.now(),
      model: 'llama-2-7b',
      prompt: prompt,
      max_tokens: 200,
      stream: true
    }));
  };

  ws.onmessage = (event) => {
    const data = JSON.parse(event.data);

    if (data.type === 'token') {
      process.stdout.write(data.token);
    } else if (data.type === 'complete') {
      console.log('\nDone!');
      ws.close();
    } else if (data.type === 'error') {
      console.error('Error:', data.message);
      ws.close();
    }
  };
}
```

### cURL Examples
```bash
# Health check
curl http://localhost:8080/health

# List models
curl -H "Authorization: Bearer $API_KEY" \
  http://localhost:8080/models

# Run inference
curl -X POST \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-2-7b",
    "prompt": "Hello, how are you?",
    "max_tokens": 50
  }' \
  http://localhost:8080/inference

# Stream inference
curl -X POST \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "model": "llama-2-7b",
    "prompt": "Tell me a joke",
    "max_tokens": 100,
    "stream": true
  }' \
  http://localhost:8080/inference/stream

# OpenAI-compatible chat
curl -X POST \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-2-7b",
    "messages": [
      {"role": "user", "content": "What is 2+2?"}
    ]
  }' \
  http://localhost:8080/v1/chat/completions
```

### Go Example
```go
package main

import (
    "bytes"
    "encoding/json"
    "fmt"
    "net/http"
)

const (
    API_KEY  = "your_api_key"
    BASE_URL = "http://localhost:8080"
)

type InferenceRequest struct {
    Model      string   `json:"model"`
    Prompt     string   `json:"prompt"`
    MaxTokens  int      `json:"max_tokens"`
    Temperature float64 `json:"temperature"`
}

type InferenceResponse struct {
    Choices []struct {
        Text string `json:"text"`
    } `json:"choices"`
}

func runInference(prompt string) (string, error) {
    reqBody := InferenceRequest{
        Model:       "llama-2-7b",
        Prompt:      prompt,
        MaxTokens:   100,
        Temperature: 0.7,
    }

    jsonData, _ := json.Marshal(reqBody)

    req, err := http.NewRequest("POST", BASE_URL+"/inference",
        bytes.NewBuffer(jsonData))
    if err != nil {
        return "", err
    }

    req.Header.Set("Authorization", "Bearer "+API_KEY)
    req.Header.Set("Content-Type", "application/json")

    client := &http.Client{}
    resp, err := client.Do(req)
    if err != nil {
        return "", err
    }
    defer resp.Body.Close()

    var result InferenceResponse
    json.NewDecoder(resp.Body).Decode(&result)

    if len(result.Choices) > 0 {
        return result.Choices[0].Text, nil
    }

    return "", fmt.Errorf("no response")
}
```

### Rust Example
```rust
use reqwest;
use serde::{Deserialize, Serialize};

const API_KEY: &str = "your_api_key";
const BASE_URL: &str = "http://localhost:8080";

#[derive(Serialize)]
struct InferenceRequest {
    model: String,
    prompt: String,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Deserialize)]
struct InferenceResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    text: String,
}

async fn run_inference(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let request = InferenceRequest {
        model: "llama-2-7b".to_string(),
        prompt: prompt.to_string(),
        max_tokens: 100,
        temperature: 0.7,
    };

    let response = client
        .post(format!("{}/inference", BASE_URL))
        .header("Authorization", format!("Bearer {}", API_KEY))
        .json(&request)
        .send()
        .await?
        .json::<InferenceResponse>()
        .await?;

    Ok(response.choices[0].text.clone())
}
```

## SDK Support

Official SDKs are planned for:
- Python (`inferno-python`)
- JavaScript/TypeScript (`@inferno/client`)
- Go (`github.com/inferno-ai/go-client`)
- Rust (`inferno-client`)
- Java (`io.inferno:client`)
- C# (`Inferno.Client`)

## Webhooks

Configure webhooks for async events:

```json
{
  "webhook_url": "https://example.com/webhook",
  "events": ["inference.complete", "batch.complete", "model.loaded"],
  "secret": "webhook_secret_key"
}
```

### Webhook Payload
```json
{
  "event": "inference.complete",
  "timestamp": "2024-01-01T12:00:00Z",
  "data": {
    "request_id": "req_123",
    "model": "llama-2-7b",
    "tokens_generated": 50,
    "duration_ms": 234
  },
  "signature": "sha256=abcdef123456..."
}
```

## API Versioning

The API follows semantic versioning:
- Current version: `v1`
- Version in URL: `/v1/endpoint`
- Header: `API-Version: 1.0`

### Deprecation Policy
- Deprecated endpoints marked with `Deprecation` header
- Minimum 6 months notice before removal
- Migration guides provided

## Security Best Practices

1. **Always use HTTPS in production**
2. **Rotate API keys regularly**
3. **Implement request signing for webhooks**
4. **Use rate limiting to prevent abuse**
5. **Enable audit logging**
6. **Validate and sanitize all inputs**
7. **Implement timeout for long-running requests**
8. **Use authentication for all endpoints**

## Support

- Documentation: https://docs.inferno.ai
- GitHub Issues: https://github.com/inferno-ai/inferno/issues
- Discord: https://discord.gg/inferno
- Email: support@inferno.ai