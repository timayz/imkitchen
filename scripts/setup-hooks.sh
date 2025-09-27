#!/usr/bin/env bash
# Pre-commit hooks setup script for IMKitchen
# Sets up Git hooks for code quality enforcement

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${BLUE}[SETUP]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SETUP]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[SETUP]${NC} $1"
}

print_error() {
    echo -e "${RED}[SETUP]${NC} $1"
}

# Function to show usage
show_usage() {
    echo "IMKitchen Pre-commit Hooks Setup Script"
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --install    Install pre-commit hooks"
    echo "  --uninstall  Remove pre-commit hooks"
    echo "  --test       Test pre-commit hooks"
    echo "  --help       Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --install   # Install pre-commit hooks"
    echo "  $0 --test      # Test hooks without committing"
}

# Check if we're in a git repository
check_git_repo() {
    if ! git rev-parse --git-dir >/dev/null 2>&1; then
        print_error "Not in a git repository"
        exit 1
    fi
}

# Install pre-commit hooks
install_hooks() {
    print_info "Installing pre-commit hooks..."
    
    local hooks_dir=".git/hooks"
    local hook_file="$hooks_dir/pre-commit"
    
    # Check if hook already exists
    if [ -f "$hook_file" ]; then
        print_warning "Pre-commit hook already exists"
        read -p "Do you want to overwrite it? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Installation cancelled"
            return 0
        fi
    fi
    
    # Copy the pre-commit hook
    if [ -f "scripts/hooks/pre-commit" ]; then
        cp "scripts/hooks/pre-commit" "$hook_file"
    else
        # Use the hook from .git/hooks if it exists
        if [ ! -f "$hook_file" ]; then
            print_error "Pre-commit hook file not found"
            exit 1
        fi
    fi
    
    # Make it executable
    chmod +x "$hook_file"
    
    print_success "Pre-commit hook installed successfully"
    print_info "The hook will run automatically on git commit"
    print_info "To bypass the hook, use: git commit --no-verify"
}

# Uninstall pre-commit hooks
uninstall_hooks() {
    print_info "Uninstalling pre-commit hooks..."
    
    local hook_file=".git/hooks/pre-commit"
    
    if [ -f "$hook_file" ]; then
        rm "$hook_file"
        print_success "Pre-commit hook removed"
    else
        print_warning "Pre-commit hook not found"
    fi
}

# Test pre-commit hooks
test_hooks() {
    print_info "Testing pre-commit hooks..."
    
    local hook_file=".git/hooks/pre-commit"
    
    if [ ! -f "$hook_file" ]; then
        print_error "Pre-commit hook not installed"
        print_info "Run '$0 --install' first"
        exit 1
    fi
    
    if [ ! -x "$hook_file" ]; then
        print_error "Pre-commit hook is not executable"
        chmod +x "$hook_file"
        print_info "Fixed hook permissions"
    fi
    
    # Run the hook script directly
    print_info "Running pre-commit checks..."
    if bash "$hook_file"; then
        print_success "All pre-commit checks passed!"
    else
        print_error "Pre-commit checks failed"
        exit 1
    fi
}

# Install development tools needed for hooks
install_dev_tools() {
    print_info "Installing development tools for pre-commit hooks..."
    
    # Install cargo-audit if not present
    if ! command -v cargo-audit >/dev/null 2>&1; then
        print_info "Installing cargo-audit..."
        cargo install cargo-audit
    else
        print_success "cargo-audit already installed"
    fi
    
    # Install cargo-watch if not present
    if ! command -v cargo-watch >/dev/null 2>&1; then
        print_info "Installing cargo-watch..."
        cargo install cargo-watch
    else
        print_success "cargo-watch already installed"
    fi
    
    print_success "Development tools installed"
}

# Show current hook status
show_status() {
    print_info "Pre-commit hooks status:"
    
    local hook_file=".git/hooks/pre-commit"
    
    if [ -f "$hook_file" ]; then
        if [ -x "$hook_file" ]; then
            print_success "✓ Pre-commit hook installed and executable"
        else
            print_warning "⚠ Pre-commit hook installed but not executable"
        fi
    else
        print_warning "✗ Pre-commit hook not installed"
    fi
    
    # Check for required tools
    print_info "Required tools status:"
    
    if command -v cargo-audit >/dev/null 2>&1; then
        print_success "✓ cargo-audit installed"
    else
        print_warning "✗ cargo-audit not installed"
    fi
    
    if command -v cargo-watch >/dev/null 2>&1; then
        print_success "✓ cargo-watch installed"
    else
        print_warning "✗ cargo-watch not installed"
    fi
}

# Main execution
main() {
    local action="${1:-status}"
    
    check_git_repo
    
    case "$action" in
        "--install"|"install")
            install_dev_tools
            install_hooks
            ;;
        "--uninstall"|"uninstall")
            uninstall_hooks
            ;;
        "--test"|"test")
            test_hooks
            ;;
        "--status"|"status"|"")
            show_status
            ;;
        "--help"|"-h"|"help")
            show_usage
            ;;
        *)
            print_error "Unknown option: $action"
            echo ""
            show_usage
            exit 1
            ;;
    esac
}

# Execute main function with all arguments
main "$@"