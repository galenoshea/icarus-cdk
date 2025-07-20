# Stable Storage Patterns

Guide to using stable storage effectively in Icarus MCP servers.

## Overview

Stable storage is the key to persistent MCP servers on ICP. Data stored in stable memory:
- Survives canister upgrades
- Persists indefinitely
- Provides up to 64GB of storage
- Offers zero-copy access for efficiency

## Basic Concepts

### Memory Management

The Internet Computer provides 64GB of stable memory per canister. Icarus SDK manages this memory through virtual memory pages:

```rust
use ic_stable_structures::memory_manager::{MemoryManager, MemoryId};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};

type Memory = VirtualMemory<DefaultMemoryImpl>;
```

### Storage Types

#### StableBTreeMap
Best for key-value storage with efficient lookups.

```rust
stable_storage! {
    USERS: StableBTreeMap<String, User, Memory> = memory_id!(0);
}

// Usage
USERS.with(|users| {
    users.borrow_mut().insert("user123".to_string(), user);
});
```

#### StableVec
Best for append-only data like logs or events.

```rust
stable_storage! {
    LOGS: StableVec<LogEntry, Memory> = memory_id!(1);
}

// Usage
LOGS.with(|logs| {
    logs.borrow_mut().push(&log_entry).unwrap();
});
```

#### StableCell
Best for single values like configuration.

```rust
stable_storage! {
    CONFIG: StableCell<AppConfig, Memory> = memory_id!(2);
}

// Usage
CONFIG.with(|config| {
    config.borrow_mut().set(new_config).unwrap();
});
```

## Common Patterns

### Pattern 1: User Data Storage

```rust
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: u64,
    pub metadata: HashMap<String, String>,
}

stable_storage! {
    USERS: StableBTreeMap<String, User, Memory> = memory_id!(0);
    USER_COUNT: u64 = 0;
}

#[icarus_module]
mod user_tools {
    #[update]
    #[icarus_tool("Create a new user")]
    pub fn create_user(username: String, email: String) -> Result<String, String> {
        // Validate inputs
        if username.is_empty() || email.is_empty() {
            return Err("Username and email are required".to_string());
        }
        
        // Generate ID
        let id = USER_COUNT.with(|count| {
            let current = *count.borrow();
            *count.borrow_mut() = current + 1;
            format!("user_{}", current + 1)
        });
        
        let user = User {
            id: id.clone(),
            username,
            email,
            created_at: ic_cdk::api::time(),
            metadata: HashMap::new(),
        };
        
        USERS.with(|users| {
            users.borrow_mut().insert(id.clone(), user);
        });
        
        Ok(id)
    }
}
```

### Pattern 2: Event Logging

```rust
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
pub struct Event {
    pub id: u64,
    pub timestamp: u64,
    pub event_type: String,
    pub data: serde_json::Value,
}

stable_storage! {
    EVENTS: StableVec<Event, Memory> = memory_id!(0);
    EVENT_COUNTER: u64 = 0;
}

#[icarus_module]
mod event_tools {
    #[update]
    #[icarus_tool("Log an event")]
    pub fn log_event(event_type: String, data: serde_json::Value) -> Result<u64, String> {
        let id = EVENT_COUNTER.with(|counter| {
            let current = *counter.borrow();
            *counter.borrow_mut() = current + 1;
            current + 1
        });
        
        let event = Event {
            id,
            timestamp: ic_cdk::api::time(),
            event_type,
            data,
        };
        
        EVENTS.with(|events| {
            events.borrow_mut().push(&event)
                .map_err(|_| "Failed to store event".to_string())?;
            Ok(id)
        })
    }
    
    #[query]
    #[icarus_tool("Get recent events")]
    pub fn get_recent_events(limit: Option<usize>) -> Vec<Event> {
        let limit = limit.unwrap_or(10);
        
        EVENTS.with(|events| {
            let events = events.borrow();
            let total = events.len();
            let start = total.saturating_sub(limit);
            
            (start..total)
                .filter_map(|i| events.get(i))
                .collect()
        })
    }
}
```

### Pattern 3: Configuration Management

```rust
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
pub struct Config {
    pub app_name: String,
    pub version: String,
    pub features: HashMap<String, bool>,
    pub limits: HashMap<String, u64>,
}

stable_storage! {
    CONFIG: StableCell<Config, Memory> = memory_id!(0);
}

#[icarus_module]
mod config_tools {
    #[update]
    #[icarus_tool("Update configuration")]
    pub fn update_config(updates: ConfigUpdate) -> Result<(), String> {
        CONFIG.with(|config| {
            let mut current = config.borrow().get().clone();
            
            if let Some(name) = updates.app_name {
                current.app_name = name;
            }
            
            if let Some(features) = updates.features {
                current.features.extend(features);
            }
            
            if let Some(limits) = updates.limits {
                current.limits.extend(limits);
            }
            
            config.borrow_mut().set(current)
                .map_err(|_| "Failed to update config".to_string())
        })
    }
}
```

### Pattern 4: Indexed Data

