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

```bash
# Build from source
git clone https://github.com/ringo380/inferno.git
cd inferno
cargo build --release

# List available models
./target/release/inferno models list

# Run inference
./target/release/inferno run --model MODEL_NAME --prompt "Your prompt here"

# Start HTTP API server
./target/release/inferno serve

# Launch desktop app
npm run tauri dev
```

## ✨ Key Features

### **🧠 AI Backends**
- ✅ **Real GGUF Support**: Full llama.cpp integration
- ✅ **Real ONNX Support**: Production ONNX Runtime with GPU acceleration
- ✅ **Model Conversion**: Real-time format conversion with optimization
- ✅ **Quantization**: Q4_0, Q4_1, Q5_0, Q5_1, Q8_0, F16, F32 support

### **🏢 Enterprise Features**
- ✅ **Authentication**: JWT tokens, API keys, role-based access
- ✅ **Monitoring**: Prometheus metrics, OpenTelemetry tracing
- ✅ **Audit Logging**: Encrypted logs with multi-channel alerting
- ✅ **Batch Processing**: Cron scheduling, retry logic, job dependencies
- ✅ **Caching**: Multi-tier caching with compression and persistence
- ✅ **Load Balancing**: Distribute inference across multiple backends

### **🔌 APIs & Integration**
- ✅ **OpenAI Compatible**: Use existing ChatGPT client libraries
- ✅ **REST API**: Standard HTTP endpoints for all operations
- ✅ **WebSocket**: Real-time streaming and bidirectional communication
- ✅ **CLI Interface**: 40+ commands for all AI/ML operations
- ✅ **Desktop App**: Cross-platform Tauri application

## 🏗️ Architecture

Built with a modular, trait-based architecture supporting pluggable backends:

```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Library exports
├── config.rs         # Configuration management
├── backends/         # AI model execution backends
├── cli/              # 40+ CLI command modules
├── api/              # HTTP/WebSocket APIs
├── batch/            # Batch processing system
├── models/           # Model discovery and metadata
└── [Enterprise]      # Advanced production features
```

## 🔧 Configuration

Create `inferno.toml`:

```toml
# Basic settings
models_dir = "/path/to/models"
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
```

## 🛠️ Development

See [CLAUDE.md](CLAUDE.md) for comprehensive development documentation.

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy

# Full verification
./verify.sh
```

## 📄 License

Licensed under either of:
- Apache License, Version 2.0
- MIT License

---

<div align="center">

**🔥 Ready to take control of your AI infrastructure? 🔥**

*Built with ❤️ by the open source community*

</div>