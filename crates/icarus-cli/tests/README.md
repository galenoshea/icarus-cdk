# Icarus CLI Test Suite

Comprehensive end-to-end test suite for the Icarus SDK CLI, validating complete workflows from project creation to deployment.

## Overview

This test suite provides comprehensive validation of the Icarus CLI functionality, following rust_best_practices.md requirements and ensuring reliable operation in CI environments.

## Test Structure

### Core Test Files

- **`e2e_workflow_tests.rs`** - Complete workflow validation (create → build → deploy)
- **`mcp_integration_tests.rs`** - MCP client integration and server management  
- **`template_validation_tests.rs`** - Template system validation and customization
- **`integration_tests.rs`** - Basic CLI command validation
- **`unit_tests.rs`** - Unit tests for individual components

### Support Infrastructure

- **`test_utils/mod.rs`** - Common test utilities and helpers
- **`Makefile`** - Test runner and CI integration
- **`README.md`** - This documentation

## Test Categories

### 1. End-to-End Workflow Tests (`e2e_workflow_tests.rs`)

Tests the complete SDK workflow:

- **Project Creation**: All templates with validation
- **Build Process**: Debug/release builds, feature flags, targets
- **Template Validation**: Structure, content, and dependencies
- **Error Handling**: Invalid inputs, edge cases
- **Performance**: Creation and build times
- **Concurrency**: Parallel operations

```rust
// Example: Complete workflow test
#[tokio::test]
async fn test_complete_basic_workflow() {
    let project = TestProject::new("workflow-basic");
    
    // Create → Validate → Build
    project.create_with_template("basic").assert().success();
    assert!(project.has_file("Cargo.toml"));
    project.build_command().assert().success();
}
```

### 2. MCP Integration Tests (`mcp_integration_tests.rs`)

Validates MCP server management:

- **Server Management**: Add, remove, list, status operations
- **Client Integration**: Claude Desktop, Claude Code, ChatGPT Desktop
- **Configuration**: JSON format, validation, backup/restore
- **Network Support**: Local, IC, testnet networks
- **Error Handling**: Invalid inputs, missing servers

```rust
// Example: MCP server management
#[test]
fn test_mcp_server_lifecycle() {
    let helper = McpTestHelper::new();
    
    // Add server
    helper.add_server("rdmx6-jaaaa-aaaaa-aaadq-cai", "claude-desktop")
        .assert().success();
    
    // Verify listing
    helper.list_servers()
        .assert().success()
        .stdout(predicate::str::contains("rdmx6-jaaaa-aaaaa-aaadq-cai"));
}
```

### 3. Template Validation Tests (`template_validation_tests.rs`)

Comprehensive template system testing:

- **All Templates**: Basic, advanced, MCP server, dApp
- **File Structure**: Required files and directories
- **Content Validation**: Cargo.toml, lib.rs, dfx.json correctness
- **Customization**: Name transformations, feature flags
- **Edge Cases**: Special characters, long names

```rust
// Example: Template validation
#[test]
fn test_template_structure() {
    let helper = TemplateTestHelper::new();
    
    helper.create_project("test-basic", "basic")
        .exists()
        .has_file("Cargo.toml")
        .cargo_toml_valid()
        .lib_rs_valid();
}
```

### 4. Integration Tests (`integration_tests.rs`)

Basic CLI functionality:

- **Command Help**: All commands and subcommands
- **Global Flags**: Verbose, quiet, force
- **Basic Operations**: Project creation, error handling
- **Flag Combinations**: Valid and invalid combinations

### 5. Unit Tests (`unit_tests.rs`)

Component-level testing:

- **Configuration**: MCP config serialization, validation
- **Templates**: String transformations, context validation
- **Utilities**: Client detection, project validation
- **Error Handling**: Custom error types and messages

## Test Utilities (`test_utils/mod.rs`)

### TestEnvironment

Isolated test environment with temporary directories and environment variables:

```rust
let env = TestEnvironment::new();
let cmd = env.icarus_cmd(); // Pre-configured with test environment
```

### ProjectTester

Project creation and validation helper:

```rust
let project = ProjectTester::new(&env, "my-project", "basic");
project.create(&env).assert().success();
project.validate_all().unwrap();
```

### McpTester

MCP operations helper:

```rust
let mcp = McpTester::new();
mcp.create_mock_config(servers);
mcp.list_servers().assert().success();
```

### PerformanceTester

Performance measurement utilities:

