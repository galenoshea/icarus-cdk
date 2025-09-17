<div align="center">

# 🚀 Icarus SDK

**Build persistent AI tools that run forever on the blockchain**

[![Crates.io](https://img.shields.io/crates/v/icarus.svg)](https://crates.io/crates/icarus)
[![Documentation](https://docs.rs/icarus/badge.svg)](https://docs.rs/icarus)
[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](LICENSE)
[![CI](https://github.com/galenoshea/icarus-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/galenoshea/icarus-sdk/actions/workflows/ci.yml)
[![Coverage](https://github.com/galenoshea/icarus-sdk/actions/workflows/coverage.yml/badge.svg)](https://github.com/galenoshea/icarus-sdk/actions/workflows/coverage.yml)
[![Release](https://github.com/galenoshea/icarus-sdk/actions/workflows/release.yml/badge.svg)](https://github.com/galenoshea/icarus-sdk/actions/workflows/release.yml)

[Quick Start](#-quick-start) • [Docs](https://docs.rs/icarus) • [Examples](examples/) • [Contributing](CONTRIBUTING.md)

</div>

---

## ✨ Why Icarus?

Traditional MCP servers are ephemeral - they lose state when restarted. **Icarus changes that.**

By combining the **Model Context Protocol** (MCP) with the **Internet Computer Protocol** (ICP), Icarus enables you to build AI tools that:

- 🔄 **Persist Forever** - No more lost state between sessions
- 🌐 **Run Globally** - Deploy once, accessible from anywhere
- 🔒 **Stay Secure** - Built-in blockchain security and authentication
- 💰 **Cost Pennies** - ICP's reverse gas model means predictable, low costs
- ⚡ **Scale Instantly** - Automatic scaling with canister architecture
- 🌍 **HTTP Outcalls** - Fetch external data from any API
- ⏰ **Autonomous Operations** - Schedule tasks with built-in timers

### 📊 Comparison

| Feature | Traditional MCP | Icarus MCP |
|---------|----------------|------------|
| **State Persistence** | ❌ Lost on restart | ✅ Permanent storage |
| **Deployment** | Manual server management | One command to ICP |
| **Global Access** | Requires hosting setup | Built-in global CDN |
| **Cost Model** | Pay for hosting | Pay per computation |
| **Authentication** | Build your own | Internet Identity built-in |

---

## 🎯 Perfect For

- **🤖 AI Assistants** - Build Claude/ChatGPT tools with persistent memory
- **📊 Data Tools** - Analytics and monitoring that never forget
- **🎮 Game Backends** - Persistent game state and player data
- **💼 Enterprise Tools** - Secure, auditable business automation
- **🔬 Research Tools** - Long-running experiments and data collection

---

## 🚀 Quick Start

### Installation

```bash
# Install the CLI
cargo install icarus-cli

# Create a new project
icarus new my-ai-tool
cd my-ai-tool

# Deploy to ICP
icarus deploy
```

### Add to Your Project

```toml
[dependencies]
# Recommended: Simple, includes everything for canister development
icarus = "0.8.0"

# Or specify features explicitly
icarus = { version = "0.7.0", features = ["canister"] }

# Other required dependencies for canister development
ic-cdk = "0.18"
candid = "0.10"
serde = { version = "1.0", features = ["derive"] }
```

### Your First Persistent Tool

```rust
use icarus::prelude::*;

#[icarus_module]
mod tools {
    // This memory persists forever on the blockchain
    stable_storage! {
        MEMORIES: StableBTreeMap<String, String, Memory> = memory_id!(0);
    }
    
    #[update]
    #[icarus_tool("Store a memory that lasts forever")]
    pub fn remember(key: String, value: String) -> Result<String, String> {
        MEMORIES.with(|m| m.borrow_mut().insert(key, value));
        Ok("Memory stored permanently! 🎉".to_string())
    }
    
    #[query]
    #[icarus_tool("Recall a memory from any session")]
    pub fn recall(key: String) -> Result<String, String> {
        MEMORIES.with(|m| 
            m.borrow()
                .get(&key)
                .ok_or_else(|| "Memory not found".to_string())
        )
    }
}
```

### Connect to AI Clients

```bash
# Add your deployed canister to AI clients (interactive selection)
icarus mcp add <your-canister-id>

# Or specify clients directly
icarus mcp add <your-canister-id> --clients claude,chatgpt,claude-code

# Start MCP server for testing (foreground mode)
icarus mcp start <your-canister-id>

# Or run in daemon mode (background)
icarus mcp start <your-canister-id> --daemon

# Now your AI clients have persistent memory! 🧠
```

---

## 📦 Project Structure

```
icarus/
├── 🧩 icarus-core        # Core MCP protocol implementation
├── 🔮 icarus-derive      # Procedural macros for less boilerplate
├── 📦 icarus-canister    # ICP canister integration
├── 🛠️  icarus-cli         # Command-line tools
└── 📚 examples/          # Ready-to-deploy examples
```

---

## 🌟 Features

### 🔧 Developer Experience

- **Zero Boilerplate** - Macros generate all the MCP metadata
- **Intelligent Parameter Translation** - Seamless JSON to Candid conversion for any parameter style
- **Type Safety** - Full Rust type checking and IDE support
- **Hot Reload** - Local development with instant feedback
- **Rich CLI** - Project scaffolding, deployment, and management

### 🌍 HTTP Outcalls

```rust
use icarus::prelude::*;

// Fetch any external API with one line
let data = http::get("https://api.example.com/data").await?;

// POST JSON with automatic serialization
let response = http::post_json(url, json!({
    "user": "alice",
    "action": "subscribe"
})).await?;

// Built-in retry logic and error handling
let config = HttpConfig {
    max_retries: 5,
    timeout_seconds: 30,
    ..Default::default()
};
let result = http::get_with_config(url, config).await?;
```

### ⏰ Autonomous Timers

```rust
use icarus::prelude::*;

// Schedule one-time tasks
let cleanup = timers::schedule_once(3600, "hourly-cleanup", || {
    // This runs after 1 hour
    cleanup_old_data();
})?;

// Create recurring tasks
let heartbeat = timers::schedule_periodic(300, "health-check", || {
    // This runs every 5 minutes forever
    check_system_health();
})?;

// Manage timers dynamically
timers::cancel_timer(cleanup)?;
let active = timers::list_active_timers();
```

### 💾 Stable Storage

```rust
// Your data structures
#[derive(IcarusStorable)]
struct UserProfile {
    id: String,
    preferences: HashMap<String, String>,
    created_at: u64,
}

// Automatic persistence with stable storage
stable_storage! {
    USERS: StableBTreeMap<String, UserProfile, Memory> = memory_id!(0);
    SETTINGS: StableVec<Settings, Memory> = memory_id!(1);
}
```

### 🔐 Built-in Security

- **Internet Identity** - Secure authentication out of the box
- **Principal-based Access** - Fine-grained permissions
- **Candid Interface** - Type-safe client generation

---

## 🏗️ Architecture & Crates

Icarus SDK is organized into focused crates for modularity and flexibility:

### Core Crates
- **[`icarus`](crates/icarus/)** - Main SDK with all features bundled
- **[`icarus-core`](crates/icarus-core/)** - Core types, traits, and session management
- **[`icarus-derive`](crates/icarus-derive/)** - Proc macros (`#[icarus_module]`, `#[icarus_tool]`)
- **[`icarus-canister`](crates/icarus-canister/)** - ICP integration and stable storage

### Infrastructure Crates
- **[`icarus-bridge`](crates/icarus-bridge/)** - MCP-to-ICP bridge for AI client communication
- **[`icarus-dev`](crates/icarus-dev/)** - Development tools, file watching, and project management
- **[`icarus-mcp`](crates/icarus-mcp/)** - MCP protocol implementation and networking

### Modular Usage

```toml
# Use the full SDK (recommended)
icarus = "0.8.0"

# Or pick specific crates for specialized use cases
icarus-bridge = "0.7.0"   # For custom bridge implementations
icarus-dev = "0.7.0"      # For development tooling
icarus-mcp = "0.7.0"      # For MCP protocol work
```

---

## 📚 Examples

Check out our [examples directory](examples/) for complete, deployable projects:

- **[Memory Assistant](examples/basic-memory/)** - Persistent note-taking for AI
- **[GitHub Integration](examples/github-tool/)** - Repository management tool
- **[Data Analytics](examples/analytics/)** - Time-series data storage

---

## 🛠️ CLI Commands

```bash
# Project Management
icarus new <name>           # Create a new project
icarus deploy              # Deploy to ICP (builds automatically)
icarus test                # Run tests

# MCP Client Management (Multi-Client Support)
icarus mcp add <id>         # Add canister to AI clients (Claude Desktop, ChatGPT, Claude Code)
icarus mcp list             # List all client configurations and servers
icarus mcp remove <id>      # Remove canister from specific clients
icarus mcp dashboard        # Interactive MCP status dashboard
icarus mcp start <id>       # Start MCP server for canister (foreground/daemon mode)

# Bridge Commands (Background Service)
icarus bridge start <id>   # Start bridge for canister (auto-detects identity)
icarus bridge status       # Check bridge status
icarus bridge stop         # Stop running bridge

# Development
icarus dev                 # Start local development
icarus logs <id>          # View canister logs
```

---

## 🔄 Migration from 0.6.x to 0.7.0

**New Architecture**: Icarus has been refactored into focused crates for better modularity.

### What's New in 0.7.0:
- 🏗️ **Modular Architecture**: Split into 6 focused crates (`icarus-bridge`, `icarus-dev`, etc.)
- 🧪 **Enhanced Testing**: 74+ new unit tests for improved reliability
- 🛠️ **Better Development**: Enhanced dev tools and file watching
- 🌉 **Improved Bridge**: More robust MCP-to-ICP communication

### To upgrade:
1. Update your dependency: `icarus = "0.8.0"`
2. Redeploy: `icarus deploy` (builds automatically)

**No breaking changes** - existing code continues to work unchanged. The modular architecture is internal - your application code remains the same.

---

## 📖 Documentation

- **[Getting Started Guide](docs/getting-started.md)** - Step-by-step tutorial
- **[Parameter Translation](docs/parameter-translation.md)** - How MCP JSON maps to ICP Candid
- **[Parameter Style Guide](docs/parameter-style-guide.md)** - Best practices for function parameters
- **[API Documentation](https://docs.rs/icarus)** - Complete API reference
- **[Architecture Overview](docs/architecture.md)** - How Icarus works
- **[Migration Guide](docs/migration-guide.md)** - Migrate existing MCP servers

---

## 🤝 Contributing

We welcome contributions! See our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/galenoshea/icarus-sdk
cd icarus-sdk

# Install dependencies
./scripts/install-deps.sh

# Run tests
cargo test

# Build everything
cargo build --all
```

### Updating Candid Interfaces

When you modify your tool functions in an Icarus project, you need to update the Candid interface (.did file) to reflect the changes:

```bash
# Install the generate-did tool (one-time setup)
cargo install candid-extractor
cargo install generate-did

# After modifying your tools, rebuild and update the .did file
cargo build --target wasm32-unknown-unknown --release
generate-did .

# The .did file is automatically updated with all your tool functions
```

The `#[icarus_module]` macro automatically generates all the necessary endpoints, and `ic_cdk::export_candid!()` embeds the interface in your WASM for extraction.

---

## 💬 Community & Support

- **[Discord](https://discord.gg/icarus)** - Join our community
- **[GitHub Issues](https://github.com/galenoshea/icarus-sdk/issues)** - Report bugs
- **[Discussions](https://github.com/galenoshea/icarus-sdk/discussions)** - Ask questions

---

## 📄 License

Icarus SDK is licensed under the Business Source License 1.1 (BSL). See [LICENSE](LICENSE) for details.

The BSL allows you to use Icarus SDK for developing and deploying MCP tools to the Icarus Marketplace.

---

<div align="center">
  
**Built with ❤️ by the Icarus Team**

[Website](https://icarus.ai) • [Twitter](https://twitter.com/icarusai) • [Blog](https://blog.icarus.ai)

</div>