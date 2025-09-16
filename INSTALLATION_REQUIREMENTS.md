# Installation Requirements and Setup Guide

This guide provides comprehensive installation instructions for Inferno AI/ML runner with all dependencies required for real GGUF and ONNX backend functionality.

## System Requirements

### Minimum Requirements
- **CPU**: x86_64 or ARM64 processor
- **RAM**: 8 GB (16 GB recommended for large models)
- **Storage**: 20 GB free space (more for model storage)
- **OS**: Linux (Ubuntu 20.04+, CentOS 8+), macOS 10.15+, Windows 10+

### Recommended Requirements
- **CPU**: 8+ cores, 3.0+ GHz
- **RAM**: 32 GB or more
- **Storage**: NVMe SSD with 100+ GB free space
- **GPU**: NVIDIA RTX 3080+ / AMD RX 6800+ / Apple M1+ (for acceleration)

## Platform-Specific Prerequisites

### Linux (Ubuntu/Debian)

```bash
# Update system packages
sudo apt update && sudo apt upgrade -y

# Install essential build tools
sudo apt install -y \
    build-essential \
    cmake \
    pkg-config \
    git \
    curl \
    wget \
    unzip

# Install OpenSSL development libraries
sudo apt install -y \
    libssl-dev \
    libssl3 \
    ca-certificates

# Install Python development headers (for tokenizers)
sudo apt install -y \
    python3-dev \
    python3-pip

# GPU Support - NVIDIA CUDA
sudo apt install -y \
    nvidia-cuda-toolkit \
    nvidia-cuda-dev \
    libnvidia-compute-510

# GPU Support - Vulkan
sudo apt install -y \
    vulkan-tools \
    libvulkan-dev \
    vulkan-validationlayers-dev

# Additional dependencies
sudo apt install -y \
    libomp-dev \
    libopenblas-dev \
    liblapack-dev
```

### Linux (CentOS/RHEL/Fedora)

```bash
# Install development tools
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y \
    cmake \
    pkg-config \
    git \
    curl \
    wget \
    unzip

# Install OpenSSL
sudo dnf install -y \
    openssl-devel \
    ca-certificates

# Python development
sudo dnf install -y \
    python3-devel \
    python3-pip

# GPU Support - NVIDIA CUDA (requires NVIDIA repos)
sudo dnf install -y \
    cuda-toolkit \
    cuda-devel

# GPU Support - Vulkan
sudo dnf install -y \
    vulkan-tools \
    vulkan-devel \
    vulkan-validation-layers-devel

# Additional dependencies
sudo dnf install -y \
    libomp-devel \
    openblas-devel \
    lapack-devel
```

### macOS

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install \
    cmake \
    pkg-config \
    openssl \
    python3 \
    git \
    wget

# Install additional tools
brew install \
    libomp \
    openblas

# Metal Performance Shaders (built-in on macOS 10.13+)
# No additional installation required
```

### Windows

```powershell
# Install Chocolatey (package manager)
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Install build tools
choco install -y \
    visualstudio2022buildtools \
    cmake \
    git \
    python3 \
    rust \
    llvm

# Install CUDA Toolkit (for NVIDIA GPUs)
choco install -y cuda

# Or download from: https://developer.nvidia.com/cuda-downloads

# Install Visual C++ Redistributable
choco install -y vcredist140
```

## Rust Installation

Inferno requires Rust 1.70 or later.

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source the environment
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version

# Update to latest stable
rustup update stable
```

## Backend Dependencies

### GGUF Backend (llama.cpp)

The GGUF backend integrates with llama.cpp for real model inference.

#### Option 1: Automatic Integration (Recommended)

The GGUF backend will automatically compile llama.cpp during the build process.

```bash
# Build with GGUF support (automatic llama.cpp integration)
cargo build --release --features gguf
```

#### Option 2: Manual llama.cpp Installation

