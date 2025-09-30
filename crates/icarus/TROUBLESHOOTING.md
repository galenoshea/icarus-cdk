# Troubleshooting Guide

This guide helps you diagnose and resolve common issues when using the Icarus SDK.

## Table of Contents
- [Build Issues](#build-issues)
- [Deployment Issues](#deployment-issues)
- [Runtime Issues](#runtime-issues)
- [HTTP Outcall Issues](#http-outcall-issues)
- [State Management Issues](#state-management-issues)
- [Feature Flag Issues](#feature-flag-issues)
- [Performance Issues](#performance-issues)
- [Getting Help](#getting-help)

---

## Build Issues

### Error: "Can't find crate for `std`"

**Symptom**:
```
error[E0463]: can't find crate for `std`
```

**Cause**: Missing WASM target for Rust compiler.

**Solution**:
```bash
rustup target add wasm32-unknown-unknown
```

**Verify**:
```bash
rustup target list --installed | grep wasm32
# Should show: wasm32-unknown-unknown
```

---

### Error: "Failed to compile WASM binary"

**Symptom**:
```
Error: Failed to compile WASM binary
Caused by: cargo build failed
```

**Common Causes**:
1. Incompatible dependencies
2. Missing `ic-cdk` dependencies
3. Incorrect Cargo.toml configuration

**Solution**:

**Check dependencies**:
```toml
[dependencies]
ic-cdk = "0.18"
ic-cdk-macros = "0.18"
candid = "0.10"
icarus = "0.9.0"
```

**Clean build**:
```bash
cargo clean
rm -rf target/
cargo build --target wasm32-unknown-unknown --release
```

**Check for missing features**:
```toml
[lib]
crate-type = ["cdylib"]
```

---

### Error: "unresolved import `icarus::prelude`"

**Symptom**:
```
error[E0432]: unresolved import `icarus::prelude`
```

**Cause**: Incorrect version or missing dependency.

**Solution**:

**Update Cargo.toml**:
```toml
[dependencies]
icarus = "0.9.0"
```

**Update imports**:
```rust
use icarus::prelude::*;
// NOT: use icarus_canister::prelude::*;
```

**Rebuild**:
```bash
cargo update
cargo clean
cargo build
```

---

### Warning: "unused `#[macro_use]` import"

**Symptom**:
```
warning: unused `#[macro_use]` import
```

**Cause**: Old-style macro imports (pre-0.9.0).

**Solution**:

**Before** (0.8.x):
```rust
#[macro_use]
extern crate icarus;
```

**After** (0.9.0):
```rust
use icarus::prelude::*;
```

---

## Deployment Issues

### Error: "Canister has no update method 'call_tool'"

**Symptom**:
```
The Replica returned an error: code 3, message: "Canister has no update method 'call_tool'"
```

**Cause**: Missing `mcp! {}` macro.

**Solution**:

**Add to your canister code**:
```rust
use icarus_macros::tool;

#[tool("My tool")]
fn my_tool(param: String) -> String {
    format!("Result: {}", param)
}

// Required: Generate MCP endpoints
icarus_macros::mcp! {}
```

**Rebuild and redeploy**:
```bash
dfx build
dfx deploy
```

---

### Error: "Canister installation failed"

**Symptom**:
```
Error: Failed to install canister
Caused by: The replica returned an error: code 5, message: "Canister installation failed"
```

**Common Causes**:
1. WASM binary too large (>2MB)
2. Out of cycles
3. Invalid Candid interface

**Solutions**:

**1. Optimize WASM size**:
```toml
# Cargo.toml
[profile.release]
opt-level = 'z'
lto = true
codegen-units = 1
strip = true
```

**2. Check cycles**:
```bash
dfx ledger --network ic balance
dfx canister status <canister-id>
```

**3. Verify Candid**:
```bash
candid-extractor target/wasm32-unknown-unknown/release/<canister>.wasm > src/<canister>.did
```

---

### Error: "Identity not found"

**Symptom**:
```
Error: Identity 'default' not found
```

**Cause**: dfx identity not configured.

**Solution**:
```bash
# Create default identity
dfx identity new default

# Use it
dfx identity use default

# Verify
dfx identity whoami
```

---

## Runtime Issues

### Error: "Principal is not authorized"

**Symptom**:
```
Error: Principal ryjl3-tyaaa-aaaaa-aaaba-cai is not authorized
```

**Cause**: Authentication required but not configured.

**Solution**:

**Check if you're using auth**:
```rust
// If your code has:
icarus_macros::mcp! {
    .auth()  // ← Authentication enabled
    .build()
}
```

**Initialize canister with admin principal**:
```bash
# Get your principal
dfx identity get-principal

# Deploy with init argument
dfx deploy --argument '(principal "YOUR-PRINCIPAL-HERE")'
```

**Or disable auth for testing**:
```rust
icarus_macros::mcp! {}  // No .auth() call
```

---

### Error: "Tool not found"

**Symptom**:
```
Error: Tool 'my_tool' not found
```

**Common Causes**:
1. Tool not registered
2. Incorrect tool name
3. Function not marked with `#[tool]`

**Solution**:

**Verify tool is registered**:
```bash
dfx canister call <canister-name> list_tools
```

**Check tool name**:
```rust
#[tool("exact-name")]  // Tool name is "exact-name"
fn my_function() -> String { ... }

// Or without custom name
#[tool("Description")]  // Tool name is "my_function"
fn my_function() -> String { ... }
```

**Ensure macro is applied**:
```rust
#[tool("Description")]  // ← This macro is required
fn my_tool() -> String { ... }
```

---

### Error: "Failed to decode arguments"

**Symptom**:
```
Error: Failed to decode arguments
```

**Cause**: Mismatch between provided JSON and expected parameters.

**Solution**:

**Check tool schema**:
```bash
dfx canister call <canister-name> list_tools
```

**Match parameter names exactly**:
```rust
#[tool("Example")]
fn example(first_name: String, age: i32) -> String { ... }

// Call with:
{
  "first_name": "John",  // Must match parameter name
  "age": 30
}

// NOT:
{
  "firstName": "John",  // ← Wrong: camelCase
  "age": 30
}
```

**Check parameter types**:
```json
// Correct types:
{
  "string_param": "text",
  "number_param": 42,
  "bool_param": true,
  "optional_param": null
}
```

---

## HTTP Outcall Issues

### Error: "HTTP request failed: OutOfCycles"

**Symptom**:
```
Error: HTTP request failed: OutOfCycles
```

**Cause**: Canister doesn't have enough cycles for HTTP outcalls.

**Solution**:

**Check canister balance**:
```bash
dfx canister status <canister-id>
```

**Top up cycles**:

**Local development**:
```bash
dfx ledger fabricate-cycles --canister <canister-id>
```

**Mainnet**:
```bash
# Transfer cycles from your wallet
dfx canister deposit-cycles <amount> <canister-id>
```

**Estimate HTTP outcall costs**:
- Base cost: ~49M cycles
- Cost per request byte: ~400 cycles
- Cost per response byte: ~800 cycles
- Typical request: 50-100M cycles

---

### Error: "HTTP request timeout"

**Symptom**:
```
Error: HTTP request failed: Timeout
```

**Common Causes**:
1. External API is slow
2. Network connectivity issues
3. Transform function too complex

**Solutions**:

**1. Increase timeout** (if API is known to be slow):
```rust
// Note: IC has max timeout limits
let request = HttpRequestArgs {
    url: url.to_string(),
    method: HttpMethod::GET,
    headers: vec![],
    body: None,
    max_response_bytes: Some(2048),
    transform: None,
};
```

**2. Use faster APIs** or add caching:
```rust
thread_local! {
    static CACHE: RefCell<HashMap<String, (String, u64)>> = RefCell::new(HashMap::new());
}
```

**3. Simplify transform function**:
```rust
fn transform_response(args: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: args.response.status,
        headers: vec![],  // Remove variable headers
        body: args.response.body,
    }
}
```

---

### Error: "Consensus failure"

**Symptom**:
```
Error: HTTP request failed: ConsensusFailure
```

**Cause**: HTTP responses differ across replicas (non-deterministic).

**Solution**:

**Add transform function**:
```rust
#[ic_cdk::query]
fn transform_http_response(args: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: args.response.status,
        headers: vec![],  // Remove variable headers like timestamps
        body: args.response.body,
    }
}

// Use in request:
let request = HttpRequestArgs {
    url: url.to_string(),
    method: HttpMethod::GET,
    headers: vec![],
    body: None,
    max_response_bytes: Some(2048),
    transform: Some(TransformContext {
        function: TransformFunc(candid::Func {
            principal: ic_cdk::api::id(),
            method: "transform_http_response".to_string(),
        }),
        context: vec![],
    }),
};
```

**Remove variable data**:
- Timestamps in headers
- Request IDs
- Server-specific headers
- Rate limit counters

---

## State Management Issues

### Issue: "State lost after canister upgrade"

**Symptom**: State resets to initial values after `dfx deploy` or `dfx canister install`.

**Cause**: Using thread-local storage without persistence.

**Solution**:

**Option 1: Use Stable Memory**:
```rust
use ic_stable_structures::{StableBTreeMap, DefaultMemoryImpl};

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = ...;
    static STATE: RefCell<StableBTreeMap<String, u64, _>> = ...;
}
```

**Option 2: Implement pre/post-upgrade hooks**:
```rust
use std::cell::RefCell;

thread_local! {
    static STATE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    let state = STATE.with(|s| s.borrow().clone());
    ic_cdk::storage::stable_save((state,))
        .expect("Failed to save state");
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    let (state,): (HashMap<String, String>,) = ic_cdk::storage::stable_restore()
        .expect("Failed to restore state");
    STATE.with(|s| *s.borrow_mut() = state);
}
```

---

### Error: "RefCell already borrowed"

**Symptom**:
```
thread panicked at 'already borrowed: BorrowMutError'
```

**Cause**: Attempting to mutably borrow a `RefCell` that's already borrowed.

**Solution**:

**Bad** (causes panic):
```rust
STATE.with(|state| {
    let borrowed = state.borrow();  // Immutable borrow
    let mut mut_borrowed = state.borrow_mut();  // ← Panic!
    *mut_borrowed = new_value;
});
```

**Good** (proper scoping):
```rust
STATE.with(|state| {
    let value = {
        let borrowed = state.borrow();
        borrowed.clone()  // Clone while borrowed
    };  // ← Borrow dropped here

    let mut mut_borrowed = state.borrow_mut();  // Now safe
    *mut_borrowed = process(value);
});
```

---

## Feature Flag Issues

### Error: "no method named `auth` found"

**Symptom**:
```
error[E0599]: no method named `auth` found for struct `McpBuilder`
```

**Cause**: Missing feature flag in dependencies.

**Solution**:

**Enable required features**:
```toml
[dependencies]
icarus = { version = "0.9.0", features = ["auth"] }
```

**Available features**:
- `auth` - Authentication and authorization
- `http` - HTTP outcalls
- `timers` - Scheduled tasks
- `storage` - Stable storage utilities

---

## Performance Issues

### Issue: "Canister response is slow"

**Symptoms**:
- Long response times (>2 seconds)
- Timeout errors
- High cycle consumption

**Common Causes**:
1. Large computation in tool function
2. Too many HTTP outcalls
3. Inefficient state access
4. Large response payloads

**Solutions**:

**1. Use async for I/O operations**:
```rust
#[tool("Fetch data")]
async fn fetch_data() -> Result<String, String> {
    // Use async for HTTP outcalls
    let data = http_get(url).await?;
    Ok(data)
}
```

**2. Implement caching**:
```rust
thread_local! {
    static CACHE: RefCell<HashMap<String, CachedValue>> = RefCell::new(HashMap::new());
}

#[tool("Get cached data")]
fn get_data(key: String) -> Result<String, String> {
    CACHE.with(|cache| {
        if let Some(cached) = cache.borrow().get(&key) {
            if !cached.is_expired() {
                return Ok(cached.value.clone());
            }
        }
        // Fetch if not cached or expired
        fetch_and_cache(key)
    })
}
```

**3. Limit response size**:
```rust
#[tool("Get large data")]
fn get_data() -> Result<String, String> {
    let data = fetch_large_dataset();

    // Truncate or paginate
    if data.len() > 10_000 {
        return Ok(data[..10_000].to_string());
    }

    Ok(data)
}
```

**4. Use stable structures efficiently**:
```rust
// Bad: Clone entire large structure
let all_data = STORAGE.with(|s| s.borrow().clone());

// Good: Query specific item
let item = STORAGE.with(|s| s.borrow().get(&key).cloned());
```

---

### Issue: "Out of cycles"

**Symptom**: Canister stops responding or returns "OutOfCycles" errors.

**Solution**:

**Monitor cycles**:
```bash
dfx canister status <canister-id>
```

**Set up cycle monitoring**:
```rust
#[ic_cdk::update]
fn check_cycles() -> u64 {
    ic_cdk::api::canister_balance()
}

#[ic_cdk::update]
fn alert_low_cycles() {
    let balance = ic_cdk::api::canister_balance();
    if balance < 1_000_000_000_000 {  // < 1T cycles
        ic_cdk::api::trap("Low cycles!");
    }
}
```

**Auto top-up** (requires cycles wallet):
```rust
// Implement auto top-up logic or use monitoring service
```

---

## Getting Help

### Before Asking for Help

1. **Check this troubleshooting guide** for your specific error
2. **Read the relevant documentation**:
   - [README.md](../../README.md) - Main documentation
   - [MIGRATION.md](./MIGRATION.md) - Version upgrade guide
   - [examples/](./examples/) - Code examples
3. **Search existing issues**: [GitHub Issues](https://github.com/getodk/icarus-sdk/issues)
4. **Try the examples**: Build and run the provided examples to verify your setup

### When Asking for Help

**Include**:
1. **Versions**:
   ```bash
   rustc --version
   dfx --version
   cat Cargo.toml | grep icarus
   ```

2. **Error messages**: Complete error output, not screenshots

3. **Minimal reproducible example**: Smallest code that demonstrates the issue

4. **What you've tried**: Steps already attempted to fix the issue

5. **Environment**: Local development, IC mainnet, or testnet

### Where to Get Help

1. **GitHub Issues**: [github.com/getodk/icarus-sdk/issues](https://github.com/getodk/icarus-sdk/issues)
   - Bug reports
   - Feature requests
   - General questions

2. **Internet Computer Forum**: [forum.dfinity.org](https://forum.dfinity.org)
   - IC-specific questions
   - Deployment issues
   - HTTP outcall problems

3. **Documentation**: [docs.rs/icarus](https://docs.rs/icarus)
   - API reference
   - Type documentation
   - Function examples

---

## Common Error Messages Quick Reference

| Error | Common Cause | Quick Fix |
|-------|-------------|-----------|
| "Can't find crate for `std`" | Missing WASM target | `rustup target add wasm32-unknown-unknown` |
| "Canister has no update method" | Missing `mcp! {}` macro | Add `icarus_macros::mcp! {}` to your code |
| "Principal is not authorized" | Authentication required | Deploy with principal argument or disable auth |
| "OutOfCycles" | Insufficient cycles | Top up canister cycles |
| "HTTP request timeout" | Slow external API | Add caching or use faster API |
| "RefCell already borrowed" | Nested borrows | Scope borrows properly |
| "State lost after upgrade" | Using thread-local without persistence | Use stable memory or upgrade hooks |
| "Tool not found" | Missing `#[tool]` macro | Add macro to function |
| "Failed to decode arguments" | Parameter name/type mismatch | Match JSON to parameter names exactly |

---

**Last Updated**: 2025-09-29
**Version**: 0.9.0