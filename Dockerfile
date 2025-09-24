# Multi-stage build for Rust application  
FROM rust:1.90-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/imkitchen-core/Cargo.toml ./crates/imkitchen-core/
COPY crates/imkitchen-shared/Cargo.toml ./crates/imkitchen-shared/
COPY crates/imkitchen-web/Cargo.toml ./crates/imkitchen-web/

# Create dummy source files for dependency building
RUN mkdir -p crates/imkitchen-core/src \
    crates/imkitchen-shared/src \
    crates/imkitchen-web/src && \
    echo "fn main() {}" > crates/imkitchen-web/src/main.rs && \
    echo "// dummy" > crates/imkitchen-core/src/lib.rs && \
    echo "// dummy" > crates/imkitchen-shared/src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release
RUN rm -rf crates/*/src

# Copy actual source code
COPY . .

# Touch files to ensure rebuild
RUN find crates/ -name "*.rs" -exec touch {} \;

# Build the application with static linking
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --release --target x86_64-unknown-linux-gnu

# Runtime stage - Alpine for small size
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    sqlite \
    libgcc \
    && addgroup -g 1000 app \
    && adduser -D -s /bin/sh -u 1000 -G app app

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/imkitchen ./imkitchen

# Make binary executable and verify
RUN chmod +x ./imkitchen && ls -la ./imkitchen

# Debug: Check binary details and dependencies
RUN file ./imkitchen && ldd ./imkitchen || echo "ldd failed"

# Copy configuration and migration files
COPY --chown=app:app config/ ./config/
COPY --chown=app:app migrations/ ./migrations/

# Create directory for database with proper permissions
RUN mkdir -p /data && chown -R app:app /data

# Switch to non-root user
USER app

# Set environment variables
ENV APP_ENVIRONMENT=production
ENV IMKITCHEN_DATABASE_URL=sqlite:/data/imkitchen.db
ENV RUST_LOG=info

# Expose port
EXPOSE 3000

# Health check using the built-in health endpoint
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

# Run the application
CMD ["./imkitchen"]