```bash
# Clone and build llama.cpp manually
git clone https://github.com/ggerganov/llama.cpp.git
cd llama.cpp

# Build with GPU support (NVIDIA)
make LLAMA_CUDA=1

# Build with GPU support (Apple Metal)
make LLAMA_METAL=1

# Build with GPU support (Vulkan)
make LLAMA_VULKAN=1

# Build CPU-only version
make

# Install to system location
sudo make install

# Set environment variable
export LLAMA_CPP_LIB_DIR=/usr/local/lib
```

### ONNX Backend (ONNX Runtime)

The ONNX backend uses the `ort` crate which automatically handles ONNX Runtime installation.

#### Automatic Installation (Default)

```bash
# Build with ONNX support (automatic ONNX Runtime download)
cargo build --release --features onnx
```

#### Manual ONNX Runtime Installation

For more control over the ONNX Runtime version:

**Linux:**
```bash
# Download ONNX Runtime
wget https://github.com/microsoft/onnxruntime/releases/download/v1.16.0/onnxruntime-linux-x64-1.16.0.tgz
tar -xzf onnxruntime-linux-x64-1.16.0.tgz

# Set environment variables
export ORT_LIB_LOCATION=/path/to/onnxruntime-linux-x64-1.16.0/lib
export LD_LIBRARY_PATH=$ORT_LIB_LOCATION:$LD_LIBRARY_PATH
```

**macOS:**
```bash
# Download ONNX Runtime
wget https://github.com/microsoft/onnxruntime/releases/download/v1.16.0/onnxruntime-osx-x86_64-1.16.0.tgz
tar -xzf onnxruntime-osx-x86_64-1.16.0.tgz

# Set environment variables
export ORT_LIB_LOCATION=/path/to/onnxruntime-osx-x86_64-1.16.0/lib
export DYLD_LIBRARY_PATH=$ORT_LIB_LOCATION:$DYLD_LIBRARY_PATH
```

**Windows:**
```powershell
# Download and extract ONNX Runtime
Invoke-WebRequest -Uri "https://github.com/microsoft/onnxruntime/releases/download/v1.16.0/onnxruntime-win-x64-1.16.0.zip" -OutFile "onnxruntime.zip"
Expand-Archive -Path "onnxruntime.zip" -DestinationPath "C:\onnxruntime"

# Set environment variable
$env:ORT_LIB_LOCATION = "C:\onnxruntime\onnxruntime-win-x64-1.16.0\lib"
$env:PATH = "$env:ORT_LIB_LOCATION;$env:PATH"
```

## GPU Acceleration Setup

### NVIDIA CUDA

**Linux:**
```bash
# Install CUDA Toolkit
wget https://developer.download.nvidia.com/compute/cuda/12.2.0/local_installers/cuda_12.2.0_535.54.03_linux.run
sudo sh cuda_12.2.0_535.54.03_linux.run

# Verify installation
nvidia-smi
nvcc --version

# Set environment variables
export CUDA_HOME=/usr/local/cuda
export PATH=$CUDA_HOME/bin:$PATH
export LD_LIBRARY_PATH=$CUDA_HOME/lib64:$LD_LIBRARY_PATH
```

**Windows:**
```powershell
# Download and install CUDA Toolkit from:
# https://developer.nvidia.com/cuda-downloads

# Verify installation
nvidia-smi
nvcc --version
```

### AMD ROCm (Linux)

```bash
# Install ROCm
wget -qO - https://repo.radeon.com/rocm/rocm.gpg.key | sudo apt-key add -
echo 'deb [arch=amd64] https://repo.radeon.com/rocm/apt/5.7/ ubuntu main' | sudo tee /etc/apt/sources.list.d/rocm.list
sudo apt update
sudo apt install -y rocm-dev rocm-libs

# Add user to render group
sudo usermod -a -G render $USER

# Set environment variables
export ROCM_PATH=/opt/rocm
export PATH=$ROCM_PATH/bin:$PATH
```

### Apple Metal (macOS)

Metal support is built-in on macOS 10.13+. No additional installation required.

```bash
# Verify Metal support
system_profiler SPDisplaysDataType | grep Metal
```

### Vulkan

