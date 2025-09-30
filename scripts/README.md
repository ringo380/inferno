# Inferno Build Scripts

This directory contains build scripts for different Inferno configurations.

## Scripts Overview

### `build-desktop.sh` - Desktop Application Builder (v0.5.0+)

**NEW in v0.5.0**: Builds the Tauri v2 desktop application with full macOS optimizations.

```bash
# Development build (fast, includes debug symbols)
./scripts/build-desktop.sh --dev

# Release build (optimized, production-ready)
./scripts/build-desktop.sh --release

# Universal binary (ARM64 + x86_64)
./scripts/build-desktop.sh --release --universal

# Clean build (removes all artifacts first)
./scripts/build-desktop.sh --clean --release

# Skip frontend rebuild (faster iteration)
./scripts/build-desktop.sh --dev --skip-frontend

# Verbose output (see all compilation details)
./scripts/build-desktop.sh --release --verbose
```

**Output locations:**
- **DMG**: `dashboard/src-tauri/target/release/bundle/dmg/`
- **App Bundle**: `dashboard/src-tauri/target/release/bundle/macos/Inferno.app`
- **Debug**: `dashboard/src-tauri/target/debug/bundle/macos/Inferno.app`

**Features:**
- ✅ Apple Silicon (M1/M2/M3/M4) optimizations
- ✅ Metal GPU acceleration
- ✅ Tauri v2 with all plugins
- ✅ GGUF backend included
- ✅ Native macOS UI with vibrancy effects
- ✅ System tray integration
- ✅ Auto-updates support

### `build.sh` - Core Library Builder

Builds the Inferno core library and CLI.

```bash
# Development build
./scripts/build.sh

# Release build with all backends
./scripts/build.sh --release

# With specific backends
./scripts/build.sh --release --features gguf,onnx
```

### `build-universal.sh` - Universal Binary Builder

Creates universal binaries for CLI distribution (ARM64 + x86_64).

```bash
# Build universal CLI binary
./scripts/build-universal.sh

# The desktop app uses this automatically with --universal flag
```

### `benchmark.sh` - Performance Benchmarking

Runs performance benchmarks for the inference engine.

```bash
# Run all benchmarks
./scripts/benchmark.sh

# Run specific benchmark
./scripts/benchmark.sh inference
```

## Quick Start

### For Desktop Development

```bash
# First time setup
cd dashboard
npm install

# Development with hot reload
cd dashboard && npm run tauri dev

# Build for distribution
./scripts/build-desktop.sh --release --universal
```

### For CLI Development

```bash
# Build CLI
cargo build --release

# Or use the script
./scripts/build.sh --release --features ml-backends
```

## Build Features

### Available Cargo Features

- `desktop` - NEW: Tauri v2 desktop with full features (v0.5.0+)
- `tauri-app` - DEPRECATED: Tauri v1 app (use `desktop` instead)
- `gguf` - GGUF/llama.cpp backend
- `onnx` - ONNX Runtime backend
- `ml-backends` - All ML backends (gguf + onnx)
- `gpu-metal` - Metal GPU acceleration (macOS)
- `cuda` - NVIDIA CUDA support
- `rocm` - AMD ROCm support

### Example Build Commands

```bash
# Desktop app with GGUF support
cd dashboard && npm run tauri build

# CLI with all backends
cargo build --release --features ml-backends

# Library only (no binaries)
cargo build --release --lib

# With Metal GPU acceleration
cargo build --release --features "gguf,gpu-metal"
```

## Platform-Specific Notes

### macOS

- **Apple Silicon**: Automatic M1/M2/M3/M4 optimizations via `.cargo/config.toml`
- **Metal Framework**: Linked automatically for GPU acceleration
- **Code Signing**: Configure in `dashboard/src-tauri/tauri.conf.json`
- **Notarization**: Set up Apple Developer credentials for distribution

### Linux (Future)

Desktop builds will support Linux once Tauri v2 Linux support is added.

### Windows (Future)

Desktop builds will support Windows once Windows-specific features are implemented.

## Troubleshooting

### "Tauri not found"

```bash
# Install Tauri CLI
cd dashboard
npm install
```

### "Rust compiler not found"

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### "Frontend build failed"

```bash
# Rebuild frontend
cd dashboard
rm -rf node_modules .next dist
npm install
npm run build
```

### "Linker errors on macOS"

```bash
# Install Xcode Command Line Tools
xcode-select --install
```

### "Universal build fails"

Make sure you have both targets installed:
```bash
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin
```

## CI/CD Integration

The build scripts are designed to work in CI/CD environments:

```yaml
# GitHub Actions example
- name: Build Desktop App
  run: ./scripts/build-desktop.sh --release --universal

# GitLab CI example
build-desktop:
  script:
    - ./scripts/build-desktop.sh --release --universal
  artifacts:
    paths:
      - dashboard/src-tauri/target/release/bundle/
```

## Performance Tips

1. **Use `--skip-frontend`** during Rust-only development
2. **Use `--dev`** for faster iteration (no optimizations)
3. **Enable `sccache`** for faster recompilation
4. **Use `cargo watch`** for automatic rebuilds during development

## Version History

- **v0.5.0** - Added `build-desktop.sh` for Tauri v2, deprecated Tauri v1
- **v0.4.0** - Added `build-universal.sh` for macOS universal binaries
- **v0.3.0** - Initial build scripts for CLI and Tauri v1

## See Also

- [CLAUDE.md](../CLAUDE.md) - Development workflow and commands
- [README.md](../README.md) - Project overview
- [Cargo.toml](../Cargo.toml) - Dependency configuration
- [.cargo/config.toml](../.cargo/config.toml) - Platform-specific compiler flags