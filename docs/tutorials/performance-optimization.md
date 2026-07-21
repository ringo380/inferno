# ⚡ Performance Optimization Tutorial

Transform your Inferno deployment from good to blazing fast with proven optimization techniques. Achieve large performance improvements through hardware acceleration, model optimization, and intelligent caching.

## Overview

This comprehensive tutorial covers every aspect of Inferno performance optimization:

- ✅ **Hardware Acceleration** - GPU setup and optimization for maximum throughput
- ✅ **Model Optimization** - Quantization, pruning, and format conversion for speed
- ✅ **Caching Strategies** - Multi-tier caching for sub-second response times
- ✅ **System Tuning** - OS-level optimizations and resource management
- ✅ **Distributed Performance** - Scale across multiple GPUs and nodes
- ✅ **Benchmarking** - Measure and track performance improvements

**Expected Results**: 5-10x latency reduction, 3-5x throughput increase, 50-80% memory reduction
**Time Required**: 45-60 minutes
**Skill Level**: Intermediate to Advanced

## Configuration Model

Inferno reads configuration from `.inferno.toml` (project), `~/.inferno.toml` (user), or
`~/.config/inferno/config.toml` (global), with environment variables (prefixed `INFERNO_`) and
CLI arguments taking precedence. There is no `inferno config set` command - edit the config file
directly. Useful commands:

```bash
inferno config show       # Print the effective configuration
inferno config init       # Generate a default config file
inferno config validate   # Validate the config file
```

## Quick Performance Wins

Start with these high-impact optimizations for immediate results:

### 1. Enable GPU Acceleration (5 minutes)

On Apple Silicon, Metal GPU acceleration is auto-detected and enabled by default. On NVIDIA/AMD
set `gpu_enabled = true` in `.inferno.toml`:

```toml
# .inferno.toml
[backend_config]
gpu_enabled = true
```

```bash
# Check detected GPUs
inferno gpu list

# Run inference (uses the GPU automatically when enabled)
inferno run --model gpt2 --prompt "test"

# Benchmark
inferno bench --model gpt2
```

**Expected Improvement**: 3-10x faster inference

### 2. Use Quantized Models (5 minutes)

```bash
# Install a quantized GGUF from HuggingFace (pick a specific file with --file)
inferno models install TheBloke/Llama-2-7B-Chat-GGUF --file llama-2-7b-chat.Q4_0.gguf

# Compare performance against another quantization
inferno bench --model llama-2-7b-chat.Q4_0.gguf

# Quality check
inferno run --model llama-2-7b-chat.Q4_0.gguf --prompt "Explain AI"
```

To remove a model, delete the file from your models directory manually, e.g.
`rm ~/models/llama-2-7b-chat.f16.gguf`. There is no CLI remove/uninstall command.

**Expected Improvement**: 2-4x faster, 75% less memory

### 3. Enable Response Caching (2 minutes)

```toml
# .inferno.toml
[cache]
enabled = true
max_size_gb = 20
```

```bash
# Test cache performance
time inferno run --model gpt2 --prompt "What is AI?"  # First run
time inferno run --model gpt2 --prompt "What is AI?"  # Cached run
```

**Expected Improvement**: Instant responses for repeated queries

## GPU Optimization

### GPU Hardware Setup

#### NVIDIA GPU Setup

```bash
# Install NVIDIA drivers (if not already installed)
sudo apt update
sudo apt install nvidia-driver-535  # Or latest version

# Install CUDA toolkit
wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.0-1_all.deb
sudo dpkg -i cuda-keyring_1.0-1_all.deb
sudo apt update
sudo apt install cuda-toolkit-12-2

# Verify installation
nvidia-smi
nvcc --version

# Optimize GPU settings
sudo nvidia-smi -pm 1                    # Enable persistence mode
sudo nvidia-smi -ac 5001,1590           # Set memory and GPU clocks
sudo nvidia-smi --auto-boost-default=0   # Disable auto boost for consistency
```

#### AMD GPU Setup (ROCm)

```bash
# Install ROCm
wget https://repo.radeon.com/amdgpu-install/latest/ubuntu/jammy/amdgpu-install_5.7.50700-1_all.deb
sudo dpkg -i amdgpu-install_5.7.50700-1_all.deb
sudo amdgpu-install --usecase=rocm

# Verify installation
rocm-smi
```

#### Apple Silicon Optimization

```bash
# Metal GPU acceleration is auto-detected and enabled by default on Apple Silicon.
# Confirm the GPU is detected:
inferno gpu list

# Monitor GPU usage
sudo powermetrics --samplers gpu_power -n 1
```

