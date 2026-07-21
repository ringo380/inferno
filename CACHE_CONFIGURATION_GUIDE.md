# Cache Configuration Guide

This guide covers Inferno's model cache: an in-memory cache of loaded models with
memory- and time-based eviction, optional warm-up, and optional persistence of the
cache state to disk.

## Overview

The cache keeps recently used models loaded in memory so subsequent inference
requests skip the load step. It provides:

- **In-memory model cache** with a configurable model count and memory ceiling
- **Eviction** by TTL (unused models are dropped) and by memory pressure
- **Warm-up** strategies to preload models before they are needed
- **Optional disk persistence** of the cache state (off by default)

Response-level caching is handled separately by the `inferno response-cache`
command and is not covered here.

## Quick Start

```bash
# View cache statistics and current status
inferno cache stats

# Configure cache limits and TTL
inferno cache configure --max-models 5 --max-memory-mb 8192 --ttl-seconds 3600

# Warm up specific models (models are positional, space-separated)
inferno cache warmup llama-7b.gguf mistral-7b.gguf

# Clear the cache
inferno cache clear
```

## Commands

### `cache stats`

Show cache statistics and status (hit/miss counts, cached models, memory use).

```bash
inferno cache stats
```

### `cache configure`

Set cache behavior. Values are also settable in the config file (see below).

```bash
inferno cache configure \
  --max-models 5 \
  --max-memory-mb 8192 \
  --ttl-seconds 3600 \
  --warmup true \
  --strategy usage-based \
  --always-warm llama-7b.gguf,mistral-7b.gguf
```

Options:
- `--max-models <N>` - maximum number of models kept in memory
- `--max-memory-mb <MB>` - memory ceiling for cached models
- `--ttl-seconds <S>` - evict a model after this many seconds unused
- `--warmup <true|false>` - enable or disable automatic warm-up
- `--strategy <S>` - warm-up strategy: `usage-based`, `predictive`, `size-optimized`, `priority`, `hybrid`
- `--always-warm <LIST>` - comma-separated models to always keep warm

### `cache warmup`

Preload models into the cache. Models are positional arguments.

```bash
# Warm specific models
inferno cache warmup llama-7b.gguf mistral-7b.gguf

# Choose a strategy and concurrency
inferno cache warmup llama-7b.gguf --strategy hybrid --concurrent 4
```

Options:
- `[MODELS]...` - models to warm up (space-separated positional args)
- `--strategy <S>` - `usage-based`, `predictive`, `size-optimized`, `priority`, `hybrid`
- `--concurrent <N>` - maximum concurrent loads (default 2)

### `cache clear`

Clear cached models.

```bash
# Clear everything
inferno cache clear

# Clear a specific model
inferno cache clear --model llama-7b.gguf

# Force clear, including always-warm models
inferno cache clear --force
```

### `cache benchmark`

Benchmark cache performance by issuing test requests.

```bash
inferno cache benchmark --requests 20 --models llama-7b.gguf --concurrent
```

Options:
- `--requests <N>` - test requests per model (default 10)
- `--models <LIST>` - test models (space-separated)
- `--concurrent` - issue requests concurrently

### `cache monitor`

Watch cache usage in real time.

```bash
inferno cache monitor --interval 5 --detailed
```

Options:
- `--interval <S>` - update interval in seconds (default 5)
- `--detailed` - show per-model statistics

### `cache export`

Export the cache configuration to a file.

```bash
inferno cache export --output cache-config.json --format json
```

Options:
- `--output <FILE>` - output file path
- `--format <F>` - `json` (default), `yaml`, or `toml`

## Configuration File

Cache settings live in the `[cache]` section of your Inferno config file
(`.inferno.toml`, `~/.inferno.toml`, or `~/.config/inferno/config.toml`). The
defaults below match `inferno config show`:

```toml
[cache]
max_cached_models = 5
max_memory_mb = 8192
model_ttl_seconds = 3600
enable_warmup = true
warmup_strategy = "UsageBased"   # UsageBased, Predictive, SizeOptimized, Priority, Hybrid
always_warm = []
predictive_loading = true
usage_window_seconds = 86400
min_usage_frequency = 0.1
memory_based_eviction = true
persist_cache = false            # write cache state to disk in the background
persist_interval_seconds = 300
```

Notes:
- The config file uses `PascalCase` strategy values (e.g. `UsageBased`); the CLI
  `--strategy` flag uses `kebab-case` (e.g. `usage-based`).
- `persist_cache` is off by default. When enabled, the cache state is written
  under the configured `cache_dir` (see the top-level `cache_dir` setting shown by
  `inferno config show`).
- Run `inferno config validate` to check the file, and `inferno config show` to
  see the effective configuration.

## Performance Tuning

- Set `max_memory_mb` to fit your available RAM and typical model sizes.
- Lower `model_ttl_seconds` to release memory sooner; raise it to keep models
  resident longer.
- List frequently used models in `always_warm` (or `--always-warm`) so they are
  preloaded and never evicted by TTL.
- Choose a warm-up `strategy` to match your workload: `usage-based` and
  `predictive` adapt to request patterns, `size-optimized` loads smaller models
  first, `priority` follows your configuration order, and `hybrid` combines them.

## Monitoring

Use `inferno cache stats` for a snapshot and `inferno cache monitor` for a live
view. For process-level metrics, the HTTP server exposes `GET /metrics` and
`GET /metrics/json` when running `inferno serve`.

## Troubleshooting

- **Low hit rate**: increase `max_cached_models` / `max_memory_mb`, raise
  `model_ttl_seconds`, or warm the models you use most with `cache warmup`.
- **High memory use**: lower `max_memory_mb` and `max_cached_models`, or reduce
  `model_ttl_seconds`; keep `memory_based_eviction = true`.
- **Models not staying warm**: add them to `always_warm` and confirm
  `enable_warmup = true`.
- **Verifying settings**: run `inferno cache stats` and `inferno config show` to
  confirm the effective values.
