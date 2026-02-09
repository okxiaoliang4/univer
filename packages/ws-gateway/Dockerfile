FROM oven/bun:1 AS base
WORKDIR /app

# Install dependencies
COPY package.json bun.lock ./
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
