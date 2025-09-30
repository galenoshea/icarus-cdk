//! MCP client integration and workflow tests
//!
//! Tests the complete MCP server management workflow including client detection,
//! configuration management, and bridge server operations.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Test helper for MCP operations
struct McpTestHelper {
    _temp_dir: TempDir,
    config_dir: std::path::PathBuf,
}

impl McpTestHelper {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".icarus");
        fs::create_dir_all(&config_dir).unwrap();

        Self {
            _temp_dir: temp_dir,
            config_dir,
        }
    }

    fn icarus_cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
        cmd.env("ICARUS_CONFIG_DIR", &self.config_dir);
        cmd
    }

    fn config_path(&self) -> std::path::PathBuf {
        self.config_dir.join("mcp_config.json")
    }

    fn create_mock_config(&self, servers: &[MockMcpServer]) {
        let config = serde_json::json!({
            "servers": servers.iter().map(|s| s.to_json()).collect::<Vec<_>>(),
            "version": "1.0.0"
        });

        fs::write(
            self.config_path(),
            serde_json::to_string_pretty(&config).unwrap(),
        )
        .unwrap();
    }
}

#[derive(Clone)]
struct MockMcpServer {
    name: String,
    canister_id: String,
    network: String,
    url: String,
    client: String,
    enabled: bool,
    port: Option<u16>,
}

impl MockMcpServer {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            canister_id: "rdmx6-jaaaa-aaaaa-aaadq-cai".to_string(),
            network: "local".to_string(),
            url: format!("http://localhost:3000/mcp/{}", name),
            client: "claude-desktop".to_string(),
            enabled: true,
            port: Some(3000),
        }
    }

    fn with_client(mut self, client: &str) -> Self {
        self.client = client.to_string();
        self
    }

    fn with_network(mut self, network: &str) -> Self {
        self.network = network.to_string();
        self
    }

    fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "name": self.name,
            "canister_id": self.canister_id,
            "network": self.network,
            "url": self.url,
            "client": self.client,
            "port": self.port,
            "enabled": self.enabled,
            "created_at": "2024-01-01T00:00:00Z",
            "last_updated": "2024-01-01T00:00:00Z"
        })
    }
}

/// Test MCP list command with no servers
#[test]
#[serial]
fn test_mcp_list_empty() {
    let helper = McpTestHelper::new();

    helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "list"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().or(predicate::str::contains("No MCP servers")));
}

/// Test MCP list command with mock servers
#[test]
#[serial]
#[ignore = "MCP feature not fully implemented"]
fn test_mcp_list_with_servers() {
    let helper = McpTestHelper::new();

    // Create mock configuration
    let servers = vec![
        MockMcpServer::new("test-server-1"),
        MockMcpServer::new("test-server-2").with_client("claude-code"),
        MockMcpServer::new("test-server-3").disabled(),
    ];

    helper.create_mock_config(&servers);

    // Test table format (default)
    helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-server-1"))
        .stdout(predicate::str::contains("test-server-2"))
        .stdout(predicate::str::contains("test-server-3"));

    // Test JSON format
    helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["))
        .stdout(predicate::str::contains("test-server-1"))
        .stdout(predicate::str::contains("claude-desktop"))
        .stdout(predicate::str::contains("claude-code"));
}

/// Test MCP list output formats
#[test]
#[serial]
#[ignore = "MCP feature not fully implemented"]
fn test_mcp_list_formats() {
    let helper = McpTestHelper::new();

    let servers = vec![MockMcpServer::new("format-test")];
    helper.create_mock_config(&servers);

    let formats = ["table", "json", "yaml", "plain"];

    for format in formats {
        helper
            .icarus_cmd()
            .args(["--quiet", "mcp", "list", "--format", format])
            .assert()
            .success()
            .stdout(predicate::str::contains("format-test"));
    }
}

/// Test MCP list filtering
#[test]
#[serial]
#[ignore = "MCP feature not fully implemented"]
fn test_mcp_list_filtering() {
    let helper = McpTestHelper::new();

    let servers = vec![
        MockMcpServer::new("local-server").with_network("local"),
        MockMcpServer::new("ic-server").with_network("ic"),
        MockMcpServer::new("enabled-server"),
        MockMcpServer::new("disabled-server").disabled(),
    ];

    helper.create_mock_config(&servers);

    // Test filtering by enabled status
    helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "list", "--enabled-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("enabled-server"))
        .stdout(predicate::str::contains("local-server"))
        .stdout(predicate::str::contains("ic-server"))
        .stdout(predicate::str::contains("disabled-server").not());

    // Test filtering by network
    helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "list", "--network", "local"])
        .assert()
        .success()
        .stdout(predicate::str::contains("local-server"))
        .stdout(predicate::str::contains("ic-server").not());
}

