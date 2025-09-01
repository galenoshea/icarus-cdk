#!/bin/bash

# Icarus CLI Packaging Script
# Creates distributable archives with metadata for each platform

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/target/release-builds"
DIST_DIR="${PROJECT_ROOT}/target/dist"
VERSION=$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | cut -d'"' -f2)
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

echo -e "${GREEN}Packaging Icarus CLI v${VERSION}${NC}"
echo "========================================="

# Ensure build directory exists and has binaries
if [ ! -d "${BUILD_DIR}" ]; then
    echo -e "${RED}Build directory not found. Run build-all.sh first.${NC}"
    exit 1
fi

# Create distribution directory
mkdir -p "${DIST_DIR}"

# Function to calculate SHA256
calculate_sha256() {
    local file=$1
    if command -v sha256sum &> /dev/null; then
        sha256sum "${file}" | cut -d' ' -f1
    elif command -v shasum &> /dev/null; then
        shasum -a 256 "${file}" | cut -d' ' -f1
    else
        echo "unavailable"
    fi
}

# Function to get file size in bytes
get_file_size() {
    local file=$1
    if command -v stat &> /dev/null; then
        # macOS/BSD stat
        stat -f%z "${file}" 2>/dev/null || \
        # GNU stat
        stat -c%s "${file}" 2>/dev/null || \
        echo "0"
    else
        echo "0"
    fi
}

# Function to package a binary
package_binary() {
    local binary_name=$1
    local platform=$2
    local archive_type=$3  # tar.gz or zip
    
    local binary_path="${BUILD_DIR}/${binary_name}"
    
    if [ ! -f "${binary_path}" ]; then
        echo -e "${YELLOW}Skipping ${platform} (binary not found)${NC}"
        return
    fi
    
    echo -e "\n${YELLOW}Packaging ${platform}...${NC}"
    
    # Create temporary directory for this package
    local temp_dir="${DIST_DIR}/temp_${platform}"
    mkdir -p "${temp_dir}"
    
    # Copy binary
    cp "${binary_path}" "${temp_dir}/icarus${4}"  # $4 is extension (.exe for Windows)
    
    # Create README
    cat > "${temp_dir}/README.md" << EOF
# Icarus CLI v${VERSION}

Platform: ${platform}
Built: ${TIMESTAMP}

## Installation

1. Move the icarus binary to a directory in your PATH
2. Make it executable (Unix only): chmod +x icarus
3. Verify installation: icarus --version

## Documentation

Visit https://icarus.dev/docs for complete documentation.

## Support

- GitHub Issues: https://github.com/icarus-platform/icarus-cli/issues
- Discord: https://discord.gg/icarus
EOF
    
    # Create LICENSE file (you may want to customize this)
    cat > "${temp_dir}/LICENSE" << EOF
Copyright (c) 2024 Icarus Platform

All rights reserved. This is proprietary software.
EOF
    
    # Create archive
    local archive_name="icarus-cli-v${VERSION}-${platform}"
    
    if [ "${archive_type}" = "zip" ]; then
        # Create ZIP for Windows
        cd "${temp_dir}"
        zip -q -r "${DIST_DIR}/${archive_name}.zip" .
        cd - > /dev/null
        local archive_path="${DIST_DIR}/${archive_name}.zip"
    else
        # Create tar.gz for Unix
        cd "${temp_dir}"
        tar -czf "${DIST_DIR}/${archive_name}.tar.gz" .
        cd - > /dev/null
        local archive_path="${DIST_DIR}/${archive_name}.tar.gz"
    fi
    
    # Calculate metadata
    local checksum=$(calculate_sha256 "${archive_path}")
    local size=$(get_file_size "${archive_path}")
    
    # Create metadata JSON
    cat > "${DIST_DIR}/${archive_name}.json" << EOF
{
    "version": "${VERSION}",
    "platform": "${platform}",
    "architecture": "$(echo ${platform} | cut -d'-' -f2)",
    "os": "$(echo ${platform} | cut -d'-' -f1)",
    "filename": "$(basename ${archive_path})",
    "checksum": "${checksum}",
    "algorithm": "sha256",
    "size": ${size},
    "created_at": "${TIMESTAMP}",
    "binary_name": "icarus${4}"
}
EOF
    
    # Clean up temp directory
    rm -rf "${temp_dir}"
    
    echo -e "${GREEN}✓ Packaged ${platform}${NC}"
    echo "  Archive: $(basename ${archive_path})"
    echo "  Size: $(( size / 1024 / 1024 )) MB"
    echo "  SHA256: ${checksum:0:16}..."
}