```rust
let perf = PerformanceTester::start();
// ... operation ...
perf.assert_within(Duration::from_secs(5), "project creation");
```

## Running Tests

### Using Makefile (Recommended)

```bash
# Run all tests
make test-all

# Run specific test categories
make test-unit          # Unit tests only
make test-integration   # Integration tests only
make test-e2e          # End-to-end workflow tests
make test-mcp          # MCP integration tests
make test-templates    # Template validation tests

# Development
make test-quick        # Quick subset for development
make dev-watch         # Watch mode for continuous testing

# CI/CD
make ci-test          # Full CI test suite
make coverage         # Generate coverage report
```

### Using Cargo Directly

```bash
# All tests
cargo test

# Specific test files
cargo test e2e_workflow_tests
cargo test mcp_integration_tests
cargo test template_validation_tests

# Specific test functions
cargo test test_complete_basic_workflow
cargo test test_mcp_integration_workflow
cargo test test_all_templates_creation

# With logging
RUST_LOG=debug cargo test test_name
```

## CI Integration

### GitHub Actions Example

```yaml
- name: Install dependencies
  run: make ci-install-deps

- name: Run test suite
  run: make ci-test

- name: Generate coverage
  run: make coverage-lcov

- name: Upload coverage
  uses: codecov/codecov-action@v3
  with:
    file: coverage.lcov
```

### Test Configuration

Tests are designed for CI compatibility:

- **Timeouts**: Generous timeouts for slower CI environments
- **Isolation**: Each test uses isolated temporary directories
- **Dependencies**: Graceful handling of missing tools (cargo, dfx, git)
- **Parallelization**: Tests use `serial_test` for resource conflicts
- **Cleanup**: Automatic cleanup of temporary resources

## Performance Considerations

### Test Execution Time

- **Unit tests**: < 5 seconds
- **Integration tests**: < 30 seconds  
- **E2E tests**: < 2 minutes (depends on cargo/dfx availability)
- **Full suite**: < 5 minutes

### Resource Usage

- **Memory**: Each test uses isolated temporary directories
- **Disk**: Temporary projects cleaned up automatically
- **Network**: No external network dependencies (except tool detection)

## Test Coverage

Current test coverage includes:

- ✅ **Project Creation**: All templates and configurations
- ✅ **Build Process**: Debug/release builds, targets, features
- ✅ **MCP Integration**: Server management, client detection
- ✅ **Template System**: Structure, content, customization
- ✅ **Error Handling**: Invalid inputs, edge cases
- ✅ **CLI Interface**: All commands, flags, help text
- ✅ **Configuration**: Serialization, validation, migration

## Adding New Tests

### Test Structure

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use crate::test_utils::*;

#[test]
#[serial]  // Use for tests that might conflict
fn test_new_feature() {
    let env = TestEnvironment::new();
    
    // Test implementation
    env.icarus_cmd()
        .args(["new-command", "--flag", "value"])
        .assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}
```

### Best Practices

1. **Use test utilities** for common operations
2. **Add `#[serial]`** for tests that use global resources
3. **Test both success and failure paths**
4. **Use descriptive test names** that explain what's being tested
5. **Add timeout for long-running operations**
6. **Clean up resources** in test teardown
7. **Test CI compatibility** with tool availability checks

## Debugging Tests

### Common Issues

1. **Missing tools**: Tests gracefully skip when cargo/dfx unavailable
2. **Timeouts**: Increase `TEST_TIMEOUT` environment variable
3. **Permissions**: Ensure test runner has write access to temp directories
4. **Resource conflicts**: Use `#[serial]` for conflicting tests

### Debug Output

```bash
# Enable debug logging
RUST_LOG=debug cargo test test_name

# Run specific test with output
cargo test test_name -- --nocapture

# Show test execution time
cargo test -- --report-time
```

## Maintenance

### Regular Tasks

- **Update dependencies**: `make update-deps`
- **Security audit**: `make audit`
- **Coverage analysis**: `make coverage`
- **Performance benchmarks**: `make bench`

### Adding New Templates

When adding new templates:

1. Add template validation in `template_validation_tests.rs`
2. Update `test_all_templates_creation` test
3. Add template-specific structure validation
4. Test customization options

### Adding New Commands

When adding new CLI commands:

1. Add help text validation in `integration_tests.rs`
2. Add functionality tests in appropriate test file
3. Add error handling tests
4. Update global flag tests

This comprehensive test suite ensures the Icarus CLI works reliably across all supported workflows and environments.