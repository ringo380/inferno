#!/usr/bin/env bash
# Standardized Metal GPU benchmark for Apple Silicon.
#
# Usage:
#   ./scripts/benchmark_metal.sh [MODEL_PATH] [ITERATIONS] [TOKENS]
#
# Defaults:
#   MODEL_PATH  test_models/TinyLlama-1.1B-Chat-v1.0.Q4_K_M.gguf
#   ITERATIONS  10
#   TOKENS      100
#
# Output:
#   Prints results to stdout and writes a timestamped JSON file.
#   Share the JSON at: https://github.com/ringo380/inferno/issues/7

set -euo pipefail

MODEL="${1:-test_models/TinyLlama-1.1B-Chat-v1.0.Q4_K_M.gguf}"
ITERATIONS="${2:-10}"
TOKENS="${3:-100}"
WARMUP=3
OUTPUT="benchmark_results_$(date +%Y%m%d_%H%M%S).json"

if [[ ! -f "$MODEL" ]]; then
    echo "Error: model file not found: $MODEL"
    echo "Usage: $0 [MODEL_PATH] [ITERATIONS] [TOKENS]"
    exit 1
fi

echo "=== Inferno Metal GPU Benchmark ==="
echo "Model:      $MODEL"
echo "Iterations: $ITERATIONS (+ $WARMUP warmup)"
echo "Tokens:     $TOKENS per iteration"
echo ""

cargo run --release -- bench \
    --model "$MODEL" \
    --iterations "$ITERATIONS" \
    --warmup "$WARMUP" \
    --tokens "$TOKENS" \
    --output-json "$OUTPUT"

echo ""
echo "JSON results: $OUTPUT"
echo ""
echo "To contribute results for issue #7, share the JSON file at:"
echo "  https://github.com/ringo380/inferno/issues/7"
echo ""
echo "Include your chip model (e.g. M2 Pro), GPU core count, and RAM size."
