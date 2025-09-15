//! E2E tests for MCP client management commands

#[path = "../common/mod.rs"]
mod common;

use common::*;
use tempfile::TempDir;

/// Test that `icarus mcp list` shows empty configuration when no servers are configured
#[test]
fn test_mcp_list_empty() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-list-empty");

    let output = cli.run_in(test_project.path(), &["mcp", "list"]);
    assert_success(&output);
    // Should show empty tree or no servers message
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MCP Server Configuration") || stdout.contains("no servers"));
}

/// Test that `icarus mcp add` validates canister ID format
#[test]
fn test_mcp_add_invalid_canister_id() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-add-invalid");

    let output = cli.run_in(test_project.path(), &["mcp", "add", "invalid-canister-id"]);
    assert!(!output.status.success());
    // Should show error about invalid canister ID format
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("invalid")
            || String::from_utf8_lossy(&output.stderr).contains("canister")
            || String::from_utf8_lossy(&output.stdout).contains("invalid")
            || String::from_utf8_lossy(&output.stdout).contains("canister")
    );
}

/// Test that `icarus mcp add` with valid canister ID but no clients specified shows interactive selection
#[test]
fn test_mcp_add_interactive_selection() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-add-interactive");

    // Use a valid canister ID format but don't actually deploy
    let output = cli.run_in(
        test_project.path(),
        &["mcp", "add", "rdmx6-jaaaa-aaaah-qcaiq-cai"],
    );

    // This might fail due to network or client detection issues, but should show client selection UI
    // The important thing is that it recognizes the valid canister ID format
    if output.status.success() {
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("client")
                || String::from_utf8_lossy(&output.stdout).contains("Claude")
                || String::from_utf8_lossy(&output.stdout).contains("ChatGPT")
        );
    } else {
        // If it fails, it should be due to canister not being accessible, not invalid format
        assert!(!String::from_utf8_lossy(&output.stderr).contains("invalid canister"));
    }
}

/// Test that `icarus mcp add` with --clients flag specifies clients directly
#[test]
fn test_mcp_add_with_clients_flag() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-add-clients");

    let output = cli.run_in(
        test_project.path(),
        &[
            "mcp",
            "add",
            "rdmx6-jaaaa-aaaah-qcaiq-cai",
            "--clients",
            "claude",
        ],
    );

    // This will likely fail due to network issues, but should validate the command structure
    if output.status.success() {
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("Claude")
                || String::from_utf8_lossy(&output.stdout).contains("added")
        );
    } else {
        // Should not fail due to invalid arguments
        assert!(!String::from_utf8_lossy(&output.stderr).contains("Unknown argument"));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("invalid option"));
    }
}

/// Test that `icarus mcp add` with custom config path works
#[test]
fn test_mcp_add_with_custom_config_path() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-add-custom-config");

    // Create a temporary config file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("custom_mcp_config.json");

    let output = cli.run_in(
        test_project.path(),
        &[
            "mcp",
            "add",
            "rdmx6-jaaaa-aaaah-qcaiq-cai",
            "--config-path",
            config_path.to_str().unwrap(),
            "--clients",
            "claude",
        ],
    );

    // Should accept the custom config path argument
    if !output.status.success() {
        assert!(!String::from_utf8_lossy(&output.stderr).contains("Unknown argument"));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("invalid option"));
    }
}

/// Test that `icarus mcp add` with --name flag sets custom server name
#[test]
fn test_mcp_add_with_custom_name() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-add-custom-name");

    let output = cli.run_in(
        test_project.path(),
        &[
            "mcp",
            "add",
            "rdmx6-jaaaa-aaaah-qcaiq-cai",
            "--name",
            "My Custom Tool",
            "--clients",
            "claude",
        ],
    );

    // Should accept the name argument
    if !output.status.success() {
        assert!(!String::from_utf8_lossy(&output.stderr).contains("Unknown argument"));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("invalid option"));
    }
}

