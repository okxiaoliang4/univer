# Build stage
FROM rust:1.93-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制依赖文件
COPY packages/univer-ot-wasm/Cargo.toml ./
COPY packages/univer-ot-wasm/Cargo.lock ./
COPY packages/univer-ot-wasm/migration/Cargo.toml ./migration/
COPY packages/univer-ot-wasm/build.rs ./
COPY packages/univer-ot-wasm/proto ./proto

# 创建 dummy 文件缓存依赖
RUN mkdir -p src/bin src/server migration/src && \
    echo "fn main() {}" > src/bin/ot-server.rs && \
    echo "// dummy" > src/lib.rs && \
    echo "// dummy" > migration/src/lib.rs

# 构建依赖（允许失败）
RUN cargo build --release --features server --bin ot-server || true

# 删除 dummy 源码和构建产物
RUN rm -rf src migration/src && \
    rm -f target/release/ot-server target/release/deps/ot_server* && \
    rm -rf target/release/deps/libmigration* target/release/deps/libuniver_ot_wasm*

# 复制真实源码
COPY packages/univer-ot-wasm/src ./src
COPY packages/univer-ot-wasm/migration/src ./migration/src

# 重新构建（这次会使用缓存的依赖，但重新编译源码）
RUN cargo build --release --features server --bin ot-server

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