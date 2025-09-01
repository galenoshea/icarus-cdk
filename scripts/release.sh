#!/bin/bash

# Release script for Icarus SDK
# This ensures versions are properly synchronized before using cargo-release

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get the release type (patch, minor, major, or specific version)
RELEASE_TYPE=${1:-patch}

echo -e "${YELLOW}🚀 Preparing to release: $RELEASE_TYPE${NC}"

# Check if cargo-release is installed
if ! command -v cargo-release &> /dev/null; then
    echo -e "${RED}❌ cargo-release is not installed${NC}"
    echo "Install it with: cargo install cargo-release"
    exit 1
fi

# Check for clean working directory
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${RED}❌ Working directory is not clean${NC}"
    echo "Please commit or stash your changes first"
    exit 1
fi

# Run tests first
echo -e "${YELLOW}🧪 Running tests...${NC}"
if cargo test --all --quiet; then
    echo -e "${GREEN}✅ Tests passed${NC}"
else
    echo -e "${RED}❌ Tests failed${NC}"
    exit 1
fi

# Run clippy
echo -e "${YELLOW}🔍 Running clippy...${NC}"
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo -e "${GREEN}✅ Clippy passed${NC}"
else
    echo -e "${RED}❌ Clippy found issues${NC}"
    exit 1
fi

# Execute the release
echo -e "${YELLOW}📦 Starting release process...${NC}"
cargo release $RELEASE_TYPE --execute

echo -e "${GREEN}✅ Release complete!${NC}"
echo -e "The GitHub Actions workflow should now be running at:"
echo -e "https://github.com/galenoshea/icarus-sdk/actions"