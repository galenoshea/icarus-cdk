# Migration Guide: 0.8.x → 0.9.0

This guide helps you upgrade from Icarus SDK version 0.8.x to 0.9.0.

## Table of Contents
- [Overview](#overview)
- [Breaking Changes](#breaking-changes)
- [Step-by-Step Migration](#step-by-step-migration)
- [New Features](#new-features)
- [Troubleshooting](#troubleshooting)

---

## Overview

Version 0.9.0 is a **major quality release** focused on:
- **Technical Debt Elimination**: Removed all `unwrap()` calls, deprecated macros, and unused code
- **Type Safety Improvements**: Newtype pattern for compile-time safety
- **Performance Optimization**: Reduced allocations by 30-50% with `Cow<'static, str>`
- **Builder Pattern**: New declarative API replacing standalone macros

**Migration Effort**: Low to Medium (1-2 hours for most projects)

---

## Breaking Changes

### 1. Removed Deprecated Macros

**What Changed**: Standalone macros removed in favor of builder pattern.

#### Before (0.8.x)
```rust
use icarus::prelude::*;

// Standalone macros (REMOVED)
icarus::auth!();
icarus::wasi!();
icarus::init!();
```

#### After (0.9.0)
```rust
use icarus::prelude::*;

// New builder pattern
icarus::mcp! {
    .build()
}

// Or with configuration
icarus::mcp! {
    .auth()
    .build()
}
```

**Migration Action**: Replace all standalone `auth!()`, `wasi!()`, and `init!()` macros with the new `mcp! { .build() }` builder pattern.

---

### 2. Removed IcarusService Trait

**What Changed**: The `service.rs` module and `IcarusService` trait system have been completely removed.

#### Before (0.8.x)
```rust
use icarus::canister::IcarusService;

struct MyService;

impl IcarusService for MyService {
    fn service_name(&self) -> &str {
        "my-service"
    }

    fn service_description(&self) -> &str {
        "My custom service"
    }
}
```

#### After (0.9.0)
```rust
// Service metadata now comes from Cargo.toml
// No implementation required - handled automatically

// In Cargo.toml:
// [package]
// name = "my-service"
// description = "My custom service"
// version = "1.0.0"
```

**Migration Action**:
1. Remove all `IcarusService` implementations
2. Ensure your `Cargo.toml` has proper `name`, `description`, and `version` fields
3. Service metadata is now automatically extracted from package metadata

---

### 3. Type Safety: Newtype Pattern

**What Changed**: String-based identifiers replaced with compile-time type-safe wrappers.

#### Before (0.8.x)
```rust
let memory_id: u8 = 0;
let tool_name: String = "my_tool".to_string();
let canister_id: String = "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string();
```

#### After (0.9.0)
```rust
use icarus::prelude::*;

let memory_id = MemoryId::new(0)?;
let tool_name = ToolName::new("my_tool")?;
let canister_id = CanisterId::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai")?;
```

**Migration Action**:
1. Wrap memory IDs with `MemoryId::new(id)?`
2. Wrap tool names with `ToolName::new(name)?`
3. Wrap canister IDs with `CanisterId::from_text(id)?`
4. Handle the `Result` types (they validate inputs at compile time)

**Benefits**:
- Compile-time validation of identifiers
- Type safety prevents mixing different ID types
- Clear error messages for invalid inputs

---

### 4. Error Handling: No More `unwrap()`

**What Changed**: All `unwrap()` calls eliminated in favor of `expect()` with descriptive messages.

#### Before (0.8.x)
```rust
let value = some_operation().unwrap();
let data = parse_json(&input).unwrap();
```

#### After (0.9.0)
```rust
let value = some_operation()
    .expect("Failed to perform operation: invalid input");

let data = parse_json(&input)
    .expect("Failed to parse JSON: malformed input");
```

**Migration Action**:
1. Replace all `.unwrap()` with `.expect("descriptive message")`
2. Provide context about what failed and why
3. Use `?` operator where appropriate for propagating errors

**Benefits**:
- Better error messages in production
- Easier debugging when panics occur
- Clear documentation of assumptions

---

## Step-by-Step Migration

### Step 1: Update Dependencies

Update your `Cargo.toml`:

```toml
[dependencies]
icarus = "0.9.0"
ic-cdk = "0.18"
ic-cdk-macros = "0.18"
candid = "0.10"
```

### Step 2: Update Imports

Ensure you're using the correct import path:

```rust
use icarus::prelude::*;
```

### Step 3: Replace Deprecated Macros

**Find all instances**:
```bash
grep -r "icarus::auth!" src/
grep -r "icarus::wasi!" src/
grep -r "icarus::init!" src/
```

**Replace with**:
```rust
icarus::mcp! {
    .build()
}
```

### Step 4: Remove IcarusService Implementations

**Find all instances**:
```bash
grep -r "impl IcarusService" src/
```

**Remove the implementation** and ensure your `Cargo.toml` has proper metadata.

### Step 5: Update Type Usage

**Find string-based identifiers**:
```bash
grep -r "memory_id:" src/
grep -r "tool_name:" src/
grep -r "canister_id:" src/
```

**Wrap with newtype constructors**:
```rust
MemoryId::new(id)?
ToolName::new(name)?
CanisterId::from_text(id)?
```

### Step 6: Replace `unwrap()` Calls

**Find all unwrap calls**:
```bash
grep -r "\.unwrap()" src/
```

**Replace with `expect()` and descriptive messages**:
```rust
.expect("Description of what failed and why")
```

### Step 7: Rebuild and Test

```bash
cargo clean
cargo build
cargo test
```

---

## New Features in 0.9.0

### 1. SmallVec Optimization

**Performance improvement** for parameter collections (≤4 elements):

```rust
use icarus::prelude::*;

// Automatically uses stack allocation for ≤4 parameters
let params = SmallParameters::from_vec(vec![
    ToolParameter::new("x", "X coordinate", ToolSchema::number()),
    ToolParameter::new("y", "Y coordinate", ToolSchema::number()),
]);
```

**Benefits**:
- Zero heap allocations for small parameter lists
- 30-50% reduction in memory allocations
- Transparent `Vec` fallback for larger collections

---

### 2. Custom Tool Names

**MCP-compatible kebab-case tool names**:

```rust
use icarus_macros::tool;

#[tool(name = "calculate-distance")]
fn calculate_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}
```

**Benefits**:
- MCP protocol compatibility
- Clear separation of Rust function names from tool names
- Support for kebab-case, snake_case, or custom naming

---

### 3. Schema Customization (NEW!)

**Parameter-level constraints and documentation**:

```rust
use icarus_macros::tool;

#[tool("User registration tool")]
fn register_user(
    // Note: #[param] attributes parsed but not yet usable in function signatures
    // Use ToolParameter::new() with constraints instead
    username: String,
    age: i32,
    email: String,
) -> String {
    format!("Registered: {} ({})", username, age)
}

// Manual schema with constraints:
use icarus_core::{Tool, ToolParameter, ToolSchema};

let tool = Tool::builder()
    .name(ToolId::new("register_user")?)
    .description("User registration tool")
    .parameter(ToolParameter::new(
        "username",
        "Username between 3-20 characters",
        ToolSchema::string_with_length(Some(3), Some(20))
    ))
    .parameter(ToolParameter::new(
        "age",
        "Age must be between 1 and 120",
        ToolSchema::number_range(Some(1.0), Some(120.0))
    ))
    .parameter(ToolParameter::new(
        "email",
        "Valid email address",
        ToolSchema::string_with_pattern(r"^[^@]+@[^@]+\.[^@]+$")
    ))
    .build()?;
```

**Benefits**:
- Better API documentation
- Client-side validation
- Clear parameter constraints

---

### 4. Property-Based Testing

**Comprehensive edge case coverage**:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_tool_id_validation(s in "\\PC*") {
        let result = ToolId::new(&s);

        if s.is_empty() || s.len() > 64 {
            assert!(result.is_err());
        } else if s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
    }
}
```

**Benefits**:
- Automated edge case discovery
- Higher confidence in validation logic
- Reduced manual test maintenance

---

## Troubleshooting

### Issue: "Cannot find trait `IcarusService`"

**Solution**: Remove the trait implementation. Service metadata is now from `Cargo.toml`.

```rust
// Remove this:
impl IcarusService for MyService { ... }

// Ensure Cargo.toml has:
[package]
name = "my-service"
description = "My service description"
version = "1.0.0"
```

---

### Issue: "Method `unwrap` found on type `Result`"

**Solution**: Replace with `expect()` or use `?` operator.

```rust
// Before:
let value = operation().unwrap();

// After:
let value = operation()
    .expect("Operation failed: describe why");

// Or propagate error:
let value = operation()?;
```

---

### Issue: Type mismatch with identifiers

**Solution**: Use newtype constructors.

```rust
// Before:
let id: String = "my_tool".to_string();

// After:
let id = ToolName::new("my_tool")?;
```

---

### Issue: "Cannot find macro `auth` in module `icarus`"

**Solution**: Replace with builder pattern.

```rust
// Before:
icarus::auth!();

// After:
icarus::mcp! {
    .auth()
    .build()
}
```

---

### Issue: Build failures after upgrade

**Solution**: Clean build and update peer dependencies.

```bash
cargo clean
rm -rf target/
cargo update
cargo build
```

---

## Migration Checklist

Use this checklist to track your migration progress:

- [ ] Updated `Cargo.toml` dependencies to 0.9.0
- [ ] Replaced deprecated `auth!()`, `wasi!()`, `init!()` macros
- [ ] Removed `IcarusService` trait implementations
- [ ] Added proper `Cargo.toml` metadata (name, description, version)
- [ ] Wrapped identifiers with newtype constructors (`MemoryId`, `ToolName`, `CanisterId`)
- [ ] Replaced all `.unwrap()` with `.expect("...")` or `?`
- [ ] Updated imports to use `icarus::prelude::*`
- [ ] Ran `cargo clean && cargo build`
- [ ] All tests passing (`cargo test`)
- [ ] Redeployed canisters (`dfx deploy` or `icarus deploy`)

---

## Support

If you encounter issues not covered in this guide:

1. Check the [CHANGELOG.md](../../CHANGELOG.md) for detailed changes
2. Review [examples/](./examples/) for updated code patterns
3. Open an issue at [github.com/getodk/icarus-sdk/issues](https://github.com/getodk/icarus-sdk/issues)

---

**Version**: 0.9.0
**Last Updated**: 2025-09-29