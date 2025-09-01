#!/bin/bash

# Quick Test Script - Fast local testing
# Usage: ./quick-test.sh

echo "⚡ Quick Local Test"
echo "=================="
echo ""

# Just the critical checks that often fail
echo "1. Testing documentation build..."
if RUSTDOCFLAGS="-D warnings" cargo doc --no-deps 2>&1 | grep -q "error"; then
    echo "❌ Documentation has errors:"
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps 2>&1 | grep "error" | head -10
    exit 1
else
    echo "✅ Docs OK"
fi

echo ""
echo "2. Testing doctests..."
if ! cargo test --doc 2>&1 | grep -q "FAILED"; then
    echo "✅ Doctests OK"
else
    echo "❌ Doctests failed:"
    cargo test --doc 2>&1 | grep -A5 "FAILED"
    exit 1
fi

echo ""
echo "3. Checking clippy..."
if cargo clippy 2>&1 | grep -q "warning"; then
    echo "⚠️  Clippy has warnings (would fail in CI with -D warnings):"
    cargo clippy 2>&1 | grep "warning" | head -5
else
    echo "✅ Clippy OK"
fi

echo ""
echo "=================="
echo "✅ Quick test passed - safe for full CI test"
echo ""
echo "Run ./test-ci.sh for complete CI simulation"