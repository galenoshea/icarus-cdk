# icarus-dev

**Development Tools for the Icarus SDK**

This crate provides enhanced development tools for building MCP servers with the Icarus SDK, including file watching, project status monitoring, and development utilities.

## Overview

The `icarus-dev` crate contains development-focused functionality that enhances the developer experience when building Icarus-based MCP servers. It provides file watching, project analysis, and interactive development tools.

## Key Components

### 📁 File Watching (`watch.rs`)
- Intelligent file system monitoring
- Hot-reload support for Rust projects
- Configurable ignore patterns
- Event filtering and batching
- Cross-platform compatibility

### 📊 Status Monitoring (`status.rs`)
- Project health checking
- Dependency validation
- Build status monitoring
- Configuration verification
- Development environment analysis

### 🛠️ Utilities (`utils.rs`)
- Progress indicators and spinners
- Colored output and formatting
- Command execution helpers
- Directory management
- Development workflow automation

### 🚀 Project Management (`init.rs`, `start.rs`, `reset.rs`)
- Project initialization and scaffolding
- Development server management
- Environment reset and cleanup
- Configuration management
- Template generation

## Features

### 🔄 Hot Reload Development

```rust
use icarus_dev::watch::*;

// Start file watcher for automatic rebuilds
let watcher = FileWatcher::new("./src")
    .ignore_patterns(&[".git", "target", "node_modules"])
    .on_change(|event| {
        println!("File changed: {:?}", event.path);
        // Trigger rebuild
    })
    .start()?;
```

### 📈 Project Status

```rust
use icarus_dev::status::*;

// Check project health
let status = ProjectStatus::check(".")?;

if status.is_icarus_project() {
    println!("✅ Valid Icarus project");
    println!("📦 Project name: {}", status.project_name()?);
    println!("🏗️  WASM exists: {}", status.wasm_exists());
}
```

### 🎨 Enhanced UI

```rust
use icarus_dev::utils::*;

// Create progress spinner
let spinner = create_spinner("Building project...");
// ... long operation ...
spinner.finish();

// Print styled messages
print_success("Build completed successfully!");
print_warning("Some dependencies are outdated");
print_error("Build failed");
```

### 🛠️ Command Execution

```rust
use icarus_dev::utils::*;

// Run commands with output capture
let output = run_command("cargo", &["build", "--release"], None).await?;
println!("Build output: {}", output);

// Run interactive commands
run_command_interactive("dfx", &["deploy"], Some(project_path)).await?;
```

## CLI Integration

This crate powers many CLI commands:

```bash
# File watching and hot reload
icarus dev                 # Start development mode with file watching

# Project status and health
icarus status             # Check project configuration and health

# Project management
icarus init               # Initialize new project
icarus reset              # Reset development environment
```

## Configuration

### Watch Configuration

```toml
[tool.icarus.watch]
# Files to watch
include = ["src/**/*.rs", "Cargo.toml", "dfx.json"]

# Files to ignore
exclude = ["target/**", ".git/**", "*.tmp"]

# Debounce delay (ms)
debounce = 500

# Commands to run on changes
on_change = ["cargo check"]
```

### Development Settings

```toml
[tool.icarus.dev]
# Auto-rebuild on changes
auto_build = true

# Restart services on config changes
auto_restart = true

# Show detailed progress
verbose = true
```

## Usage

### Basic Development Workflow

```rust
use icarus_dev::prelude::*;

// Start development environment
let dev_env = DevEnvironment::new(".")
    .with_file_watching(true)
    .with_auto_build(true)
    .with_status_monitoring(true)
    .start()?;

// Monitor for changes
dev_env.run().await?;
```

### Project Initialization

```rust
use icarus_dev::init::*;

// Create new project with templates
let project = ProjectInitializer::new("my-mcp-server")
    .template("basic-memory")
    .with_features(&["timers", "http"])
    .initialize()?;
```

### Status Checking

```rust
use icarus_dev::status::*;

// Comprehensive project analysis
let health = ProjectHealth::analyze(".")?;

if !health.is_healthy() {
    for issue in health.issues() {
        print_warning(&format!("⚠️  {}", issue));
    }
}
```

## Performance Features

- **Efficient File Watching**: Uses native OS file system events
- **Debounced Events**: Prevents excessive rebuilds from rapid changes
- **Parallel Processing**: Concurrent file operations and command execution
- **Memory Efficient**: Minimal resource usage during development
- **Cross-Platform**: Works on Windows, macOS, and Linux

## Integration

This crate is automatically included with the Icarus CLI:

```toml
[dependencies]
icarus = "0.7.0"  # Includes icarus-dev functionality via CLI
```

Or use directly for custom development tools:

```toml
[dependencies]
icarus-dev = "0.7.0"
```

## Development Tools

```bash
# Run tests
cargo test

# Test file watching
cargo test --test watch_integration

# Build with development features
cargo build --features "dev-tools"
```

## Related Crates

- [`icarus`](../icarus/) - Main SDK with all features
- [`icarus-core`](../icarus-core/) - Core types and traits
- [`icarus-bridge`](../icarus-bridge/) - MCP-to-ICP bridge
- [`icarus-canister`](../icarus-canister/) - Canister-side functionality
- [`icarus-derive`](../icarus-derive/) - Proc macros
- [`icarus-mcp`](../icarus-mcp/) - MCP protocol implementation

## License

Licensed under the Business Source License 1.1 (BSL). See [LICENSE](../../LICENSE) for details.