# 🌐 API Reference

Complete documentation for Inferno's REST API and WebSocket interfaces. Inferno provides OpenAI-compatible endpoints plus extended functionality for advanced AI operations.

## Base URL and Authentication

**Base URL**: `http://localhost:8080` (default)
**API Version**: `/v1`
**Content-Type**: `application/json`

### Authentication

Inferno supports multiple authentication methods:

```bash
# No authentication (default for local development)
curl http://localhost:8080/v1/models

# API Key authentication
curl -H "Authorization: Bearer your-api-key" http://localhost:8080/v1/models

# JWT authentication
curl -H "Authorization: Bearer jwt-token" http://localhost:8080/v1/models
```

## Quick Start

```bash
# Test the API
curl http://localhost:8080/v1/models

# Generate text
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt2",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'
```

## OpenAI-Compatible Endpoints

Inferno implements the OpenAI API specification for seamless integration with existing tools and libraries.

### Chat Completions

**Endpoint**: `POST /v1/chat/completions`

Generate conversational responses using chat-formatted input.

#### Request

```json
{
  "model": "llama-2-7b",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Explain quantum computing"}
  ],
  "max_tokens": 150,
  "temperature": 0.7,
  "top_p": 0.9,
  "stream": false,
  "stop": ["\\n\\n"],
  "presence_penalty": 0.0,
  "frequency_penalty": 0.0,
  "user": "user123"
}
```

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `model` | string | ✅ | - | Model identifier |
| `messages` | array | ✅ | - | Chat conversation history |
| `max_tokens` | integer | ❌ | 2048 | Maximum tokens to generate |
| `temperature` | float | ❌ | 0.7 | Sampling temperature (0.0-2.0) |
| `top_p` | float | ❌ | 1.0 | Nucleus sampling parameter |
| `stream` | boolean | ❌ | false | Stream response in real-time |
| `stop` | array | ❌ | null | Stop sequences |
| `presence_penalty` | float | ❌ | 0.0 | Presence penalty (-2.0 to 2.0) |
| `frequency_penalty` | float | ❌ | 0.0 | Frequency penalty (-2.0 to 2.0) |
| `user` | string | ❌ | null | User identifier for tracking |

#### Response

```json
{
  "id": "chatcmpl-7QyqpwdfhqwajicIEznoc6Q47XAyW",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "llama-2-7b",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Quantum computing is a revolutionary approach to computation..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 25,
    "completion_tokens": 150,
    "total_tokens": 175
  }
}
```

#### Streaming Response

When `stream: true`, responses are sent as Server-Sent Events:

```
data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"llama-2-7b","choices":[{"index":0,"delta":{"content":"Quantum"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"llama-2-7b","choices":[{"index":0,"delta":{"content":" computing"},"finish_reason":null}]}

data: [DONE]
```

### Text Completions

**Endpoint**: `POST /v1/completions`

Generate text completions from a prompt.

#### Request

```json
{
  "model": "gpt2",
  "prompt": "The future of artificial intelligence is",
  "max_tokens": 100,
  "temperature": 0.8,
  "top_p": 1.0,
  "n": 1,
  "stream": false,
  "logprobs": null,
  "echo": false,
  "stop": ["\\n"],
  "presence_penalty": 0.0,
  "frequency_penalty": 0.0,
  "best_of": 1,
  "user": "user123"
}
```

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `model` | string | ✅ | - | Model identifier |
| `prompt` | string/array | ✅ | - | Input text or array of texts |
| `max_tokens` | integer | ❌ | 2048 | Maximum tokens to generate |
| `temperature` | float | ❌ | 1.0 | Sampling temperature |
| `top_p` | float | ❌ | 1.0 | Nucleus sampling |
| `n` | integer | ❌ | 1 | Number of completions |
| `stream` | boolean | ❌ | false | Stream response |
| `logprobs` | integer | ❌ | null | Include log probabilities |
| `echo` | boolean | ❌ | false | Echo back prompt |
| `stop` | array | ❌ | null | Stop sequences |
| `presence_penalty` | float | ❌ | 0.0 | Presence penalty |
| `frequency_penalty` | float | ❌ | 0.0 | Frequency penalty |
| `best_of` | integer | ❌ | 1 | Generate best_of completions |
| `user` | string | ❌ | null | User identifier |

#### Response

```json
{
  "id": "cmpl-7QyqpwdfhqwajicIEznoc6Q47XAyW",
  "object": "text_completion",
  "created": 1677652288,
  "model": "gpt2",
  "choices": [
    {
      "text": " bright and full of possibilities. Machine learning algorithms...",
      "index": 0,
      "logprobs": null,
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 8,
    "completion_tokens": 92,
    "total_tokens": 100
  }
}
```

### Embeddings

