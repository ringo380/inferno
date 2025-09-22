# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Development Commands

### Building and Testing
- `cargo build` - Debug build
- `cargo build --release` - Release build with optimizations
- `cargo test` - Run unit and integration tests
- `cargo test --test integration_tests` - Run only integration tests
- `cargo test --test component_unit_tests` - Run component unit tests
- `cargo test --test feature_integration_tests` - Run feature integration tests
- `cargo test --test end_to_end_tests` - Run end-to-end tests
- `cargo bench` - Run performance benchmarks
- `cargo clippy` - Run linter
- `cargo fmt` - Format code
- `./scripts/build.sh --release` - Build with deployment script
- `./verify.sh` - Full verification script (build + test + lint + security audit)

### Running the Application
- `cargo run -- --help` - Show CLI help
- `cargo run -- tui` - Launch terminal UI
- `cargo run -- models list` - List available models
- `cargo run -- run --model MODEL --prompt "text"` - Run inference
- `cargo run -- serve` - Start HTTP API server
- `cargo run -- bench --model MODEL` - Benchmark model performance
- `cargo run -- validate MODEL_FILE` - Validate model file
- `cargo run -- convert model input.gguf output.onnx --format onnx` - Convert models
- `cargo run -- cache persist --compress gzip` - Enable persistent caching
- `cargo run -- audit enable --encryption` - Enable encrypted audit logs
- `cargo run -- batch-queue create --schedule "0 2 * * *"` - Create scheduled batch jobs

### Development Tools
- `./bootstrap.sh` - Bootstrap new project from scratch
- `cargo watch -x check` - Watch for changes and check compilation
- `INFERNO_MODELS_DIR="test_models" cargo run -- models list` - Use test models directory

## Architecture Overview

Inferno is an enterprise-grade offline AI/ML model runner built with a comprehensive, modular architecture supporting production deployment:

```
src/
├── main.rs           # CLI entry point with clap argument parsing
├── lib.rs            # Library exports and comprehensive error types
├── config.rs         # Hierarchical configuration (TOML + env vars)
├── backends/         # Trait-based model execution backends
│   ├── mod.rs        # InferenceBackend trait definition
│   ├── gguf.rs       # GGUF backend (ready for llama.cpp integration)
│   └── onnx.rs       # ONNX backend (ready for ort crate integration)
├── cli/              # Comprehensive command-line interface modules
│   ├── run.rs        # Inference execution
│   ├── serve.rs      # HTTP API server
│   ├── models.rs     # Model management
│   ├── bench.rs      # Performance benchmarking
│   ├── validate.rs   # Model validation
│   ├── batch.rs      # Batch processing
│   ├── metrics.rs    # Metrics management
│   ├── config.rs     # Configuration management
│   ├── cache.rs      # Model caching
│   ├── convert.rs    # Model format conversion
│   ├── response_cache.rs # Response caching
│   ├── monitoring.rs # Performance monitoring
│   ├── distributed.rs # Distributed inference
│   ├── ab_testing.rs # A/B testing framework
│   ├── audit.rs      # Audit logging
│   ├── batch_queue.rs # Batch queue management
│   ├── versioning.rs # Model versioning
│   ├── gpu.rs        # GPU management
│   ├── resilience.rs # Resilience patterns
│   ├── streaming.rs  # Real-time streaming
│   ├── security.rs   # Security management
│   ├── observability.rs # Observability stack
│   ├── optimization.rs # Performance optimization
│   ├── multimodal.rs # Multimodal support
│   ├── deployment.rs # Deployment automation
│   ├── marketplace.rs # Model marketplace
│   ├── federated.rs  # Federated learning
│   ├── dashboard.rs  # Web dashboard
│   ├── advanced_monitoring.rs # Advanced monitoring
│   ├── api_gateway.rs # API gateway
│   ├── model_versioning.rs # Model version control
│   ├── data_pipeline.rs # Data pipeline management
│   ├── backup_recovery.rs # Backup and recovery
│   ├── logging_audit.rs # Enhanced logging
│   ├── performance_optimization.rs # Performance tuning
│   ├── multi_tenancy.rs # Multi-tenant support
│   ├── advanced_cache.rs # Advanced caching
│   └── qa_framework.rs # Quality assurance
├── tui/              # Terminal user interface
│   ├── app.rs        # Main TUI application state
│   ├── components.rs # Reusable UI components
│   └── events.rs     # Event handling system
├── api/              # HTTP API modules
│   ├── mod.rs        # API module exports
│   ├── openai.rs     # OpenAI-compatible API
│   └── websocket.rs  # WebSocket real-time API
├── batch/            # Batch processing system
│   ├── mod.rs        # Batch processing core
│   ├── queue.rs      # Job queue management
│   └── scheduler.rs  # Task scheduling
├── models/           # Model discovery and metadata
├── io/               # I/O format handling (text, image, audio, JSON)
├── metrics/          # Performance monitoring with async event processing
└── [Enterprise Modules] # Advanced features for production deployment
```

### Key Design Patterns
- **Backend Trait**: `InferenceBackend` trait allows pluggable model execution engines
- **Async-first**: All I/O operations use tokio async runtime
- **Configuration cascade**: Config files < environment variables < CLI arguments
- **Error handling**: Uses anyhow for application errors, thiserror for library errors
- **Security**: Sandboxed execution, checksum verification, file type validation
- **Modularity**: Each feature is self-contained with clear interfaces
- **Enterprise-ready**: Comprehensive logging, monitoring, and management features

