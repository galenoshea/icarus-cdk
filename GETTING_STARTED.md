# Getting Started with Icarus CDK

Welcome to Icarus CDK! This tutorial will guide you through creating your first MCP (Model Context Protocol) server that runs as an Internet Computer Protocol (ICP) canister.

## Prerequisites

Before starting, ensure you have:

- [Rust](https://rustup.rs/) (1.70+ recommended)
- [DFX](https://internetcomputer.org/docs/current/developer-docs/setup/install) (0.15.0+ recommended)
- [Node.js](https://nodejs.org/) (for Claude Desktop integration)

## What is Icarus CDK?

Icarus CDK enables you to create **persistent AI tools** that:

- Run as **ICP canisters** with blockchain-grade security
- Integrate with **Claude Desktop** via the Model Context Protocol (MCP)
- Maintain **state across interactions** using stable memory
- Scale automatically with **pay-per-use** pricing

## Step 1: Install Icarus CLI

```bash
cargo install --git https://github.com/anthgur/icarus-cdk icarus-cli
```

Verify the installation:
```bash
icarus --version
```

## Step 2: Create Your First Project

```bash
icarus new my-calculator
cd my-calculator
```

This creates a project with the following structure:
```
my-calculator/
├── src/
│   └── lib.rs          # Your MCP tools and business logic
├── Cargo.toml          # Dependencies and metadata
├── dfx.json           # ICP deployment configuration
└── .vessel/           # ICP-specific dependencies
```

## Step 3: Understanding the Generated Code

Open `src/lib.rs` to see the generated template:

```rust
use icarus::prelude::*;

// Define your canister's stable storage
stable_storage! {
    memory 0: {
        calculations: Map<String, f64> = Map::init();
    }
}

// Define your MCP module
#[icarus_module]
mod calculator {
    use super::*;

    // Your first MCP tool - accessible from Claude Desktop
    #[icarus_tool("Perform basic arithmetic calculation")]
    pub async fn calculate(operation: String, a: f64, b: f64) -> Result<f64, String> {
        let result = match operation.as_str() {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err("Cannot divide by zero".to_string());
                }
                a / b
            }
            _ => return Err(format!("Unknown operation: {}", operation)),
        };

        // Store the result in stable memory
        STORAGE.with(|s| {
            s.borrow_mut().calculations.insert(
                format!("{} {} {}", a, operation, b),
                result
            )
        });

        Ok(result)
    }

    #[icarus_tool("Get calculation history")]
    pub async fn get_history() -> Result<Vec<String>, String> {
        STORAGE.with(|s| {
            let calculations: Vec<String> = s
                .borrow()
                .calculations
                .iter()
                .map(|(calc, result)| format!("{} = {}", calc, result))
                .collect();
            Ok(calculations)
        })
    }
}
```

## Step 4: Build Your Canister

```bash
icarus build
```

This command:
1. Compiles your Rust code to WebAssembly
2. Generates MCP metadata automatically
3. Validates your tools for MCP compatibility

## Step 5: Deploy Locally

Start a local ICP replica:
```bash
dfx start --clean --background
```

Deploy your canister:
```bash
icarus deploy --network local
```

You'll see output like:
```
✅ Canister deployed successfully!
📋 Canister ID: rdmx6-jaaaa-aaaaa-aaadq-cai
🔗 Candid UI: http://localhost:4943/?canisterId=...
```

## Step 6: Test Your Canister

Test the deployment by calling your tools directly:
```bash
dfx canister call my-calculator list_tools
```

You should see JSON output describing your MCP tools.

## Step 7: Connect to Claude Desktop

### Install the Bridge
The bridge connects Claude Desktop to your canister:

```bash
icarus bridge install
```

### Configure Claude Desktop

Add this to your Claude Desktop MCP settings (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "my-calculator": {
      "command": "icarus",
      "args": ["bridge", "start", "--canister-id", "rdmx6-jaaaa-aaaaa-aaadq-cai"]
    }
  }
}
```

### Restart Claude Desktop

After restarting Claude Desktop, you can now use your calculator:

```
Human: Calculate 15 * 23 using my calculator tool

Claude: I'll use your calculator tool to perform this multiplication.

[Uses calculate tool with operation="multiply", a=15, b=23]

The result of 15 * 23 is 345.
```

## Step 8: Add More Advanced Features

### State Management with Stable Memory

```rust
use icarus::prelude::*;

stable_storage! {
    memory 0: {
        user_preferences: Map<Principal, UserPrefs> = Map::init();
        calculations: Map<String, CalculationRecord> = Map::init();
    }
}

#[derive(IcarusStorable)]
struct UserPrefs {
    precision: u8,
    favorite_operations: Vec<String>,
}

#[derive(IcarusStorable)]
struct CalculationRecord {
    result: f64,
    timestamp: u64,
    user: Principal,
}
```

### Authentication and Authorization

```rust
#[icarus_tool("Get user's calculation history")]
pub async fn get_user_history() -> Result<Vec<String>, String> {
    let caller = ic_cdk::caller();

    // Anonymous principals can't access history
    if caller == Principal::anonymous() {
        return Err("Authentication required".to_string());
    }

    STORAGE.with(|s| {
        let user_calculations: Vec<String> = s
            .borrow()
            .calculations
            .iter()
            .filter(|(_, record)| record.user == caller)
            .map(|(calc, record)| format!("{} = {} ({})", calc, record.result, record.timestamp))
            .collect();
        Ok(user_calculations)
    })
}
```

### Error Handling Best Practices

```rust
#[icarus_tool("Advanced calculation with validation")]
pub async fn advanced_calculate(
    operation: String,
    values: Vec<f64>
) -> Result<f64, String> {
    // Input validation
    if values.is_empty() {
        return Err("At least one value is required".to_string());
    }

    // Operation validation
    let result = match operation.as_str() {
        "sum" => values.iter().sum(),
        "average" => {
            let sum: f64 = values.iter().sum();
            sum / values.len() as f64
        },
        "max" => values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
        "min" => values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
        _ => return Err(format!("Unsupported operation: {}. Available: sum, average, max, min", operation)),
    };

    // Store with metadata
    let record = CalculationRecord {
        result,
        timestamp: ic_cdk::api::time(),
        user: ic_cdk::caller(),
    };

    STORAGE.with(|s| {
        s.borrow_mut().calculations.insert(
            format!("{}({:?})", operation, values),
            record
        )
    });

    Ok(result)
}
```

## Step 9: Deploy to Mainnet

### Create Mainnet Identity

```bash
dfx identity new mainnet-identity
dfx identity use mainnet-identity
```

### Add Cycles

```bash
dfx ledger create-canister $(dfx identity get-principal) --amount 2.0 --network ic
```

### Deploy

```bash
icarus deploy --network ic
```

### Update Claude Desktop Configuration

Update your `claude_desktop_config.json` with the mainnet canister ID:

```json
{
  "mcpServers": {
    "my-calculator": {
      "command": "icarus",
      "args": ["bridge", "start", "--canister-id", "<MAINNET_CANISTER_ID>", "--network", "ic"]
    }
  }
}
```

## Step 10: Monitor and Maintain

### Check Canister Status

```bash
icarus status --canister-id <CANISTER_ID>
```

### Monitor Usage

```bash
dfx canister status <CANISTER_ID> --network ic
```

### Update Your Canister

```bash
# Make changes to your code
icarus build
icarus deploy --network ic --upgrade
```

## Common Patterns and Best Practices

### 1. Parameter Design Strategy

**Simple Functions (1-2 parameters):**
```rust
#[icarus_tool("Get a specific item")]
pub async fn get_item(id: String) -> Result<Item, String> { ... }
```

**Complex Functions (3+ parameters):**
```rust
#[derive(CandidType, Deserialize)]
pub struct CreateItemArgs {
    name: String,
    description: String,
    tags: Vec<String>,
    metadata: Option<HashMap<String, String>>,
}

#[icarus_tool("Create a new item with metadata")]
pub async fn create_item(args: CreateItemArgs) -> Result<String, String> { ... }
```

### 2. State Management Patterns

**Always use stable memory for persistence:**
```rust
stable_storage! {
    memory 0: {
        // Core business data
        items: Map<String, Item> = Map::init();
    },
    memory 1: {
        // User-specific data
        user_sessions: Map<Principal, SessionData> = Map::init();
    },
    memory 2: {
        // Configuration and metadata
        app_config: Cell<AppConfig> = Cell::init(AppConfig::default());
    }
}
```

### 3. Error Handling

**Always return descriptive errors:**
```rust
#[icarus_tool("Process user data")]
pub async fn process_data(data: String) -> Result<ProcessedData, String> {
    // Validate input
    if data.is_empty() {
        return Err("Data cannot be empty".to_string());
    }

    // Process with error context
    let parsed = serde_json::from_str::<RawData>(&data)
        .map_err(|e| format!("Invalid JSON format: {}", e))?;

    // Business logic validation
    if parsed.timestamp < MIN_TIMESTAMP {
        return Err(format!(
            "Timestamp {} is too old, minimum is {}",
            parsed.timestamp,
            MIN_TIMESTAMP
        ));
    }

    Ok(processed_data)
}
```

## Troubleshooting

### Build Issues

**"Cannot find wasm32-unknown-unknown target":**
```bash
rustup target add wasm32-unknown-unknown
```

**"Failed to resolve dependencies":**
```bash
# Clean and rebuild
cargo clean
icarus build
```

### Deployment Issues

**"Insufficient cycles":**
```bash
dfx ledger create-canister $(dfx identity get-principal) --amount 2.0 --network ic
```

**"Canister not found":**
```bash
# Check if dfx is running
dfx start --background
```

### Claude Desktop Integration Issues

**"MCP server not responding":**
```bash
# Check bridge status
icarus bridge status

# Restart bridge
icarus bridge stop
icarus bridge start --canister-id <ID>
```

**"Authentication errors":**
```bash
# Check dfx identity
dfx identity whoami
dfx identity use default
```

## Next Steps

- Explore [advanced examples](./examples/) for more complex patterns
- Read about [MCP protocol details](https://spec.modelcontextprotocol.io/)
- Join the [Icarus community](https://github.com/anthgur/icarus-cdk/discussions)
- Deploy your canister to production and share it!

## Need Help?

- 📖 [Full Documentation](./docs/)
- 🐛 [Report Issues](https://github.com/anthgur/icarus-cdk/issues)
- 💬 [Community Discussions](https://github.com/anthgur/icarus-cdk/discussions)
- 📧 [Email Support](mailto:support@icarus-cdk.com)