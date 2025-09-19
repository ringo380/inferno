# 📦 Package Manager Tutorial

Master Inferno's revolutionary package manager - install, manage, and optimize AI models like software packages with apt/yum-style commands.

## Overview

Inferno's package manager transforms AI model management from a complex, manual process into something as simple as installing software. This tutorial covers:

- ✅ **Installing models** from multiple repositories
- ✅ **Searching and discovering** models across 500K+ options
- ✅ **Managing repositories** and custom sources
- ✅ **Upgrading and maintaining** model installations
- ✅ **Advanced features** like auto-updates and dependencies

**Time Required**: 15-20 minutes
**Skill Level**: Beginner to Intermediate

## Why Use the Package Manager?

Traditional AI model management is painful:
- Manual downloads from various sources
- Complex installation procedures
- No dependency management
- No version control or updates
- Inconsistent model formats

Inferno's package manager solves all of this:
- One command installs any model
- Automatic format conversion and optimization
- Dependency resolution and conflict detection
- Seamless updates and rollbacks
- Unified interface across all repositories

## Quick Start

```bash
# Install a popular conversational model
inferno install microsoft/DialoGPT-medium

# Start using it immediately
inferno run --model DialoGPT-medium --prompt "Hello!"

# That's it! 🎉
```

## Repository System

Inferno comes pre-configured with major AI model repositories:

### Pre-configured Repositories

| Repository | Models | Specialty | Size |
|------------|--------|-----------|------|
| **🤗 Hugging Face** | 500K+ | General AI, NLP, Vision, Audio | Largest |
| **🦙 Ollama** | 100+ | Optimized for local inference | Fast |
| **📊 ONNX Model Zoo** | 200+ | Computer vision, official models | Reliable |
| **🔥 PyTorch Hub** | 1000+ | Research models, cutting-edge | Advanced |
| **🧠 TensorFlow Hub** | 4000+ | Production models, Google | Stable |

### Repository Management

```bash
# List configured repositories
inferno repo list

# Add custom repository
inferno repo add company-models https://models.company.com

# Add with authentication
inferno repo add private-repo https://private.models.com \
  --username admin --password secret

# Set repository priority
inferno repo add enterprise https://enterprise.models.com --priority 1

# Update repository metadata
inferno repo update
inferno repo update huggingface  # Update specific repo

# Enable/disable repositories
inferno repo disable pytorch-hub
inferno repo enable pytorch-hub
```

## Model Discovery and Search

### Basic Search

```bash
# Search all repositories
inferno search "language model"
inferno search "gpt"
inferno search "bert"

# Search with filters
inferno search "vision model" --category computer-vision
inferno search "embedding" --size small
inferno search "llama" --license apache
```

### Advanced Search

```bash
# Repository-specific search
inferno search "code generation" --repo huggingface
inferno search "optimization" --repo onnx

# Category filtering
inferno search "model" --category nlp
inferno search "model" --category computer-vision
inferno search "model" --category audio
inferno search "model" --category multimodal

# Size filtering
inferno search "language model" --size small     # < 1GB
inferno search "language model" --size medium    # 1-10GB
inferno search "language model" --size large     # 10-50GB
inferno search "language model" --size xlarge    # > 50GB

# Sort results
inferno search "gpt" --sort downloads     # Most popular
inferno search "bert" --sort recent       # Most recent
inferno search "llama" --sort stars       # Most starred

# Limit results
inferno search "transformer" --limit 20
```

### Search Output Formats

```bash
# Table format (default)
inferno search "gpt2"

# Compact format
inferno search "gpt2" --format compact

# JSON format for scripting
inferno search "gpt2" --format json | jq '.[0].name'

# Detailed information
inferno search "gpt2" --detailed
```

## Model Installation

### Basic Installation

```bash
# Install from default repository (Hugging Face)
inferno install microsoft/DialoGPT-medium
inferno install gpt2
inferno install bert-base-uncased

# Install from specific repository
inferno install llama2:7b --repo ollama
inferno install resnet50 --repo onnx
inferno install gpt-neo-1.3B --repo pytorch-hub
```

