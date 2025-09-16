# Cache Configuration and Persistence Guide

This guide covers Inferno's advanced multi-tier caching system with disk persistence, compression, and intelligent cache management features.

## Overview

Inferno implements a sophisticated caching architecture with multiple layers:

- **Memory Cache**: Ultra-fast in-memory LRU cache for active models and responses
- **Disk Persistence**: Compressed storage using Gzip or Zstd compression
- **Response Deduplication**: Hash-based caching using Blake3 and xxHash algorithms
- **Model Cache**: Intelligent model loading and precomputation
- **Metadata Cache**: Fast access to model information and statistics

## Quick Start

```bash
# Enable persistent caching with compression
inferno cache persist --compress gzip

# Configure cache size and TTL
inferno cache configure --max-size 10GB --ttl 24h

# Warm cache with frequently used models
inferno cache warm --model llama-7b.gguf

# View cache statistics
inferno cache stats

# Clear specific cache entries
inferno cache clear --pattern "*.onnx"
```

## Cache Types

### 1. Model Cache

Stores loaded model instances and metadata for fast access.

```bash
# Enable model caching
inferno cache enable --type model

# Configure model cache
inferno cache configure model \
  --max-models 5 \
  --memory-limit 8GB \
  --preload-strategy lru

# Preload models into cache
inferno cache warm --model llama-7b.gguf --model mistral-7b.onnx
```

### 2. Response Cache

Caches inference responses to avoid redundant computation.

```bash
# Enable response caching with deduplication
inferno cache enable --type response --deduplication blake3

# Configure response cache
inferno cache configure response \
  --max-entries 10000 \
  --max-size 2GB \
  --ttl 1h \
  --compress zstd
```

### 3. Metadata Cache

Stores model metadata and statistics for quick access.

```bash
# Enable metadata caching
inferno cache enable --type metadata

# Refresh metadata cache
inferno cache refresh --type metadata
```

## Configuration

### Basic Configuration

```toml
# .inferno.toml
[cache]
enabled = true
type = "persistent"  # memory, disk, persistent
base_dir = "/var/cache/inferno"
max_size_gb = 10
default_ttl_hours = 24

[cache.compression]
enabled = true
algorithm = "gzip"  # none, gzip, zstd
level = 6  # 1-9 for gzip, 1-21 for zstd

[cache.deduplication]
enabled = true
algorithm = "blake3"  # blake3, xxhash
chunk_size = 4096
```

### Advanced Configuration

```toml
[cache.model]
max_models = 5
memory_limit_gb = 8
preload_strategy = "lru"  # lru, mru, frequency
unload_policy = "size_based"  # time_based, size_based, manual

[cache.response]
max_entries = 10000
max_size_gb = 2
ttl_hours = 1
enable_streaming_cache = true
hash_algorithm = "blake3"

[cache.metadata]
ttl_hours = 6
auto_refresh = true
include_performance_stats = true

[cache.persistence]
sync_interval_seconds = 300
compression_threshold_kb = 64
enable_checksums = true
backup_copies = 2

[cache.cleanup]
auto_cleanup = true
cleanup_interval_hours = 24
size_threshold_percent = 90
age_threshold_days = 7
```

## Cache Operations

### Enable/Disable Caching

```bash
# Enable all caching
inferno cache enable --all

# Enable specific cache types
inferno cache enable --type model,response

# Disable caching
inferno cache disable --type response

# Check cache status
inferno cache status
```

### Cache Management

```bash
# View cache statistics
inferno cache stats --detailed

# Clear cache
inferno cache clear --all
inferno cache clear --type model
inferno cache clear --older-than 24h
inferno cache clear --pattern "llama*"

# Compact cache (remove fragmentation)
inferno cache compact

# Verify cache integrity
inferno cache verify --fix
```

### Cache Warming

```bash
# Warm model cache
inferno cache warm --model llama-7b.gguf
inferno cache warm --directory /models/production

# Warm response cache with common prompts
inferno cache warm --responses prompts.txt

# Background warming
inferno cache warm --model llama-7b.gguf --background
```

### Cache Monitoring

```bash
# Real-time cache monitoring
inferno cache monitor --interval 5s

# Cache performance report
inferno cache report --period 24h

# Export cache metrics
inferno cache export --format prometheus > cache_metrics.txt
```

## Compression

### Gzip Compression

```bash
# Enable Gzip compression (fast, good compression ratio)
inferno cache configure --compress gzip --compression-level 6

# Gzip settings in config
[cache.compression]
algorithm = "gzip"
level = 6  # 1 (fastest) to 9 (best compression)
```

### Zstd Compression

```bash
# Enable Zstd compression (faster, better compression)
inferno cache configure --compress zstd --compression-level 3

# Zstd settings in config
[cache.compression]
algorithm = "zstd"
level = 3  # 1 (fastest) to 21 (best compression)
```

### Compression Performance

| Algorithm | Speed | Compression Ratio | CPU Usage |
|-----------|-------|-------------------|-----------|
| None      | Fastest | 1.0x | Minimal |
| Gzip (6)  | Fast | 3.2x | Low |
| Gzip (9)  | Medium | 3.6x | Medium |
| Zstd (3)  | Very Fast | 3.4x | Low |
| Zstd (11) | Medium | 4.1x | Medium |

## Deduplication

### Hash Algorithms

#### Blake3 (Recommended)
```bash
inferno cache configure --deduplication blake3

# Configuration
[cache.deduplication]
algorithm = "blake3"
chunk_size = 4096  # bytes
enable_parallel = true
```

#### xxHash
```bash
inferno cache configure --deduplication xxhash

# Configuration
[cache.deduplication]
algorithm = "xxhash"
seed = 42
variant = "xxh3"  # xxh32, xxh64, xxh3
```

