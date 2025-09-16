#!/usr/bin/env bash

# Build script for Inferno AI/ML model runner
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="inferno"
BUILD_DIR="target"
RELEASE_DIR="release"

# Default values
BUILD_TYPE="debug"
TARGET=""
FEATURES=""
VERBOSE=false
CROSS_COMPILE=false

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  -r, --release          Build in release mode"
    echo "  -t, --target TARGET    Build for specific target"
    echo "  -f, --features FEAT    Enable specific features"
    echo "  -v, --verbose          Verbose output"
    echo "  -c, --cross            Use cross for compilation"
    echo "  -h, --help             Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 --release"
    echo "  $0 --target x86_64-unknown-linux-musl --cross"
    echo "  $0 --features download --release"
}

log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -r|--release)
            BUILD_TYPE="release"
            shift
            ;;
        -t|--target)
            TARGET="$2"
            shift 2
            ;;
        -f|--features)
            FEATURES="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -c|--cross)
            CROSS_COMPILE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            error "Unknown option $1"
            ;;
    esac
done

# Check dependencies
check_dependencies() {
    log "Checking dependencies..."

    if ! command -v cargo &> /dev/null; then
        error "cargo not found. Please install Rust: https://rustup.rs/"
    fi

    if [[ "$CROSS_COMPILE" == "true" ]] && ! command -v cross &> /dev/null; then
        warn "cross not found. Installing..."
        cargo install cross
    fi

    # Check for required system libraries
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if ! pkg-config --exists openssl; then
            warn "OpenSSL development libraries not found. Please install libssl-dev or openssl-devel"
        fi
    fi
}

# Clean build directory
clean_build() {
    if [[ -d "$BUILD_DIR" ]]; then
        log "Cleaning build directory..."
        cargo clean
    fi
}

# Build the project
build_project() {
    log "Building $PROJECT_NAME..."

    # Prepare build command
    local cmd="cargo"
    if [[ "$CROSS_COMPILE" == "true" && -n "$TARGET" ]]; then
        cmd="cross"
    fi

    local args=()
    args+=("build")

    if [[ "$BUILD_TYPE" == "release" ]]; then
        args+=("--release")
    fi

    if [[ -n "$TARGET" ]]; then
        args+=("--target" "$TARGET")
    fi

    if [[ -n "$FEATURES" ]]; then
        args+=("--features" "$FEATURES")
    fi

    if [[ "$VERBOSE" == "true" ]]; then
        args+=("--verbose")
    fi

    # Execute build
    log "Running: $cmd ${args[*]}"
    "$cmd" "${args[@]}"

    success "Build completed successfully!"
}

# Post-build processing
post_build() {
    local binary_name="$PROJECT_NAME"
    if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
        binary_name="${binary_name}.exe"
    fi

    local build_path
    if [[ -n "$TARGET" ]]; then
        build_path="$BUILD_DIR/$TARGET/$BUILD_TYPE/$binary_name"
    else
        build_path="$BUILD_DIR/$BUILD_TYPE/$binary_name"
    fi

    if [[ ! -f "$build_path" ]]; then
        error "Binary not found at expected location: $build_path"
    fi

    log "Binary location: $build_path"
    log "Binary size: $(du -h "$build_path" | cut -f1)"

    # Strip binary in release mode (Unix only)
    if [[ "$BUILD_TYPE" == "release" && "$OSTYPE" != "msys" && "$OSTYPE" != "cygwin" ]]; then
        log "Stripping debug symbols..."
        strip "$build_path" 2>/dev/null || warn "Failed to strip binary (this is normal on some platforms)"
    fi

    # Test binary
    log "Testing binary..."
    if "$build_path" --version &> /dev/null; then
        success "Binary test passed!"
    else
        error "Binary test failed!"
    fi
}

# Create release package
create_package() {
    if [[ "$BUILD_TYPE" != "release" ]]; then
        return
    fi

    log "Creating release package..."

    local pkg_name="$PROJECT_NAME"
    if [[ -n "$TARGET" ]]; then
        pkg_name="${pkg_name}-${TARGET}"
    fi

    local pkg_dir="$RELEASE_DIR/$pkg_name"
    mkdir -p "$pkg_dir"

    # Copy binary
    local binary_name="$PROJECT_NAME"
    if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
        binary_name="${binary_name}.exe"
    fi

    local build_path
    if [[ -n "$TARGET" ]]; then
        build_path="$BUILD_DIR/$TARGET/$BUILD_TYPE/$binary_name"
    else
        build_path="$BUILD_DIR/$BUILD_TYPE/$binary_name"
    fi

    cp "$build_path" "$pkg_dir/"

    # Copy documentation and configs
    cp README.md "$pkg_dir/" 2>/dev/null || true
    cp LICENSE* "$pkg_dir/" 2>/dev/null || true
    cp CHANGELOG.md "$pkg_dir/" 2>/dev/null || true

    # Create example config
    mkdir -p "$pkg_dir/examples"
    if [[ -d "examples" ]]; then
        cp -r examples/* "$pkg_dir/examples/" 2>/dev/null || true
    fi

    # Create tarball (Unix) or zip (Windows)
    local archive_name
    if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
        archive_name="${pkg_name}.zip"
        (cd "$RELEASE_DIR" && zip -r "$archive_name" "$pkg_name")
    else
        archive_name="${pkg_name}.tar.gz"
        (cd "$RELEASE_DIR" && tar -czf "$archive_name" "$pkg_name")
    fi

    success "Release package created: $RELEASE_DIR/$archive_name"
}

# Main execution
main() {
    log "Starting build process for $PROJECT_NAME"

    check_dependencies
    clean_build
    build_project
    post_build
    create_package

    success "Build process completed successfully!"

    # Display build information
    echo ""
    echo "Build Information:"
    echo "=================="
    echo "Project: $PROJECT_NAME"
    echo "Build Type: $BUILD_TYPE"
    echo "Target: ${TARGET:-default}"
    echo "Features: ${FEATURES:-none}"
    echo "Cross Compile: $CROSS_COMPILE"
    echo ""
}

# Execute main function
main "$@"