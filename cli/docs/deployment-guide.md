# Deployment Guide

Complete guide for deploying Icarus MCP servers to ICP.

## Overview

Deployment involves compiling your Rust code to WebAssembly, optimizing it, and deploying it to either a local ICP network or the IC mainnet. This guide covers all deployment scenarios.

## Prerequisites

- Icarus CLI installed
- Project created with `icarus new`
- dfx installed (via Icarus CLI)
- ICP tokens for mainnet deployment

## Local Development Deployment

### 1. Start Local Network

The CLI handles this automatically, but you can also start manually:

```bash
dfx start --clean
```

### 2. Build Your Project

```bash
icarus build
```

This:
- Compiles Rust to WASM
- Generates Candid interface
- Optimizes WASM size
- Prepares deployment artifacts

### 3. Deploy Locally

```bash
icarus deploy --network local
```

Output:
```
Deploying to local
✓ Build completed!
✓ Deployed successfully! 🎉

URLs:
  Backend canister via Candid interface:
    my-server: http://127.0.0.1:4943/?canisterId=be2us-64aaa-aaaaa-qaabq-cai&id=bkyz2-fmaaa-aaaaa-qaaaq-cai

Canister ID: bkyz2-fmaaa-aaaaa-qaaaq-cai
```

### 4. Test Your Deployment

Using dfx:
```bash
dfx canister call my-server get_metadata '()'
```

Using the bridge:
```bash
icarus bridge start --canister-id bkyz2-fmaaa-aaaaa-qaaaq-cai
```

## Mainnet Deployment

### 1. Get ICP Tokens

You need ICP tokens for:
- Canister creation (cycles)
- Storage and compute costs

Options:
- Buy ICP on exchanges
- Use ICP faucet (testnet)
- Transfer from another wallet

### 2. Configure Identity

```bash
# Create new identity
dfx identity new myidentity
dfx identity use myidentity

# Get your principal
dfx identity get-principal

# Check balance
dfx ledger balance
```

### 3. Convert ICP to Cycles

```bash
# Create cycles wallet
dfx ledger create-canister $(dfx identity get-principal) --amount 1.0

# Top up cycles
dfx ledger top-up --amount 0.5 <wallet-canister-id>
```

### 4. Deploy to Mainnet

```bash
icarus deploy --network ic --cycles 1000000000000
```

Options:
- `--cycles`: Initial cycles (1T cycles = ~$1.30)
- `--compute-allocation`: Guaranteed compute (0-100)
- `--memory-allocation`: Reserved memory

### 5. Verify Deployment

```bash
# Check canister status
dfx canister status my-server --network ic

# Test functionality
dfx canister call my-server get_metadata '()' --network ic
```

## Deployment Options

### Compute Allocation

Reserve guaranteed compute:
```bash
icarus deploy --network ic --compute-allocation 10
```

- 0 = Best effort (default)
- 1-100 = Guaranteed percentage

### Memory Allocation

Reserve memory upfront:
```bash
icarus deploy --network ic --memory-allocation 2GB
```

Benefits:
- Predictable costs
- No allocation failures
- Better performance

### Freezing Threshold

Set cycles threshold for freezing:
```bash
icarus deploy --network ic --freezing-threshold 30_days
```

Canister freezes when cycles would last < threshold.

## Upgrade Deployments

### Upgrading Existing Canister

Instead of fresh deployment:
```bash
icarus deploy --upgrade --network ic
```

This:
- Preserves canister ID
- Maintains stable storage
- Keeps existing data
- Updates code only

### Pre-upgrade Checks

Before upgrading:
1. Test upgrade locally
2. Backup important data
3. Check storage compatibility
4. Review breaking changes

### Safe Upgrade Process

```bash
# 1. Deploy to test canister
icarus deploy --network ic --canister-id <test-id>

# 2. Verify functionality
icarus analyze --canister-id <test-id>

# 3. Upgrade production
icarus deploy --upgrade --network ic --canister-id <prod-id>
```

## Cycle Management

### Monitor Cycles

```bash
# Check cycle balance
dfx canister status my-server --network ic

# Output includes:
# Cycles: 3_456_789_012_345
```

### Top Up Cycles

```bash
# Add cycles to canister
dfx canister deposit-cycles 1000000000000 my-server --network ic
```

### Cycle Estimation

Typical costs:
- Storage: ~$5/GB/year
- Compute: ~$0.50/million executions
- Ingress messages: Free
- Inter-canister calls: ~$0.001 each