### GPU Configuration

```toml
# .inferno.toml - GPU optimization
[backend_config]
gpu_enabled = true
gpu_layers = 35          # Number of layers to run on GPU
gpu_memory_fraction = 0.9 # Use 90% of GPU memory
tensor_parallel_size = 2  # Multi-GPU parallelism
pipeline_parallel_size = 1

[gpu]
device_placement = "auto"  # Automatic device placement
memory_growth = true       # Allow GPU memory to grow
allow_mixed_precision = true # Use mixed precision for speed
```

### Multi-GPU Configuration

Set the parallelism in `.inferno.toml`:

```toml
# .inferno.toml
[backend_config]
tensor_parallel_size = 4   # 4 GPUs
gpu_memory_fraction = 0.8  # Conservative memory usage
```

```bash
# Check available GPUs
inferno gpu list

# Benchmark
inferno bench --model llama-2-70b

# Monitor GPU utilization
watch -n 1 nvidia-smi
# or, using Inferno's own monitor:
inferno gpu monitor
```

## Model Optimization

### Quantization

Quantization reduces model precision to improve speed and reduce memory usage:

#### Quantization Types

| Type | Precision | Speed | Memory | Quality |
|------|-----------|-------|--------|---------|
| **f32** | 32-bit float | 1x | 100% | 100% |
| **f16** | 16-bit float | 1.5x | 50% | 99.5% |
| **q8_0** | 8-bit | 2x | 25% | 98% |
| **q5_1** | 5-bit | 3x | 16% | 95% |
| **q4_0** | 4-bit | 4x | 12.5% | 90% |

Conversion targets supported by `inferno convert`: `q4-0`, `q4-1`, `q5-0`, `q5-1`, `q8-0`,
`f16`, `f32`, `int8`, `int16`.

#### Quantization Examples

```bash
# Convert an existing model to a different quantization
inferno convert quantize --quantization q4-0 llama-2-7b-f16.gguf llama-2-7b-q4_0.gguf
inferno convert quantize --quantization q8-0 llama-2-7b-f16.gguf llama-2-7b-q8_0.gguf

# Install a pre-quantized GGUF from HuggingFace
inferno models install TheBloke/CodeLlama-7B-Instruct-GGUF --file codellama-7b-instruct.Q8_0.gguf

# Batch quantize multiple models
for model in gpt2 bert-base dialogpt-medium; do
    inferno convert quantize --quantization q4-0 "${model}.gguf" "${model}-q4_0.gguf"
done

# Compare quantization impact
inferno bench --model llama-2-7b-f16.gguf   # Baseline
inferno bench --model llama-2-7b-q8_0.gguf  # High quality
inferno bench --model llama-2-7b-q4_0.gguf  # Balanced
```

#### Advanced Conversion Options

```bash
# Convert with a target format and optimization level
inferno convert model \
  --format gguf \
  --quantization q4-0 \
  --optimization aggressive \
  model.safetensors model-optimized.gguf

# Target a specific precision
inferno convert model \
  --format gguf \
  --precision float16 \
  model.safetensors model-fp16.gguf
```

### Model Pruning

Remove unnecessary model weights for faster inference:

```bash
# Prune to a target sparsity, preserving an accuracy threshold
inferno optimization prune --input llama-2-7b.gguf --output llama-2-7b-pruned.gguf --sparsity 0.3
inferno optimization prune --input gpt2.gguf --output gpt2-pruned.gguf --sparsity 0.5 --accuracy-threshold 0.9

# Knowledge distillation (train a smaller student model from a larger teacher)
inferno optimization distill \
  --teacher llama-2-70b.gguf \
  --student llama-2-7b.gguf \
  --output llama-2-7b-distilled.gguf
```

### Model Format Optimization

Choose the optimal format for your use case:

```bash
# Convert to GGUF for CPU inference
inferno convert model --format gguf --optimization balanced model.pt model.gguf

# Convert to ONNX for cross-platform deployment
inferno convert model --format onnx --optimization balanced model.pt model.onnx

# Optimize for GPU with half precision
inferno convert model \
  --format onnx \
  --optimization aggressive \
  --precision float16 \
  model.gguf model-gpu.onnx

# Graph-level optimizations
inferno convert optimize --merge-ops --constant-folding --operator-fusion model.gguf model-opt.gguf
```

## Caching Optimization

### Multi-Tier Caching Strategy

