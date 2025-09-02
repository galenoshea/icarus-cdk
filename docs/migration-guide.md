# Migration Guide

This guide helps you migrate between different versions of the Icarus SDK.

## Migrating from 0.2.5 to 0.2.6

### Overview
Version 0.2.6 is primarily a maintenance release with improved documentation and synchronized versioning across all workspace crates.

### Changes

#### Version Synchronization
All crates in the workspace now use the same version number (0.2.6):
- `icarus`
- `icarus-core`
- `icarus-derive`
- `icarus-canister`
- `icarus-cli`

**Action Required**: Update your `Cargo.toml` dependencies to use version 0.2.6:
```toml
[dependencies]
icarus = "0.2.6"
# Or if using individual crates:
icarus-canister = "0.2.6"
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

1. Rebuild to regenerate the interface:
   ```bash
   icarus build
   ```

2. The new interface will be in `src/<project-name>.did`

## Getting Help

If you encounter issues during migration:

1. Check the [CHANGELOG](../CHANGELOG.md) for detailed changes
2. Review the [examples](../examples/) for updated code patterns
3. Open an [issue](https://github.com/galenoshea/icarus-sdk/issues) for help

## Version Support Policy

- **Current Version (0.2.6)**: Full support
- **Previous Minor (0.2.5)**: Security updates only
- **Older Versions**: No support, upgrade recommended

We recommend staying on the latest version for the best experience and security.