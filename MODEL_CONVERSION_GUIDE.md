# Model Conversion Guide

This guide covers the comprehensive model conversion capabilities in Inferno, which supports real-time conversion between GGUF, ONNX, PyTorch, and SafeTensors formats with advanced optimization features.

## Quick Start

```bash
# Convert GGUF to ONNX with balanced optimization
inferno convert model input.gguf output.onnx --format onnx --optimization balanced

# Convert PyTorch to GGUF with quantization
inferno convert model model.pt model.gguf --format gguf --quantization q4_0

# Convert with custom settings
inferno convert model input.safetensors output.onnx \
  --format onnx \
  --precision fp16 \
  --context-length 4096 \
  --batch-size 32 \
  --preserve-metadata
```

## Supported Formats

### Input Formats
- **GGUF** (.gguf) - llama.cpp format with metadata
- **ONNX** (.onnx) - Open Neural Network Exchange format
- **PyTorch** (.pt, .pth) - PyTorch model files
- **SafeTensors** (.safetensors) - Hugging Face safe tensor format

### Output Formats
All input formats can be converted to any output format with intelligent handling of model architecture differences.

## Conversion Options

### Optimization Levels

#### Fast (`--optimization fast`)
- Minimal optimization for quick conversion
- Preserves original model structure
- Best for testing and development
- Conversion time: ~30% of balanced

#### Balanced (`--optimization balanced`) [Default]
- Good balance between conversion time and model performance
- Applies standard optimizations
- Recommended for most use cases
- Conversion time: baseline

#### Aggressive (`--optimization aggressive`)
- Maximum optimization for best runtime performance
- May take longer to convert
- Best for production deployments
- Conversion time: ~300% of balanced

### Quantization Options

#### 4-bit Quantization
- `q4_0`: 4-bit quantization, legacy format
- `q4_1`: 4-bit quantization with delta encoding

#### 5-bit Quantization
- `q5_0`: 5-bit quantization, legacy format
- `q5_1`: 5-bit quantization with delta encoding

#### 8-bit Quantization
- `q8_0`: 8-bit quantization

#### Floating Point
- `f16`: 16-bit floating point
- `f32`: 32-bit floating point (no quantization)

### Precision Options

- `fp16`: Half precision floating point
- `fp32`: Single precision floating point
- `int8`: 8-bit integer precision

## Advanced Features

### Metadata Preservation

```bash
# Preserve original model metadata during conversion
inferno convert model input.gguf output.onnx --preserve-metadata

# View model metadata
inferno models info model.onnx
```

### Custom Configuration

```bash
# Convert with specific context length and batch size
inferno convert model input.pt output.gguf \
  --format gguf \
  --context-length 8192 \
  --batch-size 64 \
  --optimization aggressive
```

### Batch Conversion

```bash
# Convert multiple models in a directory
inferno convert batch /path/to/models /path/to/output \
  --format onnx \
  --optimization balanced \
  --quantization q4_0
```

## Format-Specific Considerations

### GGUF Conversion

**To GGUF:**
- Automatically detects model architecture
- Preserves tokenizer configuration
- Optimizes for llama.cpp compatibility
- Supports all quantization options

**From GGUF:**
- Extracts embedded metadata
- Preserves model weights and structure
- Maintains tokenizer information

### ONNX Conversion

**To ONNX:**
- Optimizes computation graph
- Supports multiple execution providers
- Configures input/output shapes
- Enables hardware acceleration

**From ONNX:**
- Analyzes model structure automatically
- Detects model type (text generation, classification, embedding)
- Preserves optimization settings

### PyTorch Conversion

**To PyTorch:**
- Maintains original model architecture
- Preserves training metadata
- Supports dynamic shapes
- Compatible with PyTorch ecosystem

**From PyTorch:**
- Extracts model weights
- Analyzes computation graph
- Handles custom layers gracefully

### SafeTensors Conversion

**To SafeTensors:**
- Provides memory-safe format
- Faster loading times
- Built-in integrity checks
- Hugging Face compatible

**From SafeTensors:**
- Zero-copy tensor loading
- Automatic format validation
- Efficient memory usage

## Performance Benchmarks

### Conversion Times

| Source → Target | Fast | Balanced | Aggressive |
|----------------|------|----------|------------|
| GGUF → ONNX    | 30s  | 90s      | 280s       |
| PyTorch → GGUF | 45s  | 120s     | 350s       |
| ONNX → SafeTensors | 20s | 60s   | 180s       |

