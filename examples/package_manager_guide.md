# Managing Models with `inferno models`

This guide covers managing local AI/ML models with the real `inferno models`
subcommands: discovering models on Hugging Face, installing them, inspecting
metadata, validating files, and viewing usage statistics.

## Quick Start

```bash
# Search Hugging Face for a model
inferno models search "llama"

# Install a model from a Hugging Face repo ID
inferno models install TheBloke/Llama-2-7B-GGUF

# List the models you have locally
inferno models list
```

## Searching Hugging Face

```bash
# Basic search (default limit is 10 results)
inferno models search "language model"

# Narrow by task and cap the number of results
inferno models search "code generation" --task text-generation --limit 5
```

## Installing Models

`inferno models install` pulls from a Hugging Face repo ID or a direct HTTPS URL.

```bash
# Install from a Hugging Face repo ID
inferno models install TheBloke/Llama-2-7B-GGUF

# Pick a specific file within the repo
inferno models install TheBloke/Llama-2-7B-GGUF --file llama-2-7b.Q4_K_M.gguf

# Override the local filename
inferno models install TheBloke/Llama-2-7B-GGUF --name llama2-7b.gguf

# Install directly from an HTTPS URL
inferno models install https://example.com/models/model.gguf
```

## Inspecting Models

```bash
# Detailed information about a local model
inferno models info llama2-7b.gguf

# Show quantization info
inferno models quant llama2-7b.gguf

# Validate a model file
inferno models validate ~/models/llama2-7b.gguf
```

## Organizing and Tracking

```bash
# Tag a local model for easier organization
inferno models tag llama2-7b.gguf production

# Usage statistics across your local models
inferno models stats
```

## Removing a Model

There is no CLI command to remove or uninstall a model. To remove one, delete
the file from your models directory yourself:

```bash
rm ~/models/llama2-7b.gguf
```

## Running an Installed Model

Once a model is installed, run inference with it:

```bash
inferno run --model llama2-7b.gguf --prompt "Explain unified memory."
```
