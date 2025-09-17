use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

use crate::utils::{ensure_directory_exists, print_info, print_success};

pub async fn execute(
    name: String,
    path: Option<String>,
    local_sdk: Option<String>,
    with_tests: bool,
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

    if let Some(sdk_path) = &local_sdk {
        print_info(&format!("Using local SDK from: {}", sdk_path));
    }

    // Create project directory
    ensure_directory_exists(&project_path)?;

    // Create project structure
    create_project_structure(&project_path, &name, local_sdk, with_tests)?;

    // Generate Cargo.lock by building the project
    print_info("Initializing project dependencies...");
    let output = std::process::Command::new("cargo")
        .args(&["build", "--target", "wasm32-unknown-unknown", "--release"])
        .current_dir(&project_path)
        .output()?;

    if !output.status.success() {
        // Don't fail if build fails, just warn
        // The user might not have all dependencies installed yet
        eprintln!("Warning: Initial build failed. You may need to resolve dependencies manually.");
        eprintln!(
            "Run 'cargo build --target wasm32-unknown-unknown --release' in the project directory."
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
    project_path: &PathBuf,
    name: &str,
    local_sdk: Option<String>,
    with_tests: bool,
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
    create_lib_template(&target_lib_rs)?;

    // Create Cargo.toml
    create_cargo_toml(project_path, name, local_sdk, with_tests)?;

    // Create dfx.json
    create_dfx_json(project_path, name)?;

    // Create minimal .did file
    create_minimal_candid_file(&src_dir, name)?;

    // Create README.md
    create_readme(project_path, name)?;

    Ok(())
}

fn create_lib_template(target: &Path) -> Result<()> {
    // Create the default lib.rs template for new projects
    let content = r#"//! Basic Memory Server
//!
//! A simple MCP server that stores and retrieves text memories.

use icarus::prelude::*;
use candid::{CandidType, Deserialize};
use serde::Serialize;
use ic_cdk::api::time;

/// A memory entry that persists across canister upgrades
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub created_at: u64,
    pub tags: Vec<String>,
}

// Declare stable storage that persists across upgrades
stable_storage! {
    // BTree map for efficient key-value storage
    MEMORIES: StableBTreeMap<String, MemoryEntry, Memory> = memory_id!(0);
    // Simple counter for generating unique IDs
    COUNTER: u64 = 0;
}

// Helper function to generate unique IDs
fn generate_id() -> String {
    COUNTER.with(|c| {
        let mut counter = c.borrow_mut();
        *counter += 1;
        format!("mem_{}", *counter)
    })
}

/// Memory service implementing Icarus tools
pub struct MemoryService;

/// MCP tools implementation using the new trait-based approach
#[icarus_tools]
impl IcarusToolProvider for MemoryService {

    /// Store a new memory with optional tags
    #[tool("Store a new memory with optional tags")]
    #[update]
    async fn memorize(content: String, tags: Option<Vec<String>>) -> Result<String, String> {
        if content.is_empty() {
            return Err("Content cannot be empty".to_string());
        }

        let id = generate_id();
        let memory = MemoryEntry {
            id: id.clone(),
            content,
            created_at: time(),
            tags: tags.unwrap_or_default(),
        };

        MEMORIES.with(|m| {
            m.borrow_mut().insert(id.clone(), memory);
        });

        Ok(id)
    }

    /// Retrieve a specific memory by ID
    #[tool("Retrieve a specific memory by ID")]
    #[query]
    async fn recall(id: String) -> Result<MemoryEntry, String> {
        MEMORIES.with(|m| {
            m.borrow()
                .get(&id)
                .ok_or_else(|| format!("Memory with ID {} not found", id))
        })
    }

    /// List all stored memories with optional limit
    #[tool("List all stored memories with optional limit")]
    #[query]
    async fn list(limit: Option<u64>) -> Result<Vec<MemoryEntry>, String> {
        Ok(MEMORIES.with(|m| {
            let memories = m.borrow();
            let iter = memories.iter();

            match limit {
                Some(n) => iter.take(n as usize).map(|(_, v)| v.clone()).collect(),
                None => iter.map(|(_, v)| v.clone()).collect(),
            }
        }))
    }

    /// Search memories by tag
    #[tool("Search memories by tag")]
    #[query]
    async fn search_by_tag(tag: String) -> Result<Vec<MemoryEntry>, String> {
        Ok(MEMORIES.with(|m| {
            m.borrow()
                .iter()
                .filter(|(_, memory)| memory.tags.contains(&tag))
                .map(|(_, v)| v.clone())
                .collect()
        }))
    }

    /// Delete a memory by ID
    #[tool("Delete a memory by ID")]
    #[update]
    async fn forget(id: String) -> Result<bool, String> {
        MEMORIES.with(|m| {
            match m.borrow_mut().remove(&id) {
                Some(_) => Ok(true),
                None => Err(format!("Memory with ID {} not found", id))
            }
        })
    }

    /// Get total number of stored memories
    #[tool("Get total number of stored memories")]
    #[query]
    async fn count() -> Result<u64, String> {
        Ok(MEMORIES.with(|m| m.borrow().len()))
    }
}

// Candid interface is automatically exported by the #[icarus_tools] macro
"#;

    fs::write(target, content)?;
    Ok(())
}

fn create_cargo_toml(
    project_path: &Path,
    name: &str,
    local_sdk: Option<String>,
    with_tests: bool,
) -> Result<()> {
    // Create Cargo.toml from template
    let content = create_cargo_toml_content(name, local_sdk, with_tests)?;

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
    fallback.insert("pocket-ic".to_string(), "9".to_string());
    fallback.insert("tokio".to_string(), "1".to_string());
    Ok(fallback)
}

fn create_cargo_toml_content(
    name: &str,
    local_sdk: Option<String>,
    with_tests: bool,
) -> Result<String> {
    let versions = get_workspace_versions()?;

    let get_version = |dep: &str| -> String {
        versions.get(dep).cloned().unwrap_or_else(|| match dep {
            "ic-cdk" => "0.18".to_string(),
            "ic-cdk-macros" => "0.18".to_string(),
            "ic-stable-structures" => "0.7".to_string(),
            "candid" => "0.10".to_string(),
            "serde" => "1.0".to_string(),
            "serde_json" => "1.0".to_string(),
            "pocket-ic" => "9".to_string(),
            "tokio" => "1".to_string(),
            _ => "1.0".to_string(),
        })
    };

    let dev_dependencies_section = if with_tests {
        format!(
            r#"

[dev-dependencies]
pocket-ic = "{}"
candid = "{}"
tokio = {{ version = "{}", features = ["full"] }}
"#,
            get_version("pocket-ic"),
            get_version("candid"),
            get_version("tokio")
        )
    } else {
        "".to_string()
    };

    let icarus_dep = if let Some(ref sdk) = local_sdk {
        format!("{{ path = \"{}\" }}", sdk)
    } else {
        let cli_version = env!("CARGO_PKG_VERSION");
        format!(
            "{{ version = \"{}\", features = [\"canister\"] }}",
            cli_version
        )
    };

    Ok(format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[package.metadata.icarus]
claude_desktop.auto_update = true
claude_desktop.config_path = ""

[dependencies]
icarus = {}
ic-cdk = "{}"
ic-cdk-macros = "{}"
ic-stable-structures = "{}"
candid = "{}"
serde = {{ version = "{}", features = ["derive"] }}
serde_json = "{}"{}

[lib]
crate-type = ["cdylib"]
"#,
        name,
        icarus_dep,
        get_version("ic-cdk"),
        get_version("ic-cdk-macros"),
        get_version("ic-stable-structures"),
        get_version("candid"),
        get_version("serde"),
        get_version("serde_json"),
        dev_dependencies_section
    ))
}

