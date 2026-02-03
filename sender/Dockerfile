# Multi-stage build for minimal Docker image

# Stage 1: Build
FROM rust:1.85-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Create app directory
WORKDIR /app

# Copy all workspace manifests for dependency resolution
COPY Cargo.toml Cargo.lock ./
COPY sender/Cargo.toml sender/
COPY receiver/Cargo.toml receiver/
COPY common/Cargo.toml common/

# Create dummy sources for all workspace members to cache dependencies
RUN mkdir -p sender/src receiver/src common/src && \
    echo "fn main() {}" > sender/src/main.rs && \
    echo "fn main() {}" > receiver/src/main.rs && \
    echo "" > common/src/lib.rs && \
    cargo build --release --bin mavlink-sender && \
    rm -rf sender/src receiver/src common/src

# Copy actual source code for sender and its dependency (common)
COPY sender/src sender/src
COPY common/src common/src

# Build for release with optimizations
RUN cargo build --release --bin mavlink-sender && \
    strip /app/target/release/mavlink-sender

# Stage 2: Runtime
FROM scratch

# Copy binary from builder
COPY --from=builder /app/target/release/mavlink-sender /mavlink-sender

# Run as non-root user (UID 1000)
USER 1000:1000

ENTRYPOINT ["/mavlink-sender"]
CMD ["--listen", "0.0.0.0:14551"]
