# Finding and Installing Models

Inferno acquires models from HuggingFace or a direct download URL, using the
`inferno models` subcommands. There is no separate repository or source to
register - you search HuggingFace and install by repo ID or URL.

## Search HuggingFace

```bash
# Search by keyword (default limit 10)
inferno models search "llama"

# Filter by task and cap the number of results
inferno models search "code generation" --task text-generation --limit 5
```

## Install a Model

`inferno models install` accepts either a HuggingFace repo ID or a direct HTTPS URL.

```bash
# Install by HuggingFace repo ID
inferno models install TheBloke/Llama-2-7B-GGUF

# Pick a specific file within the repo
inferno models install TheBloke/Llama-2-7B-GGUF --file llama-2-7b.Q4_K_M.gguf

# Override the local filename
inferno models install TheBloke/Llama-2-7B-GGUF \
  --file llama-2-7b.Q4_K_M.gguf \
  --name llama2-7b-q4

# Install directly from an HTTPS URL
inferno models install https://example.com/models/my-model.gguf
```

## Inspect Installed Models

```bash
# List models in your models directory
inferno models list

# Show details for one model
inferno models info llama2-7b-q4

# Validate a model file
inferno models validate ~/models/llama2-7b-q4.gguf

# Show quantization info
inferno models quant llama2-7b-q4

# Usage statistics across local models
inferno models stats
```

## Remove a Model

There is no uninstall command. Delete the file from your models directory:

```bash
rm ~/models/llama2-7b-q4.gguf
```

## Next Steps

```bash
# Run inference with an installed model
inferno run --model llama2-7b-q4 --prompt "Hello"

# Serve an OpenAI-compatible API
inferno serve --bind 127.0.0.1:8080
```