### Deduplication Statistics

```bash
# View deduplication stats
inferno cache stats --deduplication

# Example output:
# Total responses: 10,000
# Unique responses: 3,247
# Deduplication ratio: 67.5%
# Space saved: 1.2 GB
```

## Persistence

### Disk Storage

```bash
# Configure persistent storage
inferno cache configure \
  --base-dir /fast-ssd/inferno-cache \
  --sync-interval 300s \
  --enable-checksums

# Check disk usage
inferno cache disk-usage

# Backup cache
inferno cache backup --destination /backup/cache
```

### Backup and Restore

```bash
# Create cache backup
inferno cache backup \
  --destination /backup/inferno-cache-$(date +%Y%m%d) \
  --compress \
  --verify

# Restore from backup
inferno cache restore \
  --source /backup/inferno-cache-20240315 \
  --verify

# List available backups
inferno cache backup list
```

## Performance Tuning

### Memory Management

```bash
# Configure memory limits
inferno cache configure \
  --memory-limit 4GB \
  --max-models 3 \
  --eviction-policy lru

# Monitor memory usage
inferno cache monitor --memory
```

### I/O Optimization

```bash
# Optimize for SSD storage
inferno cache configure \
  --sync-interval 60s \
  --write-buffer-size 1MB \
  --read-ahead-size 512KB

# Optimize for HDD storage
inferno cache configure \
  --sync-interval 300s \
  --write-buffer-size 4MB \
  --read-ahead-size 2MB
```

### Network Optimization

```bash
# Enable cache sharing across nodes
inferno cache configure \
  --enable-distributed \
  --cluster-nodes node1:9090,node2:9090 \
  --replication-factor 2
```

## Monitoring and Metrics

### Cache Metrics

```bash
# View comprehensive cache metrics
inferno cache metrics

# Key metrics include:
# - Hit ratio: 89.3%
# - Miss ratio: 10.7%
# - Average response time: 15ms
# - Cache size: 2.3 GB / 10 GB
# - Compression ratio: 3.4x
# - Deduplication savings: 67%
```

### Prometheus Integration

```bash
# Enable Prometheus metrics export
inferno cache configure --prometheus-enabled

# Key metrics exported:
# - inferno_cache_hits_total
# - inferno_cache_misses_total
# - inferno_cache_size_bytes
# - inferno_cache_compression_ratio
# - inferno_cache_deduplication_ratio
```

### Alerting

```toml
[cache.alerts]
enabled = true
channels = ["email", "slack"]

[cache.alerts.thresholds]
hit_ratio_min = 0.80
size_usage_max = 0.90
compression_ratio_min = 2.0
response_time_max_ms = 100
```

## Troubleshooting

### Common Issues

#### High Cache Miss Ratio
```bash
# Analyze cache patterns
inferno cache analyze --period 24h

# Increase cache size
inferno cache configure --max-size 20GB

# Adjust TTL settings
inferno cache configure --ttl 48h
```

#### Slow Cache Performance
```bash
# Check disk I/O
inferno cache monitor --io

# Optimize compression
inferno cache configure --compress zstd --compression-level 1

# Enable read-ahead
inferno cache configure --read-ahead-size 1MB
```

#### Cache Corruption
```bash
# Verify cache integrity
inferno cache verify --detailed

# Repair corrupted entries
inferno cache repair --auto-fix

# Rebuild cache if necessary
inferno cache rebuild --from-backup
```

#### Disk Space Issues
```bash
# Check cache disk usage
inferno cache disk-usage --detailed

# Clean old entries
inferno cache cleanup --older-than 7d

# Increase compression
inferno cache configure --compress zstd --compression-level 9
```

### Debug Mode

```bash
# Enable cache debugging
INFERNO_LOG_LEVEL=debug inferno cache monitor

# Trace cache operations
inferno cache trace --operations read,write,evict

# Analyze cache efficiency
inferno cache analyze --detailed --export report.json
```

## Best Practices

### Configuration
- Use Zstd compression for better performance
- Set appropriate TTL based on usage patterns
- Enable deduplication for response caching
- Configure memory limits based on available RAM

### Monitoring
- Set up alerts for cache hit ratio < 80%
- Monitor disk usage regularly
- Track response times and performance
- Review cache patterns weekly

### Maintenance
- Schedule regular cache cleanup
- Backup cache data periodically
- Verify cache integrity monthly
- Update compression settings as needed

### Security
- Enable checksums for data integrity
- Use appropriate file permissions
- Encrypt sensitive cached data
- Audit cache access patterns

## Integration Examples

### Docker Deployment

```dockerfile
# Dockerfile with cache volume
FROM inferno:latest

VOLUME ["/cache"]

ENV INFERNO_CACHE_DIR=/cache
ENV INFERNO_CACHE_ENABLED=true
ENV INFERNO_CACHE_COMPRESS=zstd

CMD ["inferno", "serve", "--cache-persist"]
```

### Kubernetes Configuration

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: inferno-cache-config
data:
  cache.toml: |
    [cache]
    enabled = true
    type = "persistent"
    max_size_gb = 50

    [cache.compression]
    algorithm = "zstd"
    level = 3

    [cache.deduplication]
    algorithm = "blake3"
    enabled = true
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: inferno
spec:
  template:
    spec:
      containers:
      - name: inferno
        image: inferno:latest
        volumeMounts:
        - name: cache-storage
          mountPath: /cache
        - name: cache-config
          mountPath: /config
      volumes:
      - name: cache-storage
        persistentVolumeClaim:
          claimName: inferno-cache-pvc
      - name: cache-config
        configMap:
          name: inferno-cache-config
```