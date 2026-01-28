# Build stage
FROM rust:1.92-alpine as builder

# Install build dependencies
RUN apk add --no-cache \
    pkgconfig \
    openssl-dev \
    musl-dev \
    ca-certificates \
    curl \
    protobuf-dev \
    build-base

# Set working directory
WORKDIR /app

# Configure build environment
# Use dynamic linking for OpenSSL (musl target defaults to static linking)
ENV OPENSSL_STATIC=0
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV RUSTFLAGS="-C target-feature=-crt-static"

# Copy workspace Cargo files first (for dependency caching)
COPY packages/univer-ot-wasm/Cargo.toml ./
COPY packages/univer-ot-wasm/Cargo.lock ./
COPY packages/univer-ot-wasm/migration/Cargo.toml ./migration/

# Copy build script and proto files (needed for build.rs)
COPY packages/univer-ot-wasm/build.rs ./
COPY packages/univer-ot-wasm/proto ./proto

# Create dummy source files to cache dependencies
# This allows Docker to cache the dependency layer separately from source code
RUN mkdir -p src/bin src/server migration/src && \
    echo "fn main() {}" > src/bin/ot-server.rs && \
    echo "// dummy" > src/lib.rs && \
    echo "// dummy" > migration/src/lib.rs

# Fetch and build dependencies only (this layer will be cached unless Cargo.toml changes)
# First, fetch dependencies to ensure they are downloaded
RUN cargo fetch --features server || true

# Try to build with dummy code - this will compile dependencies even if the binary fails
# The dependencies will be cached in target/ directory
RUN cargo build --release --features server --bin ot-server || \
    (echo "Dummy build failed (expected), but dependencies should be cached" && \
     cargo build --release --features server --lib || true)

# Clean up dummy source files but keep the dependency cache in target/
RUN rm -rf src migration/src

# Copy actual source code
COPY packages/univer-ot-wasm/src ./src
COPY packages/univer-ot-wasm/migration/src ./migration/src

# Build the actual server binary
# This will reuse the cached dependencies layer
RUN cargo build --release --features server --bin ot-server

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libssl3 \
    curl

# Create app user
RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/ot-server /app/ot-server

# Ensure binary is executable
RUN chmod +x /app/ot-server

# Change ownership
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Expose ports
# 3000: HTTP/Socket.IO server
# 50051: gRPC server
EXPOSE 3000 50051

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the server
CMD ["/app/ot-server"]
