# Icarus SDK

Build MCP (Model Context Protocol) servers that run as Internet Computer canisters.

## Overview

Icarus SDK enables developers to create persistent AI tools by combining:
- **MCP**: The Model Context Protocol for AI assistant tools
- **ICP**: The Internet Computer's blockchain-based compute platform

Write your MCP servers in Rust, deploy them to ICP, and they run forever with built-in persistence.

## Quick Start

```rust,ignore
use icarus_canister::prelude::*;
use candid::{CandidType, Deserialize};
use serde::Serialize;

// Define your data structures
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
pub struct MemoryEntry {
    id: String,
    content: String,
    created_at: u64,
}

// Use stable storage for persistence
stable_storage! {
    MEMORIES: StableBTreeMap<String, MemoryEntry, Memory> = memory_id!(0);
}

// Define your tools with automatic metadata generation
#[icarus_module]
mod tools {
    use super::*;
    
    #[update]
    #[icarus_tool("Store a new memory")]
    pub fn memorize(content: String) -> Result<String, String> {
        let id = generate_id();
        let memory = MemoryEntry {
            id: id.clone(),
            content,
            created_at: ic_cdk::api::time(),
        };
        MEMORIES.with(|m| m.borrow_mut().insert(id.clone(), memory));
        Ok(id)
    }
}

// Export the Candid interface
ic_cdk::export_candid!();
```

## Installation

### CLI Installation

Install the Icarus CLI to create and manage MCP server projects:

```bash
# Install the CLI
cargo install icarus-cli

# Verify installation
icarus --version
```

### SDK Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
icarus = "0.2.3"
# Or if you need individual crates:
# icarus-canister = "0.2.3"
# ic-cdk = "0.16"
# candid = "0.10"
# serde = { version = "1.0", features = ["derive"] }
```

## Project Structure

The Icarus project consists of the SDK and CLI:

### `icarus-core`
Core traits and types for building MCP servers on ICP:
- Protocol types
- Error handling
- Session management
- Tool and resource abstractions

### `icarus-derive`
Procedural macros that reduce boilerplate:
- `#[icarus_module]` - Generates tool metadata and exports
- `#[icarus_tool]` - Marks functions as MCP tools
- `#[derive(IcarusStorable)]` - Enables stable storage

### `icarus-canister`
ICP canister integration with stable memory:
- `stable_storage!` macro for declaring persistent data
- Memory management utilities
- State persistence helpers

### `icarus-cli`
Command-line tool for development and deployment:
- Project scaffolding with `icarus new`
- Build and optimization with `icarus build`
- Deployment to ICP with `icarus deploy`
- MCP-ICP bridge with `icarus bridge`

## Features

- 🔧 Simple Rust macros for MCP tools
- 💾 Automatic state persistence with stable structures
- 🌐 Global accessibility via ICP
- 🔒 Blockchain-grade security
- 🚀 Deploy once, run forever
- 🧪 PocketIC integration for testing
- 📦 Zero-copy stable memory operations

## How It Works

1. **Write Tools**: Use `#[icarus_tool]` to mark functions as MCP tools
2. **Generate Metadata**: The `#[icarus_module]` macro creates a `get_metadata()` query
3. **Deploy**: Use the Icarus CLI to deploy your canister
4. **Bridge**: The CLI's bridge translates between MCP and your canister

The generated canister is a standard ICP backend with no MCP awareness. All protocol translation happens in the bridge, keeping your code clean and testable.

## Requirements

- Rust 1.75+
- dfx (ICP SDK) - via Icarus CLI
- wasm32-unknown-unknown target

## Documentation

See the [docs/](docs/) folder for:
- Getting Started Guide
- API Reference
- Best Practices
- Example Projects

## Testing

The SDK includes a comprehensive test suite:

```bash
# Run all tests
cargo test

# Run with coverage
cargo install cargo-llvm-cov
cargo llvm-cov --html

# Install pre-commit hooks
./scripts/install-hooks.sh

# Run CI tests locally  
./scripts/test-ci.sh

# Create a new release
./scripts/release.sh patch
```

See [tests/README.md](tests/README.md) for more details.

## License

⚠️ **IMPORTANT**: This SDK is licensed under the Business Source License 1.1 (BSL-1.1), 
which is **NOT** an open source license.

### Version Notice
- ❌ Version 0.1.0 was published in error and has been yanked - DO NOT USE
- ✅ Version 0.2.0+ is the official release under BSL-1.1

### ✅ You CAN:
- Build and deploy MCP tools to Icarus Marketplace
- Modify the SDK for internal use
- Integrate with your applications that use Icarus
- Use for education and research
- Create commercial MCP tools using this SDK

### ❌ You CANNOT:
- Create competing MCP marketplaces
- Redistribute the SDK or derivatives
- Remove signature verification
- White-label or rebrand the SDK
- Reverse engineer for competition
- Offer SDK as a service
- Bypass telemetry or usage reporting

### 🔒 Security & Verification
This SDK includes:
- Cryptographic signature verification for authenticity
- Telemetry for service improvement and compliance
- Connection restrictions to official Icarus infrastructure

Tampering with these mechanisms violates the license and may result in legal action.

### 📅 Open Source Conversion
On **January 1, 2029**, this SDK automatically converts to Apache License 2.0.

### 📄 Legal
- See [LICENSE](./LICENSE) for complete BSL-1.1 terms
- See [NOTICE](./NOTICE) for important restrictions
- Commercial licenses available: licensing@icarus.dev
- Report violations: legal@icarus.dev

### ™️ Trademarks
"Icarus" and the Icarus logo are registered trademarks. Unauthorized use is prohibited.