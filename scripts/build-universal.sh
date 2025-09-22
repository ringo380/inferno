#!/bin/bash

# Universal macOS binary build script for Inferno
# Builds for both Apple Silicon (aarch64) and Intel (x86_64) architectures

set -e

echo "🔥 Building Universal macOS Binary for Inferno"
echo "=============================================="

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ This script is designed for macOS only"
    exit 1
fi

# Install required targets if not present
echo "📦 Installing Rust targets..."
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Create output directory
mkdir -p target/universal-apple-darwin/release

# Build for Apple Silicon (M1/M2)
echo "🏗️  Building for Apple Silicon (aarch64)..."
cargo build --release --target aarch64-apple-darwin

# Build for Intel (x86_64)
echo "🏗️  Building for Intel (x86_64)..."
cargo build --release --target x86_64-apple-darwin

# Create universal binary using lipo
echo "🔗 Creating universal binary..."
lipo -create \
    target/aarch64-apple-darwin/release/inferno \
    target/x86_64-apple-darwin/release/inferno \
    -output target/universal-apple-darwin/release/inferno

# Verify the universal binary
echo "✅ Verifying universal binary..."
file target/universal-apple-darwin/release/inferno
lipo -info target/universal-apple-darwin/release/inferno

# Make executable
chmod +x target/universal-apple-darwin/release/inferno

echo ""
echo "🎉 Universal binary created successfully!"
echo "📍 Location: target/universal-apple-darwin/release/inferno"
echo ""
echo "To test the binary:"
echo "  ./target/universal-apple-darwin/release/inferno --version"
echo ""