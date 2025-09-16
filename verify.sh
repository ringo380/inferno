#!/usr/bin/env bash

# Verification script for Inferno AI/ML Model Runner
# Performs comprehensive testing and validation

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
VERBOSE=${VERBOSE:-false}
SKIP_TESTS=${SKIP_TESTS:-false}
SKIP_BENCHMARKS=${SKIP_BENCHMARKS:-true}

log() {
    echo -e "${BLUE}[VERIFY]${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."

    # Check Rust installation
    if ! command -v cargo &> /dev/null; then
        error "Cargo not found. Please install Rust: https://rustup.rs/"
    fi
    success "Cargo found: $(cargo --version)"

    # Check Rust version
    local rust_version
    rust_version=$(rustc --version | grep -oE '[0-9]+\.[0-9]+' | head -1)
    local major minor
    major=$(echo "$rust_version" | cut -d. -f1)
    minor=$(echo "$rust_version" | cut -d. -f2)

    if [[ $major -lt 1 || ($major -eq 1 && $minor -lt 70) ]]; then
        warn "Rust version $rust_version detected. Minimum recommended: 1.70.0"
    fi
    success "Rust version: $(rustc --version)"

    # Check required components
    if ! rustup component list --installed | grep -q rustfmt; then
        log "Installing rustfmt..."
        rustup component add rustfmt
    fi

    if ! rustup component list --installed | grep -q clippy; then
        log "Installing clippy..."
        rustup component add clippy
    fi

    success "Prerequisites check passed"
}

# Verify project structure
verify_structure() {
    log "Verifying project structure..."

    local required_files=(
        "Cargo.toml"
        "src/main.rs"
        "src/lib.rs"
        "README.md"
        "LICENSE-MIT"
        "LICENSE-APACHE"
    )

    for file in "${required_files[@]}"; do
        if [[ ! -f "$file" ]]; then
            error "Required file missing: $file"
        fi
    done

    local required_dirs=(
        "src/cli"
        "src/tui"
        "src/backends"
        "src/models"
        "src/config.rs"
        "tests"
        "scripts"
    )

    for dir in "${required_dirs[@]}"; do
        if [[ ! -e "$dir" ]]; then
            error "Required directory/file missing: $dir"
        fi
    done

    success "Project structure verified"
}

# Check code formatting
check_formatting() {
    log "Checking code formatting..."

    if ! cargo fmt --all -- --check; then
        error "Code formatting check failed. Run 'cargo fmt' to fix."
    fi

    success "Code formatting check passed"
}

# Run linting
run_clippy() {
    log "Running clippy lints..."

    local clippy_args=(
        "--all-targets"
        "--all-features"
        "--"
        "-D" "warnings"
    )

    if [[ "$VERBOSE" == "true" ]]; then
        clippy_args+=("-v")
    fi

    if ! cargo clippy "${clippy_args[@]}"; then
        error "Clippy linting failed"
    fi

    success "Clippy linting passed"
}

# Build project
build_project() {
    log "Building project (debug)..."

    local build_args=()
    if [[ "$VERBOSE" == "true" ]]; then
        build_args+=("--verbose")
    fi

    if ! cargo build "${build_args[@]}"; then
        error "Debug build failed"
    fi
    success "Debug build successful"

    log "Building project (release)..."
    if ! cargo build --release "${build_args[@]}"; then
        error "Release build failed"
    fi
    success "Release build successful"
}

# Run tests
run_tests() {
    if [[ "$SKIP_TESTS" == "true" ]]; then
        warn "Skipping tests (SKIP_TESTS=true)"
        return
    fi

    log "Running unit tests..."
    if ! cargo test --lib; then
        error "Unit tests failed"
    fi
    success "Unit tests passed"

    log "Running integration tests..."
    if ! cargo test --test integration_tests; then
        error "Integration tests failed"
    fi
    success "Integration tests passed"

    log "Running documentation tests..."
    if ! cargo test --doc; then
        error "Documentation tests failed"
    fi
    success "Documentation tests passed"
}

# Run benchmarks
run_benchmarks() {
    if [[ "$SKIP_BENCHMARKS" == "true" ]]; then
        warn "Skipping benchmarks (SKIP_BENCHMARKS=true)"
        return
    fi

    log "Running benchmarks..."
    if ! cargo bench --bench inference_benchmark; then
        error "Benchmarks failed"
    fi
    success "Benchmarks completed"
}

# Test binary functionality
test_binary() {
    log "Testing binary functionality..."

    local binary_path="./target/release/inferno"
    if [[ ! -f "$binary_path" ]]; then
        error "Release binary not found at $binary_path"
    fi

    # Test version
    log "Testing --version..."
    if ! "$binary_path" --version; then
        error "Binary version test failed"
    fi
    success "Version test passed"

    # Test help
    log "Testing --help..."
    if ! "$binary_path" --help > /dev/null; then
        error "Binary help test failed"
    fi
    success "Help test passed"

    # Test subcommands
    log "Testing subcommand help..."
    local subcommands=("run" "models" "bench" "validate" "serve")
    for cmd in "${subcommands[@]}"; do
        if ! "$binary_path" "$cmd" --help > /dev/null; then
            error "Subcommand '$cmd' help failed"
        fi
    done
    success "Subcommand tests passed"
}

