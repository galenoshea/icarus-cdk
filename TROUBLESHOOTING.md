# Troubleshooting Guide

This guide helps you diagnose and resolve common issues when developing MCP servers with the Icarus SDK.

## Table of Contents

- [Quick Diagnostics](#quick-diagnostics)
- [Development Issues](#development-issues)
- [Build and Deployment Issues](#build-and-deployment-issues)
- [Runtime Issues](#runtime-issues)
- [Performance Issues](#performance-issues)
- [Network and Connectivity Issues](#network-and-connectivity-issues)
- [Advanced Debugging](#advanced-debugging)
- [Getting Help](#getting-help)

---

## Quick Diagnostics

Start here for a quick health check of your Icarus SDK setup.

### Environment Check

```bash
# Check Rust and toolchain
rustc --version
cargo --version

# Check wasm32 target
rustup target list --installed | grep wasm32

# Check dfx and IC tools
dfx --version

# Check Icarus CLI
icarus --version

# Check project structure
icarus validate --verbose
```

### Common Quick Fixes

1. **Update everything**:
   ```bash
   rustup update
   dfx upgrade
   cargo update
   ```

2. **Clean and rebuild**:
   ```bash
   cargo clean
   icarus build
   ```

3. **Restart local replica**:
   ```bash
   dfx stop
   dfx start --clean --background
   ```

---

## Development Issues

### Compilation Errors

#### "failed to compile icarus-derive"

**Symptoms**: Proc macro compilation fails, attribute errors
```
error: proc-macro derive panicked
  --> src/lib.rs:10:10
   |
10 | #[derive(IcarusStorable)]
```

**Solutions**:
1. Check for missing trait derivations:
   ```rust
   // ❌ Missing required traits
   #[derive(IcarusStorable)]
   struct MyData { ... }

   // ✅ Include all required traits
   #[derive(CandidType, Serialize, Deserialize, Clone, Debug, IcarusStorable)]
   struct MyData { ... }
   ```

2. Verify attribute syntax:
   ```rust
   // ❌ Invalid attribute
   #[icarus_tool("description")]
   fn my_tool() { ... }

   // ✅ Correct canister method
   #[update]
   #[icarus_tool("description")]
   pub async fn my_tool() -> Result<String, String> { ... }
   ```

#### "cannot find macro `stable_storage!`"

**Symptoms**: Stable storage macro not found
```
error: cannot find macro `stable_storage!` in this scope
```

**Solutions**:
1. Add icarus-canister dependency:
   ```toml
   [dependencies]
   icarus-canister = "0.6.0"
   ```

2. Use correct import:
   ```rust
   use icarus_canister::stable_storage;
   // or
   use icarus::prelude::*;
   ```

#### Type Mismatch Errors

**Symptoms**: Complex type errors, trait bound failures
```
error[E0277]: the trait bound `MyType: Storable` is not satisfied
```

**Solutions**:
1. Implement required traits:
   ```rust
   #[derive(CandidType, Serialize, Deserialize, Clone, Debug, IcarusStorable)]
   pub struct MyType {
       pub field: String,
   }
   ```

2. Check size constraints:
   ```rust
   // For large types
   #[derive(CandidType, Serialize, Deserialize, Clone, Debug, IcarusStorable)]
   #[icarus_storable(unbounded)]
   pub struct LargeType { ... }
   ```

### Macro Issues

#### Tool Registration Not Working

**Symptoms**: Tools don't appear in `list_tools`
```rust
#[icarus_tool("My tool")]
pub async fn my_tool() -> Result<String, String> {
    Ok("result".to_string())
}
```

**Solutions**:
1. Add `#[icarus_module]` to containing module:
   ```rust
   #[icarus_module]
   mod my_tools {
       #[update]
       #[icarus_tool("My tool")]
       pub async fn my_tool() -> Result<String, String> {
           Ok("result".to_string())
       }
   }
   ```

2. Ensure proper visibility:
   ```rust
   // ❌ Private function
   #[icarus_tool("Tool")]
   async fn private_tool() -> Result<String, String> { ... }

   // ✅ Public function
   #[update]
   #[icarus_tool("Tool")]
   pub async fn public_tool() -> Result<String, String> { ... }
   ```

---

## Build and Deployment Issues

### WASM Build Failures

#### "error: linking with `rust-lld` failed"

**Symptoms**: Linker errors during WASM compilation
```
error: linking with `rust-lld` failed: exit status: 1
```

**Solutions**:
1. Check target installation:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. Clean and rebuild:
   ```bash
   cargo clean
   cargo build --target wasm32-unknown-unknown --release
   ```

3. Check for incompatible dependencies:
   ```toml
   # ❌ Dependencies with native code
   [dependencies]
   tokio = { version = "1", features = ["rt-multi-thread"] }

   # ✅ WASM-compatible features
   [dependencies]
   tokio = { version = "1", features = ["macros"] }
   ```

#### "binary too large" or Size Issues

**Symptoms**: WASM binary exceeds size limits
```
Warning: WASM binary is 3.2MB, consider optimization
```

**Solutions**:
1. Enable LTO and optimization:
   ```toml
   [profile.release]
   opt-level = "z"  # Optimize for size
   lto = true
   codegen-units = 1
   strip = true
   ```

2. Remove unused dependencies:
   ```bash
   cargo machete  # Find unused dependencies
   ```

3. Use `wasm-opt` tool:
   ```bash
   wasm-opt target/wasm32-unknown-unknown/release/my_canister.wasm \
     -Oz -o optimized.wasm
   ```

### Deployment Failures

#### "canister rejected the message"

**Symptoms**: Deployment fails with rejection
```
Error: The replica returned an HTTP Error: Http Error: status 400 Bad Request
```

**Solutions**:
1. Check canister method signatures:
   ```rust
   // ❌ Invalid signature for canister method
   #[update]
   pub async fn my_method(data: MyStruct) -> String { ... }

   // ✅ Proper error handling
   #[update]
   pub async fn my_method(data: MyStruct) -> Result<String, String> { ... }
   ```

2. Verify Candid interface:
   ```bash
   # Check generated .did file
   icarus build
   cat .icarus/my_canister.did
   ```

3. Check cycles and resource limits:
   ```bash
   dfx canister status my_canister --network local
   ```

#### "dfx deploy" Hangs or Times Out

**Symptoms**: Deployment process hangs indefinitely
```
Installing code for canister my_canister...
[Hanging...]
```

**Solutions**:
1. Restart local replica:
   ```bash
   dfx stop
   dfx start --clean --background
   ```

2. Check for port conflicts:
   ```bash
   lsof -i :4943
   netstat -an | grep 4943
   ```

3. Use explicit network:
   ```bash
   icarus deploy --network local
   ```

---

## Runtime Issues

### MCP Bridge Connection Issues

#### "Bridge failed to connect to canister"

**Symptoms**: MCP bridge can't establish canister connection
```
Error: Failed to connect to canister rdmx6-jaaaa-aaaaa-aaadq-cai
```

**Solutions**:
1. Verify canister is running:
   ```bash
   dfx canister status my_canister --network local
   ```

2. Check canister ID:
   ```bash
   # Get correct canister ID
   dfx canister id my_canister --network local
   ```

3. Restart bridge with debug:
   ```bash
   ICARUS_DEBUG=1 icarus mcp start <canister-id>
   ```

#### "Authentication failed" or Identity Issues

**Symptoms**: Principal/identity authentication failures
```
Error: Principal authentication failed for operations
```

**Solutions**:
1. Check dfx identity:
   ```bash
   dfx identity whoami
   dfx identity get-principal
   ```

2. Switch to correct identity:
   ```bash
   dfx identity use default
   # or create new one
   dfx identity new test-identity
   dfx identity use test-identity
   ```

3. Re-deploy with correct identity:
   ```bash
   icarus deploy --network local
   ```

### Tool Execution Failures

#### "Tool execution timeout"

**Symptoms**: Tools time out during execution
```
Error: Tool execution timed out after 30 seconds
```

**Solutions**:
1. Optimize tool implementation:
   ```rust
   #[update]
   #[icarus_tool("Optimized tool")]
   pub async fn my_tool(input: String) -> Result<String, String> {
       // ❌ Synchronous heavy computation
       let result = heavy_computation(&input);

       // ✅ Break into smaller chunks or use heartbeat
       for chunk in input.chunks(100) {
           process_chunk(chunk).await;
           ic_cdk::api::call::heartbeat(); // Yield execution
       }
       Ok("done".to_string())
   }
   ```

2. Use stable storage for large operations:
   ```rust
   stable_storage! {
       memory 0: {
           processing_state: Cell<ProcessingState> = Cell::init(ProcessingState::default());
       }
   }
   ```

#### "Memory allocation failed"

**Symptoms**: Out of memory errors during execution
```
Error: Canister exceeded memory limit
```

**Solutions**:
1. Use stable memory for large data:
   ```rust
   // ❌ Large data in heap memory
   let mut big_data = Vec::with_capacity(1000000);

   // ✅ Store in stable memory
   stable_storage! {
       memory 0: {
           big_data: Vec<MyData> = Vec::init();
       }
   }
   ```

2. Implement data cleanup:
   ```rust
   #[heartbeat]
   async fn cleanup() {
       // Clean up old data periodically
       STORAGE.with(|s| {
           let mut storage = s.borrow_mut();
           storage.cleanup_old_entries();
       });
   }
   ```

---

## Performance Issues

### Slow Query/Update Performance

#### Identifying Bottlenecks

**Use profiling tools**:
```bash
# Profile canister performance
icarus profile canister <canister-id> --duration 30 --concurrency 10

# Analyze WASM binary
icarus profile analyze --memory --instructions
```

**Profile tool execution**:
```rust
use std::time::Instant;

#[update]
#[icarus_tool("Profiled tool")]
pub async fn my_tool(input: String) -> Result<String, String> {
    let start = Instant::now();

    let result = expensive_operation(&input).await;

    let duration = start.elapsed();
    ic_cdk::println!("Tool execution took: {:?}", duration);

    Ok(result)
}
```

#### Optimization Strategies

1. **Batch operations**:
   ```rust
   // ❌ Multiple individual operations
   for item in items {
       STORAGE.with(|s| s.borrow_mut().insert(item.id, item));
   }

   // ✅ Batch insert
   STORAGE.with(|s| {
       let mut storage = s.borrow_mut();
       for item in items {
           storage.insert(item.id, item);
       }
   });
   ```

2. **Use appropriate data structures**:
   ```rust
   // For ordered data
   use ic_stable_structures::StableBTreeMap;

   // For fast lookup
   use std::collections::HashMap; // In-memory only

   // For large values
   #[derive(IcarusStorable)]
   #[icarus_storable(unbounded)]
   struct LargeData { ... }
   ```

3. **Implement pagination**:
   ```rust
   #[query]
   #[icarus_tool("Get paginated data")]
   pub async fn get_data(offset: u64, limit: u64) -> Result<Vec<MyData>, String> {
       let limit = limit.min(100); // Cap at 100 items
       STORAGE.with(|s| {
           s.borrow()
               .iter()
               .skip(offset as usize)
               .take(limit as usize)
               .map(|(_, v)| v)
               .collect()
       })
   }
   ```

### High Memory Usage

#### Memory Diagnostics

```bash
# Check canister memory usage
dfx canister status my_canister --network local

# Analyze WASM binary size
icarus profile analyze --memory
```

#### Memory Optimization

1. **Use stable memory efficiently**:
   ```rust
   stable_storage! {
       memory 0: {
           // Small, frequently accessed data
           metadata: Map<String, Metadata> = Map::init();
       }
       memory 1: {
           // Large, infrequently accessed data
           archive: Map<String, LargeData> = Map::init();
       }
   }
   ```

2. **Implement data archiving**:
   ```rust
   #[update]
   pub async fn archive_old_data() -> Result<(), String> {
       let cutoff = ic_cdk::api::time() - (30 * 24 * 60 * 60 * 1_000_000_000); // 30 days

       STORAGE.with(|s| {
           let mut storage = s.borrow_mut();
           storage.retain(|_, data| data.timestamp > cutoff);
       });

       Ok(())
   }
   ```

---

## Network and Connectivity Issues

### Local Development Network Issues

#### "Connection refused" to Local Replica

**Symptoms**: Cannot connect to localhost:4943
```
Error: Connection refused (os error 61)
```

**Solutions**:
1. Start local replica:
   ```bash
   dfx start --background --clean
   ```

2. Check port availability:
   ```bash
   lsof -i :4943
   ```

3. Use different port:
   ```bash
   dfx start --host 127.0.0.1:8000 --background
   ```

#### Network Configuration Issues

**Check dfx networks**:
```bash
# View current networks
dfx info networks-json

# Test connectivity
dfx ping --network local
```

**Configure custom network**:
```json
// dfx.json
{
  "networks": {
    "local": {
      "bind": "127.0.0.1:4943",
      "type": "ephemeral"
    },
    "ic": {
      "providers": ["https://ic0.app"],
      "type": "persistent"
    }
  }
}
```

### IC Mainnet Deployment Issues

#### "Insufficient cycles" Errors

**Symptoms**: Deployment fails due to lack of cycles
```
Error: Insufficient cycles: required 2_000_000_000_000, available 500_000_000
```

**Solutions**:
1. Top up canister cycles:
   ```bash
   dfx ledger fabricate-cycles --canister my_canister --amount 3.0
   ```

2. Optimize for lower cycle consumption:
   ```rust
   // Use query instead of update when possible
   #[query]  // Lower cycle cost
   #[icarus_tool("Read-only operation")]
   pub fn read_data(key: String) -> Result<String, String> { ... }

   #[update] // Higher cycle cost
   #[icarus_tool("Write operation")]
   pub fn write_data(key: String, value: String) -> Result<(), String> { ... }
   ```

#### Replica Synchronization Issues

**Symptoms**: Inconsistent state between replicas
```
Error: State inconsistency detected
```

**Solutions**:
1. Use proper state management:
   ```rust
   // ❌ Using thread_local for mutable state
   thread_local! {
       static COUNTER: Cell<u64> = Cell::new(0);
   }

   // ✅ Using stable storage
   stable_storage! {
       memory 0: {
           counter: Cell<u64> = Cell::init(0);
       }
   }
   ```

2. Implement proper upgrade handling:
   ```rust
   #[pre_upgrade]
   fn pre_upgrade() {
       // Save state before upgrade
       STORAGE.with(|s| {
           stable_storage::save_state(s.borrow().deref());
       });
   }

   #[post_upgrade]
   fn post_upgrade() {
       // Restore state after upgrade
       STORAGE.with(|s| {
           let state = stable_storage::load_state();
           *s.borrow_mut() = state;
       });
   }
   ```

---

## Advanced Debugging

### Debug Logging

#### Enable Debug Output

```bash
# CLI debug mode
ICARUS_DEBUG=1 icarus mcp start <canister-id>

# Rust log levels
RUST_LOG=debug cargo test
RUST_LOG=icarus=trace,info cargo run
```

#### Add Debug Prints to Canister

```rust
#[update]
#[icarus_tool("Debug tool")]
pub async fn debug_tool(input: String) -> Result<String, String> {
    ic_cdk::println!("Debug: Received input: {}", input);

    let result = process_input(&input);

    ic_cdk::println!("Debug: Result: {:?}", result);

    Ok(result)
}
```

### Canister Inspection

#### Query Canister State

```rust
#[query]
pub fn debug_state() -> String {
    serde_json::to_string(&DebugInfo {
        memory_usage: ic_cdk::api::performance_counter(0),
        instruction_count: ic_cdk::api::instruction_counter(),
        time: ic_cdk::api::time(),
        caller: ic_cdk::api::caller(),
    }).unwrap_or_else(|e| format!("Error: {}", e))
}

#[derive(Serialize)]
struct DebugInfo {
    memory_usage: u64,
    instruction_count: u64,
    time: u64,
    caller: Principal,
}
```

#### Monitor Canister Metrics

```bash
# Get canister status
dfx canister status my_canister --network local

# Monitor performance
icarus profile canister <canister-id> --duration 60

# Check canister logs
dfx canister logs my_canister
```

### Advanced Profiling

#### Benchmark Specific Operations

```bash
# Run benchmark suite
icarus profile bench --filter "tool_execution"

# Generate performance report
icarus profile bench --html --output results.json
```

#### Custom Performance Metrics

```rust
use std::time::Instant;

#[update]
#[icarus_tool("Monitored tool")]
pub async fn monitored_tool(input: String) -> Result<String, String> {
    let start_instructions = ic_cdk::api::instruction_counter();
    let start_time = Instant::now();

    let result = heavy_computation(&input).await;

    let end_instructions = ic_cdk::api::instruction_counter();
    let duration = start_time.elapsed();
    let instructions_used = end_instructions - start_instructions;

    // Log metrics
    ic_cdk::println!("Performance: {}ms, {} instructions",
        duration.as_millis(), instructions_used);

    Ok(result)
}
```

### Testing and Validation

#### Unit Testing with Debugging

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_with_debug() {
        // Set up test environment
        init_test_storage();

        let input = "test_input".to_string();
        let result = my_tool(input).await;

        assert!(result.is_ok());
        println!("Test result: {:?}", result);

        // Validate state changes
        let state = get_debug_state();
        println!("Final state: {}", state);
    }
}
```

#### Integration Testing

```bash
# Run E2E tests with debugging
RUST_LOG=debug cargo test --test integration_tests -- --nocapture

# Test specific scenarios
cargo test test_tool_execution --release -- --exact
```

---

## Getting Help

### Community Resources

- **GitHub Issues**: [Report bugs and request features](https://github.com/galenoshea/icarus-sdk/issues)
- **Discussions**: [Community discussions and Q&A](https://github.com/galenoshea/icarus-sdk/discussions)
- **Discord**: Join our Discord server for real-time help
- **Documentation**: [Complete SDK documentation](https://docs.rs/icarus)

### Reporting Issues

When reporting issues, please include:

1. **Environment Information**:
   ```bash
   # Run this command and include output
   icarus --version
   rustc --version
   dfx --version
   uname -a
   ```

2. **Minimal Reproduction**:
   - Create minimal example that reproduces the issue
   - Include relevant code snippets
   - Provide exact error messages

3. **Expected vs Actual Behavior**:
   - Clearly describe what you expected to happen
   - Describe what actually happened
   - Include relevant logs or screenshots

### Professional Support

For enterprise users and critical deployments:
- **Priority Support**: Expedited issue resolution
- **Architecture Review**: Best practices consultation
- **Custom Development**: Tailored solutions and extensions
- **Training**: Team training and onboarding

Contact: support@icarus-sdk.dev

---

## Checklist for Common Issues

### Before Seeking Help

- [ ] Updated all tools (`rustup update`, `dfx upgrade`, `cargo update`)
- [ ] Cleaned and rebuilt (`cargo clean && icarus build`)
- [ ] Checked environment with `icarus validate --verbose`
- [ ] Reviewed error messages for obvious issues
- [ ] Searched existing issues and discussions
- [ ] Tried on a minimal reproduction case

### Performance Investigation

- [ ] Profiled with `icarus profile canister`
- [ ] Analyzed WASM binary with `icarus profile analyze`
- [ ] Checked memory usage with `dfx canister status`
- [ ] Reviewed code for obvious bottlenecks
- [ ] Implemented appropriate data structures
- [ ] Used stable storage for persistent data

### Deployment Troubleshooting

- [ ] Verified canister builds locally
- [ ] Checked Candid interface generation
- [ ] Confirmed proper error handling in all methods
- [ ] Tested with local replica first
- [ ] Verified cycles and resource limits
- [ ] Checked network connectivity and configuration

This troubleshooting guide covers the most common issues encountered when developing with the Icarus SDK. Keep it bookmarked for quick reference during development!