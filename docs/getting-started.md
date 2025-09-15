# Getting Started with Icarus SDK

This guide will help you create your first MCP server using the Icarus SDK.

## Prerequisites

- Rust 1.75 or later
- Icarus CLI installed (`cargo install icarus-cli`)
- Basic familiarity with Rust

> **Note**: Icarus 0.7.0 introduces a modular architecture with focused crates. The CLI and core functionality remain unchanged, but you now have more flexibility to use specific components.

## Creating Your First Project

### 1. Create a New Project

```bash
icarus new my-memory-server
cd my-memory-server
```

This creates a new project with:
- `Cargo.toml` configured with Icarus dependencies
- `src/lib.rs` with a sample memory storage implementation
- `.gitignore` for common files

### 2. Understanding the Generated Code

The generated `src/lib.rs` demonstrates key Icarus concepts:

```rust
// Define persistent data structures
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub created_at: u64,
    pub tags: Vec<String>,
}

// Declare stable storage
stable_storage! {
    MEMORIES: StableBTreeMap<String, MemoryEntry, Memory> = memory_id!(0);
    COUNTER: u64 = 0;
}

// Define tools with metadata
#[icarus_module]
mod tools {
    #[update]
    #[icarus_tool("Store a new memory")]
    pub fn memorize(content: String, tags: Option<Vec<String>>) -> Result<String, String> {
        // Implementation
    }
}
```

### 3. Key Concepts

#### Stable Storage
- Use `stable_storage!` to declare persistent data
- Data survives canister upgrades
- Supports various IC stable structures

#### Tool Definition
- `#[icarus_module]` on a module generates metadata
- `#[icarus_tool("description")]` marks MCP tools
- `#[update]` for state-changing operations
- `#[query]` for read-only operations

#### Data Types
- `#[derive(IcarusStorable)]` enables stable storage
- Use `#[icarus_storable(unbounded)]` for large data
- All types must implement `CandidType`

### 4. Testing Your Code

Run tests:
```bash
cargo test
```

Note: The project is automatically built when you deploy, so there's no separate build step needed.

### 5. Local Deployment

Deploy to local network:
```bash
icarus deploy --network local
```

This will:
1. Start local dfx network if needed
2. Build your project automatically
3. Deploy your canister
4. Return the canister ID
5. Show bridge configuration

### 6. Testing with Claude Desktop

Start the bridge:
```bash
icarus bridge start --canister-id <your-canister-id>
```

Configure Claude Desktop:
```bash
icarus connect --canister-id <your-canister-id>
```

## Next Steps

- Read the [API Reference](api-reference.md) for detailed documentation
- Explore [Example Projects](../examples/) for more patterns
- Learn about [Stable Storage Patterns](stable-storage.md)
- Understand [Icarus Macros](macros.md) in depth

## Common Patterns

### Adding a New Tool

```rust
#[icarus_module]
mod tools {
    #[query]
    #[icarus_tool("Search memories by keyword")]
    pub fn search(keyword: String) -> Vec<MemoryEntry> {
        MEMORIES.with(|m| {
            m.borrow()
                .iter()
                .filter(|(_, entry)| entry.content.contains(&keyword))
                .map(|(_, entry)| entry.clone())
                .collect()
        })
    }
}
```

### Error Handling

```rust
#[update]
#[icarus_tool("Delete a memory by ID")]
pub fn delete(id: String) -> Result<(), String> {
    MEMORIES.with(|m| {
        if m.borrow_mut().remove(&id).is_some() {
            Ok(())
        } else {
            Err(format!("Memory with id {} not found", id))
        }
    })
}
```

### Using Multiple Storage Types

```rust
stable_storage! {
    USERS: StableBTreeMap<Principal, User, Memory> = memory_id!(0);
    MEMORIES: StableBTreeMap<String, MemoryEntry, Memory> = memory_id!(1);
    CONFIG: StableCell<Config, Memory> = memory_id!(2);
}
```

## Troubleshooting

### Build Errors
- Ensure `wasm32-unknown-unknown` target is installed
- Check that all dependencies versions match

### Deployment Issues
- Verify dfx is running: `dfx ping`
- Check canister logs: `dfx canister logs <canister-name>`

### Bridge Connection
- Ensure canister is deployed and running
- Verify canister ID is correct
- Check Claude Desktop configuration