### Installation Options

```bash
# Install with automatic updates
inferno install mistralai/Mistral-7B-v0.1 --auto-update

# Install with specific quantization
inferno install llama-2-7b-chat --quantization q4_0
inferno install code-llama-7b --quantization q8_0

# Force reinstallation
inferno install gpt2 --force

# Install without using cache
inferno install large-model --no-cache

# Verify checksums during installation
inferno install critical-model --verify
```

### Installation Examples by Use Case

#### Conversational AI

```bash
# Small, fast chat models
inferno install microsoft/DialoGPT-small      # 117MB, quick responses
inferno install distilgpt2                    # 82MB, very fast

# High-quality conversation
inferno install microsoft/DialoGPT-large      # 776MB, excellent quality
inferno install facebook/blenderbot-400M-distill  # 400MB, engaging chat

# Instruction-following models
inferno install microsoft/DialoGPT-medium     # 345MB, balanced
inferno install togethercomputer/RedPajama-INCITE-Chat-3B-v1  # 3GB, powerful
```

#### Code Generation

```bash
# Code completion and generation
inferno install microsoft/codebert-base       # 500MB, code understanding
inferno install salesforce/codegen-350M-mono  # 350MB, fast code gen
inferno install codellama/CodeLlama-7b-Instruct  # 3.8GB, advanced coding

# Specialized code models
inferno install microsoft/codebert-base-mlm    # 500MB, masked language modeling
inferno install huggingface/CodeBERTa-small-v1  # 84MB, compact code model
```

#### Text Analysis and Embeddings

```bash
# Sentence embeddings
inferno install sentence-transformers/all-MiniLM-L6-v2  # 90MB, fast embeddings
inferno install sentence-transformers/all-mpnet-base-v2  # 420MB, high quality

# Text classification
inferno install cardiffnlp/twitter-roberta-base-sentiment  # 500MB, sentiment
inferno install facebook/bart-large-mnli       # 1.6GB, natural language inference

# Named entity recognition
inferno install dbmdz/bert-large-cased-finetuned-conll03-english  # 1.3GB, NER
```

#### Multilingual Models

```bash
# Multilingual conversation
inferno install microsoft/DialoGPT-medium     # English, but works with other languages
inferno install facebook/mbart-large-50-many-to-many-mmt  # 2.3GB, 50 languages

# Multilingual embeddings
inferno install sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2  # 470MB
inferno install sentence-transformers/distiluse-base-multilingual-cased  # 540MB
```

## Model Management

### Listing Installed Models

```bash
# List all installed models
inferno list

# Detailed listing with metadata
inferno list --detailed

# Show file sizes and disk usage
inferno list --size

# Show usage statistics
inferno list --usage

# Filter by category
inferno list --category nlp
inferno list --category computer-vision

# Show only unused models
inferno list --unused

# Show models with available updates
inferno list --outdated
```

### Model Information

```bash
# Get detailed model information
inferno package info microsoft/DialoGPT-medium

# Show installation details
inferno package info gpt2 --installation

# Show usage statistics
inferno package info bert-base --usage

# Show dependencies
inferno package info complex-model --dependencies
```

### Model Removal

```bash
# Remove single model
inferno remove microsoft/DialoGPT-small

# Remove multiple models
inferno remove old-model1 old-model2 old-model3

# Remove with confirmation prompts
inferno remove dangerous-model --interactive

# Force removal without confirmation
inferno remove test-model --force

# Remove only cache, keep model
inferno remove cached-model --cache-only

# Complete removal including all traces
inferno remove experimental-model --purge

# Remove all unused models
inferno remove --unused

# Dry run - show what would be removed
inferno remove old-models --dry-run
```

## Package Updates and Maintenance

### Updating Models

```bash
# Update all models
inferno package upgrade

# Update specific model
inferno package upgrade microsoft/DialoGPT-medium

# Check for available updates
inferno package list-upgrades

# Update with confirmation
inferno package upgrade --interactive

# Force update even if version is same
inferno package upgrade gpt2 --force
```

### Automatic Updates

