# syntax=docker/dockerfile:1

# ============================================================================
# Guts Node Docker Image
# Multi-stage build for minimal production image
# ============================================================================

# Build stage
FROM rust:slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin guts-node --bin guts && \
    cp target/release/guts-node /guts-node && \
    cp target/release/guts /guts

# ============================================================================
# Runtime stage
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r guts && useradd -r -g guts guts

# Create directories
RUN mkdir -p /data /config && chown -R guts:guts /data /config

# Copy binaries
COPY --from=builder /guts-node /usr/local/bin/guts-node
COPY --from=builder /guts /usr/local/bin/guts

# Switch to non-root user
USER guts

# Set working directory
WORKDIR /data

# Expose ports
# 8080 - API
# 9000 - P2P
EXPOSE 8080 9000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
ENTRYPOINT ["guts-node"]
CMD ["--api-addr", "0.0.0.0:8080", "--p2p-addr", "0.0.0.0:9000", "--data-dir", "/data"]
