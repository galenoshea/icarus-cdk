# Parameter Style Guide

This guide establishes conventions for designing function parameters in Icarus SDK canisters to optimize for both human and AI understanding.

## Core Principle

Design parameters that are self-documenting and work well with both the Candid UI and AI assistants like Claude.

## Parameter Strategy Decision Tree

```
Number of Parameters?
├─ 0 → No parameters needed
├─ 1-2 → Use positional parameters
├─ 3+ → Use args record
└─ Complex single param → Use named record type
```

## Patterns by Use Case

### 1. Query Operations (Read-Only)

#### Simple Queries (1-2 params)
```rust
// ✅ Good - Clear and concise
#[query]
#[icarus_tool("Get user by ID")]
pub fn get_user(user_id: String) -> Result<User, String>

// ✅ Good - Two related parameters
#[query]
#[icarus_tool("Get users by role")]
pub fn get_users_by_role(role: String, include_inactive: bool) -> Result<Vec<User>, String>
```

#### Complex Queries (3+ params)
```rust
// ✅ Good - Self-documenting record
#[derive(CandidType, Deserialize)]
pub struct QueryUsersArgs {
    /// Filter by role (optional)
    role: Option<String>,
    /// Include inactive users
    include_inactive: bool,
    /// Maximum number of results
    limit: u32,
    /// Offset for pagination
    offset: u32,
    /// Sort field
    sort_by: Option<String>,
}

#[query]
#[icarus_tool("Query users with filters")]
pub fn query_users(args: QueryUsersArgs) -> Result<Vec<User>, String>
```

### 2. Update Operations (State-Changing)

#### Simple Updates
```rust
// ✅ Good - Clear single parameter
#[update]
#[icarus_tool("Delete user")]
pub fn delete_user(user_id: String) -> Result<String, String>

// ✅ Good - Two essential parameters
#[update]
#[icarus_tool("Transfer tokens")]
pub fn transfer(to: Principal, amount: u64) -> Result<String, String>
```

#### Complex Updates
```rust
// ✅ Good - Grouped related parameters
#[derive(CandidType, Deserialize)]
pub struct CreateUserArgs {
    /// User's display name
    name: String,
    /// Email address (must be unique)
    email: String,
    /// User role in the system
    role: UserRole,
    /// Optional profile metadata
    metadata: Option<HashMap<String, String>>,
    /// Account activation status
    active: bool,
}

#[update]
#[icarus_tool("Create a new user")]
pub fn create_user(args: CreateUserArgs) -> Result<User, String>
```

### 3. Configuration Operations

Always use records for configuration to support future expansion:

```rust
// ✅ Good - Extensible configuration
#[derive(CandidType, Deserialize)]
pub struct ConfigArgs {
    /// API endpoint URL
    api_url: Option<String>,
    /// Request timeout in seconds
    timeout_seconds: Option<u32>,
    /// Maximum retry attempts
    max_retries: Option<u8>,
    /// Enable debug logging
    debug_mode: Option<bool>,
}

#[update]
#[icarus_tool("Update system configuration")]
pub fn update_config(args: ConfigArgs) -> Result<String, String>
```

## Naming Conventions

### Record Type Names
- **Pattern**: `{Action}{Resource}Args`
- **Examples**: 
  - `CreateUserArgs`
  - `UpdateConfigArgs`
  - `QueryItemsArgs`
  - `DeleteBatchArgs`

### Field Names
- **Use snake_case**: `user_id`, `created_at`, `is_active`
- **Be descriptive**: Prefer `expiration_timestamp` over `exp`
- **Match MCP names**: Keep consistency between MCP and Candid field names
- **Indicate optionality**: Use `Option<T>` for optional fields

### Boolean Fields
- **Prefix with is/has/should**: `is_active`, `has_permission`, `should_notify`
- **Avoid negatives**: Use `is_active` instead of `is_not_inactive`

## Documentation Standards

