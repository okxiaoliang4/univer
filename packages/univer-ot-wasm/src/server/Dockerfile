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

# Copy workspace Cargo files
COPY packages/univer-ot-wasm/Cargo.toml packages/univer-ot-wasm/Cargo.lock ./
COPY packages/univer-ot-wasm/migration/Cargo.toml ./migration/

# Copy proto files and build script (needed for build.rs)
COPY packages/univer-ot-wasm/proto ./proto
COPY packages/univer-ot-wasm/build.rs ./

# Copy source code
COPY packages/univer-ot-wasm/src ./src
COPY packages/univer-ot-wasm/migration/src ./migration/src

# Build the server binary
# Use dynamic linking for OpenSSL (musl target defaults to static linking)
ENV OPENSSL_STATIC=0
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo check --features server
RUN cargo build --release --features server --bin ot-server

# Runtime stage
FROM alpine:latest

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
