# Inferno Package Manager - Complete Guide

This guide shows how to use Inferno's package manager to install, manage, and maintain AI/ML models using familiar `apt`/`yum`-style commands.

## Quick Start

### Install Your First Model
```bash
# Install a popular model from Hugging Face
inferno install microsoft/DialoGPT-medium

# Or use the short form
inferno install gpt2
```

### Search for Models
```bash
# Search across all repositories
inferno search "language model"

# Search for specific types
inferno search "llama" --limit 5
inferno search "code generation" --repo huggingface
```

### List Installed Models
```bash
# Show all installed models
inferno list

# Show detailed information
inferno list --detailed
```

## Installation Commands

### Basic Installation
```bash
# Install with automatic dependency resolution
inferno install meta-llama/Llama-2-7b-chat-hf

# Install without resolving dependencies
inferno install gpt2 --no-deps

# Install with auto-updates enabled
inferno install microsoft/DialoGPT-small --auto-update

# Skip confirmation prompts
inferno install bert-base-uncased --yes
```

### Advanced Installation
```bash
# Use the full package command
inferno package install microsoft/codebert-base \
  --no-deps \
  --auto-update \
  --yes

# Install to specific directory
inferno package install llama-7b --target /custom/path
```

## Model Discovery

### Search Commands
```bash
# Basic search
inferno search "transformer"

# Repository-specific search
inferno search "resnet" --repo onnx-models
inferno search "efficientnet" --repo pytorch-hub

# Filtered search
inferno search "llama" --limit 10 --detailed

# Search with the package command
inferno package search "code completion" \
  --repo huggingface \
  --limit 5 \
  --detailed
```

### Model Information
```bash
# Get detailed model information
inferno package info microsoft/DialoGPT-medium

# Show dependencies
inferno package info llama-7b --deps

# Show detailed metadata
inferno package info gpt2 --detailed
```

## Package Management

### Listing Packages
```bash
# List all installed packages
inferno list

# List with filter
inferno list --filter "llama"

# Show detailed information
inferno list --detailed

# Show only auto-installed packages
inferno package list --auto-only
```

### Updates and Upgrades
```bash
# Check for updates (don't install)
inferno package update --check-only

# Update specific package
inferno package update microsoft/DialoGPT-medium --yes

# Update all packages
inferno package upgrade --yes

# Dry run (show what would be updated)
inferno package upgrade --dry-run
```

### Removal
```bash
# Remove a package
inferno remove gpt2

# Remove with dependencies
inferno remove llama-7b --yes

# Remove without confirmation
inferno package remove old-model --yes --no-deps

# Remove unused dependencies
inferno package autoremove --yes
```

## Dependency Management

### Understanding Dependencies
```bash
# Show package dependencies
inferno package depends microsoft/DialoGPT-medium

# Show dependency tree
inferno package depends llama-7b --tree

# Show what depends on a package (reverse dependencies)
inferno package depends tokenizer --reverse
```

### Dependency Resolution
When you install a model, Inferno automatically:
1. Identifies required dependencies (tokenizers, configs, etc.)
2. Checks for conflicts with existing packages
3. Downloads and installs dependencies
4. Tracks which packages were auto-installed

```bash
# Install with automatic dependency resolution (default)
inferno install meta-llama/Llama-2-7b-chat-hf

# Install without dependency resolution
inferno install custom-model --no-deps
```

## Repository Integration

### Working with Multiple Repositories
```bash
# Search across all repositories
inferno search "bert"

# Search in specific repository
inferno search "llama" --repo huggingface
inferno search "resnet" --repo onnx-models
inferno search "gpt" --repo pytorch-hub

# Install from specific repository
inferno install "pytorch/vision:v0.10.0"  # PyTorch Hub
inferno install "ollama/llama2"           # Ollama
```

### Real Repository Examples

#### Hugging Face Models
```bash
# Popular language models
inferno install microsoft/DialoGPT-medium
inferno install google/flan-t5-base
inferno install microsoft/codebert-base
inferno install facebook/bart-large-cnn

# Search Hugging Face specifically
inferno search "microsoft" --repo huggingface
```

