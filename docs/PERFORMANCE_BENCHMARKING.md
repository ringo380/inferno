# Performance Benchmarking and Profiling System

The Inferno AI/ML platform includes a comprehensive performance benchmarking and profiling system designed to measure, monitor, and optimize performance across all components.

## 🎯 Performance Targets

Our performance targets are designed to ensure Inferno delivers enterprise-grade performance:

| Metric | Target | Description |
|--------|--------|-------------|
| **Inference Latency** | < 100ms | Average inference time for most models |
| **Memory Efficiency** | 50% reduction | Memory usage optimization target |
| **Throughput** | > 1000 RPS | Requests per second under normal load |
| **Model Loading** | < 5 seconds | Time to load most models into memory |
| **Cache Hit Ratio** | > 80% | Cache effectiveness for repeated requests |
| **CPU Utilization** | < 80% | CPU usage under normal operation |
| **Memory Utilization** | < 70% | Memory usage of available system memory |

## 🛠️ Benchmark Suite

The benchmarking system includes several comprehensive test suites:

### 1. Inference Performance Benchmarks
- **File**: `benches/inference_benchmark.rs`
- **Purpose**: Measure model loading, inference latency, and throughput
- **Metrics**: Latency percentiles (P50, P90, P99), requests per second, error rates

### 2. Memory Usage Benchmarks
- **File**: `benches/memory_benchmark.rs`
- **Purpose**: Track memory consumption patterns and detect leaks
- **Metrics**: Peak memory usage, memory efficiency scores, allocation patterns

### 3. Concurrent Performance Benchmarks
- **File**: `benches/concurrent_benchmark.rs`
- **Purpose**: Test performance under concurrent load
- **Metrics**: Concurrent throughput, resource contention, scalability limits

### 4. Cache Performance Benchmarks
- **File**: `benches/cache_benchmark.rs`
- **Purpose**: Evaluate caching system effectiveness
- **Metrics**: Hit/miss ratios, cache throughput, eviction performance

### 5. CPU Profiling Benchmarks
- **File**: `benches/profiling_benchmark.rs`
- **Purpose**: Generate CPU profiles and flamegraphs
- **Output**: Flamegraphs (SVG), pprof profiles, hotspot analysis

## 🚀 Quick Start

### Running All Benchmarks

```bash
# Using the benchmark script (recommended)
./scripts/benchmark.sh

# Using cargo directly
cargo bench

# Using the CLI
cargo run --release -- performance-benchmark benchmark --bench-type all
```

### Running Specific Benchmarks

```bash
# Inference performance only
./scripts/benchmark.sh inference

# Memory benchmarks with profiling
./scripts/benchmark.sh memory --profile

# Cache performance
cargo bench --bench cache_benchmark
```

### Establishing Performance Baseline

```bash
# Establish baseline for comparison
./scripts/benchmark.sh baseline

# Using CLI
cargo run --release -- performance-benchmark baseline \
  --output performance_baseline \
  --backends gguf,onnx \
  --duration 60
```

## 📊 CLI Commands

The performance benchmarking system provides comprehensive CLI commands:

### Baseline Establishment

```bash
# Establish comprehensive baseline
inferno performance-benchmark baseline \
  --output performance_baseline \
  --backends gguf,onnx \
  --duration 30

# Use custom performance targets
inferno performance-benchmark baseline \
  --targets custom_targets.json \
  --output baseline_results
```

### Running Benchmarks

```bash
# Run inference benchmarks
inferno performance-benchmark benchmark \
  --bench-type inference \
  --iterations 100 \
  --output benchmark_results

# Run with profiling enabled
inferno performance-benchmark benchmark \
  --bench-type all \
  --profile \
  --output profiled_results
```

### Performance Comparison

```bash
# Compare current results with baseline
inferno performance-benchmark compare \
  --current recent_results.json \
  --baseline baseline_results.json \
  --threshold 10.0 \
  --report
```

### Real-time Monitoring

```bash
# Monitor performance for 5 minutes
inferno performance-benchmark monitor \
  --duration 300 \
  --interval 10 \
  --format console

# Save monitoring data to file
inferno performance-benchmark monitor \
  --duration 600 \
  --format json \
  --output monitoring_results.json
```

### Stress Testing

```bash
# Stress test with 20 concurrent clients
inferno performance-benchmark stress \
  --clients 20 \
  --duration 300 \
  --rate 2.0
```

### Memory Profiling

```bash
# Profile memory usage patterns
inferno performance-benchmark memory-profile \
  --cycles 100 \
  --track \
  --output memory_profile.json
```

## 📈 Performance Analysis

### Understanding Benchmark Results

Benchmark results are provided in JSON format with detailed metrics:

```json
{
  "benchmark_type": "inference",
  "timestamp": "2024-01-01T12:00:00Z",
  "avg_latency_ms": 45.2,
  "p50_latency_ms": 42.1,
  "p90_latency_ms": 68.3,
  "p99_latency_ms": 124.7,
  "throughput_rps": 125.6,
  "error_rate": 0.002,
  "memory_usage_mb": 256
}
```

### Regression Detection

The system automatically detects performance regressions:

```bash
# Check for regressions (fails if found)
inferno performance-benchmark compare \
  --current new_results.json \
  --baseline baseline.json \
  --threshold 10.0  # 10% regression threshold
```