```toml
# .inferno.toml - Advanced caching configuration
[cache]
enabled = true
max_size_gb = 50

# L1 Cache: In-memory (fastest)
[cache.memory]
enabled = true
max_size_gb = 16
eviction_policy = "lru"
ttl_seconds = 3600

# L2 Cache: Disk-based (persistent)
[cache.disk]
enabled = true
max_size_gb = 30
compression = "zstd"
compression_level = 3
location = "/fast-ssd/inferno/cache"

# L3 Cache: Response deduplication
[cache.deduplication]
enabled = true
hash_algorithm = "blake3"
similarity_threshold = 0.95
```

### Cache Warming Strategies

```bash
# Warm the cache using a strategy
inferno cache warmup --strategy usage-based

# Warm specific models
inferno cache warmup gpt2.gguf bert-base.gguf dialogpt-medium.gguf

# Scheduled cache warming
cat > warm_cache.sh << 'EOF'
#!/bin/bash
# Run daily at 2 AM
for model in gpt2.gguf dialogpt-medium.gguf bert-base.gguf; do
    inferno cache warmup "$model"
done
EOF

# Add to crontab
echo "0 2 * * * /path/to/warm_cache.sh" | crontab -
```

### Cache Monitoring

```bash
# Cache analytics
inferno cache stats      # Show cache hit rates and status
inferno cache monitor    # Real-time cache usage
inferno cache benchmark  # Benchmark cache performance

# Clear the cache when needed
inferno cache clear            # Clear the model cache
inferno cache clear --force    # Clear even always-warm models
```

### Response Cache

Response caching and deduplication are configured in `.inferno.toml`:

```toml
# .inferno.toml
[response_cache]
enabled = true
deduplication = true
compression = true
ttl = 1800          # 30 minutes
max_entries = 100000
```

## System-Level Optimization

### CPU Optimization

```bash
# Set CPU governor to performance mode
echo 'performance' | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Disable CPU frequency scaling
sudo systemctl disable ondemand.service

# Configure CPU affinity for Inferno
taskset -c 0-7 inferno serve  # Use CPUs 0-7

# NUMA optimization
numactl --cpubind=0 --membind=0 inferno serve

# Huge pages for large models
echo 2048 | sudo tee /proc/sys/vm/nr_hugepages
sudo mount -t hugetlbfs hugetlbfs /mnt/hugepages
```

### Memory Optimization

```bash
# Configure swap settings
echo 1 | sudo tee /proc/sys/vm/swappiness  # Minimize swap usage
echo 1 | sudo tee /proc/sys/vm/vfs_cache_pressure  # Reduce cache pressure

# Monitor memory usage
watch -n 1 'ps aux | grep inferno | head -1; free -h'

# Inspect and defragment Inferno's memory pools
inferno optimization memory status
inferno optimization memory defragment
```

Memory management settings live in `.inferno.toml`:

```toml
# .inferno.toml
[backend_config]
memory_pool_size = "32GB"
memory_mapping = true
lazy_loading = true
```

### I/O Optimization

```bash
# Use faster storage for models and cache
sudo mkdir -p /fast-ssd/inferno/{models,cache}
sudo chown -R $(whoami) /fast-ssd/inferno

# Configure I/O scheduler for SSDs
echo noop | sudo tee /sys/block/nvme0n1/queue/scheduler

# Optimize file system
sudo mount -o remount,noatime,nodiratime /fast-ssd
```

Point Inferno at the fast storage in `.inferno.toml`:

```toml
# .inferno.toml
models_dir = "/fast-ssd/inferno/models"

[cache.disk]
location = "/fast-ssd/inferno/cache"
```

### Network Optimization

```bash
# Increase network buffer sizes
echo 'net.core.rmem_max = 268435456' | sudo tee -a /etc/sysctl.conf
echo 'net.core.wmem_max = 268435456' | sudo tee -a /etc/sysctl.conf
echo 'net.ipv4.tcp_rmem = 4096 87380 268435456' | sudo tee -a /etc/sysctl.conf
echo 'net.ipv4.tcp_wmem = 4096 65536 268435456' | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

Server connection settings live in `.inferno.toml`:

```toml
# .inferno.toml
[server]
max_connections = 2000
keep_alive_timeout = 30
connection_pool_size = 100
```

## Application-Level Optimization

### Batch Processing Optimization

```toml
# .inferno.toml - batching
[backend_config]
batch_size = 128            # For GPU (use 32 for CPU)
prefill_batch_size = 256
dynamic_batching = true
max_batch_delay_ms = 50
batch_timeout_ms = 100
```

```bash
# Benchmark throughput
inferno bench --model llama-2-7b

