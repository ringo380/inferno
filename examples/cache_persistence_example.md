# Cache Persistence Example

This example demonstrates how to use the cache persistence functionality in Inferno.

## Configuration

```toml
# .inferno.toml
[cache]
persist_cache = true
cache_dir = "~/.cache/inferno"
max_cached_models = 10
model_ttl_seconds = 3600
enable_warmup = true
```

## Usage

```rust
use inferno::cache::{CacheConfig, ModelCache};
use inferno::backends::BackendConfig;
use inferno::models::ModelManager;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure cache with persistence
    let mut cache_config = CacheConfig::default();
    cache_config.persist_cache = true;
    cache_config.cache_dir = Some(PathBuf::from("~/.cache/inferno"));

    let backend_config = BackendConfig::default();
    let model_manager = Arc::new(ModelManager::new(&PathBuf::from("./models")));

    // Create cache - it will automatically load from disk if available
    let cache = ModelCache::new(
        cache_config,
        backend_config,
        model_manager,
        None
    ).await?;

    // Use the cache
    let model = cache.get_model("my_model.gguf").await?;

    // Cache automatically saves periodically and on shutdown
    // You can also manually trigger a save:
    cache.save_cache().await?;

    Ok(())
}
```

## Cache Files

The cache system creates the following files:

- `cache_state.bin.zst` - Compressed cache state including:
  - Model metadata and usage statistics
  - Cache hit/miss statistics
  - Model warmup priorities
  - Usage frequency data

## Features

### Automatic Persistence
- Cache state is automatically loaded on startup
- Periodic saves every 5 minutes
- Save on shutdown
- Atomic file operations to prevent corruption

### Compression
- Uses zstd compression (level 3) for space efficiency
- Typically achieves 60-80% size reduction
- Fast compression/decompression

### Error Handling
- Gracefully handles missing or corrupt cache files
- Falls back to empty cache state on errors
- Continues operation without disk persistence if directory is unavailable

### Model Validation
- Validates model files still exist before restoration
- Checks file sizes for integrity
- Only restores recently used models (24-hour TTL)

## CLI Usage

```bash
# Enable cache persistence
inferno config set cache.persist_cache true
inferno config set cache.cache_dir ~/.cache/inferno

# View cache statistics
inferno cache stats

# Manually save cache
inferno cache save

# Clear cache
inferno cache clear
```