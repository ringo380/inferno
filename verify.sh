#!/usr/bin/env bash
#
# Inferno Verification Script
# Runs comprehensive checks: build, test, lint, format, and security audit
#

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track overall status
FAILED_CHECKS=0

echo -e "${BLUE}🔥 Inferno Verification Script${NC}"
echo -e "${BLUE}================================${NC}\n"

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    fi
}

# 1. Check Rust toolchain
echo -e "${YELLOW}Checking Rust toolchain...${NC}"
if command -v rustc &> /dev/null && command -v cargo &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    print_status 0 "Rust toolchain: $RUST_VERSION"
else
    print_status 1 "Rust toolchain not found"
fi
echo

# 2. Format check
echo -e "${YELLOW}Checking code formatting...${NC}"
if cargo fmt -- --check > /dev/null 2>&1; then
    print_status 0 "Code formatting (cargo fmt)"
else
    echo -e "${YELLOW}  Running cargo fmt to fix formatting...${NC}"
    cargo fmt
    print_status 0 "Code formatting fixed"
fi
echo

# 3. Clippy lint
echo -e "${YELLOW}Running linter (clippy)...${NC}"
if cargo clippy --all-targets --all-features -- -D warnings > /dev/null 2>&1; then
    print_status 0 "Clippy linting passed"
else
    echo -e "${YELLOW}  Clippy warnings found. Run 'cargo clippy --all-targets --all-features' for details.${NC}"
    print_status 1 "Clippy linting"
fi
echo

# 4. Build check
echo -e "${YELLOW}Building project (debug)...${NC}"
if cargo build --lib > /dev/null 2>&1; then
    print_status 0 "Debug build (lib)"
else
    print_status 1 "Debug build (lib)"
fi
echo

# 5. Build check (release)
echo -e "${YELLOW}Building project (release)...${NC}"
if cargo build --lib --release > /dev/null 2>&1; then
    print_status 0 "Release build (lib)"
else
    print_status 1 "Release build (lib)"
fi
echo

# 6. Run tests
echo -e "${YELLOW}Running tests...${NC}"

# Run fast unit tests
if cargo test --lib > /dev/null 2>&1; then
    print_status 0 "Unit tests (lib)"
else
    print_status 1 "Unit tests (lib)"
fi

# Run default test suite (basic functionality + component tests)
if cargo test > /dev/null 2>&1; then
    print_status 0 "Fast tests (basic + component)"
else
    print_status 1 "Fast tests"
fi

# Run all integration tests explicitly (these are disabled by default)
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
    "dashboard_api_tests"
    "dashboard_api_workflow_tests"
    "performance_stress_tests"
    "platform_integration"
    "error_size_analysis"
    "metrics_thread_safety"
)

for test in "${INTEGRATION_TESTS[@]}"; do
    if cargo test --test "$test" > /dev/null 2>&1; then
        print_status 0 "Integration: $test"
    else
        print_status 1 "Integration: $test"
    fi
done
echo

# 7. Security audit (optional - install with: cargo install cargo-audit)
echo -e "${YELLOW}Security audit...${NC}"
if command -v cargo-audit &> /dev/null; then
    if cargo audit > /dev/null 2>&1; then
        print_status 0 "Security audit (cargo audit)"
    else
        echo -e "${YELLOW}  Security vulnerabilities found. Run 'cargo audit' for details.${NC}"
        print_status 1 "Security audit"
    fi
else
    echo -e "${YELLOW}  cargo-audit not installed (optional). Install with: cargo install cargo-audit${NC}"
    print_status 0 "Security audit (skipped)"
fi
echo

# 8. Dependency check (optional - install with: cargo install cargo-outdated)
echo -e "${YELLOW}Checking for outdated dependencies...${NC}"
if command -v cargo-outdated &> /dev/null; then
    if cargo outdated --root-deps-only > /dev/null 2>&1; then
        print_status 0 "Dependencies up to date"
    else
        echo -e "${YELLOW}  Outdated dependencies found. Run 'cargo outdated' for details.${NC}"
        print_status 0 "Dependency check (warnings only)"
    fi
else
    echo -e "${YELLOW}  cargo-outdated not installed (optional). Install with: cargo install cargo-outdated${NC}"
    print_status 0 "Dependency check (skipped)"
fi
echo

# Summary
echo -e "${BLUE}================================${NC}"
if [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILED_CHECKS check(s) failed${NC}"
    echo -e "${YELLOW}Run individual commands for more details:${NC}"
    echo "  - cargo fmt -- --check"
    echo "  - cargo clippy --all-targets --all-features"
    echo "  - cargo build --lib"
    echo "  - cargo test --lib"
    echo "  - cargo audit"
    exit 1
fi
