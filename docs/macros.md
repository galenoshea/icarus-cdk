# Icarus Macros Guide

Detailed guide to all macros provided by the Icarus SDK.

## Overview

Icarus macros eliminate boilerplate and generate standard ICP code while providing excellent developer experience. All macros are compile-time only - they generate standard Candid interfaces with no runtime overhead.

## Module-Level Macros

### `#[icarus_module]`

The most important macro that processes a module containing tool functions.

#### What it does:
1. Exports all public functions at crate level
2. Generates `get_metadata()` query function
3. Preserves `#[update]` and `#[query]` attributes
4. Creates tool metadata from `#[icarus_tool]` attributes

#### Usage:
```rust
#[icarus_module]
mod tools {
    use super::*;
    
    #[update]
    #[icarus_tool("Store a value")]
    pub fn store(key: String, value: String) -> Result<(), String> {
        // Implementation
    }
    
    #[query]
    #[icarus_tool("Retrieve a value")]
    pub fn get(key: String) -> Option<String> {
        // Implementation
    }
}
```

#### Generated code:
- `store` and `get` functions exported at crate level
- `get_metadata()` query function returning tool information
- Standard Candid methods with proper attributes

## Function-Level Macros

### `#[icarus_tool]`

Marks a function as an MCP tool and provides metadata.

#### Syntax variations:
```rust
// Simple form with description
#[icarus_tool("Tool description")]

// With custom name
#[icarus_tool(name = "custom_name", description = "Tool description")]

// Query operations (read-only)
#[icarus_tool(description = "Read data", is_query = true)]
```

#### Requirements:
- Must be inside an `#[icarus_module]` module
- Function must be `pub`
- Should have `#[update]` or `#[query]` attribute

#### Examples:

```rust
#[icarus_module]
mod tools {
    // Basic tool
    #[update]
    #[icarus_tool("Add two numbers")]
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    
    // Tool with optional parameters
    #[update]
    #[icarus_tool("Greet a user")]
    pub fn greet(name: String, title: Option<String>) -> String {
        match title {
            Some(t) => format!("Hello, {} {}!", t, name),
            None => format!("Hello, {}!", name),
        }
    }
    
    // Tool with error handling
    #[update]
    #[icarus_tool("Divide two numbers")]
    pub fn divide(a: f64, b: f64) -> Result<f64, String> {
        if b == 0.0 {
            Err("Division by zero".to_string())
        } else {
            Ok(a / b)
        }
    }
}
```

## Derive Macros

### `#[derive(IcarusStorable)]`

Enables a type to be stored in IC stable memory.

#### Basic usage:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
pub struct User {
    pub id: String,
    pub name: String,
    pub created_at: u64,
}
```

#### Size constraints:

**Default (1MB limit):**
```rust
#[derive(IcarusStorable)]
pub struct Config {
    settings: HashMap<String, String>,
}
```

**Unbounded (no limit):**
```rust
#[derive(IcarusStorable)]
#[icarus_storable(unbounded)]
pub struct Document {
    content: String,
    attachments: Vec<Attachment>,
}
```

**Custom limit:**
```rust
#[derive(IcarusStorable)]
#[icarus_storable(max_size = "64KB")]
pub struct Message {
    text: String,
    metadata: MessageMeta,
}
```

#### Requirements:
- Type must implement `Serialize` and `Deserialize`
- Type must implement `CandidType`
- All nested types must also be storable

## Storage Macros

### `stable_storage!`

Declares stable storage variables that persist across canister upgrades.

#### Syntax:
```rust
stable_storage! {
    VARIABLE_NAME: StorageType = initialization;
}
```

#### Supported storage types:

**BTreeMap:**
```rust
stable_storage! {
    USERS: StableBTreeMap<String, User, Memory> = memory_id!(0);
    SESSIONS: StableBTreeMap<Principal, Session, Memory> = memory_id!(1);
}
```

**Vector:**
```rust
stable_storage! {
    EVENTS: StableVec<Event, Memory> = memory_id!(2);
}
```

**Cell (single value):**
```rust
stable_storage! {
    CONFIG: StableCell<AppConfig, Memory> = memory_id!(3);
}
```

**Primitives:**
```rust
stable_storage! {
    COUNTER: u64 = 0;
    LAST_UPDATE: i64 = 0;
}
```

#### Complete example:
```rust
use ic_stable_structures::{StableBTreeMap, StableVec, StableCell};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::DefaultMemoryImpl;

type Memory = VirtualMemory<DefaultMemoryImpl>;

