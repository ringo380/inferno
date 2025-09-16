# 🔥 Inferno AI/ML Model Runner

**High-performance offline AI/ML inference server for GGUF and ONNX models**

[![Build Status](https://github.com/ringo380/inferno/workflows/CI/badge.svg)](https://github.com/ringo380/inferno/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://rustlang.org)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://hub.docker.com/r/inferno/inferno)

## ✨ Features

🚀 **High Performance**
- Real GGUF backend with llama.cpp integration
- Full ONNX Runtime support with GPU acceleration
- Multi-format model conversion (GGUF ↔ ONNX ↔ PyTorch ↔ SafeTensors)
- Advanced caching with disk persistence and compression
- Thread-safe backend cloning architecture
- Optimized memory management and batch processing

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
- Real-time model format conversion with optimization
- Advanced response caching with Gzip/Zstd compression
- Complete audit system with encryption and alerting
- Batch queue with cron scheduling and retry logic
- A/B testing and canary deployments
- Distributed inference clusters with load balancing
- Hash-based deduplication (Blake3, xxHash)
- Model versioning and automated rollbacks

## 🚀 Quick Start

### Installation

#### Prerequisites

- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)
- **llama.cpp** - Required for GGUF model support
- **ONNX Runtime** - Automatically handled via the `ort` crate
- **OpenSSL** - Required for TLS support

#### Platform-specific GPU Support

**Linux:**
```bash
# CUDA support (NVIDIA GPUs)
sudo apt install nvidia-cuda-toolkit

# Vulkan support
sudo apt install vulkan-tools libvulkan-dev
```

**macOS:**
```bash
# Metal support is built-in on macOS 10.13+
# No additional installation required
```

**Windows:**
```bash
# DirectML support (Windows 10+)
# Automatically available on Windows 10 1903+
```

#### Build from Source

```bash
# Clone repository
git clone https://github.com/ringo380/inferno.git
cd inferno

# Build with all features
cargo build --release --all-features

# Or build specific features
cargo build --release --features "gguf,onnx,gpu"
```

#### Docker Installation

```bash
# Pull latest image
docker pull inferno:latest

# Run with GPU support (Linux + NVIDIA)
docker run --gpus all -p 8080:8080 inferno:latest serve

# Run CPU-only
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
inferno convert model input.gguf output.onnx --format onnx --optimization balanced
inferno convert model input.pt output.gguf --format gguf --quantization q4_0
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
inferno cache persist --compress gzip  # Enable persistent caching
inferno audit enable --encryption      # Enable encrypted audit logs
inferno batch-queue create --schedule "0 2 * * *" # Cron-based batch jobs
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

### Dashboard API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/dashboard/stats` | GET | System statistics |
| `/dashboard/models` | GET | Model status and metrics |
| `/dashboard/health` | GET | Health check with details |
| `/dashboard/config` | GET | Current configuration |
| `/dashboard/logs` | GET | Recent log entries |
| `/dashboard/metrics` | GET | Performance metrics |
| `/dashboard/cache` | GET | Cache statistics |
| `/dashboard/jobs` | GET | Batch job status |
| `/dashboard/audit` | GET | Audit log entries |
| `/dashboard/workers` | GET | Distributed worker status |
| `/dashboard/resources` | GET | System resource usage |
| `/dashboard/errors` | GET | Recent error reports |
| `/dashboard/security` | GET | Security status |
| `/dashboard/alerts` | GET | System alerts |

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
git clone https://github.com/ringo380/inferno.git
cd inferno/examples

# Start full stack
docker-compose up -d
```

This deploys:
- **Inferno**: Main inference server with real GGUF/ONNX support
- **Prometheus**: Metrics collection and alerting
- **Grafana**: Visualization dashboards with custom panels
- **Jaeger**: Distributed tracing and performance monitoring
- **Redis**: Advanced caching layer with persistence
- **Nginx**: Load balancer with SSL termination
- **Database**: Audit log storage with compression
- **MinIO**: Model storage and versioning

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

# Backend configuration
[backend_config]
gpu_enabled = true
gpu_device = "auto"  # or specific device ID
cpu_threads = 8
context_size = 4096
batch_size = 64
memory_map = true

# Cache configuration
[cache]
type = "persistent"  # memory, disk, persistent
compression = "gzip"  # none, gzip, zstd
max_size_gb = 10
ttl_hours = 24

# Model conversion settings
[conversion]
default_optimization = "balanced"  # fast, balanced, aggressive
quantization_enabled = true
default_precision = "fp16"

# Audit system
[audit]
enabled = true
compression = true
encryption = true
alert_channels = ["email", "slack", "webhook"]

# Batch processing
[batch_queue]
max_concurrent_jobs = 5
retry_attempts = 3
default_schedule = "0 2 * * *"  # Daily at 2 AM

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
│ • WebSocket     │    │ • Grafana       │    │ • RBAC          │
│ • CLI + TUI     │    │ • Health Checks │    │ • Encrypted Logs│
│ • Dashboard     │    │ • Real-time Logs│    │ • Multi-channel │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │  Core Engine    │
                    │                 │
                    │ • Model Manager │
                    │ • Cache System  │
                    │ • Batch Queue   │
                    │ • Load Balancer │
                    │ • Hash Functions│
                    └─────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  ML Backends    │    │  Storage Layer  │    │  Distributed    │
│                 │    │                 │    │                 │
│ • Real GGUF     │    │ • Persistent    │    │ • Worker Pools  │
│ • Real ONNX     │    │   Cache Store   │    │ • Load Balancing│
│ • GPU Accel     │    │ • Compressed    │    │ • Auto Scaling  │
│ • Quantization  │    │   Audit Logs    │    │ • Fault Tolerance│
│ • Conversion    │    │ • Model Versioning│  │ • Cron Scheduling│
│ • Thread Safety │    │ • Metrics Store │    │ • Retry Logic   │
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
git clone https://github.com/ringo380/inferno.git
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
- **GitHub**: [github.com/ringo380/inferno](https://github.com/ringo380/inferno)
- **Discord**: [Join our community](https://discord.gg/inferno)
- **Docker Hub**: [hub.docker.com/r/inferno/inferno](https://hub.docker.com/r/inferno/inferno)

---

<div align="center">
  <strong>🔥 Built with Rust • Powered by AI • Ready for Production 🔥</strong>
</div>