# Migration Guide

This guide helps you migrate between different versions of the Icarus SDK.

## Migrating from 0.6.x to 0.9.0

Version 0.9.0 introduces a **modular architecture** with focused crates for better maintainability and development experience. This is a **non-breaking change** - existing applications continue to work without modification.

### What's New in 0.9.0

#### 🏗️ Modular Crate Architecture
The CDK is organized into focused crates:

- **`icarus`** - Main SDK with all features bundled
- **`icarus-core`** - Core types, traits, and MCP protocol implementation
- **`icarus-derive`** - Procedural macros (`#[tool]`, derive)
- **`icarus-canister`** - ICP integration and stable storage
- **`icarus-wasi`** - WASI polyfill, detection, and optimization
- **`icarus-cli`** - Command-line interface with MCP client integration

#### 🧪 Enhanced Testing
- 412+ comprehensive tests across all crates
- Comprehensive test coverage for MCP protocol and CLI functionality
- Improved reliability and error handling

### Migration Steps

1. **Update your `Cargo.toml`**:
   ```toml
   [dependencies]
   icarus = "0.9.0"  # No other changes needed
   ```

2. **Redeploy your canister**:
   ```bash
   icarus deploy
   ```

3. **Optional: Use modular dependencies** (for specialized use cases):
   ```toml
   [dependencies]
   icarus-bridge = "0.9.0"   # For custom bridge implementations
   icarus-dev = "0.9.0"      # For development tooling
   icarus-core = "0.9.0"     # For core functionality only
   ```

### No Breaking Changes

- **Existing APIs unchanged**: All your existing code continues to work
- **Same functionality**: All features remain available
- **Backward compatibility**: No migration of existing canisters needed

### Benefits of 0.9.0

- **Faster builds**: Modular compilation improves build times
- **Better development**: Enhanced file watching and dev tools
- **Improved reliability**: Comprehensive test coverage
- **Future flexibility**: Easier to extend and maintain

---

## Migrating from 0.5.x to 0.6.0 (Historical)

> **Note**: This is a historical migration guide for legacy versions.

### Overview
Version 0.6.0 was a **breaking change** release that updated all dependencies to their latest stable versions and removed all deprecated code for production readiness.

### Breaking Changes

#### Major Dependency Updates
- **ic-cdk**: 0.13 → 0.18 (latest stable)
- **ic-stable-structures**: 0.6 → 0.7 (breaking API changes)
- **ic-cdk-timers**: 0.9 → 0.12
- All other dependencies updated to latest major.minor versions

#### Deprecated Code Removal
All deprecated functions have been removed and replaced with modern equivalents:
- `ic_cdk::api::caller()` → `ic_cdk::api::msg_caller()`
- `ic_cdk::api::id()` → `ic_cdk::api::canister_self()`
- `ic_cdk::api::print()` → `ic_cdk::api::debug_print()`

#### HTTP API Modernization
The HTTP module has been completely refactored:
- New import path: `ic_cdk::management_canister`
- New types: `HttpRequestArgs`, `HttpRequestResult`
- Manual cycles calculation removed (now automatic)

### Migration Steps

#### 1. Update Dependencies
Update your `Cargo.toml`:

```toml
[dependencies]
icarus = "0.9.0"
ic-cdk = "0.18"
candid = "0.10"
serde = { version = "1.0", features = ["derive"] }
```

#### 2. Rebuild and Redeploy
```bash
# Clean build to ensure all dependencies are updated
cargo clean
icarus build
icarus deploy
```

#### 3. No Source Code Changes Required
All breaking changes are internal to the SDK. Your existing code should continue to work without modifications.

### Verification
After migration, verify everything works:
1. Build completes without warnings: `cargo build`
2. Tests pass: `cargo test`
3. Canister deploys successfully: `icarus deploy`
4. Bridge connects properly: `icarus bridge start --canister-id <id>`

## Migrating from 0.3.1 to 0.3.2

This is a patch release with CI/CD improvements and cleanup. No code changes are required.

### What Changed
- CI/CD pipeline optimized for 40-60% faster execution
- Deprecated `require_role` function removed (use `require_role_or_higher`)
- Redundant scripts and workflow files removed

No migration required for existing code.

## Migrating from 0.3.0 to 0.3.1

### Overview
Version 0.5.7 is a patch release that fixes critical issues with the v0.3.0 release.

### Bug Fixes
- **Fixed**: Coverage workflow failures - cleared LLVM_PROFILE environment variables in subprocess cargo commands
- **Fixed**: Macro compilation errors - updated all macro paths from `::icarus_canister::` to `::icarus::canister::`
- **Fixed**: WASM builds now work correctly when run under `cargo llvm-cov`

### No Breaking Changes
This is a patch release with no breaking changes. Simply update your dependency:

```toml
[dependencies]
icarus = "0.3.2"
```

## Migrating from 0.2.7 to 0.3.0

### Overview
Version 0.3.0 introduces significant improvements to package management and fixes critical issues with Claude Desktop integration.

### Breaking Changes

#### Import Path Change
The main breaking change is the simplified import path:

**Before (0.2.x):**
```rust
use icarus_canister::prelude::*;
```

**After (0.3.0):**
```rust
use icarus::prelude::*;
```

