use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

use crate::utils::{ensure_directory_exists, print_info, print_success};

pub async fn execute(
    name: String,
    path: Option<String>,
    with_tests: bool,
    wasi: bool,
) -> Result<()> {
    // Validate project name
    if !is_valid_project_name(&name) {
        anyhow::bail!("Invalid project name. Use lowercase letters, numbers, and hyphens only.");
    }

    // Determine project path
    let project_path = if let Some(p) = path {
        PathBuf::from(p).join(&name)
    } else {
        std::env::current_dir()?.join(&name)
    };

    // Check if directory already exists
    if project_path.exists() {
        anyhow::bail!("Directory {} already exists", project_path.display());
    }

    print_info(&format!("Creating new Icarus project '{}'", name));

    // Create project directory
    ensure_directory_exists(&project_path)?;

    // Create project structure
    create_project_structure(&project_path, &name, with_tests, wasi)?;

    // Generate Cargo.lock file for deployment (like dfx does)
    print_info("Initializing project dependencies...");
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("generate-lockfile").current_dir(&project_path);

    let output = cmd.output()?;

    if !output.status.success() {
        eprintln!(
            "Warning: Could not generate Cargo.lock. You may need to run 'cargo build' manually."
        );
    }

    // Initialize git repository
    if which::which("git").is_ok() {
        std::process::Command::new("git")
            .arg("init")
            .current_dir(&project_path)
            .output()?;

        // Create .gitignore
        let gitignore = r#"# Icarus
.dfx/
.icarus/
target/
.env
*.wasm
canister_ids.json

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db
"#;
        std::fs::write(project_path.join(".gitignore"), gitignore)?;
    }

    print_success(&format!("Project '{}' created successfully!", name));

    println!("\n{}", "Next steps:".bold());
    println!("  cd {}", name);
    println!("  icarus deploy --network local");
    println!("\nFor more information, see the README.md in your project.");

    Ok(())
}

fn is_valid_project_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|c| c.is_alphanumeric() || c == '-')
        && name.chars().next().unwrap().is_alphabetic()
}

fn create_project_structure(
    project_path: &Path,
    name: &str,
    with_tests: bool,
    wasi: bool,
) -> Result<()> {
    // Create src directory
    let src_dir = project_path.join("src");
    ensure_directory_exists(&src_dir)?;

    // Create tests directory if requested
    if with_tests {
        let tests_dir = project_path.join("tests");
        ensure_directory_exists(&tests_dir)?;
    }

    // Create lib.rs from embedded template
    let target_lib_rs = src_dir.join("lib.rs");
    create_lib_template(&target_lib_rs, wasi)?;

    // Create Cargo.toml
    create_cargo_toml(project_path, name, with_tests, wasi)?;

    // Create dfx.json
    create_dfx_json(project_path, name, wasi)?;

    // Create minimal .did file
    create_minimal_candid_file(&src_dir, name)?;

    // Create README.md
    create_readme(project_path, name)?;

    Ok(())
}

fn create_lib_template(target: &Path, wasi: bool) -> Result<()> {
    let content = if wasi {
        // Full-featured WASI template with canister features
        r#"//! Hello Icarus - Full MCP Server Example
//!
//! A complete MCP server demonstrating Icarus CDK capabilities with authentication.

use candid::CandidType;
use ic_cdk::export_candid;
use icarus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CandidType)]
pub struct GreetingArgs {
    pub name: String,
}

/// Simple greeting function - accessible to everyone (no auth required)
#[icarus::tool("Say hello to Icarus")]
#[query]
pub fn hello() -> String {
    "Hello from Icarus! 🚀".to_string()
}

/// Personalized greeting - requires authenticated user
#[icarus::tool("Get personalized greeting", auth = "user")]
#[query]
pub fn greet(args: GreetingArgs) -> String {
    format!("Hello {}, welcome to Icarus! 👋", args.name)
}

/// System information - requires admin privileges
#[icarus::tool("Get system information", auth = "admin")]
#[query]
pub fn system_info() -> String {
    format!(
        "Icarus MCP Server v{} - Running on Internet Computer",
        env!("CARGO_PKG_VERSION")
    )
}

/// Echo service - demonstrates different auth levels
#[icarus::tool("Echo back your message", auth = "user")]
#[update]
pub async fn echo(message: String) -> Result<String, String> {
    if message.trim().is_empty() {
        return Err("Message cannot be empty".to_string());
    }
    Ok(format!("Echo: {}", message))
}

