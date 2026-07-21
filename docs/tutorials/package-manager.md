# 📦 Managing Models

Find, install, inspect, and run AI models with Inferno's `inferno models` commands. This tutorial walks through the real model-management workflow from a fresh install to running inference.

## Overview

Inferno manages GGUF/ONNX model files that live in your models directory. This tutorial covers:

- ✅ **Searching** HuggingFace for models
- ✅ **Installing** models from HuggingFace repos or direct URLs
- ✅ **Listing and inspecting** local models
- ✅ **Validating** and checking quantization
- ✅ **Running** inference and **serving** an HTTP API

**Time Required**: 10-15 minutes
**Skill Level**: Beginner

## Quick Start

```bash
# Search HuggingFace for a model
inferno models search "llama 7b"

# Install one from a HuggingFace repo ID
inferno models install TheBloke/Llama-2-7B-GGUF

# List what you have locally
inferno models list

# Run it
inferno run --model Llama-2-7B --prompt "Hello!"
```

## Searching for Models

`inferno models search` queries HuggingFace and returns up to 10 results by default.

```bash
# Basic search
inferno models search "mistral"
inferno models search "code generation"

# Filter by task
inferno models search "sentiment" --task text-classification

# Raise the result limit
inferno models search "llama" --limit 25
```

The results show HuggingFace repo IDs you can pass straight to `inferno models install`.

## Installing Models

`inferno models install` takes a HuggingFace repo ID or a direct HTTPS URL.

```bash
# Install from a HuggingFace repo ID
inferno models install TheBloke/Llama-2-7B-GGUF

# Pick a specific file inside the repo
inferno models install TheBloke/Llama-2-7B-GGUF --file llama-2-7b.Q4_K_M.gguf

# Override the local filename
inferno models install TheBloke/Llama-2-7B-GGUF --name llama2-7b-q4

# Install from a direct HTTPS URL
inferno models install https://example.com/models/my-model.gguf
```

Downloaded files land in your models directory (configurable via `INFERNO_MODELS_DIR` or the `models_dir` config key).

### Removing a Model

There is no CLI command to remove a model. To delete one, remove the file from your models directory manually:

```bash
# Find your models directory
inferno config show

# Delete the file
rm ~/models/llama-2-7b.Q4_K_M.gguf
```

## Listing and Inspecting Models

```bash
# List all local models
inferno models list

# Show detailed metadata for one model
inferno models info Llama-2-7B

# Usage statistics across your local models
inferno models stats
```

### Validation and Quantization

```bash
# Validate a model file (checks format, magic bytes, integrity)
inferno models validate ~/models/llama-2-7b.Q4_K_M.gguf

# Show quantization info for a model
inferno models quant Llama-2-7B
```

### Tagging

Attach a tag to a local model to help organize your collection:

```bash
inferno models tag Llama-2-7B chat
inferno models tag codellama-7b code
```

## Running Inference

Once a model is installed, run it directly:

```bash
# Single prompt
inferno run --model Llama-2-7B --prompt "Explain quantization in one sentence."

# Model can be a name or a file path
inferno run --model ~/models/llama-2-7b.Q4_K_M.gguf --prompt "Hello!"
```

On Apple Silicon, Metal GPU acceleration is auto-detected and enabled by default. To confirm a GPU is present:

```bash
inferno gpu list
```

## Serving an HTTP API

To expose models over an OpenAI-compatible HTTP API:

```bash
# Start the server (defaults to 127.0.0.1:8080)
inferno serve

# Bind to all interfaces
inferno serve --bind 0.0.0.0:8080
```

Then call the OpenAI-compatible endpoints:

```bash
# List available models
curl http://127.0.0.1:8080/v1/models

# Chat completion
curl http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "Llama-2-7B",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'

# Streaming: set "stream": true to receive a text/event-stream of data: chunks
curl http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "Llama-2-7B",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": true
  }'

# Health check
curl http://127.0.0.1:8080/health
```

Other available endpoints include `POST /v1/completions`, `POST /v1/embeddings`, `GET /metrics`, and `GET /v1/status`.

## Benchmarking a Model

Measure throughput with `inferno bench`:

```bash
# Default: 10 iterations, 100 tokens, 3 warmup runs
inferno bench --model Llama-2-7B

# Customize the run and save results
inferno bench --model Llama-2-7B \
  --iterations 20 \
  --tokens 256 \
  --prompt "Write a haiku about GPUs." \
  --output-json bench-results.json
```

## Best Practices

1. **Start small**: Begin with compact quantized models (Q4) to verify your setup before pulling larger files.
2. **Validate after install**: Run `inferno models validate <file>` on new downloads to catch corrupt or incomplete files.
3. **Check quantization**: Use `inferno models quant <model>` to understand a model's size/quality tradeoff before running it.
4. **Watch disk usage**: Model files are large. Delete unused files from your models directory to reclaim space.

## Next Steps

- **[Performance Optimization](performance-optimization.md)** - Tune inference speed and memory use
- **[Custom Backend Development](custom-backend.md)** - Support new model formats

---

You now know how to search, install, inspect, and run models with Inferno's real model-management commands.
