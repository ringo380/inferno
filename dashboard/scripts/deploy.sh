#!/bin/bash

# Inferno Dashboard Deployment Script

set -e

# Configuration
IMAGE_NAME="inferno-dashboard"
IMAGE_TAG="${1:-latest}"
REGISTRY="${REGISTRY:-localhost:5000}"
COMPOSE_FILE="docker-compose.yml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
log() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check dependencies
check_dependencies() {
    log "Checking dependencies..."

    if ! command -v docker &> /dev/null; then
        error "Docker is not installed"
    fi

    if ! command -v docker-compose &> /dev/null; then
        error "Docker Compose is not installed"
    fi

    if ! command -v node &> /dev/null; then
        error "Node.js is not installed"
    fi

    log "All dependencies are available"
}

# Build the application
build_app() {
    log "Building Inferno Dashboard..."

    # Install dependencies
    log "Installing dependencies..."
    npm ci

    # Run linting
    log "Running linting..."
    npm run lint

    # Run tests (if available)
    if npm run | grep -q "test"; then
        log "Running tests..."
        npm run test
    fi

    # Build the application
    log "Building application..."
    npm run build

    log "Application build completed"
}

# Build Docker image
build_docker() {
    log "Building Docker image..."

    docker build \
        --tag "${IMAGE_NAME}:${IMAGE_TAG}" \
        --tag "${IMAGE_NAME}:latest" \
        --build-arg NODE_ENV=production \
        .

    log "Docker image built: ${IMAGE_NAME}:${IMAGE_TAG}"
}

# Push to registry
push_image() {
    if [ "$REGISTRY" != "localhost:5000" ] && [ -n "$REGISTRY" ]; then
        log "Pushing image to registry: $REGISTRY"

        docker tag "${IMAGE_NAME}:${IMAGE_TAG}" "${REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG}"
        docker push "${REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG}"

        log "Image pushed to registry"
    else
        log "Skipping registry push (local deployment)"
    fi
}

# Deploy with Docker Compose
deploy_compose() {
    log "Deploying with Docker Compose..."

    # Create .env file if it doesn't exist
    if [ ! -f .env ]; then
        log "Creating .env file from .env.example..."
        cp .env.example .env
        warn "Please update .env file with your configuration"
    fi

    # Stop existing containers
    docker-compose down --remove-orphans

    # Start services
    docker-compose up -d

    # Wait for services to be healthy
    log "Waiting for services to be healthy..."
    sleep 10

    # Check health
    if docker-compose ps | grep -q "Up"; then
        log "Deployment successful!"
        log "Dashboard available at: http://localhost:3000"
        log "Grafana available at: http://localhost:3001 (admin/admin)"
        log "Prometheus available at: http://localhost:9090"
    else
        error "Deployment failed - check service status"
    fi
}

# Show help
show_help() {
    echo "Inferno Dashboard Deployment Script"
    echo ""
    echo "Usage: $0 [OPTIONS] [IMAGE_TAG]"
    echo ""
    echo "Options:"
    echo "  --build-only     Build the application and Docker image only"
    echo "  --deploy-only    Deploy using existing Docker image"
    echo "  --push           Push image to registry after building"
    echo "  --help           Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  REGISTRY         Docker registry URL (default: localhost:5000)"
    echo ""
    echo "Examples:"
    echo "  $0                    # Build and deploy with 'latest' tag"
    echo "  $0 v1.0.0            # Build and deploy with 'v1.0.0' tag"
    echo "  $0 --build-only      # Build only, don't deploy"
    echo "  $0 --push v1.0.0     # Build, push to registry, and deploy"
}

# Main execution
main() {
    local build_only=false
    local deploy_only=false
    local push_image_flag=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --build-only)
                build_only=true
                shift
                ;;
            --deploy-only)
                deploy_only=true
                shift
                ;;
            --push)
                push_image_flag=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            -*)
                error "Unknown option: $1"
                ;;
            *)
                IMAGE_TAG="$1"
                shift
                ;;
        esac
    done

    log "Starting Inferno Dashboard deployment..."
    log "Image tag: ${IMAGE_TAG}"

    check_dependencies

    if [ "$deploy_only" = false ]; then
        build_app
        build_docker

        if [ "$push_image_flag" = true ]; then
            push_image
        fi
    fi

    if [ "$build_only" = false ]; then
        deploy_compose
    fi

    log "Deployment process completed!"
}

# Run main function
main "$@"