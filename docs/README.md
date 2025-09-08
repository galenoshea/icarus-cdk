# Icarus SDK Documentation

Welcome to the Icarus SDK documentation! This guide will help you build MCP (Model Context Protocol) servers that run on the Internet Computer.

## 📚 Documentation Structure

### Getting Started
- **[Getting Started Guide](getting-started.md)** - Your first Icarus project
- **[Installation](../cli/docs/installation.md)** - Installing the SDK and CLI
- **[Quick Example](../examples/basic-memory/)** - Basic memory server example

### Core Concepts
- **[API Reference](api-reference.md)** - Complete API documentation
- **[Macros Guide](macros.md)** - Understanding Icarus macros
- **[Stable Storage](stable-storage.md)** - Persistent data storage
- **[Migration Guide](migration-guide.md)** - Upgrading between versions

### CLI Documentation
- **[CLI Commands](../cli/docs/commands.md)** - Complete command reference
- **[Bridge Architecture](../cli/docs/bridge-architecture.md)** - How the MCP-ICP bridge works
- **[Deployment Guide](../cli/docs/deployment-guide.md)** - Deploying to ICP
- **[Troubleshooting](../cli/docs/troubleshooting.md)** - Common issues and solutions

### Examples
- **[Basic Memory Server](../examples/basic-memory/)** - Simple storage example
- **[HTTP Fetcher](../examples/http-fetcher/)** - External API integration
- **[Auto-Refresher](../examples/auto-refresher/)** - Timers + HTTP outcalls

## 🚀 Quick Start

1. **Install the CLI:**
   ```bash
   cargo install icarus-cli
   ```

2. **Create a new project:**
   ```bash
   icarus new my-mcp-server
   cd my-mcp-server
   ```

3. **Build and deploy:**
   ```bash
   icarus build
   icarus deploy --network local
   ```

4. **Connect to Claude:**
   ```bash
   icarus bridge start --canister-id <your-canister-id>
   ```

## 📖 Learning Path

### For Beginners
1. Start with the [Getting Started Guide](getting-started.md)
2. Try the [Basic Memory Example](../examples/basic-memory/)
3. Learn about [Stable Storage](stable-storage.md)
4. Explore the [CLI Commands](../cli/docs/commands.md)

### For Experienced Developers
1. Review the [API Reference](api-reference.md)
2. Deep dive into [Macros](macros.md)
3. Understand the [Bridge Architecture](../cli/docs/bridge-architecture.md)
4. Check the [Deployment Guide](../cli/docs/deployment-guide.md)

## 🔧 Key Features

- **🔨 Simple Rust Macros** - Minimal boilerplate with powerful macros
- **💾 Persistent Storage** - Data persists across canister upgrades
- **🌐 Global Access** - Deploy once, access from anywhere
- **🔒 Blockchain Security** - Benefit from ICP's security model
- **🚀 Easy Deployment** - Simple CLI commands for building and deploying

## 📦 Project Structure

```
your-project/
├── src/
│   └── lib.rs          # Your MCP server implementation
├── Cargo.toml          # Rust dependencies
├── dfx.json           # ICP deployment configuration
└── .did               # Candid interface (generated)
```

## 🤝 Community & Support

- **GitHub**: [icarus-sdk](https://github.com/galenoshea/icarus-sdk)
- **Issues**: [Report bugs or request features](https://github.com/galenoshea/icarus-sdk/issues)
- **Contributing**: See [CONTRIBUTING.md](../CONTRIBUTING.md)

## 📝 License

The Icarus SDK is licensed under the Business Source License (BSL-1.1). See [LICENSE](../LICENSE) for details.

## 🗺️ Roadmap

- [ ] More example projects
- [ ] Web UI for canister management
- [ ] Marketplace integration
- [ ] Advanced storage patterns
- [ ] Authentication helpers
- [ ] Performance optimization guides

## Need Help?

- Check the [Troubleshooting Guide](../cli/docs/troubleshooting.md)
- Review the [Examples](../examples/)
- Open an [Issue](https://github.com/galenoshea/icarus-sdk/issues)

---

*Last updated: September 2025 | Version 0.5.6*