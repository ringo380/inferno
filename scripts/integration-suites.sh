#!/bin/bash
# The integration suites that `cargo test` skips by default (`test = false`
# in Cargo.toml), and the features they need to compile. verify.sh sources
# this file for the list and runs each suite; CI executes it directly to
# prove every suite still compiles, since `cargo clippy --all-targets` never
# sees a `test = false` target.

INTEGRATION_TESTS=(
    "integration_tests"
    "feature_integration_tests"
    "end_to_end_tests"
    "audit_system_integration_tests"
    "backend_integration_tests"
    "batch_processing_integration_tests"
    "batch_queue_integration_tests"
    "cache_persistence_integration_tests"
    "conversion_integration_tests"
    "cross_component_integration_tests"
    "performance_stress_tests"
    "platform_integration"
    "error_size_analysis"
    "metrics_thread_safety"
)

# BackendType::Gguf and ::Onnx are feature-gated, so the suites that name them
# fail to compile without these. Matches the features used by CI.
INTEGRATION_FEATURES="gguf,onnx"

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    set -euo pipefail
    args=()
    for suite in "${INTEGRATION_TESTS[@]}"; do
        args+=(--test "$suite")
    done
    echo "Compiling ${#INTEGRATION_TESTS[@]} integration suites with features: $INTEGRATION_FEATURES"
    cargo test --no-run --features "$INTEGRATION_FEATURES" "${args[@]}"
fi
