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
- **Smart CLI**: Typo detection, helpful error messages, setup guidance

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

### **📦 Package Manager - Install Models Like Software**

Inferno includes a comprehensive package manager that makes AI model management as easy as `apt` or `yum`:

```bash
# Install popular models with one command
inferno install microsoft/DialoGPT-medium
inferno install google/flan-t5-base
inferno install meta-llama/Llama-2-7b-chat-hf

# Search across multiple repositories
inferno search "language model"
inferno search "code generation" --repo huggingface

# List and manage installed models
inferno list
inferno remove old-model
inferno package upgrade  # Update all models

# Repository management
inferno repo list
inferno repo add company-models https://models.company.com
```

**Supported Repositories (Pre-configured):**
- **🤗 Hugging Face**: 500K+ models (LLMs, vision, audio)
- **🦙 Ollama**: Optimized models for local inference
- **📊 ONNX Model Zoo**: Official computer vision and NLP models
- **🔥 PyTorch Hub**: Research and production PyTorch models
- **🧠 TensorFlow Hub**: Pre-trained TensorFlow models

### **🎯 Zero-to-AI in 30 Seconds**

```bash
# Install a model (handles everything automatically)
inferno install microsoft/DialoGPT-medium

# Start chatting immediately
inferno run --model DialoGPT-medium --prompt "Hello! How are you?"

# Or start the server for API access
inferno serve --model DialoGPT-medium
```

**That's it!** You now have a private AI assistant running locally.

### **🤖 Intelligent CLI Experience**

Inferno's CLI is designed to be helpful and user-friendly:

```bash
# Typo? No problem - get helpful suggestions
$ inferno instal gpt2
💡 Did you mean 'install'?

# Confused? Get context-aware help
$ inferno package
💡 Package management commands:
   • inferno package install <model>
   • inferno package search <query>
   • inferno package list

# Errors are actionable
$ inferno install nonexistent-model
❌ Package 'nonexistent-model' not found
💡 Try these alternatives:
   • inferno search nonexistent-model
   • inferno search "language model" --repo huggingface
```

## 💡 Real-World Use Cases

### **For Individuals**
```bash
# Install and use a coding assistant
inferno install microsoft/codebert-base
inferno run --model codebert-base --prompt "Write a Python function to sort a list"

# Document summarization (keeping data private)
inferno install facebook/bart-large-cnn
inferno run --model bart-large-cnn --input documents/ --batch

# Creative writing helper
inferno install mistralai/Mistral-7B-v0.1
inferno run --model Mistral-7B-v0.1 --prompt "Write a story about..." --stream
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
# Set up enterprise model repository
inferno repo add enterprise https://models.company.com --priority 1 --verify
inferno install company/custom-llm-v2 --auto-update

# Deploy with monitoring and security
inferno serve --model custom-llm-v2 --auth --metrics --audit-logs

# Batch process customer data (stays private)
inferno batch --input customer_queries.jsonl --output responses.jsonl
```

## ✨ Key Features

### **🧠 AI Backends**
- ✅ **Real GGUF Support**: Full llama.cpp integration, not mock implementations
- ✅ **Real ONNX Support**: Production ONNX Runtime with GPU acceleration
- ✅ **Model Conversion**: Real-time format conversion with optimization
- ✅ **Quantization**: Q4_0, Q4_1, Q5_0, Q5_1, Q8_0, F16, F32 support

### **📦 Package Management**
- ✅ **apt/yum-style Commands**: `install`, `remove`, `search`, `list`, `upgrade`
- ✅ **Multi-Repository Support**: HuggingFace, Ollama, ONNX, PyTorch Hub, TensorFlow Hub
- ✅ **Dependency Resolution**: Automatic dependency handling and conflict resolution
- ✅ **Repository Management**: Add custom repositories, priority system, authentication
- ✅ **Smart CLI**: Typo detection, helpful errors, setup guidance
- ✅ **Package Database**: Track installations, usage, auto-updates

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

### **Package Management (Recommended)**
```bash
# Install models like software packages
inferno install microsoft/DialoGPT-medium     # Install from HuggingFace
inferno install ollama/llama2:7b              # Install from Ollama
inferno search "language model" --limit 10    # Search across repositories
inferno list --detailed                       # List installed models
inferno remove old-model                      # Remove models
inferno package upgrade                       # Update all models

# Repository management
inferno repo list                             # Show configured repositories
inferno repo add custom https://models.co     # Add custom repository
inferno repo update --force                   # Refresh repository metadata
```

### **Legacy Model Management**
```bash
# Direct model management (for advanced users)
inferno models list                           # See available models
inferno models download llama-2-7b            # Download from Hugging Face
inferno models info llama-2-7b                # Show model details
inferno models convert input.pt output.gguf   # Convert between formats
```

### **Running Inference**
```bash
inferno run --model DialoGPT-medium --prompt "Hello AI!"
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