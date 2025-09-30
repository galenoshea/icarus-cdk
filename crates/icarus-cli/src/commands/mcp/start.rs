use anyhow::{anyhow, Result};
use colored::Colorize;
use tokio::process::Command;
use tracing::{info, warn};

use crate::config::mcp::McpConfig;
use crate::utils::bridge::{McpBridgeServer, SimpleBridgeServer};
use crate::{commands::mcp::StartArgs, Cli};

pub(crate) async fn execute(args: StartArgs, cli: &Cli) -> Result<()> {
    info!("Starting MCP bridge server on {}:{}", args.host, args.port);

    if !cli.quiet {
        println!("{} Starting MCP bridge server", "→".bright_blue());
        println!(
            "  {} {}:{}",
            "Address:".bright_white(),
            args.host.bright_cyan(),
            args.port.to_string().bright_cyan()
        );
    }

    // Load MCP configuration
    let mcp_config = McpConfig::load().await.unwrap_or_default();

    if mcp_config.servers.is_empty() {
        warn!("No MCP servers registered. Use 'icarus mcp add' to register servers.");
        if !cli.quiet {
            println!("{}", "⚠️  No MCP servers registered.".yellow());
            println!("Use 'icarus mcp add <canister-id> --client <client>' to register servers.");
        }
    }

    // Check if port is already in use
    if is_port_in_use(&args.host, args.port).await {
        return Err(anyhow!(
            "Port {} is already in use. Use a different port or stop the existing service.",
            args.port
        ));
    }

    // Start the bridge server
    if args.daemon {
        start_daemon_server(&args, &mcp_config, cli).await
    } else {
        start_foreground_server(&args, &mcp_config, cli).await
    }
}

async fn is_port_in_use(host: &str, port: u16) -> bool {
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    let addr: SocketAddr = match format!("{}:{}", host, port).parse() {
        Ok(addr) => addr,
        Err(_) => return true, // Invalid address means "port is in use" (can't access)
    };
    TcpListener::bind(addr).await.is_err()
}

async fn start_foreground_server(
    args: &StartArgs,
    mcp_config: &McpConfig,
    cli: &Cli,
) -> Result<()> {
    if !cli.quiet {
        println!(
            "{} Starting MCP bridge in foreground mode",
            "→".bright_blue()
        );
        println!("{} Press Ctrl+C to stop", "→".bright_blue());
    }

    // Create a simple bridge server using icarus-core
    let bridge_server = create_bridge_server(args, mcp_config).await?;

    // Run the server
    run_bridge_server(bridge_server, args, cli).await
}

async fn start_daemon_server(args: &StartArgs, _mcp_config: &McpConfig, cli: &Cli) -> Result<()> {
    if !cli.quiet {
        println!("{} Starting MCP bridge in daemon mode", "→".bright_blue());
    }

    // For daemon mode, we'll spawn a background process
    let mut cmd = Command::new("icarus");
    cmd.args(&[
        "mcp",
        "start",
        "--host",
        &args.host,
        "--port",
        &args.port.to_string(),
    ]);

    if let Some(ref config_path) = args.config {
        cmd.args(&["--config", &config_path.to_string_lossy()]);
    }

    // Spawn the daemon process
    let child = cmd.spawn()?;
    let pid = child.id().expect("Failed to get process ID");

    // Save PID for later management
    save_daemon_pid(pid).await?;

    if !cli.quiet {
        println!(
            "{} MCP bridge started as daemon (PID: {})",
            "✅".green(),
            pid.to_string().bright_cyan()
        );
        println!(
            "  {} Use 'icarus mcp stop' to stop the server",
            "→".bright_blue()
        );
        println!("  {} Logs: /tmp/icarus-mcp-bridge.log", "→".bright_blue());
    }

    info!("MCP bridge daemon started with PID: {}", pid);
    Ok(())
}

async fn create_bridge_server(
    args: &StartArgs,
    mcp_config: &McpConfig,
) -> Result<Box<dyn McpBridgeServer>> {
    let bridge = SimpleBridgeServer::new(&args.host, args.port, mcp_config.clone())?;

    Ok(Box::new(bridge))
}

async fn run_bridge_server(
    mut server: Box<dyn McpBridgeServer>,
    args: &StartArgs,
    cli: &Cli,
) -> Result<()> {
    // Setup graceful shutdown handling
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    // Handle Ctrl+C
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        let _ = shutdown_tx.send(());
    });

    if !cli.quiet {
        println!("\n{}", "🚀 MCP Bridge Server Running".bright_green().bold());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!(
            "{} {}:{}",
            "Listening on:".bright_white(),
            args.host.bright_cyan(),
            args.port.to_string().bright_cyan()
        );
        println!(
            "{} Ready to accept MCP connections",
            "Status:".bright_white()
        );
        println!();
    }

    // Start the server
    let server_task = tokio::spawn(async move { server.run().await });

    // Wait for shutdown signal
    tokio::select! {
        result = server_task => {
            match result {
                Ok(Ok(())) => {
                    if !cli.quiet {
                        println!("{} Server stopped gracefully", "✅".green());
                    }
                }
                Ok(Err(e)) => {
                    return Err(anyhow!("Server error: {}", e));
                }
                Err(e) => {
                    return Err(anyhow!("Server task error: {}", e));
                }
            }
        }
        _ = shutdown_rx => {
            if !cli.quiet {
                println!("\n{} Shutting down server...", "→".bright_blue());
            }
            info!("MCP bridge server shutdown requested");
        }
    }

    Ok(())
}

async fn save_daemon_pid(pid: u32) -> Result<()> {
    use tokio::fs;

    let pid_file = "/tmp/icarus-mcp-bridge.pid";
    fs::write(pid_file, pid.to_string()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_port_availability_check() {
        // Test with a port that should be available
        let _available = is_port_in_use("127.0.0.1", 0).await; // Port 0 means OS assigns available port
                                                               // This test might be flaky depending on system state

        // Test with a clearly invalid host
        let invalid = is_port_in_use("999.999.999.999", 1234).await;
        assert!(invalid); // Should fail to bind to invalid host
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_daemon_pid_save() {
        let result = save_daemon_pid(12345).await;
        assert!(result.is_ok());

        // Check if PID file was created
        let pid_content = tokio::fs::read_to_string("/tmp/icarus-mcp-bridge.pid")
            .await
            .unwrap();
        assert_eq!(pid_content, "12345");

        // Clean up
        let _ = tokio::fs::remove_file("/tmp/icarus-mcp-bridge.pid").await;
    }

    #[test]
    fn test_start_args_validation() {
        let args = StartArgs {
            port: 3000,
            host: "localhost".to_string(),
            daemon: false,
            config: None,
        };

        assert_eq!(args.port, 3000);
        assert_eq!(args.host, "localhost");
        assert!(!args.daemon);
    }
}
