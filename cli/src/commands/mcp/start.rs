use anyhow::{Context, Result};
use colored::Colorize;
use std::str::FromStr;
use tokio::io::{stdin, stdout};

use crate::utils::{create_spinner, print_info};

/// Start the MCP server
pub async fn execute(canister_id: String, daemon: bool) -> Result<()> {
    // Validate canister ID
    let canister_principal =
        candid::Principal::from_str(&canister_id).context("Invalid canister ID format")?;

    // Check if we're being run by Claude Desktop (MCP mode)
    // Claude Desktop runs MCP servers as subprocesses and communicates via stdio
    let is_mcp_mode =
        !is_terminal::is_terminal(std::io::stdin()) && !is_terminal::is_terminal(std::io::stdout());

    if is_mcp_mode {
        // In MCP mode, disable all logging and run the built-in MCP server directly
        colored::control::set_override(false);
        std::env::set_var("RUST_LOG", "error");

        print_info(&format!("Starting MCP server for canister {}", canister_id));

        // Use the new icarus-mcp crate directly
        return run_mcp_server_stdio(canister_principal).await;
    }

    print_info(&format!(
        "Starting Icarus MCP server for canister {}",
        canister_id
    ));

    if daemon {
        start_daemon_mode(canister_principal).await
    } else {
        start_foreground_mode(canister_principal).await
    }
}

/// Run the MCP server directly using stdio (for MCP client connection)
async fn run_mcp_server_stdio(canister_id: candid::Principal) -> Result<()> {
    use icarus_mcp::{McpConfig, McpServer};

    // Create configuration
    let config = McpConfig::local(canister_id);

    // Create and start server using new API
    let server = McpServer::from_config(config)
        .await
        .context("Failed to create MCP server")?;

    // Serve over stdio for MCP client
    let serving_server = server
        .serve(stdin(), stdout())
        .await
        .context("Failed to start MCP server")?;

    // Run the server
    serving_server.run().await.context("MCP server failed")
}

/// Start MCP server in foreground mode (for development)
async fn start_foreground_mode(canister_id: candid::Principal) -> Result<()> {
    println!("\n{}", "MCP server running in foreground mode".cyan());
    println!("Connect your MCP client to this process via stdio");
    println!("Press Ctrl+C to stop\n");

    // Run the server directly
    run_mcp_server_stdio(canister_id).await
}

/// Start MCP server in daemon mode (background)
async fn start_daemon_mode(canister_id: candid::Principal) -> Result<()> {
    let spinner = create_spinner("Starting MCP server in background");

    // For now, just inform the user that daemon mode isn't fully implemented
    // In the future, this could spawn icarus-mcp binary as a separate process
    spinner.finish_and_clear();

    print_info("Daemon mode not yet implemented, falling back to foreground mode");
    println!("The MCP server will run in the current terminal");
    println!(
        "For background operation, use: icarus-mcp --canister-id {} &",
        canister_id
    );

    start_foreground_mode(canister_id).await
}

