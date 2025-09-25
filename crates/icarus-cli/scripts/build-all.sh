#!/bin/bash

# Icarus CLI Multi-Platform Build Script
# Builds release binaries for all supported platforms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/target/release-builds"
VERSION=$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | cut -d'"' -f2)

echo -e "${GREEN}Building Icarus CLI v${VERSION}${NC}"
echo "========================================="

# Ensure we're in the project root
cd "${PROJECT_ROOT}"

# Create build directory
mkdir -p "${BUILD_DIR}"

# Function to build for a specific target
build_target() {
    local target=$1
    local output_name=$2
    local extension=${3:-""}
    
    echo -e "\n${YELLOW}Building for ${target}...${NC}"
    
    # Check if target is installed
    if ! rustup target list | grep -q "${target} (installed)"; then
        echo -e "${YELLOW}Installing target ${target}...${NC}"
        rustup target add "${target}"
    fi
    
    # Build the target
    if cargo build --release --target "${target}"; then
        # Copy binary to build directory with platform-specific name
        local source_file="${PROJECT_ROOT}/target/${target}/release/icarus${extension}"
        local dest_file="${BUILD_DIR}/icarus-${output_name}${extension}"
        
        if [ -f "${source_file}" ]; then
            cp "${source_file}" "${dest_file}"
            echo -e "${GREEN}✓ Built ${output_name}${NC}"
            
            # Strip debug symbols for smaller size (Unix only)
            if [[ "$extension" != ".exe" ]] && command -v strip &> /dev/null; then
                strip "${dest_file}" 2>/dev/null || true
            fi
            
            # Make executable (Unix only)
            if [[ "$extension" != ".exe" ]]; then
                chmod +x "${dest_file}"
            fi
            
            # Display file size
            if command -v du &> /dev/null; then
                local size=$(du -h "${dest_file}" | cut -f1)
                echo "  Size: ${size}"
            fi
        else
            echo -e "${RED}✗ Binary not found at ${source_file}${NC}"
            return 1
        fi
    else
        echo -e "${RED}✗ Build failed for ${target}${NC}"
        return 1
    fi
}

# Build for each platform
echo -e "\n${GREEN}Starting multi-platform builds...${NC}\n"

# macOS Apple Silicon (M1/M2/M3)
build_target "aarch64-apple-darwin" "darwin-arm64"

# macOS Intel
build_target "x86_64-apple-darwin" "darwin-x64"

# Linux x64
build_target "x86_64-unknown-linux-gnu" "linux-x64"

# Linux ARM64
build_target "aarch64-unknown-linux-gnu" "linux-arm64"

# Windows x64
build_target "x86_64-pc-windows-gnu" "windows-x64" ".exe"

# Summary
echo -e "\n${GREEN}=========================================${NC}"
echo -e "${GREEN}Build Summary:${NC}"
echo -e "${GREEN}=========================================${NC}"

if [ -d "${BUILD_DIR}" ]; then
    echo -e "\nBuilt binaries:"
    ls -lh "${BUILD_DIR}" | grep icarus- | awk '{print "  " $9 " (" $5 ")"}'
    
    # Calculate total size
    if command -v du &> /dev/null; then
        total_size=$(du -sh "${BUILD_DIR}" | cut -f1)
        echo -e "\nTotal size: ${total_size}"
    fi
else
    echo -e "${RED}No binaries were built${NC}"
    exit 1
fi

echo -e "\n${GREEN}✓ All builds completed successfully!${NC}"
echo -e "Binaries are available in: ${BUILD_DIR}"