# Testing icarus-core

This document explains how to properly test the icarus-core crate.

## Quick Testing

Use the provided test script for the easiest experience:

```bash
./test.sh
```

## Manual Testing

### Required Feature Flag

Due to the Internet Computer integration, tests require the `test-utils` feature to avoid IC environment dependencies:

```bash
# Run all tests
cargo test --features test-utils

# Run specific test categories
cargo test --features test-utils --lib           # Unit tests only
cargo test --features test-utils --test integration_comprehensive  # Integration tests
cargo test --features test-utils --test property_tests           # Property tests
cargo test --features test-utils --doc           # Documentation tests
```

### Why the test-utils Feature is Needed

The icarus-core crate uses IC-specific time functions (`ic_cdk::api::time()`) that only work inside Internet Computer canisters. The `test-utils` feature enables fallback implementations using `std::time::SystemTime` for testing in normal environments.

Without `test-utils`, you'll see errors like:
```
time should only be called inside canisters.
```

## Test Categories

### Unit Tests (40 tests)
Located in `src/` files with `#[cfg(test)]` modules:
- Newtype validation and operations
- Protocol type serialization
- Error handling and conversion
- Tool schema validation

### Integration Tests (13 tests)
Located in `tests/integration_comprehensive.rs`:
- End-to-end workflows
- Cross-module functionality
- Serialization round-trips
- Concurrent operations

### Property Tests (14 tests)
Located in `tests/property_tests.rs`:
- Invariant validation with proptest
- Roundtrip properties
- Error handling properties
- Zero-copy optimization validation

### Documentation Tests (8 tests)
Embedded in documentation comments:
- Example code validation
- API usage demonstrations

## Performance Testing

```bash
# Run benchmarks
cargo bench

# Run with optimizations
cargo test --features test-utils --release
```

## CI/CD Testing

For automated testing environments, always use:

```bash
cargo test --features test-utils --all-targets
```

## Testing in IC Environment

When testing actual canister deployment, the `test-utils` feature should be disabled so that real IC time functions are used:

```bash
# For canister testing (requires IC environment)
cargo test --no-default-features
```

This will use the actual IC time APIs and should only be run in a proper IC testing environment.