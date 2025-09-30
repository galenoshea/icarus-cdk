#!/bin/bash
# Test script for icarus-core that automatically enables test-utils feature

echo "Running icarus-core tests with test-utils feature..."
cargo test --features test-utils "$@"