**Endpoint**: `POST /v1/embeddings`

Generate embeddings for text inputs.

#### Request

```json
{
  "model": "text-embedding-ada-002",
  "input": ["Hello world", "How are you?"],
  "user": "user123"
}
```

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `model` | string | ✅ | - | Embedding model identifier |
| `input` | string/array | ✅ | - | Text(s) to embed |
| `user` | string | ❌ | null | User identifier |

#### Response

```json
{
  "object": "list",
  "data": [
    {
      "object": "embedding",
      "embedding": [0.1, 0.2, 0.3, ...],
      "index": 0
    },
    {
      "object": "embedding",
      "embedding": [0.4, 0.5, 0.6, ...],
      "index": 1
    }
  ],
  "model": "text-embedding-ada-002",
  "usage": {
    "prompt_tokens": 8,
    "total_tokens": 8
  }
}
```

### Models

**Endpoint**: `GET /v1/models`

List available models.

#### Response

```json
{
  "object": "list",
  "data": [
    {
      "id": "gpt2",
      "object": "model",
      "created": 1677652288,
      "owned_by": "openai",
      "permission": [],
      "root": "gpt2",
      "parent": null
    },
    {
      "id": "llama-2-7b",
      "object": "model",
      "created": 1677652288,
      "owned_by": "meta",
      "permission": [],
      "root": "llama-2-7b",
      "parent": null
    }
  ]
}
```

**Endpoint**: `GET /v1/models/{model_id}`

Get specific model information.

#### Response

```json
{
  "id": "gpt2",
  "object": "model",
  "created": 1677652288,
  "owned_by": "openai",
  "permission": [],
  "root": "gpt2",
  "parent": null
}
```

## Inferno Extended API

Extended functionality beyond OpenAI compatibility.

### Model Management

#### Load Model

**Endpoint**: `POST /v1/models/{model_id}/load`

Load a model into memory.

```bash
curl -X POST http://localhost:8080/v1/models/llama-2-7b/load
```

**Response**:
```json
{
  "status": "success",
  "message": "Model loaded successfully",
  "model_id": "llama-2-7b",
  "backend": "gguf",
  "memory_usage": "4.2GB"
}
```

#### Unload Model

**Endpoint**: `POST /v1/models/{model_id}/unload`

Unload a model from memory.

```bash
curl -X POST http://localhost:8080/v1/models/llama-2-7b/unload
```

#### Model Status

**Endpoint**: `GET /v1/models/{model_id}/status`

Get model loading status and performance metrics.

```json
{
  "model_id": "llama-2-7b",
  "status": "loaded",
  "backend": "gguf",
  "memory_usage": "4.2GB",
  "metrics": {
    "total_requests": 1234,
    "avg_latency_ms": 125,
    "tokens_per_second": 45.2,
    "last_used": "2024-01-15T10:30:00Z"
  }
}
```

### Batch Processing

**Endpoint**: `POST /v1/batch`

Process multiple requests in a single call.

#### Request

```json
{
  "model": "gpt2",
  "requests": [
    {
      "id": "req1",
      "messages": [{"role": "user", "content": "Hello"}]
    },
    {
      "id": "req2",
      "messages": [{"role": "user", "content": "Goodbye"}]
    }
  ],
  "max_tokens": 50
}
```

#### Response

```json
{
  "object": "batch_completion",
  "responses": [
    {
      "id": "req1",
      "choices": [{"message": {"content": "Hello! How can I help?"}}],
      "usage": {"total_tokens": 12}
    },
    {
      "id": "req2",
      "choices": [{"message": {"content": "Goodbye! Have a great day!"}}],
      "usage": {"total_tokens": 15}
    }
  ]
}
```

### Performance and Monitoring

#### Health Check

**Endpoint**: `GET /health`

System health status.

```json
{
  "status": "healthy",
  "version": "1.0.0",
  "models_loaded": ["gpt2", "llama-2-7b"],
  "system": {
    "cpu_usage": 45.2,
    "memory_usage": "8.5GB",
    "gpu_usage": 78.1,
    "uptime": "2h 15m 30s"
  }
}
```

#### Metrics

**Endpoint**: `GET /metrics`

Prometheus-compatible metrics.

```
# HELP inferno_requests_total Total number of requests
# TYPE inferno_requests_total counter
inferno_requests_total{model="gpt2",endpoint="chat"} 1234

# HELP inferno_request_duration_seconds Request duration
# TYPE inferno_request_duration_seconds histogram
inferno_request_duration_seconds_bucket{le="0.1"} 100
inferno_request_duration_seconds_bucket{le="0.5"} 500
```

### Model Conversion

**Endpoint**: `POST /v1/convert`

Convert models between formats.

#### Request

