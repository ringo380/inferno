# Multi-stage build for Inferno v0.8.0+
# Production-grade Docker image with health checks, proper resource management
# Supports both AMD64 (x86_64) and ARM64 (aarch64) architectures

# ============================================================================
# BUILDER STAGE - Compilation
# ============================================================================
FROM --platform=$BUILDPLATFORM rust:1.75-bookworm as builder

LABEL stage=builder

# Install build dependencies (will be removed from final image)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

WORKDIR /build

# Copy Cargo manifest files first (better layer caching)
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY build.rs ./

# Build for the target platform with optimizations
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
    "linux/amd64") \
        echo "Building for x86_64..." && \
        rustup target add x86_64-unknown-linux-gnu && \
        cargo build --release --target x86_64-unknown-linux-gnu && \
        mv target/x86_64-unknown-linux-gnu/release/inferno target/release/inferno \
        ;; \
    "linux/arm64") \
        echo "Building for aarch64..." && \
        rustup target add aarch64-unknown-linux-gnu && \
        apt-get update && apt-get install -y --no-install-recommends gcc-aarch64-linux-gnu && \
        cargo build --release --target aarch64-unknown-linux-gnu && \
        mv target/aarch64-unknown-linux-gnu/release/inferno target/release/inferno \
        ;; \
    *) \
        echo "ERROR: Unsupported platform: $TARGETPLATFORM" && exit 1 \
        ;; \
    esac

# Verify binary was created
RUN test -f /build/target/release/inferno || (echo "Build failed: binary not found" && exit 1)

# ============================================================================
# RUNTIME STAGE - Minimal production image
# ============================================================================
FROM debian:bookworm-slim

LABEL maintainer="Inferno Developers <dev@inferno.ai>"
LABEL version="0.8.0"
LABEL description="Enterprise-grade AI/ML model inference engine with real-time streaming"

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Create non-root user for security
RUN useradd -m -s /bin/bash -u 1000 inferno

# Create required directories with proper permissions
RUN mkdir -p \
    /home/inferno/.inferno/models \
    /home/inferno/.inferno/cache \
    /home/inferno/.inferno/config \
    /home/inferno/.inferno/queue \
    /home/inferno/.inferno/logs \
    && chown -R inferno:inferno /home/inferno/.inferno \
    && chmod -R 755 /home/inferno/.inferno

# Copy compiled binary from builder stage
COPY --from=builder --chown=inferno:inferno /build/target/release/inferno /usr/local/bin/inferno
RUN chmod +x /usr/local/bin/inferno

# Switch to non-root user
USER inferno
WORKDIR /home/inferno

# Set environment variables
ENV INFERNO_MODELS_DIR=/home/inferno/.inferno/models
ENV INFERNO_CACHE_DIR=/home/inferno/.inferno/cache
ENV INFERNO_CONFIG_DIR=/home/inferno/.inferno/config
ENV INFERNO_LOG_LEVEL=info

# Declare volumes for persistence
VOLUME ["/home/inferno/.inferno/models", "/home/inferno/.inferno/cache", "/home/inferno/.inferno/queue"]

# Expose API port
EXPOSE 8000

# Health check - actually tests HTTP endpoint (startup probe)
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Default command: start API server
ENTRYPOINT ["inferno"]
CMD ["serve", "--bind", "0.0.0.0:8000"]