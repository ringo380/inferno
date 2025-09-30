#!/bin/bash
# Build script for Inferno Desktop Application (Tauri v2)
# This script builds the desktop interface as a native macOS application

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}Inferno Desktop Build Script (v0.5.0)${NC}"
echo -e "${BLUE}=====================================${NC}"
echo ""

# Parse arguments
BUILD_MODE="dev"
VERBOSE=false
CLEAN=false
UNIVERSAL=false
SKIP_FRONTEND=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_MODE="release"
            shift
            ;;
        --dev)
            BUILD_MODE="dev"
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --universal)
            UNIVERSAL=true
            shift
            ;;
        --skip-frontend)
            SKIP_FRONTEND=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --release         Build in release mode (default: dev)"
            echo "  --dev             Build in development mode"
            echo "  --verbose         Show detailed build output"
            echo "  --clean           Clean build artifacts before building"
            echo "  --universal       Build universal binary (ARM64 + x86_64)"
            echo "  --skip-frontend   Skip frontend build (use existing dist/)"
            echo "  --help            Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 --release              # Build optimized release"
            echo "  $0 --dev --verbose        # Build debug with verbose output"
            echo "  $0 --release --universal  # Build universal binary"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Run '$0 --help' for usage information"
            exit 1
            ;;
    esac
done

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}Error: Desktop app currently only supports macOS${NC}"
    exit 1
fi

# Check for required tools
echo -e "${BLUE}Checking prerequisites...${NC}"

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found. Install Rust from https://rustup.rs/${NC}"
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo -e "${RED}Error: npm not found. Install Node.js from https://nodejs.org/${NC}"
    exit 1
fi

echo -e "${GREEN}✓ All prerequisites found${NC}"
echo ""

# Clean if requested
if [ "$CLEAN" = true ]; then
    echo -e "${YELLOW}Cleaning build artifacts...${NC}"
    cd "$PROJECT_ROOT"
    cargo clean
    cd dashboard
    rm -rf node_modules dist .next
    echo -e "${GREEN}✓ Clean complete${NC}"
    echo ""
fi

# Build frontend
if [ "$SKIP_FRONTEND" = false ]; then
    echo -e "${BLUE}Building frontend...${NC}"
    cd "$PROJECT_ROOT/dashboard"

    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        echo "Installing npm dependencies..."
        npm install
    fi

    # Build frontend
    echo "Building Next.js frontend..."
    npm run build

    echo -e "${GREEN}✓ Frontend build complete${NC}"
    echo ""
else
    echo -e "${YELLOW}Skipping frontend build (using existing)${NC}"
    echo ""
fi

# Build Tauri app
echo -e "${BLUE}Building Tauri desktop application...${NC}"
cd "$PROJECT_ROOT/dashboard"

TAURI_BUILD_ARGS=""

if [ "$BUILD_MODE" = "release" ]; then
    TAURI_BUILD_ARGS="-- --release"
    echo "Build mode: ${GREEN}Release (optimized)${NC}"
else
    echo "Build mode: ${YELLOW}Development (debug)${NC}"
fi

if [ "$UNIVERSAL" = true ]; then
    echo "Target: ${GREEN}Universal (ARM64 + x86_64)${NC}"
    # Tauri v2 handles universal builds automatically with proper config
    TAURI_BUILD_ARGS="$TAURI_BUILD_ARGS --target universal-apple-darwin"
else
    echo "Target: ${GREEN}Native ($(uname -m))${NC}"
fi

echo ""

# Run Tauri build
if [ "$VERBOSE" = true ]; then
    npm run tauri build $TAURI_BUILD_ARGS
else
    npm run tauri build $TAURI_BUILD_ARGS 2>&1 | grep -E "Finished|Built|Compiling|error" || true
fi

BUILD_EXIT_CODE=${PIPESTATUS[0]}

if [ $BUILD_EXIT_CODE -ne 0 ]; then
    echo ""
    echo -e "${RED}Build failed with exit code $BUILD_EXIT_CODE${NC}"
    exit $BUILD_EXIT_CODE
fi

echo ""
echo -e "${GREEN}=====================================${NC}"
echo -e "${GREEN}✓ Build Complete!${NC}"
echo -e "${GREEN}=====================================${NC}"
echo ""

# Show output locations
if [ "$BUILD_MODE" = "release" ]; then
    BUNDLE_DIR="$PROJECT_ROOT/dashboard/src-tauri/target/release/bundle"
else
    BUNDLE_DIR="$PROJECT_ROOT/dashboard/src-tauri/target/debug/bundle"
fi

echo -e "${BLUE}Build artifacts:${NC}"

if [ -d "$BUNDLE_DIR/dmg" ]; then
    echo -e "  DMG: ${GREEN}$BUNDLE_DIR/dmg/${NC}"
fi

if [ -d "$BUNDLE_DIR/macos" ]; then
    echo -e "  App: ${GREEN}$BUNDLE_DIR/macos/Inferno.app${NC}"
fi

echo ""
echo -e "${BLUE}To run the app:${NC}"
if [ "$BUILD_MODE" = "release" ]; then
    echo -e "  open $BUNDLE_DIR/macos/Inferno.app"
else
    echo -e "  cd dashboard && npm run tauri dev"
fi

echo ""
echo -e "${GREEN}Build process completed successfully!${NC}"