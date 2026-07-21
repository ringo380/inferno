# 🚀 Getting Started Guide

Complete setup and configuration guide for Inferno - from installation to your first production deployment.

## Overview

This comprehensive guide will walk you through:
- ✅ **Installation** on all major platforms
- ✅ **Configuration** for your specific needs
- ✅ **First model** installation and inference
- ✅ **API integration** with your applications
- ✅ **Performance tuning** for optimal results
- ✅ **Production setup** with monitoring and security

**Estimated Time**: 30-60 minutes
**Skill Level**: Beginner to Intermediate

## Prerequisites

### System Requirements

**Minimum Requirements:**
- **OS**: Linux, macOS, Windows 10+
- **CPU**: 4 cores, 2.0 GHz
- **RAM**: 8GB (16GB recommended)
- **Storage**: 20GB free space
- **Network**: Internet connection for model downloads

**Recommended Requirements:**
- **CPU**: 8+ cores, 3.0+ GHz
- **RAM**: 32GB+ (for large models)
- **GPU**: NVIDIA RTX 3060+ / AMD RX 6600+ / Apple M1+
- **Storage**: 100GB+ SSD
- **Network**: High-speed connection for faster downloads

### Software Dependencies

**Required:**
- Git (for source installation)
- Modern web browser (for dashboard)

**Optional but Recommended:**
- Docker and Docker Compose
- NVIDIA drivers (for GPU acceleration)
- Python 3.8+ (for SDK usage)

## Installation Methods

### Method 1: Docker (Recommended)

Docker provides the fastest and most reliable installation experience.

#### Basic Docker Setup

```bash
# Pull and run Inferno
docker run -p 8080:8080 ghcr.io/ringo380/inferno:latest serve

# With persistent storage
mkdir -p ./inferno-data/models ./inferno-data/cache
docker run -p 8080:8080 \
  -v ./inferno-data/models:/data/models \
  -v ./inferno-data/cache:/data/cache \
  ghcr.io/ringo380/inferno:latest serve
```

#### Production Docker Setup

```bash
# Create directory structure
mkdir -p inferno/{models,cache,config,logs}

# Create configuration file (Inferno discovers it at ~/.config/inferno/config.toml)
cat > inferno/config/config.toml << EOF
models_dir = "/data/models"
log_level = "info"

[server]
bind_address = "0.0.0.0"
port = 8080

[backend_config]
gpu_enabled = true
context_size = 4096

[cache]
enabled = true
max_size_gb = 10
EOF

# Run with full configuration
docker run -d \
  --name inferno \
  -p 8080:8080 \
  -v ./inferno/models:/data/models \
  -v ./inferno/cache:/data/cache \
  -v ./inferno/config:/root/.config/inferno \
  -v ./inferno/logs:/var/log/inferno \
  --restart unless-stopped \
  ghcr.io/ringo380/inferno:latest serve
```

#### GPU-Enabled Docker

```bash
# NVIDIA GPU support
docker run --gpus all -p 8080:8080 \
  -v ./inferno-data:/data \
  ghcr.io/ringo380/inferno:latest serve

# AMD GPU support (ROCm)
docker run --device=/dev/kfd --device=/dev/dri \
  -p 8080:8080 -v ./inferno-data:/data \
  ghcr.io/ringo380/inferno:rocm serve

# Apple Silicon (automatic GPU support)
docker run -p 8080:8080 \
  -v ./inferno-data:/data \
  ghcr.io/ringo380/inferno:latest serve
```

### Method 2: Binary Installation

Pre-built binaries for major platforms.

#### Linux

```bash
# Download latest release (replace v0.10.5 with current version)
# Check https://github.com/ringo380/inferno/releases for latest version
wget https://github.com/ringo380/inferno/releases/download/v0.10.5/inferno-linux-x86_64.tar.gz

# Extract and install
tar xzf inferno-linux-x86_64.tar.gz
sudo mv inferno /usr/local/bin/
sudo chmod +x /usr/local/bin/inferno

# Verify installation
inferno --version
```

