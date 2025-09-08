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

# Check version consistency before release
echo -e "${YELLOW}🔍 Checking version consistency...${NC}"
if ./scripts/check-versions.sh > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Version consistency check passed${NC}"
else
    echo -e "${YELLOW}⚠️  Version inconsistencies detected (will be fixed during release)${NC}"
fi

# Run all tests including E2E (releases must pass all tests)
echo -e "${YELLOW}🧪 Running unit and integration tests...${NC}"
if cargo test --all --lib --bins --tests --quiet; then
    echo -e "${GREEN}✅ Unit and integration tests passed${NC}"
else
    echo -e "${RED}❌ Tests failed${NC}"
    exit 1
fi

# Build CLI for E2E tests
echo -e "${YELLOW}🔨 Building CLI for E2E tests...${NC}"
if cargo build --package icarus-cli --bin icarus --release --quiet; then
    echo -e "${GREEN}✅ CLI built successfully${NC}"
else
    echo -e "${RED}❌ CLI build failed${NC}"
    exit 1
fi

# Run E2E tests (required for releases)
echo -e "${YELLOW}🧪 Running E2E tests (this may take a few minutes)...${NC}"
if (cd cli && cargo test --test '*' --release --quiet); then
    echo -e "${GREEN}✅ E2E tests passed${NC}"
else
    echo -e "${RED}❌ E2E tests failed - cannot release with failing E2E tests${NC}"
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