/// Admin-only function to reset or get status
#[icarus::tool("Get detailed system status", auth = "admin")]
#[query]
pub fn admin_status() -> String {
    let auth_info = get_current_user();
    let users_count = list_users().len();
    format!(
        "System Status:\n- Caller: {}\n- Authenticated: {}\n- Role: {:?}\n- Users: {}",
        auth_info.principal,
        auth_info.is_authenticated,
        auth_info.role,
        users_count
    )
}

// NEW: Builder pattern - marketplace-compatible MCP canister
// Automatically includes auth, init(owner), and all required functions
icarus::mcp! {
    .with_wasi()
    .build()
};

// Export the Candid interface for dfx deployment
export_candid!();
"#
        .to_string()
    } else {
        // Simple template with canister features but no WASI dependencies
        r#"//! Hello Icarus - Simple MCP Server Example
//!
//! A minimal MCP server demonstrating Icarus CDK capabilities with authentication.

use candid::CandidType;
use ic_cdk::export_candid;
use icarus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CandidType)]
pub struct GreetingArgs {
    pub name: String,
}

/// Personalized greeting - requires authenticated user
#[icarus::tool("Get personalized greeting", auth = "user")]
#[query]
pub fn greet(args: GreetingArgs) -> String {
    format!("Hello {}, welcome to Icarus! 👋", args.name)
}

/// Echo service - demonstrates async functionality
#[icarus::tool("Echo back your message", auth = "user")]
#[update]
pub async fn echo(message: String) -> Result<String, String> {
    if message.trim().is_empty() {
        return Err("Message cannot be empty".to_string());
    }
    Ok(format!("Echo: {}", message))
}

// NEW: Builder pattern - marketplace-compatible MCP canister
// Automatically includes auth, init(owner), and all required functions
icarus::mcp! {
    .build()
};

// Export the Candid interface for dfx deployment
export_candid!();
"#
        .to_string()
    };

    fs::write(target, content)?;
    Ok(())
}

fn create_cargo_toml(project_path: &Path, name: &str, with_tests: bool, wasi: bool) -> Result<()> {
    // Create Cargo.toml from template
    let content = create_cargo_toml_content(name, with_tests, wasi)?;

    fs::write(project_path.join("Cargo.toml"), content)?;
    Ok(())
}

fn get_workspace_versions() -> Result<std::collections::HashMap<String, String>> {
    // Try to find workspace Cargo.toml relative to CLI location
    let workspace_cargo_paths = [
        "../Cargo.toml",
        "../../Cargo.toml",
        "../../../Cargo.toml",
        "Cargo.toml",
    ];

    for path in &workspace_cargo_paths {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(toml_value) = content.parse::<Value>() {
                if let Some(workspace) = toml_value.get("workspace") {
                    if let Some(deps) = workspace.get("dependencies") {
                        let mut versions = std::collections::HashMap::new();

                        if let Some(deps_table) = deps.as_table() {
                            for (name, value) in deps_table {
                                let version = match value {
                                    Value::String(v) => v.clone(),
                                    Value::Table(t) => {
                                        if let Some(v) = t.get("version") {
                                            v.as_str().unwrap_or("").to_string()
                                        } else {
                                            continue;
                                        }
                                    }
                                    _ => continue,
                                };
                                versions.insert(name.clone(), version);
                            }
                        }
                        return Ok(versions);
                    }
                }
            }
        }
    }

    // Fallback to hardcoded versions if workspace not found
    let mut fallback = std::collections::HashMap::new();
    fallback.insert("ic-cdk".to_string(), "0.18".to_string());
    fallback.insert("ic-cdk-macros".to_string(), "0.18".to_string());
    fallback.insert("ic-stable-structures".to_string(), "0.7".to_string());
    fallback.insert("candid".to_string(), "0.10".to_string());
    fallback.insert("serde".to_string(), "1.0".to_string());
    fallback.insert("serde_json".to_string(), "1.0".to_string());
    fallback.insert("tokio".to_string(), "1".to_string());
    Ok(fallback)
}