```json
{
  "source_model": "model.pt",
  "target_format": "gguf",
  "optimization": "balanced",
  "quantization": "q4_0"
}
```

#### Response

```json
{
  "job_id": "conv_123456",
  "status": "processing",
  "estimated_duration": "5m",
  "progress": 15
}
```

**Check Progress**: `GET /v1/convert/{job_id}`

```json
{
  "job_id": "conv_123456",
  "status": "completed",
  "output_file": "model.gguf",
  "size_reduction": "35%",
  "duration": "4m 23s"
}
```

### Authentication and Security

#### Generate API Key

**Endpoint**: `POST /v1/auth/keys`

```json
{
  "name": "my-app",
  "permissions": ["inference", "model_management"],
  "expires_in": "30d"
}
```

**Response**:
```json
{
  "key_id": "key_abc123",
  "api_key": "sk-1234567890abcdef",
  "expires": "2024-02-15T10:30:00Z",
  "permissions": ["inference", "model_management"]
}
```

#### User Information

**Endpoint**: `GET /v1/auth/user`

```json
{
  "user_id": "user123",
  "username": "alice",
  "role": "admin",
  "permissions": ["*"],
  "api_keys": 3,
  "last_login": "2024-01-15T09:00:00Z"
}
```

## WebSocket API

Real-time communication for streaming inference and live monitoring.

### Connection

**URL**: `ws://localhost:8080/ws`

### Message Format

All WebSocket messages use JSON format:

```json
{
  "type": "request_type",
  "id": "unique_request_id",
  "data": { /* request-specific data */ }
}
```

### Chat Streaming

Send real-time chat requests and receive streaming responses.

#### Request

```json
{
  "type": "chat",
  "id": "chat_001",
  "data": {
    "model": "llama-2-7b",
    "messages": [
      {"role": "user", "content": "Tell me a story"}
    ],
    "stream": true
  }
}
```

#### Response Stream

```json
{"type": "chat_chunk", "id": "chat_001", "data": {"delta": {"content": "Once"}}}
{"type": "chat_chunk", "id": "chat_001", "data": {"delta": {"content": " upon"}}}
{"type": "chat_chunk", "id": "chat_001", "data": {"delta": {"content": " a"}}}
{"type": "chat_done", "id": "chat_001", "data": {"usage": {"total_tokens": 150}}}
```

### Live Metrics

Subscribe to real-time system and model metrics.

#### Subscribe

```json
{
  "type": "subscribe_metrics",
  "id": "metrics_001",
  "data": {
    "interval": 1000,
    "metrics": ["cpu", "memory", "gpu", "inference_rate"]
  }
}
```

#### Metric Updates

```json
{
  "type": "metrics_update",
  "id": "metrics_001",
  "data": {
    "timestamp": "2024-01-15T10:30:00Z",
    "cpu": 45.2,
    "memory": 68.5,
    "gpu": 78.1,
    "inference_rate": 12.3
  }
}
```

### Model Status

Monitor model loading, unloading, and status changes.

#### Subscribe

```json
{
  "type": "subscribe_models",
  "id": "models_001"
}
```

#### Status Updates

```json
{
  "type": "model_status",
  "id": "models_001",
  "data": {
    "model_id": "llama-2-7b",
    "status": "loading",
    "progress": 65,
    "eta": "30s"
  }
}
```

## Error Handling

### HTTP Status Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 200 | OK | Request successful |
| 201 | Created | Resource created successfully |
| 400 | Bad Request | Invalid request format or parameters |
| 401 | Unauthorized | Authentication required |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Model or resource not found |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server error |
| 503 | Service Unavailable | Server overloaded or maintenance |

### Error Response Format

```json
{
  "error": {
    "type": "invalid_request_error",
    "code": "model_not_found",
    "message": "The requested model 'gpt-5' was not found",
    "details": {
      "available_models": ["gpt2", "llama-2-7b"],
      "suggestion": "Use 'GET /v1/models' to see available models"
    }
  }
}
```

### Common Error Codes

| Code | Description | Solution |
|------|-------------|----------|
| `model_not_found` | Requested model doesn't exist | Check available models with `/v1/models` |
| `model_not_loaded` | Model exists but isn't loaded | Load model with `/v1/models/{id}/load` |
| `invalid_parameters` | Request parameters are invalid | Check parameter types and ranges |
| `rate_limit_exceeded` | Too many requests | Reduce request rate or upgrade plan |
| `insufficient_memory` | Not enough memory to load model | Free memory or use smaller model |
| `gpu_unavailable` | GPU acceleration unavailable | Check GPU drivers or use CPU |
| `authentication_failed` | Invalid credentials | Check API key or authentication |

## Code Examples

### Python with OpenAI Library

