# Inferno API Documentation

Inferno provides a fully OpenAI-compatible REST API for inference, embeddings, model management, and streaming operations. This documentation covers all endpoints, request/response formats, error handling, and usage examples.

## Table of Contents

- [Overview](#overview)
- [Authentication](#authentication)
- [API Endpoints](#api-endpoints)
- [Chat Completions](#chat-completions)
- [Completions](#completions)
- [Embeddings](#embeddings)
- [Models](#models)
- [WebSocket Streaming](#websocket-streaming)
- [Flow Control & Backpressure](#flow-control--backpressure)
- [Streaming Enhancements](#streaming-enhancements)
- [Error Handling](#error-handling)
- [Rate Limiting](#rate-limiting)
- [Examples](#examples)
- [OpenAI Compatibility Matrix](#openai-compatibility-matrix)

---

## Overview

Inferno is a drop-in replacement for OpenAI's API that runs inference locally on your hardware. The API follows OpenAI's specification for maximum compatibility with existing applications.

### Base URL

```
http://localhost:8000/v1
```

### API Version

```
2023-06-01
```

### Supported Models

- LLaMA models (7B, 13B, 70B, etc.)
- Mistral models
- Mixtral models
- Custom GGUF models
- ONNX models

---

## Authentication

Inferno currently operates with optional API keys for backwards compatibility. Future versions will support more sophisticated authentication.

```bash
curl -H "Authorization: Bearer YOUR_API_KEY" \
  http://localhost:8000/v1/chat/completions
```

---

## API Endpoints

### Base Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/models` | List available models |
| POST | `/chat/completions` | Chat completion |
| POST | `/completions` | Text completion |
| POST | `/embeddings` | Generate embeddings |

### Streaming Endpoints

| Protocol | Endpoint | Description |
|----------|----------|-------------|
| WebSocket | `/ws/stream` | WebSocket streaming |
| SSE | `/stream/sse` | Server-Sent Events streaming |

### Profiling & Monitoring Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/metrics/profiles/recent` | Recent inference profiles |
| GET | `/metrics/profiles/stats` | Aggregated statistics |
| GET | `/metrics/queue/status` | Queue status |
| GET | `/health` | Health check |

---

## Chat Completions

Generate chat-based completions with message history support.

### Request

```
POST /v1/chat/completions
Content-Type: application/json
```

### Request Body

```json
{
  "model": "llama-7b",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant."
    },
    {
      "role": "user",
      "content": "What is machine learning?"
    }
  ],
  "temperature": 0.7,
  "top_p": 0.9,
  "top_k": 40,
  "max_tokens": 512,
  "stream": false,
  "stop": ["\n", "Human:"],
  "presence_penalty": 0.0,
  "frequency_penalty": 0.0
}
```

### Request Parameters

| Parameter | Type | Default | Range | Description |
|-----------|------|---------|-------|-------------|
| `model` | string | required | - | Model identifier |
| `messages` | array | required | - | Array of message objects |
| `temperature` | float | 0.7 | 0.0-2.0 | Sampling temperature |
| `top_p` | float | 0.9 | 0.0-1.0 | Nucleus sampling parameter |
| `top_k` | integer | 40 | 1-100 | Top-K sampling |
| `max_tokens` | integer | 512 | 1-2,000,000 | Max output tokens |
| `stream` | boolean | false | - | Stream responses |
| `stop` | array | null | - | Stop sequences |
| `presence_penalty` | float | 0.0 | -2.0-2.0 | Presence penalty |
| `frequency_penalty` | float | 0.0 | -2.0-2.0 | Frequency penalty |
| `user` | string | null | - | User identifier |

### Message Object

```json
{
  "role": "user|assistant|system",
  "content": "Message content",
  "name": "optional_name"
}
```

### Response (Non-Streaming)

```json
{
  "id": "chatcmpl-8c1a2b3c4d5e6f7g8h9i",
  "object": "chat.completion",
  "created": 1694812345,
  "model": "llama-7b",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Machine learning is a subset of artificial intelligence..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 45,
    "completion_tokens": 150,
    "total_tokens": 195
  }
}
```

### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique request ID |
| `object` | string | Always "chat.completion" |
| `created` | integer | Unix timestamp |
| `model` | string | Model used |
| `choices` | array | Completion choices |
| `choices[].message` | object | Generated message |
| `choices[].finish_reason` | string | "stop" or "length" |
| `usage` | object | Token usage |

### Response (Streaming)

When `stream: true`, responses are sent as Server-Sent Events:

```
data: {"id":"chatcmpl-...","object":"chat.completion.chunk","choices":[{"delta":{"role":"assistant"},"index":0}]}

data: {"id":"chatcmpl-...","object":"chat.completion.chunk","choices":[{"delta":{"content":"Machine"},"index":0}]}

data: {"id":"chatcmpl-...","object":"chat.completion.chunk","choices":[{"delta":{"content":" learning"},"index":0}]}

...

data: [DONE]
```

---

## Completions

Generate text completions from prompts.

### Request

```
POST /v1/completions
Content-Type: application/json
```

### Request Body

```json
{
  "model": "llama-7b",
  "prompt": "The future of AI is",
  "max_tokens": 100,
  "temperature": 0.8,
  "top_p": 0.9,
  "top_k": 40,
  "stream": false,
  "stop": ["\n\n"],
  "presence_penalty": 0.0,
  "frequency_penalty": 0.0
}
```

### Prompt Formats

**Single string:**
```json
{"prompt": "Hello, world!"}
```

**Array of strings:**
```json
{"prompt": ["Hello, world!", "Hi there!", "Greetings!"]}
```

### Response

```json
{
  "id": "cmpl-8c1a2b3c4d5e6f7g8h9i",
  "object": "text_completion",
  "created": 1694812345,
  "model": "llama-7b",
  "choices": [
    {
      "text": " is boundless and filled with possibilities",
      "index": 0,
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 50,
    "total_tokens": 60
  }
}
```

---

## Embeddings

Generate vector embeddings from text.

### Request

```
POST /v1/embeddings
Content-Type: application/json
```

### Request Body

```json
{
  "model": "text-embedding-ada-002",
  "input": "The quick brown fox jumps over the lazy dog"
}
```

### Input Formats

**Single string:**
```json
{"input": "Single text"}
```

**Array of strings:**
```json
{"input": ["Text 1", "Text 2", "Text 3"]}
```

### Input Constraints

- Maximum input length: 8,000 characters
- Maximum batch size: 100 inputs
- Empty inputs are rejected

### Response

```json
{
  "object": "list",
  "data": [
    {
      "object": "embedding",
      "embedding": [0.0023064255, -0.009327292, ... 1536 values total],
      "index": 0
    }
  ],
  "model": "text-embedding-ada-002",
  "usage": {
    "prompt_tokens": 20,
    "total_tokens": 20
  }
}
```

### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `object` | string | Always "list" |
| `data` | array | Embedding objects |
| `data[].embedding` | array | Float vector (1536 dimensions) |
| `data[].index` | integer | Input index |
| `model` | string | Model used |
| `usage` | object | Token usage |

---

## Models

### List Models

Get all available models.

#### Request

```
GET /v1/models
```

#### Response

```json
{
  "object": "list",
  "data": [
    {
      "id": "llama-7b",
      "object": "model",
      "created": 1694812345,
      "owned_by": "inferno",
      "permission": [],
      "root": "llama-7b",
      "parent": null
    },
    {
      "id": "llama-13b",
      "object": "model",
      "created": 1694812346,
      "owned_by": "inferno",
      "permission": [],
      "root": "llama-13b",
      "parent": null
    }
  ]
}
```

---

## WebSocket Streaming

Real-time streaming via WebSocket connections with flow control.

### Connection

```
ws://localhost:8000/ws/stream
```

### Message Format

```json
{
  "type": "inference",
  "model": "llama-7b",
  "prompt": "Write a poem",
  "params": {
    "max_tokens": 100,
    "temperature": 0.7
  }
}
```

### Response Messages

**Initial response:**
```json
{
  "type": "start",
  "request_id": "req_123",
  "model": "llama-7b"
}
```

**Token chunks:**
```json
{
  "type": "token",
  "token": "Roses",
  "accumulated": "Roses"
}
```

**Completion:**
```json
{
  "type": "complete",
  "total_tokens": 45,
  "finish_reason": "stop"
}
```

### Flow Control

The API implements automatic flow control with three backpressure levels:

| Level | Buffer Usage | Behavior |
|-------|-------------|----------|
| Healthy | 0-70% | Normal processing |
| Moderate | 70-90% | Apply light backpressure |
| Critical | >90% | Apply heavy backpressure, may drop tokens |

---

## Streaming Enhancements

### Compression Support

Inferno supports multiple compression formats for bandwidth optimization.

#### Accept-Encoding Header

```
Accept-Encoding: gzip, deflate, br
```

#### Supported Formats

| Format | Header Value | Compression Ratio |
|--------|--------------|-------------------|
| None | (none) | 1.0x |
| gzip | `gzip` | 2.5-3.5x |
| deflate | `deflate` | 2.0-3.0x |
| brotli | `br` | 3.0-4.0x |

### Server-Sent Events

Alternative to WebSocket streaming using HTTP Server-Sent Events.

```
GET /v1/stream/sse?model=llama-7b&prompt=...
```

Response format:
```
event: token
data: {"token":"Hello","index":0}

event: token
data: {"token":" ","index":1}

event: complete
data: {"total_tokens":45,"finish_reason":"stop"}
```

### Token Batching

Tokens are automatically batched (2-3 tokens per message) to reduce frame overhead.

**Default settings:**
- Batch size: 3 tokens
- Max wait time: 50ms
- Reduces HTTP frames by ~66%

### Timeouts

| Timeout | Default | Description |
|---------|---------|-------------|
| Inference | 5 minutes | Total request timeout |
| Token | 30 seconds | Time between tokens |
| ACK | 30 seconds | Acknowledgment timeout |
| Keep-alive | 30 seconds | Connection heartbeat |

---

## Error Handling

### Error Response Format

```json
{
  "error": {
    "message": "Model not found",
    "type": "invalid_request_error",
    "param": "model",
    "code": "model_not_found"
  }
}
```

### HTTP Status Codes

| Code | Type | Description |
|------|------|-------------|
| 200 | Success | Request successful |
| 400 | Bad Request | Invalid parameters |
| 401 | Unauthorized | Authentication failed |
| 403 | Forbidden | Permission denied |
| 404 | Not Found | Model not found |
| 500 | Server Error | Internal server error |
| 504 | Gateway Timeout | Request timeout |
| 507 | Insufficient Storage | Out of memory |

### Error Types

| Type | Status | Description |
|------|--------|-------------|
| `invalid_request_error` | 400 | Invalid request parameters |
| `authentication_error` | 401 | Authentication failed |
| `permission_error` | 403 | Insufficient permissions |
| `not_found_error` | 404 | Resource not found |
| `rate_limit_error` | 429 | Rate limit exceeded |
| `server_error` | 500 | Server error |
| `timeout_error` | 504 | Request timeout |

---

## Rate Limiting

### Rate Limit Headers

```
X-RateLimit-Limit-Requests: 100
X-RateLimit-Limit-Tokens: 10000
X-RateLimit-Remaining-Requests: 98
X-RateLimit-Remaining-Tokens: 9850
X-RateLimit-Reset-Requests: 1694812400
X-RateLimit-Reset-Tokens: 1694812500
```

### Default Limits

- **Requests:** 100 per minute
- **Tokens:** 10,000 per minute
- **Concurrent connections:** 100

### Exceeding Limits

```json
{
  "error": {
    "message": "Rate limit exceeded",
    "type": "rate_limit_error"
  }
}
```

---

## Examples

### Example 1: Simple Chat Completion

```bash
curl http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-7b",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'
```

### Example 2: Streaming Chat Completion

```bash
curl http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-7b",
    "messages": [
      {"role": "user", "content": "Write a haiku"}
    ],
    "stream": true
  }' | jq -R 'fromjson?'
```

### Example 3: Embeddings with Multiple Inputs

```bash
curl http://localhost:8000/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{
    "model": "text-embedding-ada-002",
    "input": [
      "Embedding 1",
      "Embedding 2",
      "Embedding 3"
    ]
  }'
```

### Example 4: List Models

```bash
curl http://localhost:8000/v1/models
```

### Example 5: Python Client

```python
import requests

# Chat Completion
response = requests.post(
    "http://localhost:8000/v1/chat/completions",
    json={
        "model": "llama-7b",
        "messages": [{"role": "user", "content": "Hello"}],
        "temperature": 0.7
    }
)

result = response.json()
print(result["choices"][0]["message"]["content"])

# Streaming
response = requests.post(
    "http://localhost:8000/v1/chat/completions",
    json={
        "model": "llama-7b",
        "messages": [{"role": "user", "content": "Write a poem"}],
        "stream": True
    },
    stream=True
)

for line in response.iter_lines():
    if line:
        print(line.decode('utf-8'))
```

### Example 6: JavaScript/Node.js

```javascript
const response = await fetch("http://localhost:8000/v1/chat/completions", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    model: "llama-7b",
    messages: [{ role: "user", content: "Hello" }],
    temperature: 0.7
  })
});

const result = await response.json();
console.log(result.choices[0].message.content);
```

---

## OpenAI Compatibility Matrix

Inferno aims for maximum compatibility with OpenAI's API. This matrix shows the compatibility status for each endpoint and parameter.

### Chat Completions

| Feature | Status | Notes |
|---------|--------|-------|
| Basic completion | ✅ Supported | Full compatibility |
| Streaming | ✅ Supported | Server-Sent Events |
| Temperature | ✅ Supported | 0.0-2.0 range |
| Top P | ✅ Supported | 0.0-1.0 range |
| Top K | ✅ Supported | Inferno extension |
| Max Tokens | ✅ Supported | 1-2,000,000 |
| Stop Sequences | ✅ Supported | Multiple sequences |
| Penalties | ✅ Supported | Presence & frequency |
| System Prompts | ✅ Supported | Via message role |
| Function Calling | ⏳ Planned | v0.9.0 |

### Completions

| Feature | Status | Notes |
|---------|--------|-------|
| Text completion | ✅ Supported | Full compatibility |
| Streaming | ✅ Supported | Server-Sent Events |
| Logprobs | ⏳ Planned | v0.9.0 |
| Echo | ✅ Supported | Returns prompt |
| Best of | ⏳ Planned | v0.9.0 |

### Embeddings

| Feature | Status | Notes |
|---------|--------|-------|
| Single input | ✅ Supported | Full compatibility |
| Batch inputs | ✅ Supported | Up to 100 inputs |
| Dimensions | ✅ Supported | 1536 default |
| Encoding format | ⏳ Planned | v0.9.0 |

### Models

| Feature | Status | Notes |
|---------|--------|-------|
| List models | ✅ Supported | Full compatibility |
| Get model info | ✅ Supported | Standard format |
| Create model | ❌ Not Supported | N/A for local models |
| Delete model | ❌ Not Supported | N/A for local models |

### Extensions (Non-OpenAI)

| Feature | Description | Status |
|---------|-------------|--------|
| WebSocket Streaming | Real-time token streaming | ✅ v0.8.0 |
| Flow Control | Adaptive backpressure | ✅ v0.8.0 |
| Compression | gzip/deflate/brotli | ✅ v0.8.0 |
| Profiling | Performance metrics | ✅ v0.8.0 |
| Queue Management | Request prioritization | ✅ v0.8.0 |

---

## Support & Feedback

For issues, feature requests, or feedback:
- GitHub: https://github.com/anthropics/inferno
- Email: support@inferno.ai
- Documentation: https://docs.inferno.ai

---

**Last Updated:** 2024-Q4
**API Version:** 2023-06-01 (OpenAI compatible)
**Inferno Version:** v0.8.0
