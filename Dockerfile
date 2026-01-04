# Multi-stage build for sparebank1-to-ynab

# Stage 1: Build the application
FROM rust:1.83-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY tests ./tests

# Build the release binary
RUN cargo build --release --bin sparebank1-to-ynab-sync

# Stage 2: Create minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN useradd -m -u 1000 appuser

# Set working directory
WORKDIR /app

# Create necessary directories
RUN mkdir -p /app/config /data && \
    chown -R appuser:appuser /app /data

# Copy the binary from builder
COPY --from=builder /app/target/release/sparebank1-to-ynab-sync /usr/local/bin/app

# Make binary executable
RUN chmod +x /usr/local/bin/app

# Switch to non-root user
USER appuser

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/app"]