fn create_dfx_json(project_path: &Path, name: &str) -> Result<()> {
    let dfx_json = format!(
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
    );
    std::fs::write(project_path.join("dfx.json"), dfx_json)?;
    Ok(())
}

fn create_minimal_candid_file(src_dir: &Path, name: &str) -> Result<()> {
    // Create a minimal .did file with just the essential endpoints
    // The #[icarus_module] macro generates all the actual endpoints
    let candid_content = r#"// Minimal Candid interface for Icarus MCP server
// The actual interface is generated by the #[icarus_module] macro

service : (principal) -> {
  // MCP tool discovery endpoint
  list_tools : () -> (text) query;
  
  // Auth management endpoints (generated by icarus_module)
  add_authorized_user : (text, text) -> (variant { Ok : text; Err : text });
  remove_authorized_user : (text) -> (variant { Ok : text; Err : text });
  update_user_role : (text, text) -> (variant { Ok : text; Err : text });
  list_authorized_users : () -> (text) query;
  get_auth_status : () -> (text) query;
}"#;

    let candid_path = src_dir.join(format!("{}.did", name));
    std::fs::write(candid_path, candid_content)?;
    Ok(())
}

fn create_readme(project_path: &Path, name: &str) -> Result<()> {
    let readme = format!(
        r#"# {}

An MCP (Model Context Protocol) server running on the Internet Computer.

## Development

### Prerequisites

- [Rust](https://rustup.rs/)
- [dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install)
- [Icarus CLI](https://crates.io/crates/icarus-cli)

For updating Candid interfaces after modifying tools:
- `cargo install candid-extractor`
- `cargo install generate-did`

### Updating the Candid Interface

After modifying your tool functions in `src/lib.rs`:

```bash
# Build the WASM
cargo build --target wasm32-unknown-unknown --release

# Update the .did file
generate-did .
```

This extracts the Candid interface from your WASM and updates the .did file.

### Local Deployment

```bash
# Start local Internet Computer
dfx start --clean

# Deploy the canister
icarus deploy --network local
```

### Using with Claude Desktop

After deployment, register your canister with Claude Desktop:

```bash
icarus bridge start --canister-id <your-canister-id>
```

Then add the bridge configuration to Claude Desktop's config file.

## Project Structure

- `src/lib.rs` - Main canister code with MCP tool implementations
- `dfx.json` - Internet Computer configuration
- `Cargo.toml` - Project dependencies and metadata

## License

See LICENSE file for details.
"#,
        name
    );
    std::fs::write(project_path.join("README.md"), readme)?;
    Ok(())
}
