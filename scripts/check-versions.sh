#!/bin/bash

# Version consistency checker for Icarus SDK
# Ensures all version references are aligned across the project

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🔍 Checking version consistency across the project...${NC}"

# Get the main version from workspace Cargo.toml
MAIN_VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
echo -e "Main version from Cargo.toml: ${GREEN}$MAIN_VERSION${NC}"

# Track if any mismatches are found
ERRORS=0

# Function to check version in a file
# Now with context awareness - skips historical references
check_version() {
    local file=$1
    local pattern=$2
    local description=$3
    local skip_historical=${4:-false}
    
    if [ -f "$file" ]; then
        # For migration guide, skip historical version references
        if [[ "$file" == *"migration-guide.md"* ]] && [ "$skip_historical" = "true" ]; then
            # Only check lines that indicate current version
            local lines_to_check=$(grep -n "$pattern" "$file" | grep -E "(Current Version|current version|Latest|latest)" | cut -d: -f1)
            if [ -n "$lines_to_check" ]; then
                for line_num in $lines_to_check; do
                    local line_content=$(sed -n "${line_num}p" "$file")
                    local found_version=$(echo "$line_content" | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' | head -1)
                    if [ -n "$found_version" ] && [ "$found_version" != "$MAIN_VERSION" ]; then
                        echo -e "${RED}❌ Version mismatch in $file line $line_num: found $found_version (expected $MAIN_VERSION) - $description${NC}"
                        ERRORS=$((ERRORS + 1))
                    fi
                done
            fi
        else
            # Standard version checking for non-migration files
            if grep -q "$pattern" "$file"; then
                local found_versions=$(grep -o "$pattern" "$file" | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' | sort -u)
                for version in $found_versions; do
                    if [ "$version" != "$MAIN_VERSION" ]; then
                        echo -e "${RED}❌ Version mismatch in $file: found $version (expected $MAIN_VERSION) - $description${NC}"
                        ERRORS=$((ERRORS + 1))
                    fi
                done
            fi
        fi
    fi
}

# Check Cargo.toml files - only check package version and icarus dependencies
echo -e "\n${YELLOW}Checking Cargo.toml files...${NC}"

# Check workspace root version
echo -e "${GREEN}✅ Workspace root version: $MAIN_VERSION${NC}"

# Check icarus dependencies in workspace
for dep in "icarus-core" "icarus-derive" "icarus-canister"; do
    if grep -q "$dep = { version = " Cargo.toml; then
        version=$(grep "$dep = { version = " Cargo.toml | grep -o '"[0-9]\+\.[0-9]\+\.[0-9]\+"' | tr -d '"')
        if [ "$version" != "$MAIN_VERSION" ]; then
            echo -e "${RED}❌ Version mismatch for $dep in workspace: found $version (expected $MAIN_VERSION)${NC}"
            ERRORS=$((ERRORS + 1))
        else
            echo -e "${GREEN}✅ $dep version correct in workspace${NC}"
        fi
    fi
done

# Check CLI and crate versions (they should use workspace inheritance or match)
for cargo_file in cli/Cargo.toml crates/*/Cargo.toml; do
    if [ -f "$cargo_file" ]; then
        # Check if using workspace inheritance
        if grep -q '^version.workspace = true' "$cargo_file"; then
            echo -e "${GREEN}✅ $(basename $(dirname $cargo_file)) uses workspace version${NC}"
        else
            # Check if version is specified directly
            pkg_version=$(grep '^version = ' "$cargo_file" | head -1 | cut -d'"' -f2)
            if [ -n "$pkg_version" ] && [ "$pkg_version" != "$MAIN_VERSION" ]; then
                echo -e "${RED}❌ Version mismatch in $cargo_file: found $pkg_version (expected $MAIN_VERSION)${NC}"
                ERRORS=$((ERRORS + 1))
            elif [ -n "$pkg_version" ]; then
                echo -e "${GREEN}✅ $(basename $(dirname $cargo_file)) version correct${NC}"
            fi
        fi
    fi
done

# Check README files
echo -e "\n${YELLOW}Checking README files...${NC}"
check_version "README.md" 'icarus = "[0-9]\+\.[0-9]\+\.[0-9]\+"' "dependency example"
check_version "README.md" 'icarus-canister = "[0-9]\+\.[0-9]\+\.[0-9]\+"' "canister dependency"
check_version "README.md" 'icarus-cli@[0-9]\+\.[0-9]\+\.[0-9]\+' "CLI installation"
check_version "docs/README.md" 'Version [0-9]\+\.[0-9]\+\.[0-9]\+' "docs version footer"

# Check migration guide - only check current version references
echo -e "\n${YELLOW}Checking migration guide...${NC}"
# Only check the "Current Version" line in Version Support Policy section
check_version "docs/migration-guide.md" 'Current Version ([0-9]\+\.[0-9]\+\.[0-9]\+)' "current version" true

# Special check for 0.2.0+ reference (this should stay as is)
echo -e "\n${YELLOW}Checking special version references...${NC}"
if grep -q "Version 0.2.0+" README.md; then
    echo -e "${GREEN}✅ Found Version 0.2.0+ reference (this is correct for BSL notice)${NC}"
fi

# Summary
echo -e "\n${YELLOW}========================================${NC}"
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}✅ All version references are consistent! ($MAIN_VERSION)${NC}"
    exit 0
else
    echo -e "${RED}❌ Found $ERRORS version inconsistencies!${NC}"
    echo -e "${YELLOW}Run './scripts/release.sh' to automatically fix these.${NC}"
    exit 1
fi