fn create_cargo_toml_content(name: &str, with_tests: bool, wasi: bool) -> Result<String> {
    let versions = get_workspace_versions()?;

    let get_version = |dep: &str| -> String {
        versions.get(dep).cloned().unwrap_or_else(|| match dep {
            "ic-cdk" => "0.18".to_string(),
            "ic-cdk-macros" => "0.18".to_string(),
            "ic-stable-structures" => "0.7".to_string(),
            "candid" => "0.10".to_string(),
            "serde" => "1.0".to_string(),
            "serde_json" => "1.0".to_string(),
            "tokio" => "1".to_string(),
            _ => "1.0".to_string(),
        })
    };

    let dev_dependencies_section = if with_tests {
        format!(
            r#"

[dev-dependencies]
candid = "{}"
tokio = {{ version = "{}", features = ["full"] }}
"#,
            get_version("candid"),
            get_version("tokio")
        )
    } else {
        "".to_string()
    };

    let icarus_dep = {
        let cli_version = env!("CARGO_PKG_VERSION");
        if wasi {
            // WASI projects need both "canister" and "wasi" features
            format!(
                "{{ version = \"{}\", default-features = false, features = [\"canister\", \"wasi\"] }}",
                cli_version
            )
        } else {
            // Simple projects only need "canister" feature
            format!(
                "{{ version = \"{}\", default-features = false, features = [\"canister\"] }}",
                cli_version
            )
        }
    };

    // Conditional WASI content
    let wasi_dependency = if wasi {
        let cli_version = env!("CARGO_PKG_VERSION");
        format!(
            "\nic-stable-structures = \"{}\"\n\n# WASI support (self-contained)\nicarus-wasi = \"{}\"\n",
            get_version("ic-stable-structures"),
            cli_version
        )
    } else {
        "".to_string()
    };

    let features_section = ""; // No features needed - icarus-wasi handles automatically

    Ok(format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[package.metadata.icarus]
claude_desktop.auto_update = true
claude_desktop.config_path = ""
claude_code.auto_update = false
claude_code.config_path = ""
chatgpt.auto_update = false
chatgpt.config_path = ""

[dependencies]
# Core IC CDK
ic-cdk = "{}"
candid = "{}"{}

# Icarus framework
icarus = {}

# Serialization
serde = {{ version = "{}", features = ["derive"] }}
serde_json = "{}"{}

[lib]
crate-type = ["cdylib"]{}

[profile.release]
lto = true
strip = "debuginfo"
overflow-checks = false
debug = false
codegen-units = 1
panic = "abort"
rpath = false
"#,
        name,
        get_version("ic-cdk"),
        get_version("candid"),
        wasi_dependency,
        icarus_dep,
        get_version("serde"),
        get_version("serde_json"),
        dev_dependencies_section,
        features_section
    ))
}

fn create_dfx_json(project_path: &Path, name: &str, wasi: bool) -> Result<()> {
    let dfx_json = if wasi {
        // WASI projects use custom type but let Icarus handle building
        format!(
            r#"{{
  "canisters": {{
    "{}": {{
      "type": "custom",
      "package": "{}",
      "wasm": "target/wasm32-wasip1/release/{}_ic.wasm",
      "candid": "src/{}.did",
      "optimize": "cycles",
      "metadata": [
        {{
          "name": "candid:service"
        }}
      ]
    }}
  }},
  "defaults": {{
    "build": {{
      "packtool": ""
    }}
  }},
  "networks": {{
    "local": {{
      "bind": "127.0.0.1:4943",
      "type": "ephemeral"
    }}
  }},
  "version": 1
}}
"#,
            name,
            name,
            name.replace('-', "_"),
            name
        )
    } else {
        // Regular projects use dfx native Rust support
        format!(
            r#"{{
  "canisters": {{
    "{}": {{
      "type": "rust",
      "package": "{}",
      "candid": "src/{}.did"
    }}
  }},
  "defaults": {{
    "build": {{
      "packtool": ""
    }}
  }},
  "networks": {{
    "local": {{
      "bind": "127.0.0.1:4943",
      "type": "ephemeral"
    }}
  }},
  "version": 1
}}
"#,
            name, name, name
        )
    };
    std::fs::write(project_path.join("dfx.json"), dfx_json)?;
    Ok(())
}

fn create_minimal_candid_file(src_dir: &Path, name: &str) -> Result<()> {
    // Create a placeholder .did file that will be overwritten by the build script
    // The actual interface is auto-generated from the Rust code
    let candid_content = r#"// Placeholder Candid interface for Icarus MCP server
// This file will be auto-generated when you run 'icarus build' or 'dfx deploy'
// The actual interface is extracted from your Rust code using icarus::auth!() and icarus::mcp!()

service : (principal) -> {
  // This is a placeholder - run 'icarus build' to generate the real interface
}
"#;

    let candid_path = src_dir.join(format!("{}.did", name));
    std::fs::write(candid_path, candid_content)?;
    Ok(())
}

