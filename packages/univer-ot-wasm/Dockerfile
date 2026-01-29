# Build stage
FROM rust:1.83-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all crate manifests
COPY crates/ot-core/Cargo.toml ./crates/ot-core/
COPY crates/ot-server/Cargo.toml ./crates/ot-server/
COPY crates/ot-server/build.rs ./crates/ot-server/
COPY migration/Cargo.toml ./migration/

# Copy proto files for gRPC
COPY crates/ot-server/proto ./crates/ot-server/proto

# Create dummy source files to cache dependencies
RUN mkdir -p crates/ot-core/src \
    crates/ot-server/src \
    migration/src && \
    echo "pub fn main() {}" > crates/ot-core/src/lib.rs && \
    echo "fn main() {}" > crates/ot-server/src/main.rs && \
    echo "pub fn main() {}" > migration/src/lib.rs

# Build dependencies (may fail due to dummy files, that's OK)
RUN cargo build -p ot-server --release || true

# Remove dummy source files and build artifacts
RUN rm -rf crates/ot-core/src \
    crates/ot-server/src \
    migration/src && \
    rm -f target/release/ot-server \
    target/release/deps/ot_server* \
    target/release/deps/libot_core* \
    target/release/deps/libmigration*

# Copy real source code
COPY crates/ot-core/src ./crates/ot-core/src
COPY crates/ot-server/src ./crates/ot-server/src
COPY migration/src ./migration/src

# Build the actual binary (will use cached dependencies)
RUN cargo build -p ot-server --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -g 1000 appuser && \
    useradd -u 1000 -g appuser -s /bin/bash -m appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/ot-server /app/ot-server

# Set permissions
RUN chmod +x /app/ot-server && \
    chown appuser:appuser /app/ot-server

USER appuser

EXPOSE 3000 50051

HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["/app/ot-server"]
