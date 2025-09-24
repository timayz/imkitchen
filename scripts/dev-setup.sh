#!/bin/bash
# Complete development environment setup script for imkitchen

set -e

echo "🚀 Setting up imkitchen development environment"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo is not installed. Please install Rust first."
    exit 1
fi

echo "✅ Rust/Cargo is installed"

# Check if sqlx-cli is installed
if ! command -v /home/snapiz/.cargo/bin/sqlx &> /dev/null; then
    echo "📦 Installing SQLx CLI..."
    cargo install sqlx-cli --no-default-features --features sqlite
else
    echo "✅ SQLx CLI is installed"
fi

# Check if cargo-watch is installed
if ! command -v /home/snapiz/.cargo/bin/cargo-watch &> /dev/null; then
    echo "📦 Installing cargo-watch..."
    cargo install cargo-watch
else
    echo "✅ cargo-watch is installed"
fi

# Make scripts executable
echo "🔧 Making scripts executable..."
chmod +x scripts/*.sh

# Initialize database
echo "💾 Initializing database..."
./scripts/init-db.sh

# Build the project
echo "🔨 Building project..."
cargo build

# Test the build
echo "🧪 Testing build..."
DATABASE_URL="sqlite:imkitchen.db" timeout 10 cargo run --bin imkitchen &
SERVER_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to start..."
sleep 3

# Health check
if ./scripts/health-check.sh; then
    echo "✅ Server health check passed"
else
    echo "❌ Server health check failed"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

# Stop server
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

echo ""
echo "🎉 Development environment setup complete!"
echo ""
echo "Next steps:"
echo "  • Run development server: ./scripts/dev-server.sh"
echo "  • Reset database: ./scripts/reset-db.sh"  
echo "  • Check health: ./scripts/health-check.sh"
echo ""