fn create_readme(project_path: &Path, name: &str) -> Result<()> {
    let readme = format!(
        r#"# {}

A simple "Hello Icarus!" MCP (Model Context Protocol) server running on the Internet Computer with authentication.

## Features

- 🚀 Simple greeting functions demonstrating MCP tool system
- 🔐 Built-in authentication with user management
- 👥 Three auth levels: public, user, admin
- 🏗️ WASI-native architecture for maximum compatibility
- 📦 Auto-generated Candid interfaces

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) with `wasm32-wasip1` target
- [dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install)
- [Icarus CLI](https://crates.io/crates/icarus-cli)
- Tools: `wasi2ic`, `candid-extractor` (installed with Icarus CLI)

### Development

```bash
# Build the project (WASI → IC WASM + Candid extraction)
icarus build

# Start local Internet Computer
dfx start --clean

# Deploy with an owner principal (replace with your principal)
dfx deploy {} --argument '(principal "your-principal-id-here")'
```

### Testing the Functions

```bash
# Public function (no auth required)
dfx canister call {} hello

# User function (requires authentication)
dfx canister call {} greet '(record {{ name = "World" }})'

# Admin function (requires admin role)
dfx canister call {} system_info

# Check current authentication status
dfx canister call {} get_current_user

# Add a user (admin/owner only)
dfx canister call {} add_user '(principal "user-principal-id", "user")'

# List all users (admin/owner only)
dfx canister call {} list_users
```

### Authentication System

The project includes a full authentication system:

- **Public functions** (no auth): `hello`, `get_current_user`, `get_tools`
- **User functions** (auth = "user"): `greet`, `echo`
- **Admin functions** (auth = "admin"): `system_info`, `admin_status`
- **Generated auth functions**: `add_user`, `remove_user`, `update_user_role`, etc.

When deployed, the owner principal (passed to init) becomes the system owner and can manage all users.

### Using with Claude Desktop

After deployment, set up the MCP bridge:

```bash
# Get your canister ID from deployment output, then:
icarus bridge start --canister-id <your-canister-id>
```

Add the bridge configuration to Claude Desktop's config file.

## Project Structure

- `src/lib.rs` - Main canister with greeting functions and auth macros
- `src/{}.did` - Auto-generated Candid interface
- `dfx.json` - Internet Computer deployment configuration
- `Cargo.toml` - Project dependencies with WASI support

## Customization

Modify `src/lib.rs` to add your own MCP tools:

```rust
#[icarus::tool("Your tool description", auth = "user")]
#[update]
pub async fn your_tool(args: YourArgs) -> Result<String, String> {{
    // Your tool implementation
    Ok("Success!".to_string())
}}
```

Auth levels:
- `auth = "none"` or no auth parameter: Public access
- `auth = "user"`: Requires authenticated user
- `auth = "admin"`: Requires admin role or higher

## License

See LICENSE file for details.
"#,
        name, name, name, name, name, name, name, name, name
    );
    std::fs::write(project_path.join("README.md"), readme)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_lib_template_with_wasi() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        create_lib_template(&src_dir.join("lib.rs"), true).unwrap();

        let lib_rs_content = fs::read_to_string(src_dir.join("lib.rs")).unwrap();

        // Should contain new builder pattern with WASI
        assert!(
            lib_rs_content.contains("icarus::mcp! {"),
            "Should contain new builder pattern"
        );
        assert!(
            lib_rs_content.contains(".with_wasi()"),
            "WASI-enabled project should use .with_wasi()"
        );
        assert!(
            lib_rs_content.contains(".build()"),
            "Should contain .build() call"
        );
        assert!(
            lib_rs_content.contains("marketplace-compatible"),
            "Should mention marketplace compatibility"
        );
    }

    #[test]
    fn test_create_lib_template_without_wasi() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        create_lib_template(&src_dir.join("lib.rs"), false).unwrap();

        let lib_rs_content = fs::read_to_string(src_dir.join("lib.rs")).unwrap();

        // Should contain new builder pattern without WASI
        assert!(
            lib_rs_content.contains("icarus::mcp! {"),
            "Should contain new builder pattern"
        );
        assert!(
            !lib_rs_content.contains(".with_wasi()"),
            "Non-WASI project should not contain .with_wasi()"
        );
        assert!(
            lib_rs_content.contains(".build()"),
            "Should contain .build() call"
        );
        assert!(
            lib_rs_content.contains("marketplace-compatible"),
            "Should mention marketplace compatibility"
        );
    }

    #[test]
    fn test_create_cargo_toml_with_wasi() {
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml_content = create_cargo_toml_content("test-project", false, true).unwrap();

        // Write the content to the temp directory for verification
        fs::write(temp_dir.path().join("Cargo.toml"), &cargo_toml_content).unwrap();

        // Should contain icarus-wasi dependency
        assert!(
            cargo_toml_content.contains("icarus-wasi"),
            "WASI-enabled project should have icarus-wasi dependency"
        );

        // Should have wasi feature in icarus dependency
        assert!(
            cargo_toml_content.contains(r#"features = ["canister", "wasi"]"#),
            "icarus dependency should include wasi feature"
        );
    }

    #[test]
    fn test_create_cargo_toml_without_wasi() {
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml_content = create_cargo_toml_content("test-project", false, false).unwrap();

        // Write the content to the temp directory for verification
        fs::write(temp_dir.path().join("Cargo.toml"), &cargo_toml_content).unwrap();

        // Should NOT contain WASI-specific dependencies
        assert!(
            !cargo_toml_content.contains("icarus-wasi")
                && !cargo_toml_content.contains("ic-wasi-polyfill"),
            "Non-WASI project should not have WASI dependencies. Content: {}",
            cargo_toml_content
        );

        // Should NOT have features section
        assert!(
            !cargo_toml_content.contains("[features]"),
            "Non-WASI project should not have features section"
        );

        // Should NOT mention wasi anywhere
        assert!(
            !cargo_toml_content.contains("wasi"),
            "Non-WASI project should not reference wasi at all"
        );
    }

    #[tokio::test]
    async fn test_execute_with_wasi_flag() {
        let temp_dir = TempDir::new().unwrap();
        let project_name = "test-wasi-project";
        let project_path = temp_dir.path().join(project_name);

        // Execute with wasi=true
        execute(
            project_name.to_string(),
            Some(project_path.to_string_lossy().to_string()),
            false,
            true,
        )
        .await
        .unwrap();

        // Verify WASI-specific content was generated
        // The execute function creates the project in a subdirectory with the project name
        let actual_project_path = project_path.join(project_name);
        let cargo_toml = fs::read_to_string(actual_project_path.join("Cargo.toml")).unwrap();
        assert!(
            cargo_toml.contains("icarus-wasi")
                || cargo_toml.contains(r#"features = ["canister", "wasi"]"#),
            "Generated project should have WASI dependency"
        );

        let lib_rs = fs::read_to_string(actual_project_path.join("src/lib.rs")).unwrap();
        assert!(
            lib_rs.contains(".with_wasi()"),
            "Generated WASI project should use .with_wasi() builder pattern"
        );
    }

    #[tokio::test]
    async fn test_execute_without_wasi_flag() {
        let temp_dir = TempDir::new().unwrap();
        let project_name = "test-no-wasi-project";
        let project_path = temp_dir.path().join(project_name);

        // Execute with wasi=false
        execute(
            project_name.to_string(),
            Some(project_path.to_string_lossy().to_string()),
            false,
            false,
        )
        .await
        .unwrap();

        // Verify NO WASI content was generated
        // The execute function creates the project in a subdirectory with the project name
        let actual_project_path = project_path.join(project_name);
        let cargo_toml = fs::read_to_string(actual_project_path.join("Cargo.toml")).unwrap();
        assert!(
            !cargo_toml.contains("icarus-wasi") && !cargo_toml.contains("ic-wasi-polyfill"),
            "Generated project should not have WASI dependency"
        );

        let lib_rs = fs::read_to_string(actual_project_path.join("src/lib.rs")).unwrap();
        assert!(
            !lib_rs.contains("WASI initialization is automatic"),
            "Generated project should not mention WASI initialization"
        );
    }

    #[test]
    fn test_cargo_toml_formatting_escapes_braces_correctly() {
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml_content = create_cargo_toml_content("test-project", false, true).unwrap();

        // Write the content to the temp directory
        fs::write(temp_dir.path().join("Cargo.toml"), &cargo_toml_content).unwrap();

        let read_content = fs::read_to_string(temp_dir.path().join("Cargo.toml")).unwrap();

        // Should have proper TOML formatting without {{ }} artifacts
        // Check for the current WASI dependency format (icarus-wasi or icarus dependency with wasi features)
        let has_proper_formatting = read_content.contains(r#"icarus-wasi = "0.8.0""#)
            || read_content.contains(r#"features = ["canister", "wasi"]"#);
        assert!(
            has_proper_formatting,
            "Should have properly formatted dependency declaration. Content: {}",
            read_content
        );

        // Should not contain template artifacts
        assert!(
            !read_content.contains("{{{{"),
            "Should not contain template escape sequences"
        );
    }
}
