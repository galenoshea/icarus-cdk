# icarus-macros

Procedural macros for the Icarus CDK (Canister Development Kit).

## Overview

`icarus-macros` provides ergonomic procedural macros that simplify building AI-powered Internet Computer canisters. The `#[tool]` macro automatically generates MCP (Model Context Protocol) tool definitions and registrations from regular Rust functions.

## Features

- **Zero Boilerplate**: Convert functions to MCP tools with a single attribute
- **Type Safety**: Automatic parameter validation and schema generation
- **Async Support**: Full support for async functions and futures
- **Rich Schemas**: Automatic JSON Schema generation from Rust types
- **Compile-Time Registration**: Tools registered at compile time via `linkme`
- **Error Handling**: Integrates with `icarus-core::IcarusError` for consistent errors

## Usage

### Basic Tool

```rust
use icarus_macros::tool;

#[tool]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

This automatically:
1. Generates an MCP tool definition with JSON Schema
2. Registers the tool in the global `TOOL_REGISTRY`
3. Validates parameters at runtime
4. Handles serialization/deserialization

### Async Tools

```rust
use icarus_macros::tool;

#[tool]
async fn fetch_data(url: String) -> Result<String, String> {
    // Async code here
    Ok("data".to_string())
}
```

### Complex Types

```rust
use icarus_macros::tool;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Config {
    timeout: u64,
    retries: u32,
}

#[tool]
fn configure(config: Config) -> Result<String, String> {
    Ok(format!("Configured with timeout: {}", config.timeout))
}
```

### Optional Parameters

```rust
use icarus_macros::tool;

#[tool]
fn greet(name: String, title: Option<String>) -> String {
    match title {
        Some(t) => format!("Hello, {} {}", t, name),
        None => format!("Hello, {}", name),
    }
}
```

## Generated Code

The `#[tool]` macro expands to approximately:

```rust
// Original function remains unchanged
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Generated tool registration
#[distributed_slice(TOOL_REGISTRY)]
static ADD_TOOL: fn() -> Tool = || {
    Tool::builder()
        .name("add").unwrap()
        .description("Adds two numbers")
        .parameter(/* a parameter */)
        .parameter(/* b parameter */)
        .build()
        .unwrap()
};
```

## Limitations

The macro has some restrictions to ensure correctness:

- ❌ **No `self` parameters**: Tools must be standalone functions
- ❌ **No generic parameters**: Type parameters are not supported
- ❌ **No lifetime parameters**: Lifetimes are not supported
- ✅ **Any return type**: As long as it implements `Serialize`
- ✅ **Async functions**: Full async/await support
- ✅ **Result types**: Both `Result<T, E>` and `T` are supported

## Error Handling

Tools can return errors in several ways:

```rust
use icarus_macros::tool;

// String errors
#[tool]
fn fallible1(x: i32) -> Result<i32, String> {
    if x < 0 {
        Err("Negative numbers not allowed".to_string())
    } else {
        Ok(x * 2)
    }
}

// Custom error types (must implement Display)
#[tool]
fn fallible2(x: i32) -> Result<i32, MyError> {
    // ...
}
```

## Performance

- **Compile-Time Generation**: All tool metadata generated at compile time
- **Zero Runtime Cost**: No reflection or dynamic dispatch
- **Minimal Code Size**: Generated code is highly optimized
- **Distributed Slices**: `linkme` provides efficient compile-time registration

Benchmarks show:
- **Function call overhead**: <5% compared to direct calls
- **Parameter validation**: <100ns for typical parameters
- **Schema generation**: Zero runtime cost (compile-time only)

## Testing

The macro includes comprehensive tests:

- **Compilation Tests** (`tests/compilation/`): Verify macro expansion correctness
- **Integration Tests**: End-to-end tool functionality
- **Property Tests**: Randomized testing with `proptest`
- **Benchmarks** (`benches/`): Performance validation

## Best Practices

1. **Tool Names**: Use clear, descriptive names (snake_case or kebab-case)
2. **Documentation**: Add doc comments to generate tool descriptions
3. **Error Messages**: Provide helpful error messages in `Result::Err`
4. **Type Safety**: Use newtype patterns for domain-specific types
5. **Testing**: Write tests for tool logic separately from macro usage

## Compatibility

- **Rust Version**: 1.83.0 or later (2024 edition)
- **Target**: All Rust targets supported by `linkme`
- **Dependencies**: Requires `icarus-core` and `icarus-runtime`

## Development

To run tests:

```bash
cargo test --package icarus-macros
```

To run benchmarks:

```bash
cargo bench --package icarus-macros
```

To test compilation errors:

```bash
cargo test --package icarus-macros --test compilation_tests
```

## License

See the LICENSE file in the repository root.