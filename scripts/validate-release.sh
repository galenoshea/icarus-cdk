#!/bin/bash

# Local validation of release readiness
# Mirrors the checks done in .github/workflows/release.yml validate-release job

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${YELLOW}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

echo -e "${YELLOW}🔍 Validating release readiness...${NC}"
echo

# Get version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
echo -e "Version to validate: ${GREEN}$VERSION${NC}"
echo

# Check 1: Version consistency
print_step "Checking version consistency..."
if ./scripts/check-versions.sh > /dev/null 2>&1; then
    print_success "Version consistency OK"
else
    print_error "Version inconsistencies detected!"
    echo "Run './scripts/check-versions.sh' for details"
    exit 1
fi

# Check 2: CHANGELOG entry
print_step "Checking CHANGELOG entry..."
if grep -q "## \[$VERSION\]" CHANGELOG.md; then
    print_success "CHANGELOG entry found for $VERSION"
else
    print_error "No CHANGELOG entry found for version $VERSION"
    echo "Add an entry like: ## [$VERSION] - $(date +%Y-%m-%d)"
    exit 1
fi

# Check 3: Workspace crate dependencies
print_step "Validating crate dependencies..."
failed=false
for crate in crates/*; do
    if [[ -f "$crate/Cargo.toml" ]]; then
        if ! grep -q 'version.workspace = true' "$crate/Cargo.toml"; then
            print_error "Crate $(basename $crate) doesn't use workspace version"
            failed=true
        fi
    fi
done

if [ "$failed" = false ]; then
    print_success "All crates use workspace versioning"
else
    exit 1
fi

# Check 4: Build warnings (bonus check not in CI but useful locally)
print_step "Checking for build warnings..."
if RUSTFLAGS="-D warnings" cargo build --package icarus-cli --bin icarus --release > /dev/null 2>&1; then
    print_success "No build warnings"
else
    print_error "Build has warnings that will fail in CI"
    echo "Run: RUSTFLAGS=\"-D warnings\" cargo build --package icarus-cli --bin icarus --release"
    exit 1
fi

echo
echo -e "${GREEN}🎉 All release validation checks passed!${NC}"
echo -e "Ready to tag and release version ${GREEN}$VERSION${NC}"
echo
echo "Next steps:"
echo "  1. Commit all changes"
echo "  2. Tag: git tag v$VERSION -m \"Release version $VERSION\""
echo "  3. Push: git push origin v$VERSION"