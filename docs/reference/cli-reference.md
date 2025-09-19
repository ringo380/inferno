# 🖥️ CLI Reference

Complete documentation for all Inferno command-line interface commands. Inferno provides 45+ commands organized into logical groups for model management, inference, monitoring, and enterprise features.

## Quick Reference

```bash
# Package Management (Recommended)
inferno install <model>              # Install a model package
inferno remove <model>               # Remove a model package
inferno search <query>               # Search for models
inferno list                         # List installed models
inferno repo add <name> <url>        # Add model repository

# Inference & Execution
inferno run                          # Run inference
inferno batch                        # Batch processing
inferno serve                        # Start API server
inferno tui                          # Terminal interface

# Model Management
inferno models list                  # List models
inferno models info <model>          # Model details
inferno convert <input> <output>     # Convert formats
inferno validate <model>             # Validate model

# Performance & Monitoring
inferno bench                        # Benchmark performance
inferno metrics                      # Collect metrics
inferno monitor                      # Real-time monitoring
inferno cache                        # Cache management

# Enterprise Features
inferno security                     # Security management
inferno audit                        # Audit logging
inferno distributed                  # Distributed inference
inferno observability               # Monitoring stack
```

## Package Management Commands

### `inferno install`
Install AI models like software packages from configured repositories.

```bash
# Basic installation
inferno install microsoft/DialoGPT-medium
inferno install huggingface/gpt2
inferno install ollama/llama2:7b

# From specific repository
inferno install microsoft/DialoGPT-medium --repo huggingface
inferno install custom-model --repo enterprise

# With options
inferno install llama-2-7b --auto-update         # Enable auto-updates
inferno install gpt-3.5 --quantization q4_0      # Install with quantization
inferno install bert-base --no-cache             # Skip caching
inferno install mistral-7b --verify              # Verify checksums
```

**Options:**
- `--repo <name>`: Install from specific repository
- `--auto-update`: Enable automatic updates
- `--quantization <type>`: Quantization level (q4_0, q4_1, q5_0, q5_1, q8_0, f16, f32)
- `--no-cache`: Skip caching during installation
- `--verify`: Verify model checksums and signatures
- `--force`: Force reinstallation if already exists

### `inferno remove`
Remove installed model packages.

```bash
# Remove single model
inferno remove microsoft/DialoGPT-medium

# Remove multiple models
inferno remove gpt2 bert-base llama-2-7b

# Remove with options
inferno remove old-model --purge                 # Remove all traces
inferno remove --unused                          # Remove unused models
inferno remove --cache-only                      # Keep model, remove cache
```

**Options:**
- `--purge`: Remove all associated files and cache
- `--unused`: Remove models not used in last 30 days
- `--cache-only`: Remove only cache files, keep model
- `--force`: Skip confirmation prompts

### `inferno search`
Search for AI models across configured repositories.

```bash
# Basic search
inferno search "language model"
inferno search "code generation"
inferno search gpt

# Repository-specific search
inferno search "vision model" --repo huggingface
inferno search "embedding" --repo onnx

# Advanced search
inferno search "llama" --category nlp            # Filter by category
inferno search "bert" --size small               # Filter by model size
inferno search "gpt" --license mit               # Filter by license
inferno search "mistral" --sort downloads        # Sort by downloads
inferno search "code" --limit 20                 # Limit results
```

**Options:**
- `--repo <name>`: Search specific repository
- `--category <cat>`: Filter by category (nlp, vision, audio, multimodal)
- `--size <size>`: Filter by size (small, medium, large, xlarge)
- `--license <license>`: Filter by license (mit, apache, custom)
- `--sort <field>`: Sort by field (downloads, stars, recent, name)
- `--limit <num>`: Limit number of results
- `--format <fmt>`: Output format (table, json, compact)

### `inferno list`
List installed model packages with details.

