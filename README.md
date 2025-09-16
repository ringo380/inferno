# 🔥 Inferno - Your Personal AI Infrastructure

> **Run any AI model locally with enterprise-grade performance and privacy**

[![Build Status](https://github.com/ringo380/inferno/workflows/CI/badge.svg)](https://github.com/ringo380/inferno/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://rustlang.org)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://hub.docker.com/r/inferno/inferno)

Inferno is a **production-ready AI inference server** that runs entirely on your hardware. Think of it as your private ChatGPT that works offline, supports any model format, and gives you complete control over your AI infrastructure.

## 🎯 Why Inferno?

### **🔒 Privacy First**
- **100% Local**: All processing happens on your hardware
- **No Cloud Dependency**: Works completely offline
- **Your Data Stays Yours**: Zero telemetry or external data transmission

### **🚀 Universal Model Support**
- **GGUF Models**: Native support for Llama, Mistral, CodeLlama, and more
- **ONNX Models**: Run models from PyTorch, TensorFlow, scikit-learn
- **Format Conversion**: Convert between GGUF ↔ ONNX ↔ PyTorch ↔ SafeTensors
- **Auto-Optimization**: Automatic quantization and hardware optimization

### **⚡ Enterprise Performance**
- **GPU Acceleration**: NVIDIA, AMD, Apple Silicon, Intel support
- **Smart Caching**: Remember previous responses for instant results
- **Batch Processing**: Handle thousands of requests efficiently
- **Load Balancing**: Distribute work across multiple models/GPUs

### **🔧 Developer Friendly**
- **OpenAI-Compatible API**: Drop-in replacement for ChatGPT API
- **REST & WebSocket**: Standard APIs plus real-time streaming
- **Multiple Languages**: Python, JavaScript, Rust, cURL examples
- **Docker Ready**: One-command deployment

## 🚀 Quick Start

### Installation

```bash
# Option 1: Docker (Recommended)
docker run -p 8080:8080 inferno:latest serve

# Option 2: Build from source
git clone https://github.com/ringo380/inferno.git
cd inferno
cargo build --release
./target/release/inferno serve
```

### Your First AI Request

```bash
# Start Inferno
inferno serve

# Download a model (one-time setup)
inferno models download llama-2-7b-chat

# Ask your AI a question
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-2-7b-chat",
    "messages": [{"role": "user", "content": "What is the capital of France?"}],
    "max_tokens": 100
  }'
```

**That's it!** You now have a private AI assistant running locally.

## 💡 Real-World Use Cases

### **For Individuals**
```bash
# Private coding assistant
inferno run --model codellama-13b --prompt "Write a Python function to sort a list"

# Document summarization (keeping data private)
inferno run --model llama-2-7b --input documents/ --batch

# Creative writing helper
inferno run --model mistral-7b --prompt "Write a story about..." --stream
```

### **For Developers**
```python
# OpenAI-compatible client
from openai import OpenAI

# Point to your local Inferno instance
client = OpenAI(base_url="http://localhost:8080/v1", api_key="not-needed")

response = client.chat.completions.create(
    model="llama-2-7b",
    messages=[{"role": "user", "content": "Debug this code..."}]
)
```

### **For Businesses**
```bash
# Convert proprietary model to GGUF for deployment
inferno convert your-pytorch-model.bin --output optimized.gguf --quantization q4_0

# Deploy with monitoring and security
inferno serve --auth --metrics --audit-logs

# Batch process customer data (stays private)
inferno batch --input customer_queries.jsonl --output responses.jsonl
```

## ✨ Key Features

### **🧠 AI Backends**
- ✅ **Real GGUF Support**: Full llama.cpp integration, not mock implementations
- ✅ **Real ONNX Support**: Production ONNX Runtime with GPU acceleration
- ✅ **Model Conversion**: Real-time format conversion with optimization
- ✅ **Quantization**: Q4_0, Q4_1, Q5_0, Q5_1, Q8_0, F16, F32 support

### **🏢 Enterprise Features**
- ✅ **Authentication**: JWT tokens, API keys, role-based access
- ✅ **Monitoring**: Prometheus metrics, Grafana dashboards, OpenTelemetry
- ✅ **Audit Logging**: Encrypted logs with multi-channel alerting
- ✅ **Batch Processing**: Cron scheduling, retry logic, job dependencies
- ✅ **Caching**: Multi-tier caching with compression and persistence
- ✅ **Load Balancing**: Distribute inference across multiple backends

### **🔌 APIs & Integration**
- ✅ **OpenAI Compatible**: Use existing ChatGPT client libraries
- ✅ **REST API**: Standard HTTP endpoints for all operations
- ✅ **WebSocket**: Real-time streaming and bidirectional communication
- ✅ **CLI Interface**: Full command-line management
- ✅ **Web Dashboard**: Browser-based monitoring and management

## 🛠️ Common Commands

```bash
# Model management
inferno models list                           # See available models
inferno models download llama-2-7b            # Download from Hugging Face
inferno models info llama-2-7b                # Show model details
inferno models convert input.pt output.gguf   # Convert between formats

# Running inference
inferno run --model llama-2-7b --prompt "Hello AI!"
inferno run --model llama-2-7b --input file.txt --output response.txt
inferno run --model llama-2-7b --batch --input batch.jsonl

# Server operations
inferno serve                                 # Start HTTP server
inferno serve --bind 0.0.0.0:8080           # Custom address
inferno serve --auth --metrics              # Production mode

# Advanced features
inferno cache warm --model llama-2-7b        # Pre-load for speed
inferno batch-queue create --schedule "0 2 * * *"  # Cron jobs
inferno security init                        # Set up authentication
inferno observability start                  # Monitoring stack
```

## 🏗️ Architecture

Inferno is built with a modular, production-ready architecture:

```
   ┌─── Client Libraries ────┐    ┌─── Security & Auth ────┐
   │ • Python SDK            │    │ • JWT Authentication   │
   │ • JavaScript/TypeScript │    │ • API Key Management   │
   │ • REST API              │    │ • Rate Limiting        │
   │ • WebSocket Streaming   │    │ • Audit Logging        │
   └─────────────────────────┘    └─────────────────────────┘
                  │                              │
                  └──────────┬───────────────────┘
                             │
            ┌─────────── Core Engine ───────────┐
            │ • Model Manager & Conversion      │
            │ • Multi-tier Caching System      │
            │ • Batch Queue & Scheduling       │
            │ • Load Balancer & Health Checks  │
            └───────────────────────────────────┘
                             │
     ┌───────────────────────┼───────────────────────┐
     │                       │                       │
┌─── AI Backends ───┐  ┌─── Storage ───┐  ┌─── Monitoring ───┐
│ • GGUF (llama.cpp)│  │ • Cache Store │  │ • Prometheus      │
│ • ONNX Runtime   │  │ • Audit Logs  │  │ • Grafana         │
│ • GPU Acceleration│  │ • Model Store │  │ • OpenTelemetry   │
│ • Quantization   │  │ • Compression │  │ • Health Checks   │
└──────────────────┘  └───────────────┘  └───────────────────┘
```

## 🐳 Deployment Options

### **Docker (Recommended)**
```bash
# Basic deployment
docker run -p 8080:8080 -v ./models:/data/models inferno:latest

# Production with GPU
docker run --gpus all -p 8080:8080 \
  -v ./models:/data/models \
  -v ./config:/etc/inferno \
  inferno:latest serve --config /etc/inferno/config.toml

# Full stack with monitoring
docker-compose up -d  # Includes Prometheus, Grafana, Redis
```

### **Kubernetes**
```bash
# Deploy to Kubernetes
kubectl apply -f deploy/kubernetes/

# With GPU support
kubectl apply -f deploy/kubernetes/gpu/
```

### **Binary Installation**
```bash
# Download release
wget https://github.com/ringo380/inferno/releases/latest/inferno-linux-x86_64.tar.gz
tar xzf inferno-linux-x86_64.tar.gz
./inferno serve
```

## 🔧 Configuration

Create `inferno.toml`:

```toml
# Basic settings
models_dir = "/data/models"
log_level = "info"

[server]
bind_address = "0.0.0.0"
port = 8080

[backend_config]
gpu_enabled = true
context_size = 4096
batch_size = 64

[cache]
enabled = true
compression = "zstd"
max_size_gb = 10

[auth]
enabled = true
jwt_secret = "your-secret-key"

[observability]
prometheus_enabled = true
metrics_port = 9090
```

## 🤝 Contributing

We welcome contributions! Inferno is built by developers, for developers.

### **Quick Contributing Guide**
1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Commit** your changes: `git commit -m 'Add amazing feature'`
4. **Push** to the branch: `git push origin feature/amazing-feature`
5. **Open** a Pull Request

### **Development Setup**
```bash
git clone https://github.com/ringo380/inferno.git
cd inferno
cargo build
cargo test
./verify.sh  # Run full test suite
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## 🌟 Community

- **🐛 Issues**: [Report bugs](https://github.com/ringo380/inferno/issues)
- **💡 Discussions**: [Feature requests and community help](https://github.com/ringo380/inferno/discussions)
- **📚 Wiki**: [Full documentation](https://github.com/ringo380/inferno/wiki)
- **🏢 Enterprise**: Contact maintainer for specialized installation assistance (information and pricing available)

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

## 🚀 What's Next?

- **⭐ Star** this repo if you find it useful
- **🔄 Follow** for updates on new features
- **💬 Join** GitHub Discussions to connect with other users
- **🐛 Report** issues to help us improve
- **🤝 Contribute** code, docs, or ideas

---

<div align="center">

**🔥 Ready to take control of your AI infrastructure? 🔥**

[**Get Started →**](#-quick-start) • [**Community →**](https://github.com/ringo380/inferno/discussions) • [**Wiki →**](https://github.com/ringo380/inferno/wiki)

*Built with ❤️ by the open source community*

</div>