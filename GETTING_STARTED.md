# Getting Started with Inferno

Welcome to Inferno! This guide will help you get up and running with the Inferno AI/ML inference server quickly.

## 📋 Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Model Management](#model-management)
- [API Usage](#api-usage)
- [Monitoring & Observability](#monitoring--observability)
- [Production Deployment](#production-deployment)
- [Next Steps](#next-steps)

## 🔧 Prerequisites

### System Requirements
- **OS**: Linux, macOS, or Windows
- **RAM**: Minimum 8GB (16GB+ recommended for larger models)
- **CPU**: Modern multi-core processor
- **Storage**: 10GB+ free space for models and cache
- **GPU** (Optional): NVIDIA CUDA, AMD ROCm, or Apple Metal for acceleration

### Software Dependencies
- **Rust**: 1.85+ (for building from source)
- **Docker**: 20.10+ (for containerized deployment)
- **curl**: For testing API endpoints

### Install Rust (if building from source)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

## 📦 Installation

### Option 1: Build from Source

```bash
# Clone the repository
git clone https://github.com/ringo380/inferno.git
cd inferno

# Build the project
cargo build --release

# The binary will be at target/release/inferno
```

### Option 2: Using Docker

```bash
# Pull the latest image
docker pull inferno:latest

# Or build from source
docker build -t inferno .
```

### Option 3: Pre-built Binaries (Coming Soon)

```bash
# Download for your platform
curl -L -o inferno https://github.com/ringo380/inferno/releases/latest/download/inferno-linux-x86_64
chmod +x inferno
sudo mv inferno /usr/local/bin/
```

## 🚀 Quick Start

### 1. Verify Installation

```bash
# Check version
inferno --version

# View help
inferno --help
```

### 2. Initialize Configuration

```bash
# Create default configuration
inferno config init

# View current configuration
inferno config show
```

### 3. Set Up Models Directory

```bash
# Create models directory
mkdir -p ~/inferno/models

# Set models directory
export INFERNO_MODELS_DIR=~/inferno/models

# Or set models_dir in your config file (~/.inferno.toml)
```

### 4. Download a Sample Model

```bash
# For this example, we'll create a mock model file
# In practice, you would download actual GGUF or ONNX models
mkdir -p ~/inferno/models
echo "GGUF" > ~/inferno/models/sample-model.gguf

# List available models
inferno models list
```

### 5. Start the Server

```bash
# Start with default settings
inferno serve

# Or with a custom bind address
inferno serve --bind 0.0.0.0:8080
```

### 6. Test the API

```bash
# Health check
curl http://localhost:8080/health

# List models
curl http://localhost:8080/models

# Run inference (with a mock model)
curl -X POST http://localhost:8080/inference \
  -H "Content-Type: application/json" \
  -d '{
    "model": "sample-model",
    "prompt": "Hello, world!",
    "max_tokens": 50
  }'
```

## ⚙️ Configuration

### Configuration File Locations

Inferno searches for configuration files in this order:
1. `.inferno.toml` (current directory)
2. `~/.inferno.toml` (home directory)
3. `~/.config/inferno/config.toml` (config directory)

### Basic Configuration

Create a configuration file:

```bash
cat > ~/.inferno.toml << EOF
# Basic settings
models_dir = "~/inferno/models"
cache_dir = "~/inferno/cache"
log_level = "info"
log_format = "pretty"

# Server settings
[server]
bind_address = "127.0.0.1"
port = 8080
max_concurrent_requests = 10
request_timeout_seconds = 300

# Backend settings
[backend_config]
context_size = 4096
batch_size = 512
gpu_layers = 32

# Security settings
[model_security]
verify_checksums = true
allowed_model_extensions = ["gguf", "onnx"]
max_model_size_gb = 50.0
sandbox_enabled = true

# Metrics settings
[metrics]
enabled = true
bind_address = "127.0.0.1"
port = 9090
path = "/metrics"
collection_interval_seconds = 10

# Observability settings
[observability]
prometheus_enabled = true
prometheus_endpoint = "/metrics"
otel_enabled = false
grafana_enabled = false
EOF
```

### Environment Variables

You can override any configuration with environment variables:

```bash
export INFERNO_LOG_LEVEL=debug
export INFERNO_SERVER__PORT=8081
export INFERNO_BACKEND_CONFIG__GPU_LAYERS=40
```

## 📦 Model Management

### Supported Formats
- **GGUF**: Quantized models (recommended for CPU inference)
- **ONNX**: Optimized neural networks

### Loading Models

Models are loaded on demand at inference time - there is no separate load step. GPU acceleration (Metal on Apple Silicon) is auto-detected and enabled by default; the number of GPU layers is controlled via the `[backend_config]` `gpu_layers` setting in your config file.

```bash
# Install a model from HuggingFace (repo ID or direct HTTPS URL)
inferno models install TheBloke/Llama-2-7B-GGUF

# Run inference - the model loads automatically
inferno run --model llama-2-7b --prompt "Hello, world!"
```

### Model Information

```bash
# List all models
inferno models list

# Show model details
inferno models info llama-2-7b

# Validate model file
inferno validate ~/inferno/models/llama-2-7b.gguf
```

## 🔌 API Usage

### Authentication (Optional)

If authentication is enabled, you'll need an API key:

```bash
# Generate API key
inferno security api-key create --user admin --name my-key

# Use in requests
curl -H "Authorization: Bearer YOUR_API_KEY" \
  http://localhost:8080/models
```

### Basic Inference

```bash
curl -X POST http://localhost:8080/inference \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-2-7b",
    "prompt": "Explain quantum computing in simple terms",
    "max_tokens": 200,
    "temperature": 0.7,
    "top_p": 0.9
  }'
```

### Streaming Inference

```bash
curl -X POST http://localhost:8080/inference/stream \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "model": "llama-2-7b",
    "prompt": "Write a short poem about AI",
    "max_tokens": 100,
    "stream": true
  }'
```

### OpenAI-Compatible API

```bash
# Chat completions
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-2-7b",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "What is machine learning?"}
    ]
  }'

# Text completions
curl -X POST http://localhost:8080/v1/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-2-7b",
    "prompt": "The future of AI is",
    "max_tokens": 50
  }'
```

### Batch Processing

```bash
# Submit batch job
curl -X POST http://localhost:8080/batch \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-2-7b",
    "requests": [
      {"id": "1", "prompt": "What is Python?"},
      {"id": "2", "prompt": "Explain machine learning"},
      {"id": "3", "prompt": "What is quantum computing?"}
    ],
    "max_tokens": 100
  }'

# Check batch status
curl http://localhost:8080/batch/BATCH_ID

# Get results
curl http://localhost:8080/batch/BATCH_ID/results
```

## 📊 Monitoring & Observability

### Enable Metrics

```bash
# Metrics are exposed by default while the server runs
inferno serve

# View Prometheus metrics
curl http://localhost:8080/metrics
```

### Using the Observability Stack

```bash
# Initialize observability
inferno observability init --prometheus --grafana

# Start metrics server
inferno observability metrics serve

# Show observability status
inferno observability status
```

### Key Metrics to Monitor

- `inferno_inference_requests_total`: Total inference requests
- `inferno_inference_duration_seconds`: Request duration
- `inferno_models_loaded`: Number of loaded models
- `inferno_memory_usage_bytes`: Memory consumption
- `inferno_errors_total`: Error count

## 🚀 Production Deployment

### Using Docker Compose

```bash
# Clone the examples
git clone https://github.com/ringo380/inferno.git
cd inferno/examples

# Start the full stack
docker-compose up -d

# View logs
docker-compose logs -f inferno

# Scale horizontally
docker-compose up -d --scale inferno=3
```

### Environment Setup

```bash
# Production environment variables
export INFERNO_LOG_LEVEL=warn
export INFERNO_LOG_FORMAT=json
export INFERNO_AUTH_ENABLED=true
export INFERNO_RATE_LIMITING_ENABLED=true
export INFERNO_MAX_REQUESTS_PER_MINUTE=1000
export INFERNO_PROMETHEUS_ENABLED=true
export INFERNO_OTEL_ENABLED=true
```

### Security Configuration

```bash
# Enable authentication
inferno security init

# Create admin user
inferno security user create admin --role admin

# Generate API keys
inferno security api-key create --user admin --name production

# Configure rate limiting
inferno security rate-limit set --requests-per-minute 1000
```

### Health Checks

```bash
# Built-in health check
curl http://localhost:8080/health

# Custom health check script
#!/bin/bash
response=$(curl -s http://localhost:8080/health)
if echo "$response" | grep -q "healthy"; then
    echo "Service is healthy"
    exit 0
else
    echo "Service is unhealthy"
    exit 1
fi
```

## 📚 Next Steps

### Explore Advanced Features

1. **Real-time Streaming**
   ```bash
   inferno streaming interactive --model llama-2-7b
   ```

2. **Distributed Inference**
   ```bash
   inferno distributed start
   ```

3. **GPU Acceleration**
   ```bash
   # GPU acceleration is auto-detected and enabled by default; list detected GPUs:
   inferno gpu list
   ```

4. **A/B Testing**
   ```bash
   inferno ab-test start --name model-comparison --control-model llama-2-7b --treatment-model llama-2-13b
   ```

### Integration Examples

- [Python Client](examples/python_client.py)
- [JavaScript Client](examples/javascript_client.js)
- [Go Client](examples/go_client.go)
- [Rust Client](examples/rust_client.rs)

### Documentation

- [Full API Reference](API.md)
- [Examples Directory](examples/)
- [Architecture Overview](ARCHITECTURE.md)

### Community

- **GitHub**: [Issues and discussions](https://github.com/ringo380/inferno)
- **GitHub Discussions**: [Community help](https://github.com/ringo380/inferno/discussions)
- **Documentation**: [Wiki](https://github.com/ringo380/inferno/wiki)

## 🔧 Troubleshooting

### Common Issues

1. **Model not loading**: Check file format and size limits
2. **Out of memory**: Reduce context size or enable GPU offloading
3. **Slow inference**: Enable GPU acceleration or increase batch size
4. **API errors**: Verify authentication and rate limits

### Getting Help

```bash
# Validate configuration
inferno config validate

# Test model file
inferno validate /path/to/model.gguf

# Show current configuration
inferno config show
```

Logs are written to stdout (or the file configured by your log settings). Set `INFERNO_LOG_LEVEL=debug` for more detail.

### Debug Mode

```bash
# Run with debug logging
INFERNO_LOG_LEVEL=debug inferno serve

# Enable trace logging
INFERNO_LOG_LEVEL=trace RUST_BACKTRACE=full inferno serve
```

---

🎉 **Congratulations!** You're now ready to use Inferno for AI/ML inference. Check out the [examples directory](examples/) for more detailed usage patterns and integration examples.