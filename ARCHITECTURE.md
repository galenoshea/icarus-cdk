# Icarus SDK Architecture

Technical architecture of the Icarus SDK - the open source foundation for building MCP servers on ICP.

## Overview

The Icarus SDK provides a Rust framework that bridges the Model Context Protocol (MCP) with the Internet Computer Protocol (ICP). It enables developers to build persistent AI tools without blockchain expertise.

## SDK Structure

```
icarus-sdk/
├── icarus-core/        # Core traits and types
│   ├── src/
│   │   ├── lib.rs      # Public API exports
│   │   ├── types.rs    # Core data types
│   │   ├── traits.rs   # Tool and storage traits
│   │   └── metadata.rs # MCP metadata generation
│   └── Cargo.toml
│
├── icarus-derive/      # Procedural macros
│   ├── src/
│   │   ├── lib.rs      # Macro entry points
│   │   ├── module.rs   # #[icarus_module] implementation
│   │   ├── tool.rs     # #[icarus_tool] implementation
│   │   └── storable.rs # #[derive(IcarusStorable)]
│   └── Cargo.toml
│
└── icarus-canister/    # ICP integration
    ├── src/
    │   ├── lib.rs      # Public prelude
    │   ├── storage.rs  # Stable storage abstractions
    │   ├── memory.rs   # Memory management
    │   └── testing.rs  # PocketIC utilities
    └── Cargo.toml
```

## Core Components

### 1. icarus-core

The foundation crate defining core abstractions:

```rust
// Core traits that all tools must implement
pub trait IcarusTool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;
}

// Metadata for MCP discovery
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

// Result types for tool execution
pub type ToolResult<T> = Result<T, ToolError>;
```

### 2. icarus-derive

Procedural macros that generate boilerplate code:

#### #[icarus_module]
Transforms a module into an MCP-compatible service:
```rust
#[icarus_module]
mod tools {
    // Regular Rust functions become MCP tools
}
```

Generates:
- Candid interface definitions
- Metadata collection functions
- Query/update method routing

#### #[icarus_tool]
Marks functions as MCP tools:
```rust
#[query]
#[icarus_tool("Search memories by keyword")]
pub fn search(keyword: String) -> Vec<Memory> {
    // Implementation
}
```

Generates:
- JSON schema from Rust types
- MCP tool registration
- Error handling wrappers

#### #[derive(IcarusStorable)]
Makes types compatible with stable storage:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
pub struct Memory {
    id: String,
    content: String,
}
```

### 3. icarus-canister

ICP-specific functionality and stable storage:

```rust
// Stable storage macro
stable_storage! {
    MEMORIES: StableBTreeMap<String, Memory, Memory> = memory_id!(0);
    EVENTS: StableVec<Event, Memory> = memory_id!(1);
}

// Memory management
pub fn memory_id(id: u8) -> Memory {
    MemoryId::new(id).into()
}
```

## Architecture Principles

### 1. Zero Runtime Overhead

All MCP complexity is handled at compile time:
- Macros generate static code
- No runtime reflection or parsing
- Direct Candid serialization

### 2. Clean Separation

The SDK maintains clear boundaries:
- **Compile Time**: Macros generate ICP code
- **Runtime**: Pure ICP canister execution
- **Bridge Layer**: Handles MCP↔ICP translation

### 3. Progressive Disclosure

Start simple, add complexity as needed:
```rust
// Minimal tool
#[icarus_tool("Say hello")]
pub fn hello() -> String {
    "Hello!".to_string()
}

// Advanced tool with storage
#[icarus_tool("Store data")]
pub fn store(key: String, value: Data) -> Result<(), String> {
    STORAGE.with(|s| {
        s.borrow_mut().insert(key, value);
        Ok(())
    })
}
```

## Data Flow

### 1. Development Time

```
Developer Code
    ↓
Procedural Macros
    ↓
Generated ICP Code
    ↓
WASM Compilation
    ↓
Deployed Canister
```

### 2. Runtime Flow

```
MCP Request → Bridge → Canister Method → Response
                           ↓
                    Stable Storage
```

## Storage Architecture

### Memory Layout

```
┌─────────────────────────────────┐
│      Heap Memory (4GB)          │
│  ┌──────────────────────────┐   │
│  │ Runtime Data             │   │
│  │ - Function calls         │   │
│  │ - Temporary variables    │   │
│  │ - IC System State        │   │
│  └──────────────────────────┘   │
└─────────────────────────────────┘

┌─────────────────────────────────┐
│    Stable Memory (64GB)          │
│  ┌──────────────────────────┐   │
│  │ Memory 0: User Data      │   │
│  │ - BTreeMap structures    │   │
│  │ - Persisted state        │   │
│  ├──────────────────────────┤   │
│  │ Memory 1: Event Logs     │   │
│  │ - StableVec append-only  │   │
│  ├──────────────────────────┤   │
│  │ Memory 2: Configuration  │   │
│  │ - StableCell single val  │   │
│  ├──────────────────────────┤   │
│  │ Memory 3-255: User defined│   │
│  └──────────────────────────┘   │
└─────────────────────────────────┘
```

### Storage Patterns

```rust
// Singleton pattern
stable_storage! {
    CONFIG: StableCell<Config, Memory> = memory_id!(0);
}

