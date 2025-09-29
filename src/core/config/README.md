# Inferno Configuration System

**Version**: 0.4.0+
**Status**: Stable
**Migration**: Fully backward compatible

## Overview

The Inferno configuration system provides a modern, type-safe approach to managing application settings. It features:

- **Builder Pattern**: Fluent API for constructing configurations
- **Presets**: Predefined profiles for common scenarios
- **Type Safety**: Enums instead of strings for common values
- **Validation**: Comprehensive validation with clear error messages
- **Backward Compatibility**: Old `Config::load()` still works

## Quick Start

### Simple Configuration

```rust
use inferno::core::config::ConfigBuilder;

let config = ConfigBuilder::new()
    .models_dir("./models")
    .build()?;
```

### Using Presets

```rust
use inferno::core::config::{ConfigBuilder, Preset};

// Development preset
let config = ConfigBuilder::new()
    .preset(Preset::Development)
    .build()?;

// Production preset
let config = ConfigBuilder::new()
    .preset(Preset::Production)
    .build()?;
```

### Customizing Presets

```rust
use inferno::core::config::{ConfigBuilder, Preset, LogLevel};

let config = ConfigBuilder::new()
    .preset(Preset::Production)
    .models_dir("./production-models")
    .log_level(LogLevel::Warn)
    .build()?;
```

## Available Presets

### Development
- **Purpose**: Local development with fast startup
- **Logging**: Debug level, pretty format
- **Features**: Minimal, fast compilation
- **Use When**: Developing locally, debugging

### Production
- **Purpose**: Optimized for deployment
- **Logging**: Info level, JSON format
- **Features**: Full monitoring, security enabled
- **Use When**: Deploying to production

### Testing
- **Purpose**: Deterministic test behavior
- **Logging**: Error level only, compact format
- **Features**: All disabled for fast execution
- **Use When**: Running automated tests

### Benchmark
- **Purpose**: Performance measurement
- **Logging**: Warn level only
- **Features**: All monitoring disabled
- **Use When**: Running benchmarks

## Configuration Types

### LogLevel

Type-safe logging levels:

```rust
use inferno::core::config::LogLevel;

let config = ConfigBuilder::new()
    .log_level(LogLevel::Debug)  // trace, debug, info, warn, error
    .build()?;
```

### LogFormat

Type-safe log formats:

```rust
use inferno::core::config::LogFormat;

let config = ConfigBuilder::new()
    .log_format(LogFormat::Json)  // pretty, compact, json
    .build()?;
```

## Builder API

### Core Settings

```rust
ConfigBuilder::new()
    .models_dir("./models")          // Where model files are stored
    .cache_dir("./cache")            // Where cache files are stored
    .log_level(LogLevel::Info)       // Logging verbosity
    .log_format(LogFormat::Pretty)   // Log output format
    .build()?;
```

### Validation

The builder automatically:
1. Validates all settings
2. Creates required directories
3. Returns clear error messages if validation fails

```rust
let config = ConfigBuilder::new()
    .models_dir("/path/to/models")
    .build()?;  // Will create /path/to/models if it doesn't exist
```

### Unchecked Building

For testing only, you can skip validation:

```rust
let config = ConfigBuilder::new()
    .models_dir("./test-models")
    .build_unchecked();  // ⚠️  No validation, no directory creation
```

## Migration Guide

### From Old System

**Old way (still works)**:
```rust
use inferno::config::Config;

let config = Config::load()?;  // Loads from files + env
```

**New way (recommended)**:
```rust
use inferno::core::config::{ConfigBuilder, Preset};

let config = ConfigBuilder::new()
    .preset(Preset::Development)
    .build()?;
```

### Gradual Migration

You can use both systems during migration:

```rust
// Old system for complex configs
let old_config = inferno::config::Config::load()?;

// New system for simple configs
let new_config = inferno::core::config::ConfigBuilder::new()
    .preset(Preset::Development)
    .build()?;
```

## Examples

### Development Environment

```rust
use inferno::core::config::{ConfigBuilder, Preset, LogLevel};

let config = ConfigBuilder::new()
    .preset(Preset::Development)
    .models_dir("./models")
    .log_level(LogLevel::Trace)  // Override preset
    .build()?;
```

### Production Environment

```rust
use inferno::core::config::{ConfigBuilder, Preset};

let config = ConfigBuilder::new()
    .preset(Preset::Production)
    .models_dir("/var/lib/inferno/models")
    .cache_dir("/var/cache/inferno")
    .build()?;
```

### Testing Environment

```rust
use inferno::core::config::{ConfigBuilder, Preset};
use tempfile::tempdir;

let temp_dir = tempdir()?;
let config = ConfigBuilder::new()
    .preset(Preset::Testing)
    .models_dir(temp_dir.path().join("models"))
    .build()?;
```

### Benchmark Environment

```rust
use inferno::core::config::{ConfigBuilder, Preset};

let config = ConfigBuilder::new()
    .preset(Preset::Benchmark)
    .models_dir("./benchmark-models")
    .build()?;
```

## Best Practices

### 1. Use Presets as Starting Points

Start with a preset and customize:

```rust
ConfigBuilder::new()
    .preset(Preset::Production)
    .models_dir("./custom-path")
    .build()?
```

### 2. Type-Safe Values

Use enums instead of strings:

```rust
// Good ✅
.log_level(LogLevel::Debug)

// Avoid ❌ (old system)
log_level: "debug".to_string()
```

### 3. Let Builder Handle Validation

Trust the builder to validate:

```rust
// Builder will validate and create directories
let config = builder.build()?;

// Don't manually validate
config.validate()?;  // ❌ Not needed with builder
```

### 4. Environment-Specific Configs

Use presets for environment detection:

```rust
let preset = if cfg!(debug_assertions) {
    Preset::Development
} else {
    Preset::Production
};

let config = ConfigBuilder::new()
    .preset(preset)
    .build()?;
```

## Advanced Usage

### Custom Validation

```rust
let config = ConfigBuilder::new()
    .preset(Preset::Production)
    .build()?;

// Add custom validation
if !config.models_dir.join("required-model.gguf").exists() {
    return Err(anyhow!("Required model not found"));
}
```

### Path Helpers

```rust
let config = builder.build()?;

// Get full path for a model
let model_path = config.model_path("llama-2-7b.gguf");

// Get full path for cache file
let cache_path = config.cache_path("embeddings");
```

## Future Enhancements

Planned improvements for future versions:

- **Feature Configs**: Modular configs for infrastructure, operations, etc.
- **Config Files**: Load builder settings from TOML/JSON
- **Environment Variables**: Builder support for env var overrides
- **Config Profiles**: Save and load named configurations
- **Validation Rules**: Custom validation rules per preset

## Troubleshooting

### Directory Creation Fails

```rust
// Error: Permission denied
let config = ConfigBuilder::new()
    .models_dir("/root/models")  // ❌ No permission
    .build()?;

// Solution: Use accessible directory
let config = ConfigBuilder::new()
    .models_dir("./models")  // ✅ Relative to working dir
    .build()?;
```

### Validation Errors

The builder provides clear error messages:

```rust
let result = ConfigBuilder::new()
    .models_dir("")  // ❌ Empty path
    .build();

// Error message will explain what's wrong
assert!(result.is_err());
```

## API Reference

See module documentation for complete API details:

- `ConfigBuilder`: Builder for constructing configurations
- `CoreConfig`: Core configuration struct
- `Preset`: Predefined configuration profiles
- `LogLevel`: Type-safe log levels
- `LogFormat`: Type-safe log formats