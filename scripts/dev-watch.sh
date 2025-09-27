#!/usr/bin/env bash
# Development watch script for IMKitchen
# Provides convenient shortcuts for different watch modes

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
show_usage() {
    echo "IMKitchen Development Watch Script"
    echo "Usage: $0 [MODE]"
    echo ""
    echo "Available modes:"
    echo "  check    - Fast syntax and type checking (cargo check)"
    echo "  test     - Run tests on file changes (cargo test)"
    echo "  lint     - Run clippy linting on changes"
    echo "  fmt      - Auto-format code on changes"
    echo "  build    - Build project on changes"
    echo "  full     - Complete build, test, and quality checks"
    echo "  web      - Watch and restart web server"
    echo "  migrate  - Watch for migration changes and apply them"
    echo ""
    echo "Examples:"
    echo "  $0 test     # Watch for changes and run tests"
    echo "  $0 check    # Fast feedback during development"
    echo "  $0 full     # Comprehensive development workflow"
    echo "  $0 web      # Development server with auto-restart"
}

# Check if cargo-watch is installed
check_cargo_watch() {
    if ! command -v cargo-watch &> /dev/null; then
        print_error "cargo-watch is not installed"
        print_info "Install it with: cargo install cargo-watch"
        exit 1
    fi
}

# Main execution
main() {
    local mode="${1:-help}"
    
    check_cargo_watch
    
    case "$mode" in
        "check")
            print_info "Starting fast check mode..."
            cargo watch -x check
            ;;
        "test")
            print_info "Starting test watch mode..."
            cargo watch -x test
            ;;
        "lint")
            print_info "Starting lint watch mode..."
            cargo watch -x "clippy -- -D warnings"
            ;;
        "fmt")
            print_info "Starting format watch mode..."
            cargo watch -x fmt
            ;;
        "build")
            print_info "Starting build watch mode..."
            cargo watch -x build
            ;;
        "full")
            print_info "Starting comprehensive watch mode..."
            cargo watch -x build -x test -x "clippy -- -D warnings"
            ;;
        "web")
            print_info "Starting web server development mode..."
            print_warning "This will restart the server on every change"
            cargo watch -x "run -- web start --host 127.0.0.1 --port 3000"
            ;;
        "migrate")
            print_info "Starting migration watch mode..."
            cargo watch -w migrations -x "run -- migrate up"
            ;;
        "help"|"-h"|"--help")
            show_usage
            ;;
        *)
            print_error "Unknown mode: $mode"
            echo ""
            show_usage
            exit 1
            ;;
    esac
}

# Execute main function with all arguments
main "$@"