#### macOS

```bash
# Check https://github.com/ringo380/inferno/releases for latest version
VERSION="v0.10.5"

# Intel Macs
curl -LO "https://github.com/ringo380/inferno/releases/download/${VERSION}/inferno-macos-x86_64.tar.gz"
tar xzf inferno-macos-x86_64.tar.gz

# Apple Silicon Macs
curl -LO "https://github.com/ringo380/inferno/releases/download/${VERSION}/inferno-macos-aarch64.tar.gz"
tar xzf inferno-macos-aarch64.tar.gz

# Install
sudo mv inferno /usr/local/bin/
sudo chmod +x /usr/local/bin/inferno

# Verify installation
inferno --version
```

#### Windows

```powershell
# Check https://github.com/ringo380/inferno/releases for latest version
$VERSION = "v0.10.5"

# Download from releases page
Invoke-WebRequest -Uri "https://github.com/ringo380/inferno/releases/download/$VERSION/inferno-windows-x86_64.exe.zip" -OutFile "inferno.zip"

# Extract
Expand-Archive -Path "inferno.zip" -DestinationPath "C:\Program Files\Inferno"

# Add to PATH
$env:PATH += ";C:\Program Files\Inferno"

# Verify installation
inferno --version
```

### Method 3: Build from Source

Build from source for customization or latest features.

#### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify Rust installation
rustc --version
cargo --version
```

#### Build Process

```bash
# Clone repository
git clone https://github.com/ringo380/inferno.git
cd inferno

# Build release version
cargo build --release

# Install binary
sudo cp target/release/inferno /usr/local/bin/

# Verify installation
inferno --version
```

#### Development Build

```bash
# Clone and build
git clone https://github.com/ringo380/inferno.git
cd inferno

# Development build (faster compilation)
cargo build

# Run directly
cargo run -- --help

# Run tests
cargo test

# Run with optimizations
cargo run --release -- serve
```

## Initial Configuration

### Configuration File Setup

Create your main configuration file:

```bash
# Create config directory
mkdir -p ~/.config/inferno

# Create configuration file
cat > ~/.config/inferno/config.toml << EOF
# Inferno Configuration File

# Basic settings
models_dir = "~/.local/share/inferno/models"
log_level = "info"
log_format = "pretty"

[server]
bind_address = "127.0.0.1"  # Use 0.0.0.0 for external access
port = 8080
workers = 4

[backend_config]
gpu_enabled = true
context_size = 4096
batch_size = 64

[cache]
enabled = true
max_size_gb = 10
compression = "zstd"
persist = true

[security]
auth_enabled = false  # Enable for production
rate_limit = 1000
cors_enabled = true

[observability]
prometheus_enabled = true
metrics_port = 9090
tracing_enabled = false
EOF
```

### Environment Variables

Set up environment variables for easy configuration:

```bash
# Add to ~/.bashrc or ~/.zshrc
export INFERNO_MODELS_DIR="$HOME/.local/share/inferno/models"
export INFERNO_LOG_LEVEL="info"
export INFERNO_GPU_ENABLED="true"

# For development
export INFERNO_LOG_LEVEL="debug"
export INFERNO_LOG_FORMAT="json"

# Reload shell configuration
source ~/.bashrc  # or source ~/.zshrc
```

### Directory Structure Setup

```bash
# Create necessary directories
mkdir -p ~/.local/share/inferno/{models,cache,logs}
mkdir -p ~/.config/inferno

# Set permissions
chmod 700 ~/.local/share/inferno
chmod 700 ~/.config/inferno
```

## First Model Installation

### Installing Models (Recommended)

`inferno models install` pulls models straight from a HuggingFace repository ID or a direct HTTPS URL:

```bash
# Install a conversational model
inferno models install microsoft/DialoGPT-medium

