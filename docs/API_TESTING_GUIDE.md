# Inferno API Testing Guide

This guide covers testing the Inferno OpenAI-compatible API across all endpoints, error scenarios, and performance characteristics.

## Table of Contents

- [Quick Start](#quick-start)
- [Unit Tests](#unit-tests)
- [Integration Tests](#integration-tests)
- [Manual Testing](#manual-testing)
- [Postman Collection](#postman-collection)
- [Performance Testing](#performance-testing)
- [Load Testing](#load-testing)
- [Error Scenario Coverage](#error-scenario-coverage)
- [Compliance Testing](#compliance-testing)

---

## Quick Start

### Prerequisites

1. **Rust & Cargo**: Install from https://rustup.rs/
2. **Node.js** (optional): For Postman CLI testing
3. **curl** (optional): For manual endpoint testing

### Running Tests

```bash
# Run all API tests
cargo test --test api_integration_tests

# Run with output
cargo test --test api_integration_tests -- --nocapture

# Run specific test
cargo test --test api_integration_tests test_chat_completions_basic_request

# Run with multiple threads
cargo test --test api_integration_tests -- --test-threads=4
```

---

## Unit Tests

### Test Coverage Summary

The API integration test suite (`tests/api_integration_tests.rs`) provides comprehensive coverage:

**Total Tests: 60+**

#### Chat Completions (8 tests)
- ✅ Basic request handling
- ✅ Streaming capability
- ✅ Temperature validation (0.0-2.0)
- ✅ Top-P validation (0.0-1.0)
- ✅ Max tokens validation (1-2,000,000)
- ✅ Model validation (required field)
- ✅ Multi-turn conversations
- ✅ Parameter combinations

#### Completions (5 tests)
- ✅ Single string prompts
- ✅ Array prompts
- ✅ Stop sequences
- ✅ Presence/frequency penalties
- ✅ Parameter validation

#### Embeddings (5 tests)
- ✅ Single input
- ✅ Multiple inputs
- ✅ Input length validation (max 8,000 chars)
- ✅ Empty input handling
- ✅ Boundary testing (exactly 8,000 chars)

#### Flow Control (6 tests)
- ✅ Healthy backpressure state
- ✅ Moderate backpressure (70% utilization)
- ✅ Critical backpressure (90% utilization)
- ✅ Buffer overflow handling
- ✅ Message lifecycle (add/send)
- ✅ Token management with limits

#### Streaming Enhancements (8 tests)
- ✅ Compression format parsing
- ✅ Compression header values
- ✅ SSE message formatting
- ✅ SSE optional fields
- ✅ Token batching
- ✅ Token batcher timeout
- ✅ Timeout manager tracking
- ✅ Keep-alive detection

#### OpenAI Compliance (6 tests)
- ✅ Model info creation
- ✅ HTTP status code mapping
- ✅ Error type handling
- ✅ Validation pipeline
- ✅ Concurrent operations
- ✅ Backwards compatibility

#### Combined Scenarios (4 tests)
- ✅ Streaming with flow control
- ✅ Full validation pipeline
- ✅ Concurrent flow control
- ✅ Error combinations

#### Error Scenarios (5 tests)
- ✅ Multiple invalid parameters
- ✅ Empty encoding header
- ✅ Partial matches
- ✅ Timeout expiry
- ✅ Input length exceeding limits

---

## Integration Tests

### Running Full Integration Suite

```bash
# Run all integration tests
./verify.sh

# Run specific integration test
cargo test --test feature_integration_tests

# Run with logging
RUST_LOG=debug cargo test --test feature_integration_tests -- --nocapture
```

### Key Integration Test Areas

1. **End-to-End Workflows**
   - Full chat completion request-response cycle
   - Streaming token delivery
   - Error handling and recovery

2. **Cross-Component Integration**
   - Queue system + API handlers
   - Profiling + inference execution
   - Flow control + WebSocket streaming

3. **Data Flow Verification**
   - Request validation → Processing → Response
   - Token batching → Compression → Transmission
   - Error generation → Error response formatting

---

## Manual Testing

### Test Using curl

#### 1. List Available Models

```bash
curl http://localhost:8000/v1/models
```

Expected response:
```json
{
  "object": "list",
  "data": [
    {
      "id": "llama-7b",
      "object": "model",
      "created": 1694812345,
      "owned_by": "inferno"
    }
  ]
}
```

#### 2. Simple Chat Completion

```bash
curl http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-7b",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

#### 3. Streaming Chat

```bash
curl http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-7b",
    "messages": [{"role": "user", "content": "Write a haiku"}],
    "stream": true
  }' | jq -R 'fromjson?'
```

#### 4. Embeddings

```bash
curl http://localhost:8000/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{
    "model": "text-embedding-ada-002",
    "input": "Test input"
  }' | jq .
```

#### 5. Error Scenario - Invalid Temperature

```bash
curl http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-7b",
    "messages": [{"role": "user", "content": "Hello"}],
    "temperature": 3.0
  }'
```

Expected response: 400 Bad Request with validation error

### Test Using Python

```python
import requests

BASE_URL = "http://localhost:8000/v1"

# Chat Completion
response = requests.post(
    f"{BASE_URL}/chat/completions",
    json={
        "model": "llama-7b",
        "messages": [{"role": "user", "content": "Hello"}],
        "temperature": 0.7
    }
)
print(response.json())

# Streaming
response = requests.post(
    f"{BASE_URL}/chat/completions",
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

# Embeddings
response = requests.post(
    f"{BASE_URL}/embeddings",
    json={
        "model": "text-embedding-ada-002",
        "input": "Test input"
    }
)
embedding = response.json()["data"][0]["embedding"]
print(f"Embedding dimension: {len(embedding)}")
```

### Test Using JavaScript/Node.js

```javascript
const BASE_URL = "http://localhost:8000/v1";

// Chat Completion
const response = await fetch(`${BASE_URL}/chat/completions`, {
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

// Streaming
const streamResponse = await fetch(`${BASE_URL}/chat/completions`, {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    model: "llama-7b",
    messages: [{ role: "user", content: "Write a poem" }],
    stream: true
  })
});

const reader = streamResponse.body.getReader();
const decoder = new TextDecoder();

while (true) {
  const { done, value } = await reader.read();
  if (done) break;
  console.log(decoder.decode(value));
}
```

---

## Postman Collection

### Import Collection

1. Download `docs/Inferno_API_Postman.json`
2. Open Postman
3. Click "Import" → Select the JSON file
4. Configure environment variable:
   - Set `base_url` to `http://localhost:8000/v1`

### Using Postman CLI

```bash
# Install Newman (Postman CLI)
npm install -g newman

# Run collection
newman run docs/Inferno_API_Postman.json \
  -e postman_env.json \
  --reporters cli,json

# Export results
newman run docs/Inferno_API_Postman.json \
  --reporters json \
  --reporter-json-export results.json
```

### Postman Variables

**Collection Variables:**

| Variable | Value | Usage |
|----------|-------|-------|
| `base_url` | `http://localhost:8000/v1` | Base URL for all requests |
| `model` | `llama-7b` | Default model |
| `embedding_model` | `text-embedding-ada-002` | Embedding model |

**Pre-request Scripts** (Optional):

```javascript
// Set timestamp for tracing
pm.environment.set("request_timestamp", new Date().toISOString());

// Generate unique request ID
pm.environment.set("request_id", "req_" + Math.random().toString(36).substr(2, 9));
```

---

## Performance Testing

### Latency Testing

```bash
# Test response time for chat completions
time curl -X POST http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"llama-7b","messages":[{"role":"user","content":"Hello"}]}'
```

### Throughput Testing

```bash
# Test 100 sequential requests
for i in {1..100}; do
  curl -s -X POST http://localhost:8000/v1/chat/completions \
    -H "Content-Type: application/json" \
    -d '{"model":"llama-7b","messages":[{"role":"user","content":"Hi"}]}' > /dev/null
  echo "Request $i completed"
done
```

### Streaming Performance

```bash
# Test streaming throughput
curl -X POST http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"llama-7b","messages":[{"role":"user","content":"Write 1000 words"}],"stream":true}' \
  | wc -l
```

---

## Load Testing

### Using Apache Bench

```bash
# Install ab (comes with Apache)
ab -n 100 -c 10 -p request.json \
  -T application/json \
  http://localhost:8000/v1/chat/completions
```

### Using wrk

```bash
# Install wrk from https://github.com/wg/wrk

wrk -t4 -c100 -d30s \
  --script=test.lua \
  http://localhost:8000/v1/models
```

**test.lua:**
```lua
request = function()
   wrk.method = "GET"
   wrk.uri = "/v1/models"
   return wrk.format(nil)
end

response = function(status, headers, body)
   if status ~= 200 then
      print("Error: " .. status)
   end
end
```

### Using k6

```bash
# Install k6 from https://k6.io/

k6 run load_test.js
```

**load_test.js:**
```javascript
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  vus: 50,
  duration: '30s',
};

export default function() {
  let response = http.post(
    'http://localhost:8000/v1/chat/completions',
    JSON.stringify({
      model: 'llama-7b',
      messages: [{ role: 'user', content: 'Hello' }]
    }),
    {
      headers: { 'Content-Type': 'application/json' }
    }
  );

  check(response, {
    'status is 200': (r) => r.status === 200,
    'has choices': (r) => r.json('choices').length > 0
  });
}
```

---

## Error Scenario Coverage

### Validation Errors (400)

| Scenario | Request | Expected Response |
|----------|---------|-------------------|
| Missing model | `{messages: [...]}` | 400, "model is required" |
| Invalid temperature | `{temperature: 3.0}` | 400, "temperature must be between 0 and 2" |
| Invalid top_p | `{top_p: 1.5}` | 400, "top_p must be between 0 and 1" |
| Invalid max_tokens | `{max_tokens: 0}` | 400, "max_tokens must be between 1 and 2000000" |
| Empty embedding input | `{input: ""}` | 400, "input is required" |
| Embedding too long | `{input: "a" * 8001}` | 400, "input length must not exceed 8000" |

### Not Found Errors (404)

| Scenario | Request | Expected Response |
|----------|---------|-------------------|
| Unknown model | `{model: "unknown"}` | 404, "model not found" |
| Invalid endpoint | GET `/v1/invalid` | 404, "endpoint not found" |

### Server Errors (500)

| Scenario | Cause | Expected Response |
|----------|-------|-------------------|
| Backend crash | Model crash | 500, "inference failed" |
| Memory exhaustion | Large batch | 507, "insufficient memory" |
| Timeout | Long inference | 504, "request timeout" |

### Test Coverage

```bash
# Run error scenario tests
cargo test --test api_integration_tests test_invalid \
  -- --nocapture

# Run with detailed error info
RUST_LOG=debug cargo test --test api_integration_tests \
  test_invalid_chat_all_parameters -- --nocapture
```

---

## Compliance Testing

### OpenAI API Compliance

The API is tested for compatibility with OpenAI's specification:

#### ✅ Fully Compatible
- Chat completions (streaming and non-streaming)
- Text completions (streaming and non-streaming)
- Embeddings (single and batch)
- Model listing
- Error response format
- Status code mapping

#### ⏳ Partially Implemented
- Function calling (planned v0.9.0)
- Logprobs (planned v0.9.0)
- Best of parameter (planned v0.9.0)

#### ✅ Extended Features (Non-OpenAI)
- WebSocket streaming
- Flow control & backpressure
- Server-Sent Events alternative
- Compression support (gzip, deflate, brotli)
- Token batching
- Profiling & monitoring endpoints

### Test Compatibility

```bash
# Test backwards compatibility
cargo test --test api_integration_tests test_openai_request_format_compatibility
cargo test --test api_integration_tests test_openai_response_format_compatibility
```

---

## Continuous Integration

### GitHub Actions

Add to `.github/workflows/api-tests.yml`:

```yaml
name: API Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run API tests
        run: cargo test --test api_integration_tests
      - name: Run full suite
        run: ./verify.sh
```

### Pre-commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
cargo test --test api_integration_tests --quiet
if [ $? -ne 0 ]; then
  echo "API tests failed. Commit aborted."
  exit 1
fi
```

---

## Troubleshooting

### Common Issues

**Issue:** "Connection refused"
- Solution: Ensure server is running (`cargo run -- serve`)

**Issue:** "Model not found"
- Solution: Check models directory and load a model first

**Issue:** Tests timeout
- Solution: Increase timeout or reduce test scale

**Issue:** Port already in use**
- Solution: Change port with `--port 8001` or kill process using port 8000

---

## Best Practices

1. **Isolate tests**: Each test should be independent
2. **Use fixtures**: Pre-create test data/models
3. **Mock external calls**: Don't depend on external services
4. **Test edge cases**: Empty inputs, max values, etc.
5. **Measure performance**: Track latency and throughput
6. **Document scenarios**: Explain what each test validates
7. **Maintain coverage**: Keep coverage >80%
8. **Run regularly**: CI/CD should run tests on every push

---

## Additional Resources

- [OpenAI API Documentation](https://platform.openai.com/docs)
- [Inferno API Documentation](./API_DOCUMENTATION.md)
- [Test Results](#) (see `test-results.html` after running tests)

---

**Last Updated:** 2024-Q4
**Test Coverage:** 60+ unit tests, 100+ integration scenarios
