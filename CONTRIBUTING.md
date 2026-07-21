# Contributing to Inferno

Thank you for your interest in contributing to Inferno! This guide will help you get started.

## 🤝 How to Contribute

### Reporting Issues
- Use the [issue tracker](https://github.com/ringo380/inferno/issues) to report bugs
- Search existing issues before creating a new one
- Use the issue templates for bug reports and feature requests
- Provide as much detail as possible, including:
  - Operating system and version
  - Rust version (`rustc --version`)
  - Command that caused the issue
  - Expected vs actual behavior
  - Log output (with `RUST_LOG=debug`)

### Feature Requests
- Open a [discussion](https://github.com/ringo380/inferno/discussions) first for major features
- Use the feature request issue template
- Explain the use case and why it would benefit Inferno users
- Consider if it fits with Inferno's goal of being a local AI inference platform

### Pull Requests
1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Implement** your changes with tests
4. **Run** the full test suite: `./verify.sh`
5. **Commit** with a clear message
6. **Push** and create a Pull Request

## 🛠️ Development Setup

### Prerequisites
- **Rust 1.85+**: Install via [rustup](https://rustup.rs/)
- **Git**: For version control
- **Docker** (optional): For testing containerized deployments

### Clone and Build
```bash
git clone https://github.com/ringo380/inferno.git
cd inferno

# Build in development mode
cargo build

# Run tests
cargo test

# Run full verification (build + test + lint + audit)
./verify.sh
```

### Development Dependencies
```bash
# Install development tools
cargo install cargo-watch    # Auto-rebuild on changes
cargo install cargo-audit    # Security audits
cargo install cargo-tarpaulin # Code coverage

# Optional: Install pre-commit hooks
cargo install pre-commit
pre-commit install
```

## 📝 Code Guidelines

### Code Style
- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common issues
- Follow Rust naming conventions
- Write self-documenting code with clear variable names
- Add comments for complex logic or algorithms

### Error Handling
- Use `anyhow::Result` for application errors
- Use `thiserror` for library errors
- Provide helpful error messages with context
- Don't panic in library code (use `Result` instead)

### Testing
- Write unit tests for all public functions
- Add integration tests for complex workflows
- Use `#[cfg(test)]` for test-only code
- Mock external dependencies in tests
- Aim for >80% code coverage

### Documentation
- Document all public APIs with rustdoc comments
- Include examples in documentation
- Update relevant documentation files
- Add entries to CHANGELOG.md for user-facing changes

## 🏗️ Project Structure

```
inferno/
├── src/
│   ├── backends/           # AI model backends (GGUF, ONNX)
│   ├── api/               # HTTP and WebSocket APIs
│   ├── cli/               # Command-line interface
│   ├── tui/               # Terminal user interface
│   ├── cache.rs           # Caching system
│   ├── config.rs          # Configuration management
│   └── lib.rs             # Library entry point
├── tests/                 # Integration tests
├── examples/              # Usage examples
├── docs/                  # Additional documentation
└── scripts/               # Build and deployment scripts
```

## 🧪 Testing

### Running Tests
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests

# Specific test
cargo test test_gguf_backend

# With logging
RUST_LOG=debug cargo test -- --nocapture
```

### Test Categories
- **Unit Tests**: Fast, isolated tests for individual functions
- **Integration Tests**: Test component interactions
- **Performance Tests**: Benchmark critical paths
- **End-to-End Tests**: Full workflow testing

### Writing Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_model_loading() {
        let temp_dir = TempDir::new().unwrap();
        let model_manager = ModelManager::new(temp_dir.path());

        // Test implementation
        assert!(model_manager.list_models().await.is_ok());
    }
}
```

## 🎯 Areas for Contribution

### High Priority
- **Backend Improvements**: Enhance GGUF/ONNX implementations
- **Performance**: Optimize inference speed and memory usage
- **Documentation**: Improve guides and API documentation
- **Testing**: Increase test coverage and add edge cases

### Medium Priority
- **New Model Formats**: Add support for additional formats
- **Platform Support**: Improve Windows/macOS compatibility
- **Monitoring**: Enhance metrics and observability
- **Security**: Strengthen authentication and authorization

### Low Priority
- **UI/UX**: Improve CLI and TUI interfaces
- **Examples**: Add more usage examples
- **Integrations**: Add client libraries for other languages
- **Deployment**: Docker, Kubernetes, cloud deployment guides

## 📋 Commit Guidelines

### Commit Message Format
```
type(scope): brief description

Longer description explaining the change and why it was made.

Fixes #123
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Examples
```
feat(backends): add GPU memory optimization for GGUF models

Implements dynamic memory allocation that reduces GPU memory usage
by 30% for large models while maintaining inference speed.

Fixes #456

---

fix(cache): resolve race condition in concurrent cache access

The cache was not properly handling concurrent reads and writes,
leading to occasional panics. Added proper synchronization using
Arc<RwLock<>> pattern.

Fixes #789
```

## 🔍 Code Review Process

### For Contributors
- Keep PRs focused and reasonably sized
- Write clear PR descriptions explaining the change
- Respond to feedback promptly and constructively
- Update documentation and tests as needed

### For Reviewers
- Be constructive and helpful in feedback
- Focus on code correctness, performance, and maintainability
- Check that tests adequately cover the changes
- Verify documentation is updated

## 🚀 Release Process

### Versioning
- Follow [Semantic Versioning](https://semver.org/)
- Major: Breaking changes
- Minor: New features (backward compatible)
- Patch: Bug fixes (backward compatible)

### Release Checklist
1. Update CHANGELOG.md
2. Update version in Cargo.toml
3. Run full test suite
4. Create release PR
5. Tag release after merge
6. Publish to crates.io
7. Update Docker images

## 🏷️ Labels

We use these labels to organize issues and PRs:

- `bug`: Something isn't working
- `enhancement`: New feature or improvement
- `documentation`: Documentation needs
- `good first issue`: Good for newcomers
- `help wanted`: Extra attention needed
- `performance`: Performance improvements
- `security`: Security-related changes

## 🤔 Questions?

- **GitHub Discussions**: [Community help and questions](https://github.com/ringo380/inferno/discussions)
- **Issues**: [Report bugs or request features](https://github.com/ringo380/inferno/issues)

Thank you for contributing to Inferno! 🔥