#!/bin/bash

# Icarus CLI Installation Script
# Downloads and installs the Icarus CLI from the ICP marketplace

set -e

# Configuration
ICARUS_API="${ICARUS_API:-https://uzt4z-lp777-77774-qaabq-cai.icp0.io}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.icarus/bin}"
CONFIG_DIR="${HOME}/.icarus"
VERSION="${VERSION:-latest}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Detect platform
detect_platform() {
    local os=""
    local arch=""
    
    # Detect OS
    case "$(uname -s)" in
        Darwin*)
            os="darwin"
            ;;
        Linux*)
            os="linux"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            os="windows"
            ;;
        *)
            echo -e "${RED}Unsupported operating system: $(uname -s)${NC}"
            exit 1
            ;;
    esac
    
    # Detect architecture
    case "$(uname -m)" in
        x86_64)
            arch="x64"
            ;;
        arm64|aarch64)
            arch="arm64"
            ;;
        *)
            echo -e "${RED}Unsupported architecture: $(uname -m)${NC}"
            exit 1
            ;;
    esac
    
    echo "${os}-${arch}"
}

# Check for required commands
check_requirements() {
    local missing=()
    
    command -v curl >/dev/null 2>&1 || missing+=("curl")
    command -v tar >/dev/null 2>&1 || missing+=("tar")
    
    if [ ${#missing[@]} -gt 0 ]; then
        echo -e "${RED}Missing required commands: ${missing[*]}${NC}"
        echo "Please install them and try again."
        exit 1
    fi
}

# Get or create authentication token
get_auth_token() {
    local token_file="${CONFIG_DIR}/auth_token"
    
    # Check if token exists and is valid
    if [ -f "${token_file}" ]; then
        local token=$(cat "${token_file}")
        # Validate token by making a test request
        if curl -s -H "Authorization: Bearer ${token}" \
                "${ICARUS_API}/api/v1/cli/validate-token" | grep -q "valid"; then
            echo "${token}"
            return 0
        fi
    fi
    
    echo -e "${YELLOW}Authentication required${NC}"
    echo "Please visit the following URL to authenticate:"
    echo -e "${BLUE}${ICARUS_API}/cli/auth${NC}"
    echo ""
    echo "After authenticating, paste your token here:"
    read -r token
    
    # Save token for future use
    mkdir -p "${CONFIG_DIR}"
    echo "${token}" > "${token_file}"
    chmod 600 "${token_file}"
    
    echo "${token}"
}

# Download CLI binary
download_cli() {
    local platform=$1
    local token=$2
    local temp_dir=$(mktemp -d)
    
    echo -e "${YELLOW}Downloading Icarus CLI for ${platform}...${NC}"
    
    # Request download token from canister
    local download_response=$(curl -s -H "Authorization: Bearer ${token}" \
        "${ICARUS_API}/api/v1/cli/download-token?platform=${platform}&version=${VERSION}")
    
    if echo "${download_response}" | grep -q "error"; then
        echo -e "${RED}Failed to get download token: ${download_response}${NC}"
        rm -rf "${temp_dir}"
        exit 1
    fi
    
    # Extract download URL and checksum
    local download_url=$(echo "${download_response}" | grep -o '"url":"[^"]*"' | cut -d'"' -f4)
    local expected_checksum=$(echo "${download_response}" | grep -o '"checksum":"[^"]*"' | cut -d'"' -f4)
    
    # Download the archive
    local archive_path="${temp_dir}/icarus.tar.gz"
    echo "Downloading from: ${download_url}"
    
    if ! curl -L -o "${archive_path}" "${download_url}"; then
        echo -e "${RED}Download failed${NC}"
        rm -rf "${temp_dir}"
        exit 1
    fi
    
    # Verify checksum
    echo -e "${YELLOW}Verifying checksum...${NC}"
    local actual_checksum=""
    
    if command -v sha256sum >/dev/null 2>&1; then
        actual_checksum=$(sha256sum "${archive_path}" | cut -d' ' -f1)
    elif command -v shasum >/dev/null 2>&1; then
        actual_checksum=$(shasum -a 256 "${archive_path}" | cut -d' ' -f1)
    else
        echo -e "${YELLOW}Warning: Cannot verify checksum (sha256sum/shasum not found)${NC}"
    fi
    
    if [ -n "${actual_checksum}" ] && [ "${actual_checksum}" != "${expected_checksum}" ]; then
        echo -e "${RED}Checksum verification failed!${NC}"
        echo "Expected: ${expected_checksum}"
        echo "Actual: ${actual_checksum}"
        rm -rf "${temp_dir}"
        exit 1
    fi
    
    # Extract archive
    echo -e "${YELLOW}Extracting...${NC}"
    tar -xzf "${archive_path}" -C "${temp_dir}"
    
    # Install binary
    mkdir -p "${INSTALL_DIR}"
    local binary_name="icarus"
    [ "${platform}" = "windows-x64" ] && binary_name="icarus.exe"
    
    if [ -f "${temp_dir}/${binary_name}" ]; then
        mv "${temp_dir}/${binary_name}" "${INSTALL_DIR}/"
        chmod +x "${INSTALL_DIR}/${binary_name}"
        echo -e "${GREEN}✓ Installed to ${INSTALL_DIR}/${binary_name}${NC}"
    else
        echo -e "${RED}Binary not found in archive${NC}"
        rm -rf "${temp_dir}"
        exit 1
    fi
    
    # Clean up
    rm -rf "${temp_dir}"
}

# Add to PATH
add_to_path() {
    local shell_rc=""
    
    # Detect shell and RC file
    if [ -n "${BASH_VERSION}" ]; then
        shell_rc="${HOME}/.bashrc"
        [ -f "${HOME}/.bash_profile" ] && shell_rc="${HOME}/.bash_profile"
    elif [ -n "${ZSH_VERSION}" ]; then
        shell_rc="${HOME}/.zshrc"
    elif [ -f "${HOME}/.profile" ]; then
        shell_rc="${HOME}/.profile"
    else
        echo -e "${YELLOW}Could not detect shell RC file${NC}"
        echo "Please add ${INSTALL_DIR} to your PATH manually:"
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        return
    fi
    
    # Check if already in PATH
    if echo "$PATH" | grep -q "${INSTALL_DIR}"; then
        echo -e "${GREEN}✓ ${INSTALL_DIR} is already in PATH${NC}"
        return
    fi
    
    # Add to RC file
    echo "" >> "${shell_rc}"
    echo "# Icarus CLI" >> "${shell_rc}"
    echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "${shell_rc}"
    
    echo -e "${GREEN}✓ Added to PATH in ${shell_rc}${NC}"
    echo -e "${YELLOW}Please restart your terminal or run:${NC}"
    echo "  source ${shell_rc}"
}

# Main installation flow
main() {
    echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║     Icarus CLI Installation Script     ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
    echo ""
    
    # Check requirements
    check_requirements
    
    # Detect platform
    PLATFORM=$(detect_platform)
    echo -e "${GREEN}✓ Detected platform: ${PLATFORM}${NC}"
    
    # Get authentication token
    echo ""
    AUTH_TOKEN=$(get_auth_token)
    echo -e "${GREEN}✓ Authentication successful${NC}"
    
    # Download and install
    echo ""
    download_cli "${PLATFORM}" "${AUTH_TOKEN}"
    
    # Add to PATH
    echo ""
    add_to_path
    
    # Verify installation
    echo ""
    if "${INSTALL_DIR}/icarus" --version >/dev/null 2>&1; then
        local installed_version=$("${INSTALL_DIR}/icarus" --version | cut -d' ' -f2)
        echo -e "${GREEN}✓ Icarus CLI v${installed_version} installed successfully!${NC}"
    else
        echo -e "${YELLOW}Installation complete, but could not verify${NC}"
    fi
    
    echo ""
    echo -e "${GREEN}Next steps:${NC}"
    echo "  1. Restart your terminal or run: source ~/.bashrc"
    echo "  2. Verify installation: icarus --version"
    echo "  3. Initialize bridge: icarus bridge init"
    echo "  4. Browse marketplace: icarus marketplace list"
    echo ""
    echo -e "${BLUE}Happy building with Icarus! 🚀${NC}"
}

# Handle errors
trap 'echo -e "\n${RED}Installation failed${NC}"; exit 1' ERR

# Run main installation
main "$@"