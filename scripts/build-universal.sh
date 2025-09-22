#!/bin/bash

# Universal Binary Build Script for macOS
# Builds Inferno for both Apple Silicon and Intel architectures

set -e

# Color output for better readability
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔥 Building Inferno Universal Binary for macOS${NC}"
echo ""

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}❌ This script is designed for macOS only${NC}"
    exit 1
fi

# Set build mode (default to release)
BUILD_MODE=${1:-release}
FEATURES=${2:-"tauri-app"}

if [ "$BUILD_MODE" = "debug" ]; then
    BUILD_FLAGS=""
    TARGET_DIR="target"
else
    BUILD_FLAGS="--release"
    TARGET_DIR="target"
fi

echo -e "${YELLOW}📋 Build Configuration:${NC}"
echo "  Mode: $BUILD_MODE"
echo "  Features: $FEATURES"
echo ""

# Create output directory
OUTPUT_DIR="target/universal"
mkdir -p "$OUTPUT_DIR"

# Target architectures
APPLE_SILICON="aarch64-apple-darwin"
INTEL="x86_64-apple-darwin"

# Install targets if not present
echo -e "${BLUE}🎯 Ensuring build targets are installed...${NC}"
rustup target add "$APPLE_SILICON" || true
rustup target add "$INTEL" || true

# Build function for each target
build_target() {
    local target=$1
    local arch_name=$2

    echo -e "${YELLOW}🔨 Building for $arch_name ($target)...${NC}"

    # Build the main CLI binary
    cargo build $BUILD_FLAGS \
        --target "$target" \
        --bin inferno

    # Build the Tauri app if features include tauri-app
    if [[ "$FEATURES" == *"tauri-app"* ]]; then
        echo -e "${YELLOW}📱 Building Tauri app for $arch_name...${NC}"
        cargo build $BUILD_FLAGS \
            --target "$target" \
            --bin inferno_app \
            --features "$FEATURES"
    fi

    echo -e "${GREEN}✅ $arch_name build complete${NC}"
}

# Build for Apple Silicon
build_target "$APPLE_SILICON" "Apple Silicon"

# Build for Intel
build_target "$INTEL" "Intel"

# Create universal binaries using lipo
echo -e "${BLUE}🔗 Creating universal binaries...${NC}"

# Universal CLI binary
if [ "$BUILD_MODE" = "debug" ]; then
    ARM_BIN="target/$APPLE_SILICON/debug/inferno"
    INTEL_BIN="target/$INTEL/debug/inferno"
    UNIVERSAL_BIN="$OUTPUT_DIR/inferno"
else
    ARM_BIN="target/$APPLE_SILICON/release/inferno"
    INTEL_BIN="target/$INTEL/release/inferno"
    UNIVERSAL_BIN="$OUTPUT_DIR/inferno"
fi

lipo -create "$ARM_BIN" "$INTEL_BIN" -output "$UNIVERSAL_BIN"
echo -e "${GREEN}✅ Universal CLI binary created: $UNIVERSAL_BIN${NC}"

# Universal Tauri app binary (if built)
if [[ "$FEATURES" == *"tauri-app"* ]]; then
    if [ "$BUILD_MODE" = "debug" ]; then
        ARM_APP="target/$APPLE_SILICON/debug/inferno_app"
        INTEL_APP="target/$INTEL/debug/inferno_app"
        UNIVERSAL_APP="$OUTPUT_DIR/inferno_app"
    else
        ARM_APP="target/$APPLE_SILICON/release/inferno_app"
        INTEL_APP="target/$INTEL/release/inferno_app"
        UNIVERSAL_APP="$OUTPUT_DIR/inferno_app"
    fi

    lipo -create "$ARM_APP" "$INTEL_APP" -output "$UNIVERSAL_APP"
    echo -e "${GREEN}✅ Universal Tauri app binary created: $UNIVERSAL_APP${NC}"
fi

# Verify the universal binaries
echo -e "${BLUE}🔍 Verifying universal binaries...${NC}"
echo ""

verify_binary() {
    local binary=$1
    local name=$2

    if [ -f "$binary" ]; then
        echo -e "${YELLOW}📊 $name architecture support:${NC}"
        lipo -archs "$binary"
        echo ""

        echo -e "${YELLOW}📋 $name detailed info:${NC}"
        lipo -detailed_info "$binary"
        echo ""

        echo -e "${YELLOW}📏 $name file size:${NC}"
        ls -lh "$binary" | awk '{print $5 " " $9}'
        echo ""
    fi
}

verify_binary "$UNIVERSAL_BIN" "CLI Binary"

if [[ "$FEATURES" == *"tauri-app"* ]] && [ -f "$UNIVERSAL_APP" ]; then
    verify_binary "$UNIVERSAL_APP" "Tauri App"
fi

# Create Tauri bundle if requested
if [[ "$FEATURES" == *"tauri-app"* ]]; then
    echo -e "${BLUE}📦 Creating Tauri application bundle...${NC}"

    # Change to dashboard directory for Tauri build
    cd dashboard

    # Install Node dependencies if needed
    if [ ! -d "node_modules" ]; then
        echo -e "${YELLOW}📦 Installing Node.js dependencies...${NC}"
        npm install
    fi

    # Build the frontend
    echo -e "${YELLOW}🏗️ Building frontend...${NC}"
    npm run build

    # Create Tauri bundle with universal binary
    echo -e "${YELLOW}📱 Creating Tauri bundle...${NC}"

    # Copy universal binary to expected location
    mkdir -p "src-tauri/target/universal/release"
    cp "../$UNIVERSAL_APP" "src-tauri/target/universal/release/inferno_app"

    # Build Tauri bundle
    npm run tauri build -- --target universal-apple-darwin

    cd ..

    echo -e "${GREEN}✅ Tauri application bundle created${NC}"
fi

echo ""
echo -e "${GREEN}🎉 Universal binary build complete!${NC}"
echo ""
echo -e "${YELLOW}📁 Output files:${NC}"
echo "  CLI Binary: $UNIVERSAL_BIN"
if [[ "$FEATURES" == *"tauri-app"* ]]; then
    echo "  Tauri App: $UNIVERSAL_APP"
    if [ -d "dashboard/src-tauri/target/universal-apple-darwin/release/bundle" ]; then
        echo "  App Bundle: dashboard/src-tauri/target/universal-apple-darwin/release/bundle/"
    fi
fi
echo ""

# Performance test
echo -e "${BLUE}🚀 Quick performance test...${NC}"
echo ""
echo "CLI Version:"
"$UNIVERSAL_BIN" --version
echo ""

if [[ "$FEATURES" == *"tauri-app"* ]] && [ -f "$UNIVERSAL_APP" ]; then
    echo "App Version:"
    "$UNIVERSAL_APP" --version || echo "Tauri app version check skipped (requires GUI context)"
    echo ""
fi

echo -e "${GREEN}✨ Build process completed successfully!${NC}"