#### Ollama Models (Optimized for Local)
```bash
# Ollama optimized models
inferno search "llama" --repo ollama
inferno install ollama/llama2:7b
inferno install ollama/codellama:7b-code
```

#### ONNX Models
```bash
# Computer vision models
inferno search "resnet" --repo onnx-models
inferno install onnx/resnet50-v2-7
inferno install onnx/mobilenetv2-7
```

## Maintenance Commands

### Cleaning and Verification
```bash
# Clean package cache
inferno package clean --packages

# Clean metadata cache
inferno package clean --metadata

# Clean everything
inferno package clean --all

# Verify installed packages
inferno package check

# Deep verification with auto-fix
inferno package check --deep --fix

# Check specific package
inferno package check microsoft/DialoGPT-medium
```

### System Maintenance
```bash
# Remove unused dependencies
inferno package autoremove

# Show what would be removed (dry run)
inferno package autoremove --dry-run

# Update all repository metadata
inferno repo update

# Test repository connections
inferno repo test huggingface
```

## Advanced Features

### Package History
```bash
# Show installation history
inferno package history

# Show history for specific package
inferno package history microsoft/DialoGPT-medium --limit 5
```

### Configuration
```bash
# Show current repositories
inferno repo list

# Add custom repository
inferno repo add company-models https://models.company.com

# Configure repository priority
inferno repo priority huggingface 1
inferno repo priority company-models 2
```

## Real-World Workflows

### Data Scientist Workflow
```bash
# Set up for NLP research
inferno search "transformer" --limit 10
inferno install google/flan-t5-base
inferno install microsoft/DialoGPT-medium
inferno install facebook/bart-large-cnn

# Check what's installed
inferno list --detailed
```

### ML Engineer Workflow
```bash
# Install production models
inferno install microsoft/codebert-base --auto-update
inferno install google/universal-sentence-encoder

# Set up monitoring
inferno package check --deep

# Clean up development models
inferno package autoremove
```

### Computer Vision Workflow
```bash
# Search for vision models
inferno search "resnet" --repo onnx-models
inferno search "efficientnet" --repo pytorch-hub

# Install vision models
inferno install onnx/resnet50-v2-7
inferno install pytorch/vision:efficientnet_b0
```

### Code Generation Workflow
```bash
# Search for code models
inferno search "code" --repo huggingface
inferno search "codegen" --limit 5

# Install code generation models
inferno install microsoft/codebert-base
inferno install salesforce/codegen-350M-mono
```

## Tips and Best Practices

### Performance Tips
1. **Use specific repository searches** when you know where the model is
2. **Clean caches regularly** to free up disk space
3. **Use --yes flag** in scripts to avoid prompts
4. **Enable auto-updates** for production models

### Security Best Practices
1. **Verify model sources** before installation
2. **Use trusted repositories** (like Hugging Face, PyTorch Hub)
3. **Check model dependencies** before installation
4. **Regularly update** installed models for security patches

### Organization Tips
1. **Use descriptive names** when installing models
2. **Tag and filter** installed models by use case
3. **Document dependencies** for team projects
4. **Use repository priorities** to control model sources

## Troubleshooting

### Common Issues

#### Model Not Found
```bash
# Check repository status
inferno repo list
inferno repo test huggingface

# Update repository metadata
inferno repo update --force

# Search more broadly
inferno search "partial-model-name"
```

#### Installation Failures
```bash
# Check package integrity
inferno package check problematic-model --deep

# Reinstall with verbose output
inferno install model-name --yes

# Clear cache and retry
inferno package clean --all
inferno install model-name
```

#### Dependency Conflicts
```bash
# Show dependency tree
inferno package depends conflicting-model --tree

# Remove conflicting packages
inferno remove old-model --yes
inferno install new-model
```

## Migration from Manual Management

If you're currently managing models manually, here's how to migrate:

```bash
# Discover existing models in your directory
inferno models list

# Search for equivalent packages
inferno search "your-model-name"

# Install managed versions
inferno install official/model-name

# Clean up old manual installations
# (manually remove old model files)
```

This package manager makes AI/ML model management as easy as managing system packages, with the added benefits of dependency tracking, automatic updates, and multi-repository support!