```bash
# List all installed models
inferno list

# Detailed listing
inferno list --detailed                          # Show full details
inferno list --format json                       # JSON output
inferno list --size                              # Include file sizes
inferno list --usage                             # Show usage statistics

# Filter listings
inferno list --category nlp                      # Filter by category
inferno list --unused                            # Show unused models
inferno list --outdated                          # Show outdated models
```

**Options:**
- `--detailed`: Show comprehensive model information
- `--format <fmt>`: Output format (table, json, yaml, compact)
- `--size`: Include file sizes and disk usage
- `--usage`: Show usage statistics and last access time
- `--category <cat>`: Filter by model category
- `--unused`: Show models not used recently
- `--outdated`: Show models with available updates

### `inferno repo`
Manage model repositories and sources.

```bash
# List repositories
inferno repo list
inferno repo list --detailed                     # Show full details

# Add repository
inferno repo add enterprise https://models.company.com
inferno repo add custom https://my-models.com --priority 1

# Manage repositories
inferno repo update                               # Update all repositories
inferno repo update huggingface                  # Update specific repo
inferno repo remove custom                       # Remove repository
inferno repo enable huggingface                  # Enable repository
inferno repo disable custom                      # Disable repository

# Authentication
inferno repo auth huggingface --token <token>    # Set auth token
inferno repo auth enterprise --username user --password pass
```

**Subcommands:**
- `list`: List configured repositories
- `add <name> <url>`: Add new repository
- `remove <name>`: Remove repository
- `update [name]`: Update repository metadata
- `enable <name>`: Enable repository
- `disable <name>`: Disable repository
- `auth <name>`: Configure repository authentication

## Inference & Execution Commands

### `inferno run`
Run AI inference on text, images, or audio.

```bash
# Text inference
inferno run --model gpt2 --prompt "Hello world"
inferno run --model llama-2-7b --prompt "Explain quantum computing"

# File input/output
inferno run --model bert --input document.txt --output summary.txt
inferno run --model gpt-3.5 --input questions.jsonl --output answers.jsonl

# Interactive mode
inferno run --model DialoGPT-medium --interactive
inferno run --model llama-2-7b --chat                # Chat interface

# Streaming output
inferno run --model gpt-3.5 --prompt "Write a story" --stream
inferno run --model code-llama --input code.py --stream

# Advanced options
inferno run --model llama-2-7b --temperature 0.7     # Creativity control
inferno run --model gpt2 --max-tokens 500            # Token limit
inferno run --model mistral --context-size 4096      # Context window
inferno run --model bert --batch-size 32             # Batch processing
```

**Options:**
- `--model <name>`: Model to use for inference
- `--prompt <text>`: Input text prompt
- `--input <file>`: Input file path
- `--output <file>`: Output file path
- `--interactive`: Interactive chat mode
- `--chat`: Enhanced chat interface
- `--stream`: Stream output in real-time
- `--temperature <val>`: Sampling temperature (0.0-2.0)
- `--max-tokens <num>`: Maximum tokens to generate
- `--context-size <size>`: Context window size
- `--batch-size <size>`: Batch processing size

### `inferno batch`
Process multiple inputs efficiently in batch mode.

```bash
# Basic batch processing
inferno batch --model gpt2 --input batch.jsonl --output results.jsonl
inferno batch --model llama-2-7b --input questions/ --output answers/

# Advanced batch options
inferno batch --model bert --input data.jsonl --workers 4
inferno batch --model gpt-3.5 --input large.jsonl --chunk-size 100
inferno batch --model mistral --input prompts.txt --parallel 8

# Monitoring and resume
inferno batch --model llama --input data.jsonl --progress
inferno batch --model gpt2 --input data.jsonl --resume checkpoint.json
```

**Options:**
- `--model <name>`: Model for batch processing
- `--input <path>`: Input file or directory
- `--output <path>`: Output file or directory
- `--workers <num>`: Number of worker processes
- `--chunk-size <size>`: Items per chunk
- `--parallel <num>`: Parallel processing level
- `--progress`: Show progress bar
- `--resume <file>`: Resume from checkpoint