#### Simplified Dependencies
Projects no longer need to include `icarus-canister` separately:

**Before (0.2.x):**
```toml
[dependencies]
icarus = "0.2.7"
icarus-canister = "0.2.7"  # No longer needed!
```

**After (0.3.0):**
```toml
[dependencies]
icarus = "0.3.0"  # Includes everything via feature flags
```

### New Features

- **Feature flags**: The main `icarus` crate now uses feature flags for modular dependency management
- **Claude Desktop fix**: The CLI now uses full paths to resolve the executable location  
- **Cleaner project templates**: `icarus new` generates projects with the simplified import structure
- **Professional documentation**: Completely redesigned README with badges and better organization

### Migration Steps

1. Update your `Cargo.toml`:
   ```toml
   [dependencies]
   icarus = "0.3.0"
   # Remove icarus-canister line if present
   ```

2. Update your imports:
   ```rust
   // Change this:
   use icarus_canister::prelude::*;
   
   // To this:
   use icarus::prelude::*;
   ```

3. Rebuild your project:
   ```bash
   cargo clean
   cargo build --target wasm32-unknown-unknown --release
   ```

## Migrating from 0.2.5 to 0.3.0

### Overview
Version 0.5.7 is primarily a maintenance release with improved documentation and synchronized versioning across all workspace crates.

### Changes

#### Version Synchronization
All crates in the workspace now use the same version number (0.5.7):
- `icarus`
- `icarus-core`
- `icarus-derive`
- `icarus-canister`
- `icarus-cli`

**Action Required**: Update your `Cargo.toml` dependencies to use version 0.5.7:
```toml
[dependencies]
icarus = "0.3.0"
# Or if using individual crates:
icarus-canister = "0.3.0"
```

#### No Breaking Changes
This version contains no breaking API changes. Your existing code should work without modifications.

## Migrating from 0.2.4 to 0.2.5

### Security Updates
Version 0.2.5 includes critical security updates:
- Fixed slab vulnerability (RUSTSEC-2025-0047)
- Updated all dependencies to latest secure versions

**Action Required**: Run `cargo update` to get the latest dependency versions.

### Documentation Improvements
- Fixed doc test issues
- Improved README with better examples
- Removed non-existent marketplace URLs

## Migrating from Earlier Versions

### From 0.2.x to 0.2.4

#### CLI Integration
The Icarus CLI is now part of the main SDK repository. If you were using the CLI from a separate repository:

1. Uninstall the old CLI:
   ```bash
   cargo uninstall icarus-cli
   ```

2. Install the new integrated CLI:
   ```bash
   cargo install icarus-cli
   ```

#### License Change
The project now uses the Business Source License (BSL-1.1). Review the LICENSE file for details.

### From 0.1.x to 0.2.x

#### Major API Changes

1. **Macro Syntax Updates**
   The `#[icarus_module]` macro now requires explicit tool registration:
   ```rust
   // Old (0.1.x)
   #[icarus_tools]
   mod tools {
       pub fn my_tool() -> Result<String, String> { ... }
   }
   
   // New (0.2.x)
   #[icarus_module]
   mod tools {
       #[update]
       #[icarus_tool("Tool description")]
       pub fn my_tool() -> Result<String, String> { ... }
   }
   ```

2. **Storage API Changes**
   The stable storage API has been simplified:
   ```rust
   // Old (0.1.x)
   use icarus::storage::StableStorage;
   
   // New (0.2.x)
   use icarus_canister::prelude::*;
   
   stable_storage! {
       MEMORIES: StableBTreeMap<String, MemoryEntry, Memory> = memory_id!(0);
   }
   ```

3. **Error Handling**
   Error types are now more consistent across the SDK:
   ```rust
   // Old (0.1.x)
   use icarus::Error;
   
   // New (0.2.x)
   use icarus_core::error::IcarusError;
   ```

## Common Migration Issues

### Dependency Conflicts
If you encounter dependency conflicts after upgrading:

1. Clean your build cache:
   ```bash
   cargo clean
   ```

2. Update all dependencies:
   ```bash
   cargo update
   ```

3. Rebuild your project:
   ```bash
   cargo build --release
   ```

### WASM Optimization Issues
If you're having issues with WASM optimization:

1. Ensure you have the latest wasm-opt installed:
   ```bash
   npm install -g wasm-opt
   ```

2. For ic-wasm issues, reinstall:
   ```bash
   cargo install ic-wasm --force
   ```

### Candid Interface Changes
If your Candid interface needs updating:

1. Rebuild to regenerate the interface by deploying:
   ```bash
   icarus deploy --network local
   ```

2. The new interface will be in `src/<project-name>.did`

## Getting Help

If you encounter issues during migration:

1. Check the [CHANGELOG](../CHANGELOG.md) for detailed changes
2. Review the [examples](../examples/) for updated code patterns
3. Open an [issue](https://github.com/galenoshea/icarus-sdk/issues) for help

## Version Support Policy

- **Current Version (0.9.0)**: Full support with modular architecture
- **Previous Minor (0.5.8)**: Critical fixes only
- **Previous Minor (0.4.x)**: Security updates only
- **Older Versions**: No support, upgrade recommended

We recommend staying on the latest version for the best experience and security.