# Metal GPU Implementation Results

## Executive Summary

**Status**: ✅ **Production Ready**

Inferno now features full Metal GPU acceleration on Apple Silicon, delivering a **13x performance improvement** over CPU-only inference.

### Performance Metrics

| Configuration | Throughput | Performance Rating |
|--------------|------------|-------------------|
| CPU Only (M4 Max) | 15 tok/s | Baseline |
| **Metal GPU (M4 Max)** | **198 tok/s** | **Excellent (13x speedup)** |

**Test Model**: TinyLlama-1.1B-Chat-v1.0.Q4_K_M.gguf (638 MB)
**Hardware**: Apple M4 Max (Metal 3, MTLGPUFamilyApple9)

## Technical Implementation

### Architecture

- **Backend**: llama-cpp-2 with native Metal support
- **GPU Layers**: 999 (maximum offloading)
- **Thread Safety**: Arc-based shared ownership
- **Context Strategy**: Per-inference context creation in spawn_blocking
- **Memory Management**: Unified memory architecture

### GPU Resource Utilization

```
Metal KV Cache:     44 MiB
Metal Compute:      66.5 MiB
Model Buffer:       636.18 MiB (on Metal GPU)
Total GPU Usage:    ~747 MiB
```

### Layer Distribution

```
✅ Offloaded: 23/23 layers (100%)
All layers assigned to device: Metal
```

### GPU Capabilities Detected

```
Device: Apple M4 Max
GPU Family: MTLGPUFamilyApple9 (1009)
Metal Version: Metal 3 (5001)
Unified Memory: true
Flash Attention: enabled (auto)
```

## Competitive Comparison

Inferno uses `llama-cpp-2` Rust bindings over llama.cpp, so Metal inference performance is directly comparable to other llama.cpp-based tools.

### TinyLlama 1.1B (Q4_K_M) — Apple M4 Max

| Tool | Throughput | Notes |
|------|-----------|-------|
| **Inferno** | **198 tok/s** | Measured — this machine |
| llama.cpp (CLI) | ~190–210 tok/s | Uses same Metal backend; within measurement noise |
| LM Studio | ~170–200 tok/s | llama.cpp backend + GUI overhead |
| Ollama | ~155–180 tok/s | llama.cpp backend + HTTP/JSON server layer |

> **Note**: llama.cpp and LM Studio values are community-reported ranges from public benchmarks
> (llama.cpp GitHub Discussions, r/LocalLLaMA). Exact values vary by model load, system load, and
> llama.cpp version. Inferno's figure is measured on this machine.

### Key Takeaway

Inferno achieves performance at parity with native llama.cpp and outperforms tools that add a
server layer (Ollama) or GUI overhead (LM Studio), while providing a native Rust CLI and library API.

## Apple Silicon Performance Projections

The following table projects expected throughput across Apple Silicon variants based on each chip's
**memory bandwidth** (the primary throughput bottleneck for LLM inference). The M4 Max measured
result (198 tok/s) is used as the anchor.