# Install a coding assistant
inferno models install microsoft/codebert-base

# Install an embedding model
inferno models install sentence-transformers/all-MiniLM-L6-v2

# Install a quantized GGUF build
inferno models install TheBloke/Llama-2-7B-Chat-GGUF

# List installed models
inferno models list
```

### Manual Model Management

For advanced users who prefer direct control:

```bash
# Install models manually
inferno models install microsoft/DialoGPT-medium
inferno models install huggingface/CodeBERTa-small-v1

# Convert between formats
inferno convert model model.pt model.gguf --format gguf
inferno convert model model.gguf model.onnx --format onnx

# Validate model files
inferno validate microsoft/DialoGPT-medium
```

### Popular Model Recommendations

**For Beginners:**
```bash
# Small, fast models for testing
inferno models install distilgpt2                    # 82MB, very fast
inferno models install microsoft/DialoGPT-small      # 117MB, good for chat
inferno models install sentence-transformers/all-MiniLM-L6-v2  # 90MB, embeddings
```

**For General Use:**
```bash
# Balanced performance and quality
inferno models install microsoft/DialoGPT-medium     # 345MB, better conversations
inferno models install microsoft/codebert-base       # 500MB, code understanding
inferno models install facebook/opt-1.3b             # 2.6GB, general language model
```

**For Production:**
```bash
# High-quality models (requires more resources)
inferno models install microsoft/DialoGPT-large      # 776MB, excellent conversations
inferno models install codellama/CodeLlama-7b-Instruct  # 3.8GB, advanced coding
inferno models install mistralai/Mistral-7B-v0.1     # 4.1GB, state-of-the-art
```

## First Inference

### Command Line Inference

Test your setup with simple command-line inference:

```bash
# Basic text generation
inferno run --model DialoGPT-medium --prompt "Hello! How are you today?"

# Interactive chat session (terminal UI)
inferno tui

# File-based input/output
echo "Explain quantum computing" > input.txt
inferno run --model DialoGPT-medium --input input.txt --output response.txt

# Streaming output (real-time)
inferno run --model DialoGPT-medium --prompt "Tell me a story" --stream
```

### Advanced Inference Options

```bash
# Control output length and creativity
inferno run --model DialoGPT-medium \
  --prompt "Write a poem about AI" \
  --max-tokens 200 \
  --temperature 0.8

# Batch processing
inferno batch --model DialoGPT-medium \
  --input questions.jsonl \
  --output answers.jsonl

# Code generation
inferno run --model codebert-base \
  --prompt "def fibonacci(n):" \
  --max-tokens 100 \
  --temperature 0.2
```

## API Server Setup

### Start the API Server

```bash
# Basic server
inferno serve

# Production server binding to all interfaces with a worker pool
inferno serve \
  --bind 0.0.0.0:8080 \
  --workers 8
```

### Test API Access

```bash
# Test server health
curl http://localhost:8080/health

# List available models
curl http://localhost:8080/v1/models

# Simple API request
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "DialoGPT-medium",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'
```

### Web Dashboard Access

Open your browser and navigate to: **http://localhost:8080/dashboard**

The dashboard provides:
- 📊 Real-time metrics and performance monitoring
- 🎛️ Model management (load, unload, switch models)
- 💬 Interactive chat interface for testing
- 🔧 Configuration management
- 📈 Usage analytics and trends

## Performance Optimization

### Hardware Optimization

#### GPU Acceleration

```bash
# Check GPU availability
inferno gpu list
```
GPU acceleration (Metal on Apple Silicon) is auto-detected and enabled by default. To force it in `~/.inferno.toml`:
```toml
[backend_config]
gpu_enabled = true
```
```bash
# Test GPU performance
inferno bench --model DialoGPT-medium

# Monitor performance in real time
inferno monitor watch
```

#### Memory Optimization

Configure memory settings in `~/.inferno.toml`:
```toml
[backend_config]
context_size = 2048
batch_size = 32
```
```bash
# Watch resource usage in real time
inferno monitor watch