# Tune dynamic batching interactively
inferno optimization batch --help
```

### Async Processing Optimization

```toml
# .inferno.toml - Async optimization
[server]
workers = 16              # 2x CPU cores
async_workers = 32        # 4x CPU cores
io_threads = 8            # For I/O operations
compute_threads = 16      # For CPU computation

[performance]
request_queue_size = 1000
response_buffer_size = 10000
async_timeout_ms = 30000
```

### Model Loading Optimization

Models are loaded on demand at inference time. Tune loading behavior in `.inferno.toml`:

```toml
# .inferno.toml
[backend_config]
memory_mapping = true
lazy_loading = true
```

## Distributed Performance

### Multi-Worker Serving

```bash
# Start the HTTP server with distributed worker pools
inferno serve --distributed --workers 8

# Or run a standalone distributed inference server
inferno distributed start --workers 8 --load-balancing --preload-model llama-2-7b.gguf

# Inspect worker statistics
inferno distributed stats

# Benchmark distributed throughput
inferno distributed benchmark --model llama-2-7b.gguf --concurrent 10 --requests 5
```

### GPU Parallelism

Configure tensor/pipeline parallelism in `.inferno.toml`:

```toml
# .inferno.toml
[backend_config]
tensor_parallel_size = 4
pipeline_parallel_size = 2
```

## Benchmarking and Monitoring

### Benchmarking

```bash
# Basic performance benchmark
inferno bench --model gpt2

# Benchmark with more iterations and JSON output for tracking
inferno bench --model llama-2-7b \
  --iterations 100 \
  --tokens 128 \
  --output-json benchmark_results.json

# Memory profiling
inferno performance-benchmark memory-profile --model llama-2-7b --cycles 50 --track

# Stress test
inferno performance-benchmark stress --model gpt2 --duration 300 --clients 10
```

### Performance Monitoring

```bash
# Real-time monitoring
inferno monitor watch --interval 5

# Monitoring dashboard (HTTP)
inferno monitor dashboard --port 3000

# Performance profiling
inferno optimization profile --model llama-2-7b --detailed --format json

# Periodic metric snapshots
cat > monitor.sh << 'EOF'
#!/bin/bash
while true; do
    inferno metrics snapshot --pretty > "metrics_$(date +%s).json"
    sleep 60
done
EOF
```

### Performance Baselines

```bash
# Establish a performance baseline
inferno performance-benchmark baseline --output performance_baseline --duration 30

# Run a benchmark that produces current results
inferno performance-benchmark benchmark --output benchmark_results

# Compare current results against the baseline (regression testing)
inferno performance-benchmark compare \
  --current benchmark_results/results.json \
  --baseline performance_baseline/results.json \
  --threshold 5.0 \
  --report
```

## Advanced Optimization Techniques

### Precision and Mixed-Precision

```toml
# .inferno.toml
[backend_config]
mixed_precision = true
fp16 = true
allow_mixed_precision = true
```

### Memory Optimization Techniques

```bash
# Inspect memory status and trigger defragmentation
inferno optimization memory status
inferno optimization memory defragment

# Apply a comprehensive optimization pass to a model
inferno optimization optimize --help
```

## Real-World Optimization Examples

### Example 1: High-Throughput Chat Service

```toml
# .inferno.toml - chat service handling many concurrent users
[server]
workers = 32
max_connections = 5000

[backend_config]
batch_size = 256

[cache]
enabled = true
max_size_gb = 64

[response_cache]
enabled = true
ttl = 1800
```

```bash
# Use a quantized model for speed
inferno models install TheBloke/Llama-2-7B-Chat-GGUF --file llama-2-7b-chat.Q4_0.gguf

# Serve with worker pools
inferno serve --distributed --workers 32
```

### Example 2: Code Generation Service

```bash
# High-quality quantization for code
inferno models install TheBloke/CodeLlama-7B-Instruct-GGUF --file codellama-7b-instruct.Q8_0.gguf
```

```toml
# .inferno.toml
[backend_config]
context_size = 8192   # Longer context for code
temperature = 0.1     # Lower temperature for deterministic code
```

```bash
# Warm the cache before serving
inferno cache warmup codellama-7b-instruct.Q8_0.gguf
```

### Example 3: Image and Audio Input

Inferno's `run` command accepts non-text input types:

```bash
# Run inference on an image or audio file
inferno run --model llava-model.gguf --input-type image --input photo.png
inferno run --model whisper-model.gguf --input-type audio --input clip.wav
```

## Performance Troubleshooting

### Common Performance Issues

#### Slow Model Loading

```bash
# Diagnose with a benchmark
inferno bench --model llama-2-7b

