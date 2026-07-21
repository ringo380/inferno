# Inferno CLI User Experience Demo

A quick walkthrough of the real Inferno CLI: discovering commands, finding and
installing a model, running inference, serving an API, and checking your GPU.

## Discovering Commands

Start with the top-level help to see every available command:

```bash
inferno --help
```

Any subcommand accepts `--help` for its own options:

```bash
inferno models --help
inferno run --help
inferno serve --help
```

## Finding and Installing a Model

Search HuggingFace for a model, then install it by repository ID:

```bash
# Search HuggingFace (default limit 10)
inferno models search "llama 7b" --limit 5

# Install from a HuggingFace repo ID
inferno models install TheBloke/Llama-2-7B-GGUF --file llama-2-7b.Q4_K_M.gguf

# Install by direct HTTPS URL, overriding the local filename
inferno models install https://example.com/model.gguf --name my-model.gguf
```

List what you have locally and inspect a specific model:

```bash
inferno models list
inferno models info llama-2-7b.Q4_K_M.gguf
inferno models validate llama-2-7b.Q4_K_M.gguf
```

To remove a model, delete the file from your models directory manually:

```bash
rm ~/models/llama-2-7b.Q4_K_M.gguf
```

## Running Inference

```bash
inferno run --model llama-2-7b.Q4_K_M.gguf --prompt "Explain unified memory in one sentence."
```

## Serving an OpenAI-Compatible API

```bash
# Binds 127.0.0.1:8080 by default
inferno serve

# Bind to all interfaces
inferno serve --bind 0.0.0.0:8080
```

Once running, the server exposes OpenAI-compatible endpoints:

```bash
# List models
curl http://127.0.0.1:8080/v1/models

# Chat completion
curl http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model": "llama-2-7b.Q4_K_M.gguf", "messages": [{"role": "user", "content": "Hello!"}]}'

# Health check
curl http://127.0.0.1:8080/health
```

Set `"stream": true` on a chat or completion request to receive a
`text/event-stream` of `data:` chunks ending with `data: [DONE]`.

## Checking Your GPU

```bash
# List detected GPUs (use this to confirm a GPU is available)
inferno gpu list

# Detailed info for a specific GPU (requires the GPU id)
inferno gpu info 0

# GPU health check
inferno gpu health
```

GPU acceleration (Metal on Apple Silicon) is auto-detected and enabled by default.

## Benchmarking

```bash
inferno bench --model llama-2-7b.Q4_K_M.gguf --iterations 10 --tokens 100 --output-json bench.json
```

## Typo Detection

Mistyped subcommands surface a suggestion instead of a bare error. For example,
typing `model` instead of `models`:

```bash
$ inferno model list
❓ Did you mean 'models'?
   You typed: model
💡 Try: inferno models
error: unrecognized subcommand 'model'

  tip: some similar subcommands exist: 'model-versioning', 'models'
```

## Configuration

```bash
inferno config show
inferno config init
inferno config validate
```

Run `inferno --help` at any time to see the full command list.
