#!/usr/bin/env bash

# Installation script for Inferno AI/ML model runner
set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
REPO_URL="https://github.com/inferno-ai/inferno"
BINARY_NAME="inferno"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
VERSION="${VERSION:-latest}"
FORCE_INSTALL=false

# Platform detection
detect_platform() {
    local os
    local arch

    case "$OSTYPE" in
        linux-gnu*) os="linux" ;;
        darwin*) os="macos" ;;
        msys*|cygwin*) os="windows" ;;
        *)
            echo -e "${RED}Unsupported OS: $OSTYPE${NC}"
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *)
            echo -e "${RED}Unsupported architecture: $(uname -m)${NC}"
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
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

usage() {
    echo "Inferno AI/ML Model Runner Installation Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -d, --install-dir DIR    Installation directory (default: /usr/local/bin)"
    echo "  -v, --version VERSION    Version to install (default: latest)"
    echo "  -f, --force              Force installation (overwrite existing)"
    echo "  -h, --help               Show this help"
    echo ""
    echo "Environment Variables:"
    echo "  INSTALL_DIR              Installation directory"
    echo "  VERSION                  Version to install"
    echo ""
    echo "Examples:"
    echo "  $0"
    echo "  $0 --install-dir ~/.local/bin"
    echo "  $0 --version v0.1.0"
    echo "  curl -fsSL https://install.inferno.ai | bash"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -f|--force)
            FORCE_INSTALL=true
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

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."

    # Check for required commands
    for cmd in curl tar; do
        if ! command -v $cmd &> /dev/null; then
            error "$cmd is required but not installed"
        fi
    done

    # Check if install directory exists and is writable
    if [[ ! -d "$INSTALL_DIR" ]]; then
        log "Creating installation directory: $INSTALL_DIR"
        if ! mkdir -p "$INSTALL_DIR" 2>/dev/null; then
            error "Cannot create installation directory: $INSTALL_DIR"
        fi
    fi

    if [[ ! -w "$INSTALL_DIR" ]]; then
        error "Installation directory is not writable: $INSTALL_DIR"
    fi
}

# Get download URL
get_download_url() {
    local platform="$1"
    local version="$2"

    if [[ "$version" == "latest" ]]; then
        log "Fetching latest release information..."
        local api_url="${REPO_URL/github.com/api.github.com/repos}/releases/latest"
        version=$(curl -s "$api_url" | grep '"tag_name"' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')

        if [[ -z "$version" ]]; then
            error "Failed to fetch latest version"
        fi

        log "Latest version: $version"
    fi

    local filename
    case "$platform" in
        linux-x86_64) filename="inferno-linux-x86_64" ;;
        linux-aarch64) filename="inferno-linux-aarch64" ;;
        macos-x86_64) filename="inferno-macos-x86_64" ;;
        macos-aarch64) filename="inferno-macos-aarch64" ;;
        windows-x86_64) filename="inferno-windows-x86_64.exe" ;;
        *) error "Unsupported platform: $platform" ;;
    esac

    echo "${REPO_URL}/releases/download/${version}/${filename}"
}

# Download and install
install_inferno() {
    local platform
    platform=$(detect_platform)

    log "Detected platform: $platform"

    local download_url
    download_url=$(get_download_url "$platform" "$VERSION")

    log "Download URL: $download_url"

    # Check if already installed
    local binary_path="$INSTALL_DIR/$BINARY_NAME"
    if [[ -f "$binary_path" ]] && [[ "$FORCE_INSTALL" != "true" ]]; then
        warn "Inferno is already installed at $binary_path"
        warn "Use --force to overwrite, or uninstall first"

        if "$binary_path" --version &>/dev/null; then
            local current_version
            current_version=$("$binary_path" --version | head -1)
            echo "Current version: $current_version"
        fi

        exit 1
    fi

    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    trap "rm -rf $temp_dir" EXIT

    log "Downloading Inferno..."
    local temp_file="$temp_dir/inferno_download"

    if ! curl -fL "$download_url" -o "$temp_file"; then
        error "Failed to download Inferno from $download_url"
    fi

    # Make executable and install
    chmod +x "$temp_file"

    log "Installing to $binary_path..."
    if ! mv "$temp_file" "$binary_path"; then
        error "Failed to install binary"
    fi

    success "Inferno installed successfully!"
}

# Verify installation
verify_installation() {
    local binary_path="$INSTALL_DIR/$BINARY_NAME"

    log "Verifying installation..."

    if [[ ! -f "$binary_path" ]]; then
        error "Binary not found at $binary_path"
    fi

    if ! "$binary_path" --version &>/dev/null; then
        error "Binary is not executable or corrupted"
    fi

    local version_output
    version_output=$("$binary_path" --version)
    success "Installation verified: $version_output"

    # Check if install directory is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "Installation directory $INSTALL_DIR is not in your PATH"
        warn "Add it to your PATH by running:"
        warn "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.bashrc"
        warn "  source ~/.bashrc"
    fi
}

# Create configuration
setup_configuration() {
    log "Setting up initial configuration..."

    local config_dir="$HOME/.config/inferno"
    local models_dir="$HOME/.local/share/inferno/models"

    mkdir -p "$config_dir"
    mkdir -p "$models_dir"

    if [[ ! -f "$config_dir/config.toml" ]]; then
        cat > "$config_dir/config.toml" << EOF
# Inferno AI/ML Model Runner Configuration

models_dir = "$models_dir"
cache_dir = "$HOME/.cache/inferno"
log_level = "info"
log_format = "pretty"

[backend_config]
gpu_enabled = false
context_size = 2048
batch_size = 32
memory_map = true

[server]
bind_address = "127.0.0.1"
port = 8080
max_concurrent_requests = 10
request_timeout_seconds = 300

[security]
verify_checksums = true
allowed_model_extensions = ["gguf", "onnx"]
max_model_size_gb = 50.0
sandbox_enabled = true
EOF

        success "Configuration created at $config_dir/config.toml"
    fi

    log "Models directory: $models_dir"
    log "Configuration directory: $config_dir"
}

# Show getting started information
show_getting_started() {
    echo ""
    echo -e "${GREEN}🎉 Inferno is now installed!${NC}"
    echo ""
    echo "Getting Started:"
    echo "================"
    echo ""
    echo "1. Check installation:"
    echo "   inferno --version"
    echo ""
    echo "2. View help:"
    echo "   inferno --help"
    echo ""
    echo "3. List models (initially empty):"
    echo "   inferno models list"
    echo ""
    echo "4. Launch TUI:"
    echo "   inferno tui"
    echo ""
    echo "5. Download a model (place .gguf or .onnx files in ~/.local/share/inferno/models/)"
    echo ""
    echo "6. Run inference:"
    echo "   inferno run --model MODEL_NAME --prompt \"Hello, world!\""
    echo ""
    echo "For more information, visit: $REPO_URL"
    echo ""
}

# Main installation process
main() {
    log "Installing Inferno AI/ML Model Runner"

    check_prerequisites
    install_inferno
    verify_installation
    setup_configuration
    show_getting_started
}

# Run main function
main "$@"