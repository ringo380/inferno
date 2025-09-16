# 🔥 Inferno AI/ML Model Runner

**High-performance offline AI/ML inference server for GGUF and ONNX models**

[![Build Status](https://github.com/inferno-ai/inferno/workflows/CI/badge.svg)](https://github.com/inferno-ai/inferno/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://rustlang.org)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://hub.docker.com/r/inferno/inferno)

## ✨ Features

🚀 **High Performance**
- Multi-threaded async inference engine
- GPU acceleration (CUDA, Metal, Vulkan)
- Optimized memory management
- Batch processing support

🔒 **Enterprise Security**
- JWT and API key authentication
- Role-based access control
- Rate limiting and IP filtering
- Comprehensive audit logging

📊 **Production Observability**
- Prometheus metrics export
- OpenTelemetry distributed tracing
- Grafana dashboard integration
- Real-time health monitoring

🌐 **Multiple APIs**
- RESTful HTTP API
- OpenAI-compatible endpoints
- WebSocket real-time streaming
- Comprehensive CLI interface

🔧 **Advanced Features**
- A/B testing and canary deployments
- Distributed inference clusters
- Response caching and deduplication
- Model versioning and rollbacks

## 🚀 Quick Start

### Installation

```bash
# Build from source
git clone https://github.com/inferno-ai/inferno.git
cd inferno
cargo build --release

# Or use Docker
docker run -p 8080:8080 inferno:latest serve
```

### Basic Usage

```bash
# Start the server
inferno serve

# List available models
curl http://localhost:8080/models

# Run inference
curl -X POST http://localhost:8080/inference \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-2-7b",
    "prompt": "What is artificial intelligence?",
    "max_tokens": 100
  }'
```

[📖 **Full Getting Started Guide**](GETTING_STARTED.md)

## 🛠️ CLI Commands

The Inferno CLI provides comprehensive management capabilities:

### Model Management
```bash
inferno models list                    # List available models
inferno models load llama-2-7b         # Load model into memory
inferno models info llama-2-7b         # Show model details
inferno validate model.gguf            # Validate model file
```

### Inference Operations
```bash
inferno run --model llama-2-7b --prompt "Hello"  # Single inference
inferno batch --input prompts.txt                # Batch processing
inferno streaming interactive                     # Interactive streaming
```

### Server Operations
```bash
inferno serve                          # Start HTTP server
inferno serve --bind 0.0.0.0:8080     # Custom bind address
```

### Security & Observability
```bash
inferno security init                  # Initialize security
inferno observability init --prometheus --grafana
inferno observability metrics serve    # Start metrics server
```

### Advanced Features
```bash
inferno distributed coordinator start  # Start coordinator
inferno ab-test create --name test1    # A/B testing
inferno cache warm --model llama-2-7b  # Cache warm-up
```

## 📚 API Documentation

### REST API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/models` | GET | List available models |
| `/models/{id}/load` | POST | Load model into memory |
| `/inference` | POST | Run inference |
| `/inference/stream` | POST | Streaming inference |
| `/batch` | POST | Batch processing |
| `/embeddings` | POST | Generate embeddings |

### OpenAI-Compatible API

| Endpoint | Description |
|----------|-------------|
| `/v1/chat/completions` | Chat completions |
| `/v1/completions` | Text completions |
| `/v1/models` | List models |
| `/v1/embeddings` | Generate embeddings |

[📖 **Complete API Reference**](API.md)

## 💻 Examples

### Python Client

```python
from inferno_client import InfernoClient

client = InfernoClient("http://localhost:8080", api_key="your_key")

# Simple inference
response = client.inference("llama-2-7b", "What is AI?", max_tokens=100)
print(response)

# Streaming inference
for token in client.stream_inference("llama-2-7b", "Tell a story"):
    print(token, end="", flush=True)
```

### JavaScript/TypeScript

```typescript
import { InfernoClient } from '@inferno/client';

const client = new InfernoClient('http://localhost:8080', 'your_key');

// Async inference
const response = await client.inference('llama-2-7b', 'Hello world');
console.log(response);

// WebSocket streaming
const wsClient = new InfernoWebSocketClient();
await wsClient.connect();
wsClient.sendInference('llama-2-7b', 'Tell me a joke');
```

[🔗 **More Examples**](examples/)

## 🐳 Deployment

### Docker Compose (Recommended)

```bash
# Clone repository
git clone https://github.com/inferno-ai/inferno.git
cd inferno/examples

# Start full stack
docker-compose up -d
```

This deploys:
- **Inferno**: Main inference server
- **Prometheus**: Metrics collection
- **Grafana**: Visualization dashboards
- **Jaeger**: Distributed tracing
- **Redis**: Caching layer
- **Nginx**: Load balancer

### Configuration

```toml
# Basic settings
models_dir = "/data/models"
cache_dir = "/data/cache"
log_level = "info"

# Server configuration
[server]
bind_address = "0.0.0.0"
port = 8080
max_concurrent_requests = 100

# Security configuration
[auth_security]
auth_enabled = true
rate_limiting_enabled = true
max_requests_per_minute = 1000

# Observability configuration
[observability]
prometheus_enabled = true
otel_enabled = true
grafana_enabled = true
```

## 🏗️ Architecture

Inferno is built with a modular, trait-based architecture designed for scalability and extensibility:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client APIs   │    │   Observability │    │    Security     │
│                 │    │                 │    │                 │
│ • REST API      │    │ • Prometheus    │    │ • JWT Auth      │
│ • OpenAI API    │    │ • OpenTelemetry │    │ • Rate Limiting │
│ • WebSocket     │    │ • Grafana       │    │ • Access Control│
│ • CLI           │    │ • Health Checks │    │ • Audit Logs    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │  Core Engine    │
                    │                 │
                    │ • Model Manager │
                    │ • Cache System  │
                    │ • Task Queue    │
                    │ • Load Balancer │
                    └─────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  ML Backends    │    │  Storage Layer  │    │  Distributed    │
│                 │    │                 │    │                 │
│ • GGUF Support  │    │ • Model Store   │    │ • Worker Pools  │
│ • ONNX Support  │    │ • Response Cache│    │ • Load Balancing│
│ • GPU Offload   │    │ • Metrics Store │    │ • Auto Scaling  │
│ • Quantization  │    │ • Config Store  │    │ • Fault Tolerance│
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🔒 Security

### Authentication & Authorization

```bash
# Initialize security
inferno security init

# Create users
inferno security user create admin --role admin
inferno security user create john --role user

# Generate API keys
inferno security api-key create --user admin --name production

# Configure permissions
inferno security user update john --permissions read_models,run_inference
```

### Rate Limiting & Access Control

```bash
# Configure rate limits
inferno security rate-limit set --requests-per-minute 100 --user john

# IP access control
inferno security ip allow 192.168.1.0/24
inferno security ip block 10.0.0.0/8

# View audit logs
inferno security audit logs --limit 100
```

## 📊 Monitoring

### Prometheus Metrics

```bash
# Start metrics server
inferno observability metrics serve

# View metrics
curl http://localhost:9090/metrics
```

Key metrics:
- `inferno_inference_requests_total`
- `inferno_inference_duration_seconds`
- `inferno_models_loaded`
- `inferno_memory_usage_bytes`

### Grafana Dashboards

```bash
# Initialize observability stack
inferno observability init --prometheus --grafana

# Access Grafana
open http://localhost:3000
```

### OpenTelemetry Tracing

```bash
# Enable tracing
inferno observability tracing collect

# View traces
open http://localhost:16686
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone repository
git clone https://github.com/inferno-ai/inferno.git
cd inferno

# Install dependencies
cargo build

# Run tests
cargo test

# Run verification
./verify.sh
```

## 📄 License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🔗 Links

- **Documentation**: [Full API Docs](API.md) • [Getting Started](GETTING_STARTED.md) • [Examples](examples/)
- **GitHub**: [github.com/inferno-ai/inferno](https://github.com/inferno-ai/inferno)
- **Discord**: [Join our community](https://discord.gg/inferno)
- **Docker Hub**: [hub.docker.com/r/inferno/inferno](https://hub.docker.com/r/inferno/inferno)

---

<div align="center">
  <strong>🔥 Built with Rust • Powered by AI • Ready for Production 🔥</strong>
</div>