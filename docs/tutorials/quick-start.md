# 🚀 Quick Start Tutorial

Get Inferno running and perform your first AI inference in under 5 minutes!

## Overview

By the end of this tutorial, you'll have:
- ✅ Inferno installed and running
- ✅ Your first AI model loaded
- ✅ Performed text generation
- ✅ Accessed the web dashboard

**Time Required**: 5 minutes
**Skill Level**: Beginner
**Prerequisites**: None

## Step 1: Installation (2 minutes)

Choose your preferred installation method:

### Option A: Docker (Recommended)

```bash
# Pull and run Inferno in one command
docker run -p 8080:8080 inferno:latest serve

# Or with persistent storage
docker run -p 8080:8080 -v ./models:/data/models inferno:latest serve
```

### Option B: Build from Source

```bash
# Clone and build (requires Rust 1.85+)
git clone https://github.com/ringo380/inferno.git
cd inferno
cargo build --release

# Run Inferno
./target/release/inferno serve
```

### Option C: Binary Download

```bash
# Download pre-built binary
wget https://github.com/ringo380/inferno/releases/latest/inferno-linux-x86_64.tar.gz
tar xzf inferno-linux-x86_64.tar.gz
./inferno serve
```

**✅ Checkpoint**: You should see output like:
```
🔥 Inferno AI Server starting...
🌐 Server running at http://localhost:8080
🎛️  Dashboard available at http://localhost:8080/dashboard
📊 Metrics endpoint: http://localhost:8080/metrics
```

## Step 2: Install Your First Model (1 minute)

Inferno's package manager makes installing models as easy as installing software:

```bash
# Open a new terminal and install a conversational model
inferno models install microsoft/DialoGPT-medium

# Or try a coding assistant
inferno models install microsoft/codebert-base

# Or a larger language model (requires more memory)
inferno models install microsoft/DialoGPT-large
```

**What's happening?**
- Inferno downloads the model from HuggingFace
- Automatically converts it to the optimal format for your hardware
- Validates the model integrity
- Makes it available for inference

**✅ Checkpoint**: You should see:
```
📦 Installing microsoft/DialoGPT-medium...
⬇️  Downloading model (150MB)...
🔄 Converting to GGUF format...
✅ Model installed successfully!
```

## Step 3: Your First AI Inference (1 minute)

Now let's chat with your AI model:

### Command Line Chat

```bash
# Start a conversation
inferno run --model DialoGPT-medium --prompt "Hello! How are you today?"

# Ask a technical question
inferno run --model codebert-base --prompt "Write a Python function to sort a list"

# Creative writing
inferno run --model DialoGPT-medium --prompt "Tell me a short story about a robot learning to paint"
```

### Terminal UI

```bash
# Launch the interactive terminal user interface
inferno tui
```

**Example output:**
```
🤖 DialoGPT-medium: Hello! I'm doing great, thank you for asking!
I'm excited to help you with any questions or tasks you have.
What would you like to talk about today?
```

## Step 4: API Usage (1 minute)

Inferno provides an OpenAI-compatible API, so you can use existing tools:

### Test with cURL

```bash
# Simple completion
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "DialoGPT-medium",
    "messages": [
      {"role": "user", "content": "Explain quantum computing in simple terms"}
    ]
  }'
```

### Use with Python

```python
from openai import OpenAI

# Point to your local Inferno instance
client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="not-needed"  # Inferno doesn't require API keys by default
)

# Chat with your model
response = client.chat.completions.create(
    model="DialoGPT-medium",
    messages=[
        {"role": "user", "content": "What are the benefits of local AI?"}
    ]
)

print(response.choices[0].message.content)
```

### Streaming Responses

```python
# Get streaming responses for real-time output
for chunk in client.chat.completions.create(
    model="DialoGPT-medium",
    messages=[{"role": "user", "content": "Write a poem about AI"}],
    stream=True
):
    if chunk.choices[0].delta.content:
        print(chunk.choices[0].delta.content, end="")
```

## Step 5: Web Dashboard

Open your browser and visit: **http://localhost:8080/dashboard**

The dashboard provides:
- 📊 **Real-time metrics**: Token generation rate, memory usage, GPU utilization
- 🎛️ **Model management**: View, load, and switch between models
- 💬 **Chat interface**: Test models directly in the browser
- 🔧 **Configuration**: Adjust settings without restarting
- 📈 **Performance monitoring**: Track inference latency and throughput

## 🎉 Congratulations!

You now have a fully functional local AI infrastructure! Here's what you've accomplished:

- ✅ **Installed Inferno** using your preferred method
- ✅ **Downloaded and optimized** an AI model automatically
- ✅ **Generated text** using command line and API
- ✅ **Accessed the web dashboard** for visual management
- ✅ **Used OpenAI-compatible APIs** for easy integration

## Next Steps

### Immediate Next Steps (5-10 minutes)
1. **[Try More Models](package-manager.md)**: Install specialized models for different tasks
2. **[Explore the CLI](../reference/cli-reference.md)**: Learn about Inferno's 45+ commands
3. **[Performance Optimization](performance-optimization.md)**: Make your models run faster

### For Developers (15-30 minutes)
1. **[API Integration](../examples/rest-api.md)**: Build applications using Inferno's API
2. **[Model Management](model-management.md)**: Upload your own models and convert formats
3. **[Batch Processing](batch-processing.md)**: Process large datasets efficiently

### For Production (1-2 hours)
1. **[Docker Deployment](../guides/docker.md)**: Deploy with Docker Compose
2. **[Security Setup](../guides/security.md)**: Enable authentication and monitoring
3. **[Performance Tuning](../guides/performance-tuning.md)**: Optimize for your hardware

## Quick Reference Commands

```bash
# Model Management
inferno models install <model>       # Install a model
inferno models list                  # List installed models
inferno models search "language model"  # Search for models
inferno models info <model>          # Show model details
# To remove a model, delete its file from your models directory:
#   rm <models_dir>/<model>.gguf

# Running Inference
inferno run --model <model> --prompt "text"    # One-off inference
inferno serve                                   # Start API server
inferno tui                                     # Terminal UI

# Convert Formats
inferno convert model <input> <output> --format onnx  # Convert model formats

# System
inferno --help                       # Show all commands
inferno <command> --help             # Show command-specific help
```

## Troubleshooting

### Common Issues

**Server won't start:**
```bash
# Check if port 8080 is already in use
lsof -i :8080

# Use a different port
inferno serve --bind 127.0.0.1:8081
```

**Model download fails:**
```bash
# Check your internet connection and re-run the install
inferno models install microsoft/DialoGPT-medium
```

**Out of memory:**
```bash
# Use a smaller model
inferno models install distilgpt2

# Or reduce context_size / batch_size in your config file
# (see [backend_config] in ~/.inferno.toml)
```

**Need help?**
- 📚 [Full Troubleshooting Guide](../guides/troubleshooting.md)
- 💬 [GitHub Discussions](https://github.com/ringo380/inferno/discussions)
- 🐛 [Report Issues](https://github.com/ringo380/inferno/issues)

---

**🔥 Ready for more?** Check out the [Package Manager Tutorial](package-manager.md) to learn how to install and manage dozens of different AI models!