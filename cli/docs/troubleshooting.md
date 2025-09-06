# Troubleshooting Guide

Solutions for common issues when using the Icarus CLI.

## Installation Issues

### Command Not Found

After installation, if `icarus` command is not found:

**Problem**: Binary not in PATH
```bash
# Check if installed
ls /usr/local/bin/icarus

# Check PATH
echo $PATH
```

**Solution**:
```bash
# Add to PATH (add to ~/.bashrc or ~/.zshrc)
export PATH="$PATH:/usr/local/bin"

# Reload shell
source ~/.bashrc
```

### Permission Denied During Installation

**Problem**: Can't write to /usr/local/bin
```
Error: Permission denied (os error 13)
```

**Solution**:
```bash
# Use sudo
sudo mv icarus /usr/local/bin/

# Or install to user directory
mkdir -p ~/.local/bin
mv icarus ~/.local/bin/
export PATH="$PATH:$HOME/.local/bin"
```

### SSL Certificate Errors

**Problem**: Certificate verification failed
```
Error: error sending request: error trying to connect: invalid certificate
```

**Solution**:
```bash
# Update certificates
# macOS
brew install ca-certificates

# Ubuntu/Debian
sudo apt-get update && sudo apt-get install ca-certificates

# If behind corporate proxy
export NODE_TLS_REJECT_UNAUTHORIZED=0  # Not recommended for production
```

## Build Issues

### WASM Target Not Found

**Problem**: 
```
error[E0463]: can't find crate for `std`
note: the `wasm32-unknown-unknown` target may not be installed
```

**Solution**:
```bash
# Install WASM target
rustup target add wasm32-unknown-unknown
```

### Out of Memory During Build

**Problem**: Build process killed or very slow

**Solution**:
```bash
# Increase memory for build
export CARGO_BUILD_JOBS=2  # Reduce parallel jobs

# Or use release mode
icarus build --release
```

### Dependency Conflicts

**Problem**: Version conflicts in Cargo.toml
```
error: failed to select a version for `ic-cdk`
```

**Solution**:
```bash
# Update dependencies
cargo update

# Or specify exact versions
[dependencies]
ic-cdk = "=0.16.0"
candid = "=0.10.0"
```

## Deployment Issues

### DFX Not Found

**Problem**: 
```
Error: dfx command not found
```

**Solution**:
```bash
# Install dfx
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"

# Verify installation
dfx --version
```

### Local Network Not Starting

**Problem**: 
```
Error: Failed to start local replica
```

**Solution**:
```bash
# Kill existing dfx processes
dfx stop
pkill dfx

# Start clean
dfx start --clean

# Check if port is in use
lsof -i :4943
```

### Insufficient Cycles

**Problem**:
```
Error: Insufficient cycles to create canister
```

**Solution**:
```bash
# For local development (free cycles)
dfx start --clean

# For mainnet
dfx wallet balance
dfx wallet send <amount> <canister-id>
```

### Canister Installation Failed

**Problem**:
```
Error: Failed to install canister code
```

**Solution**:
```bash
# Check WASM size
ls -lh .dfx/local/canisters/*/**.wasm

# If too large, optimize
icarus build --release

# Clear cache and retry
rm -rf .dfx
icarus deploy --network local
```

## Bridge Issues

### Bridge Won't Start

**Problem**:
```
Error: Failed to start bridge service
```

**Solution**:
```bash
# Check if already running
ps aux | grep icarus-bridge

# Kill existing process
pkill -f "icarus bridge"

# Start with debug logging
RUST_LOG=debug icarus bridge start --canister-id <id>
```

### Connection to Canister Failed

**Problem**:
```
Error: Failed to connect to canister
```

**Solution**:
```bash
# Verify canister is running
dfx canister status <canister-id>

# Check network
dfx ping

# Try explicit network
icarus bridge start --canister-id <id> --ic-host http://localhost:4943
```

### Claude Desktop Can't Connect

**Problem**: Claude Desktop shows connection error