# Warm the cache
inferno cache warmup --strategy usage-based

# Move models to faster storage
sudo mv /slow-disk/models/* /fast-ssd/models/
```

Enable memory mapping in `.inferno.toml`:

```toml
# .inferno.toml
[backend_config]
memory_mapping = true
```

#### High Memory Usage

```bash
# Diagnose
inferno optimization memory status
ps aux | grep inferno

# Clear old cache entries
inferno cache clear

# Reduce the model's memory footprint by lowering context size in .inferno.toml
```

```toml
# .inferno.toml
[backend_config]
context_size = 2048
```

Models are loaded on demand and released automatically - there is no manual unload command.

#### GPU Underutilization

```bash
# Diagnose
nvidia-smi dmon -s pucvmet -d 1
inferno gpu monitor

# Increase batch size and GPU layers in .inferno.toml
```

```toml
# .inferno.toml
[backend_config]
batch_size = 128
gpu_layers = 35
tensor_parallel_size = 2
```

#### Network Bottlenecks

```bash
# Diagnose
iftop -i eth0
```

Tune server connection settings in `.inferno.toml`:

```toml
# .inferno.toml
[server]
connection_pool_size = 200
keep_alive_timeout = 60
compression = true
```

### Performance Debugging

```bash
# Enable detailed logging
export INFERNO_LOG_LEVEL=debug
export INFERNO_LOG_FORMAT=json
inferno serve

# Memory debugging with valgrind
valgrind --tool=massif inferno serve
```

## Performance Metrics and KPIs

### Key Performance Indicators

| Metric | Target | Excellent | Good | Needs Improvement |
|--------|--------|-----------|------|-------------------|
| **Latency (P95)** | <200ms | <100ms | <500ms | >1000ms |
| **Throughput** | >100 req/s | >500 req/s | >50 req/s | <10 req/s |
| **GPU Utilization** | >80% | >90% | >60% | <40% |
| **Cache Hit Rate** | >80% | >95% | >70% | <50% |
| **Memory Efficiency** | <16GB | <8GB | <32GB | >64GB |

### Monitoring and Alerts

```bash
# Start the monitoring dashboard
inferno monitor dashboard --port 3001

# Configure alert thresholds
inferno monitor configure \
  --max-response-time 500 \
  --min-throughput 100 \
  --max-error-rate 5 \
  --min-cache-hit-rate 70

# List active alerts
inferno monitor alerts

# Generate a performance report
inferno monitor report --hours 24 --detailed --recommendations
```

## Best Practices Summary

### Hardware Optimization
1. **Use GPUs** whenever possible for 3-10x speedup
2. **Optimize GPU settings** with persistence mode and fixed clocks
3. **Use fast storage** (NVMe SSD) for models and cache
4. **Configure NUMA** for multi-socket systems

### Model Optimization
1. **Use quantized models** (q4_0 or q8_0) for best speed/quality balance
2. **Convert to optimal formats** (GGUF for CPU, ONNX for cross-platform)
3. **Apply graph optimizations** with `inferno convert optimize`
4. **Implement model pruning** for specialized use cases

### Caching Strategy
1. **Enable multi-tier caching** with memory and disk layers
2. **Implement cache warming** for popular models
3. **Use response deduplication** for repeated queries
4. **Monitor cache hit rates** and optimize accordingly

### System Configuration
1. **Tune OS settings** for performance (CPU governor, memory, I/O)
2. **Configure application threads** appropriately for your hardware
3. **Optimize network settings** for high-throughput scenarios
4. **Implement proper monitoring** to track performance metrics

### Scaling Strategy
1. **Start with single-node optimization** before scaling out
2. **Use horizontal scaling** for increased throughput
3. **Implement load balancing** across multiple instances
4. **Monitor resource utilization** across all nodes

## Next Steps

Now that you've optimized your Inferno deployment:

### Immediate Actions
1. **[Benchmarking Guide](../reference/benchmarks.md)** - Establish performance baselines
2. **[Monitoring Setup](../guides/monitoring.md)** - Track performance continuously
3. **[Load Testing](../guides/load-testing.md)** - Validate performance under load

### Advanced Optimization
1. **[Custom Backend Development](custom-backend.md)** - Optimize for specific models
2. **[Distributed Inference](../guides/distributed-inference.md)** - Scale across multiple machines
3. **[GPU Cluster Setup](../guides/gpu-cluster.md)** - Multi-GPU optimization

---

**🚀 You've transformed your Inferno deployment into a high-performance AI infrastructure.** Continue monitoring and fine-tuning based on your specific workload patterns.
