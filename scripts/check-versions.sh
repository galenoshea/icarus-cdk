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
            # Skip lines that contain historical markers
            local lines_to_check=$(grep -n "$pattern" "$file" | grep -v -E "(Historical|legacy|was a)" | grep -E "(Current Version|current version|Latest|latest)" | cut -d: -f1)
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

# Check workspace.package version
WORKSPACE_VERSION=$(grep -A1 '^\[workspace\.package\]' Cargo.toml | grep '^version = ' | cut -d'"' -f2)
if [ "$WORKSPACE_VERSION" != "$MAIN_VERSION" ]; then
    echo -e "${RED}❌ Workspace package version mismatch: found $WORKSPACE_VERSION (expected $MAIN_VERSION)${NC}"
    ERRORS=$((ERRORS + 1))
else
    echo -e "${GREEN}✅ Workspace package version: $WORKSPACE_VERSION${NC}"
fi

# Check ALL icarus dependencies in the root Cargo.toml (both [dependencies] and [workspace.dependencies])
echo -e "\n${YELLOW}Checking internal dependencies in root Cargo.toml...${NC}"
for dep in "icarus-core" "icarus-derive" "icarus-canister"; do
    # Find ALL occurrences of the dependency with version
    all_versions=$(grep "$dep = {" Cargo.toml | grep -o 'version = "[^"]*"' | grep -o '"[^"]*"' | tr -d '"' | sort -u)
    
    if [ -n "$all_versions" ]; then
        for version in $all_versions; do
            if [ "$version" != "$MAIN_VERSION" ]; then
                echo -e "${RED}❌ Version mismatch for $dep: found $version (expected $MAIN_VERSION)${NC}"
                ERRORS=$((ERRORS + 1))
            fi
        done
        
        # Check if all are consistent
        version_count=$(echo "$all_versions" | wc -l | tr -d ' ')
        if [ "$version_count" -eq 1 ] && [ "$all_versions" = "$MAIN_VERSION" ]; then
            echo -e "${GREEN}✅ $dep version consistent: $MAIN_VERSION${NC}"
        fi
    fi
done

# Check CLI and crate versions (they should use workspace inheritance or match)
# Note: Examples have their own versions and are excluded
echo -e "\n${YELLOW}Checking workspace member Cargo.toml files...${NC}"
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

# Check examples separately - they have their own versions but should use correct SDK version
echo -e "\n${YELLOW}Checking example dependencies...${NC}"
for example_dir in examples/*/; do
    if [ -d "$example_dir" ]; then
        example_name=$(basename "$example_dir")
        cargo_file="${example_dir}Cargo.toml"
        if [ -f "$cargo_file" ]; then
            # Check icarus dependency version in examples
            if grep -q 'icarus = {' "$cargo_file"; then
                # For local development, path dependencies are fine
                if grep -q 'icarus = { path' "$cargo_file"; then
                    echo -e "${GREEN}✅ $example_name uses local icarus${NC}"
                else
                    # Check version if specified
                    icarus_version=$(grep 'icarus = ' "$cargo_file" | grep -o '"[0-9]\+\.[0-9]\+\.[0-9]\+"' | tr -d '"')
                    if [ -n "$icarus_version" ] && [ "$icarus_version" != "$MAIN_VERSION" ]; then
                        echo -e "${RED}❌ $example_name uses icarus version $icarus_version (expected $MAIN_VERSION)${NC}"
                        ERRORS=$((ERRORS + 1))
                    fi
                fi
            fi
        fi
    fi
done

# Check README files
echo -e "\n${YELLOW}Checking README files...${NC}"
check_version "README.md" 'icarus = "[0-9]\+\.[0-9]\+\.[0-9]\+"' "icarus dependency in README"
check_version "README.md" 'icarus-cli = "[0-9]\+\.[0-9]\+\.[0-9]\+"' "icarus-cli dependency in README"
check_version "docs/README.md" 'icarus = "[0-9]\+\.[0-9]\+\.[0-9]\+"' "icarus dependency in docs README"
check_version "docs/README.md" 'version [0-9]\+\.[0-9]\+\.[0-9]\+' "version reference in docs"

# Check migration guide - skip historical references
echo -e "\n${YELLOW}Checking migration guide...${NC}"
check_version "docs/migration-guide.md" '[0-9]\+\.[0-9]\+\.[0-9]\+' "version in migration guide" true

# Check for version references in scripts
echo -e "\n${YELLOW}Checking special version references...${NC}"
# Check if there are any hardcoded versions in release.sh
if [ -f "scripts/release.sh" ]; then
    if grep -q '[0-9]\+\.[0-9]\+\.[0-9]\+' "scripts/release.sh"; then
        # Only flag if it's not a comment or example
        hardcoded=$(grep '[0-9]\+\.[0-9]\+\.[0-9]\+' "scripts/release.sh" | grep -v '^#' | grep -v 'echo' || true)
        if [ -n "$hardcoded" ]; then
            echo -e "${YELLOW}⚠️  Found hardcoded version in release.sh - please verify${NC}"
        fi
    fi
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