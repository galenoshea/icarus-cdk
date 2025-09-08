#!/bin/bash
set -e

# Pre-build test artifacts for faster CI execution
# This script builds shared components that E2E tests can reuse

echo "🚀 Pre-building test artifacts for CI..."

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${YELLOW}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Build the CLI in release mode
print_step "Building CLI..."
cargo build --package icarus-cli --bin icarus --release
print_success "CLI built"

# Build WASM targets
print_step "Building WASM targets..."
cargo build --target wasm32-unknown-unknown --release
print_success "WASM targets built"

# Create a shared test project for E2E tests
print_step "Creating shared test project..."
SHARED_PROJECT_DIR="/tmp/icarus-e2e-shared-project"
rm -rf "$SHARED_PROJECT_DIR"
mkdir -p "$SHARED_PROJECT_DIR"

# Run 'icarus new' to create the project
./target/release/icarus new shared-test-project --path "$SHARED_PROJECT_DIR"

# Build the shared project once
print_step "Building shared test project..."
cd "$SHARED_PROJECT_DIR/shared-test-project"
../../target/release/icarus build

print_success "Shared test project ready at: $SHARED_PROJECT_DIR"

# Calculate size savings
WASM_SIZE=$(du -h "$SHARED_PROJECT_DIR/shared-test-project/target/wasm32-unknown-unknown/release/"*.wasm | cut -f1)
print_success "WASM size: $WASM_SIZE"

echo -e "${GREEN}✨ Test artifacts pre-built successfully!${NC}"
echo "E2E tests can now reuse these artifacts for faster execution."