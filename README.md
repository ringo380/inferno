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
- **GPU Acceleration**: Metal (Apple Silicon, 13x speedup ✅), NVIDIA, AMD, Intel support
- **Smart Caching**: Remember previous responses for instant results
- **Batch Processing**: Handle thousands of requests efficiently
- **Load Balancing**: Distribute work across multiple models/GPUs

### **🔧 Developer Friendly**
- **OpenAI-Compatible API**: Drop-in replacement for ChatGPT API
- **REST & WebSocket**: Standard APIs plus real-time streaming
- **Multiple Languages**: Python, JavaScript, Rust, cURL examples
- **Docker Ready**: One-command deployment
- **Smart CLI**: Typo detection, helpful error messages, setup guidance

## 📦 Installation

Choose your preferred installation method:

### 🍎 macOS

#### Desktop App (NEW in v0.5.0) - Recommended for macOS users
**Native macOS application with Metal GPU capabilities detection, optimized for Apple Silicon (M1/M2/M3/M4)**

1. Visit [Releases](https://github.com/ringo380/inferno/releases/latest)
2. Download `Inferno.dmg` (universal binary for Intel & Apple Silicon)
3. Open the DMG and drag Inferno to Applications
4. Launch from Applications folder

**Features:**
- 🎨 Native macOS UI with vibrancy effects
- 🔔 System tray integration with live metrics
- ⚡ **Metal GPU acceleration with 13x speedup** (Phases 2.1-2.3 ✅)
- 🍎 Apple Silicon optimization (M1/M2/M3/M4 detection)
- 🔄 Automatic model downloads and updates
- 📊 Real-time performance monitoring with GPU metrics
- 🔐 Built-in security and API key management
- 🧠 Neural Engine detection for AI workloads

**Build from source:**
```bash
# Clone and build
git clone https://github.com/ringo380/inferno.git
cd inferno
./scripts/build-desktop.sh --release --universal

# Development mode with hot reload
cd dashboard && npm run tauri dev
```

#### CLI Tools (for automation and scripting)

**Homebrew** (coming soon)
```bash
# Add tap and install (once tap is available)
brew tap ringo380/tap
brew install inferno

# Or directly
brew install ringo380/tap/inferno

# Start as service
brew services start inferno
```

> **Note**: The Homebrew tap is being set up. Until then, use the Quick Install Script or build from source.

**Quick Install Script**
```bash
curl -sSL https://github.com/ringo380/inferno/releases/latest/download/install-inferno.sh | bash
```

### 🐳 Docker

#### GitHub Container Registry
```bash
# Pull the latest image
docker pull ghcr.io/ringo380/inferno:latest

# Run with GPU support
docker run --gpus all -p 8080:8080 ghcr.io/ringo380/inferno:latest

# With custom models directory
docker run -v /path/to/models:/home/inferno/.inferno/models \
           -p 8080:8080 ghcr.io/ringo380/inferno:latest
```

#### Docker Compose
```yaml
version: '3.8'
services:
  inferno:
    image: ghcr.io/ringo380/inferno:latest
    ports:
      - "8080:8080"
    volumes:
      - ./models:/home/inferno/.inferno/models
      - ./config:/home/inferno/.inferno/config
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: all
              capabilities: [gpu]
```

### 📦 Package Managers

#### Cargo (Rust)
```bash
# From crates.io (once published)
cargo install inferno-ai

# From source (recommended until crates.io publish)
git clone https://github.com/ringo380/inferno.git
cd inferno && cargo install --path .
```

#### NPM (Desktop App)
```bash
# From GitHub Packages
npm install @ringo380/inferno-desktop

# From npm registry
npm install inferno-desktop
```

### 🐧 Linux

#### Binary Download
```bash
# Download for your architecture
wget https://github.com/ringo380/inferno/releases/latest/download/inferno-linux-x86_64
# or
wget https://github.com/ringo380/inferno/releases/latest/download/inferno-linux-aarch64

# Make executable and move to PATH
chmod +x inferno-linux-*
sudo mv inferno-linux-* /usr/local/bin/inferno
```

### 🪟 Windows

#### Binary Download
1. Download `inferno-windows-x86_64.exe` from [Releases](https://github.com/ringo380/inferno/releases/latest)
2. Add to your PATH or run directly

#### Via Cargo
```powershell
cargo install inferno-ai
```

### 🔨 Build from Source

```bash
# Clone the repository
git clone https://github.com/ringo380/inferno.git
cd inferno

# Build release binary
cargo build --release

# Install globally (optional)
cargo install --path .

# Build desktop app (optional)
cd desktop-app && npm install && npm run build
```

### ⬆️ Upgrading

#### Automatic Updates (Built-in)
```bash
inferno upgrade check     # Check for updates
inferno upgrade install   # Install latest version
```

#### Package Managers
```bash
# Homebrew
brew upgrade inferno

# Docker
docker pull ghcr.io/ringo380/inferno:latest

# Cargo
cargo install inferno-ai --force

# NPM
npm update @ringo380/inferno-desktop
```

**Note**: DMG and installer packages automatically detect existing installations and preserve your settings during upgrade.

### 🔐 Verify Installation

```bash
# Check version
inferno --version

# Verify GPU support
inferno gpu status

# Run health check
inferno doctor
```

## 🚀 Quick Start

```bash
# List available models
inferno models list

# Run inference
inferno run --model MODEL_NAME --prompt "Your prompt here"

# Start HTTP API server
inferno serve

# Launch terminal UI
inferno tui

# Launch desktop app (if installed from DMG)
open /Applications/Inferno.app
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