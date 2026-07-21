# 🖥️ CLI Reference

Inferno exposes **28 top-level commands**, each with its own subcommands and
flags. This page lists the top-level commands and the most common subcommand
groups.

> **Authoritative source:** the exact, always-current syntax comes from the CLI
> itself. Run `inferno --help` for the command list and
> `inferno <command> --help` (e.g. `inferno models --help`) for a command's
> subcommands and flags. The online documentation also publishes a generated CLI
> reference under **API Reference → CLI Commands**.

## Top-level commands

| Command | Description |
|---------|-------------|
| `run` | Run inference on text, image, or audio input |
| `batch` | Process multiple inputs in batch mode |
| `serve` | Start local HTTP API server |
| `models` | Manage and list available models |
| `metrics` | Metrics collection and export |
| `bench` | Benchmark model performance |
| `validate` | Validate model files and configurations |
| `config` | Manage configuration settings |
| `cache` | Model caching and warm-up management |
| `convert` | Convert and optimize models between formats |
| `response-cache` | Response caching and deduplication management |
| `monitor` | Real-time performance monitoring and alerting |
| `distributed` | Distributed inference with worker pools |
| `ab-test` | A/B testing and canary deployment management |
| `audit` | Comprehensive audit logging and compliance tracking |
| `queue` | Advanced batch processing with job queues and scheduling |
| `version` | Model versioning and rollback management |
| `gpu` | GPU acceleration support and management |
| `resilience` | Production resilience patterns and error recovery |
| `streaming` | Real-time streaming inference and monitoring |
| `security` | Security and access control management |
| `observability` | Observability stack for metrics, tracing, and dashboards |
| `optimization` | Model optimization with quantization, pruning, and distillation |
| `deployment` | Generate Kubernetes manifests and Helm charts |
| `model-versioning` | Model versioning and A/B testing framework |
| `performance-benchmark` | Performance benchmarking and baseline establishment |
| `upgrade` | Application upgrade and update management |
| `tui` | Launch terminal user interface |

## Common workflows

### Run inference

```bash
inferno run --model MODEL_NAME --prompt "Your prompt here"
```

### Model management

Model discovery and installation live **under `inferno models`** (there is no
top-level `install`/`search`/`remove` command):

```bash
inferno models list                 # List local models
inferno models info <MODEL>         # Show model details
inferno models search <QUERY>       # Search HuggingFace for models
inferno models install <ID|URL>     # Install from HuggingFace or a direct URL
inferno models validate <FILE>      # Validate a model file
inferno models quant <MODEL>        # Show quantization information
inferno models tag <MODEL> <TAG>    # Tag a local model
inferno models stats                # Usage statistics for local models
```

### Serve the HTTP API

```bash
inferno serve                       # Default bind: 127.0.0.1:8080
inferno serve --bind 0.0.0.0:8080
```

See [API Reference](./api-reference.md) for the HTTP endpoints.

### GPU management

```bash
inferno gpu list                    # List detected GPUs
inferno gpu info                    # Detailed GPU information
inferno gpu monitor                 # Live GPU usage
inferno gpu health                  # GPU health check
```

GPU acceleration (Metal on Apple Silicon) is auto-detected and enabled by
default; there is no `gpu enable`/`gpu status` command - use `gpu list`.

### Terminal UI

```bash
inferno tui
```

## Getting help

```bash
inferno --help                      # All top-level commands
inferno <command> --help            # Subcommands and flags for a command
inferno --version                   # Version information
```
