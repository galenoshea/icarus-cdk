# icarus-bridge

**MCP-to-ICP Bridge for the Icarus SDK**

This crate provides the bridge functionality that enables Model Context Protocol (MCP) servers to run on Internet Computer Protocol (ICP) canisters.

## Overview

The `icarus-bridge` crate handles the translation between MCP protocol messages and ICP canister calls, enabling AI clients like Claude Desktop to communicate with persistent tools running on the Internet Computer.

## Key Components

### 🌉 Bridge Server (`rmcp_server.rs`)
- Translates MCP protocol to canister calls
- Handles dynamic identity switching via dfx
- Manages session persistence and tool registration
- Provides real-time MCP server functionality

### 🔐 Authentication (`auth.rs`)
- Internet Identity integration
- Principal-based access control
- dfx identity management
- Secure canister authentication

### 🔌 Canister Client (`canister_client.rs`)
- IC agent management and connection pooling
- Canister communication and error handling
- Memory and storage utilities
- Network configuration management

### 🔄 Parameter Mapping (`param_mapper.rs`)
- JSON to Candid parameter translation
- Type detection and conversion
- Support for positional and record-style parameters
- Automatic fallback handling

### 🛠️ Builder (`builder.rs`)
- Fluent API for bridge configuration
- Network and authentication setup
- Timeout and concurrency management
- Development and production modes

## Usage

### Basic Bridge Setup

```rust
use icarus_bridge::prelude::*;

// Create a bridge to connect MCP clients to your canister
let bridge = BridgeBuilder::new()
    .canister_id("your-canister-id")
    .network("local") // or "ic" for mainnet
    .timeout(30)
    .max_concurrent_requests(10)
    .build()?;

// Start the bridge server
bridge.start().await?;
```

### Authentication Configuration

```rust
use icarus_bridge::auth::*;

// Check dfx availability
if is_dfx_available() {
    let identity = get_current_identity().await?;
    println!("Using identity: {}", identity);

    // Create authenticated agent
    let agent = create_authenticated_agent("local").await?;
}
```

### Canister Communication

```rust
use icarus_bridge::canister_client::*;

// Create canister client
let client = CanisterClient::new(
    "your-canister-id".to_string(),
    agent
)?;

// Call canister methods
let tools = client.list_tools().await?;
let result = client.call_tool("tool_name", args).await?;
```

## Features

- **Dynamic Identity**: Automatically detects and uses current dfx identity
- **Type Safety**: Full Rust type checking with Candid integration
- **Error Handling**: Comprehensive error types and recovery
- **Performance**: Connection pooling and concurrent request handling
- **Development**: Hot-reload support and local testing utilities

## Integration

This crate is automatically included when using the main `icarus` crate:

```toml
[dependencies]
icarus = "0.7.0"  # Includes icarus-bridge
```

Or use directly for specialized bridge applications:

```toml
[dependencies]
icarus-bridge = "0.7.0"
```

## Architecture

```
┌─────────────────┐    MCP Protocol    ┌─────────────────┐
│   AI Clients    │◄──────────────────►│  Bridge Server  │
│ (Claude, etc.)  │    JSON-RPC        │ (rmcp_server)   │
└─────────────────┘                    └─────────────────┘
                                                │
                                       Candid/HTTP
                                                │
                                                ▼
                                    ┌─────────────────┐
                                    │  ICP Canister   │
                                    │  (Your Tools)   │
                                    └─────────────────┘
```

## Development

```bash
# Run tests
cargo test

# Test with specific features
cargo test --features "dev"

# Build for production
cargo build --release
```

## Related Crates

- [`icarus`](../icarus/) - Main SDK with all features
- [`icarus-core`](../icarus-core/) - Core types and traits
- [`icarus-canister`](../icarus-canister/) - Canister-side functionality
- [`icarus-dev`](../icarus-dev/) - Development tools
- [`icarus-derive`](../icarus-derive/) - Proc macros
- [`icarus-mcp`](../icarus-mcp/) - MCP protocol implementation

## License

Licensed under the Business Source License 1.1 (BSL). See [LICENSE](../../LICENSE) for details.