stable_storage! {
    // User storage with string keys
    USERS: StableBTreeMap<String, User, Memory> = memory_id!(0);
    
    // Event log
    EVENT_LOG: StableVec<Event, Memory> = memory_id!(1);
    
    // Global configuration
    CONFIG: StableCell<Config, Memory> = memory_id!(2);
    
    // Simple counters
    USER_COUNT: u64 = 0;
    LAST_EVENT_ID: u64 = 0;
}
```

### `memory_id!`

Creates a memory ID for stable storage allocation.

#### Usage:
```rust
memory_id!(0)  // First memory region
memory_id!(1)  // Second memory region
memory_id!(2)  // Third memory region
```

#### Rules:
1. IDs must be unique per storage variable
2. Start from 0 and increment
3. Cannot skip numbers
4. Cannot reuse IDs

## Macro Patterns

### Pattern 1: CRUD Operations

```rust
#[icarus_module]
mod tools {
    #[update]
    #[icarus_tool("Create a new item")]
    pub fn create(data: ItemData) -> Result<String, String> {
        let id = generate_id();
        let item = Item { id: id.clone(), data };
        ITEMS.with(|items| items.borrow_mut().insert(id.clone(), item));
        Ok(id)
    }
    
    #[query]
    #[icarus_tool("Read an item by ID")]
    pub fn read(id: String) -> Option<Item> {
        ITEMS.with(|items| items.borrow().get(&id))
    }
    
    #[update]
    #[icarus_tool("Update an existing item")]
    pub fn update(id: String, data: ItemData) -> Result<(), String> {
        ITEMS.with(|items| {
            let mut items = items.borrow_mut();
            match items.get(&id) {
                Some(mut item) => {
                    item.data = data;
                    items.insert(id, item);
                    Ok(())
                }
                None => Err(format!("Item {} not found", id))
            }
        })
    }
    
    #[update]
    #[icarus_tool("Delete an item")]
    pub fn delete(id: String) -> Result<(), String> {
        ITEMS.with(|items| {
            if items.borrow_mut().remove(&id).is_some() {
                Ok(())
            } else {
                Err(format!("Item {} not found", id))
            }
        })
    }
}
```

### Pattern 2: Batch Operations

```rust
#[icarus_module]
mod tools {
    #[update]
    #[icarus_tool("Import multiple items")]
    pub fn batch_import(items: Vec<ImportItem>) -> Result<BatchResult, String> {
        let mut success = 0;
        let mut failed = 0;
        let mut errors = vec![];
        
        ITEMS.with(|storage| {
            let mut storage = storage.borrow_mut();
            for item in items {
                match validate_item(&item) {
                    Ok(valid_item) => {
                        storage.insert(valid_item.id.clone(), valid_item);
                        success += 1;
                    }
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("{}: {}", item.id, e));
                    }
                }
            }
        });
        
        Ok(BatchResult { success, failed, errors })
    }
}
```

### Pattern 3: Complex Queries

```rust
#[icarus_module]
mod tools {
    #[query]
    #[icarus_tool("Search items with filters")]
    pub fn search(filters: SearchFilters) -> SearchResult {
        ITEMS.with(|items| {
            let items = items.borrow();
            let mut results = vec![];
            
            for (_, item) in items.iter() {
                if matches_filters(&item, &filters) {
                    results.push(item.clone());
                }
            }
            
            // Sort and paginate
            results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            let total = results.len();
            let page_results = results
                .into_iter()
                .skip(filters.offset.unwrap_or(0))
                .take(filters.limit.unwrap_or(10))
                .collect();
                
            SearchResult {
                items: page_results,
                total,
                offset: filters.offset.unwrap_or(0),
                limit: filters.limit.unwrap_or(10),
            }
        })
    }
}
```

## Best Practices

1. **Module Organization**: Group related tools in the same module
2. **Error Messages**: Provide clear, actionable error messages
3. **Parameter Validation**: Validate inputs before processing
4. **Query vs Update**: Use `#[query]` for read-only operations
5. **Memory Efficiency**: Choose appropriate storage types and limits
6. **Documentation**: Use descriptive tool descriptions

## Common Pitfalls

1. **Forgetting `#[update]` or `#[query]`**: Always specify the method type
2. **Private functions**: Tool functions must be `pub`
3. **Memory ID conflicts**: Each storage variable needs a unique ID
4. **Size limits**: Consider data growth when setting limits
5. **Missing derives**: Ensure all required traits are derived

## Debugging

To see generated code:
```bash
cargo expand
```

To check metadata generation:
```rust
#[test]
fn test_metadata() {
    let metadata = get_metadata();
    println!("{}", metadata);
}
```