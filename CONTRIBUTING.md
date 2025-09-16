# Contributing to Inferno

Thank you for your interest in contributing to Inferno! This document provides guidelines and information for contributors.

## 🤝 Code of Conduct

This project adheres to a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## 🚀 Getting Started

### Prerequisites

- Rust (latest stable version)
- Git
- System dependencies:
  - **Linux**: `libssl-dev`, `pkg-config`
  - **macOS**: Xcode command line tools
  - **Windows**: Visual Studio Build Tools

### Development Setup

1. **Fork and clone the repository:**
   ```bash
   git clone https://github.com/YOUR_USERNAME/inferno.git
   cd inferno
   ```

2. **Install dependencies:**
   ```bash
   # Install Rust if you haven't already
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install additional components
   rustup component add rustfmt clippy
   ```

3. **Build the project:**
   ```bash
   cargo build
   ```

4. **Run tests:**
   ```bash
   cargo test
   ```

5. **Run the application:**
   ```bash
   cargo run -- --help
   ```

## 🔧 Development Workflow

### Branch Strategy

- `main` - Production-ready code
- `develop` - Integration branch for features
- `feature/*` - Feature branches
- `bugfix/*` - Bug fix branches
- `release/*` - Release preparation branches

### Making Changes

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes and commit:**
   ```bash
   git add .
   git commit -m "feat: add new feature description"
   ```

3. **Follow commit message conventions:**
   - `feat:` - New features
   - `fix:` - Bug fixes
   - `docs:` - Documentation changes
   - `style:` - Code style changes
   - `refactor:` - Code refactoring
   - `test:` - Test additions or modifications
   - `chore:` - Maintenance tasks

4. **Push changes and create a pull request:**
   ```bash
   git push origin feature/your-feature-name
   ```

### Code Quality Standards

Before submitting a pull request, ensure your code meets our quality standards:

```bash
# Format code
cargo fmt

# Check for linting issues
cargo clippy -- -D warnings

# Run all tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Check documentation
cargo doc --no-deps --document-private-items

# Security audit
cargo audit
```

### Testing

We maintain comprehensive test coverage:

- **Unit tests**: Test individual functions and modules
- **Integration tests**: Test complete workflows
- **Benchmarks**: Performance testing
- **Example tests**: Ensure examples work correctly

```bash
# Run specific test types
cargo test --lib                    # Unit tests only
cargo test --test integration_tests # Integration tests only
cargo bench                        # Benchmarks
```

## 📝 Documentation

### Code Documentation

- Document all public APIs with rustdoc comments
- Include examples in documentation where helpful
- Keep documentation up-to-date with code changes

