# MCP Client Management

This document provides comprehensive guidance for managing MCP (Model Context Protocol) client configurations with the Icarus CLI.

## Overview

The Icarus CLI supports multiple AI clients, allowing you to deploy your MCP servers to various platforms:

- **🤖 Claude Desktop** - Anthropic's desktop client with MCP support
- **💬 ChatGPT Desktop** - OpenAI's desktop client (when MCP support is added)
- **🎨 Claude Code/Cline** - VS Code extension for Claude integration

## Quick Start

### Basic Usage

```bash
# Interactive client selection
icarus mcp add your-canister-id

# Add to specific clients
icarus mcp add your-canister-id --clients claude,chatgpt,claude-code

# View all configurations
icarus mcp list

# Interactive dashboard
icarus mcp dashboard
```

## Commands Reference

### `icarus mcp add <canister-id>`

Add a canister to one or more AI clients with interactive selection or direct specification.

**Options:**
- `--clients <list>` - Specify clients directly (comma-separated: `claude,chatgpt,claude-code`)
- `--config-path <path>` - Custom configuration file path
- `--name <name>` - Custom server name (defaults to canister ID)

**Examples:**
```bash
# Interactive selection with beautiful UI
icarus mcp add rdmx6-jaaaa-aaaah-qcaiq-cai

# Add to specific clients
icarus mcp add rdmx6-jaaaa-aaaah-qcaiq-cai --clients claude,claude-code

# Custom configuration path
icarus mcp add rdmx6-jaaaa-aaaah-qcaiq-cai --config-path ~/.config/claude/custom_config.json

# Custom server name
icarus mcp add rdmx6-jaaaa-aaaah-qcaiq-cai --name "My AI Tool"
```

### `icarus mcp list`

Display all client configurations with a beautiful tree view showing:
- Client installation status
- Configured MCP servers
- Configuration file paths
- Server count per client

**Example Output:**
```
🌳 MCP Server Configuration Tree
├── 🤖 Claude Desktop
│   ├── 🚀 my-ai-tool
│   └── 🚀 analytics-server
├── 💬 ChatGPT Desktop
│   └── (no servers configured)
└── 🎨 Claude Code/Cline
    ├── 🚀 my-ai-tool
    └── 🚀 development-server
```

### `icarus mcp remove <canister-id>`

Remove a canister from specific AI clients with interactive selection.

**Options:**
- `--clients <list>` - Specify clients to remove from
- `--all` - Remove from all configured clients

**Examples:**
```bash
# Interactive removal
icarus mcp remove rdmx6-jaaaa-aaaah-qcaiq-cai

# Remove from specific clients
icarus mcp remove rdmx6-jaaaa-aaaah-qcaiq-cai --clients claude,chatgpt

# Remove from all clients
icarus mcp remove rdmx6-jaaaa-aaaah-qcaiq-cai --all
```

### `icarus mcp dashboard`

Launch an interactive dashboard showing:
- System health status
- Client installation status
- MCP server configurations
- Performance metrics
- Troubleshooting recommendations

**Features:**
- Real-time status updates
- Beautiful progress bars and animations
- Configuration validation
- Quick access to common tasks

### `icarus mcp start <canister-id>`

Start an MCP server for a specific canister, allowing AI clients to connect and use your tools.

**Options:**
- `--daemon` - Run in background/daemon mode (default: foreground)

**Examples:**
```bash
# Start MCP server in foreground mode (for development/testing)
icarus mcp start rdmx6-jaaaa-aaaah-qcaiq-cai

# Start in daemon mode (background)
icarus mcp start rdmx6-jaaaa-aaaah-qcaiq-cai --daemon
```

**How it Works:**
- **Foreground Mode**: Server runs in the current terminal, perfect for development and testing
- **Daemon Mode**: Server runs in the background, suitable for production deployments
- **Auto-Detection**: Automatically detects if being run by an AI client via stdio
- **Identity Management**: Uses current `dfx` identity for canister authentication

**Integration with AI Clients:**
When you add a canister using `icarus mcp add`, the configuration automatically uses `icarus mcp start` as the command, creating a seamless workflow:

1. Deploy your canister: `icarus deploy`
2. Add to AI clients: `icarus mcp add <canister-id>`
3. The AI client will automatically run `icarus mcp start <canister-id>` when connecting

## Client-Specific Configuration

### Claude Desktop

**Default Configuration Path:**
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%/Claude/claude_desktop_config.json`
- **Linux**: `~/.config/claude/claude_desktop_config.json`

**Configuration Format:**
```json
{
  "mcpServers": {
    "my-ai-tool": {
      "command": "icarus",
      "args": ["mcp", "start", "rdmx6-jaaaa-aaaah-qcaiq-cai"],
      "env": {}
    }
  }
}
```

### ChatGPT Desktop

**Default Configuration Path:**
- **macOS**: `~/Library/Application Support/ChatGPT/chatgpt_config.json`

**Note:** ChatGPT Desktop MCP support is anticipated but not yet available. The CLI includes it for future compatibility.

### Claude Code/Cline (VS Code Extension)

**Default Configuration Path:**
- **All Platforms**: `~/.vscode/extensions/saoudrizwan.claude-dev/settings/cline_mcp_settings.json`

**Configuration Format:**
```json
{
  "mcpServers": {
    "my-ai-tool": {
      "command": "icarus",
      "args": ["mcp", "start", "rdmx6-jaaaa-aaaah-qcaiq-cai"],
      "env": {},
      "type": "stdio"
    }
  }
}
```

## Configuration Management

### Custom Configuration Paths

Use the `--config-path` flag to specify custom configuration file locations:

```bash
# Custom Claude Desktop config
icarus mcp add <canister-id> --clients claude --config-path ~/my-custom-claude-config.json