```bash
# Enable auto-updates for specific model
inferno package config microsoft/DialoGPT-medium --auto-update

# Disable auto-updates
inferno package config gpt2 --no-auto-update

# Set update schedule (daily, weekly, monthly)
inferno package config llama-2-7b --auto-update --schedule weekly

# Check auto-update status
inferno package config --list-auto-updates
```

### Package Database Maintenance

```bash
# Update repository metadata
inferno repo update

# Repair package database
inferno package repair

# Clean package cache
inferno package clean

# Verify package integrity
inferno package verify --all

# Rebuild package database
inferno package rebuild-db
```

## Advanced Features

### Dependency Management

```bash
# Install with dependencies
inferno install complex-model --include-dependencies

# Show dependency tree
inferno package dependencies microsoft/DialoGPT-medium

# Check for dependency conflicts
inferno package check-conflicts

# Resolve conflicts automatically
inferno package resolve-conflicts --auto
```

### Version Management

```bash
# Install specific version
inferno install gpt2@v1.0.0

# List available versions
inferno package versions gpt2

# Pin to specific version (no auto-updates)
inferno package pin gpt2@v1.0.0

# Unpin version
inferno package unpin gpt2

# Rollback to previous version
inferno package rollback gpt2
```

### Model Variants and Quantization

```bash
# List available variants
inferno package variants llama-2-7b

# Install specific quantization
inferno install llama-2-7b-chat-q4_0    # 4-bit quantization
inferno install llama-2-7b-chat-q8_0    # 8-bit quantization
inferno install llama-2-7b-chat-f16     # 16-bit float

# Install with automatic quantization
inferno install large-model --auto-quantize

# Compare quantization performance
inferno bench llama-2-7b-f16
inferno bench llama-2-7b-q4_0
```

### Batch Operations

```bash
# Install multiple models from file
cat > models.txt << EOF
microsoft/DialoGPT-medium
microsoft/codebert-base
sentence-transformers/all-MiniLM-L6-v2
EOF

inferno install --batch models.txt

# Install models by pattern
inferno install "microsoft/*" --pattern

# Export installed models list
inferno list --format json > installed_models.json

# Install from exported list
inferno install --from-file installed_models.json
```

## Custom Repositories

### Setting Up Private Repositories

```bash
# Add private repository with authentication
inferno repo add company-models https://models.company.com \
  --username admin \
  --password secret \
  --verify-ssl false

# Add with API token
inferno repo add private-hub https://private.hub.com \
  --token your-api-token

# Add with priority (higher number = higher priority)
inferno repo add enterprise https://enterprise.models.com --priority 10
```

### Repository Configuration

```bash
# List repository details
inferno repo list --detailed

# Configure repository settings
inferno repo config company-models --timeout 300
inferno repo config company-models --retry-count 3
inferno repo config company-models --mirror https://mirror.company.com

# Test repository connectivity
inferno repo test company-models

# Enable/disable repository verification
inferno repo config company-models --verify-checksums true
```

### Creating a Repository Index

For hosting your own repository:

```yaml
# models.yaml - Repository index format
models:
  - name: "company/custom-gpt"
    version: "1.0.0"
    description: "Custom GPT model for company use"
    size: 2400000000
    format: "gguf"
    checksum: "sha256:abcdef123456..."
    download_url: "https://models.company.com/custom-gpt-v1.gguf"
    dependencies:
      - "tokenizer-base"
    metadata:
      category: "nlp"
      license: "proprietary"
      authors: ["Company AI Team"]
```

## Performance and Optimization

### Cache Management

```bash
# Check cache status
inferno cache status

# Warm cache with popular models
inferno cache warm --popular

# Pre-load specific models
inferno cache warm microsoft/DialoGPT-medium gpt2

# Clear cache for specific model
inferno cache clear --model gpt2

# Optimize cache storage
inferno cache optimize
```

### Download Optimization

```bash
# Use multiple connections for faster downloads
inferno install large-model --parallel 8

# Resume interrupted downloads
inferno install large-model --resume

# Use specific mirror
inferno install gpt2 --mirror https://mirror.huggingface.co

# Limit download bandwidth
inferno install large-model --limit-rate 10M
```

