#!/bin/bash
# Development server with hot reload for imkitchen

set -e

export DATABASE_URL="${DATABASE_URL:-sqlite:imkitchen.db}"
export RUST_LOG="${RUST_LOG:-info}"

echo "Starting development server with hot reload..."
echo "Database URL: $DATABASE_URL"
echo "Log level: $RUST_LOG"
echo ""
echo "The server will automatically restart when you make changes to the code."
echo "Press Ctrl+C to stop."
echo ""

# Use cargo-watch to watch for changes and restart the server
/home/snapiz/.cargo/bin/cargo-watch -x "run --bin imkitchen"