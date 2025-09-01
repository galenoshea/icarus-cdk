# Icarus SDK Tests

This directory contains the comprehensive test suite for the Icarus SDK.

## Structure

- **unit/** - Unit tests for individual functions and modules
- **integration/** - Integration tests for cross-crate functionality
- **examples/** - Example usage that also serves as documentation and tests

## Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --test unit_*

# Run only integration tests  
cargo test --test integration_*

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html

# Run specific test
cargo test test_name
```

## Writing Tests

### Unit Tests
Unit tests should focus on testing individual functions in isolation. Place them in `tests/unit/` with descriptive names like `test_auth.rs`, `test_storage.rs`.

### Integration Tests
Integration tests should test the interaction between multiple components. Place them in `tests/integration/` with names like `test_tool_lifecycle.rs`.

### Examples
Examples in `tests/examples/` should demonstrate real-world usage patterns and also serve as tests to ensure the examples work.

## Coverage Goals

- Minimum: 70% overall coverage
- Target: 85% for core functionality
- Critical paths: 95% coverage