# Package each platform
echo -e "\n${GREEN}Creating distribution packages...${NC}"

# Unix platforms (tar.gz)
package_binary "icarus-darwin-arm64" "darwin-arm64" "tar.gz"
package_binary "icarus-darwin-x64" "darwin-x64" "tar.gz"
package_binary "icarus-linux-x64" "linux-x64" "tar.gz"
package_binary "icarus-linux-arm64" "linux-arm64" "tar.gz"

# Windows (zip)
package_binary "icarus-windows-x64.exe" "windows-x64" "zip" ".exe"

# Create master manifest
echo -e "\n${YELLOW}Creating master manifest...${NC}"

cat > "${DIST_DIR}/manifest.json" << EOF
{
    "version": "${VERSION}",
    "created_at": "${TIMESTAMP}",
    "platforms": [
EOF

# Add each platform to manifest
first=true
for json_file in "${DIST_DIR}"/*.json; do
    if [[ "$(basename ${json_file})" != "manifest.json" ]]; then
        if [ "$first" = true ]; then
            first=false
        else
            echo "," >> "${DIST_DIR}/manifest.json"
        fi
        # Extract just the platform info
        platform=$(basename "${json_file}" .json | sed "s/icarus-cli-v${VERSION}-//")
        echo -n "        \"${platform}\"" >> "${DIST_DIR}/manifest.json"
    fi
done

cat >> "${DIST_DIR}/manifest.json" << EOF

    ],
    "files": {
EOF

# Add file details to manifest
first=true
for json_file in "${DIST_DIR}"/*.json; do
    if [[ "$(basename ${json_file})" != "manifest.json" ]]; then
        if [ "$first" = true ]; then
            first=false
        else
            echo "," >> "${DIST_DIR}/manifest.json"
        fi
        platform=$(basename "${json_file}" .json | sed "s/icarus-cli-v${VERSION}-//")
        echo -n "        \"${platform}\": " >> "${DIST_DIR}/manifest.json"
        cat "${json_file}" | tr '\n' ' ' | sed 's/  */ /g' >> "${DIST_DIR}/manifest.json"
    fi
done

cat >> "${DIST_DIR}/manifest.json" << EOF

    }
}
EOF

# Summary
echo -e "\n${GREEN}=========================================${NC}"
echo -e "${GREEN}Packaging Summary:${NC}"
echo -e "${GREEN}=========================================${NC}"

echo -e "\nCreated packages:"
for archive in "${DIST_DIR}"/*.{tar.gz,zip} 2>/dev/null; do
    if [ -f "${archive}" ]; then
        size=$(du -h "${archive}" | cut -f1)
        echo "  $(basename ${archive}) (${size})"
    fi
done

echo -e "\nMetadata files:"
ls -1 "${DIST_DIR}"/*.json | while read json_file; do
    echo "  $(basename ${json_file})"
done

if command -v du &> /dev/null; then
    total_size=$(du -sh "${DIST_DIR}" | cut -f1)
    echo -e "\nTotal distribution size: ${total_size}"
fi

echo -e "\n${GREEN}✓ Packaging completed successfully!${NC}"
echo -e "Distribution packages are available in: ${DIST_DIR}"