### `inferno serve`
Start the HTTP API server for web and API access.

```bash
# Basic server
inferno serve
inferno serve --port 8081                        # Custom port
inferno serve --bind 0.0.0.0:8080               # Custom address

# Production server
inferno serve --auth                              # Enable authentication
inferno serve --metrics                          # Enable metrics endpoint
inferno serve --cors                             # Enable CORS
inferno serve --rate-limit 1000                  # Rate limiting

# SSL/TLS
inferno serve --ssl --cert server.crt --key server.key
inferno serve --ssl-auto                         # Auto SSL certificates

# Multi-model serving
inferno serve --models gpt2,llama-2-7b,bert     # Serve multiple models
inferno serve --model-config models.toml         # Model configuration
```

**Options:**
- `--port <port>`: Server port (default: 8080)
- `--bind <addr>`: Bind address (default: 127.0.0.1)
- `--auth`: Enable authentication
- `--metrics`: Enable Prometheus metrics
- `--cors`: Enable CORS headers
- `--rate-limit <num>`: Requests per minute limit
- `--ssl`: Enable SSL/TLS
- `--cert <file>`: SSL certificate file
- `--key <file>`: SSL private key file
- `--models <list>`: Comma-separated model list
- `--workers <num>`: Number of worker threads

### `inferno tui`
Launch the terminal user interface for interactive management.

```bash
# Launch TUI
inferno tui

# TUI with options
inferno tui --theme dark                         # Color theme
inferno tui --refresh 1000                       # Refresh rate (ms)
inferno tui --model gpt2                         # Start with specific model
```

**Features:**
- Real-time metrics dashboard
- Interactive model management
- Chat interface
- Performance monitoring
- Configuration management

## Model Management Commands

### `inferno models`
Direct model management for advanced users.

```bash
# List models
inferno models list                               # List available models
inferno models list --detailed                   # Detailed information
inferno models list --format json                # JSON output

# Model information
inferno models info llama-2-7b                   # Show model details
inferno models info gpt2 --format yaml           # YAML format
inferno models validate bert-base                # Validate model

# Model operations
inferno models download microsoft/DialoGPT-medium # Download model
inferno models load gpt2                         # Load into memory
inferno models unload llama-2-7b                 # Unload from memory
inferno models optimize bert --quantization q4_0  # Optimize model
```

**Subcommands:**
- `list`: List available models
- `info <model>`: Show model information
- `validate <model>`: Validate model integrity
- `download <model>`: Download model from repository
- `load <model>`: Load model into memory
- `unload <model>`: Unload model from memory
- `optimize <model>`: Optimize model performance

### `inferno convert`
Convert models between different formats.

```bash
# Basic conversion
inferno convert model.gguf model.onnx --format onnx
inferno convert model.pt model.gguf --format gguf
inferno convert model.safetensors model.onnx --format onnx

# Advanced conversion
inferno convert model.gguf model.onnx --optimization balanced
inferno convert model.pt model.gguf --quantization q4_0
inferno convert model.onnx model.gguf --precision fp16
inferno convert large-model.pt small-model.gguf --compression
```

**Supported Formats:**
- GGUF (llama.cpp format)
- ONNX (Open Neural Network Exchange)
- PyTorch (.pt, .pth)
- SafeTensors (.safetensors)
- TensorFlow SavedModel

**Options:**
- `--format <fmt>`: Target format (gguf, onnx, pytorch, safetensors)
- `--optimization <level>`: Optimization level (fast, balanced, aggressive)
- `--quantization <type>`: Quantization type (q4_0, q4_1, q5_0, q5_1, q8_0, f16, f32)
- `--precision <prec>`: Precision (fp16, fp32, int8)
- `--compression`: Enable compression for smaller file size

### `inferno validate`
Validate model files and configurations.