# Check cache usage
inferno cache stats
```

### Model Optimization

#### Quantization

```bash
# Quantize a model to reduce size and speed up inference
inferno convert quantize --quantization q4-0 llama-7b.gguf llama-7b-q4.gguf

# Install pre-quantized models
inferno models install TheBloke/Llama-2-7B-Chat-GGUF

# Compare performance
inferno bench --model llama-7b
inferno bench --model llama-7b-q4
```

#### Caching Optimization

Enable caching in `~/.inferno.toml`:
```toml
[cache]
enabled = true
max_size_gb = 20
```
```bash
# Warm up the cache for a specific model
inferno cache warmup DialoGPT-medium

# Or warm up based on recent usage
inferno cache warmup --strategy usage-based

# Monitor cache performance
inferno cache stats
```

## Production Setup

### Security Configuration

```bash
# Initialize security system
inferno security init

# Create API keys
inferno security api-key generate --user admin --name production-app
inferno security api-key generate --user admin --name monitoring --permissions metrics
```
Configure authentication in `~/.inferno.toml`:
```toml
[security]
auth_enabled = true
rate_limit = 5000
```
```bash
# Enable audit logging
inferno audit configure --enable true
```

### Monitoring Setup

```bash
# Initialize the observability stack (Prometheus metrics)
inferno observability init --prometheus
```
Configure Prometheus metrics in `~/.inferno.toml`:
```toml
[observability]
prometheus_enabled = true
metrics_port = 9090
```
```bash
# Watch performance in real time (update every 30s)
inferno monitor watch --interval 30
```

### Service Management

#### Systemd Service (Linux)

```bash
# Create service file
sudo tee /etc/systemd/system/inferno.service << EOF
[Unit]
Description=Inferno AI Server
After=network.target

[Service]
Type=simple
User=inferno
WorkingDirectory=/opt/inferno
# Config is read from the standard locations (e.g. /opt/inferno/.inferno.toml
# or ~/.config/inferno/config.toml); set INFERNO_* env vars to override.
ExecStart=/usr/local/bin/inferno serve
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl enable inferno
sudo systemctl start inferno

# Check service status
sudo systemctl status inferno
```

#### Docker Compose (Production)

```yaml
# docker-compose.yml
version: '3.8'

services:
  inferno:
    image: ghcr.io/ringo380/inferno:latest
    ports:
      - "8080:8080"
      - "9090:9090"
    volumes:
      - ./data/models:/data/models
      - ./data/cache:/data/cache
      - ./config:/etc/inferno
      - ./logs:/var/log/inferno
    environment:
      - INFERNO_LOG_LEVEL=info
      - INFERNO_GPU_ENABLED=true
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 16G
        reservations:
          memory: 8G

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9091:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    volumes:
      - ./monitoring/grafana:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
```

```bash
# Start production stack
docker-compose up -d

# View logs
docker-compose logs -f inferno

# Scale service
docker-compose up -d --scale inferno=3
```

## Integration Examples

### Python Integration

```python
# install_requires = ["openai>=1.0.0", "requests"]

from openai import OpenAI
import json

# Configure client
client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="your-api-key"  # Use actual key in production
)

# Simple chat
def chat_with_inferno(message, model="DialoGPT-medium"):
    response = client.chat.completions.create(
        model=model,
        messages=[
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": message}
        ]
    )
    return response.choices[0].message.content

# Streaming chat
def streaming_chat(message, model="DialoGPT-medium"):
    for chunk in client.chat.completions.create(
        model=model,
        messages=[{"role": "user", "content": message}],
        stream=True
    ):
        if chunk.choices[0].delta.content:
            yield chunk.choices[0].delta.content

# Embeddings
def get_embeddings(texts, model="text-embedding-ada-002"):
    response = client.embeddings.create(
        model=model,
        input=texts
    )
    return [data.embedding for data in response.data]

