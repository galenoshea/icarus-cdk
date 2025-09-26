# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Core Commands

### Development & Testing
```bash
# Build everything
cargo build --all
make build

# Run tests (multiple levels available)
make test           # Unit and integration tests only (fastest)
make test-quick     # Unit tests only (quickest feedback)
make test-e2e       # End-to-end CLI tests (local only)
make test-all       # All tests (unit + integration + E2E)
make test-pre-push  # Full pre-push hook validation

# Development workflow
make ci             # Run CI simulation locally
make coverage       # Generate coverage reports
```

### CLI Development & Testing
```bash
# Build CLI binary for testing
cargo build --package icarus-cli --bin icarus --release

# Test single CLI command
cargo test --package icarus-cli --test test_new_command --release

# Run all CLI integration tests
cd crates/icarus-cli && cargo test --test '*' --release
```

### Code Quality
```bash
# Format and lint
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings

# Install git hooks for automatic quality checks
make install-hooks
```

## Architecture Overview

### Modular Crate Structure
The Icarus SDK uses a modular architecture with focused crates:

- **`icarus`** - Main SDK facade, re-exports all functionality
- **`icarus-core`** - Core MCP protocol traits, types, and abstractions
- **`icarus-derive`** - Procedural macros (`#[tool]`, `#[auth]`, `#[mcp]`, `#[wasi]`)
- **`icarus-canister`** - ICP canister integration, stable storage, authentication
- **`icarus-wasi`** - WASI polyfill detection and optimization
- **`icarus-cli`** - Command-line interface with MCP client management

### Key Design Patterns

**MCP Tool Architecture**: Tools are defined with `#[icarus::tool]` macro and automatically generate MCP metadata. The system supports:
- Authentication levels: public, user, admin
- Automatic parameter translation between JSON (MCP) and Candid (ICP)
- Dual protocol support (both MCP and native ICP calls)

**Authentication System**: Three-tier authentication using Internet Computer principals:
- **Public**: No authentication required
- **User**: Must be authenticated via Internet Identity
- **Admin**: Must be the canister owner/controller

**Storage Layer**: Stable memory integration for persistent data:
- Uses `IcarusStorable` trait for automatic serialization
- Stable structures for high-performance persistence
- Memory statistics and optimization

### Important Code Conventions

**Macros**: The system relies heavily on procedural macros:
- `icarus::auth!()` - Generates authentication management functions
- `icarus::mcp!()` - Generates MCP tool discovery endpoints
- `ic_cdk::export_candid!()` - Exports Candid interface for tools

**Feature Flags**: Core functionality is feature-gated:
- `canister` - ICP canister functionality (default)
- `core` - Core MCP types only
- `wasi` - WASI support detection
- `mcp` - MCP protocol implementation

**Error Handling**: Consistent error types:
- `IcarusError` - Main error type with conversion traits
- `ToolError` - Tool-specific errors
- `Result<T, E>` aliases for common patterns

## Development Workflow

### Creating New Features
1. Add functionality to appropriate crate (`icarus-core` for protocol, `icarus-canister` for ICP)
2. Add tests in the same crate (`tests/` or inline `#[test]`)
3. Update exports in main `icarus` crate if needed
4. Run `make test-pre-push` to validate all changes

### Testing Strategy
- **Unit tests**: In each crate's `tests/` directory or inline
- **Integration tests**: Test interactions between crates
- **E2E tests**: CLI integration tests with real canister deployment
- **Doc tests**: Examples in documentation are tested

### CLI Command Structure
CLI commands are organized in `crates/icarus-cli/src/commands/`:
- `new.rs` - Project creation with template generation
- `build.rs` - Build commands with WASM optimization
- `deploy.rs` - ICP canister deployment
- `mcp/` - MCP client integration and management

### Release Process
Version management uses workspace inheritance:
- All crates share version from workspace `Cargo.toml`
- Use `make release-patch|minor|major` for releases
- CI/CD handles crate publishing automatically

## Common Patterns

### Tool Definition
```rust
#[ic_cdk::update]
#[icarus::tool("Tool description", auth = "user")]
pub async fn my_tool(param: String) -> Result<String, String> {
    Ok(format!("Processed: {}", param))
}
```

### Storage Usage
```rust
#[derive(IcarusStorable)]
struct MyData {
    field: String,
}

// Automatic stable memory persistence
```

### Error Handling
```rust
use icarus::prelude::*;

fn my_function() -> Result<String> {
    // IcarusError with automatic conversions
}
```

## Testing Notes

- E2E tests require building the CLI binary first: `cargo build --package icarus-cli --bin icarus --release`
- E2E tests are excluded from CI for performance but included in pre-push hooks
- Use `SKIP_E2E=1 git push` for emergency bypass of E2E tests
- All tests use serial execution to avoid conflicts with shared resources