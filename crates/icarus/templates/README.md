# Icarus SDK Templates

This directory contains template files demonstrating how to build MCP servers on the Internet Computer using the Icarus SDK.

## Usage

These templates are designed to be copied into your IC project's `src/` directory and deployed as canisters using `dfx`.

```bash
# Create your IC project
dfx new my_mcp_server
cd my_mcp_server

# Copy a template to your src directory
cp /path/to/icarus/templates/basic_calculator.rs src/my_mcp_server_backend/src/lib.rs

# Update dfx.json to include canister configuration

# Start local Internet Computer
dfx start --background

# Deploy the canister
dfx deploy
```

**Note**: These are **template files**, not Cargo examples. They require IC canister deployment via `dfx`, not `cargo run`.

---

## Templates

### 1. Basic Calculator (`basic_calculator.rs`)

**Difficulty**: Beginner
**Topics**: Tool definition, basic types, error handling

A simple calculator demonstrating the fundamentals of creating MCP tools.

**Features**:
- Four basic arithmetic operations (add, subtract, multiply, divide)
- Error handling for edge cases (division by zero)
- Minimal boilerplate with `#[tool]` macro
- Comprehensive unit tests

**Learning Objectives**:
- How to define tools with `#[tool]` macro
- Basic parameter types (`f64`, `i32`, `String`)
- Return values and error handling
- Testing tool functions

**Run**:
```bash
dfx deploy basic_calculator

# Test addition
dfx canister call basic_calculator call_tool '(
  record {
    name = "add";
    arguments = "{\"a\": 5.0, \"b\": 3.0}"
  }
)'
```

---

### 2. Async HTTP Tools (`async_http_tools.rs`)

**Difficulty**: Intermediate
**Topics**: HTTP outcalls, async/await, external APIs, JSON parsing

Demonstrates how to fetch data from external APIs using Internet Computer's HTTP outcalls feature.

**Features**:
- Async tool functions with `async fn`
- HTTP GET requests to real APIs
- JSON parsing and validation
- Error handling for network failures
- Integration with CoinGecko, IPGeolocation APIs

**Learning Objectives**:
- How to make HTTP outcalls from canisters
- Async/await patterns in IC canisters
- Parsing external JSON responses
- Handling network errors gracefully
- Managing cycles for HTTP requests

**Run**:
```bash
dfx deploy async_http_tools

# Get Bitcoin price
dfx canister call async_http_tools call_tool '(
  record {
    name = "get_btc_price";
    arguments = "{}"
  }
)'

# Lookup IP information
dfx canister call async_http_tools call_tool '(
  record {
    name = "get_ip_info";
    arguments = "{\"ip\": \"8.8.8.8\"}"
  }
)'
```

**Important**: HTTP outcalls require cycles. Make sure your canister has sufficient cycles balance.

---

### 3. Stateful Counter (`stateful_counter.rs`)

**Difficulty**: Intermediate
**Topics**: State management, thread-local storage, persistence, canister upgrades

Shows how to manage persistent and volatile state in Internet Computer canisters.

**Features**:
- Thread-local state with `RefCell`
- Multiple independent counters
- Atomic increment/decrement operations
- State inspection and reset
- Named counter collections

**Learning Objectives**:
- Thread-local storage patterns
- Mutable state with `RefCell`
- State persistence strategies
- Canister upgrade handling
- When to use volatile vs stable memory

**Run**:
```bash
dfx deploy stateful_counter

# Increment global counter
dfx canister call stateful_counter call_tool '(
  record {
    name = "increment";
    arguments = "{}"
  }
)'

# Get current value
dfx canister call stateful_counter call_tool '(
  record {
    name = "get_count";
    arguments = "{}"
  }
)'

# Use named counter
dfx canister call stateful_counter call_tool '(
  record {
    name = "increment_named";
    arguments = "{\"name\": \"visits\"}"
  }
)'
```

---

## Example Comparison Matrix

| Example | Complexity | Async | HTTP Outcalls | State Management | Best For |
|---------|------------|-------|---------------|------------------|----------|
| **basic_calculator** | ⭐ | No | No | None | Learning basics |
| **async_http_tools** | ⭐⭐ | Yes | Yes | None | External APIs |
| **stateful_counter** | ⭐⭐ | No | No | Thread-local | State patterns |

---

## Development Workflow

### 1. Local Development

```bash
# Start local replica
dfx start --background

# Deploy your canister
dfx deploy

# Test tools
dfx canister call <canister-name> list_tools

# Call a tool
dfx canister call <canister-name> call_tool '(
  record {
    name = "tool_name";
    arguments = "{\"param\": \"value\"}"
  }
)'
```

### 2. Testing

Each example includes unit tests:

```bash
cargo test --example basic_calculator
cargo test --example async_http_tools
cargo test --example stateful_counter
```

### 3. Integration with AI Clients

Once deployed, connect your canister to AI clients:

```bash
# Get canister ID
dfx canister id <canister-name>

# Start Icarus bridge (if using CLI)
icarus mcp add <canister-id>
```

Then your AI assistant (Claude, ChatGPT) can call your tools directly!

---

## Common Patterns

### Pattern 1: Simple Synchronous Tool

```rust
use icarus_macros::tool;

#[tool("Description of what the tool does")]
fn my_tool(param: String) -> String {
    format!("Result: {}", param)
}
```

### Pattern 2: Async Tool with HTTP Outcall

```rust
use icarus_macros::tool;

#[tool("Fetches external data")]
async fn fetch_data(url: String) -> Result<String, String> {
    let request = ic_cdk::api::management_canister::http_request::HttpRequestArgs {
        url,
        method: ic_cdk::api::management_canister::http_request::HttpMethod::GET,
        headers: vec![],
        body: None,
        max_response_bytes: Some(1024),
        transform: None,
    };

    let (response,) = ic_cdk::api::management_canister::http_request::http_request(request)
        .await
        .map_err(|e| format!("HTTP request failed: {:?}", e))?;

    String::from_utf8(response.body)
        .map_err(|e| format!("Invalid UTF-8: {}", e))
}
```

### Pattern 3: Stateful Tool

```rust
use icarus_macros::tool;
use std::cell::RefCell;

thread_local! {
    static STATE: RefCell<u64> = RefCell::new(0);
}

#[tool("Updates and returns state")]
fn update_state(new_value: u64) -> u64 {
    STATE.with(|state| {
        let mut s = state.borrow_mut();
        *s = new_value;
        *s
    })
}
```

---

## Next Steps

1. **Read the Migration Guide**: See [MIGRATION.md](../MIGRATION.md) for upgrading from older versions
2. **Review the Main README**: Check [README.md](../../README.md) for full SDK documentation
3. **Explore Advanced Features**: Authentication, stable memory, timers, and more
4. **Build Your Own**: Start with these examples and customize for your use case

---

## Troubleshooting

### "Failed to compile WASM"

Ensure you have the correct Rust target installed:

```bash
rustup target add wasm32-unknown-unknown
```

### "Canister has no update method 'call_tool'"

Make sure you've included the `icarus_macros::mcp! {}` macro at the bottom of your file.

### "HTTP request failed: Rejected"

HTTP outcalls require cycles. Top up your canister:

```bash
dfx ledger fabricate-cycles --canister <canister-id>
```

### More Help

See the full [Troubleshooting Guide](../TROUBLESHOOTING.md) for common issues and solutions.

---

## Contributing

Found a bug or want to add an example? Contributions welcome!

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

---

**Happy Building! 🚀**