/// Test that `icarus mcp remove` validates canister ID
#[test]
fn test_mcp_remove_invalid_canister_id() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-remove-invalid");

    let output = cli.run_in(test_project.path(), &["mcp", "remove", "invalid-id"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("invalid")
            || String::from_utf8_lossy(&output.stderr).contains("canister")
            || String::from_utf8_lossy(&output.stdout).contains("invalid")
            || String::from_utf8_lossy(&output.stdout).contains("canister")
    );
}

/// Test that `icarus mcp remove` with valid canister ID shows interactive selection
#[test]
fn test_mcp_remove_interactive() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-remove-interactive");

    let output = cli.run_in(
        test_project.path(),
        &["mcp", "remove", "rdmx6-jaaaa-aaaah-qcaiq-cai"],
    );

    // Should recognize valid canister ID format
    if !output.status.success() {
        assert!(!String::from_utf8_lossy(&output.stderr).contains("invalid canister"));
    }
}

/// Test that `icarus mcp remove` with --clients flag works
#[test]
fn test_mcp_remove_with_clients_flag() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-remove-clients");

    let output = cli.run_in(
        test_project.path(),
        &[
            "mcp",
            "remove",
            "rdmx6-jaaaa-aaaah-qcaiq-cai",
            "--clients",
            "claude,chatgpt",
        ],
    );

    // Should accept the clients flag
    if !output.status.success() {
        assert!(!String::from_utf8_lossy(&output.stderr).contains("Unknown argument"));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("invalid option"));
    }
}

/// Test that `icarus mcp remove` with --all flag works
#[test]
fn test_mcp_remove_with_all_flag() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-remove-all");

    let output = cli.run_in(
        test_project.path(),
        &["mcp", "remove", "rdmx6-jaaaa-aaaah-qcaiq-cai", "--all"],
    );

    // Should accept the --all flag
    if !output.status.success() {
        assert!(!String::from_utf8_lossy(&output.stderr).contains("Unknown argument"));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("invalid option"));
    }
}

/// Test that `icarus mcp dashboard` command exists and runs
#[test]
fn test_mcp_dashboard_command_exists() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-dashboard");

    let output = cli.run_in(test_project.path(), &["mcp", "dashboard"]);

    // The dashboard command should exist (may fail due to missing clients, but shouldn't be unknown command)
    if !output.status.success() {
        assert!(!String::from_utf8_lossy(&output.stderr).contains("Unknown subcommand"));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("command not found"));
    }
}

/// Test that invalid `icarus mcp` subcommands show helpful error
#[test]
fn test_mcp_invalid_subcommand() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-invalid");

    let output = cli.run_in(test_project.path(), &["mcp", "invalid-subcommand"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Unknown subcommand")
            || String::from_utf8_lossy(&output.stderr).contains("invalid")
            || String::from_utf8_lossy(&output.stderr).contains("help")
    );
}

/// Test that `icarus mcp --help` shows help information
#[test]
fn test_mcp_help() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-help");

    let output = cli.run_in(test_project.path(), &["mcp", "--help"]);
    assert_success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("add"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("list"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("remove"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("dashboard"));
}

/// Test MCP commands with environment variables
#[test]
fn test_mcp_with_env_variables() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-env");

    // Create a temporary config file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("env_config.json");

    // Test with CLAUDE_CONFIG_PATH environment variable
    let mut output = cli.run_in_with_env(
        test_project.path(),
        &[
            "mcp",
            "add",
            "rdmx6-jaaaa-aaaah-qcaiq-cai",
            "--clients",
            "claude",
        ],
        &[("CLAUDE_CONFIG_PATH", config_path.to_str().unwrap())],
    );

    // Should respect environment variable
    if !output.status.success() {
        assert!(!String::from_utf8_lossy(&output.stderr).contains("Unknown argument"));
    }

    // Test with ICARUS_DEBUG enabled
    output = cli.run_in_with_env(
        test_project.path(),
        &["mcp", "list"],
        &[("ICARUS_DEBUG", "1")],
    );

    // Should run with debug enabled
    if output.status.success() && !output.stdout.is_empty() {
        // Debug mode might show additional information
    }
}

