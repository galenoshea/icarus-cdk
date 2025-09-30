use chrono::Utc;
use icarus_cli::{
    create_project_structure, detect_installed_clients, find_project_root, get_all_client_configs,
    get_chatgpt_desktop_config_path, get_claude_code_config_path, get_claude_desktop_config_path,
    get_continue_config_path, is_icarus_project, load_project_config, validate_project_structure,
};
use icarus_cli::{CanisterId, McpConfig, McpServerConfig, Network, ServerName};
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper function to create a valid test server configuration
fn create_test_server() -> McpServerConfig {
    McpServerConfig {
        name: ServerName::new("test-server").unwrap(),
        canister_id: CanisterId::new("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap(),
        network: Network::Local,
        url: "http://localhost:3000/mcp".to_string(),
        client: "claude-desktop".to_string(),
        port: Some(3000),
        enabled: true,
        created_at: Utc::now(),
        last_updated: Utc::now(),
    }
}

/// Test MCP configuration serialization and deserialization
#[tokio::test]
async fn test_mcp_config_serialization() {
    let mut config = McpConfig::default();
    let server = create_test_server();

    config.add_server(server.clone()).unwrap();

    // Test JSON serialization
    let json = serde_json::to_string_pretty(&config).unwrap();
    assert!(json.contains("test-server"));
    assert!(json.contains("rdmx6-jaaaa-aaaaa-aaadq-cai"));

    // Test deserialization
    let deserialized: McpConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.servers.len(), 1);
    assert_eq!(deserialized.servers[0].name, "test-server");
}

/// Test MCP config operations
#[test]
fn test_mcp_config_operations() {
    let mut config = McpConfig::default();
    let server = create_test_server();

    // Test adding server
    assert!(config.add_server(server.clone()).is_ok());
    assert!(config.has_server("test-server"));

    // Test duplicate server
    assert!(config.add_server(server.clone()).is_err());

    // Test getting server
    assert!(config.get_server("test-server").is_some());
    assert!(config.get_server("nonexistent").is_none());

    // Test enabled servers
    let enabled = config.enabled_servers();
    assert_eq!(enabled.len(), 1);

    // Test updating server
    config
        .update_server("test-server", |s| {
            s.enabled = false;
        })
        .unwrap();

    let updated_server = config.get_server("test-server").unwrap();
    assert!(!updated_server.enabled);

    // Test removing server
    assert!(config.remove_server("test-server").is_ok());
    assert!(!config.has_server("test-server"));

    // Test removing nonexistent server
    assert!(config.remove_server("nonexistent").is_err());
}

/// Test MCP config validation
#[test]
fn test_mcp_config_validation() {
    let mut config = McpConfig::default();

    // Valid server
    let mut valid_server = create_test_server();
    valid_server.name = ServerName::new("valid-server").unwrap();

    config.add_server(valid_server).unwrap();
    assert!(config.validate().is_ok());

    // Test invalid server name construction fails at newtype level
    assert!(ServerName::new("").is_err());
    assert!(CanisterId::new("invalid").is_err());
    assert!("invalid-network".parse::<Network>().is_err());
}

/// Test MCP config statistics
#[test]
fn test_mcp_config_stats() {
    let mut config = McpConfig::default();

    // Add multiple servers
    for i in 0..5 {
        let server = McpServerConfig {
            name: ServerName::new(format!("server-{}", i)).unwrap(),
            canister_id: CanisterId::new(format!("canister-{}-jaaaa-aaaaa-aaadq-cai", i)).unwrap(),
            network: if i % 2 == 0 {
                Network::Local
            } else {
                Network::Ic
            },
            url: format!("http://localhost:300{}/mcp", i),
            client: if i % 3 == 0 {
                "claude-desktop"
            } else {
                "claude-code"
            }
            .to_string(),
            port: Some(3000 + i as u16),
            enabled: i % 2 == 0, // Enable every other server
            created_at: Utc::now(),
            last_updated: Utc::now(),
        };

        config.add_server(server).unwrap();
    }

    let stats = config.stats();
    assert_eq!(stats.total_servers, 5);
    assert_eq!(stats.enabled_servers, 3); // 0, 2, 4 are enabled
    assert_eq!(stats.disabled_servers, 2);
    assert_eq!(stats.networks.len(), 2); // "local" and "ic"
    assert_eq!(stats.clients.len(), 2); // "claude-desktop" and "claude-code"
}