```bash
# Validate models
inferno validate llama-2-7b                      # Validate model
inferno validate models/                          # Validate directory
inferno validate --all                           # Validate all models

# Validation options
inferno validate gpt2 --checksum                 # Verify checksums
inferno validate bert --format                   # Validate format
inferno validate mistral --load-test             # Test loading
inferno validate llama --benchmark               # Run benchmark
```

**Options:**
- `--checksum`: Verify file checksums
- `--format`: Validate file format structure
- `--load-test`: Test model loading
- `--benchmark`: Run performance benchmark
- `--fix`: Attempt to fix minor issues
- `--report <file>`: Generate validation report

## Performance & Monitoring Commands

### `inferno bench`
Benchmark model performance and establish baselines.

```bash
# Basic benchmarking
inferno bench --model gpt2                       # Benchmark model
inferno bench --model llama-2-7b --iterations 100
inferno bench --model bert --duration 60s        # Time-based benchmark

# Comprehensive benchmarking
inferno bench --all                              # Benchmark all models
inferno bench --model gpt2 --detailed           # Detailed metrics
inferno bench --model llama --memory            # Memory usage analysis
inferno bench --model mistral --gpu             # GPU utilization

# Output formats
inferno bench --model gpt2 --format json        # JSON output
inferno bench --model bert --output report.txt  # Save to file
inferno bench --model llama --compare baseline.json # Compare to baseline
```

**Options:**
- `--model <name>`: Model to benchmark
- `--iterations <num>`: Number of iterations
- `--duration <time>`: Benchmark duration
- `--detailed`: Include detailed metrics
- `--memory`: Monitor memory usage
- `--gpu`: Monitor GPU utilization
- `--format <fmt>`: Output format (table, json, csv)
- `--output <file>`: Save results to file
- `--compare <file>`: Compare to baseline

### `inferno metrics`
Collect and export performance metrics.

```bash
# View metrics
inferno metrics show                              # Show current metrics
inferno metrics show --model gpt2               # Model-specific metrics
inferno metrics show --format json              # JSON output

# Export metrics
inferno metrics export metrics.json             # Export to file
inferno metrics export --format prometheus      # Prometheus format
inferno metrics export --filter inference       # Filter metric types

# Metrics management
inferno metrics reset                            # Reset all metrics
inferno metrics reset --model bert              # Reset model metrics
inferno metrics enable                           # Enable collection
inferno metrics disable                          # Disable collection
```

**Subcommands:**
- `show`: Display current metrics
- `export <file>`: Export metrics to file
- `reset`: Reset metric counters
- `enable`: Enable metrics collection
- `disable`: Disable metrics collection

### `inferno monitor`
Real-time performance monitoring and alerting.

```bash
# Start monitoring
inferno monitor start                            # Start monitoring
inferno monitor start --interval 5s             # Custom interval
inferno monitor start --dashboard               # Web dashboard

# Configure monitoring
inferno monitor config --cpu-threshold 80       # CPU alert threshold
inferno monitor config --memory-threshold 4GB   # Memory threshold
inferno monitor config --alert-email admin@company.com

# View monitoring status
inferno monitor status                           # Show status
inferno monitor logs                             # View monitoring logs
inferno monitor stop                             # Stop monitoring
```

**Subcommands:**
- `start`: Start monitoring system
- `stop`: Stop monitoring
- `status`: Show monitoring status
- `config`: Configure monitoring settings
- `logs`: View monitoring logs

### `inferno cache`
Manage model and response caching.

```bash
# Cache management
inferno cache status                             # Show cache status
inferno cache clear                              # Clear all cache
inferno cache clear --model gpt2                # Clear model cache
inferno cache clear --responses                 # Clear response cache

# Cache warming
inferno cache warm --model llama-2-7b           # Pre-load model
inferno cache warm --all                        # Pre-load all models
inferno cache warm --popular                    # Pre-load popular models

# Cache configuration
inferno cache config --size 10GB                # Set cache size limit
inferno cache config --compression zstd         # Enable compression
inferno cache config --persist                  # Enable persistence
```

