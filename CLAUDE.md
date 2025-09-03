# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Icarus SDK enables developers to create MCP (Model Context Protocol) servers that run as Internet Computer Protocol (ICP) canisters, providing persistent AI tools with blockchain-grade security.

**Key License Restrictions (BSL-1.1)**:
- Cannot create competing MCP marketplaces
- Cannot redistribute SDK or derivatives
- Automatic conversion to Apache 2.0 on January 1, 2029
- Version 0.1.0 is yanked - only use 0.2.0+

## Development Commands

### Building and Testing
```bash
# Run unit and integration tests
make test
cargo test --all --lib --bins

# Run E2E tests (builds CLI first)
make test-e2e
cd cli && cargo test --test '*' -- --test-threads=1

# Run all tests
make test-all

# Build all crates
make build
cargo build --all

# Build specific targets
cargo build --package icarus-cli --bin icarus --release
cargo build --target wasm32-unknown-unknown --release

# Run CI checks locally before pushing
make ci
./scripts/test-ci.sh

# Check code quality
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

# Check version consistency
./scripts/check-versions.sh

# Deep clean all artifacts
make deep-clean
./scripts/clean.sh
```

### Release Process
```bash
# Create releases (uses cargo-release)
make release-patch  # 0.2.6 → 0.2.7
make release-minor  # 0.2.x → 0.3.0
make release-major  # 0.x.x → 1.0.0

# Or use script directly
./scripts/release.sh patch|minor|major

# Release process automatically:
# 1. Checks version consistency
# 2. Runs tests and clippy
# 3. Updates all version references via release.toml
# 4. Creates commit and tag
# 5. Triggers GitHub Actions on tag push
```

### CLI Development
```bash
# Build CLI
cd cli
cargo build --release

# Run CLI commands from source
cargo run -- new my-project
cargo run -- build
cargo run -- deploy --network local

# Install CLI locally
cargo install --path cli

# CLI bridge operations
icarus bridge start --canister-id <id>
icarus bridge status
icarus bridge stop
```

## Architecture

### Workspace Structure
```
icarus-sdk/
├── crates/
│   ├── icarus-core/      # Core protocol types, traits, and session management
│   ├── icarus-derive/     # Proc macros (#[icarus_module], #[icarus_tool])
│   └── icarus-canister/   # ICP integration, stable memory, storage macros
├── cli/                   # Command-line tool for project management
├── examples/              # Example MCP servers
└── scripts/               # Development and release automation
```

### Key Architectural Components

#### 1. **Macro System** (`icarus-derive`)
- `#[icarus_module]`: Generates MCP metadata and exports for canister modules
- `#[icarus_tool("description")]`: Marks functions as MCP tools with automatic metadata
- `#[derive(IcarusStorable)]`: Enables types for stable storage

#### 2. **Stable Storage** (`icarus-canister`)
- `stable_storage!` macro declares persistent data structures
- Uses IC stable memory for data persistence across upgrades
- Memory IDs allocate separate memory regions (0-254)
- Supports StableBTreeMap, StableVec, StableCell

#### 3. **Bridge Architecture** (`cli/src/bridge/`)
- `rmcp_server.rs`: Translates between MCP protocol and canister calls
- `canister_client.rs`: Handles IC agent and canister communication
- `auth.rs`: Manages IC identity and authentication
- Bridge runs as subprocess that Claude Desktop connects to

#### 4. **Tool Registration** (`icarus-core`)
- Tools self-register via `linkme` crate sections
- `TOOL_REGISTRY` collects all tools at compile time
- `get_metadata()` query endpoint exposes tools to MCP clients

#### 5. **Session Management**
- Sessions stored in canister stable memory
- Each session has unique ID and persistence
- Session cleanup on disconnect

### Testing Strategy

**Progressive Testing Levels**:
1. **Unit Tests**: No blockchain, test pure logic
2. **Canister Tests**: Local dfx, test canister behavior
3. **MCP Protocol Tests**: Test protocol compliance
4. **Integration Tests**: Full E2E with bridge

**E2E Test Helpers** (`cli/tests/e2e/helpers/`):
- `CliRunner`: Executes CLI commands for testing
- `TestProject`: Creates temporary test projects
- Assertion helpers for output validation

### Version Management

**Automated via `release.toml`**:
- Single version in workspace root `Cargo.toml`
- All crates use `version.workspace = true`
- Documentation versions auto-update on release
- CI validates version consistency

### CI/CD Pipeline

**GitHub Actions Workflows**:
- `ci.yml`: Tests, clippy, formatting, version check on every push
- `release.yml`: Publishes to crates.io on version tags (v0.2.6)
- `coverage.yml`: Code coverage reporting

**Pre-commit Hooks** (via `scripts/install-hooks.sh`):
- Format check
- Clippy warnings as errors
- Test execution

### Important Files and Patterns

**Workspace Configuration**:
- All dependencies defined in workspace root
- Shared profile settings for optimization
- Workspace-wide version management

**Error Handling**:
- Use `thiserror` for error types
- Return `Result<T, String>` in canister methods
- Bridge converts errors to MCP error responses

**Async Patterns**:
- CLI uses `tokio` for async runtime
- Canisters use `ic_cdk::spawn` for async operations
- Bridge handles async canister calls

**Security Considerations**:
- Authentication via IC identity
- Whitelist-based access control
- Input validation in canister methods

### Common Pitfalls to Avoid

1. **Version Mismatches**: Always run `./scripts/check-versions.sh`
2. **Unused Imports**: Will fail CI - run clippy before committing
3. **Direct WASM Path**: Use validate command's fallback logic for WASM discovery
4. **Profile in Sub-crates**: Profiles must be in workspace root only
5. **Test Isolation**: E2E tests must use `--test-threads=1`

### MCP-ICP Bridge Protocol Flow

1. Claude Desktop connects to bridge subprocess
2. Bridge translates MCP requests to canister calls
3. Canister executes tool functions
4. Results returned through bridge to Claude
5. State persists in stable memory

This architecture enables MCP servers to run permanently on ICP with built-in persistence, while maintaining clean separation between protocol translation (bridge) and business logic (canister).