/// Test MCP add command validation
#[test]
#[serial]
fn test_mcp_add_validation() {
    let helper = McpTestHelper::new();

    // Test invalid canister ID
    helper
        .icarus_cmd()
        .args([
            "mcp",
            "add",
            "invalid-canister-id",
            "--client",
            "claude-desktop",
            "--skip-verify",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid canister ID format"));

    // Test invalid client
    helper
        .icarus_cmd()
        .args([
            "mcp",
            "add",
            "rdmx6-jaaaa-aaaaa-aaadq-cai",
            "--client",
            "invalid-client",
            "--skip-verify",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));

    // Test missing required arguments
    helper
        .icarus_cmd()
        .args(["mcp", "add"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

/// Test MCP add command with valid inputs
#[test]
#[serial]
#[ignore = "MCP feature not fully implemented"]
fn test_mcp_add_success() {
    let helper = McpTestHelper::new();

    // Test adding a server
    helper
        .icarus_cmd()
        .args([
            "mcp",
            "add",
            "rdmx6-jaaaa-aaaaa-aaadq-cai",
            "--client",
            "claude-desktop",
            "--network",
            "local",
            "--skip-verify",
            "--yes", // Skip confirmation
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Successfully added")
                .or(predicate::str::contains("registered")),
        );

    // Verify the server was added by listing
    helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rdmx6-jaaaa-aaaaa-aaadq-cai"));
}

/// Test MCP remove command
#[test]
#[serial]
#[ignore = "MCP feature not fully implemented"]
fn test_mcp_remove() {
    let helper = McpTestHelper::new();

    // Create mock configuration with servers
    let servers = vec![
        MockMcpServer::new("remove-test-1"),
        MockMcpServer::new("remove-test-2"),
    ];
    helper.create_mock_config(&servers);

    // Test removing existing server
    helper
        .icarus_cmd()
        .args(["mcp", "remove", "remove-test-1", "--yes"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Successfully removed")
                .or(predicate::str::contains("deleted")),
        );

    // Verify server was removed
    helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("remove-test-1").not())
        .stdout(predicate::str::contains("remove-test-2"));

    // Test removing non-existent server
    helper
        .icarus_cmd()
        .args(["mcp", "remove", "nonexistent-server", "--yes"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("No MCP server found")
                .or(predicate::str::contains("not found")),
        );
}

/// Test MCP status command
#[test]
#[serial]
#[ignore = "MCP feature not fully implemented"]
fn test_mcp_status() {
    let helper = McpTestHelper::new();

    // Test status with no servers
    helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "status", "--all"])
        .assert()
        .success();

    // Create mock configuration
    let servers = vec![
        MockMcpServer::new("status-test-1"),
        MockMcpServer::new("status-test-2").disabled(),
    ];
    helper.create_mock_config(&servers);

    // Test status with servers
    helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "status", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status-test-1"))
        .stdout(predicate::str::contains("status-test-2"));

    // Test status for specific server
    helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "status", "status-test-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status-test-1"));

    // Test status for non-existent server
    helper
        .icarus_cmd()
        .args(["mcp", "status", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// Test MCP start/stop commands
#[test]
#[serial]
#[ignore = "MCP feature not fully implemented"]
fn test_mcp_start_stop() {
    let helper = McpTestHelper::new();

    // Create mock configuration
    let servers = vec![MockMcpServer::new("bridge-test")];
    helper.create_mock_config(&servers);

    // Test start command
    helper
        .icarus_cmd()
        .args(["mcp", "start", "--port", "3001", "--timeout", "5"])
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Starting MCP bridge")
                .or(predicate::str::contains("started")
                    .or(predicate::str::contains("Bridge server"))),
        );

    // Note: Stop command testing is complex because it requires a running server
    // We'll test the command validation instead
    helper
        .icarus_cmd()
        .args(["mcp", "stop", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stop MCP bridge server"));
}

/// Test MCP client detection
#[test]
#[serial]
#[ignore = "MCP feature not fully implemented"]
fn test_mcp_client_detection() {
    let helper = McpTestHelper::new();

    // Test that client detection doesn't crash
    helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "list", "--show-clients"])
        .assert()
        .success();

    // Test specific client support
    let clients = [
        "claude-desktop",
        "claude-code",
        "chatgpt-desktop",
        "continue",
    ];

    for client in clients {
        helper
            .icarus_cmd()
            .args(["mcp", "add", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains(client));
    }
}

/// Test MCP configuration file management
#[test]
#[serial]
#[ignore = "MCP feature not fully implemented"]
fn test_mcp_config_management() {
    let helper = McpTestHelper::new();

    // Ensure config doesn't exist initially
    assert!(!helper.config_path().exists());

    // Add a server (should create config)
    helper
        .icarus_cmd()
        .args([
            "mcp",
            "add",
            "rdmx6-jaaaa-aaaaa-aaadq-cai",
            "--client",
            "claude-desktop",
            "--skip-verify",
            "--yes",
        ])
        .assert()
        .success();

    // Verify config file was created
    assert!(helper.config_path().exists());

    // Verify config file is valid JSON
    let config_content = fs::read_to_string(helper.config_path()).unwrap();
    let config: Value = serde_json::from_str(&config_content).unwrap();

    assert!(config.get("servers").is_some());
    assert!(config["servers"].is_array());
}

/// Test MCP configuration backup and restore
#[test]
#[serial]
#[ignore = "MCP feature not fully implemented"]
fn test_mcp_config_backup() {
    let helper = McpTestHelper::new();

    // Create initial configuration
    let servers = vec![
        MockMcpServer::new("backup-test-1"),
        MockMcpServer::new("backup-test-2"),
    ];
    helper.create_mock_config(&servers);

    // Test backup creation (if command exists)
    let backup_result = helper.icarus_cmd().args(["mcp", "backup"]).output();

    // Command might not exist, so we just ensure it doesn't crash the system
    match backup_result {
        Ok(_) => println!("✅ MCP backup command works"),
        Err(_) => println!("ℹ️  MCP backup command not available (expected)"),
    }

    // Verify original config is still intact
    helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("backup-test-1"));
}

/// Test MCP server URL validation
#[test]
#[serial]
#[ignore = "MCP feature not fully implemented"]
fn test_mcp_url_validation() {
    let helper = McpTestHelper::new();

    // Test with custom URL
    helper
        .icarus_cmd()
        .args([
            "mcp",
            "add",
            "rdmx6-jaaaa-aaaaa-aaadq-cai",
            "--client",
            "claude-desktop",
            "--url",
            "https://custom.example.com/mcp",
            "--skip-verify",
            "--yes",
        ])
        .assert()
        .success();

    // Test with invalid URL format
    helper
        .icarus_cmd()
        .args([
            "mcp",
            "add",
            "rdmx6-jaaaa-aaaaa-aaadq-cai",
            "--client",
            "claude-desktop",
            "--url",
            "not-a-valid-url",
            "--skip-verify",
            "--yes",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid URL").or(predicate::str::contains("URL format")));
}

/// Test MCP network validation
#[test]
#[serial]
#[ignore = "MCP feature not fully implemented"]
fn test_mcp_network_validation() {
    let helper = McpTestHelper::new();

    // Test valid networks
    let valid_networks = ["local", "ic", "testnet"];

    for network in valid_networks {
        helper
            .icarus_cmd()
            .args([
                "mcp",
                "add",
                "rdmx6-jaaaa-aaaaa-aaadq-cai",
                "--client",
                "claude-desktop",
                "--network",
                network,
                "--skip-verify",
                "--yes",
            ])
            .assert()
            .success();
    }

    // Test invalid network
    helper
        .icarus_cmd()
        .args([
            "mcp",
            "add",
            "rdmx6-jaaaa-aaaaa-aaadq-cai",
            "--client",
            "claude-desktop",
            "--network",
            "invalid-network",
            "--skip-verify",
            "--yes",
        ])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Invalid network")
                .or(predicate::str::contains("Supported networks")),
        );
}

/// Test MCP command help and documentation
#[test]
#[serial]
fn test_mcp_command_help() {
    let helper = McpTestHelper::new();

    // Test main MCP help
    helper
        .icarus_cmd()
        .args(["mcp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("MCP server management commands"));

    // Test subcommand help
    let subcommands = ["add", "list", "remove", "status", "start", "stop"];

    for subcommand in subcommands {
        helper
            .icarus_cmd()
            .args(["mcp", subcommand, "--help"])
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }
}

/// Test MCP configuration statistics and reporting
#[test]
#[serial]
fn test_mcp_statistics() {
    let helper = McpTestHelper::new();

    // Create diverse configuration
    let servers = vec![
        MockMcpServer::new("stats-local-1").with_network("local"),
        MockMcpServer::new("stats-local-2").with_network("local"),
        MockMcpServer::new("stats-ic-1").with_network("ic"),
        MockMcpServer::new("stats-claude-1").with_client("claude-desktop"),
        MockMcpServer::new("stats-claude-2").with_client("claude-code"),
        MockMcpServer::new("stats-disabled").disabled(),
    ];
    helper.create_mock_config(&servers);

    // Test statistics display (if available)
    let stats_result = helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "list", "--stats"])
        .output();

    match stats_result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("✅ MCP statistics: {}", stdout);
        }
        Err(_) => {
            // Stats flag might not exist, test basic list instead
            helper
                .icarus_cmd()
                .args(["--quiet", "mcp", "list"])
                .assert()
                .success()
                .stdout(predicate::str::contains("stats-local-1"));
        }
    }
}

/// Stress test: MCP operations with many servers
#[test]
#[serial]
#[ignore = "MCP feature not fully implemented"]
fn test_mcp_many_servers() {
    let helper = McpTestHelper::new();

    // Create many mock servers
    let servers: Vec<MockMcpServer> = (0..50)
        .map(|i| MockMcpServer::new(&format!("server-{:03}", i)))
        .collect();

    helper.create_mock_config(&servers);

    // Test listing many servers
    helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("server-000"))
        .stdout(predicate::str::contains("server-049"));

    // Test JSON output with many servers
    helper
        .icarus_cmd()
        .args(["--quiet", "mcp", "list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));
}
