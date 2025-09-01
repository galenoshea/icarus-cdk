# icarus-canister

ICP canister integration with stable memory for persistent MCP servers.

This crate provides:

- `stable_storage!` macro for declaring persistent data structures
- Memory management utilities for ICP stable memory
- State persistence helpers
- Integration with IC-CDK for canister development

## Features

- Zero-copy stable memory operations
- Type-safe stable structures
- Automatic state persistence
- Built-in memory management

## Usage

This crate is typically used as part of the main `icarus` SDK:

```toml
[dependencies]
icarus = "0.1"
```

Then use stable storage in your canister:

```rust
use icarus::prelude::*;

stable_storage! {
    MEMORIES: StableBTreeMap<String, MemoryEntry, Memory> = memory_id!(0);
}
```

## License

Apache 2.0