**Subcommands:**
- `status`: Show cache status and statistics
- `clear`: Clear cache contents
- `warm`: Pre-load models into cache
- `config`: Configure cache settings

## Enterprise Features

### `inferno security`
Security and access control management.

```bash
# Initialize security
inferno security init                            # Setup authentication
inferno security init --jwt                     # JWT authentication
inferno security init --ldap                    # LDAP integration

# User management
inferno security user add alice --role admin    # Add user
inferno security user remove bob                # Remove user
inferno security user list                      # List users
inferno security user update alice --role user  # Update user role

# API key management
inferno security key create --name app1         # Create API key
inferno security key revoke key123              # Revoke API key
inferno security key list                       # List API keys

# Security configuration
inferno security config --rate-limit 1000       # Set rate limits
inferno security config --ip-whitelist 10.0.0.0/8 # IP restrictions
inferno security config --audit-all             # Enable full auditing
```

**Subcommands:**
- `init`: Initialize security system
- `user`: User management commands
- `key`: API key management
- `config`: Security configuration

### `inferno audit`
Comprehensive audit logging and compliance.

```bash
# View audit logs
inferno audit logs                               # Show recent logs
inferno audit logs --user alice                 # Filter by user
inferno audit logs --action inference           # Filter by action
inferno audit logs --date 2024-01-01            # Filter by date

# Audit configuration
inferno audit enable                             # Enable audit logging
inferno audit disable                           # Disable audit logging
inferno audit encrypt --key mykey               # Enable encryption
inferno audit compress --algorithm gzip         # Enable compression

# Compliance reports
inferno audit report --type security            # Security report
inferno audit report --type usage               # Usage report
inferno audit export compliance.json            # Export audit data
```

**Subcommands:**
- `logs`: View and filter audit logs
- `enable/disable`: Control audit logging
- `encrypt`: Configure log encryption
- `compress`: Configure log compression
- `report`: Generate compliance reports
- `export`: Export audit data

### `inferno distributed`
Distributed inference with worker pools.

```bash
# Worker management
inferno distributed worker start                # Start worker
inferno distributed worker stop                 # Stop worker
inferno distributed worker list                 # List workers
inferno distributed worker status               # Worker status

# Cluster management
inferno distributed cluster init                # Initialize cluster
inferno distributed cluster join <master-url>   # Join cluster
inferno distributed cluster status              # Cluster status
inferno distributed cluster scale 5             # Scale to 5 workers

# Load balancing
inferno distributed balance round-robin         # Round-robin balancing
inferno distributed balance weighted            # Weighted balancing
inferno distributed balance adaptive            # Adaptive balancing
```

**Subcommands:**
- `worker`: Worker node management
- `cluster`: Cluster management
- `balance`: Load balancing configuration

### `inferno observability`
Comprehensive observability stack.

```bash
# Start observability stack
inferno observability start                     # Start full stack
inferno observability start --prometheus        # Prometheus only
inferno observability start --grafana           # Grafana only
inferno observability start --jaeger            # Jaeger tracing

# Configuration
inferno observability config --retention 30d    # Data retention
inferno observability config --dashboard custom.json # Custom dashboard

# Status and management
inferno observability status                    # Show status
inferno observability stop                      # Stop services
inferno observability restart                   # Restart services
```

**Subcommands:**
- `start`: Start observability services
- `stop`: Stop observability services
- `status`: Show service status
- `config`: Configure observability settings

## Advanced Commands

### `inferno optimization`
Model optimization with quantization and pruning.

```bash
# Quantization
inferno optimization quantize llama-2-7b --type q4_0
inferno optimization quantize bert --type int8  # Integer quantization
inferno optimization quantize gpt2 --auto       # Auto-select best type

# Model pruning
inferno optimization prune large-model --ratio 0.2  # Remove 20%
inferno optimization prune bert --structured     # Structured pruning

# Distillation
inferno optimization distill teacher student    # Knowledge distillation
inferno optimization compress model --algorithm zstd # Model compression
```

