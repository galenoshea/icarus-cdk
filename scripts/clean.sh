#!/bin/bash

# Comprehensive cleanup script for the Icarus SDK project
# This script removes all build artifacts, temporary files, and generated content

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🧹 Icarus SDK Deep Clean"
echo "========================"
echo ""

# Function to print colored output
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Function to safely remove directories/files
safe_remove() {
    if [ -e "$1" ]; then
        rm -rf "$1"
        print_status "Removed: $1"
    fi
}

cd "$PROJECT_ROOT"

# 1. Clean Rust/Cargo build artifacts
echo "Cleaning Rust build artifacts..."
if command -v cargo &> /dev/null; then
    cargo clean
    print_status "Cargo clean completed"
else
    print_warning "Cargo not found, skipping cargo clean"
fi

# 2. Clean dfx artifacts
echo ""
echo "Cleaning dfx artifacts..."
safe_remove ".dfx"
safe_remove "canister_ids.json"
safe_remove ".vessel"

# Stop dfx if running
if command -v dfx &> /dev/null; then
    dfx stop 2>/dev/null || true
    print_status "Stopped dfx (if running)"
fi

# 3. Clean WASM artifacts
echo ""
echo "Cleaning WASM artifacts..."
find . -name "*.wasm" -type f -delete 2>/dev/null || true
find . -name "*.wasm.gz" -type f -delete 2>/dev/null || true
find . -name "*.did" -type f ! -path "./src/*" -delete 2>/dev/null || true
print_status "Removed WASM and Candid files"

# 4. Clean target directories (including workspace members)
echo ""
echo "Cleaning target directories..."
safe_remove "target"
safe_remove "cli/target"
safe_remove "crates/*/target"
safe_remove "examples/*/target"

# 5. Clean test artifacts
echo ""
echo "Cleaning test artifacts..."
find . -name "*.profraw" -type f -delete 2>/dev/null || true
find . -name "*.profdata" -type f -delete 2>/dev/null || true
safe_remove "tarpaulin-report.html"
safe_remove "cobertura.xml"

# 6. Clean node_modules (if any)
echo ""
echo "Cleaning Node.js artifacts..."
safe_remove "node_modules"
safe_remove "package-lock.json"

# 7. Clean temporary and cache files
echo ""
echo "Cleaning temporary files..."
find . -name ".DS_Store" -type f -delete 2>/dev/null || true
find . -name "Thumbs.db" -type f -delete 2>/dev/null || true
find . -name "*~" -type f -delete 2>/dev/null || true
find . -name "*.swp" -type f -delete 2>/dev/null || true
find . -name "*.swo" -type f -delete 2>/dev/null || true
find . -name "*.orig" -type f -delete 2>/dev/null || true
find . -name "*.rej" -type f -delete 2>/dev/null || true
print_status "Removed temporary files"

# 8. Clean .icarus directories
echo ""
echo "Cleaning .icarus directories..."
find . -name ".icarus" -type d -exec rm -rf {} + 2>/dev/null || true
print_status "Removed .icarus directories"

# 9. Clean Cargo.lock files in examples (keep main one)
echo ""
echo "Cleaning example Cargo.lock files..."
find examples -name "Cargo.lock" -type f -delete 2>/dev/null || true
print_status "Removed example Cargo.lock files"

# 10. Clean any Python artifacts
echo ""
echo "Cleaning Python artifacts..."
find . -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
find . -name "*.pyc" -type f -delete 2>/dev/null || true
find . -name "*.pyo" -type f -delete 2>/dev/null || true
find . -name ".pytest_cache" -type d -exec rm -rf {} + 2>/dev/null || true
print_status "Removed Python artifacts"

# 11. Clean IDE directories (optional, prompted)
echo ""
read -p "Remove IDE directories (.vscode, .idea)? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    safe_remove ".vscode"
    safe_remove ".idea"
    safe_remove "*.iml"
    print_status "Removed IDE directories"
else
    print_status "Skipped IDE directories"
fi

# 12. Clean release artifacts
echo ""
echo "Cleaning release artifacts..."
safe_remove "target/package"
safe_remove "target/release"
safe_remove "target/debug"
print_status "Removed release artifacts"

# 13. Summary
echo ""
echo "🎉 Deep clean complete!"
echo ""
echo "The following have been cleaned:"
echo "  • Rust build artifacts (target/)"
echo "  • DFX artifacts (.dfx/, canister_ids.json)"
echo "  • WASM and Candid files"
echo "  • Test coverage reports"
echo "  • Temporary and cache files"
echo "  • Python artifacts"

# Check disk usage after cleanup
if command -v du &> /dev/null; then
    echo ""
    echo "Project size after cleanup:"
    du -sh "$PROJECT_ROOT" 2>/dev/null || true
fi

echo ""
echo "To rebuild the project, run:"
echo "  cargo build"
echo ""
echo "To run a specific example:"
echo "  cd examples/<example-name>"
echo "  icarus build"