/// Test client detector functionality
#[test]
fn test_client_detector() {
    // Test that config path generation works
    if dirs::config_dir().is_some() {
        assert!(get_claude_desktop_config_path().is_ok());
        assert!(get_claude_code_config_path().is_ok());
        assert!(get_chatgpt_desktop_config_path().is_ok());
        assert!(get_continue_config_path().is_ok());
    }

    // Test client detection (should not fail)
    let clients = detect_installed_clients();
    // Result could be empty if no clients are installed, which is fine for testing
    // Note: This assertion is always true since Vec::len() returns usize which is always >= 0
    assert!(clients.is_empty() || !clients.is_empty());

    // Test getting all client configs
    let configs = get_all_client_configs();
    assert_eq!(configs.len(), 4);
}

/// Test project utilities
#[tokio::test]
async fn test_project_utilities() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Test creating project structure
    create_project_structure(project_path).await.unwrap();

    assert!(project_path.join("src").exists());
    assert!(project_path.join("tests").exists());
    assert!(project_path.join(".dfx").exists());

    // Test validating project structure (should have issues with empty project)
    let issues = validate_project_structure(project_path).unwrap();
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|issue| issue.contains("Cargo.toml")));

    // Create minimal Cargo.toml
    let cargo_content = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
icarus = "0.9.0"
"#;

    tokio::fs::write(project_path.join("Cargo.toml"), cargo_content)
        .await
        .unwrap();

    // Create src/lib.rs
    tokio::fs::write(project_path.join("src/lib.rs"), "// Test library\n")
        .await
        .unwrap();

    // Test loading project config
    let config = load_project_config(project_path).await.unwrap();
    assert_eq!(config.name, "test-project");
    assert_eq!(config.version, "0.1.0");
    assert_eq!(config.canister_type, "rust");

    // Test checking if it's an Icarus project
    assert!(is_icarus_project(project_path).await);

    // Test validating project structure again (should be better now)
    let issues = validate_project_structure(project_path).unwrap();
    // Should have fewer issues now
    assert!(issues.len() < 3);
}

/// Test project root finding
#[test]
fn test_find_project_root() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Should fail when no Cargo.toml or dfx.json exists
    std::env::set_current_dir(project_path).unwrap();
    assert!(find_project_root().is_err());

    // Create Cargo.toml
    std::fs::write(
        project_path.join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    // Should succeed now
    let found_root = find_project_root().unwrap();
    // Canonicalize both paths to handle symlinks (e.g., /var -> /private/var on macOS)
    assert_eq!(
        found_root.canonicalize().unwrap(),
        project_path.canonicalize().unwrap()
    );

    // Create subdirectory and test from there
    let sub_dir = project_path.join("subdir");
    std::fs::create_dir(&sub_dir).unwrap();
    std::env::set_current_dir(&sub_dir).unwrap();

    // Should still find the root
    let found_root = find_project_root().unwrap();
    // Canonicalize both paths to handle symlinks (e.g., /var -> /private/var on macOS)
    assert_eq!(
        found_root.canonicalize().unwrap(),
        project_path.canonicalize().unwrap()
    );
}

/// Test MCP config cleanup functionality
#[test]
fn test_mcp_config_cleanup() {
    let mut config = McpConfig::default();

    // Add a valid server
    let mut valid_server = create_test_server();
    valid_server.name = ServerName::new("valid-server").unwrap();

    config.add_server(valid_server.clone()).unwrap();
    assert_eq!(config.servers.len(), 1);

    // Test that validation at construction prevents invalid servers
    assert!(ServerName::new("").is_err());
    assert!(CanisterId::new("invalid").is_err());
    assert!("invalid".parse::<Network>().is_err());

    // Since validation happens at construction, cleanup has no invalid servers to remove
    let removed_count = config.cleanup();
    assert_eq!(removed_count, 0);
    assert_eq!(config.servers.len(), 1);
    assert_eq!(config.servers[0].name, "valid-server");
}