### `inferno multimodal`
Multi-modal inference with vision, audio, and text.

```bash
# Image processing
inferno multimodal image --model clip --input image.jpg
inferno multimodal vision --model yolo --input video.mp4

# Audio processing
inferno multimodal audio --model whisper --input audio.wav
inferno multimodal speech --model tts --text "Hello world"

# Mixed modal
inferno multimodal mixed --vision clip --text bert --input image.jpg --prompt "Describe this"
```

### `inferno deployment`
Advanced deployment automation.

```bash
# Kubernetes deployment
inferno deployment k8s deploy --namespace inferno
inferno deployment k8s scale --replicas 5
inferno deployment k8s update --image inferno:v2.0

# Docker deployment
inferno deployment docker build                 # Build image
inferno deployment docker push                  # Push to registry
inferno deployment docker compose up            # Docker Compose

# Cloud deployment
inferno deployment cloud aws --region us-west-2
inferno deployment cloud gcp --project my-project
inferno deployment cloud azure --resource-group rg1
```

### `inferno marketplace`
Model marketplace and registry integration.

```bash
# Browse marketplace
inferno marketplace browse                       # Browse all models
inferno marketplace browse --category nlp       # Browse by category
inferno marketplace search "code generation"    # Search marketplace

# Model publishing
inferno marketplace publish my-model             # Publish model
inferno marketplace unpublish my-model          # Remove from marketplace
inferno marketplace update my-model             # Update published model

# Marketplace management
inferno marketplace config --registry custom.com # Set registry
inferno marketplace auth --token <token>        # Authenticate
```

## Global Options

All commands support these global options:

- `--config <file>`: Use custom configuration file
- `--log-level <level>`: Set logging level (trace, debug, info, warn, error)
- `--log-format <format>`: Set log format (pretty, json, compact)
- `--models-dir <path>`: Override models directory
- `--help`: Show command help
- `--version`: Show version information
- `--verbose`: Enable verbose output
- `--quiet`: Suppress non-error output
- `--dry-run`: Show what would be done without executing

## Environment Variables

Configure Inferno using environment variables:

```bash
export INFERNO_MODELS_DIR="/data/models"        # Models directory
export INFERNO_LOG_LEVEL="info"                 # Logging level
export INFERNO_LOG_FORMAT="json"                # Log format
export INFERNO_CONFIG="/etc/inferno.toml"       # Configuration file
export INFERNO_API_KEY="your-api-key"           # API authentication
export INFERNO_GPU_ENABLED="true"               # Enable GPU acceleration
export INFERNO_CACHE_DIR="/tmp/inferno"         # Cache directory
export INFERNO_PROMETHEUS_PORT="9090"           # Metrics port
```

## Configuration File

Create `inferno.toml` for persistent configuration:

```toml
# Basic settings
models_dir = "/data/models"
log_level = "info"
cache_dir = "/tmp/inferno"

[server]
bind_address = "0.0.0.0"
port = 8080
workers = 4

[backend_config]
gpu_enabled = true
context_size = 4096
batch_size = 64

[cache]
enabled = true
max_size_gb = 10
compression = "zstd"

[security]
auth_enabled = true
rate_limit = 1000
audit_enabled = true

[observability]
prometheus_enabled = true
metrics_port = 9090
tracing_enabled = true
```

## Exit Codes

Inferno uses standard exit codes:

- `0`: Success
- `1`: General error
- `2`: Invalid command or arguments
- `3`: Configuration error
- `4`: Model not found or invalid
- `5`: Network or download error
- `6`: Permission or authentication error
- `7`: Resource exhaustion (memory, disk)
- `8`: GPU or hardware error

## See Also

- [API Reference](api-reference.md) - REST API documentation
- [Configuration Reference](configuration.md) - Complete configuration options
- [Troubleshooting Guide](../guides/troubleshooting.md) - Common issues and solutions
- [Performance Tuning](../guides/performance-tuning.md) - Optimization strategies