```rust
stable_storage! {
    DOCUMENTS: StableBTreeMap<String, Document, Memory> = memory_id!(0);
    TAGS_INDEX: StableBTreeMap<String, Vec<String>, Memory> = memory_id!(1);
    AUTHOR_INDEX: StableBTreeMap<Principal, Vec<String>, Memory> = memory_id!(2);
}

#[icarus_module]
mod document_tools {
    #[update]
    #[icarus_tool("Create document with indexing")]
    pub fn create_document(
        title: String,
        content: String,
        tags: Vec<String>
    ) -> Result<String, String> {
        let author = ic_cdk::caller();
        let id = generate_id();
        
        let doc = Document {
            id: id.clone(),
            title,
            content,
            author,
            tags: tags.clone(),
            created_at: ic_cdk::api::time(),
        };
        
        // Store document
        DOCUMENTS.with(|docs| {
            docs.borrow_mut().insert(id.clone(), doc);
        });
        
        // Update tag index
        for tag in tags {
            TAGS_INDEX.with(|index| {
                let mut index = index.borrow_mut();
                let mut doc_ids = index.get(&tag).unwrap_or_default();
                doc_ids.push(id.clone());
                index.insert(tag, doc_ids);
            });
        }
        
        // Update author index
        AUTHOR_INDEX.with(|index| {
            let mut index = index.borrow_mut();
            let mut doc_ids = index.get(&author).unwrap_or_default();
            doc_ids.push(id.clone());
            index.insert(author, doc_ids);
        });
        
        Ok(id)
    }
}
```

## Advanced Patterns

### Memory-Efficient Large Data

For large data that doesn't need frequent access:

```rust
#[derive(IcarusStorable)]
#[icarus_storable(unbounded)]
pub struct LargeData {
    pub chunks: Vec<Vec<u8>>,
    pub metadata: DataMetadata,
}

stable_storage! {
    LARGE_DATA: StableBTreeMap<String, LargeData, Memory> = memory_id!(0);
}
```

### Versioned Storage

For data that needs migration support:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum StoredData {
    V1(DataV1),
    V2(DataV2),
}

impl IcarusStorable for StoredData {
    // Custom implementation for versioning
}
```

### Transactional Updates

Ensure consistency with multiple updates:

```rust
#[update]
#[icarus_tool("Transfer ownership atomically")]
pub fn transfer_ownership(
    item_id: String,
    from: Principal,
    to: Principal
) -> Result<(), String> {
    // Read current state
    let item = ITEMS.with(|items| {
        items.borrow().get(&item_id)
    }).ok_or("Item not found")?;
    
    if item.owner != from {
        return Err("Not the owner".to_string());
    }
    
    // Prepare updates
    let mut updated_item = item.clone();
    updated_item.owner = to;
    
    // Apply all changes atomically
    ITEMS.with(|items| {
        items.borrow_mut().insert(item_id, updated_item);
    });
    
    // Update indices
    update_ownership_index(from, to, item_id)?;
    
    Ok(())
}
```

## Best Practices

### 1. Memory Planning

Plan your memory usage upfront:
```rust
// Reserve memory IDs logically
const USER_DATA: MemoryId = MemoryId::new(0);
const USER_INDEX: MemoryId = MemoryId::new(1);
const TEMP_DATA: MemoryId = MemoryId::new(10);
const LOGS: MemoryId = MemoryId::new(20);
```

### 2. Key Design

Use efficient key structures:
```rust
// Good: Compact, sortable keys
format!("user:{}:{}", timestamp, user_id)

// Bad: Long, unstructured keys
format!("user_data_for_{}_created_at_{}", user_name, date_string)
```

### 3. Batch Operations

Minimize storage access:
```rust
#[update]
#[icarus_tool("Batch update users")]
pub fn batch_update(updates: Vec<UserUpdate>) -> Result<(), String> {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        for update in updates {
            if let Some(mut user) = users.get(&update.id) {
                apply_update(&mut user, update);
                users.insert(update.id, user);
            }
        }
        Ok(())
    })
}
```

### 4. Size Management

Monitor and manage data size:
```rust
#[query]
#[icarus_tool("Get storage statistics")]
pub fn get_storage_stats() -> StorageStats {
    StorageStats {
        user_count: USERS.with(|u| u.borrow().len()),
        event_count: EVENTS.with(|e| e.borrow().len()),
        total_size_estimate: estimate_total_size(),
    }
}
```

## Troubleshooting

### Common Issues

1. **"Memory ID already in use"**
   - Ensure each `memory_id!` is unique
   - Check for duplicate declarations

2. **"Failed to deserialize"**
   - Data structure changed incompatibly
   - Implement migration strategy

3. **"Out of memory"**
   - Monitor memory usage
   - Implement data pruning
   - Consider multi-canister architecture

### Debugging Tools

```rust
#[query]
#[icarus_tool("Debug storage info")]
pub fn debug_storage() -> serde_json::Value {
    json!({
        "users": USERS.with(|u| u.borrow().len()),
        "memory_usage": get_memory_usage(),
        "heap_size": core::mem::size_of::<User>() * USERS.with(|u| u.borrow().len()),
    })
}
```

## Migration Strategies

When data structures change:

1. **Add versioning to your types**
2. **Create migration functions**
3. **Test migrations thoroughly**
4. **Keep old versions readable**

Example migration:
```rust
#[update]
#[icarus_tool("Migrate data to v2")]
pub fn migrate_to_v2() -> Result<(), String> {
    let old_data = OLD_STORAGE.with(|s| s.borrow().clone());
    
    for (key, old_value) in old_data {
        let new_value = migrate_value(old_value)?;
        NEW_STORAGE.with(|s| {
            s.borrow_mut().insert(key, new_value);
        });
    }
    
    Ok(())
}
```