```python
from openai import OpenAI

# Configure client for Inferno
client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="your-api-key"  # or "not-needed" for no auth
)

# Chat completion
response = client.chat.completions.create(
    model="llama-2-7b",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Explain machine learning"}
    ]
)
print(response.choices[0].message.content)

# Streaming chat
for chunk in client.chat.completions.create(
    model="llama-2-7b",
    messages=[{"role": "user", "content": "Tell me a joke"}],
    stream=True
):
    if chunk.choices[0].delta.content:
        print(chunk.choices[0].delta.content, end="")

# Embeddings
embeddings = client.embeddings.create(
    model="text-embedding-ada-002",
    input=["Hello world", "How are you?"]
)
print(embeddings.data[0].embedding)
```

### JavaScript/Node.js

```javascript
// Using fetch API
async function chatCompletion() {
    const response = await fetch('http://localhost:8080/v1/chat/completions', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': 'Bearer your-api-key'
        },
        body: JSON.stringify({
            model: 'gpt2',
            messages: [
                {role: 'user', content: 'Hello AI!'}
            ]
        })
    });

    const data = await response.json();
    console.log(data.choices[0].message.content);
}

// Streaming with Server-Sent Events
async function streamChat() {
    const response = await fetch('http://localhost:8080/v1/chat/completions', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': 'Bearer your-api-key'
        },
        body: JSON.stringify({
            model: 'gpt2',
            messages: [{role: 'user', content: 'Tell me a story'}],
            stream: true
        })
    });

    const reader = response.body.getReader();
    const decoder = new TextDecoder();

    while (true) {
        const { value, done } = await reader.read();
        if (done) break;

        const chunk = decoder.decode(value);
        const lines = chunk.split('\\n');

        for (const line of lines) {
            if (line.startsWith('data: ')) {
                const data = line.slice(6);
                if (data === '[DONE]') return;

                try {
                    const parsed = JSON.parse(data);
                    const content = parsed.choices[0]?.delta?.content;
                    if (content) process.stdout.write(content);
                } catch (e) {
                    // Skip invalid JSON
                }
            }
        }
    }
}
```

### cURL Examples

```bash
# Simple chat completion
curl -X POST http://localhost:8080/v1/chat/completions \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer your-api-key" \\
  -d '{
    "model": "gpt2",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'

# Text completion with parameters
curl -X POST http://localhost:8080/v1/completions \\
  -H "Content-Type: application/json" \\
  -d '{
    "model": "gpt2",
    "prompt": "The future of AI is",
    "max_tokens": 50,
    "temperature": 0.8
  }'

# Get embeddings
curl -X POST http://localhost:8080/v1/embeddings \\
  -H "Content-Type: application/json" \\
  -d '{
    "model": "text-embedding-ada-002",
    "input": "Hello world"
  }'

# List models
curl http://localhost:8080/v1/models

# Load a model
curl -X POST http://localhost:8080/v1/models/llama-2-7b/load

# Check health
curl http://localhost:8080/health

# Get metrics
curl http://localhost:8080/metrics
```

### Rust with reqwest

```rust
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Chat completion
    let response = client
        .post("http://localhost:8080/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer your-api-key")
        .json(&json!({
            "model": "gpt2",
            "messages": [
                {"role": "user", "content": "Hello AI!"}
            ]
        }))
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;
    println!("{}", result["choices"][0]["message"]["content"]);

    Ok(())
}
```

## Rate Limiting

Inferno implements rate limiting to prevent abuse and ensure fair usage.

### Headers

Rate limit information is included in response headers:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
```

### Configuration

Configure rate limits in `inferno.toml`:

```toml
[security]
rate_limit = 1000  # requests per minute
burst_limit = 100  # burst requests
```

## Best Practices

### Performance

1. **Model Loading**: Load models once and reuse them
2. **Batch Requests**: Use batch API for multiple requests
3. **Streaming**: Use streaming for real-time applications
4. **Caching**: Enable response caching for repeated queries

### Security

1. **Authentication**: Always use API keys in production
2. **HTTPS**: Use SSL/TLS in production
3. **Rate Limiting**: Implement client-side rate limiting
4. **Input Validation**: Validate and sanitize inputs

### Error Handling

1. **Retry Logic**: Implement exponential backoff for retries
2. **Status Codes**: Handle different HTTP status codes appropriately
3. **Error Messages**: Parse error responses for actionable information
4. **Fallbacks**: Implement fallback strategies for critical applications

## See Also

- [CLI Reference](cli-reference.md) - Command-line interface
- [OpenAI Compatibility](../examples/openai-compatibility.md) - Drop-in replacement guide
- [WebSocket Integration](../examples/websocket.md) - Real-time streaming examples
- [Performance Tuning](../guides/performance-tuning.md) - API optimization strategies