### Record Types
```rust
/// Arguments for creating a new user account
/// 
/// All fields are validated before user creation.
/// Email must be unique across the system.
#[derive(CandidType, Deserialize)]
pub struct CreateUserArgs {
    /// User's display name (2-100 characters)
    pub name: String,
    
    /// Email address (must be valid format and unique)
    pub email: String,
    
    /// Initial account status
    /// Set to false for email verification flow
    pub is_active: bool,
}
```

### Function Documentation
```rust
/// Create a new user account with the specified details.
/// 
/// # Arguments
/// * `args` - User creation parameters
/// 
/// # Returns
/// * `Ok(User)` - The created user object with generated ID
/// * `Err(String)` - Error message if validation fails
/// 
/// # Errors
/// - "Email already exists" - If email is not unique
/// - "Invalid email format" - If email validation fails
/// - "Name too short" - If name is less than 2 characters
#[update]
#[icarus_tool("Create a new user account")]
pub fn create_user(args: CreateUserArgs) -> Result<User, String> {
    // Implementation
}
```

## Anti-Patterns to Avoid

### ❌ Too Many Positional Parameters
```rust
// Bad - Confusing parameter order
pub fn create_item(
    name: String,
    description: String,
    category: String,
    price: u64,
    quantity: u32,
    is_active: bool,
    vendor_id: String,
) -> Result<Item, String>
```

### ❌ Unclear Parameter Names
```rust
// Bad - What do these parameters mean?
pub fn process(s1: String, n: u64, f: bool) -> Result<String, String>
```

### ❌ Mixed Naming Conventions
```rust
// Bad - Inconsistent naming
pub struct ConfigArgs {
    apiURL: String,      // Wrong: Should be api_url
    timeout_ms: u32,     // OK
    MaxRetries: u8,      // Wrong: Should be max_retries
}
```

### ❌ Single String for Multiple Values
```rust
// Bad - Parsing required
pub fn create_user(csv_data: String) -> Result<User, String>
// Expects: "name,email,role"

// Good - Structured data
pub fn create_user(args: CreateUserArgs) -> Result<User, String>
```

## Migration Examples

### Before (Positional)
```rust
#[update]
pub fn memorize(
    key: String,
    content: String,
    tags: Vec<String>,
    expires_at: Option<u64>,
) -> Result<String, String>
```

### After (Args Record)
```rust
#[derive(CandidType, Deserialize)]
pub struct MemorizeArgs {
    /// Unique identifier for the memory
    key: String,
    /// Content to store
    content: String,
    /// Categorization tags
    tags: Vec<String>,
    /// Optional expiration timestamp
    expires_at: Option<u64>,
}

#[update]
#[icarus_tool("Store a memory with metadata")]
pub fn memorize(args: MemorizeArgs) -> Result<String, String>
```

## Testing Considerations

When using args records, testing becomes cleaner:

```rust
#[test]
fn test_create_user() {
    let args = CreateUserArgs {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        role: UserRole::Member,
        metadata: None,
        active: true,
    };
    
    let result = create_user(args);
    assert!(result.is_ok());
}
```

## Benefits for AI Integration

This parameter style guide optimizes for AI assistants like Claude by:

1. **Self-Documentation**: Field names provide context without external docs
2. **Type Safety**: Structured types prevent parameter confusion
3. **Consistency**: Predictable patterns make it easier for AI to generate correct calls
4. **Evolution**: Optional fields allow API growth without breaking changes

## Summary

- **0-2 parameters**: Use positional parameters
- **3+ parameters**: Use args records with descriptive field names
- **Configuration**: Always use records for future extensibility
- **Documentation**: Include clear doc comments on complex types
- **Naming**: Follow `{Action}{Resource}Args` pattern with snake_case fields

This approach ensures your Icarus canisters are both human-readable and AI-friendly, solving the Candid UI documentation problem while maintaining clean, maintainable code.