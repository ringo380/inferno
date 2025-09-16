# Repository Management Examples

This guide shows how to manage model repositories in Inferno, using real-world authoritative sources.

## Default Repositories

Inferno comes pre-configured with these authoritative model repositories:

### 1. Hugging Face (Priority 1)
- **URL**: https://huggingface.co
- **API**: https://huggingface.co/api/models
- **Description**: Largest collection of open-source models
- **Models**: 500K+ models including LLMs, vision, audio
- **Formats**: Transformers, GGUF, ONNX, SafeTensors

### 2. Ollama Registry (Priority 2)
- **URL**: https://registry.ollama.ai
- **API**: https://registry.ollama.ai/v2/_catalog
- **Description**: Optimized models for local inference
- **Models**: Curated collection of quantized models
- **Formats**: GGUF, custom Ollama format

### 3. ONNX Model Zoo (Priority 3)
- **URL**: https://github.com/onnx/models
- **API**: https://api.github.com/repos/onnx/models/contents
- **Description**: Official ONNX model collection
- **Models**: Computer vision, NLP, speech
- **Formats**: ONNX

### 4. PyTorch Hub (Priority 4)
- **URL**: https://pytorch.org/hub
- **API**: https://pytorch.org/hub/api/v1/models
- **Description**: PyTorch ecosystem models
- **Models**: Research and production models
- **Formats**: PyTorch, TorchScript

### 5. TensorFlow Hub (Priority 5)
- **URL**: https://tfhub.dev
- **API**: https://tfhub.dev/api/index
- **Description**: TensorFlow model repository
- **Models**: Pre-trained TF models
- **Formats**: TensorFlow SavedModel, TFLite

## Repository Commands

### List Repositories
```bash
# Show all configured repositories
inferno repo list

# Show detailed repository information
inferno repo list --detailed

# Show only enabled repositories
inferno repo list --enabled-only
```

### Add Custom Repositories
```bash
# Add a company-internal repository
inferno repo add company-models https://models.company.com --priority 10

# Add with authentication requirements
inferno repo add private-models https://private.example.com --verify

# Add and disable initially
inferno repo add experimental https://experiments.ai.com --disabled
```

### Manage Repository State
```bash
# Enable/disable repositories
inferno repo toggle ollama --disable
inferno repo toggle company-models --enable

# Update repository metadata
inferno repo update huggingface
inferno repo update --force  # Update all repositories

# Test repository connectivity
inferno repo test huggingface
inferno repo test company-models
```

### Repository Information
```bash
# Get detailed repository info
inferno repo info huggingface --models

# Set repository priority (lower = higher priority)
inferno repo priority ollama 1
inferno repo priority huggingface 2

# Clean repository cache
inferno repo clean huggingface --metadata
inferno repo clean --models  # Clean all model caches
```

## Real-World Usage Examples

### Installing Models from Different Repositories

```bash
# Install from Hugging Face (highest priority)
inferno install microsoft/DialoGPT-medium

# Search across all repositories
inferno search "llama" --limit 10

# Search in specific repository
inferno search "code generation" --repo huggingface

# Install specific model format
inferno install "meta-llama/Llama-2-7b-chat-hf"
```

### Repository-Specific Operations

```bash
# List models available in Ollama
inferno repo info ollama --models

# Search only in ONNX models
inferno search "resnet" --repo onnx-models

# Install from PyTorch Hub
inferno install "pytorch/vision:v0.10.0"
```

### Enterprise Setup

```bash
# Add internal model registry
inferno repo add enterprise https://models.enterprise.com \
  --priority 1 \
  --verify

# Disable external repositories for security
inferno repo toggle huggingface --disable
inferno repo toggle ollama --disable

# Use only enterprise repository
inferno search "internal-model" --repo enterprise
inferno install company/custom-llm-v2
```

### Academic/Research Setup

```bash
# Enable all research repositories
inferno repo toggle pytorch-hub --enable
inferno repo toggle tensorflow-hub --enable
inferno repo toggle onnx-models --enable

# Search for research models
inferno search "transformer" --repo pytorch-hub
inferno search "bert" --repo tensorflow-hub
inferno search "efficientnet" --repo onnx-models
```

## Repository Priority System

Repositories are searched in priority order (lower number = higher priority):

1. **Priority 1**: Hugging Face (default)
2. **Priority 2**: Ollama Registry
3. **Priority 3**: ONNX Model Zoo
4. **Priority 4**: PyTorch Hub
5. **Priority 5**: TensorFlow Hub

When searching or installing, Inferno will:
1. Search repositories in priority order
2. Return results from highest priority first
3. Install from first repository that has the model
4. Handle conflicts by preferring higher priority repositories

### Customizing Priorities

```bash
# Make Ollama highest priority for local inference
inferno repo priority ollama 1
inferno repo priority huggingface 2

# Prioritize internal repository
inferno repo add internal https://models.company.com --priority 1
inferno repo priority huggingface 10
```

## Authentication Examples

### Hugging Face with API Token
```bash
# Set HF token in environment
export HUGGINGFACE_TOKEN="hf_your_token_here"

# Or configure in repository
inferno repo add huggingface-pro https://huggingface.co \
  --priority 1 \
  --verify
```

### Private Repository with Authentication
```bash
# Add private repository with verification
inferno repo add private-models https://private.company.com \
  --priority 5 \
  --verify

# Repository will prompt for API key or use environment variables
```

## Best Practices

### 1. Repository Management
- Keep high-trust repositories (like HuggingFace) at high priority
- Use verification for internal/private repositories
- Regularly update repository metadata
- Test repository connectivity before important operations

### 2. Security
- Enable verification for unknown repositories
- Use authentication for private repositories
- Regularly audit trusted publishers list
- Monitor repository access logs

### 3. Performance
- Disable unused repositories to speed up searches
- Clean repository caches periodically
- Use specific repository searches when possible
- Set appropriate cache sizes for your use case

### 4. Organization
- Use descriptive repository names
- Document custom repositories for team members
- Maintain consistent priority schemes
- Back up repository configurations