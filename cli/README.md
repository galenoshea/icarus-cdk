# Icarus CLI

Command-line tool for creating, building, testing, and deploying Model Context Protocol (MCP) servers to the Internet Computer Protocol (ICP).

## Overview

The Icarus CLI is part of the Icarus ecosystem that enables developers to build and deploy MCP servers as ICP canisters. This provides:

- **Persistent State**: Your MCP servers maintain state across sessions
- **Global Accessibility**: Access your tools from anywhere
- **Blockchain Security**: Benefit from ICP's decentralized infrastructure
- **Easy Deployment**: Simple commands to go from code to deployed canister

## Installation

### From crates.io (Recommended)

```bash
cargo install icarus-cli
```

### From Source

```bash
git clone https://github.com/icarus-mcp/icarus-sdk
cd icarus-sdk/cli
cargo build --release
cargo install --path .
```

## Quick Start

1. **Create a new project**
   ```bash
   icarus new my-mcp-server
   cd my-mcp-server
   ```

2. **Deploy locally**
   ```bash
   icarus deploy --network local
   ```
   Deploys to your local dfx network and returns the canister ID.

4. **Configure AI Clients**
   ```bash
   icarus mcp add <your-canister-id>
   ```
   Interactive selection to add your canister to AI clients (Claude Desktop, ChatGPT Desktop, Claude Code).

5. **Start the bridge** (when needed)
   ```bash
   icarus bridge start --canister-id <your-canister-id>
   ```
   Starts a background bridge service for real-time communication with AI clients.

## Commands

### Project Management

- `icarus new <name>` - Create a new MCP server project
- `icarus test` - Run tests with progressive testing strategy
- `icarus deploy` - Deploy to ICP (local or IC mainnet)

### MCP Client Management

Multi-client support for connecting your canisters to various AI clients:

- `icarus mcp add <canister-id>` - Add canister to AI clients (interactive selection)
- `icarus mcp add <canister-id> --clients claude,chatgpt,claude-code` - Add to specific clients
- `icarus mcp add <canister-id> --config-path <path>` - Use custom configuration path
- `icarus mcp list` - List all client configurations and MCP servers
- `icarus mcp remove <canister-id>` - Remove canister from specific clients (interactive)
- `icarus mcp dashboard` - Interactive MCP status dashboard with system health

### Bridge Management

Background service for real-time communication with AI clients:

- `icarus bridge start --canister-id <id>` - Start bridge for a specific canister (auto-detects dfx identity)
- `icarus bridge status` - Check if bridge is running and show active connections
- `icarus bridge stop` - Stop the running bridge service

### Advanced Commands

- `icarus analyze` - Analyze canister for MCP compatibility
- `icarus generate` - Generate project files or components
- `icarus publish` - Publish your MCP server to the marketplace (coming soon)

### Utilities

- `icarus update` - Self-update the CLI to latest version

## Progressive Testing

The CLI supports a progressive testing strategy:

1. **Level 1**: Unit tests (no blockchain)
2. **Level 2**: Canister tests (local dfx)
3. **Level 3**: MCP protocol tests
4. **Level 4**: Full integration tests

Run specific levels with:
```bash
icarus test --level 2
```

Or run all tests:
```bash
icarus test --all
```

## Authentication

The CLI uses a device authorization flow to connect to the Icarus marketplace:

1. Run a command that requires authentication
2. Visit the provided URL and enter the device code
3. The CLI receives a session token for future use

## Development Workflow

1. **Create**: Use `icarus new` to create a project
2. **Develop**: Write your MCP server logic in Rust
3. **Test**: Use progressive testing to verify functionality
4. **Deploy**: Deploy to local network for testing, then IC mainnet
5. **Publish**: Share on the marketplace (optional)

## Configuration

Configuration is stored in `~/.icarus/config.toml` and includes:
- Session tokens
- Telemetry preferences
- Update check timestamps

## Troubleshooting

### dfx not found
Install dfx from: https://internetcomputer.org/docs/current/developer-docs/setup/install/

### MCP Client Issues
- Use `icarus mcp dashboard` to check client status and configurations
- Verify AI clients are installed and running
- Check configuration paths with `icarus mcp list`
- Use `--config-path` flag for custom installation locations

### Bridge connection issues
- Check bridge status with `icarus bridge status`
- Verify canister ID is correct and deployed
- Ensure dfx identity has access to the canister
- Check if port is available (default: random port)

### Build failures
- Ensure Rust is up to date
- Check that wasm32-unknown-unknown target is installed
- Verify all dependencies in Cargo.toml

## Contributing

This is a proprietary project. For bug reports and feature requests, please contact the Icarus team.

## Related Projects

- **icarus-sdk**: Open source SDK for building MCP servers
- **icarus-app**: NFT marketplace for discovering and purchasing MCP servers

## License

This software is licensed under the Business Source License 1.1 (BSL 1.1).

### Permitted Uses
✅ **You CAN:**
- Use the CLI to develop and deploy your own MCP tools
- Integrate the CLI into your own applications  
- Use the CLI for internal business purposes
- Modify the CLI for your own use (non-competitive)
- Use for personal, educational, and research purposes
- Deploy MCP tools to your own canisters
- Create commercial MCP tools using this CLI

### Prohibited Uses
❌ **You CANNOT:**
- Operate a competing MCP tool marketplace service
- Offer MCP deployment services that compete with Icarus Marketplace
- Create derivative works that provide a competing marketplace
- Resell or redistribute the CLI as a commercial service

### License Conversion
On **January 1, 2029**, this software will automatically convert to the Apache License 2.0, making it fully open source.

### Commercial License
For uses not permitted under the BSL, please contact the Icarus team for a commercial license.

Copyright © 2025 Icarus Team. All rights reserved.

See the [LICENSE](./LICENSE) file for the full license text.