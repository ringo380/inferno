#!/bin/bash

# Inferno Performance Benchmarking Script
# This script runs comprehensive performance benchmarks and generates reports

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BENCH_RESULTS_DIR="$PROJECT_ROOT/benchmark_results"
BASELINE_DIR="$PROJECT_ROOT/performance_baseline"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1"
}

success() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] SUCCESS:${NC} $1"
}

# Help function
show_help() {
    cat << EOF
Inferno Performance Benchmarking Script

Usage: $0 [OPTIONS] [BENCHMARK_TYPE]

OPTIONS:
    -h, --help              Show this help message
    -o, --output DIR        Output directory (default: ./benchmark_results)
    -b, --baseline DIR      Baseline directory (default: ./performance_baseline)
    -t, --timeout SECONDS   Benchmark timeout (default: 300)
    -p, --profile           Enable CPU profiling with flamegraphs
    -c, --compare           Compare with baseline after running benchmarks
    -r, --report            Generate HTML report
    --ci                    CI mode (fail on performance regression)
    --targets FILE          Custom performance targets file

BENCHMARK_TYPE:
    all                     Run all benchmarks (default)
    inference               Inference performance benchmarks
    memory                  Memory usage benchmarks
    concurrent              Concurrent performance benchmarks
    cache                   Cache performance benchmarks
    profiling              CPU profiling benchmarks
    baseline                Establish performance baseline

Examples:
    $0                      # Run all benchmarks
    $0 inference            # Run only inference benchmarks
    $0 --profile all        # Run all benchmarks with profiling
    $0 --compare --ci       # Run benchmarks and fail on regression
    $0 baseline             # Establish new performance baseline

EOF
}

# Parse command line arguments
OUTPUT_DIR="$BENCH_RESULTS_DIR"
BASELINE_DIR="$BASELINE_DIR"
TIMEOUT=300
ENABLE_PROFILING=false
COMPARE_BASELINE=false
GENERATE_REPORT=false
CI_MODE=false
CUSTOM_TARGETS=""
BENCHMARK_TYPE="all"

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -b|--baseline)
            BASELINE_DIR="$2"
            shift 2
            ;;
        -t|--timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        -p|--profile)
            ENABLE_PROFILING=true
            shift
            ;;
        -c|--compare)
            COMPARE_BASELINE=true
            shift
            ;;
        -r|--report)
            GENERATE_REPORT=true
            shift
            ;;
        --ci)
            CI_MODE=true
            shift
            ;;
        --targets)
            CUSTOM_TARGETS="$2"
            shift 2
            ;;
        *)
            BENCHMARK_TYPE="$1"
            shift
            ;;
    esac
done

# Validate benchmark type
case $BENCHMARK_TYPE in
    all|inference|memory|concurrent|cache|profiling|baseline)
        ;;
    *)
        error "Invalid benchmark type: $BENCHMARK_TYPE"
        echo "Valid types: all, inference, memory, concurrent, cache, profiling, baseline"
        exit 1
        ;;
esac

# Setup
setup_environment() {
    log "Setting up benchmark environment"

    # Create output directories
    mkdir -p "$OUTPUT_DIR"
    mkdir -p "$BASELINE_DIR"

    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        error "cargo is not installed or not in PATH"
        exit 1
    fi

    # Check if we're in a Rust project
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        error "Not in a Rust project directory"
        exit 1
    fi

    # Build the project first
    log "Building project in release mode"
    cd "$PROJECT_ROOT"
    cargo build --release --quiet

    success "Environment setup complete"
}

# Run specific benchmark
run_benchmark() {
    local bench_name="$1"
    local output_file="$OUTPUT_DIR/${bench_name}_${TIMESTAMP}.json"

    log "Running $bench_name benchmark"

    # Set up profiling if enabled
    local cargo_args=""
    if [[ "$ENABLE_PROFILING" == "true" ]]; then
        cargo_args="--features criterion/profiling"
        export CRITERION_PROFILER="pprof"
    fi

    # Run the benchmark with timeout
    if timeout "$TIMEOUT" cargo bench --bench "$bench_name" $cargo_args -- --output-format json > "$output_file" 2>&1; then
        success "Completed $bench_name benchmark"

        # If profiling was enabled, move flamegraphs to output directory
        if [[ "$ENABLE_PROFILING" == "true" ]]; then
            local flamegraph_dir="$OUTPUT_DIR/flamegraphs_${TIMESTAMP}"
            mkdir -p "$flamegraph_dir"
            find target/criterion -name "*.svg" -exec cp {} "$flamegraph_dir/" \; 2>/dev/null || true
            find target/criterion -name "*.pb" -exec cp {} "$flamegraph_dir/" \; 2>/dev/null || true
        fi
    else
        error "Failed to run $bench_name benchmark (timeout: ${TIMEOUT}s)"
        return 1
    fi
}

# Run benchmarks based on type
run_benchmarks() {
    log "Starting benchmark suite: $BENCHMARK_TYPE"

    local benchmark_failed=false

    case $BENCHMARK_TYPE in
        all)
            for bench in inference_benchmark memory_benchmark concurrent_benchmark cache_benchmark profiling_benchmark; do
                if ! run_benchmark "$bench"; then
                    benchmark_failed=true
                fi
            done
            ;;
        inference)
            run_benchmark "inference_benchmark" || benchmark_failed=true
            ;;
        memory)
            run_benchmark "memory_benchmark" || benchmark_failed=true
            ;;
        concurrent)
            run_benchmark "concurrent_benchmark" || benchmark_failed=true
            ;;
        cache)
            run_benchmark "cache_benchmark" || benchmark_failed=true
            ;;
        profiling)
            ENABLE_PROFILING=true
            run_benchmark "profiling_benchmark" || benchmark_failed=true
            ;;
        baseline)
            establish_baseline || benchmark_failed=true
            ;;
    esac

    if [[ "$benchmark_failed" == "true" ]]; then
        error "Some benchmarks failed"
        return 1
    fi

    success "All benchmarks completed successfully"
}

