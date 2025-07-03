# Icarus SDK

Build MCP (Model Context Protocol) servers that run as Internet Computer canisters.

## Overview

Icarus SDK enables developers to create persistent AI tools by combining:
- **MCP**: The Model Context Protocol for AI assistant tools
- **ICP**: The Internet Computer's blockchain-based compute platform

Write your MCP servers in Rust, deploy them to ICP, and they run forever with built-in persistence.

## Quick Start

```rust
use icarus::prelude::*;

#[derive(IcarusTool)]
#[icarus_tool(name = "memory", description = "Remember facts")]
struct MemoryTool;

#[icarus_server(name = "memory-server", version = "1.0.0")]
pub struct MemoryServer {
    facts: HashMap<String, String>,
}

// Generate candid interface
icarus::export_candid!();
```

## Project Structure

- `icarus-core` - Core traits and types
- `icarus-derive` - Procedural macros
- `icarus-canister` - ICP canister integration
- `icarus-bridge` - MCP-ICP protocol bridge
- `icarus-test` - Testing utilities
- `icarus` - Main SDK crate

## Features

- 🔧 Simple Rust macros for MCP tools
- 💾 Automatic state persistence
- 🌐 Global accessibility via ICP
- 🔒 Blockchain-grade security
- 🚀 Deploy once, run forever
- 🧪 Local testing utilities

## Requirements

- Rust 1.75+
- dfx (ICP SDK)
- Internet Computer local replica (for development)

## License

Apache 2.0