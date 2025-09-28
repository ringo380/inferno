# Multi-stage build for Inferno
# Supports both AMD64 and ARM64 architectures

# Builder stage
FROM --platform=$BUILDPLATFORM rust:1.75-bookworm as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /build

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY build.rs ./

# Build for the target platform
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
    "linux/amd64") \
        rustup target add x86_64-unknown-linux-gnu && \
        cargo build --release --target x86_64-unknown-linux-gnu && \
        mv target/x86_64-unknown-linux-gnu/release/inferno target/release/inferno \
        ;; \
    "linux/arm64") \
        rustup target add aarch64-unknown-linux-gnu && \
        apt-get update && apt-get install -y gcc-aarch64-linux-gnu && \
        cargo build --release --target aarch64-unknown-linux-gnu && \
        mv target/aarch64-unknown-linux-gnu/release/inferno target/release/inferno \
        ;; \
    *) \
        echo "Unsupported platform: $TARGETPLATFORM" && exit 1 \
        ;; \
    esac

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create inferno user
RUN useradd -m -s /bin/bash inferno

# Copy binary from builder
COPY --from=builder /build/target/release/inferno /usr/local/bin/inferno
RUN chmod +x /usr/local/bin/inferno

# Create directories for models and config
RUN mkdir -p /home/inferno/.inferno/models \
    /home/inferno/.inferno/cache \
    /home/inferno/.inferno/config \
    && chown -R inferno:inferno /home/inferno/.inferno

# Switch to inferno user
USER inferno
WORKDIR /home/inferno

# Set environment variables
ENV INFERNO_MODELS_DIR=/home/inferno/.inferno/models
ENV INFERNO_CACHE_DIR=/home/inferno/.inferno/cache
ENV INFERNO_CONFIG_DIR=/home/inferno/.inferno/config

# Expose default port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD inferno --version || exit 1

# Default command
ENTRYPOINT ["inferno"]
CMD ["serve", "--bind", "0.0.0.0:8080"]