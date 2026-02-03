# Multi-stage build for minimal Docker image

# Stage 1: Build
FROM rust:1.85-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Create app directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml .
COPY sender/Cargo.toml sender/

# Create dummy source to cache dependencies
RUN mkdir sender/src && \
    echo "fn main() {}" > sender/src/main.rs && \
    cargo build --release --bin mavlink-sender && \
    rm -rf sender/src

# Copy actual source code
COPY sender/src sender/src

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