# Portable configuration
icarus mcp add <canister-id> --config-path ./project-claude-config.json
```

### Environment Variables

The CLI respects the following environment variables:

- `CLAUDE_CONFIG_PATH` - Override Claude Desktop config path
- `ICARUS_DEBUG` - Enable debug logging for MCP operations

```bash
# Use custom config via environment
export CLAUDE_CONFIG_PATH=~/my-claude-config.json
icarus mcp add <canister-id> --clients claude

# Debug MCP operations
ICARUS_DEBUG=1 icarus mcp dashboard
```

## Troubleshooting

### Client Detection Issues

**Problem:** Client shows as "not installed" but you know it's installed.

**Solutions:**
1. Check if the application is in standard locations:
   ```bash
   # macOS
   ls /Applications/Claude.app
   ls /Applications/ChatGPT.app

   # Check VS Code extensions
   ls ~/.vscode/extensions/saoudrizwan.claude-dev
   ```

2. Use custom configuration paths:
   ```bash
   icarus mcp add <canister-id> --config-path /path/to/your/config.json
   ```

3. Manual configuration verification:
   ```bash
   icarus mcp dashboard  # Check system health
   ```

### Configuration Validation

**Problem:** MCP servers not appearing in AI clients.

**Solutions:**
1. Validate configuration format:
   ```bash
   icarus mcp list  # Shows configuration status
   ```

2. Check file permissions:
   ```bash
   # Ensure config files are readable/writable
   ls -la ~/Library/Application\ Support/Claude/claude_desktop_config.json
   ```

3. Restart AI clients after configuration changes.

### Bridge Connection Issues

**Problem:** AI clients can't connect to MCP servers.

**Solutions:**
1. Verify bridge is running:
   ```bash
   icarus bridge status
   ```

2. Start bridge manually:
   ```bash
   icarus bridge start --canister-id <your-canister-id>
   ```

3. Check canister accessibility:
   ```bash
   # Verify canister is deployed and accessible
   dfx canister status <canister-id>
   ```

### Performance Issues

**Problem:** Slow response times or timeouts.

**Solutions:**
1. Check network connectivity to ICP
2. Verify canister is not overloaded
3. Use local development network for testing:
   ```bash
   icarus deploy --network local
   ```

## Best Practices

### Development Workflow

1. **Use Local Development:**
   ```bash
   # Deploy locally first
   icarus deploy --network local
   icarus mcp add <local-canister-id>
   ```

2. **Test Before Production:**
   ```bash
   # Validate configuration
   icarus mcp dashboard

   # Test with bridge
   icarus bridge start --canister-id <canister-id>
   ```

3. **Production Deployment:**
   ```bash
   # Deploy to mainnet
   icarus deploy --network ic
   icarus mcp add <mainnet-canister-id> --clients claude
   ```

### Configuration Management

1. **Version Control Configurations:**
   ```bash
   # Keep project-specific configs in version control
   icarus mcp add <canister-id> --config-path ./mcp-config.json
   ```

2. **Environment-Specific Configs:**
   ```bash
   # Development
   icarus mcp add <dev-canister-id> --name "MyTool-Dev"

   # Production
   icarus mcp add <prod-canister-id> --name "MyTool"
   ```

3. **Backup Configurations:**
   ```bash
   # Backup current configurations
   cp ~/.claude/claude_desktop_config.json ~/.claude/claude_desktop_config.json.backup
   ```

### Security Considerations

1. **Verify Canister IDs:** Always double-check canister IDs before adding them to client configurations.

2. **Use Principle of Least Privilege:** Only add canisters to clients that need them.

3. **Regular Audits:** Periodically review configured MCP servers:
   ```bash
   icarus mcp list
   icarus mcp dashboard
   ```

## Advanced Usage

### Scripting and Automation

```bash
#!/bin/bash
# Automated deployment and configuration script

CANISTER_ID=$(icarus deploy --network ic | grep "Canister ID:" | cut -d: -f2 | xargs)
echo "Deployed canister: $CANISTER_ID"

# Add to all available clients
icarus mcp add "$CANISTER_ID" --clients claude,chatgpt,claude-code

# Start bridge for immediate use
icarus bridge start --canister-id "$CANISTER_ID" &

echo "MCP server configured and bridge started"
```

### CI/CD Integration

```yaml
# .github/workflows/deploy.yml
- name: Deploy and Configure MCP
  run: |
    icarus deploy --network ic
    CANISTER_ID=$(icarus canister-id)
    icarus mcp add "$CANISTER_ID" --clients claude
```

### Multi-Environment Management

```bash
# Development environment
icarus mcp add dev-canister-id --name "MyTool-Dev" --clients claude-code

# Staging environment
icarus mcp add staging-canister-id --name "MyTool-Staging" --clients claude

# Production environment
icarus mcp add prod-canister-id --name "MyTool" --clients claude,chatgpt
```

## Support

For additional help:

1. **CLI Help:** `icarus mcp --help`
2. **Dashboard:** `icarus mcp dashboard` for interactive troubleshooting
3. **GitHub Issues:** Report bugs at [icarus-sdk GitHub](https://github.com/icarus-mcp/icarus-sdk)
4. **Documentation:** Check the main [README.md](../README.md) for general usage

## Related Documentation

- [Main README](../README.md) - General CLI usage
- [Bridge Documentation](BRIDGE.md) - Bridge service details
- [Testing Guide](TESTING.md) - Testing your MCP servers