# icarus-core

Core types and protocol implementation for the Icarus CDK (Canister Development Kit).

## Overview

`icarus-core` provides the foundational types, traits, and protocol implementations for building AI-powered Internet Computer canisters with the Model Context Protocol (MCP). This crate is the heart of the Icarus SDK, implementing type-safe wrappers, comprehensive error handling, and the MCP protocol specification.

## Features

- **Type Safety**: Newtype pattern for `ToolId`, `UserId`, `SessionId`, and `Timestamp` with validation
- **Error Handling**: Comprehensive error types using `thiserror` with rich context
- **MCP Protocol**: Full implementation of JSON-RPC 2.0 and MCP tool specification
- **Performance**: Zero-cost abstractions with `SmallVec` optimization for common cases
- **Validation**: Input validation and schema support for tool parameters
- **No `unsafe` Code**: Pure safe Rust for security and reliability

## Core Types

### Newtypes

```rust
use icarus_core::{ToolId, UserId, SessionId, Timestamp};

// Type-safe identifiers with validation
let tool_id = ToolId::new("calculator.add")?;
let user_id = UserId::new("rdmx6-jaaaa-aaaah-qcaiq-cai")?;
let session_id = SessionId::generate();
let timestamp = Timestamp::now();

// Implement AsRef<str> for idiomatic string access
let id_str: &str = tool_id.as_ref();
```

### Tools

```rust
use icarus_core::{Tool, ToolParameter, ToolSchema};

let tool = Tool::builder()
    .name("add")?
    .description("Adds two numbers together")
    .parameter(
        ToolParameter::builder()
            .name("a")
            .description("First number")
            .schema(ToolSchema::number())
            .required(true)
            .build()?
    )
    .parameter(
        ToolParameter::builder()
            .name("b")
            .description("Second number")
            .schema(ToolSchema::number())
            .required(true)
            .build()?
    )
    .build()?;
```

### Error Handling

```rust
use icarus_core::{IcarusError, ResultExt};

fn process_tool(tool_id: &str) -> Result<String, IcarusError> {
    let id = ToolId::new(tool_id)
        .with_context(|| format!("Invalid tool ID: {}", tool_id))?;

    // ... process tool

    Ok("Success".to_string())
}
```

## Architecture

The crate is organized into focused modules:

- `error`: Comprehensive error types with severity levels and context
- `newtypes`: Type-safe wrappers with validation
- `protocol`: MCP protocol types and JSON-RPC implementation
- `tool`: Tool definitions, parameters, and schema validation
- `version`: Version information and compatibility

## Performance

- **SmallVec Optimization**: Tool parameters use `SmallVec<[T; 4]>` for stack allocation of common cases
- **Zero-Copy**: Where possible, uses borrowing instead of cloning
- **Compile-Time Validation**: `const fn` constructors enable compile-time checking
- **Inline Functions**: Critical paths marked `#[inline]` for optimal performance

## Best Practices

This crate follows the patterns documented in `rust_best_practices.md`:

1. **Newtype Pattern**: Type safety for domain concepts (Section 1)
2. **Builder Pattern**: Ergonomic construction of complex types (Section 2)
3. **Error Handling**: Rich error types with context (Section 3)
4. **Iterator Patterns**: Functional composition over loops (Section 5)
5. **Zero-Cost Abstractions**: Performance without runtime overhead (Section 8)

## Compatibility

- **Rust Version**: 1.83.0 or later
- **no_std**: Not currently supported (requires `std` for error handling)
- **WASM**: Full support for `wasm32-unknown-unknown` target
- **Internet Computer**: Native support with `ic-canister` feature

## License

See the LICENSE file in the repository root.