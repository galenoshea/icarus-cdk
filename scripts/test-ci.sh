#!/bin/bash

# CI Simulation Script for Icarus SDK
# Runs the same checks as GitHub Actions CI locally

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print header
print_header() {
    echo -e "\n${BLUE}═══════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}\n"
}

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

# Main execution
print_header "CI Simulation - Running All Checks"

# Check Rust version
print_step "Checking Rust version..."
rustc --version
cargo --version
print_success "Rust toolchain ready"

# Format check
print_step "Checking code formatting..."
if cargo fmt -- --check; then
    print_success "Formatting check passed"
else
    print_error "Formatting check failed. Run 'cargo fmt' to fix."
    exit 1
fi

# Clippy check
print_step "Running clippy with warnings as errors..."
if cargo clippy -- -D warnings; then
    print_success "Clippy check passed"
else
    print_error "Clippy found issues"
    exit 1
fi

# Run tests
print_step "Running unit tests..."
if cargo test --lib --verbose; then
    print_success "Unit tests passed"
else
    print_error "Unit tests failed"
    exit 1
fi

print_step "Running integration tests..."
if cargo test --test '*' --verbose; then
    print_success "Integration tests passed"
else
    print_error "Integration tests failed"
    exit 1
fi

print_step "Running doc tests..."
if cargo test --doc --verbose; then
    print_success "Doc tests passed"
else
    print_error "Doc tests failed"
    exit 1
fi

# Build for WASM
print_step "Building for WASM target..."
if cargo build --target wasm32-unknown-unknown --release; then
    print_success "WASM build successful"
else
    print_error "WASM build failed"
    exit 1
fi

# Build regular target
print_step "Building debug target..."
if cargo build --all; then
    print_success "Debug build successful"
else
    print_error "Debug build failed"
    exit 1
fi

print_header "CI Simulation Complete"
print_success "All CI checks passed! Ready to push."
echo ""
echo "This simulates the GitHub Actions CI pipeline locally."
echo "If all checks pass here, they should pass in CI as well."
