#!/bin/bash

# Icarus CLI Release Script
# Tags and releases a new version of the CLI

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)

echo -e "${BLUE}Icarus CLI Release Tool${NC}"
echo "========================================="
echo -e "Current version: ${YELLOW}v${CURRENT_VERSION}${NC}"
echo ""

# Check for uncommitted changes
if [[ -n $(git status -s) ]]; then
    echo -e "${RED}Error: You have uncommitted changes${NC}"
    echo "Please commit or stash your changes before releasing."
    exit 1
fi

# Check we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [[ "$CURRENT_BRANCH" != "main" ]]; then
    echo -e "${YELLOW}Warning: You're on branch '${CURRENT_BRANCH}', not 'main'${NC}"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Get new version
echo "Enter new version number (without 'v' prefix):"
read -p "New version: " NEW_VERSION

# Validate version format (basic semver check)
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9-]+)?$ ]]; then
    echo -e "${RED}Invalid version format. Please use semantic versioning (e.g., 1.0.0 or 1.0.0-beta.1)${NC}"
    exit 1
fi

# Check if tag already exists
if git rev-parse "v${NEW_VERSION}" >/dev/null 2>&1; then
    echo -e "${RED}Tag v${NEW_VERSION} already exists${NC}"
    exit 1
fi

echo ""
echo "Release Summary:"
echo "================"
echo -e "Current version: ${CURRENT_VERSION}"
echo -e "New version:     ${GREEN}${NEW_VERSION}${NC}"
echo ""

# Show recent commits that will be included
echo "Commits to be included in this release:"
echo "----------------------------------------"
git log --oneline -10
echo ""

read -p "Proceed with release? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Release cancelled."
    exit 1
fi

echo ""
echo -e "${YELLOW}Step 1: Updating version in Cargo.toml${NC}"
# Update version in Cargo.toml
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/" Cargo.toml
else
    # Linux
    sed -i "s/^version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/" Cargo.toml
fi

# Update Cargo.lock
echo -e "${YELLOW}Step 2: Updating Cargo.lock${NC}"
cargo update -p icarus-cli

# Commit version change
echo -e "${YELLOW}Step 3: Committing version change${NC}"
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to ${NEW_VERSION}"

# Create and push tag
echo -e "${YELLOW}Step 4: Creating tag v${NEW_VERSION}${NC}"
git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"

# Push changes
echo -e "${YELLOW}Step 5: Pushing to remote${NC}"
echo "This will trigger the CI/CD pipeline to build and upload binaries."
read -p "Push to remote? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}Changes committed locally but not pushed.${NC}"
    echo "To push manually, run:"
    echo "  git push origin main"
    echo "  git push origin v${NEW_VERSION}"
    exit 0
fi

git push origin main
git push origin "v${NEW_VERSION}"

echo ""
echo -e "${GREEN}✓ Release v${NEW_VERSION} initiated successfully!${NC}"
echo ""
echo "Next steps:"
echo "1. Monitor the GitHub Actions workflow at:"
echo "   https://github.com/icarus-platform/icarus-cli/actions"
echo ""
echo "2. Once the workflow completes, verify the release at:"
echo "   https://github.com/icarus-platform/icarus-cli/releases/tag/v${NEW_VERSION}"
echo ""
echo "3. Check that binaries are uploaded to ICP canister:"
echo "   dfx canister call backend getLatestCLIVersion"
echo ""
echo -e "${BLUE}Release complete! 🚀${NC}"