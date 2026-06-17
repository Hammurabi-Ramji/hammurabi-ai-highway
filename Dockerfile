# Multi-stage Dockerfile for Hammurabi AI Highway
# Build: docker build -t hammurabi-cli:latest .
# Run: docker run --rm -v $(pwd)/.env:/app/.env hammurabi-cli:latest

# Stage 1: Builder
FROM rust:latest as builder

WORKDIR /app

# Copy manifest first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY benches ./benches
COPY tests ./tests

# Build release binary
RUN cargo build --release --target x86_64-unknown-linux-gnu

# Strip binary for smaller size
RUN strip target/x86_64-unknown-linux-gnu/release/hammurabi

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/hammurabi /usr/local/bin/

# Copy documentation
COPY docs ./docs
COPY README.md .

# Create app user (principle of least privilege)
RUN useradd -m -u 1000 hammurabi && chown -R hammurabi:hammurabi /app

USER hammurabi

# Healthcheck
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD hammurabi --version || exit 1

# Default entrypoint
ENTRYPOINT ["hammurabi"]
CMD ["--help"]

# Metadata
LABEL maintainer="Hammurabi Coding Company"
LABEL description="Hammurabi AI Highway CLI — Natural language to gas-free on-chain execution"
LABEL version="1.2.0"