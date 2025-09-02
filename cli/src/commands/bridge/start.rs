use anyhow::{Context, Result};
use colored::Colorize;
use std::process::{Command, Stdio};
use tokio::time::{sleep, Duration};

// use crate::auth::ensure_authenticated;  // TODO: Re-enable when marketplace auth is ready
use crate::utils::{
    create_spinner, platform::get_bridge_binary_path, print_error, print_info, print_success,
};

pub async fn execute(canister_id: String, port: u16, daemon: bool) -> Result<()> {
    // Always authenticate - use local for now
    execute_with_auth(canister_id, port, daemon, true, true).await
}

pub async fn execute_with_auth(
    canister_id: String,
    port: u16,
    daemon: bool,
    _authenticate: bool,
    _use_local: bool,
) -> Result<()> {
    // Check if we're being run by Claude Desktop (MCP mode)
    // Claude Desktop runs MCP servers as subprocesses and communicates via stdio
    let is_mcp_mode =
        !is_terminal::is_terminal(std::io::stdin()) && !is_terminal::is_terminal(std::io::stdout());

    if is_mcp_mode {
        // In MCP mode, disable all logging and run the built-in MCP server directly
        // Disable colored output
        colored::control::set_override(false);
        // Set log level to error only (or disable completely)
        std::env::set_var("RUST_LOG", "error");

        // Always authenticate in MCP mode (using local for development)
        return crate::bridge::rmcp_server::run_with_auth(canister_id, true, true).await;
    }

    // Check if bridge is installed
    let bridge_path = get_bridge_binary_path()?;
    if !bridge_path.exists() {
        print_error("Icarus bridge not installed");
        println!("Run: icarus bridge install");
        anyhow::bail!("Bridge not installed");
    }

    // Validate canister ID
    if !is_valid_canister_id(&canister_id) {
        anyhow::bail!("Invalid canister ID format");
    }

    // Check if already running on this port
    if is_port_in_use(port).await {
        print_error(&format!("Port {} is already in use", port));
        println!("Choose a different port or stop the existing service");
        anyhow::bail!("Port already in use");
    }

    // TODO: Re-enable marketplace authentication when ready
    // For now, we're using Internet Identity authentication
    // let session = ensure_authenticated().await?;
    //
    // // Check if user has access to this canister
    // if !session.authorized_canisters.is_empty() && !session.can_access_canister(&canister_id) {
    //     print_warning("You don't have access to this canister");
    //     println!("Purchase access at: https://icarus.dev/marketplace");
    //     anyhow::bail!("Unauthorized canister access");
    // }

    print_info(&format!(
        "Starting Icarus bridge for canister {}",
        canister_id
    ));

    if daemon {
        start_daemon(bridge_path, canister_id, port).await
    } else {
        start_foreground(bridge_path, canister_id, port).await
    }
}

async fn start_foreground(
    bridge_path: std::path::PathBuf,
    canister_id: String,
    port: u16,
) -> Result<()> {
    println!("\n{}", "Bridge running in foreground mode".cyan());
    println!("Press Ctrl+C to stop\n");

    let mut child = Command::new(&bridge_path)
        .arg("--canister-id")
        .arg(&canister_id)
        .arg("--port")
        .arg(port.to_string())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start bridge")?;

    print_success(&format!("Bridge started on port {}", port));
    println!("\n{}", "Connection details:".bold());
    println!("  WebSocket URL: ws://localhost:{}", port);
    println!("  Canister ID: {}", canister_id);
    println!("\nThe bridge is now ready to accept connections from Claude Desktop");

    // Wait for the process to exit
    let status = child.wait()?;

    if !status.success() {
        print_error("Bridge exited with error");
        anyhow::bail!("Bridge process failed");
    }

    Ok(())
}

async fn start_daemon(
    bridge_path: std::path::PathBuf,
    canister_id: String,
    port: u16,
) -> Result<()> {
    let spinner = create_spinner("Starting bridge in background");

    // Start bridge process
    let mut child = Command::new(&bridge_path)
        .arg("--canister-id")
        .arg(&canister_id)
        .arg("--port")
        .arg(port.to_string())
        .arg("--daemon")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start bridge")?;

    // Give it a moment to start
    sleep(Duration::from_secs(2)).await;

    // Check if it's still running
    match child.try_wait()? {
        Some(status) => {
            spinner.finish_and_clear();
            if !status.success() {
                print_error("Bridge failed to start");
                anyhow::bail!("Bridge process exited immediately");
            }
        }
        None => {
            // Still running, good
            let pid = child.id();

            // Save PID for later management
            save_bridge_pid(port, pid)?;

            spinner.finish_and_clear();
            print_success(&format!("Bridge started in background (PID: {})", pid));

            println!("\n{}", "Connection details:".bold());
            println!("  WebSocket URL: ws://localhost:{}", port);
            println!("  Canister ID: {}", canister_id);
            println!("\n{}", "Management commands:".bold());
            println!("  Check status: icarus bridge status");
            println!("  Stop bridge: icarus bridge stop --port {}", port);
        }
    }

    Ok(())
}

fn is_valid_canister_id(id: &str) -> bool {
    id.len() > 10 && id.chars().all(|c| c.is_alphanumeric() || c == '-')
}

async fn is_port_in_use(port: u16) -> bool {
    tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .is_err()
}

fn save_bridge_pid(port: u16, pid: u32) -> Result<()> {
    let config_dir = crate::config::IcarusConfig::config_dir()?;
    let pid_file = config_dir.join(format!("bridge-{}.pid", port));

    std::fs::write(&pid_file, pid.to_string())?;
    Ok(())
}