### Profiling Output

CPU profiling generates several output formats:

- **Flamegraphs** (`.svg`): Visual representation of CPU hotspots
- **pprof profiles** (`.pb`): Binary profiles for detailed analysis
- **Text reports**: Human-readable performance summaries

## 🔧 Configuration

### Custom Performance Targets

Create a custom targets file:

```json
{
  "inference_latency_ms": 80,
  "memory_efficiency_mb": 400,
  "throughput_rps": 1500,
  "model_loading_time_ms": 3000,
  "cache_hit_ratio": 0.85,
  "cpu_utilization": 0.75,
  "memory_utilization": 0.65
}
```

### Environment Variables

```bash
export INFERNO_MODELS_DIR="./test_models"
export INFERNO_LOG_LEVEL="info"
export CRITERION_PROFILER="pprof"  # Enable profiling
```

## 🤖 Continuous Integration

The performance benchmarking system integrates with CI/CD:

### GitHub Actions Workflow

The `.github/workflows/performance-ci.yml` workflow:

- Runs benchmarks on every push to main
- Compares PR performance with baseline
- Establishes new baselines automatically
- Generates performance reports
- Fails CI on significant regressions

### Running in CI

```yaml
- name: Run performance benchmarks
  run: |
    ./scripts/benchmark.sh --ci --compare
```

## 📂 Output Files and Reports

### Directory Structure

```
benchmark_results/
├── inference_benchmark.json     # Inference metrics
├── memory_benchmark.json        # Memory usage data
├── concurrent_benchmark.json    # Concurrency results
├── cache_benchmark.json         # Cache performance
└── benchmark_report.html        # HTML summary report

performance_baseline/
├── baseline_results.json        # Baseline metrics
├── performance_targets.json     # Target definitions
└── baseline_report.md          # Baseline report

flamegraphs/
├── inference_profile.svg        # Inference flamegraph
├── memory_profile.svg          # Memory flamegraph
└── concurrent_profile.svg      # Concurrency flamegraph
```

### Report Generation

```bash
# Generate HTML report
./scripts/benchmark.sh --report

# Generate comparison report
inferno performance-benchmark compare \
  --current results.json \
  --baseline baseline.json \
  --report
```

## 🔍 Advanced Usage

### Custom Benchmark Development

Create custom benchmarks by extending the existing framework:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn custom_benchmark(c: &mut Criterion) {
    c.bench_function("custom_operation", |b| {
        b.iter(|| {
            // Your benchmark code here
            black_box(expensive_operation())
        })
    });
}

criterion_group!(benches, custom_benchmark);
criterion_main!(benches);
```

### Integration with Monitoring Systems

Export metrics to external systems:

```bash
# Export to Prometheus format
inferno metrics export --format prometheus > metrics.txt

# Integration with monitoring dashboard
inferno performance-benchmark monitor \
  --format json \
  --output /monitoring/inferno_metrics.json
```

### Memory Leak Detection

Use the memory profiling tools to detect leaks:

```bash
# Track memory over extended period
inferno performance-benchmark memory-profile \
  --cycles 1000 \
  --track \
  --output long_term_memory.json

# Analyze memory patterns
grep "memory_delta" long_term_memory.json
```

## 🚨 Troubleshooting

### Common Issues

1. **Benchmark Timeout**
   ```bash
   # Increase timeout
   ./scripts/benchmark.sh --timeout 600
   ```

2. **Memory Constraints**
   ```bash
   # Use smaller test models
   export INFERNO_MODELS_DIR="./small_test_models"
   ```

3. **Permission Issues**
   ```bash
   # Ensure benchmark script is executable
   chmod +x ./scripts/benchmark.sh
   ```

### Performance Debugging

1. **High Latency**
   - Check CPU profiling flamegraphs
   - Verify model loading optimization
   - Examine cache hit ratios

2. **Memory Issues**
   - Run memory benchmarks
   - Check for memory leaks
   - Analyze allocation patterns

3. **Low Throughput**
   - Test concurrent performance
   - Check resource contention
   - Verify backend efficiency

## 📚 Best Practices

### Regular Benchmarking

1. **Run benchmarks before releases**
2. **Establish baselines for major versions**
3. **Monitor performance trends over time**
4. **Set up alerts for regressions**

### Performance Optimization

1. **Profile before optimizing**
2. **Focus on hot paths identified in flamegraphs**
3. **Validate optimizations with benchmarks**
4. **Document performance trade-offs**

### CI Integration

1. **Fail builds on significant regressions**
2. **Track performance metrics over time**
3. **Generate automated performance reports**
4. **Alert teams to performance issues**

## 🔗 Related Documentation

- [Architecture Overview](./ARCHITECTURE.md)
- [Backend Development](./BACKENDS.md)
- [Caching System](./CACHING.md)
- [Monitoring and Observability](./MONITORING.md)
- [Deployment Guide](./DEPLOYMENT.md)

## 🤝 Contributing

When contributing performance-related changes:

1. Run full benchmark suite
2. Include performance impact analysis
3. Update baselines if improvements are significant
4. Document any performance trade-offs
5. Add tests for new performance features

For questions or issues with the performance benchmarking system, please open an issue with the `performance` label.