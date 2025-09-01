# Testing Guide for Icarus SDK

This document provides comprehensive guidance on testing the Icarus SDK.

## Overview

The Icarus SDK uses a multi-layered testing approach to ensure code quality and prevent regressions:

1. **Unit Tests** - Test individual functions and modules
2. **Integration Tests** - Test cross-module interactions
3. **Pre-commit Hooks** - Catch issues before committing
4. **CI/CD Pipeline** - Automated testing on every push
5. **Coverage Reporting** - Track test coverage metrics

## Test Structure

```
tests/
├── unit/              # Unit tests for individual modules
│   ├── test_prompts.rs
│   ├── test_session.rs
│   └── test_tools.rs
├── integration/       # Integration tests
│   └── test_sdk_workflow.rs
└── examples/         # Example usage (also serves as tests)
```

## Running Tests

### Basic Commands

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run specific test
cargo test test_prompt_builder

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release
```

### Coverage

```bash
# Install coverage tool
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --html
# Open target/llvm-cov/html/index.html

# Generate lcov report for CI
cargo llvm-cov --lcov --output-path lcov.info
```

## Writing Tests

### Unit Tests

Unit tests should be placed in `tests/unit/` and focus on testing individual functions:

```rust
#[test]
fn test_function_behavior() {
    let result = function_under_test(input);
    assert_eq!(result, expected);
}

#[test]
fn test_error_handling() {
    let result = function_that_may_fail(bad_input);
    assert!(result.is_err());
}
```

### Integration Tests

Integration tests go in `tests/integration/` and test complete workflows:

```rust
#[tokio::test]
async fn test_complete_workflow() {
    // Setup
    let system = setup_test_system().await;
    
    // Execute workflow
    let result = system.execute_workflow().await;
    
    // Verify
    assert!(result.is_successful());
    
    // Cleanup
    cleanup_test_system(system).await;
}
```

### Async Tests

For async code, use `#[tokio::test]`:

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

## Pre-commit Hooks

Install the pre-commit hooks to catch issues early:

```bash
./scripts/install-hooks.sh
```

This installs two hooks:
- **pre-commit**: Runs formatting, clippy, and tests
- **pre-push**: Runs the full CI test suite

To skip hooks temporarily:
```bash
git commit --no-verify
git push --no-verify
```

## CI/CD Testing

The CI pipeline runs on every push and pull request:

1. **Formatting Check** - Ensures consistent code style
2. **Clippy Analysis** - Static analysis for common mistakes
3. **Unit Tests** - All unit tests must pass
4. **Integration Tests** - All integration tests must pass
5. **Doc Tests** - Documentation examples must compile
6. **Documentation Build** - Ensures docs build without warnings
7. **WASM Build** - Verifies WASM compilation

## Local CI Testing

Before pushing, run the local CI test:

```bash
./scripts/test-ci.sh
```

This runs the same checks as GitHub Actions locally.

## Test Best Practices

### 1. Test Naming
- Use descriptive names: `test_auth_with_invalid_principal`
- Group related tests with common prefixes
- Use `should_` prefix for behavior: `test_should_reject_anonymous`

### 2. Test Organization
- One test file per module
- Group related assertions in single test
- Use helper functions to reduce duplication

### 3. Test Data
- Use builders for complex test data
- Create fixtures for reusable test data
- Clean up resources in tests

### 4. Assertions
- Use specific assertions: `assert_eq!` over `assert!`
- Include helpful messages in assertions
- Test both success and failure paths

### 5. Coverage Goals
- Minimum: 70% overall coverage
- Target: 85% for core functionality
- Critical paths: 95% coverage

## Troubleshooting

### Tests Pass Locally but Fail in CI

1. Check for environment differences
2. Ensure all files are committed
3. Check for timing/race conditions
4. Verify dependencies are specified

### Clippy Warnings

Run clippy locally with the same flags as CI:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Coverage Not Generated

Ensure you have the required tools:
```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
```

## Release Testing

Before releasing, run:
```bash
./scripts/test-release.sh
```

This verifies:
- No uncommitted changes
- All tests pass
- All crates can be published
- Documentation builds

## Performance Testing

For performance-critical code:

```rust
#[bench]
fn bench_critical_function(b: &mut Bencher) {
    b.iter(|| {
        critical_function()
    });
}
```

Run benchmarks:
```bash
cargo bench
```

## Security Testing

- Run `cargo audit` regularly
- Use fuzzing for input validation
- Test with malformed inputs
- Verify authentication/authorization

## Continuous Improvement

- Review test failures for patterns
- Add tests for every bug fix
- Refactor tests when they become complex
- Keep tests fast and isolated