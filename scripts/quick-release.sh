#!/bin/bash
set -e

# Quick release script that skips E2E tests for emergency fixes
# Usage: ./scripts/quick-release.sh [patch|minor|major]

RELEASE_TYPE="${1:-patch}"

echo "🚀 Quick Release (skipping E2E tests)"
echo "Release type: $RELEASE_TYPE"
echo ""
echo "⚠️  WARNING: This skips E2E tests!"
echo "Only use for emergency fixes where E2E tests are known to be safe to skip."
echo ""
read -p "Are you sure you want to continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Release cancelled."
    exit 1
fi

# Run the release with [skip-e2e] in commit message
echo "Running release with E2E tests skipped..."

# Create a temporary commit message file
COMMIT_MSG_FILE=$(mktemp)
echo "chore: release version [skip-e2e]" > "$COMMIT_MSG_FILE"
echo "" >> "$COMMIT_MSG_FILE"
echo "Emergency release - E2E tests skipped" >> "$COMMIT_MSG_FILE"

# Use cargo-release with custom commit message
CARGO_RELEASE_COMMIT_MESSAGE="chore: release version {{version}} [skip-e2e]" \
cargo release "$RELEASE_TYPE" \
  --execute \
  --no-confirm

rm -f "$COMMIT_MSG_FILE"

echo "✅ Quick release completed!"
echo ""
echo "Note: The release will skip E2E tests in CI due to [skip-e2e] tag."
echo "Please run E2E tests locally before the next regular release."