**Linux:**
```bash
# Install Vulkan SDK
wget -qO - https://packages.lunarg.com/lunarg-signing-key-pub.asc | sudo apt-key add -
sudo wget -qO /etc/apt/sources.list.d/lunarg-vulkan-jammy.list https://packages.lunarg.com/vulkan/lunarg-vulkan-jammy.list
sudo apt update
sudo apt install -y vulkan-sdk

# Verify installation
vkvia
```

**Windows:**
```powershell
# Download and install Vulkan SDK from:
# https://vulkan.lunarg.com/sdk/home#windows

# Verify installation
vkvia.exe
```

## Building Inferno

### Full Build with All Features

```bash
# Clone the repository
git clone https://github.com/ringo380/inferno.git
cd inferno

# Build with all features
cargo build --release --all-features

# Or build specific feature combinations
cargo build --release --features "gguf,onnx,gpu,dashboard,audit"
```

### Feature Flags

Available Cargo features:

- `gguf`: Enable GGUF backend with llama.cpp integration
- `onnx`: Enable ONNX backend with ONNX Runtime
- `gpu`: Enable GPU acceleration support
- `dashboard`: Enable web dashboard API endpoints
- `audit`: Enable audit logging system
- `batch`: Enable batch processing and scheduling
- `cache`: Enable advanced caching with persistence
- `conversion`: Enable model format conversion
- `distributed`: Enable distributed inference capabilities
- `security`: Enable enhanced security features
- `monitoring`: Enable advanced monitoring and metrics

### Build Options

```bash
# Debug build (faster compilation, slower runtime)
cargo build --features "gguf,onnx"

# Release build (optimized for production)
cargo build --release --features "gguf,onnx,gpu"

# Build with specific optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release --features "gguf,onnx,gpu"

# Build with link-time optimization (slower build, better performance)
cargo build --release --features "gguf,onnx,gpu" --config profile.release.lto=true
```

## Docker Installation

### Using Pre-built Images

```bash
# Pull the latest image
docker pull inferno:latest

# Run with CPU support
docker run -d \
  --name inferno \
  -p 8080:8080 \
  -v /path/to/models:/models \
  inferno:latest serve

# Run with GPU support (NVIDIA)
docker run -d \
  --name inferno \
  --gpus all \
  -p 8080:8080 \
  -v /path/to/models:/models \
  inferno:latest serve --gpu-enabled
```

### Building Docker Image

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    cmake \
    pkg-config \
    libssl-dev \
    python3-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Build Inferno
RUN cargo build --release --all-features

# Runtime image
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    python3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/inferno /usr/local/bin/

# Create directories
RUN mkdir -p /models /cache /config

# Set environment variables
ENV INFERNO_MODELS_DIR=/models
ENV INFERNO_CACHE_DIR=/cache

# Expose port
EXPOSE 8080

# Default command
CMD ["inferno", "serve"]
```

```bash
# Build Docker image
docker build -t inferno:local .

# Build with GPU support
docker build --build-arg FEATURES="gguf,onnx,gpu" -t inferno:gpu .
```

## Kubernetes Deployment

### Basic Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: inferno
spec:
  replicas: 3
  selector:
    matchLabels:
      app: inferno
  template:
    metadata:
      labels:
        app: inferno
    spec:
      containers:
      - name: inferno
        image: inferno:latest
        ports:
        - containerPort: 8080
        env:
        - name: INFERNO_MODELS_DIR
          value: "/models"
        - name: INFERNO_CACHE_DIR
          value: "/cache"
        volumeMounts:
        - name: models
          mountPath: /models
        - name: cache
          mountPath: /cache
        resources:
          requests:
            cpu: 2
            memory: 4Gi
          limits:
            cpu: 4
            memory: 8Gi
      volumes:
      - name: models
        persistentVolumeClaim:
          claimName: inferno-models
      - name: cache
        persistentVolumeClaim:
          claimName: inferno-cache
```

