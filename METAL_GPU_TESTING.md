# Metal GPU Testing Guide

## Implementation Status ✅

The Metal GPU acceleration for GGUF models is **fully implemented** and ready for testing.

### What's Been Implemented

1. **Real Metal GPU Inference** (`src/backends/gguf.rs`)
   - Full integration with llama-cpp-2's Metal backend
   - GPU layer configuration (999 layers for maximum Metal utilization)
   - Thread-safe architecture using Arc<LlamaBackend>
   - Per-inference LlamaContext creation to handle !Send types
   - Complete token generation loop with greedy sampling
   - Async inference via tokio::spawn_blocking

2. **Desktop App Integration** (`dashboard/src-tauri/src/backend_manager.rs`)
   - Automatic Metal GPU enablement on macOS
   - BackendConfig with `gpu_enabled = true` on macOS
   - Context size: 2048, Batch size: 512

3. **Thread Safety Architecture**
   - LlamaBackend wrapped in Arc for safe sharing across threads
   - LlamaContext created per-inference within spawn_blocking
   - Proper async/await integration with blocking operations

## Quick Test (CLI)

### 1. Download a Test Model

```bash
# Create models directory
mkdir -p models && cd models

# Download TinyLlama (638MB, fast inference)
hf download TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF \
  tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf \
  --local-dir .

cd ..
```

### 2. Build and Run

```bash
# Build with GGUF support (first build takes 3-5 minutes)
cargo build --features gguf --release

# Run inference with Metal GPU
INFERNO_MODELS_DIR="models" \
RUST_LOG=info \
./target/release/inferno run \
  --model tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf \
  --prompt "What is 2+2? Explain briefly." \
  --backend gguf
```

### 3. Verify Metal GPU Usage

Look for these indicators in the output:
- `ggml_metal_init: loaded Metal backend` - Metal backend initialized
- `ggml_metal_add_buffer: allocated 'data' buffer` - GPU memory allocated
- `llama_new_context_with_model: n_ctx = 2048` - Context created
- GPU layer messages showing offloading to Metal

### 4. Performance Benchmarking

```bash
# Benchmark tokens/second with Metal GPU
INFERNO_MODELS_DIR="models" \
./target/release/inferno bench \
  --model tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf \
  --backend gguf
```

**Expected Performance (TinyLlama 1.1B Q4_K_M)**:
- M1 Max: ~60-80 tokens/sec
- M2 Max: ~80-100 tokens/sec
- M3 Max: ~100-120 tokens/sec
- M4 Max: ~120-150 tokens/sec

## Integration Test

Run the Metal-specific integration test:

```bash
# Run Metal GPU integration test
cargo test --test metal_gpu_test --features gguf -- --nocapture

# Expected output:
# ✅ Metal GPU inference successful!
#    Prompt: What is 2+2?
#    Response: [model response]
```

## Desktop App Testing

### 1. Build Desktop App

```bash
cd dashboard

# Install dependencies (first time only)
npm install

# Run development build with Metal GPU
npm run tauri dev
```

### 2. Test in Desktop UI

1. Open the Models panel
2. Click "Discover Models" - should find TinyLlama
3. Click "Load Model" - backend will automatically enable Metal GPU on macOS
4. Go to Inference panel
5. Enter prompt: "What is 2+2?"
6. Click "Run Inference"
7. Check Activity Log for Metal initialization messages

### 3. Monitor GPU Usage

```bash
# In another terminal, monitor GPU usage
sudo powermetrics --samplers gpu_power -i 1000

# Look for:
# - GPU active residency increasing during inference
# - ANE (Apple Neural Engine) usage if available
# - Power consumption spike during inference
```

## Troubleshooting

### Issue: "Model not found"

```bash
# List available models
INFERNO_MODELS_DIR="models" ./target/release/inferno models list
```

### Issue: "Backend not loaded"

Make sure you built with the gguf feature:
```bash
cargo build --features gguf --release
```

### Issue: Slow performance

Check that Metal GPU is actually being used:
```bash
# Enable debug logging
RUST_LOG=debug ./target/release/inferno run ...

# Look for:
# - "Using N GPU layers" where N > 0
# - "Metal backend loaded successfully"
```

### Issue: Out of memory

Reduce context size in config:
```toml
# .inferno.toml
[backend_config]
context_size = 1024  # Reduce from 2048
batch_size = 256     # Reduce from 512
```

## Performance Comparison

### Test CPU vs GPU Performance

```bash
# CPU only (gpu_enabled = false)
time ./target/release/inferno run \
  --model tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf \
  --prompt "Write a short poem" \
  --backend gguf

# GPU enabled (default on macOS)
time ./target/release/inferno run \
  --model tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf \
  --prompt "Write a short poem" \
  --backend gguf
```

Expected speedup: **3-5x faster with Metal GPU** vs CPU-only.

## Next Steps

1. ✅ Metal GPU implementation complete
2. ⏳ Add temperature-based sampling (currently using greedy)
3. ⏳ True streaming with tokio channels from spawn_blocking
4. ⏳ GPU memory monitoring in desktop UI
5. ⏳ Performance profiling across different Apple Silicon chips
6. ⏳ Benchmark against larger models (7B, 13B, 70B)

## Technical Details

### How It Works

1. **Initialization**: LlamaBackend::init() creates global backend (GPU setup)
2. **Model Loading**: Model loaded with `n_gpu_layers = 999` (all layers on GPU)
3. **Context Creation**: Per-inference context with configurable size
4. **Inference Loop**:
   - Tokenize input with AddBos::Always
   - Create LlamaBatch and add tokens
   - Decode batch (runs on Metal GPU)
   - Sample next token (greedy: pick highest probability)
   - Repeat until EOS or max_tokens reached
5. **Cleanup**: Context dropped after inference, memory freed

### Thread Safety Model

- **LlamaBackend**: Wrapped in Arc, shared across async boundaries
- **LlamaModel**: Wrapped in Arc, cloned for spawn_blocking
- **LlamaContext**: Created inside spawn_blocking (avoids Send issues)
- **Inference**: Runs in blocking thread pool, returns via Result

### Memory Usage

- TinyLlama 1.1B Q4_K_M: ~700MB
- Context (2048): ~256MB
- Batch (512): ~64MB
- **Total**: ~1GB for small model

For larger models:
- 7B Q4_K_M: ~4GB
- 13B Q4_K_M: ~8GB
- 70B Q4_K_M: ~40GB (requires M3/M4 Max/Ultra with 64GB+ unified memory)
