# Multi-stage build for ImKitchen Rust application

# Build stage
FROM rust:1.70 as builder

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Cache dependencies
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY src ./src
COPY templates ./templates
COPY static ./static
COPY migrations ./migrations
COPY askama.toml ./

# Build the actual application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 -s /bin/bash appuser

WORKDIR /app

# Copy built application
COPY --from=builder /app/target/release/imkitchen ./
COPY --from=builder /app/templates ./templates
COPY --from=builder /app/static ./static
COPY --from=builder /app/migrations ./migrations

# Change ownership to app user
RUN chown -R appuser:appuser /app

USER appuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["./imkitchen"]