# Establish performance baseline
establish_baseline() {
    log "Establishing performance baseline"

    local baseline_output="$BASELINE_DIR/baseline_${TIMESTAMP}.json"

    # Use the baseline subcommand if available
    if cargo run --release --quiet -- baseline --output "$baseline_output" 2>&1; then
        success "Performance baseline established"
    else
        error "Failed to establish baseline"
        return 1
    fi
}

# Compare with baseline
compare_with_baseline() {
    log "Comparing benchmark results with baseline"

    local latest_results=$(find "$OUTPUT_DIR" -name "*_${TIMESTAMP}.json" -type f | head -1)
    local baseline_file=$(find "$BASELINE_DIR" -name "baseline_*.json" -type f | sort | tail -1)

    if [[ -z "$latest_results" ]]; then
        warn "No recent benchmark results found"
        return 0
    fi

    if [[ -z "$baseline_file" ]]; then
        warn "No baseline file found. Run '$0 baseline' first."
        return 0
    fi

    log "Comparing $latest_results with $baseline_file"

    # This would be implemented in the Rust code
    if cargo run --release --quiet -- compare-performance --current "$latest_results" --baseline "$baseline_file" 2>&1; then
        success "Performance comparison completed"
    else
        error "Performance regression detected"
        if [[ "$CI_MODE" == "true" ]]; then
            exit 1
        fi
    fi
}

# Generate HTML report
generate_html_report() {
    log "Generating HTML benchmark report"

    local report_file="$OUTPUT_DIR/benchmark_report_${TIMESTAMP}.html"

    # Create a simple HTML report
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Inferno Performance Benchmark Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { background-color: #f0f0f0; padding: 20px; border-radius: 5px; }
        .section { margin: 20px 0; }
        .benchmark-result { background-color: #f9f9f9; padding: 15px; margin: 10px 0; border-radius: 5px; }
        .success { color: green; }
        .warning { color: orange; }
        .error { color: red; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Inferno Performance Benchmark Report</h1>
        <p>Generated: $(date)</p>
        <p>Benchmark Type: $BENCHMARK_TYPE</p>
        <p>Profiling Enabled: $ENABLE_PROFILING</p>
    </div>

    <div class="section">
        <h2>Benchmark Results</h2>
        <p>Results are stored in: $OUTPUT_DIR</p>
        <p>Baseline directory: $BASELINE_DIR</p>
    </div>

    <div class="section">
        <h2>Performance Targets</h2>
        <table>
            <tr><th>Metric</th><th>Target</th><th>Description</th></tr>
            <tr><td>Inference Latency</td><td>&lt; 100ms</td><td>Average inference time for most models</td></tr>
            <tr><td>Memory Efficiency</td><td>50% reduction</td><td>Memory usage optimization target</td></tr>
            <tr><td>Throughput</td><td>&gt; 1000 RPS</td><td>Requests per second under load</td></tr>
            <tr><td>Model Loading</td><td>&lt; 5 seconds</td><td>Time to load most models</td></tr>
            <tr><td>Cache Hit Ratio</td><td>&gt; 80%</td><td>Cache effectiveness for repeated requests</td></tr>
        </table>
    </div>

    <div class="section">
        <h2>Files Generated</h2>
        <ul>
EOF

    # List generated files
    for file in "$OUTPUT_DIR"/*_${TIMESTAMP}.*; do
        if [[ -f "$file" ]]; then
            echo "            <li>$(basename "$file")</li>" >> "$report_file"
        fi
    done

    cat >> "$report_file" << EOF
        </ul>
    </div>

    <div class="section">
        <h2>Next Steps</h2>
        <ul>
            <li>Review benchmark results in JSON files</li>
            <li>Compare with performance targets</li>
            <li>Investigate any performance regressions</li>
            <li>Update baseline if improvements are confirmed</li>
        </ul>
    </div>
</body>
</html>
EOF

    success "HTML report generated: $report_file"
}

# Cleanup function
cleanup() {
    log "Cleaning up temporary files"
    # Remove any temporary benchmark files
    find "$PROJECT_ROOT/target" -name "*.tmp" -delete 2>/dev/null || true
}

# Signal handlers
trap cleanup EXIT
trap 'error "Benchmark interrupted"; exit 130' INT TERM

# Main execution
main() {
    log "Starting Inferno performance benchmarking"
    log "Benchmark type: $BENCHMARK_TYPE"
    log "Output directory: $OUTPUT_DIR"
    log "Baseline directory: $BASELINE_DIR"
    log "Profiling enabled: $ENABLE_PROFILING"
    log "CI mode: $CI_MODE"

    setup_environment

    if ! run_benchmarks; then
        error "Benchmark execution failed"
        exit 1
    fi

    if [[ "$COMPARE_BASELINE" == "true" ]]; then
        compare_with_baseline
    fi

    if [[ "$GENERATE_REPORT" == "true" ]]; then
        generate_html_report
    fi

    success "Benchmarking completed successfully"
    log "Results available in: $OUTPUT_DIR"

    if [[ "$ENABLE_PROFILING" == "true" ]]; then
        log "Flamegraphs and profiles available in: $OUTPUT_DIR/flamegraphs_${TIMESTAMP}"
    fi
}

# Run main function
main "$@"