// Key-value pattern
stable_storage! {
    DATA: StableBTreeMap<String, Value, Memory> = memory_id!(1);
}

// Append-only pattern
stable_storage! {
    LOGS: StableVec<LogEntry, Memory> = memory_id!(2);
}
```

## Type System

### Supported Types

The SDK automatically handles conversion between Rust and Candid types:

| Rust Type | Candid Type | MCP (JSON) Type |
|-----------|-------------|-----------------|
| String | Text | string |
| u64, i64 | Nat64, Int64 | number |
| Vec<T> | Vec | array |
| Option<T> | Opt | null/value |
| Result<T,E> | Variant | value/error |
| Struct | Record | object |
| Enum | Variant | string/object |

### Custom Types

```rust
#[derive(Serialize, Deserialize, CandidType, IcarusStorable)]
pub struct CustomData {
    #[serde(rename = "userId")]
    pub user_id: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
}
```

### Error Propagation

```rust
#[icarus_tool("Complex operation")]
pub fn complex_op(input: String) -> Result<Output, String> {
    validate_input(&input)?;
    
    let data = STORAGE.with(|s| {
        s.borrow()
         .get(&input)
         .ok_or_else(|| "Not found".to_string())
    })?;
    
    process_data(data).map_err(|e| e.to_string())
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tool_logic() {
        let result = hello();
        assert_eq!(result, "Hello!");
    }
}
```

### Integration Tests

```rust
use pocket_ic::PocketIc;

#[tokio::test]
async fn test_canister_integration() {
    let pic = PocketIc::new();
    let canister_id = pic.deploy_canister(WASM_BYTES);
    
    let result = pic.query_call(
        canister_id,
        "hello",
        candid::encode_args(()).unwrap()
    ).await;
    
    assert!(result.is_ok());
}
```

## Performance Optimization

### 1. Query vs Update

```rust
#[query]  // Read-only, no consensus needed
#[icarus_tool("Get data")]
pub fn get_data(key: String) -> Option<Data> {
    DATA.with(|d| d.borrow().get(&key))
}

#[update]  // Modifies state, requires consensus
#[icarus_tool("Set data")]
pub fn set_data(key: String, value: Data) -> Result<(), String> {
    DATA.with(|d| d.borrow_mut().insert(key, value));
    Ok(())
}
```

### 2. Batch Operations

```rust
#[icarus_tool("Batch insert")]
pub fn batch_insert(items: Vec<(String, Data)>) -> Result<u32, String> {
    DATA.with(|d| {
        let mut map = d.borrow_mut();
        let count = items.len() as u32;
        for (key, value) in items {
            map.insert(key, value);
        }
        Ok(count)
    })
}
```

### 3. Memory Efficiency

```rust
// Use references for large data
#[icarus_tool("Process large data")]
pub fn process(data_ref: &[u8]) -> Result<Summary, String> {
    // Process without copying
}

// Paginate results
#[icarus_tool("List with pagination")]
pub fn list_paginated(offset: u32, limit: u32) -> Vec<Item> {
    ITEMS.with(|items| {
        items.borrow()
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect()
    })
}
```

## Security Considerations

### 1. Input Validation

```rust
#[icarus_tool("Secure operation")]
pub fn secure_op(input: String) -> Result<String, String> {
    // Validate length
    if input.len() > 1000 {
        return Err("Input too large".to_string());
    }
    
    // Validate content
    if !input.chars().all(|c| c.is_alphanumeric()) {
        return Err("Invalid characters".to_string());
    }
    
    // Process safely
    Ok(process_validated(input))
}
```

### 2. Access Control

```rust
#[icarus_tool("Admin operation")]
pub fn admin_op() -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    if !is_admin(caller) {
        return Err("Unauthorized".to_string());
    }
    
    // Perform admin operation
    Ok(())
}
```

## Future Enhancements

### Planned Features

1. **Async Tool Support**
   ```rust
   #[icarus_tool("Async operation")]
   pub async fn async_op() -> Result<Data, String> {
       let external = fetch_external().await?;
       process(external)
   }
   ```

2. **Streaming Responses**
   ```rust
   #[icarus_tool("Stream data")]
   pub fn stream() -> impl Stream<Item = Data> {
       // Return streaming iterator
   }
   ```

3. **Tool Composition**
   ```rust
   #[icarus_compose]
   pub fn composed_tool() -> Result<Output, String> {
       let a = tool_a()?;
       let b = tool_b(a)?;
       tool_c(b)
   }
   ```

## Best Practices

1. **Keep Tools Focused**: Each tool should do one thing well
2. **Use Appropriate Annotations**: `#[query]` for reads, `#[update]` for writes
3. **Handle Errors Gracefully**: Return meaningful error messages
4. **Document Parameters**: Use descriptive tool descriptions
5. **Test Thoroughly**: Unit and integration tests
6. **Monitor Storage**: Track memory usage and growth
7. **Version Carefully**: Consider upgrade compatibility

## Conclusion

The Icarus SDK architecture provides a powerful yet simple foundation for building persistent MCP servers. By leveraging Rust's type system and ICP's infrastructure, developers can create robust AI tools without the complexity of traditional blockchain development.