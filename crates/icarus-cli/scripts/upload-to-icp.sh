#!/bin/bash

# Icarus CLI Upload Script
# Uploads built CLI binaries to ICP canister for distribution

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/target/dist"
VERSION="${1:-$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | cut -d'"' -f2)}"
NETWORK="${NETWORK:-local}"
CANISTER="${CANISTER:-backend}"

echo -e "${BLUE}Icarus CLI Upload to ICP${NC}"
echo "========================================="
echo -e "Version: ${YELLOW}${VERSION}${NC}"
echo -e "Network: ${YELLOW}${NETWORK}${NC}"
echo -e "Canister: ${YELLOW}${CANISTER}${NC}"
echo ""

# Check if dist directory exists
if [ ! -d "${DIST_DIR}" ]; then
    echo -e "${RED}Error: Distribution directory not found at ${DIST_DIR}${NC}"
    echo "Please run ./scripts/build-all.sh and ./scripts/package.sh first"
    exit 1
fi

# Check if dfx is installed
if ! command -v dfx &> /dev/null; then
    echo -e "${RED}Error: dfx is not installed${NC}"
    echo "Please install dfx: https://internetcomputer.org/docs/current/developer-docs/setup/install/"
    exit 1
fi

# Function to convert file to hex blob for Motoko
file_to_hex_blob() {
    local file=$1
    echo -n "blob \""
    xxd -p -c 0 "$file" | tr -d '\n' | sed 's/\(..\)/\\\\x\1/g'
    echo -n "\""
}

# Function to upload a single platform binary
upload_platform() {
    local platform=$1
    local platform_variant=$2
    local archive_ext=$3
    
    local archive_path="${DIST_DIR}/icarus-cli-v${VERSION}-${platform}${archive_ext}"
    local checksum_file="${archive_path}.sha256"
    
    if [ ! -f "${archive_path}" ]; then
        echo -e "${YELLOW}Skipping ${platform} (not found)${NC}"
        return
    fi
    
    echo -e "\n${YELLOW}Uploading ${platform}...${NC}"
    
    # Read checksum
    local checksum=$(cat "${checksum_file}" | cut -d' ' -f1)
    echo "  Archive: $(basename "${archive_path}")"
    echo "  Size: $(du -h "${archive_path}" | cut -f1)"
    echo "  SHA256: ${checksum:0:16}..."
    
    # Create temporary file with the Motoko record
    local temp_file=$(mktemp)
    
    # Convert binary to hex (this is memory intensive for large files)
    echo -n '(record { 
        version = "'"${VERSION}"'";
        platform = variant { '"${platform_variant}"' };
        binary_data = ' >> "${temp_file}"
    
    # Convert file to blob format
    file_to_hex_blob "${archive_path}" >> "${temp_file}"
    
    echo -n ';
        checksum = "'"${checksum}"'";
        release_notes = opt "Release v'"${VERSION}"' - Uploaded via script";
    })' >> "${temp_file}"
    
    # Upload to canister
    if dfx canister --network "${NETWORK}" call "${CANISTER}" uploadCLIBinary "$(cat "${temp_file}")" 2>/dev/null; then
        echo -e "${GREEN}✓ Successfully uploaded ${platform}${NC}"
    else
        echo -e "${RED}✗ Failed to upload ${platform}${NC}"
        echo "  This might be because:"
        echo "  - You're not authorized (admin only)"
        echo "  - The binary already exists for this version"
        echo "  - The canister is not deployed"
    fi
    
    rm -f "${temp_file}"
}

# Main upload process
echo -e "${GREEN}Starting upload process...${NC}"

# Upload each platform
upload_platform "darwin-arm64" "DarwinArm64" ".tar.gz"
upload_platform "darwin-x64" "DarwinX64" ".tar.gz"
upload_platform "linux-x64" "LinuxX64" ".tar.gz"
upload_platform "linux-arm64" "LinuxArm64" ".tar.gz"
upload_platform "windows-x64" "WindowsX64" ".zip"

echo ""
echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}Upload process complete!${NC}"
echo ""

# Verify the upload
echo -e "${YELLOW}Verifying upload...${NC}"
echo ""

echo "Latest version in canister:"
dfx canister --network "${NETWORK}" call "${CANISTER}" getLatestCLIVersion || echo "Failed to get latest version"

echo ""
echo "Available platforms for v${VERSION}:"
dfx canister --network "${NETWORK}" call "${CANISTER}" getCLIPlatforms "(\"${VERSION}\")" || echo "Failed to get platforms"

echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "1. Test download from the Install page in your webapp"
echo "2. Verify the installation script works:"
echo "   curl -sSL https://your-app.ic0.app/install | bash"
echo ""
echo -e "${GREEN}✨ Done!${NC}"