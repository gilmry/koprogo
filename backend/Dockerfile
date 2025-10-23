# Multi-stage build for optimal size and security

# Stage 1: Build
FROM rustlang/rust:nightly-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations
COPY tests ./tests
COPY benches ./benches
COPY .sqlx ./.sqlx

# Set SQLx offline mode for Docker builds
ENV SQLX_OFFLINE=true

# Build for release
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/koprogo-api /app/koprogo-api
COPY --from=builder /app/migrations /app/migrations

# Create non-root user
RUN useradd -m -u 1000 koprogo && \
    chown -R koprogo:koprogo /app

USER koprogo

EXPOSE 8080

CMD ["/app/koprogo-api"]
