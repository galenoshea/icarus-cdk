# Parameter Translation Guide

The Icarus SDK provides intelligent parameter translation between MCP (Model Context Protocol) JSON format and ICP (Internet Computer Protocol) Candid format. This ensures seamless bridge operation regardless of how developers design their tool parameters.

## Overview

When Claude Desktop calls your canister tools through the MCP protocol, it sends arguments as JSON objects. However, ICP canisters expect parameters in Candid format. The Icarus bridge automatically handles this translation using the `ParamMapper` module.

## Parameter Styles

### 1. Positional Parameters

Most common for simple functions with a few parameters. Arguments are passed in a specific order.

```rust
#[icarus_tool("Store a memory with a unique key")]
pub fn memorize(key: String, content: String) -> Result<String, String> {
    // Function receives two separate parameters
}
```

The bridge automatically generates metadata:
```json
{
  "x-icarus-params": {
    "style": "positional",
    "order": ["key", "content"],
    "types": ["text", "text"]
  }
}
```

### 2. Record Parameters

Used for complex functions with many parameters or when you want a single struct parameter.

```rust
#[derive(CandidType, Deserialize)]
pub struct CreateUserArgs {
    name: String,
    email: String,
    age: u32,
    active: bool,
}

#[icarus_tool("Create a new user")]
pub fn create_user(args: CreateUserArgs) -> Result<String, String> {
    // Function receives a single struct parameter
}
```

### 3. Empty Parameters

For functions that take no parameters.

```rust
#[icarus_tool("Get system status")]
pub fn get_status() -> Result<String, String> {
    // No parameters needed
}
```

## Automatic Detection

The SDK automatically detects the appropriate parameter style based on your function signature:

- **0 parameters** → Empty style
- **1-5 simple parameters** → Positional style
- **Complex struct parameter** → Record style
- **Many parameters (>5)** → Record style

## Custom Parameter Hints

You can explicitly specify parameter handling using the `x-icarus-params` extension in your tool's input schema:

```rust
// In your custom tool implementation
fn get_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "key": { "type": "string" },
            "value": { "type": "string" }
        },
        "required": ["key", "value"],
        "x-icarus-params": {
            "style": "positional",
            "order": ["key", "value"],
            "types": ["text", "text"]
        }
    })
}
```

## Type Mapping

The bridge automatically maps JSON types to Candid types:

| JSON Type | Candid Type | Rust Type |
|-----------|-------------|-----------|
| string    | text        | String    |
| number    | nat64       | u64       |
| integer   | int64       | i64       |
| boolean   | bool        | bool      |
| object    | record      | struct    |
| array     | vec         | Vec<T>    |

## Fallback Strategies

If the bridge cannot determine the parameter style, it uses intelligent fallback strategies:

1. **Try as positional** - For objects with ≤5 properties
2. **Try as single value** - For simple JSON values
3. **Try as JSON string** - For complex objects

This ensures your tools work even without explicit parameter hints.

## Example: Memento Tool

Here's how the Memento tool uses positional parameters:

```rust
#[icarus_module]
mod tools {
    /// Store a memory with a unique key
    #[update]
    #[icarus_tool("Store a memory with a unique key")]
    pub fn memorize(key: String, content: String) -> Result<String, String> {
        // Validate inputs
        if key.trim().is_empty() {
            return Err("Key cannot be empty".to_string());
        }
        
        // Store the memory
        let memory = MemoryEntry {
            key: key.clone(),
            content,
            created_at: ic_cdk::api::time(),
        };
        
        MEMORIES.with(|m| {
            m.borrow_mut().insert(key.clone(), memory);
            Ok(key)
        })
    }
}
```

When Claude Desktop calls this tool:
```json
{
  "method": "memorize",
  "params": {
    "key": "important_note",
    "content": "Remember to update the documentation"
  }
}
```

The bridge automatically translates this to two separate Candid parameters:
```
memorize("important_note", "Remember to update the documentation")
```

## Troubleshooting

### "Failed to decode call arguments" Error

This typically means the parameter translation failed. Check:

1. **Parameter count mismatch** - Ensure the number of parameters matches
2. **Type mismatch** - Verify JSON types match expected Candid types
3. **Missing required fields** - All required parameters must be provided

### Debugging Parameter Translation

Enable debug mode to see parameter translation details:

```bash
ICARUS_DEBUG=1 icarus bridge start --canister-id <your-canister>
```

This will show:
- Detected parameter style
- JSON to Candid conversion process
- Any fallback strategies used

## Best Practices

### Recommended Parameter Strategy

Follow this decision tree for optimal AI and human understanding:

```
Number of Parameters?
├─ 0 → No parameters needed
├─ 1-2 → Use positional parameters (simple and clear)
├─ 3+ → Use args record (self-documenting)
└─ Complex single param → Use named record type
```

### Why This Matters

1. **AI Integration**: Args records maintain structure from MCP JSON to Candid, reducing confusion for Claude
2. **Candid UI**: Field names in records provide documentation even when the UI lacks details
3. **ICP Alignment**: Follows patterns used by major ICP projects (NNS, OpenChat)
4. **Evolution**: Records with optional fields allow API growth without breaking changes

### Implementation Guidelines

1. **Simple functions (1-2 params)**: Keep positional for clarity
   ```rust
   pub fn get_user(id: String) -> Result<User, String>
   pub fn transfer(to: Principal, amount: u64) -> Result<(), String>
   ```

2. **Complex functions (3+ params)**: Use records with descriptive names
   ```rust
   pub struct CreateUserArgs {
       name: String,           // Self-documenting
       email: String,          // Clear purpose
       role: UserRole,         // Type-safe
       metadata: Option<Map>,  // Optional fields obvious
   }
   ```

3. **Naming convention**: Always use `{Action}{Resource}Args` pattern
   - `CreateUserArgs`, `UpdateConfigArgs`, `QueryItemsArgs`

4. **Document thoroughly**: Add doc comments to complex types
5. **Test with Claude Desktop**: Verify the parameter translation works smoothly
6. **Handle errors gracefully**: Return clear messages for parameter validation failures

## Migration from 0.4.0

If you're upgrading from 0.4.0, the parameter translation is now automatic. You don't need to change your code - the bridge will handle the translation intelligently.

However, if you were experiencing "failed to decode call arguments" errors, they should now be resolved automatically.

## Further Reading

- [Candid Type System](https://internetcomputer.org/docs/current/developer-docs/backend/candid/)
- [MCP Protocol Specification](https://github.com/anthropics/mcp)
- [Icarus Tool Development](./tool-development.md)