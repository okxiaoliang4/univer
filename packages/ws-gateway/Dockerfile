FROM oven/bun:1 AS base
WORKDIR /app

# Build arguments for registry authentication
ARG GITHUB_TOKEN
ARG PRIVATE_REGISTRY_TOKEN

# Install dependencies
COPY package.json bun.lock ./

# Configure registries
RUN echo "@pagepeek:registry=https://npm.pkg.github.com/" >> .npmrc && \
    echo "//npm.pkg.github.com/:_authToken=${GITHUB_TOKEN}" >> .npmrc && \
    echo "@univerjs:registry=http://43.198.241.230:4873/" >> .npmrc && \
    echo "//43.198.241.230:4873/:_authToken=${PRIVATE_REGISTRY_TOKEN}" >> .npmrc

RUN bun install --production --frozen-lockfile || bun install --production

# Copy source
COPY src/ ./src/
COPY tsconfig.json ./

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run
CMD ["bun", "run", "src/index.ts"]