### GPU-Enabled Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: inferno-gpu
spec:
  replicas: 1
  selector:
    matchLabels:
      app: inferno-gpu
  template:
    metadata:
      labels:
        app: inferno-gpu
    spec:
      containers:
      - name: inferno
        image: inferno:gpu
        ports:
        - containerPort: 8080
        env:
        - name: INFERNO_GPU_ENABLED
          value: "true"
        resources:
          requests:
            nvidia.com/gpu: 1
            cpu: 4
            memory: 8Gi
          limits:
            nvidia.com/gpu: 1
            cpu: 8
            memory: 16Gi
      nodeSelector:
        accelerator: nvidia-tesla-k80
```

## Configuration

### Environment Variables

```bash
# Core settings
export INFERNO_MODELS_DIR="/path/to/models"
export INFERNO_CACHE_DIR="/path/to/cache"
export INFERNO_CONFIG_FILE="/path/to/config.toml"
export INFERNO_LOG_LEVEL="info"

# Backend settings
export INFERNO_GPU_ENABLED="true"
export INFERNO_CUDA_VISIBLE_DEVICES="0,1"
export INFERNO_OMP_NUM_THREADS="8"

# Security settings
export INFERNO_AUTH_ENABLED="true"
export INFERNO_API_KEY="your-secure-api-key"
export INFERNO_TLS_ENABLED="true"

# Performance settings
export INFERNO_MAX_CONCURRENT_REQUESTS="100"
export INFERNO_REQUEST_TIMEOUT="300"
export INFERNO_CACHE_SIZE_GB="10"
```

### Configuration File

Create `/etc/inferno/config.toml`:

```toml
# Core configuration
models_dir = "/var/lib/inferno/models"
cache_dir = "/var/cache/inferno"
log_level = "info"

[server]
bind_address = "0.0.0.0"
port = 8080
max_concurrent_requests = 100
request_timeout_seconds = 300
tls_enabled = false

[backend_config]
gpu_enabled = true
cpu_threads = 8
context_size = 4096
batch_size = 32
memory_map = true

[cache]
enabled = true
type = "persistent"
max_size_gb = 10
compression = "zstd"
ttl_hours = 24

[security]
auth_enabled = false
rate_limiting_enabled = true
max_requests_per_minute = 1000

[observability]
prometheus_enabled = true
otel_enabled = false
grafana_enabled = false
```

## Verification

### Test Installation

```bash
# Verify Inferno installation
inferno --version

# Test basic functionality
inferno --help

# Validate configuration
inferno config validate

# Test backend functionality
inferno backends test

# Check GPU availability
inferno gpu info

# Run health check
inferno health check
```

### Performance Testing

```bash
# Benchmark system performance
inferno benchmark system

# Test model loading
inferno benchmark model-loading --model test-model.gguf

# Test inference performance
inferno benchmark inference --model test-model.gguf --prompts test-prompts.txt

# Stress test
inferno benchmark stress --duration 60s --concurrent 10
```

## Troubleshooting

### Common Installation Issues

**Rust compilation errors:**
```bash
# Update Rust toolchain
rustup update

# Clear cargo cache
cargo clean

# Rebuild with verbose output
cargo build --release --verbose
```

**Missing system dependencies:**
```bash
# Ubuntu/Debian
sudo apt install build-essential cmake pkg-config libssl-dev

# CentOS/RHEL
sudo dnf groupinstall "Development Tools"
sudo dnf install cmake pkg-config openssl-devel
```

**GPU acceleration not working:**
```bash
# Check GPU drivers
nvidia-smi  # For NVIDIA
rocm-smi   # For AMD
system_profiler SPDisplaysDataType | grep Metal  # For Apple

# Verify CUDA installation
nvcc --version

# Check Vulkan installation
vkvia
```

**ONNX Runtime issues:**
```bash
# Set explicit ONNX Runtime path
export ORT_LIB_LOCATION=/path/to/onnxruntime/lib

# Download specific version manually
cargo clean
ORT_STRATEGY=download cargo build --release
```

### Getting Help

- **Documentation**: Check the comprehensive guides in this repository
- **GitHub Issues**: Report bugs and request features
- **GitHub Discussions**: Community help and support
- **Stack Overflow**: Tag questions with `inferno-ai`