# Example usage
if __name__ == "__main__":
    # Simple chat
    response = chat_with_inferno("What is machine learning?")
    print(f"Response: {response}")

    # Streaming response
    print("Streaming response:")
    for chunk in streaming_chat("Tell me a short story"):
        print(chunk, end="", flush=True)
    print()

    # Get embeddings
    embeddings = get_embeddings(["Hello", "World"])
    print(f"Embeddings shape: {len(embeddings)} x {len(embeddings[0])}")
```

### JavaScript/Node.js Integration

```javascript
// package.json dependencies: "openai": "^4.0.0"

const OpenAI = require('openai');

// Configure client
const openai = new OpenAI({
    baseURL: 'http://localhost:8080/v1',
    apiKey: 'your-api-key' // Use actual key in production
});

// Simple chat function
async function chatWithInferno(message, model = 'DialoGPT-medium') {
    try {
        const response = await openai.chat.completions.create({
            model: model,
            messages: [
                { role: 'system', content: 'You are a helpful assistant.' },
                { role: 'user', content: message }
            ]
        });
        return response.choices[0].message.content;
    } catch (error) {
        console.error('Error:', error);
        throw error;
    }
}

// Streaming chat function
async function streamingChat(message, model = 'DialoGPT-medium') {
    try {
        const stream = await openai.chat.completions.create({
            model: model,
            messages: [{ role: 'user', content: message }],
            stream: true
        });

        let fullResponse = '';
        for await (const chunk of stream) {
            const content = chunk.choices[0]?.delta?.content || '';
            process.stdout.write(content);
            fullResponse += content;
        }
        console.log(); // New line
        return fullResponse;
    } catch (error) {
        console.error('Error:', error);
        throw error;
    }
}

// Example usage
async function main() {
    try {
        // Simple chat
        const response = await chatWithInferno('What is artificial intelligence?');
        console.log('Response:', response);

        // Streaming chat
        console.log('Streaming response:');
        await streamingChat('Tell me about renewable energy');

        // List available models
        const models = await openai.models.list();
        console.log('Available models:', models.data.map(m => m.id));
    } catch (error) {
        console.error('Error in main:', error);
    }
}

main();
```

### cURL Integration

```bash
#!/bin/bash

# Configuration
BASE_URL="http://localhost:8080"
API_KEY="your-api-key"
MODEL="DialoGPT-medium"

# Helper function for API calls
call_api() {
    local endpoint="$1"
    local method="${2:-GET}"
    local data="$3"

    curl -s -X "$method" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $API_KEY" \
        ${data:+-d "$data"} \
        "$BASE_URL$endpoint"
}

# List available models
echo "Available models:"
call_api "/v1/models" | jq -r '.data[].id'

# Simple chat completion
echo -e "\nChat completion:"
response=$(call_api "/v1/chat/completions" "POST" '{
    "model": "'$MODEL'",
    "messages": [
        {"role": "user", "content": "What is the capital of France?"}
    ]
}')
echo "$response" | jq -r '.choices[0].message.content'

# Check server health
echo -e "\nServer health:"
call_api "/health" | jq '.'

# Get model status
echo -e "\nModel status:"
call_api "/v1/models/$MODEL/status" | jq '.'
```

## Troubleshooting

### Common Issues

#### Installation Issues

**Issue**: "Command not found: inferno"
```bash
# Solution: Check PATH
echo $PATH
which inferno

# Add to PATH if needed
export PATH="$PATH:/usr/local/bin"
echo 'export PATH="$PATH:/usr/local/bin"' >> ~/.bashrc
```

**Issue**: "Permission denied"
```bash
# Solution: Fix permissions
sudo chown -R $(whoami) ~/.local/share/inferno
chmod +x /usr/local/bin/inferno
```

#### Model Download Issues

**Issue**: "Model download failed"
```bash
# Solution: Check connectivity and retry
ping huggingface.co
inferno models install microsoft/DialoGPT-medium

