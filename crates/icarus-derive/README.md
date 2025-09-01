# icarus-derive

Procedural macros for the Icarus SDK that enable automatic MCP tool metadata generation.

This crate provides:

- `#[icarus_module]` - Generates tool metadata and exports for your canister
- `#[icarus_tool]` - Marks functions as MCP tools with automatic metadata
- `#[derive(IcarusStorable)]` - Enables types for stable storage

## Usage

This crate is typically used as part of the main `icarus` SDK:

```toml
[dependencies]
icarus = "0.1"
```

Then use the macros in your code:

```rust
use icarus::prelude::*;

#[icarus_module]
mod tools {
    #[update]
    #[icarus_tool("Store a memory")]
    pub fn memorize(content: String) -> Result<String, String> {
        // Your implementation
    }
}
```

## License

Apache 2.0