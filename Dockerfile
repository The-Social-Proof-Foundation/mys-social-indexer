FROM rust:1.75-slim-bookworm as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y \
    build-essential \
    libpq-dev \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 -U -s /bin/sh -d /app appuser

# Set up build directory and copy source
WORKDIR /app
COPY --chown=appuser:appuser . .

# Build the application
RUN cargo build --release

# Create runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy app user
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

# Create app directories
WORKDIR /app
RUN mkdir -p /app/data && chown -R appuser:appuser /app

# Copy binary and migrations
COPY --from=builder --chown=appuser:appuser /app/target/release/mys-social-indexer /app/
COPY --from=builder --chown=appuser:appuser /app/migrations /app/migrations

# Switch to app user
USER appuser

# Set default environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=postgres://postgres:postgres@postgres:5432/myso_social_indexer
ENV DATABASE_MAX_CONNECTIONS=10
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV CHECKPOINT_URL=https://checkpoints.testnet.mysocial.network
ENV START_CHECKPOINT=0
ENV INDEXER_CONCURRENCY=5

# Expose API port
EXPOSE 8080

# Run the application
CMD ["/app/mys-social-indexer"]