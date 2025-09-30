use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

/// Test the main CLI binary
#[test]
fn test_icarus_help() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("command-line interface"));
}

/// Test version flag
#[test]
fn test_icarus_version() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("icarus"));
}

/// Test new command help
#[test]
fn test_new_command_help() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.args(["new", "--help"]);
    cmd.assert().success().stdout(predicate::str::contains(
        "Create a new MCP canister project",
    ));
}

/// Test build command help
#[test]
fn test_build_command_help() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.args(["build", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Build the current project"));
}

/// Test deploy command help
#[test]
fn test_deploy_command_help() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.args(["deploy", "--help"]);
    cmd.assert().success().stdout(predicate::str::contains(
        "Deploy the canister to Internet Computer",
    ));
}

/// Test mcp command help
#[test]
fn test_mcp_command_help() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.args(["mcp", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("MCP server management commands"));
}

/// Test mcp add command help
#[test]
fn test_mcp_add_help() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.args(["mcp", "add", "--help"]);
    cmd.assert().success().stdout(predicate::str::contains(
        "Register MCP server with AI clients",
    ));
}

/// Test mcp list command help
#[test]
fn test_mcp_list_help() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.args(["mcp", "list", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List registered MCP servers"));
}

/// Test MCP list command with no servers
#[test]
#[serial]
fn test_mcp_list_empty() {
    // Use quiet flag to minimize output
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.args(["--quiet", "mcp", "list"]);

    // This should succeed even with no servers registered
    cmd.assert().success();
}

/// Test MCP list command with JSON output
#[test]
#[serial]
fn test_mcp_list_json() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.args(["--quiet", "mcp", "list", "--format", "json"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::starts_with("[").or(predicate::str::starts_with("[]")));
}

/// Test build command in non-project directory
#[test]
fn test_build_no_project() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.current_dir(temp_dir.path());
    cmd.args(["build"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Not in an Icarus project"));
}

/// Test deploy command in non-project directory
#[test]
fn test_deploy_no_project() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.current_dir(temp_dir.path());
    cmd.args(["deploy"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Not in an Icarus project"));
}

/// Test global flags
#[test]
fn test_verbose_flag() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.args(["--verbose", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_quiet_flag() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.args(["--quiet", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_force_flag() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.args(["--force", "--help"]);
    cmd.assert().success();
}

/// Test invalid command
#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.arg("invalid-command");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

/// Test MCP add command with invalid canister ID
#[test]
fn test_mcp_add_invalid_canister() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.args([
        "mcp",
        "add",
        "invalid-id",
        "--client",
        "claude-desktop",
        "--skip-verify",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid canister ID format"));
}

/// Test MCP status command with no servers
#[test]
#[serial]
fn test_mcp_status_no_servers() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.args(["--quiet", "mcp", "status", "--all"]);

    // Should succeed but indicate no servers
    cmd.assert().success();
}

/// Test MCP remove command with non-existent server
#[test]
#[serial]
fn test_mcp_remove_nonexistent() {
    let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
    cmd.args(["mcp", "remove", "nonexistent-server", "--yes"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No MCP server found"));
}

/// Test that all commands accept global flags
#[test]
fn test_global_flags_with_commands() {
    let commands = vec!["new", "build", "deploy", "mcp"];

    for command in commands {
        // Test verbose flag
        let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
        cmd.args(["--verbose", command, "--help"]);
        cmd.assert().success();

        // Test quiet flag
        let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
        cmd.args(["--quiet", command, "--help"]);
        cmd.assert().success();

        // Test force flag
        let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
        cmd.args(["--force", command, "--help"]);
        cmd.assert().success();
    }
}

/// Test output formats for MCP list command
#[test]
#[serial]
fn test_mcp_list_formats() {
    let formats = vec!["table", "json", "yaml", "plain"];

    for format in formats {
        let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
        cmd.args(["--quiet", "mcp", "list", "--format", format]);
        cmd.assert().success();
    }
}

/// Test MCP clients enum
#[test]
fn test_mcp_add_clients() {
    let clients = vec![
        "claude-desktop",
        "claude-code",
        "chatgpt-desktop",
        "continue",
    ];

    for client in clients {
        let mut cmd = Command::cargo_bin("icarus-cli").unwrap();
        cmd.args(["mcp", "add", "--help"]);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(client));
    }
}