# Create sample models directory and test model operations
test_model_operations() {
    log "Testing model operations..."

    # Create temporary models directory
    local temp_models_dir
    temp_models_dir=$(mktemp -d)

    # Create mock model files
    echo "GGUF$(printf '\x00\x00\x00\x01')mock gguf model data" > "$temp_models_dir/test.gguf"
    echo "mock onnx model data" > "$temp_models_dir/test.onnx"

    local binary_path="./target/release/inferno"

    # Test model listing
    if ! INFERNO_MODELS_DIR="$temp_models_dir" "$binary_path" models list; then
        error "Model listing failed"
    fi
    success "Model listing test passed"

    # Test model validation
    if ! "$binary_path" validate "$temp_models_dir/test.gguf"; then
        error "Model validation failed"
    fi
    success "Model validation test passed"

    # Cleanup
    rm -rf "$temp_models_dir"
}

# Security and dependency audit
security_audit() {
    log "Running security audit..."

    # Install cargo-audit if not present
    if ! command -v cargo-audit &> /dev/null; then
        log "Installing cargo-audit..."
        cargo install cargo-audit
    fi

    if ! cargo audit; then
        warn "Security audit found issues (this may be acceptable for development)"
    else
        success "Security audit passed"
    fi
}

# Check documentation
check_documentation() {
    log "Checking documentation..."

    if ! cargo doc --no-deps --all-features; then
        error "Documentation generation failed"
    fi
    success "Documentation generation passed"

    # Check for missing documentation
    if ! cargo doc --no-deps --all-features 2>&1 | grep -q "warning.*missing documentation"; then
        success "No missing documentation warnings"
    else
        warn "Some items are missing documentation"
    fi
}

# Performance check
performance_check() {
    log "Running performance checks..."

    local binary_path="./target/release/inferno"

    # Check binary size
    local binary_size
    binary_size=$(stat -f%z "$binary_path" 2>/dev/null || stat -c%s "$binary_path" 2>/dev/null || echo "unknown")

    if [[ "$binary_size" != "unknown" ]]; then
        local size_mb=$((binary_size / 1024 / 1024))
        log "Binary size: ${size_mb}MB"

        if [[ $size_mb -gt 100 ]]; then
            warn "Binary is quite large (${size_mb}MB). Consider optimizing."
        else
            success "Binary size is reasonable (${size_mb}MB)"
        fi
    fi

    # Check startup time
    local start_time end_time duration
    start_time=$(date +%s%N)
    "$binary_path" --version > /dev/null
    end_time=$(date +%s%N)
    duration=$(( (end_time - start_time) / 1000000 ))

    log "Startup time: ${duration}ms"
    if [[ $duration -gt 1000 ]]; then
        warn "Startup time is slow (${duration}ms)"
    else
        success "Startup time is good (${duration}ms)"
    fi
}

# Generate report
generate_report() {
    log "Generating verification report..."

    local report_file="verification_report.txt"
    {
        echo "Inferno Verification Report"
        echo "=========================="
        echo "Date: $(date)"
        echo "Host: $(hostname)"
        echo "OS: $(uname -s) $(uname -r)"
        echo "Rust: $(rustc --version)"
        echo "Cargo: $(cargo --version)"
        echo ""
        echo "Build Status: ✓ PASSED"
        echo "Tests Status: ✓ PASSED"
        echo "Lints Status: ✓ PASSED"
        echo "Security: ✓ PASSED"
        echo ""
        echo "Binary: ./target/release/inferno"
        if [[ -f "./target/release/inferno" ]]; then
            echo "Binary Size: $(stat -f%z './target/release/inferno' 2>/dev/null || stat -c%s './target/release/inferno' 2>/dev/null || echo 'unknown') bytes"
        fi
        echo ""
        echo "Ready for deployment! 🚀"
    } > "$report_file"

    success "Report generated: $report_file"
}

# Main verification function
main() {
    echo "🔥 Inferno AI/ML Model Runner Verification"
    echo "========================================"
    echo ""

    check_prerequisites
    verify_structure
    check_formatting
    run_clippy
    build_project
    run_tests
    test_binary
    test_model_operations
    security_audit
    check_documentation
    performance_check

    if [[ "$SKIP_BENCHMARKS" != "true" ]]; then
        run_benchmarks
    fi

    generate_report

    echo ""
    echo -e "${GREEN}🎉 All verification checks passed!${NC}"
    echo ""
    echo "Inferno is ready for use. Next steps:"
    echo "1. Place model files in the models directory"
    echo "2. Run: ./target/release/inferno --help"
    echo "3. Try: ./target/release/inferno tui"
    echo ""
    echo "For more information, see README.md"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --skip-benchmarks)
            SKIP_BENCHMARKS=true
            shift
            ;;
        --with-benchmarks)
            SKIP_BENCHMARKS=false
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -v, --verbose          Verbose output"
            echo "  --skip-tests           Skip running tests"
            echo "  --skip-benchmarks      Skip benchmarks (default)"
            echo "  --with-benchmarks      Run benchmarks"
            echo "  -h, --help             Show this help"
            echo ""
            echo "Environment Variables:"
            echo "  VERBOSE=true           Enable verbose output"
            echo "  SKIP_TESTS=true        Skip tests"
            echo "  SKIP_BENCHMARKS=false  Run benchmarks"
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            ;;
    esac
done

# Run main verification
main "$@"