**Solution**:
1. Verify bridge is running:
```bash
icarus bridge status
```

2. Check Claude Desktop config:
```json
{
  "mcpServers": {
    "my-server": {
      "command": "/usr/local/bin/icarus",
      "args": ["bridge", "start", "--canister-id", "<id>"]
    }
  }
}
```

3. Test manually:
```bash
# Run bridge in foreground
icarus bridge start --canister-id <id>
```

### Type Conversion Errors

**Problem**:
```
Error: Failed to convert arguments: invalid type
```

**Solution**:
- Check tool parameter types match Candid interface
- Ensure optional values are properly wrapped
- Verify array types are homogeneous

## Runtime Issues

### Canister Trapped

**Problem**:
```
Error: Canister trapped: heap out of bounds
```

**Solution**:
```bash
# Check canister logs
dfx canister logs <canister-name>

# Common causes:
# 1. Index out of bounds
# 2. Unwrap on None
# 3. Integer overflow

# Debug locally first
icarus test --level 2
```

### State Not Persisting

**Problem**: Data lost after upgrade

**Solution**:
```rust
// Ensure using stable storage
stable_storage! {
    DATA: StableBTreeMap<String, Data, Memory> = memory_id!(0);
}

// Not regular variables
// static mut DATA: HashMap<String, Data> = HashMap::new(); // Wrong!
```

### Performance Issues

**Problem**: Slow responses from canister

**Solution**:
1. Use `#[query]` for read operations
2. Optimize data structures
3. Add pagination for large results
4. Monitor cycle consumption

## Testing Issues

### Tests Timing Out

**Problem**:
```
Error: Test exceeded timeout of 60s
```

**Solution**:
```bash
# Increase timeout
RUST_TEST_TIME_LIMIT=120 cargo test

# Or run specific test
icarus test --filter specific_test_name
```

### PocketIC Connection Failed

**Problem**: Integration tests can't connect

**Solution**:
```bash
# Update PocketIC
cargo update -p pocket-ic

# Clear test cache
cargo clean
```

## Common Error Messages

### "Actor reference is not a function"

**Cause**: Wrong canister ID or method name
**Fix**: Verify canister ID and check method exists

### "Replica returned an error"

**Cause**: Network issues or canister error
**Fix**: Check logs and retry

### "Variant field does not exist"

**Cause**: Candid type mismatch
**Fix**: Regenerate types with `dfx generate`

### "Call was rejected"

**Cause**: Authorization failure
**Fix**: Check caller principal and permissions

## Debug Techniques

### Enable Verbose Logging

```bash
# CLI logging
RUST_LOG=debug icarus build

# Bridge logging
RUST_LOG=icarus_bridge=debug icarus bridge start --canister-id <id>

# Canister logging
ic_cdk::println!("Debug: {:?}", value);
```

### Check Intermediate Files

```bash
# View generated Candid
cat src/*.did

# Check WASM size
du -h .dfx/local/canisters/*/*.wasm

# View metadata
dfx canister call <name> list_tools '()'
```

### Test in Isolation

```bash
# Test single function
cargo test test_function_name -- --nocapture

# Test with print output
RUST_LOG=debug cargo test
```

## Getting Help

### Collect Diagnostics

When reporting issues, include:

```bash
# Version info
icarus --version
dfx --version
rustc --version

# System info
uname -a

# Error logs
RUST_LOG=debug icarus <command> 2>&1 | tee error.log
```

### Resources

1. **GitHub Issues**: https://github.com/icarus-mcp/icarus-cli/issues
2. **Discord Community**: https://discord.gg/icarus
3. **Stack Overflow**: Tag with `icarus-mcp`
4. **Email Support**: support@icarus.dev

### Emergency Recovery

If all else fails:

```bash
# Full reset
rm -rf ~/.icarus
rm -rf .dfx
dfx start --clean

# Reinstall
curl -L https://icarus.dev/install.sh | sh

# Start fresh
icarus new test-project
cd test-project
icarus build
icarus deploy --network local
```