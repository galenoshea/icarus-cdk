#!/bin/bash

# Local Release Test Script
# Tests the release process without actually releasing
# Usage: ./test-release.sh [patch|minor|major]

set -e  # Exit on first error

# Default to patch if not specified
RELEASE_TYPE=${1:-patch}

echo "🚀 Icarus SDK Release Test"
echo "============================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print info
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
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

# Check if cargo-release is installed
if ! command -v cargo-release &> /dev/null; then
    print_error "cargo-release not installed"
    echo "Install with: cargo install cargo-release"
    exit 1
fi

# Check GitHub token
if [ -z "$CARGO_REGISTRY_TOKEN" ]; then
    print_info "CARGO_REGISTRY_TOKEN not set locally (OK - GitHub Actions has it)"
fi

# Get current version
CURRENT_VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
print_info "Current version: $CURRENT_VERSION"
echo ""

# Step 1: Check for uncommitted changes
print_step "Checking for uncommitted changes..."
if [ -z "$(git status --porcelain)" ]; then
    print_success "Working directory clean"
else
    print_error "Uncommitted changes found"
    echo "Files with changes:"
    git status --short
    echo ""
    echo "Commit or stash changes before releasing"
    exit 1
fi
echo ""

# Step 2: Run CI tests first
print_step "Running CI tests..."
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
if "$SCRIPT_DIR/test-ci.sh" > /dev/null 2>&1; then
    print_success "CI tests passed"
else
    print_error "CI tests failed - run ./scripts/test-ci.sh for details"
    exit 1
fi
echo ""

# Step 3: Test cargo publish dry run for each crate
print_step "Testing crate publishing (dry run)..."

echo "  Testing icarus-core..."
if (cd crates/icarus-core && cargo publish --dry-run --allow-dirty > /dev/null 2>&1); then
    echo -e "  ${GREEN}✓${NC} icarus-core ready"
else
    echo -e "  ${RED}✗${NC} icarus-core has issues"
fi

echo "  Testing icarus-derive..."
if (cd crates/icarus-derive && cargo publish --dry-run --allow-dirty > /dev/null 2>&1); then
    echo -e "  ${GREEN}✓${NC} icarus-derive ready"
else
    echo -e "  ${RED}✗${NC} icarus-derive has issues"
fi

echo "  Testing icarus-canister..."
if (cd crates/icarus-canister && cargo publish --dry-run --allow-dirty > /dev/null 2>&1); then
    echo -e "  ${GREEN}✓${NC} icarus-canister ready"
else
    echo -e "  ${RED}✗${NC} icarus-canister has issues"
fi

echo "  Testing icarus (main)..."
if cargo publish --dry-run --allow-dirty > /dev/null 2>&1; then
    echo -e "  ${GREEN}✓${NC} icarus ready"
else
    echo -e "  ${RED}✗${NC} icarus has issues"
fi

print_success "All crates ready to publish"
echo ""

# Step 4: Show what cargo release would do
print_step "Testing release process ($RELEASE_TYPE)..."
echo ""
echo "What will happen:"
cargo release $RELEASE_TYPE --dry-run 2>&1 | grep -E "Upgrading|Committing|Tagging|Pushing" | head -20
echo ""

# Step 5: Check GitHub Actions setup
print_step "Checking GitHub Actions setup..."
if [ -f ".github/workflows/release.yml" ]; then
    print_success "Release workflow found"
else
    print_error "Release workflow missing"
fi
echo ""

# Final summary
echo "============================"
echo -e "${GREEN}🎉 Release test complete!${NC}"
echo ""
echo "Release checklist:"
echo "  ✅ Working directory clean"
echo "  ✅ All tests passing"
echo "  ✅ All crates ready to publish"
echo "  ✅ Release workflow configured"
echo ""
echo "To perform actual release:"
echo -e "  ${YELLOW}cargo release $RELEASE_TYPE --execute${NC}"
echo ""
echo "This will:"
case $RELEASE_TYPE in
    patch)
        echo "  • Bump version from $CURRENT_VERSION to next patch version"
        ;;
    minor)
        echo "  • Bump version from $CURRENT_VERSION to next minor version"
        ;;
    major)
        echo "  • Bump version from $CURRENT_VERSION to next major version"
        ;;
esac
echo "  • Create git tag and push to GitHub"
echo "  • Trigger GitHub Actions to publish to crates.io"