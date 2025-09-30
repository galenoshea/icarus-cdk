//! Basic "Hello World" template for Icarus MCP canister projects.
//!
//! This module provides a simple, working MCP server template for new projects.

use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;

/// Template content for Cargo.toml
const CARGO_TOML: &str = r#"[package]
name = "{{PROJECT_NAME}}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
icarus = { version = "0.9", features = ["macros"] }
ic-cdk = "0.16"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
candid = "0.10"

[dev-dependencies]
candid = "0.10"
"#;

/// Template content for src/lib.rs
const LIB_RS: &str = r#"use icarus::tool;
use serde::{Deserialize, Serialize};

/// A simple hello world tool that returns a greeting.
#[tool("Returns a personalized greeting")]
fn hello_world(name: String) -> String {
    format!("Hello, {}! Welcome to Icarus MCP.", name)
}

/// A calculator tool that adds two numbers.
#[tool("Adds two numbers together")]
fn add_numbers(a: i64, b: i64) -> i64 {
    a + b
}

/// System info tool that returns basic canister information.
#[tool("Returns system information about this canister")]
fn system_info() -> SystemInfo {
    SystemInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: env!("CARGO_PKG_NAME").to_string(),
        description: "A simple Icarus MCP canister".to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemInfo {
    version: String,
    name: String,
    description: String,
}

// Export candid interface
ic_cdk::export_candid!();
"#;

/// Template content for dfx.json
const DFX_JSON: &str = r#"{
  "version": 1,
  "canisters": {
    "{{PROJECT_NAME}}": {
      "type": "rust",
      "candid": "{{PROJECT_NAME}}.did",
      "package": "{{PROJECT_NAME}}",
      "build": [
        "cargo build --target wasm32-unknown-unknown --release --package {{PROJECT_NAME}}"
      ],
      "wasm": "target/wasm32-unknown-unknown/release/{{PROJECT_NAME}}.wasm"
    }
  },
  "defaults": {
    "build": {
      "packtool": ""
    }
  }
}
"#;

/// Template content for README.md
const README_MD: &str = r#"# {{PROJECT_NAME}}

A simple MCP (Model Context Protocol) canister built with the Icarus SDK.

## Features

- **hello_world**: Returns a personalized greeting
- **add_numbers**: Adds two numbers together
- **system_info**: Returns canister information

## Getting Started

### Prerequisites

- Rust toolchain (1.70+)
- dfx (Internet Computer SDK)
- wasm32-unknown-unknown target

### Installation

```bash
# Install the wasm target
rustup target add wasm32-unknown-unknown

# Install dfx if not already installed
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
```

### Building

```bash
# Build the canister
icarus build

# Or use dfx directly
dfx build
```

### Deploying

```bash
# Deploy locally
icarus deploy --network local

# Or use dfx
dfx deploy --network local
```

### Testing Tools

You can test the MCP tools using the Icarus CLI:

```bash
# List available tools
icarus mcp list

# Test the hello_world tool
icarus mcp call hello_world '{"name": "Alice"}'

# Test the add_numbers tool
icarus mcp call add_numbers '{"a": 5, "b": 3}'
```

## Project Structure

```
{{PROJECT_NAME}}/
├── Cargo.toml          # Rust dependencies and package config
├── dfx.json            # Internet Computer canister config
├── src/
│   └── lib.rs         # Main MCP server implementation
└── README.md          # This file
```

## Learn More

- [Icarus SDK Documentation](https://github.com/galenoshea/icarus-cdk)
- [Internet Computer Documentation](https://internetcomputer.org/docs)
- [MCP Protocol Specification](https://modelcontextprotocol.io)

## License

This project was created with Icarus SDK.
"#;

/// Template content for .gitignore
const GITIGNORE: &str = r#"# Rust
/target
Cargo.lock
**/*.rs.bk

# IC / dfx
.dfx/
*.did
*.wasm

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Environment
.env
.env.local
"#;

/// Generate a basic Icarus project from templates.
///
/// This function creates all necessary files for a minimal working
/// Icarus MCP canister project.
pub async fn generate_project(project_name: &str, project_path: &Path) -> Result<()> {
    // Create src directory
    let src_dir = project_path.join("src");
    fs::create_dir_all(&src_dir)
        .await
        .context("Failed to create src directory")?;

    // Replace {{PROJECT_NAME}} placeholder in all templates
    let cargo_toml = CARGO_TOML.replace("{{PROJECT_NAME}}", project_name);
    let dfx_json = DFX_JSON.replace("{{PROJECT_NAME}}", project_name);
    let readme_md = README_MD.replace("{{PROJECT_NAME}}", project_name);

    // Write Cargo.toml
    fs::write(project_path.join("Cargo.toml"), cargo_toml)
        .await
        .context("Failed to write Cargo.toml")?;

    // Write src/lib.rs
    fs::write(project_path.join("src/lib.rs"), LIB_RS)
        .await
        .context("Failed to write src/lib.rs")?;

    // Write dfx.json
    fs::write(project_path.join("dfx.json"), dfx_json)
        .await
        .context("Failed to write dfx.json")?;

    // Write README.md
    fs::write(project_path.join("README.md"), readme_md)
        .await
        .context("Failed to write README.md")?;

    // Write .gitignore
    fs::write(project_path.join(".gitignore"), GITIGNORE)
        .await
        .context("Failed to write .gitignore")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_generate_basic_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_name = "test-project";
        let project_path = temp_dir.path().join(project_name);

        fs::create_dir_all(&project_path).await.unwrap();

        generate_project(project_name, &project_path).await.unwrap();

        // Verify all files were created
        assert!(project_path.join("Cargo.toml").exists());
        assert!(project_path.join("src/lib.rs").exists());
        assert!(project_path.join("dfx.json").exists());
        assert!(project_path.join("README.md").exists());
        assert!(project_path.join(".gitignore").exists());

        // Verify content substitution
        let cargo_content = fs::read_to_string(project_path.join("Cargo.toml"))
            .await
            .unwrap();
        assert!(cargo_content.contains(&format!("name = \"{}\"", project_name)));
    }
}