# Or retry the install
inferno models install microsoft/DialoGPT-medium
```

**Issue**: "Insufficient disk space"
```bash
# Solution: Check space and clean up
df -h
inferno cache clear
inferno models list

# Delete an unused model file manually to reclaim space
rm ~/.local/share/inferno/models/unused-model.gguf
```

#### Performance Issues

**Issue**: "Slow inference"
```bash
# Solution: Check GPU and optimize
inferno gpu list
inferno bench --model your-model

# Use a quantized model
inferno models install TheBloke/Llama-2-7B-Chat-GGUF
```
GPU acceleration is enabled by default; to force it, set `gpu_enabled = true` under `[backend_config]` in `~/.inferno.toml`.

**Issue**: "Out of memory"
```bash
# Use a smaller model
inferno models install distilgpt2
```
Reduce memory usage in `~/.inferno.toml`:
```toml
[backend_config]
context_size = 1024
batch_size = 16
```

#### API Issues

**Issue**: "Connection refused"
```bash
# Solution: Check server status
inferno serve --bind 127.0.0.1:8081  # Try different port
netstat -tulpn | grep 8080  # Check if port is in use

# Check firewall
sudo ufw status
sudo ufw allow 8080
```

**Issue**: "Authentication failed"
```bash
# Solution: Check API keys
inferno security api-key list --user admin
inferno security api-key generate --user admin --name test
```
Or disable auth for testing by setting `auth_enabled = false` under `[security]` in `~/.inferno.toml`.

### Getting Help

#### Log Analysis

```bash
# View application logs
inferno config show | grep -i log_level
tail -f ~/.local/share/inferno/logs/inferno.log

# Enable debug logging
export INFERNO_LOG_LEVEL=debug
inferno serve

# View system logs (systemd)
sudo journalctl -u inferno -f
```

#### Diagnostic Information

```bash
# System information
inferno --version
inferno gpu list
inferno config show

# Model information
inferno models list
inferno cache stats

# Performance diagnostics
inferno bench --model your-model
inferno monitor status
```

#### Community Support

- **📚 Documentation**: [Full documentation](../README.md)
- **💬 GitHub Discussions**: [Community help](https://github.com/ringo380/inferno/discussions)
- **🐛 Bug Reports**: [GitHub Issues](https://github.com/ringo380/inferno/issues)
- **📖 Wiki**: [Community wiki](https://github.com/ringo380/inferno/wiki)

## Next Steps

Now that you have Inferno set up and running, explore these advanced topics:

### Immediate Next Steps
1. **[Package Manager Tutorial](../tutorials/package-manager.md)** - Master model installation and management
2. **[Performance Optimization](../tutorials/performance-optimization.md)** - Achieve maximum performance
3. **[API Integration](../examples/rest-api.md)** - Build applications with Inferno's API

### Development and Integration
1. **[Python SDK Guide](../examples/python.md)** - Deep dive into Python integration
2. **[WebSocket Streaming](../examples/websocket.md)** - Real-time inference streaming
3. **[Custom Backend Development](../tutorials/custom-backend.md)** - Extend Inferno with new formats

### Production Deployment
1. **[Docker Deployment](docker.md)** - Production containerization
2. **[Security Configuration](security.md)** - Enterprise security setup
3. **[Monitoring and Observability](monitoring.md)** - Production monitoring stack

### Advanced Features
1. **[Distributed Inference](distributed-inference.md)** - Scale across multiple machines
2. **[Multi-tenancy](multi-tenancy.md)** - Isolate workloads and users
3. **[Custom Dashboard](../tutorials/dashboard-customization.md)** - Extend the web interface

---

**🎉 Congratulations!** You now have a fully functional Inferno AI infrastructure. Explore the advanced guides to unlock the full potential of your local AI platform.