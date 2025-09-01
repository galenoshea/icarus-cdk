#!/bin/bash

# Local CI Test Script
# Runs the same checks as GitHub Actions CI locally
# Usage: ./test-ci.sh

set -e  # Exit on first error

echo "🔍 Icarus SDK Local CI Test"
echo "============================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print step
print_step() {
    echo -e "${YELLOW}▶ $1${NC}"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Function to print error
print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Error: Not in icarus-sdk root directory"
    exit 1
fi

# Step 1: Run tests
print_step "Running tests..."
if cargo test --verbose; then
    print_success "Tests passed"
else
    print_error "Tests failed"
    exit 1
fi
echo ""

# Step 2: Run clippy with strict warnings
print_step "Running clippy (strict mode)..."
if cargo clippy -- -D warnings; then
    print_success "Clippy passed"
else
    print_error "Clippy failed - fix warnings before pushing"
    exit 1
fi
echo ""

# Step 3: Check formatting
print_step "Checking formatting..."
if cargo fmt -- --check; then
    print_success "Formatting correct"
else
    print_error "Formatting issues found - run 'cargo fmt' to fix"
    exit 1
fi
echo ""

# Step 4: Build WASM target
print_step "Building WASM target..."
if cargo build --target wasm32-unknown-unknown --release; then
    print_success "WASM build successful"
else
    print_error "WASM build failed"
    exit 1
fi
echo ""

# Step 5: Build documentation with strict warnings
print_step "Building documentation (strict mode)..."
if RUSTDOCFLAGS="-D warnings" cargo doc --no-deps; then
    print_success "Documentation build successful"
else
    print_error "Documentation build failed - fix doc warnings"
    exit 1
fi
echo ""

# Step 6: Run doctests
print_step "Running doctests..."
if cargo test --doc; then
    print_success "Doctests passed"
else
    print_error "Doctests failed"
    exit 1
fi
echo ""

# Final summary
echo "============================"
echo -e "${GREEN}🎉 All CI checks passed!${NC}"
echo "Safe to push to GitHub"
echo ""
echo "Next steps:"
echo "  1. git push"
echo "  2. Check GitHub Actions: https://github.com/galenoshea/icarus-sdk/actions"
echo "  3. Once CI passes, release: cargo release patch --execute"