### Backend Implementation
The backend system uses a trait-based approach for pluggable model execution:

```rust
#[async_trait::async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn load_model(&mut self, model_info: &ModelInfo) -> Result<()>;
    async fn unload_model(&mut self) -> Result<()>;
    async fn is_loaded(&self) -> bool;
    async fn get_model_info(&self) -> Option<ModelInfo>;
    async fn infer(&mut self, input: &str, params: &InferenceParams) -> Result<String>;
    async fn infer_stream(&mut self, input: &str, params: &InferenceParams) -> Result<TokenStream>;
    async fn get_embeddings(&mut self, input: &str) -> Result<Vec<f32>>;
    fn get_backend_type(&self) -> BackendType;
    fn get_metrics(&self) -> Option<InferenceMetrics>;
}
```

## Configuration Management

### Configuration Hierarchy (highest to lowest priority)
1. CLI arguments
2. Environment variables (prefixed with `INFERNO_`)
3. Local project config (`.inferno.toml`)
4. User config (`~/.inferno.toml`)
5. Global config (`~/.config/inferno/config.toml`)
6. Default values

### Key Configuration Sections
- `models_dir`: Where to find model files
- `backend_config`: GPU settings, context size, batch size
- `server`: HTTP API server configuration
- `security`: Sandboxing, file validation, size limits
- `observability`: Monitoring and tracing configuration
- `distributed`: Cluster and worker configuration
- `cache`: Caching strategies and limits

### Environment Variables
- `INFERNO_MODELS_DIR` - Override models directory
- `INFERNO_LOG_LEVEL` - Set logging level (trace, debug, info, warn, error)
- `INFERNO_LOG_FORMAT` - Set log format (pretty, json, compact)

## Development Workflow

### Adding New Features
1. Create feature branch from `develop`
2. Implement with comprehensive tests
3. Update documentation and examples
4. Run full verification: `./verify.sh`
5. Create PR with clear description

### Adding New CLI Commands
1. Create new module in `src/cli/`
2. Add command struct with clap derives
3. Add to `Commands` enum in `src/cli/mod.rs`
4. Add execute function with proper error handling
5. Add command match case in `src/main.rs`
6. Add comprehensive tests and help documentation

### Adding New Backend
1. Implement `InferenceBackend` trait in `src/backends/new_backend.rs`
2. Add to `BackendType` enum in `src/backends/mod.rs`
3. Update `Backend::new()` constructor to include new backend
4. Add file extension detection in `BackendType::from_model_path()`
5. Add comprehensive tests and benchmarks
6. Update documentation and CLI help

## Testing Strategy

### Test Organization
- **Unit tests**: In-module `#[cfg(test)]` blocks for individual functions
- **Integration tests**: `tests/integration_tests.rs` for end-to-end workflows
- **Component tests**: `tests/component_unit_tests.rs` for component-level testing
- **Feature tests**: `tests/feature_integration_tests.rs` for feature integration
- **End-to-end tests**: `tests/end_to_end_tests.rs` for complete workflow testing
- **Benchmarks**: `benches/inference_benchmark.rs` for performance testing
- **Examples**: `examples/` directory with runnable examples

### Testing Commands
- `cargo test` - Run all tests
- `cargo test --test integration_tests` - Integration tests only
- `cargo test --test component_unit_tests` - Component tests only
- `INFERNO_MODELS_DIR="test_models" cargo test` - Test with specific model directory

## Performance Considerations

### Memory Management
- **On-demand loading** with intelligent memory management
- **Thread-safe backend cloning** via `BackendHandle` architecture
- **Stream processing** for large outputs to avoid memory spikes
- **Atomic metrics** collection with zero-copy operations
- **Memory mapping** for efficient model file access
- **GPU memory management** with configurable limits
- **Cache eviction** policies to prevent OOM conditions
- **Resource monitoring** with automatic cleanup

### Async Best Practices
- All backends implement async traits using `#[async_trait::async_trait]`
- Use tokio::spawn for CPU-intensive model operations
- Metrics system uses mpsc channels for async event processing
- Stream inference returns `Pin<Box<dyn Stream<Item = Result<String>>>>` for real-time output

## Security Guidelines

### Model Safety
- Checksum verification for model integrity
- File type validation before loading
- Size limits to prevent resource exhaustion
- Sandboxed execution environments

### Input Validation
- Sanitize all user inputs
- Validate file paths to prevent directory traversal
- Rate limiting for API endpoints
- Timeout enforcement for long-running operations

## Debugging and Troubleshooting

### Logging
- Set `INFERNO_LOG_LEVEL=debug` for verbose output
- Use `INFERNO_LOG_FORMAT=json` for structured logs
- TUI shows real-time logs in the left panel
- Audit logs available through `inferno audit` commands

### Common Issues
- **Model not found**: Check models directory and file permissions
- **GPU not detected**: Verify platform-specific GPU libraries installed
- **Out of memory**: Reduce context_size or batch_size in config
- **Slow inference**: Check if GPU acceleration is enabled
- **Permission denied**: Check file permissions and security settings
- **GGUF loading failed**: Ensure file has valid GGUF magic bytes
- **ONNX Runtime error**: Check execution provider compatibility