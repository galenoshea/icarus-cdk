# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Icarus SDK enables developers to create MCP (Model Context Protocol) servers that run as Internet Computer Protocol (ICP) canisters, providing persistent AI tools with blockchain-grade security.

**Key License Restrictions (BSL-1.1)**:
- Cannot create competing MCP marketplaces
- Cannot redistribute SDK or derivatives
- Automatic conversion to Apache 2.0 on January 1, 2029
- Version 0.1.0 is yanked - only use 0.2.0+

## Git Commit Guidelines

**IMPORTANT**: Claude must NEVER be listed as a commit author or co-author.
- Always use the user's identity for all commits
- Claude provides code changes but does not author commits
- When creating commits, use only the user's name and email
- Never include "Co-Authored-By: Claude" or similar attribution

## Development Commands

### Building and Testing
```bash
# Run unit and integration tests
make test
cargo test --all --lib --bins

# Run E2E tests locally (not in CI)
make test-e2e
cd cli && cargo test --test '*' --release

# Run all tests (including E2E)
make test-all

# Run pre-push test suite (what runs before git push)
make test-pre-push

# Build all crates
make build
cargo build --all

# Build specific targets
cargo build --package icarus-cli --bin icarus --release
cargo build --target wasm32-unknown-unknown --release

# Run CI checks locally before pushing
make ci
./scripts/ci.sh

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
│   ├── icarus-canister/   # ICP integration, stable memory, storage macros
│   ├── icarus-mcp/        # MCP protocol implementation
│   ├── icarus-bridge/     # MCP-to-ICP bridge for canister communication
│   └── icarus-dev/        # Development tools (watch, status, monitoring)
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

#### 3. **Bridge Architecture** (`icarus-bridge`)
- `rmcp_server.rs`: Translates between MCP protocol and canister calls with dynamic identity switching
- `canister_client.rs`: Handles IC agent and canister communication
- `auth.rs`: Simplified authentication helpers with dfx identity detection
- Bridge runs as subprocess that Claude Desktop connects to
- **Dynamic Identity**: Bridge checks current dfx identity before each canister call
- **No Session Binding**: Identity can be switched without restarting bridge

#### 4. **Development Tools** (`icarus-dev`)
- `watch.rs`: File watching with intelligent debouncing and auto-rebuild
- `status.rs`: Development environment status checking and health monitoring
- `init.rs`: Interactive development environment setup and configuration
- `start.rs`: Development server startup with hot-reload capabilities
- `reset.rs`: Environment cleanup and reset utilities

#### 5. **Tool Registration** (`icarus-core`)
- Tools self-register via `linkme` crate sections
- `TOOL_REGISTRY` collects all tools at compile time
- `list_tools()` query endpoint exposes tools to MCP clients

#### 6. **Session Management**
- Sessions stored in canister stable memory
- Each session has unique ID and persistence
- Session cleanup on disconnect

### Testing Strategy

**Local vs CI Testing**:
- **Local (pre-push hooks)**: All tests including E2E run automatically before push
- **CI Pipeline**: Unit, integration, and doc tests only (E2E excluded for performance)
- **Emergency bypass**: Use `SKIP_E2E=1 git push` to skip E2E tests in emergencies
- **Release process**: Full test suite including E2E runs locally before release

**Progressive Testing Levels**:
1. **Unit Tests**: No blockchain, test pure logic
2. **Canister Tests**: Local dfx, test canister behavior
3. **MCP Protocol Tests**: Test protocol compliance
4. **Integration Tests**: Full E2E with bridge (local only)

**E2E Test Helpers** (`cli/tests/common/`):
- `CliRunner`: Executes CLI commands for testing
- `TestProject`: Creates temporary test projects
- `SharedTestProject`: Reusable test project for faster E2E tests
- `PocketIC`: Local ICP test environment for auth testing
- Assertion helpers for output validation

### Version Management

**Automated via `release.toml`**:
- Single version in workspace root `Cargo.toml`
- All crates use `version.workspace = true`
- Documentation versions auto-update on release
- CI validates version consistency

### CI/CD Pipeline

**GitHub Actions Workflows**:
- `ci.yml`: Unit/integration tests, clippy, formatting, version check (E2E tests excluded)
- `release.yml`: Publishes to crates.io on version tags
- `coverage.yml`: Code coverage reporting

**Git Hooks** (via `scripts/install-hooks.sh`):
- **Pre-commit**: Format check, clippy warnings as errors
- **Pre-push**: Full test suite including E2E tests
  - Version consistency check
  - Build with warnings as errors
  - Format and clippy checks
  - All unit, integration, and E2E tests
  - Use `SKIP_E2E=1 git push` to bypass E2E tests in emergencies

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
5. **Test Isolation**: E2E tests run with proper isolation in pre-push hooks

### MCP-ICP Bridge Protocol Flow

1. Claude Desktop connects to bridge subprocess
2. Bridge translates MCP requests to canister calls
3. Canister executes tool functions
4. Results returned through bridge to Claude
5. State persists in stable memory

This architecture enables MCP servers to run permanently on ICP with built-in persistence, while maintaining clean separation between protocol translation (bridge) and business logic (canister).

### Parameter Design Strategy

**Use the hybrid approach for function parameters**:

#### Simple Functions (1-2 parameters)
Use positional parameters for straightforward operations:
```rust
#[icarus_tool("Get a specific item")]
pub fn get_item(id: String) -> Result<Item, String>

#[icarus_tool("Delete an item")]
pub fn delete_item(id: String, confirm: bool) -> Result<String, String>
```

#### Complex Functions (3+ parameters)
Use args records for better self-documentation and AI understanding:
```rust
#[derive(CandidType, Deserialize)]
pub struct CreateItemArgs {
    name: String,              // Clear field names help Claude
    description: String,        // Self-documenting in Candid UI
    tags: Vec<String>,
    metadata: Option<HashMap<String, String>>,
    expires_at: Option<u64>,
}

#[icarus_tool("Create a new item with metadata")]
pub fn create_item(args: CreateItemArgs) -> Result<String, String>
```

**Why this matters for Claude**:
- Args records maintain the same structure from MCP JSON through to Candid
- Field names provide context even when Candid UI lacks documentation
- Claude can construct named field objects more reliably than positional parameters
- Aligns with ICP ecosystem patterns (NNS, OpenChat, Internet Identity)

**Naming Convention**:
- Always use `Args` suffix for record types: `CreateUserArgs`, `QueryArgs`, `UpdateConfigArgs`
- Use descriptive field names that match the MCP parameter names
- Document complex types with doc comments for better understanding

### Coverage and Testing

**Two-Phase Coverage Approach**:
The project uses a specialized coverage system to handle the conflict between LLVM coverage instrumentation and E2E tests that compile WASM:

```bash
# Comprehensive coverage (recommended)
make coverage           # Runs ./scripts/coverage.sh

# Individual phases
make coverage-unit      # Unit/integration tests with coverage
make coverage-e2e       # E2E tests without coverage instrumentation
```

**Coverage Script** (`./scripts/coverage.sh`):
- **Phase 1**: Runs unit and integration tests with LLVM coverage
- **Phase 2**: Runs E2E tests without coverage to avoid profiler_builtins conflicts
- Generates HTML reports and LCOV data
- Achieves ~80% coverage from unit/integration tests

**Running Single Tests**:
```bash
# Run specific unit test
cargo test --package icarus-core test_session_management

# Run specific E2E test
cd cli && cargo test --test test_authentication test_owner_initialization --release

# Run test with output
cargo test test_name -- --nocapture
```