*Times shown for 7B parameter model on 16-core CPU*

### Model Size Impact

| Quantization | Size Reduction | Quality Loss |
|-------------|----------------|--------------|
| q4_0        | ~75%           | Minimal      |
| q4_1        | ~75%           | Minimal      |
| q5_0        | ~68%           | Very Low     |
| q5_1        | ~68%           | Very Low     |
| q8_0        | ~50%           | Negligible   |
| f16         | ~50%           | None         |

## Error Handling and Troubleshooting

### Common Issues

#### "Unsupported model architecture"
```bash
# Check model format and structure
inferno validate input_model.gguf

# Try with different optimization level
inferno convert model input.gguf output.onnx --optimization fast
```

#### "Out of memory during conversion"
```bash
# Reduce batch size
inferno convert model input.pt output.gguf --batch-size 16

# Use quantization to reduce memory usage
inferno convert model input.pt output.gguf --quantization q4_0
```

#### "Conversion failed with format error"
```bash
# Verify input file integrity
inferno validate input_model.pt

# Check file permissions
ls -la input_model.pt
```

### Validation

```bash
# Validate converted model
inferno validate output_model.onnx

# Compare models before and after conversion
inferno models compare original.gguf converted.onnx

# Test inference with converted model
inferno run --model converted.onnx --prompt "Test conversion"
```

## Configuration File

Create `.inferno.toml` for default conversion settings:

```toml
[conversion]
default_optimization = "balanced"
default_quantization = "q4_0"
default_precision = "fp16"
preserve_metadata = true
batch_size = 32
context_length = 4096

[conversion.format_specific]
# GGUF-specific settings
[conversion.format_specific.gguf]
enable_mmap = true
vocab_only = false

# ONNX-specific settings
[conversion.format_specific.onnx]
opset_version = 17
enable_optimization = true
execution_providers = ["CUDAExecutionProvider", "CPUExecutionProvider"]

# PyTorch-specific settings
[conversion.format_specific.pytorch]
torch_script = false
dynamic_shapes = true
```

## API Integration

### REST API

```bash
# Start conversion via API
curl -X POST http://localhost:8080/convert \
  -H "Content-Type: application/json" \
  -d '{
    "input_path": "/models/input.gguf",
    "output_path": "/models/output.onnx",
    "format": "onnx",
    "optimization": "balanced",
    "quantization": "q4_0"
  }'

# Check conversion status
curl http://localhost:8080/convert/status/job_id

# List conversion jobs
curl http://localhost:8080/convert/jobs
```

### Python Client

```python
from inferno_client import InfernoClient

client = InfernoClient("http://localhost:8080")

# Start conversion
job = client.convert_model(
    input_path="input.gguf",
    output_path="output.onnx",
    format="onnx",
    optimization="balanced",
    quantization="q4_0"
)

# Monitor progress
while not job.is_complete():
    print(f"Progress: {job.progress}%")
    time.sleep(5)

print(f"Conversion completed: {job.output_path}")
```

## Best Practices

### Model Selection
- Use GGUF for llama.cpp deployment
- Use ONNX for cross-platform inference
- Use SafeTensors for Hugging Face integration
- Use PyTorch for training and fine-tuning

### Optimization Strategy
- Start with balanced optimization
- Use aggressive optimization for production
- Test model quality after conversion
- Benchmark inference performance

### Resource Management
- Monitor memory usage during conversion
- Use appropriate batch sizes for your hardware
- Consider using quantization for large models
- Clean up temporary files after conversion

### Quality Assurance
- Always validate converted models
- Test inference with sample inputs
- Compare outputs before and after conversion
- Document conversion settings for reproducibility

## Integration with Other Features

### Cache Integration
Converted models are automatically added to the model cache:

```bash
# Enable persistent caching for converted models
inferno cache persist --include-conversions

# Warm cache with converted model
inferno cache warm --model converted_model.onnx
```

### Audit Logging
All conversions are logged in the audit system:

```bash
# View conversion audit logs
inferno audit logs --filter conversion

# Enable detailed conversion logging
inferno audit enable --include-conversions --encryption
```

### Batch Processing
Schedule regular model conversions:

```bash
# Create scheduled conversion job
inferno batch-queue create \
  --name "nightly_conversion" \
  --schedule "0 2 * * *" \
  --command "convert model /staging/model.pt /production/model.gguf --format gguf"
```