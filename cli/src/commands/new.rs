use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

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
    println!("  icarus build");
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

    // Create Cargo.toml with test dependencies
    let dev_dependencies_section = if with_tests {
        r#"

[dev-dependencies]
pocket-ic = "4.0"
candid = "0.10"
tokio = { version = "1", features = ["full"] }
"#
        .to_string()
    } else {
        String::new()
    };

    // Determine the SDK paths for both icarus and icarus-canister
    let icarus_dep = if let Some(ref sdk) = local_sdk {
        // Use provided SDK path for local development
        let base_path = if sdk.ends_with("icarus-sdk") {
            sdk.clone()
        } else {
            sdk.clone()
        };
        format!("{{ path = \"{}\" }}", base_path)
    } else {
        // Use the same version as the CLI from crates.io
        let cli_version = env!("CARGO_PKG_VERSION");
        format!("\"{}\"", cli_version)
    };

    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
icarus = {}
ic-cdk = "0.16"
ic-cdk-macros = "0.16"
candid = "0.10"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
ic-stable-structures = "0.6"{}

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = 'z'       # Optimize for size
lto = true            # Enable link-time optimization  
codegen-units = 1     # Single codegen unit for better optimization
strip = "debuginfo"   # Strip debug info
panic = "abort"       # Smaller binaries, matches WASM behavior
overflow-checks = false # Disable runtime overflow checks
"#,
        name, icarus_dep, dev_dependencies_section
    );
    std::fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

    // Create lib.rs with Memento - a simple key-value memory storage tool
    let lib_rs = r#"//! Memento - A simple key-value memory storage tool for the Internet Computer

use icarus::prelude::*;

// Define storage using stable_storage macro
stable_storage! {
    MEMORIES: StableBTreeMap<String, MemoryEntry, Memory> = memory_id!(0);
}

/// Simple memory entry
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusType)]
pub struct MemoryEntry {
    pub key: String,
    pub content: String,
    pub created_by: Principal,
    pub created_at: u64,
}

#[icarus_module]
mod tools {
    use super::*;
    
    /// Store a memory with a unique key
    #[update]
    #[icarus_tool("Store a memory with a unique key")]
    pub fn memorize(key: String, content: String) -> Result<String, String> {
        // Require User role or higher (User, Admin, Owner can write)
        require_role_or_higher(AuthRole::User);
        let caller = ic_cdk::caller();
        
        // Validate inputs
        if key.trim().is_empty() {
            return Err("Key cannot be empty".to_string());
        }
        if content.trim().is_empty() {
            return Err("Content cannot be empty".to_string());
        }
        if content.len() > 50_000 {
            return Err("Content too large (max 50KB)".to_string());
        }
        
        // Check for duplicate key
        MEMORIES.with(|m| {
            if m.borrow().contains_key(&key) {
                return Err(format!("Key '{}' already exists", key));
            }
            
            let memory = MemoryEntry {
                key: key.clone(),
                content,
                created_by: caller,
                created_at: api::time(),
            };
            
            m.borrow_mut().insert(key.clone(), memory);
            Ok(key)
        })
    }
    
    /// Retrieve a memory by its key
    #[query]
    #[icarus_tool("Retrieve a memory by its key")]
    pub fn recall(key: String) -> Option<MemoryEntry> {
        // Anyone except ReadOnly can recall memories
        require_none_of_roles(&[AuthRole::ReadOnly]);
        
        MEMORIES.with(|m| m.borrow().get(&key))
    }
    
    /// List all stored memories
    #[query]
    #[icarus_tool("List all stored memories")]
    pub fn list() -> Vec<MemoryEntry> {
        // Anyone except ReadOnly can list memories
        require_none_of_roles(&[AuthRole::ReadOnly]);
        
        MEMORIES.with(|m| {
            m.borrow()
                .iter()
                .map(|(_, memory)| memory)
                .collect()
        })
    }
    
    /// Remove a memory by its key
    #[update]
    #[icarus_tool("Remove a memory by its key")]
    pub fn forget(key: String) -> Result<bool, String> {
        // User or higher can forget individual memories
        require_role_or_higher(AuthRole::User);
        
        MEMORIES.with(|m| {
            Ok(m.borrow_mut().remove(&key).is_some())
        })
    }
    