### Storage Management

```bash
# Check disk usage
inferno package disk-usage

# Find largest models
inferno list --size --sort size

# Compress models to save space
inferno package compress old-models

# Move models to different location
inferno package relocate --models-dir /new/location
```

## Integration with Development Workflow

### CI/CD Integration

```bash
#!/bin/bash
# ci-install-models.sh

# Install required models for testing
models=(
    "microsoft/DialoGPT-small"
    "sentence-transformers/all-MiniLM-L6-v2"
    "gpt2"
)

for model in "${models[@]}"; do
    echo "Installing $model..."
    inferno install "$model" --no-cache --verify
done

# Verify installations
inferno list --format json > installed_models.json
```

### Docker Integration

```dockerfile
FROM inferno:latest

# Install models during image build
RUN inferno install microsoft/DialoGPT-medium --no-cache
RUN inferno install gpt2 --no-cache

# Warm cache
RUN inferno cache warm --all

EXPOSE 8080
CMD ["inferno", "serve"]
```

### Configuration Management

```toml
# inferno.toml - Model configuration
[packages]
auto_update = true
update_schedule = "weekly"
verify_checksums = true

[repositories]
default_timeout = 300
max_parallel_downloads = 4
use_mirrors = true

[cache]
max_size_gb = 50
compression = "zstd"
```

## Troubleshooting

### Common Issues

#### Download Failures

```bash
# Check network connectivity
ping huggingface.co

# Retry with verbose output
inferno install model-name --verbose --retry

# Use different repository
inferno install model-name --repo ollama

# Check repository status
inferno repo test huggingface
```

#### Storage Issues

```bash
# Check available space
df -h $(inferno config get models_dir)

# Clean up unused models
inferno remove --unused

# Clear cache
inferno cache clear

# Compress models
inferno package compress --all
```

#### Authentication Issues

```bash
# Check repository authentication
inferno repo list --auth-status

# Re-authenticate
inferno repo auth huggingface --token new-token

# Test authentication
inferno repo test private-repo
```

#### Dependency Conflicts

```bash
# Check for conflicts
inferno package check-conflicts

# Resolve automatically
inferno package resolve-conflicts --auto

# Manual resolution
inferno package dependencies --tree
inferno remove conflicting-model
```

### Debug Mode

```bash
# Enable debug logging
export INFERNO_LOG_LEVEL=debug

# Verbose package operations
inferno install model-name --verbose

# Trace package operations
inferno --trace install model-name
```

## Best Practices

### Model Selection

1. **Start Small**: Begin with compact models for testing
2. **Match Use Case**: Choose models optimized for your task
3. **Consider Resources**: Select models that fit your hardware
4. **Test Quantization**: Try quantized models for better performance

### Repository Management

1. **Use Priorities**: Set repository priorities based on trust
2. **Regular Updates**: Keep repository metadata up to date
3. **Monitor Size**: Track repository and model sizes
4. **Backup Lists**: Export model lists for disaster recovery

### Maintenance

1. **Regular Cleanup**: Remove unused models periodically
2. **Monitor Updates**: Keep models updated for improvements
3. **Check Health**: Verify model integrity regularly
4. **Optimize Cache**: Manage cache size and compression

## Next Steps

Now that you've mastered the package manager, explore these advanced topics:

### Immediate Next Steps
1. **[Performance Optimization](performance-optimization.md)** - Optimize installed models
2. **[Model Management](model-management.md)** - Advanced model operations
3. **[Custom Backend Development](custom-backend.md)** - Support new model formats

### Production Topics
1. **[Security Configuration](../guides/security.md)** - Secure package management
2. **[Monitoring Setup](../guides/monitoring.md)** - Monitor package operations
3. **[Backup and Recovery](../guides/backup-recovery.md)** - Protect your model investments

---

**🎉 Congratulations!** You now have complete mastery over Inferno's package manager. You can discover, install, and manage AI models with the same ease as installing software packages. The entire AI model ecosystem is now at your fingertips!