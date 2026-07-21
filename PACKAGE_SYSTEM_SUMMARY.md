# Model Management Summary

Inferno manages local AI/ML models under the `inferno models` command. It can search
HuggingFace, install models by repo ID or direct URL, and inspect, validate, tag, and
report on models already on disk.

## Commands

All model operations are subcommands of `inferno models`:

- `inferno models list` - List local models
- `inferno models info <MODEL>` - Show details for a local model
- `inferno models search <QUERY> [--task <T>] [--limit <N>]` - Search HuggingFace (default limit 10)
- `inferno models install <MODEL> [--file <F>] [--name <N>]` - Install a model
- `inferno models validate <FILE>` - Validate a model file
- `inferno models quant <MODEL>` - Show quantization info
- `inferno models tag <MODEL> <TAG>` - Tag a local model
- `inferno models stats` - Usage statistics for local models

## Searching and installing

Search HuggingFace for models, optionally filtering by task:

```bash
inferno models search llama --task text-generation --limit 5
```

Install from a HuggingFace repo ID or a direct HTTPS URL. `--file` selects a specific
file within an HF repo; `--name` overrides the local filename:

```bash
# From a HuggingFace repo ID
inferno models install TheBloke/Llama-2-7B-GGUF

# Pick a specific file from the repo
inferno models install TheBloke/Llama-2-7B-GGUF --file llama-2-7b.Q4_K_M.gguf

# From a direct URL, with a custom local name
inferno models install https://example.com/model.gguf --name my-model.gguf
```

## Inspecting local models

```bash
inferno models list
inferno models info my-model.gguf
inferno models quant my-model.gguf
inferno models validate my-model.gguf
inferno models tag my-model.gguf production
inferno models stats
```

## Removing a model

There is no CLI command to remove or update a model. To remove one, delete the file
from your models directory manually, e.g. `rm ~/models/my-model.gguf`. The models
directory can be overridden with `INFERNO_MODELS_DIR`.
