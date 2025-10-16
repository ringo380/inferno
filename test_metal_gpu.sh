#!/bin/bash
# Metal GPU Testing Script for Inferno
# Run this after the build completes

set -e

echo "🔥 Inferno Metal GPU Test Suite"
echo "================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if binary exists
if [ ! -f "target/release/inferno" ]; then
    echo -e "${RED}❌ Binary not found. Building now...${NC}"
    echo "This will take 3-5 minutes on first build."
    cargo build --features gguf --bin inferno --release
fi

echo -e "${GREEN}✅ Binary found${NC}"
echo ""

# Check if model exists
if [ ! -f "models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf" ]; then
    echo -e "${YELLOW}⚠️  TinyLlama model not found${NC}"
    echo "Downloading TinyLlama 1.1B Chat (Q4_K_M, ~638MB)..."
    mkdir -p models && cd models
    hf download TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF \
        tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf \
        --local-dir .
    cd ..
fi

echo -e "${GREEN}✅ Model found${NC}"
echo ""

# Test 1: Simple inference
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 1: Simple Inference${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Prompt: 'What is 2+2? Answer briefly.'"
echo ""

INFERNO_MODELS_DIR="models" \
RUST_LOG=info \
./target/release/inferno run \
    --model tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf \
    --prompt "What is 2+2? Answer briefly." \
    --backend gguf \
    2>&1 | tee /tmp/inference_test.log

echo ""

# Check for Metal GPU usage
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Metal GPU Verification${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

if grep -q "Metal" /tmp/inference_test.log; then
    echo -e "${GREEN}✅ Metal GPU detected in logs${NC}"
    grep -i "metal\|gpu" /tmp/inference_test.log | head -10
else
    echo -e "${YELLOW}⚠️  No Metal references found (check logs manually)${NC}"
fi

echo ""

# Test 2: Performance benchmark
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 2: Performance Benchmark${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

INFERNO_MODELS_DIR="models" \
./target/release/inferno bench \
    --model tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf \
    --backend gguf \
    2>&1 | tee /tmp/benchmark_test.log

echo ""

# Test 3: Integration test
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 3: Integration Test${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

cargo test --test metal_gpu_test --features gguf -- --nocapture

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ All Tests Complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Test logs saved to:"
echo "  - /tmp/inference_test.log"
echo "  - /tmp/benchmark_test.log"
echo ""
echo "For more details, see METAL_GPU_TESTING.md"