```rust
/// Loads a model from the specified path.
///
/// # Arguments
///
/// * `path` - The file path to the model
/// * `config` - Configuration options for loading
///
/// # Returns
///
/// Returns a `Result` containing the loaded model or an error.
///
/// # Examples
///
/// ```rust
/// use inferno::models::ModelManager;
///
/// let manager = ModelManager::new("./models");
/// let model = manager.load_model("llama-7b.gguf").await?;
/// ```
pub async fn load_model(&self, path: &Path, config: &Config) -> Result<Model> {
    // Implementation
}
```

### User Documentation

- Update README.md for user-facing changes
- Add examples for new features
- Update configuration documentation

## 🐛 Bug Reports

When reporting bugs, please include:

1. **Environment information:**
   - Operating system and version
   - Rust version (`rustc --version`)
   - Inferno version

2. **Steps to reproduce:**
   - Minimal example that demonstrates the issue
   - Expected vs. actual behavior

3. **Additional context:**
   - Error messages and stack traces
   - Relevant configuration
   - Model information (if applicable)

Use the bug report template when creating issues.

## 💡 Feature Requests

For feature requests, please provide:

1. **Use case description:**
   - What problem does this solve?
   - Who would benefit from this feature?

2. **Proposed solution:**
   - How should the feature work?
   - API design considerations

3. **Alternatives considered:**
   - Other approaches you've thought about
   - Why this approach is preferred

## 🏗️ Architecture Overview

### Core Components

- **CLI (`src/cli/`)**: Command-line interface implementation
- **TUI (`src/tui/`)**: Terminal user interface
- **Backends (`src/backends/`)**: Model execution backends (GGUF, ONNX)
- **Models (`src/models/`)**: Model management and metadata
- **Config (`src/config.rs`)**: Configuration management
- **I/O (`src/io/`)**: Input/output format handling
- **Metrics (`src/metrics/`)**: Performance monitoring

### Design Principles

1. **Performance**: Prioritize speed and efficiency
2. **Safety**: Use Rust's type system to prevent bugs
3. **Modularity**: Keep components loosely coupled
4. **Testability**: Design for easy testing
5. **Documentation**: Maintain clear, helpful documentation

### Adding New Features

When adding new features:

1. **Design first**: Consider the API and user experience
2. **Start small**: Implement a minimal viable version
3. **Add tests**: Ensure comprehensive test coverage
4. **Document**: Update relevant documentation
5. **Consider backwards compatibility**: Avoid breaking changes

### Backend Implementation

To add a new model backend:

1. **Implement the `InferenceBackend` trait:**
   ```rust
   #[async_trait::async_trait]
   impl InferenceBackend for NewBackend {
       async fn load_model(&mut self, model_info: &ModelInfo) -> Result<()> {
           // Implementation
       }

       async fn infer(&mut self, input: &str, params: &InferenceParams) -> Result<String> {
           // Implementation
       }

       // Other required methods...
   }
   ```

2. **Add backend type to enum:**
   ```rust
   #[derive(Debug, Clone, Copy, ValueEnum)]
   pub enum BackendType {
       Gguf,
       Onnx,
       NewBackend, // Add here
   }
   ```

3. **Update backend creation:**
   ```rust
   pub fn new(backend_type: BackendType, config: &BackendConfig) -> Result<Self> {
       let backend_impl: Box<dyn InferenceBackend> = match backend_type {
           BackendType::Gguf => Box::new(gguf::GgufBackend::new(config.clone())?),
           BackendType::Onnx => Box::new(onnx::OnnxBackend::new(config.clone())?),
           BackendType::NewBackend => Box::new(new::NewBackend::new(config.clone())?),
       };
       Ok(Self { backend_impl })
   }
   ```

4. **Add comprehensive tests**
5. **Update documentation**

## 🔄 Release Process

Releases follow semantic versioning (SemVer):

- **Major** (X.0.0): Breaking changes
- **Minor** (0.X.0): New features, backwards compatible
- **Patch** (0.0.X): Bug fixes, backwards compatible

### Release Checklist

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run full test suite
4. Create release PR to `main`
5. Tag release after merge
6. GitHub Actions will build and publish artifacts

## 🌍 Internationalization

While not currently implemented, we plan to support internationalization:

- Use `fluent` for message localization
- Extract user-facing strings to resource files
- Support major languages (English, Spanish, French, German, Chinese, Japanese)

## ⚡ Performance Guidelines

### Memory Management

- Use `Arc` and `Rc` judiciously for shared ownership
- Prefer borrowing over cloning when possible
- Be mindful of large model loading and memory usage

### Async Programming

- Use `async/await` for I/O operations
- Avoid blocking in async contexts
- Use `tokio::spawn` for CPU-intensive tasks

### Error Handling

- Use `Result` types consistently
- Provide meaningful error messages
- Use `anyhow` for application errors, `thiserror` for library errors

## 🧪 Testing Guidelines

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Test implementation
    }

    #[tokio::test]
    async fn test_async_function() {
        // Async test implementation
    }
}
```

### Integration Tests

Place integration tests in `tests/` directory:

```rust
// tests/integration_tests.rs
use inferno::*;

#[tokio::test]
async fn test_full_workflow() {
    // Test complete user workflows
}
```

### Benchmarks

Use Criterion for benchmarks:

```rust
// benches/my_benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("function_name", |b| {
        b.iter(|| {
            // Code to benchmark
        })
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

## 📞 Getting Help

If you need help:

1. Check existing [documentation](README.md)
2. Search [existing issues](https://github.com/inferno-ai/inferno/issues)
3. Join our [discussions](https://github.com/inferno-ai/inferno/discussions)
4. Ask questions in issues (use the "question" label)

## 🎉 Recognition

Contributors will be recognized in:

- `CONTRIBUTORS.md` file
- Release notes for significant contributions
- Special thanks in documentation

Thank you for contributing to Inferno! 🔥