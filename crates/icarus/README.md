# Icarus CDK

<div align="center">

**Build MCP servers on Internet Computer canisters with ease.**

[![Crate](https://img.shields.io/crates/v/icarus.svg)](https://crates.io/crates/icarus)
[![Documentation](https://docs.rs/icarus/badge.svg)](https://docs.rs/icarus)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

---

## Overview

Icarus CDK is a modern Rust framework for creating MCP (Model Context Protocol) servers that run on Internet Computer canisters. It provides a declarative API with automatic tool discovery, type-safe execution, and comprehensive error handling.

### Key Features

- **🔧 Declarative API**: Use `#[tool]` to automatically expose Rust functions as MCP tools
- **🚀 Zero-overhead**: Compile-time tool registration with <10ms execution times
- **🛡️ Type Safety**: Comprehensive type system with domain newtypes and validation
- **⚡ Performance**: Zero-copy patterns, memory efficiency, and SIMD optimizations
- **🌐 IC Native**: Built specifically for Internet Computer canister architecture
- **📊 Observability**: Built-in metrics, tracing, and performance monitoring
- **🔄 Async Support**: Optional async execution for I/O-bound tools
- **🧪 Well Tested**: Comprehensive test suite with property-based testing

---

## Quick Start

### Installation

Add Icarus to your `Cargo.toml`:

```toml
[dependencies]
icarus = "1.0.0"
```

### Create Your First MCP Server

```rust
use icarus::prelude::*;

/// Add two numbers together
#[tool]
fn add(a: f64, b: f64) -> f64 {
    a + b
}

/// Calculate the square of a number
#[tool]
fn square(x: f64) -> f64 {
    x * x
}

// Generate the MCP server
mcp! {
    name = "calculator",
    description = "A simple calculator service",
    version = "1.0.0"
}
```

### Deploy to Internet Computer

```bash
# Initialize dfx project (if not already done)
dfx new my_mcp_server
cd my_mcp_server

# Build and deploy
dfx build
dfx deploy

# Test your tools
dfx canister call my_mcp_server mcp_list_tools
```

---

## Architecture

Icarus CDK follows a clean layered architecture:

```text
┌─────────────────────────────────────────────────────────────┐
│                     Your Application                        │
│                   #[tool] functions                         │
├─────────────────────────────────────────────────────────────┤
│                    icarus (facade)                          │
│              Public API and re-exports                      │
├─────────────────────────────────────────────────────────────┤
│   icarus-macros     │    icarus-runtime    │  icarus-core   │
│  #[tool], mcp!{}    │   Tool execution     │  Core types    │
│  Proc macros        │   Registry, cache    │  Protocols     │
├─────────────────────────────────────────────────────────────┤
│              Internet Computer (IC)                         │
│            Canister Runtime Environment                     │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Concepts

### Tools

Tools are Rust functions decorated with `#[tool]` that become available as MCP tools:

```rust
use icarus::prelude::*;

/// Greet a user with optional title
#[tool]
fn greet(name: String, title: Option<String>) -> String {
    match title {
        Some(t) => format!("Hello, {} {}!", t, name),
        None => format!("Hello, {}!", name),
    }
}
```

### Type Safety

Icarus uses domain-specific newtypes for enhanced type safety:

```rust
use icarus::prelude::*;

// These are all distinct types that prevent mixing up IDs
let tool_id = ToolId::new("calculator_add")?;
let user_id = UserId::new("user_12345")?;
let session_id = SessionId::new("session_abcdef")?;
```

### Error Handling

Comprehensive error handling with rich context:

```rust
use icarus::prelude::*;

#[tool]
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}
```

---

## Examples

### Async Tools

```rust
use icarus::prelude::*;

#[tool]
async fn fetch_data(url: String) -> Result<String, String> {
    // Async operations are fully supported
    match reqwest::get(&url).await {
        Ok(response) => response.text().await.map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}
```

### Stateful Tools

```rust
use icarus::prelude::*;
use std::collections::HashMap;

thread_local! {
    static COUNTERS: std::cell::RefCell<HashMap<String, u64>> =
        std::cell::RefCell::new(HashMap::new());
}

#[tool]
fn increment_counter(session: String) -> u64 {
    COUNTERS.with(|counters| {
        let mut map = counters.borrow_mut();
        let counter = map.entry(session).or_insert(0);
        *counter += 1;
        *counter
    })
}
```

---

## Feature Flags

Icarus supports the following feature flags:

| Feature | Default | Description |
|---------|---------|-------------|
| `async` | ✅ Yes | Enables async tool execution |

### Disabling Default Features

```toml
[dependencies]
icarus = { version = "0.9", default-features = false }
```

---

## Performance

Icarus is designed for high performance:

- **Sub-10ms execution**: Most tools execute in under 10 milliseconds
- **Zero-copy patterns**: Using `Cow<str>` and references where possible
- **Memory efficiency**: `SmallVec` for small collections, pre-allocation
- **Compile-time optimization**: Tool registration happens at compile time

### Performance Tuning

For production deployments, use these optimizations:

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
```

---

## Best Practices

1. **Use descriptive tool names**: Tool names become part of the MCP API
2. **Include documentation**: Doc comments become tool descriptions
3. **Handle errors gracefully**: Use `Result` types for fallible operations
4. **Keep tools focused**: Each tool should do one thing well
5. **Validate inputs**: Use newtypes and validation for safety
6. **Test thoroughly**: Write unit tests for all tool functions

---

## Monitoring and Observability

Track execution metrics for your tools:

```rust
use icarus::prelude::*;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let mut executor = ToolExecutor::new().with_cache();

// Create a tool call
let tool_id = ToolId::new("my_tool")?;
let tool_call = ToolCall::new(tool_id);

// Execute tools and get metrics
let result = executor.execute(tool_call).await?;
let metrics = executor.metrics();

println!("Success rate: {:.2}%", metrics.success_rate());
println!("Average execution time: {:.2}ms", metrics.avg_execution_time_ms);
# Ok(())
# }
```

---

## Documentation

- [API Documentation](https://docs.rs/icarus)
- [Internet Computer Documentation](https://internetcomputer.org/docs)
- [MCP Protocol Specification](https://modelcontextprotocol.io)

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

---

## Acknowledgments

Built with ❤️ for the Internet Computer ecosystem.