    /// Clear all memories
    #[update]
    #[icarus_tool("Clear all memories")]
    pub fn forget_all() -> Result<u64, String> {
        // Only Admin or Owner can clear all memories
        require_role_or_higher(AuthRole::Admin);
        
        MEMORIES.with(|m| {
            let count = m.borrow().len();
            m.borrow_mut().clear_new();
            Ok(count)
        })
    }
}

// Export candid interface
ic_cdk::export_candid!();
"#;
    std::fs::write(src_dir.join("lib.rs"), lib_rs)?;

    // Create dfx.json
    let dfx_json = format!(
        r#"{{
  "canisters": {{
    "{}": {{
      "type": "rust",
      "package": "{}",
      "candid": "src/{}.did",
      "optimize": "cycles"
    }}
  }},
  "defaults": {{
    "build": {{
      "args": "",
      "packtool": ""
    }}
  }},
  "version": 1
}}
"#,
        name, name, name
    );
    std::fs::write(project_path.join("dfx.json"), dfx_json)?;

    // Candid file will be generated during build using candid-extractor

    // Create README.md
    let readme = format!(
        r#"# {}

A memory canister for the Internet Computer with bridge registration functionality.

## Features

- **Memory Storage**: Store and retrieve memories with tags
- **Bridge Registration**: Secure bridge registration with delegation tokens
- **Access Control**: Owner and authorized bridge access validation
- **Stable Storage**: Persistent data across canister upgrades

## Development

### Prerequisites

- [DFX](https://internetcomputer.org/docs/current/developer-docs/setup/install/) (Internet Computer SDK)
- Rust with `wasm32-unknown-unknown` target

### Building

```bash
# Build the canister
dfx build

# Or using cargo directly
cargo build --target wasm32-unknown-unknown --release
```

### Testing

First, download the PocketIC binary:

```bash
# Download PocketIC binary for your platform
# macOS Apple Silicon
curl -L https://github.com/dfinity/pocketic/releases/download/4.0.0/pocket-ic-x86_64-darwin.gz -o pocket-ic.gz
# macOS Intel
curl -L https://github.com/dfinity/pocketic/releases/download/4.0.0/pocket-ic-x86_64-darwin.gz -o pocket-ic.gz
# Linux
curl -L https://github.com/dfinity/pocketic/releases/download/4.0.0/pocket-ic-x86_64-linux.gz -o pocket-ic.gz

# Extract and make executable
gunzip pocket-ic.gz
chmod +x pocket-ic

# Run the tests
cargo test
```

### Deploying

```bash
# Start local replica
dfx start --clean

# Deploy to local network
dfx deploy

# Deploy to IC mainnet
dfx deploy --network ic
```

## Bridge Registration

This canister supports bridge registration using delegation tokens from the Icarus marketplace:

1. **Purchase Tool**: Purchase the tool from the Icarus marketplace to receive a delegation token
2. **Register Bridge**: Use the delegation token to register your bridge with the canister
3. **Access Tools**: Once registered, the bridge can access all tool functions on behalf of the owner

### Bridge Management

#### For Canister Owners

```candid
// View all authorized bridges (owner only)
get_authorized_bridges() -> (vec BridgeRegistration)

// Revoke bridge access (owner only)
revoke_bridge(bridge_principal: principal) -> (variant {{ Ok: bool; Err: text }})

// Remove bridge completely (owner only)
remove_bridge(bridge_principal: principal) -> (variant {{ Ok: bool; Err: text }})
```

#### For Bridges

```candid
// Register using delegation token from marketplace
register_bridge(token: DelegationToken) -> (variant {{ Ok: null; Err: text }})
```

### Security Features

- **Token Validation**: Delegation tokens are cryptographically verified
- **Expiration**: Tokens have configurable expiration times
- **Access Control**: Only registered bridges can access tool functions
- **Owner Override**: Canister owner can always access and manage bridges

## Candid Interface

### Tool Functions

- `memorize(content: text, tags: opt vec text) -> (variant {{ Ok: text; Err: text }})`
- `forget(id: text) -> (variant {{ Ok: bool; Err: text }})`
- `forget_oldest() -> (variant {{ Ok: bool; Err: text }})`
- `recall_latest() -> (opt MemoryEntry)`
- `list() -> (vec MemoryEntry)`
- `get_metadata() -> (text)`

### Bridge Management Functions

- `register_bridge(token: DelegationToken) -> (variant {{ Ok: null; Err: text }})`
- `get_authorized_bridges() -> (vec BridgeRegistration)` (owner only)
- `revoke_bridge(bridge_principal: principal) -> (variant {{ Ok: bool; Err: text }})` (owner only)
- `remove_bridge(bridge_principal: principal) -> (variant {{ Ok: bool; Err: text }})` (owner only)

### Data Types

```candid
type MemoryEntry = record {{
    id: text;
    content: text;
    created_at: nat64;
    tags: vec text;
}};

type DelegationToken = record {{
    owner: principal;
    canister_id: principal;
    tool_id: text;
    expiration: nat64;
    nonce: nat64;
    signature: vec nat8;
    created_at: nat64;
}};

type BridgeRegistration = record {{
    bridge_principal: principal;
    owner: principal;
    token_nonce: nat64;
    registered_at: nat64;
    last_used: opt nat64;
    active: bool;
}};
```

## License

MIT
"#,
        name
    );
    std::fs::write(project_path.join("README.md"), readme)?;

    // Create integration test file if requested
    if with_tests {
        let tests_dir = project_path.join("tests");
        let integration_test = format!(
            r#"//! Integration tests for the {} canister using PocketIC

use candid::{{encode_args, decode_args, CandidType, Principal, Deserialize}};
use pocket_ic::{{PocketIc, WasmResult}};

// Define types matching the canister
#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct MemoryEntry {{
    pub id: String,
    pub content: String,
    pub created_at: u64,
    pub tags: Vec<String>,
}}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct DelegationToken {{
    pub owner: Principal,
    pub canister_id: Principal,
    pub tool_id: String,
    pub expiration: u64,
    pub nonce: u64,
    pub signature: Vec<u8>,
    pub created_at: u64,
}}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct BridgeRegistration {{
    pub bridge_principal: Principal,
    pub owner: Principal,
    pub token_nonce: u64,
    pub registered_at: u64,
    pub last_used: Option<u64>,
    pub active: bool,
}}

// Define the Result types for Candid decoding
#[derive(CandidType, candid::Deserialize)]
enum MemorizeResult {{
    Ok(String),
    Err(String),
}}

#[derive(CandidType, candid::Deserialize)]
enum ForgetResult {{
    Ok(bool),
    Err(String),
}}

#[derive(CandidType, candid::Deserialize)]
enum BridgeResult {{
    Ok(()),
    Err(String),
}}

#[derive(CandidType, candid::Deserialize)]
enum BridgeRemovalResult {{
    Ok(bool),
    Err(String),
}}

#[test]
fn test_memorize_and_recall() {{
    let pic = PocketIc::new();
    
    let canister_id = pic.create_canister();
    let wasm_module = include_bytes!("../target/wasm32-unknown-unknown/release/{}.wasm");
    
    // Install canister with owner
    let owner = Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap();
    let init_args = encode_args((owner,)).unwrap();
    pic.install_canister(canister_id, wasm_module.to_vec(), init_args, None);
    
    // Test memorizing a fact as owner
    let args = encode_args(("The sky is blue".to_string(), Some(vec!["color".to_string(), "nature".to_string()]))).unwrap();
    let result = pic.update_call(
        canister_id,
        owner,
        "memorize",
        args
    ).unwrap();
    
    match result {{
        WasmResult::Reply(bytes) => {{
            let memory_id: (MemorizeResult,) = decode_args(&bytes).unwrap();
            match memory_id.0 {{
                MemorizeResult::Ok(id) => {{
                    // Test recalling the latest memory
                    let args = encode_args(()).unwrap();
                    let result = pic.query_call(
                        canister_id,
                        owner,
                        "recall_latest",
                        args
                    ).unwrap();
                    
                    match result {{
                        WasmResult::Reply(bytes) => {{
                            let memory: (Option<MemoryEntry>,) = decode_args(&bytes).unwrap();
                            assert!(memory.0.is_some());
                            assert_eq!(memory.0.unwrap().content, "The sky is blue");
                        }}
                        WasmResult::Reject(msg) => panic!("Query rejected: {{}}", msg),
                    }}
                }}
                MemorizeResult::Err(e) => panic!("Memorize failed: {{}}", e),
            }}
        }}
        WasmResult::Reject(msg) => panic!("Update rejected: {{}}", msg),
    }}
}}

#[test]
fn test_unauthorized_access() {{
    let pic = PocketIc::new();
    
    let canister_id = pic.create_canister();
    let wasm_module = include_bytes!("../target/wasm32-unknown-unknown/release/{}.wasm");
    
    let owner = Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap();
    let unauthorized_user = Principal::from_text("rrkah-fqaaa-aaaah-qcaiq-cai").unwrap();
    
    let init_args = encode_args((owner,)).unwrap();
    pic.install_canister(canister_id, wasm_module.to_vec(), init_args, None);
    
    // Test that unauthorized user cannot memorize
    let args = encode_args(("Unauthorized content".to_string(), None::<Vec<String>>)).unwrap();
    let result = pic.update_call(
        canister_id,
        unauthorized_user,
        "memorize",
        args
    ).unwrap();
    
    match result {{
        WasmResult::Reply(bytes) => {{
            let memory_result: (MemorizeResult,) = decode_args(&bytes).unwrap();
            match memory_result.0 {{
                MemorizeResult::Ok(_) => panic!("Should have failed authorization"),
                MemorizeResult::Err(err) => assert!(err.contains("Unauthorized")),
            }}
        }}
        WasmResult::Reject(_) => {{
            // This is also acceptable for authorization failures
        }}
    }}
}}

#[test]
fn test_bridge_registration() {{
    let pic = PocketIc::new();
    
    let canister_id = pic.create_canister();
    let wasm_module = include_bytes!("../target/wasm32-unknown-unknown/release/{}.wasm");
    
    let owner = Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap();
    let bridge_principal = Principal::from_text("rrkah-fqaaa-aaaah-qcaiq-cai").unwrap();
    
    let init_args = encode_args((owner,)).unwrap();
    pic.install_canister(canister_id, wasm_module.to_vec(), init_args, None);
    
    // Create a delegation token
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    
    let delegation_token = DelegationToken {{
        owner,
        canister_id,
        tool_id: "test_tool".to_string(),
        expiration: now + 365 * 24 * 60 * 60 * 1_000_000_000, // 1 year
        nonce: 12345,
        signature: vec![1, 2, 3, 4], // Simple non-empty signature
        created_at: now,
    }};
    
    // Register bridge
    let args = encode_args((delegation_token,)).unwrap();
    let result = pic.update_call(
        canister_id,
        bridge_principal,
        "register_bridge",
        args
    ).unwrap();
    
    match result {{
        WasmResult::Reply(bytes) => {{
            let bridge_result: (BridgeResult,) = decode_args(&bytes).unwrap();
            match bridge_result.0 {{
                BridgeResult::Ok(()) => {{
                    // Success - now test that bridge can access tools
                    let args = encode_args(("Bridge memory".to_string(), None::<Vec<String>>)).unwrap();
                    let result = pic.update_call(
                        canister_id,
                        bridge_principal,
                        "memorize",
                        args
                    ).unwrap();
                    
                    match result {{
                        WasmResult::Reply(bytes) => {{
                            let memory_result: (MemorizeResult,) = decode_args(&bytes).unwrap();
                            match memory_result.0 {{
                                MemorizeResult::Ok(_) => {{
                                    // Success - bridge can now access tools
                                }},
                                MemorizeResult::Err(e) => panic!("Bridge should be authorized: {{}}", e),
                            }}
                        }}
                        WasmResult::Reject(msg) => panic!("Bridge call rejected: {{}}", msg),
                    }}
                }},
                BridgeResult::Err(e) => panic!("Bridge registration failed: {{}}", e),
            }}
        }}
        WasmResult::Reject(msg) => panic!("Bridge registration rejected: {{}}", msg),
    }}
}}

#[test]
fn test_bridge_management() {{
    let pic = PocketIc::new();
    
    let canister_id = pic.create_canister();
    let wasm_module = include_bytes!("../target/wasm32-unknown-unknown/release/{}.wasm");
    
    let owner = Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap();
    let bridge_principal = Principal::from_text("rrkah-fqaaa-aaaah-qcaiq-cai").unwrap();
    
    let init_args = encode_args((owner,)).unwrap();
    pic.install_canister(canister_id, wasm_module.to_vec(), init_args, None);
    
    // Register bridge first
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    
    let delegation_token = DelegationToken {{
        owner,
        canister_id,
        tool_id: "test_tool".to_string(),
        expiration: now + 365 * 24 * 60 * 60 * 1_000_000_000,
        nonce: 12345,
        signature: vec![1, 2, 3, 4],
        created_at: now,
    }};
    
    let args = encode_args((delegation_token,)).unwrap();
    pic.update_call(canister_id, bridge_principal, "register_bridge", args).unwrap();
    
    // Test getting authorized bridges (only owner should see them)
    let args = encode_args(()).unwrap();
    let result = pic.query_call(
        canister_id,
        owner,
        "get_authorized_bridges",
        args
    ).unwrap();
    
    match result {{
        WasmResult::Reply(bytes) => {{
            let bridges: (Vec<BridgeRegistration>,) = decode_args(&bytes).unwrap();
            assert_eq!(bridges.0.len(), 1);
            assert_eq!(bridges.0[0].bridge_principal, bridge_principal);
            assert!(bridges.0[0].active);
        }}
        WasmResult::Reject(msg) => panic!("Query rejected: {{}}", msg),
    }}
    
    // Test revoking bridge access
    let args = encode_args((bridge_principal,)).unwrap();
    let result = pic.update_call(
        canister_id,
        owner,
        "revoke_bridge",
        args
    ).unwrap();
    
    match result {{
        WasmResult::Reply(bytes) => {{
            let revoke_result: (BridgeRemovalResult,) = decode_args(&bytes).unwrap();
            match revoke_result.0 {{
                BridgeRemovalResult::Ok(success) => assert!(success),
                BridgeRemovalResult::Err(e) => panic!("Revoke failed: {{}}", e),
            }}
        }}
        WasmResult::Reject(msg) => panic!("Revoke rejected: {{}}", msg),
    }}
    
    // Verify bridge can no longer access tools
    let args = encode_args(("Should fail".to_string(), None::<Vec<String>>)).unwrap();
    let result = pic.update_call(
        canister_id,
        bridge_principal,
        "memorize",
        args
    ).unwrap();
    
    match result {{
        WasmResult::Reply(bytes) => {{
            let memory_result: (MemorizeResult,) = decode_args(&bytes).unwrap();
            match memory_result.0 {{
                MemorizeResult::Ok(_) => panic!("Bridge should no longer be authorized"),
                MemorizeResult::Err(err) => assert!(err.contains("Unauthorized")),
            }}
        }}
        WasmResult::Reject(_) => {{
            // This is also acceptable for authorization failures
        }}
    }}
}}

#[test]
fn test_list_all_memories() {{
    let pic = PocketIc::new();
    
    let canister_id = pic.create_canister();
    let wasm_module = include_bytes!("../target/wasm32-unknown-unknown/release/{}.wasm");
    
    let owner = Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap();
    let init_args = encode_args((owner,)).unwrap();
    pic.install_canister(canister_id, wasm_module.to_vec(), init_args, None);
    
    // Add multiple memories as owner
    for i in 0..5 {{
        let args = encode_args((format!("Memory {{}}", i), None::<Vec<String>>)).unwrap();
        pic.update_call(
            canister_id,
            owner,
            "memorize",
            args
        ).unwrap();
    }}
    
    // List all memories
    let args = encode_args(()).unwrap();
    let result = pic.query_call(
        canister_id,
        owner,
        "list",
        args
    ).unwrap();
    
    match result {{
        WasmResult::Reply(bytes) => {{
            let memories: (Vec<MemoryEntry>,) = decode_args(&bytes).unwrap();
            assert_eq!(memories.0.len(), 5);
        }}
        WasmResult::Reject(msg) => panic!("Query rejected: {{}}", msg),
    }}
}}
"#,
            name,
            name.replace('-', "_"),
            name.replace('-', "_"),
            name.replace('-', "_"),
            name.replace('-', "_"),
            name.replace('-', "_")
        );
        std::fs::write(tests_dir.join("integration_test.rs"), integration_test)?;
    }

    Ok(())
}
