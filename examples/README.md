# Inferno Examples

This directory contains comprehensive examples for using the Inferno AI/ML inference server.

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [Client Examples](#client-examples)
- [Docker Deployment](#docker-deployment)
- [API Examples](#api-examples)
- [Production Setup](#production-setup)
- [Troubleshooting](#troubleshooting)

## 🚀 Quick Start

### 1. Start Inferno Server

```bash
# Build and run locally
cargo run -- serve --bind 0.0.0.0:8080

# Or using Docker
docker run -p 8080:8080 inferno:latest serve
```

### 2. Test Basic Functionality

```bash
# Health check
curl http://localhost:8080/health

# List models
curl -H "Authorization: Bearer YOUR_API_KEY" \
  http://localhost:8080/models

# Run inference
curl -X POST \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model": "llama-2-7b", "prompt": "Hello!", "max_tokens": 50}' \
  http://localhost:8080/inference
```

## 👨‍💻 Client Examples

### Python Client (`python_client.py`)

A comprehensive Python client with support for:
- Synchronous and streaming inference
- Batch processing
- WebSocket real-time communication
- OpenAI-compatible API
- Embeddings generation

```bash
# Install dependencies
pip install requests websocket-client sseclient-py

# Run example
python python_client.py
```

**Key Features:**
```python
client = InfernoClient(api_key="your_key")

# Simple inference
response = client.inference("llama-2-7b", "What is AI?", max_tokens=100)

# Streaming
for token in client.stream_inference("llama-2-7b", "Tell a story"):
    print(token, end="")

# Chat completion (OpenAI compatible)
response = client.chat_completion("llama-2-7b", [
    {"role": "user", "content": "Hello!"}
])
```

### JavaScript/Node.js Client (`javascript_client.js`)

Full-featured JavaScript client for both Node.js and browser environments:

```bash
# Install dependencies
npm install node-fetch ws eventsource

# Run example
node javascript_client.js
```

**Key Features:**
```javascript
const client = new InfernoClient('http://localhost:8080', 'your_key');

// Async/await inference
const response = await client.inference('llama-2-7b', 'Hello world');

// WebSocket streaming
const wsClient = new InfernoWebSocketClient();
await wsClient.connect();
wsClient.sendInference('llama-2-7b', 'Tell me a joke');
```

### Rust Client (`rust_client.rs`)

High-performance Rust client with full async support:

```bash
# Install cargo-script
cargo install cargo-script

# Run example
cargo script rust_client.rs
```

**Key Features:**
```rust
let client = InfernoClient::new("http://localhost:8080", Some("your_key".to_string()));

// Async inference
let response = client.inference("llama-2-7b", "Hello", 100, 0.7).await?;

// WebSocket streaming
let ws_client = InfernoWebSocketClient::new("ws://localhost:8080/ws", Some("your_key".to_string()));
ws_client.run_streaming("llama-2-7b", "Tell a story").await?;
```

### Go Client (`go_client.go`)

Efficient Go client with comprehensive API coverage:

```bash
# Initialize module
go mod init inferno-example
go get github.com/gorilla/websocket

# Run example
go run go_client.go
```

**Key Features:**
```go
client := NewClient("http://localhost:8080", "your_key")

// Simple inference
response, err := client.Inference("llama-2-7b", "Hello", 100, 0.7)

// WebSocket
wsClient := NewWebSocketClient("ws://localhost:8080/ws", "your_key")
wsClient.Connect()
wsClient.SendInference("llama-2-7b", "Tell a joke", 50)
```

## 🐳 Docker Deployment

### Complete Stack (`docker-compose.yml`)

Deploy Inferno with full observability stack:

```bash
# Start complete stack
docker-compose up -d

# View logs
docker-compose logs -f inferno

# Scale Inferno instances
docker-compose up -d --scale inferno=3
```

**Included Services:**
- **Inferno**: Main inference server (port 8080)
- **Prometheus**: Metrics collection (port 9091)
- **Grafana**: Visualization dashboards (port 3000)
- **Jaeger**: Distributed tracing (port 16686)
- **Redis**: Caching layer (port 6379)
- **Nginx**: Load balancer (port 80/443)

### Access Services

```bash
# Inferno API
curl http://localhost:8080/health

# Prometheus metrics
open http://localhost:9091

# Grafana dashboards (admin/admin)
open http://localhost:3000

# Jaeger tracing
open http://localhost:16686
```

## 📡 API Examples

### Basic Inference

```bash
curl -X POST http://localhost:8080/inference \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-2-7b",
    "prompt": "Explain quantum computing in simple terms",
    "max_tokens": 200,
    "temperature": 0.7
  }'
```

### Streaming Inference

```bash
curl -X POST http://localhost:8080/inference/stream \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "model": "llama-2-7b",
    "prompt": "Write a short poem about AI",
    "max_tokens": 100,
    "stream": true
  }'
```

### OpenAI-Compatible Chat

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-2-7b",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "What is the capital of France?"}
    ]
  }'
```

### Batch Processing

```bash
# Submit batch
curl -X POST http://localhost:8080/batch \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-2-7b",
    "requests": [
      {"id": "1", "prompt": "What is Python?"},
      {"id": "2", "prompt": "Explain machine learning"}
    ],
    "max_tokens": 100
  }'

# Check status
curl http://localhost:8080/batch/BATCH_ID \
  -H "Authorization: Bearer YOUR_API_KEY"

# Get results
curl http://localhost:8080/batch/BATCH_ID/results \
  -H "Authorization: Bearer YOUR_API_KEY"
```

### Embeddings

```bash
curl -X POST http://localhost:8080/embeddings \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-2-7b",
    "input": ["Hello world", "How are you?"],
    "encoding_format": "float"
  }'
```

## 🏭 Production Setup

### Environment Configuration

```bash
# Create environment file
cat > .env << EOF
INFERNO_LOG_LEVEL=info
INFERNO_LOG_FORMAT=json
INFERNO_MODELS_DIR=/data/models
INFERNO_CACHE_DIR=/data/cache
INFERNO_BIND_ADDRESS=0.0.0.0
INFERNO_PORT=8080
INFERNO_MAX_CONCURRENT_REQUESTS=100
INFERNO_REQUEST_TIMEOUT_SECONDS=300

# Security
INFERNO_AUTH_ENABLED=true
INFERNO_JWT_SECRET=your-secret-key
INFERNO_API_KEY_ENABLED=true
INFERNO_RATE_LIMITING_ENABLED=true
INFERNO_MAX_REQUESTS_PER_MINUTE=1000

# Observability
INFERNO_PROMETHEUS_ENABLED=true
INFERNO_PROMETHEUS_PORT=9090
INFERNO_OTEL_ENABLED=true
INFERNO_OTEL_ENDPOINT=http://jaeger:14268/api/traces
INFERNO_GRAFANA_ENABLED=true
INFERNO_GRAFANA_ENDPOINT=http://grafana:3000
EOF
```

### SSL/TLS Configuration

```bash
# Generate self-signed certificates (for testing)
mkdir -p ssl
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout ssl/inferno.key \
  -out ssl/inferno.crt \
  -subj "/C=US/ST=CA/L=SF/O=Inferno/CN=localhost"
```

### Load Balancing

```nginx
# nginx.conf
upstream inferno_backend {
    least_conn;
    server inferno:8080 max_fails=3 fail_timeout=30s;
    server inferno_2:8080 max_fails=3 fail_timeout=30s;
    server inferno_3:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    location / {
        proxy_pass http://inferno_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_connect_timeout 30s;
        proxy_send_timeout 300s;
        proxy_read_timeout 300s;
    }
}
```

### Monitoring Setup

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'inferno'
    static_configs:
      - targets: ['inferno:9090']
    scrape_interval: 15s
    metrics_path: /metrics

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
```

### Health Checks

```bash
#!/bin/bash
# health-check.sh

INFERNO_URL="http://localhost:8080"
API_KEY="your_api_key"

# Health endpoint
health_status=$(curl -s -o /dev/null -w "%{http_code}" "$INFERNO_URL/health")
if [ "$health_status" != "200" ]; then
    echo "Health check failed: $health_status"
    exit 1
fi

# Model availability
models_count=$(curl -s -H "Authorization: Bearer $API_KEY" \
  "$INFERNO_URL/models" | jq '.models | length')
if [ "$models_count" -eq 0 ]; then
    echo "No models available"
    exit 1
fi

echo "Health check passed"
```

## 🔧 Troubleshooting

### Common Issues

#### 1. Model Not Loading

```bash
# Check model file
ls -la /path/to/models/

# Check logs
docker logs inferno-server

# Validate model
inferno validate /path/to/model.gguf
```

#### 2. Memory Issues

```bash
# Check memory usage
docker stats inferno-server

# Reduce model size or use GPU offloading
curl -X POST http://localhost:8080/models/llama-2-7b/load \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{"gpu_layers": 32, "context_size": 2048}'
```

#### 3. Performance Issues

```bash
# Check metrics
curl http://localhost:8080/metrics

# Monitor in Grafana
open http://localhost:3000

# Enable GPU acceleration
inferno gpu status
inferno gpu enable
```

#### 4. API Authentication

```bash
# Generate new API key
inferno security api-key create --user admin --name production

# Test authentication
curl -H "Authorization: Bearer NEW_API_KEY" \
  http://localhost:8080/models
```

#### 5. Network Issues

```bash
# Check Docker networks
docker network ls
docker network inspect inferno_inferno-network

# Test internal connectivity
docker exec inferno-server curl http://prometheus:9090/-/healthy
```

### Debug Mode

```bash
# Run with debug logging
INFERNO_LOG_LEVEL=debug cargo run -- serve

# Enable trace logging
INFERNO_LOG_LEVEL=trace RUST_BACKTRACE=1 cargo run -- serve

# Monitor WebSocket connections
wscat -c ws://localhost:8080/ws
```

### Performance Tuning

```bash
# CPU optimization
export RAYON_NUM_THREADS=8

# Memory optimization
export INFERNO_CACHE_SIZE=4GB
export INFERNO_MODEL_MEMORY_POOL=8GB

# GPU optimization
export CUDA_VISIBLE_DEVICES=0,1
export INFERNO_GPU_MEMORY_FRACTION=0.8
```

## 📚 Additional Resources

- [Full API Documentation](../API.md)
- [Configuration Guide](../CLAUDE.md)
- [Security Best Practices](../SECURITY.md)
- [Performance Optimization](../PERFORMANCE.md)
- [Contributing Guide](../CONTRIBUTING.md)

## 💬 Support

- **GitHub Issues**: [Report bugs or request features](https://github.com/inferno-ai/inferno/issues)
- **Discussions**: [Community discussions](https://github.com/inferno-ai/inferno/discussions)
- **Discord**: [Join our community](https://discord.gg/inferno)
- **Email**: support@inferno.ai

## 📄 License

This project is licensed under the MIT OR Apache-2.0 license. See the [LICENSE](../LICENSE) file for details.