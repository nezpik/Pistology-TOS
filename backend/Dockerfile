# Build stage
FROM rust:1.83-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev sqlite-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml ./

# Create dummy main to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src
COPY schema.sql ./

# Build application
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache libgcc sqlite-libs

# Create app user
RUN addgroup -g 1000 rustapp && \
    adduser -D -u 1000 -G rustapp rustapp

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/pistology-backend /app/pistology-backend
COPY --from=builder /app/schema.sql /app/schema.sql

# Create data directory
RUN mkdir -p /app/data && \
    chown -R rustapp:rustapp /app

USER rustapp

EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3001/health || exit 1

CMD ["/app/pistology-backend"]
