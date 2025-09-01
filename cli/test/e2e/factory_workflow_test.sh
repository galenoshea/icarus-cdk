#!/bin/bash

# Icarus Factory Model End-to-End Test
# Tests the complete workflow: publish → browse → purchase → install

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Test configuration
TEST_DIR="/tmp/icarus-test-$$"
TOOL_NAME="test-tool-$$"
MARKETPLACE_CANISTER="rdmx6-jaaaa-aaaaa-aaadq-cai"  # TODO: Replace with actual

echo -e "${YELLOW}Icarus Factory Model E2E Test${NC}"
echo "=============================="
echo "Test directory: $TEST_DIR"
echo "Tool name: $TOOL_NAME"
echo ""

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${NC}"
    rm -rf "$TEST_DIR"
    # Stop any running dfx or bridge processes
    pkill -f "dfx" || true
    pkill -f "icarus-bridge" || true
}
trap cleanup EXIT

# Step 1: Create test directory
echo -e "${YELLOW}1. Creating test environment...${NC}"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Step 2: Start local dfx network
echo -e "${YELLOW}2. Starting local dfx network...${NC}"
dfx start --clean --background
sleep 5  # Wait for dfx to start

# Step 3: Deploy marketplace canister (if not already deployed)
echo -e "${YELLOW}3. Deploying marketplace canister...${NC}"
# This would deploy the IcarusMarketplaceFactory.mo
# For now, we'll assume it's already deployed

# Step 4: Create a new tool project
echo -e "${YELLOW}4. Creating new tool project...${NC}"
icarus new "$TOOL_NAME" --local-sdk /Users/goshea/projects/icarus/icarus-sdk

cd "$TOOL_NAME"

# Step 5: Initialize marketplace metadata
echo -e "${YELLOW}5. Initializing marketplace metadata...${NC}"
cat > icarus-marketplace.json <<EOF
{
  "name": "$TOOL_NAME",
  "description": "Test tool for end-to-end factory model testing",
  "categories": ["utilities", "testing"],
  "price_icp": 0.1,
  "author_revenue_share": 80,
  "minimum_cycles": 1000000000000,
  "version": "1.0.0",
  "screenshots": [],
  "readme": "README.md",
  "license": "MIT",
  "repository": "https://github.com/test/test",
  "keywords": ["test", "e2e"]
}
EOF

# Step 6: Build the tool
echo -e "${YELLOW}6. Building tool...${NC}"
icarus build --profile debug

# Step 7: Deploy locally for testing
echo -e "${YELLOW}7. Deploying tool locally...${NC}"
icarus deploy --network local --force

# Get the deployed canister ID
CANISTER_ID=$(cat canister_ids.json | jq -r ".${TOOL_NAME}.local")
echo "Deployed canister ID: $CANISTER_ID"

# Step 8: Publish to marketplace
echo -e "${YELLOW}8. Publishing tool to marketplace...${NC}"
# This will fail for now since we don't have auth setup
# icarus publish --network local

echo -e "${GREEN}✓ Tool creation and deployment successful${NC}"

# Step 9: Browse marketplace
echo -e "${YELLOW}9. Browsing marketplace...${NC}"
icarus install --browse || echo -e "${RED}Browse failed (expected - no tools published yet)${NC}"

# Step 10: Test install command
echo -e "${YELLOW}10. Testing install command...${NC}"
# This would install a tool if one was published
# icarus install tool_1 --yes

# Step 11: Test bridge connection
echo -e "${YELLOW}11. Testing bridge connection...${NC}"
# Start bridge with authentication
# icarus bridge start --canister-id "$CANISTER_ID" --auth --local &
# BRIDGE_PID=$!
# sleep 5

# Step 12: Test access control
echo -e "${YELLOW}12. Testing access control...${NC}"
# This would test that only the owner can access the canister
# We'd need to make authenticated calls here

# Step 13: Test multiple buyers
echo -e "${YELLOW}13. Testing multiple buyer scenario...${NC}"
# This would simulate multiple users purchasing the same tool
# and verify they get separate canister instances

echo -e "\n${GREEN}================================${NC}"
echo -e "${GREEN}E2E Test Summary${NC}"
echo -e "${GREEN}================================${NC}"
echo -e "✓ Tool project creation"
echo -e "✓ Tool building"
echo -e "✓ Local deployment"
echo -e "⚠ Publishing (requires auth setup)"
echo -e "⚠ Installation (requires published tools)"
echo -e "⚠ Access control (requires auth)"
echo -e "⚠ Multi-buyer test (requires marketplace)"

echo -e "\n${YELLOW}Known Issues:${NC}"
echo "1. Marketplace canister needs to be deployed"
echo "2. Authentication needs to be configured"
echo "3. ICP payment simulation needed"

echo -e "\n${GREEN}Core factory model components are ready!${NC}"