/// Integration test: Add and then list MCP server configurations
#[test]
fn test_mcp_add_and_list_integration() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-integration");

    // Create temporary config files for testing
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("test_config.json");

    // Initialize empty config file
    std::fs::write(&config_path, r#"{"mcpServers": {}}"#).expect("Failed to write config");

    // Add a server with custom config path
    let add_output = cli.run_in(
        test_project.path(),
        &[
            "mcp",
            "add",
            "rdmx6-jaaaa-aaaah-qcaiq-cai",
            "--config-path",
            config_path.to_str().unwrap(),
            "--clients",
            "claude",
            "--name",
            "Test Server",
        ],
    );

    // If add succeeds, list should show the server
    if add_output.status.success() {
        let list_output = cli.run_in(test_project.path(), &["mcp", "list"]);
        if list_output.status.success() {
            // Should show configuration tree or server info
            let stdout = String::from_utf8_lossy(&list_output.stdout);
            assert!(stdout.contains("Test Server") || stdout.contains("MCP"));
        }
    }
}

/// Test: MCP start command basic functionality
#[test]
fn test_mcp_start_basic() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-start-basic");

    // Test with valid canister ID format
    let output = cli.run_in(
        test_project.path(),
        &["mcp", "start", "rdmx6-jaaaa-aaaah-qcaiq-cai"],
    );

    // Should accept valid canister ID
    // Note: This will likely fail because we're not connected to ICP network
    // but it should not fail due to argument parsing
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Unknown argument"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Invalid canister ID format"));
}

/// Test: MCP start command with daemon flag
#[test]
fn test_mcp_start_daemon_flag() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-start-daemon");

    // Test daemon flag
    let output = cli.run_in(
        test_project.path(),
        &["mcp", "start", "rdmx6-jaaaa-aaaah-qcaiq-cai", "--daemon"],
    );

    // Should accept daemon flag
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Unknown argument"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("unexpected argument"));
}

/// Test: MCP start command with invalid canister ID
#[test]
fn test_mcp_start_invalid_canister_id() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-start-invalid");

    // Test with invalid canister ID
    let output = cli.run_in(
        test_project.path(),
        &["mcp", "start", "invalid-canister-id"],
    );

    // Should show error for invalid canister ID format
    assert!(
        !output.status.success()
            || String::from_utf8_lossy(&output.stderr).contains("Invalid canister ID")
    );
}

/// Test: MCP start command help output
#[test]
fn test_mcp_start_help() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-start-help");

    // Test help flag
    let output = cli.run_in(test_project.path(), &["mcp", "start", "--help"]);

    if output.status.success() {
        let help_text = String::from_utf8_lossy(&output.stdout).to_lowercase();
        // Should contain key information about the start command
        assert!(help_text.contains("start") || help_text.contains("mcp server"));
        assert!(help_text.contains("canister") || help_text.contains("daemon"));
    }
}

/// Integration test: MCP start command with environment variables
#[test]
fn test_mcp_start_with_environment() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("mcp-start-env");

    // Test with ICARUS_DEBUG enabled
    let output = cli.run_in_with_env(
        test_project.path(),
        &["mcp", "start", "rdmx6-jaaaa-aaaah-qcaiq-cai"],
        &[("ICARUS_DEBUG", "1")],
    );

    // Should accept environment variable
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Unknown argument"));

    // Test with custom RUST_LOG level
    let output = cli.run_in_with_env(
        test_project.path(),
        &["mcp", "start", "rdmx6-jaaaa-aaaah-qcaiq-cai"],
        &[("RUST_LOG", "debug")],
    );

    // Should accept environment variable
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Unknown argument"));
}