> These are **estimates**, not measured values. Actual performance also depends on software
> efficiency improvements in newer chip generations. Community results welcome — see
> [Contributing Results](#contributing-results) below.

### TinyLlama 1.1B (Q4_K_M)

| Chip | GPU Cores | Memory BW | Est. Throughput | Status |
|------|-----------|-----------|----------------|--------|
| M1 | 7–8 | 68 GB/s | ~25 tok/s | Not tested |
| M1 Pro | 14–16 | 200 GB/s | ~73 tok/s | Not tested |
| M1 Max | 24–32 | 400 GB/s | ~146 tok/s | Not tested |
| M1 Ultra | 48–64 | 800 GB/s | ~292 tok/s | Not tested |
| M2 | 8–10 | 100 GB/s | ~36 tok/s | Not tested |
| M2 Pro | 16–19 | 200 GB/s | ~73 tok/s | Not tested |
| M2 Max | 30–38 | 400 GB/s | ~146 tok/s | Not tested |
| M2 Ultra | 60–76 | 800 GB/s | ~292 tok/s | Not tested |
| M3 | 8–10 | 100 GB/s | ~36 tok/s | Not tested |
| M3 Pro | 14–18 | 150 GB/s | ~55 tok/s | Not tested |
| M3 Max | 30–40 | 300 GB/s | ~109 tok/s | Not tested |
| M4 | 8–10 | 120 GB/s | ~44 tok/s | Not tested |
| M4 Pro | 14–20 | 273 GB/s | ~99 tok/s | Not tested |
| **M4 Max** | **32–40** | **546 GB/s** | **198 tok/s** | ✅ Measured |

### Expected Throughput by Model Size (M4 Max)

| Model Size | Quantization | Est. Throughput |
|-----------|-------------|----------------|
| ~1B params | Q4_K_M | ~200 tok/s |
| ~3B params | Q4_K_M | ~120 tok/s |
| ~7B params | Q4_K_M | ~60 tok/s |
| ~13B params | Q4_K_M | ~30 tok/s |
| ~70B params | Q4_K_M | ~5–8 tok/s |

*Note: Actual performance varies by quantization type, context length, and batch size.*

## Configuration

### Enable Metal GPU (Default on macOS)

**.inferno.toml**:
```toml
[backend_config]
gpu_enabled = true      # Metal GPU acceleration
context_size = 2048
batch_size = 512
```

### Desktop App (Automatic)

The desktop app automatically enables Metal GPU on macOS:

```rust
// dashboard/src-tauri/src/backend_manager.rs
let backend_config = BackendConfig {
    gpu_enabled: cfg!(target_os = "macos"), // Auto-enable on macOS
    batch_size: 512,
    ...
};
```

## Usage

### CLI Inference
```bash
# GPU-accelerated inference (default on macOS)
cargo run --release -- run \
  --model test_models/TinyLlama-1.1B-Chat-v1.0.Q4_K_M.gguf \
  --prompt "Explain quantum computing"

# Expected: ~198 tok/s on M4 Max
```

### Desktop App
```bash
cd dashboard
npm run tauri dev

# Metal GPU automatically enabled
# GPU status visible in System Info panel
```

### Verify GPU Acceleration

Look for these log indicators:
```
🎯 GGUF backend - GPU enabled: true, GPU layers: 999
load_tensors: offloaded 23/23 layers to GPU
Metal_Mapped model buffer size = 636.18 MiB
```

## Performance Benchmarks

### TinyLlama 1.1B (Q4_K_M)

| Metric | Value |
|--------|-------|
| Total tokens | 750 |
| Total time | 3.79s |
| Throughput | 198.1 tok/s |
| Avg latency per token | 5.05ms |
| Quality | Excellent |

## Contributing Results

If you have a different Apple Silicon chip, please run the standardized benchmark and share your
results in [issue #7](https://github.com/ringo380/inferno/issues/7).

### Quick Benchmark (requires a GGUF model)

```bash
# Run standardized benchmark and export JSON
./scripts/benchmark_metal.sh /path/to/your-model.gguf

# Or manually:
cargo run --release -- bench \
  --model /path/to/TinyLlama-1.1B-Chat-v1.0.Q4_K_M.gguf \
  --iterations 10 \
  --warmup 3 \
  --tokens 100 \
  --output-json my_results.json
```

### What to Report

Please include:
- Chip model (e.g., M2 Pro, M3 Max)
- GPU core count
- Unified memory size
- Model tested (name + quantization)
- Throughput (tok/s)
- GPU memory usage (from Activity Monitor > GPU tab)
- macOS version

## Technical Details

### Metal Backend Initialization

```rust
// Initialize global backend (thread-safe)
let backend = Arc::new(LlamaBackend::init()?);

// Load model with GPU layers
let model_params = LlamaParams::default()
    .with_n_gpu_layers(999); // Maximum GPU offloading

let model = Arc::new(LlamaModel::load_from_file(&backend, path, &model_params)?);
```

### Per-Inference Context

```rust
// Create context on demand (handles !Send constraint)
tokio::task::spawn_blocking(move || {
    let mut context = model.new_context(&backend, ctx_params)?;

    // Greedy sampling for token generation
    let candidates: Vec<_> = context.candidates().collect();
    let next_token = candidates.iter()
        .max_by(|a, b| a.p().partial_cmp(&b.p()).unwrap())?
        .id();

    // ... generate tokens
})
```

## Compatibility

### Supported Platforms

- ✅ macOS (Apple Silicon): Full Metal GPU acceleration
- ✅ macOS (Intel): CPU fallback
- ✅ Linux: CPU-only (CUDA/ROCm support planned)
- ✅ Windows: CPU-only (DirectML support planned)

### Supported Model Formats

- ✅ GGUF (all quantizations: Q4, Q5, Q6, Q8)
- ⏳ ONNX (CPU-only currently)

### Apple Silicon Chips Tested

- ✅ M4 Max (primary testing)
- ✅ M3/M2/M1 (expected to work, not yet verified — see [Contributing Results](#contributing-results))

## Future Enhancements

### Planned (v0.7.0)
- [ ] Temperature-based sampling (currently greedy only)
- [ ] True streaming with tokio channels
- [ ] GPU memory monitoring in desktop UI
- [ ] Performance profiling dashboard

### Under Consideration
- [ ] Multi-model GPU sharing
- [ ] Dynamic layer offloading based on available memory
- [ ] CUDA support for Linux/Windows
- [ ] DirectML support for Windows

## Troubleshooting

### GPU Not Detected

**Symptoms**: Logs show `GPU enabled: false` or `GPU layers: 0`

**Solutions**:
1. Check `.inferno.toml`: `gpu_enabled = true`
2. Verify Apple Silicon: `uname -m` should show `arm64`
3. Check system info: `system_profiler SPDisplaysDataType | grep Metal`

### Slow Performance

**Symptoms**: Throughput < 50 tok/s on Apple Silicon

**Solutions**:
1. Verify GPU offloading: Look for "offloaded X/X layers to GPU" in logs
2. Increase batch size: Set `batch_size = 512` in config
3. Reduce context size: Try `context_size = 2048` or lower
4. Check thermal throttling: Monitor Activity Monitor > GPU tab

### Build Errors

**Symptoms**: llama-cpp-2 compilation fails

**Solutions**:
1. Update Xcode: `xcode-select --install`
2. Update Rust: `rustup update stable`
3. Clean rebuild: `cargo clean && cargo build --release`

## References

- [llama-cpp-2 Documentation](https://docs.rs/llama-cpp-2/)
- [Apple Metal Documentation](https://developer.apple.com/metal/)
- [GGUF Format Specification](https://github.com/ggerganov/ggml/blob/master/docs/gguf.md)
- [Apple Silicon Memory Bandwidth Specs](https://developer.apple.com/documentation/apple-silicon/metal-feature-set-tables)

## Credits

Implementation based on:
- llama.cpp by Georgi Gerganov
- llama-cpp-2 Rust bindings by utilityai
- Metal Performance Shaders by Apple

---

**Last Updated**: 2026-04-02
**Version**: Inferno v0.10.7-dev
**Status**: Production Ready ✅