## Multi-Canister Deployment

### Deploy Multiple Canisters

Create `deploy.json`:
```json
{
  "canisters": {
    "backend": {
      "type": "rust",
      "path": "src/backend"
    },
    "storage": {
      "type": "rust", 
      "path": "src/storage"
    }
  }
}
```

Deploy all:
```bash
icarus deploy --config deploy.json --network ic
```

### Inter-Canister Communication

```rust
// Call another canister
let other_canister = Principal::from_text("...").unwrap();
let result: (String,) = ic_cdk::call(
    other_canister,
    "method_name",
    (arg1, arg2)
).await?;
```

## Environment-Specific Configuration

### Development vs Production

Use environment variables:
```rust
#[cfg(debug_assertions)]
const NETWORK: &str = "local";

#[cfg(not(debug_assertions))]
const NETWORK: &str = "ic";
```

### Feature Flags

Enable features per environment:
```toml
[features]
default = []
mainnet = ["production-logging", "cycle-monitoring"]
testnet = ["debug-mode", "test-data"]
```

Deploy with features:
```bash
icarus deploy --network ic --features mainnet
```

## Deployment Automation

### GitHub Actions

`.github/workflows/deploy.yml`:
```yaml
name: Deploy to IC
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Icarus
        run: |
          curl -L https://icarus.dev/install.sh | sh
          
      - name: Deploy
        env:
          DFX_IDENTITY: ${{ secrets.DFX_IDENTITY }}
        run: |
          echo "$DFX_IDENTITY" | base64 -d > identity.pem
          dfx identity import ci identity.pem
          dfx identity use ci
          icarus deploy --network ic
```

### Deployment Scripts

`scripts/deploy.sh`:
```bash
#!/bin/bash
set -e

# Build
echo "Building project..."
icarus build --release

# Run tests
echo "Running tests..."
icarus test --all

# Deploy
echo "Deploying to mainnet..."
icarus deploy --network ic --cycles 5000000000000

# Verify
echo "Verifying deployment..."
icarus analyze --canister-id $(cat .dfx/ic/canister_ids.json | jq -r .my_server.ic)
```

## Monitoring & Maintenance

### Health Checks

Regular monitoring script:
```bash
#!/bin/bash
CANISTER_ID="your-canister-id"

# Check cycles
CYCLES=$(dfx canister status $CANISTER_ID --network ic | grep Cycles | awk '{print $2}')
echo "Cycles remaining: $CYCLES"

# Test endpoint
RESPONSE=$(dfx canister call $CANISTER_ID get_metadata '()' --network ic 2>&1)
if [[ $RESPONSE == *"error"* ]]; then
    echo "ERROR: Canister not responding"
    exit 1
fi
```

### Backup Strategy

```rust
#[update]
#[icarus_tool("Export all data for backup")]
pub fn export_backup() -> BackupData {
    BackupData {
        timestamp: ic_cdk::api::time(),
        users: USERS.with(|u| u.borrow().iter().collect()),
        config: CONFIG.with(|c| c.borrow().get()),
    }
}
```

## Troubleshooting

### Common Issues

#### "Insufficient cycles"
```bash
# Check wallet balance
dfx wallet balance --network ic

# Top up canister
dfx canister deposit-cycles 10000000000000 my-server --network ic
```

#### "Canister trapped"
- Check error in canister logs
- Review recent code changes
- Test locally first

#### "Replica returned an error"
- Network issues
- Try different IC URL
- Wait and retry

### Debug Deployment

```bash
# Verbose output
icarus deploy --network ic --verbose

# Check canister logs
dfx canister logs my-server --network ic
```

## Best Practices

1. **Test Locally First**: Always deploy to local network before mainnet
2. **Monitor Cycles**: Set up alerts for low cycles
3. **Use Version Control**: Tag deployments in git
4. **Document Changes**: Maintain deployment changelog
5. **Gradual Rollout**: Test on small subset first
6. **Backup Data**: Before major upgrades
7. **Security Audit**: Before mainnet deployment

## Cost Optimization

### Reduce Storage Costs
- Compress data before storing
- Clean up old data regularly
- Use efficient data structures

### Optimize Compute
- Use `#[query]` for read operations
- Batch operations when possible
- Cache frequent computations

### Example Optimization
```rust
// Before: Individual updates
for item in items {
    store_item(item);
}

// After: Batch update
store_items_batch(items);
```

This reduces inter-canister calls and compute costs.