# Icarus SDK API Reference

Complete API documentation for the Icarus SDK crates.

## icarus-core

Core traits and types for building MCP servers on ICP.

### Types

#### `IcarusError`
Standard error type for Icarus operations.

```rust
pub enum IcarusError {
    Serialization(serde_json::Error),
    Network(String),
    Canister(String),
    NotFound(String),
    Unauthorized,
    Custom(String),
}
```

#### `ToolResult`
Standard result type for tool functions.

```rust
pub type ToolResult = Result<serde_json::Value, IcarusError>;
```

### Traits

#### `Persistent`
Trait for types that can be stored persistently.

```rust
pub trait Persistent: Sized {
    async fn get(key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(key: String, value: Vec<u8>) -> Result<()>;
    async fn delete(key: &str) -> Result<bool>;
    async fn list_keys(prefix: Option<&str>) -> Result<Vec<String>>;
}
```

## icarus-derive

Procedural macros for code generation.

### Macros

#### `#[icarus_module]`
Generates tool metadata and exports functions at crate level.

**Attributes:**
- Applied to a module containing tool functions
- Automatically processes `#[icarus_tool]` attributes
- Generates `get_metadata()` query function

**Example:**
```rust
#[icarus_module]
mod tools {
    #[update]
    #[icarus_tool("Store data")]
    pub fn store(data: String) -> Result<(), String> {
        // Implementation
    }
}
```

#### `#[icarus_tool]`
Marks a function as an MCP tool.

**Attributes:**
- `description`: Tool description (required)
- `name`: Override function name (optional)

**Example:**
```rust
#[icarus_tool("Fetch data by ID")]
pub fn fetch(id: String) -> Result<Data, String> {
    // Implementation
}

#[icarus_tool(name = "remove", description = "Delete an item")]
pub fn delete_item(id: String) -> Result<(), String> {
    // Implementation
}
```

#### `#[derive(IcarusStorable)]`
Enables a type to be stored in stable memory.

**Attributes:**
- `unbounded`: No size limit (for large data)
- `max_size = "SIZE"`: Custom size limit (e.g., "64KB")

**Example:**
```rust
#[derive(IcarusStorable)]
pub struct User {
    id: String,
    name: String,
}

#[derive(IcarusStorable)]
#[icarus_storable(unbounded)]
pub struct Document {
    content: String,
    metadata: HashMap<String, String>,
}

#[derive(IcarusStorable)]
#[icarus_storable(max_size = "10KB")]
pub struct Config {
    settings: Vec<Setting>,
}
```

## icarus-canister

ICP canister integration with stable memory.

### Macros

#### `stable_storage!`
Declares stable storage variables.

**Syntax:**
```rust
stable_storage! {
    NAME: Type = memory_id!(ID);
    // ...
}
```

**Supported Types:**
- `StableBTreeMap<K, V, Memory>`: Key-value storage
- `StableVec<T, Memory>`: Vector storage
- `StableCell<T, Memory>`: Single value storage
- `u64`, `i64`, etc.: Primitive types

**Example:**
```rust
use ic_stable_structures::{StableBTreeMap, StableVec, StableCell};

stable_storage! {
    USERS: StableBTreeMap<String, User, Memory> = memory_id!(0);
    LOGS: StableVec<LogEntry, Memory> = memory_id!(1);
    CONFIG: StableCell<AppConfig, Memory> = memory_id!(2);
    COUNTER: u64 = 0;
}
```

#### `memory_id!`
Creates a memory ID for stable storage.

**Syntax:**
```rust
memory_id!(number)
```

**Rules:**
- IDs must be unique within a canister
- Start from 0 and increment
- Cannot reuse IDs after deletion

### Functions

#### `init_storage()`
Initializes stable storage (called automatically).

```rust
pub fn init_storage() {
    // Automatically called by stable_storage!
}
```

### Memory Management

The SDK uses IC's stable memory with the following guarantees:
- 64GB total stable memory available
- Zero-copy operations for large data
- Automatic memory management
- Persistence across upgrades

### Best Practices

1. **Memory IDs**: Always use sequential IDs starting from 0
2. **Size Limits**: Set appropriate limits for bounded types
3. **Key Design**: Use efficient key structures for maps
4. **Batch Operations**: Group related updates

## Type Conversions

### Candid ↔ Rust

The SDK automatically handles conversions:

| Rust Type | Candid Type |
|-----------|-------------|
| `String` | `text` |
| `Vec<T>` | `vec T` |
| `Option<T>` | `opt T` |
| `Result<T, E>` | `variant { Ok: T; Err: E }` |
| `u64` | `nat64` |
| `i64` | `int64` |
| `bool` | `bool` |
| Custom structs | `record` |

### JSON ↔ Candid

The bridge handles these conversions automatically.

## Error Handling

### Best Practices

1. Use `Result<T, String>` for simple errors
2. Return descriptive error messages
3. Log errors for debugging
4. Handle panics gracefully

### Example

```rust
#[update]
#[icarus_tool("Update user profile")]
pub fn update_profile(id: String, name: String) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        match users.get(&id) {
            Some(mut user) => {
                user.name = name;
                users.insert(id, user);
                Ok(())
            }
            None => Err(format!("User {} not found", id))
        }
    })
}
```

## Advanced Topics

### Custom Memory Layouts

```rust
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};

const CUSTOM_MEMORY_ID: MemoryId = MemoryId::new(10);

stable_storage! {
    CUSTOM_DATA: StableBTreeMap<u64, CustomData, Memory> = {
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(CUSTOM_MEMORY_ID))
        )
    };
}
```

### Upgrade Handlers

```rust
#[pre_upgrade]
fn pre_upgrade() {
    // Automatically handled by stable storage
}

#[post_upgrade]
fn post_upgrade() {
    // Automatically handled by stable storage
}
```

### Performance Optimization

1. Use `#[query]` for read-only operations
2. Batch updates when possible
3. Use appropriate data structures
4. Consider memory layout for cache efficiency