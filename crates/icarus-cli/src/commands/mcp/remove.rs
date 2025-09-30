use anyhow::{anyhow, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use tracing::info;

use crate::config::mcp::McpConfig;
use crate::{commands::mcp::RemoveArgs, Cli};

pub(crate) async fn execute(args: RemoveArgs, cli: &Cli) -> Result<()> {
    info!("Removing MCP server registration: {}", args.identifier);

    // Load existing configuration
    let mut mcp_config = McpConfig::load().await.unwrap_or_default();

    // Find server by identifier (name or canister ID)
    let server = mcp_config
        .servers
        .iter()
        .find(|s| s.name == args.identifier || s.canister_id == args.identifier)
        .cloned();

    let Some(server) = server else {
        return Err(anyhow!(
            "No MCP server found with identifier: {}",
            args.identifier
        ));
    };

    if !cli.quiet {
        println!("{} Removing MCP server", "→".bright_blue());
        println!(
            "  {} {}",
            "Name:".bright_white(),
            server.name.to_string().bright_cyan()
        );
        println!(
            "  {} {}",
            "Canister ID:".bright_white(),
            server.canister_id.to_string().bright_cyan()
        );
        println!(
            "  {} {}",
            "Client:".bright_white(),
            server.client.bright_cyan()
        );
    }

    // Confirm removal unless --yes flag is used
    if !args.yes && !cli.force {
        let theme = ColorfulTheme::default();
        let confirmed = Confirm::with_theme(&theme)
            .with_prompt(&format!(
                "Remove MCP server '{}'? This will unregister it from all AI clients.",
                server.name
            ))
            .default(false)
            .interact()?;

        if !confirmed {
            return Err(anyhow!("Removal cancelled"));
        }
    }

    // Remove from client configuration if client is specified or remove from all
    if let Some(ref client) = args.client {
        remove_from_client(&server, client).await?;
    } else {
        remove_from_all_clients(&server).await?;
    }

    // Remove from our configuration
    mcp_config.remove_server(server.name.as_str())?;
    mcp_config.save().await?;

    if !cli.quiet {
        println!(
            "\n{}",
            "✅ MCP Server Removed Successfully!".bright_green().bold()
        );
        println!(
            "  {} {}",
            "Server:".bright_white(),
            server.name.to_string().bright_cyan()
        );
        println!(
            "\n{}",
            "Note: Restart your AI clients to apply changes.".bright_yellow()
        );
    }

    info!("MCP server removed successfully");
    Ok(())
}

async fn remove_from_client(
    server: &crate::config::mcp::McpServerConfig,
    client: &crate::commands::mcp::McpClient,
) -> Result<()> {
    match client {
        crate::commands::mcp::McpClient::ClaudeDesktop => remove_from_claude_desktop(server).await,
        crate::commands::mcp::McpClient::ClaudeCode => remove_from_claude_code(server).await,
        crate::commands::mcp::McpClient::ChatgptDesktop => {
            remove_from_chatgpt_desktop(server).await
        }
        crate::commands::mcp::McpClient::Continue => remove_from_continue(server).await,
        crate::commands::mcp::McpClient::Custom => {
            // Custom clients require manual configuration
            Ok(())
        }
    }
}

async fn remove_from_all_clients(server: &crate::config::mcp::McpServerConfig) -> Result<()> {
    // Try to remove from all known clients (ignore errors for clients that aren't installed)
    let _ = remove_from_claude_desktop(server).await;
    let _ = remove_from_claude_code(server).await;
    let _ = remove_from_chatgpt_desktop(server).await;
    let _ = remove_from_continue(server).await;
    Ok(())
}

async fn remove_from_claude_desktop(server: &crate::config::mcp::McpServerConfig) -> Result<()> {
    use crate::utils::client_detector;
    use tokio::fs;

    let config_path = client_detector::get_claude_desktop_config_path()?;
    if !config_path.exists() {
        return Ok(()); // No config file, nothing to remove
    }

    let config_content = fs::read_to_string(&config_path).await?;
    let mut config: serde_json::Value = serde_json::from_str(&config_content)?;

    // Remove server from mcpServers
    if let Some(mcp_servers) = config.get_mut("mcpServers") {
        if let Some(obj) = mcp_servers.as_object_mut() {
            obj.remove(server.name.as_str());
        }
    }

    // Write updated configuration
    let updated_config = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, updated_config).await?;

    Ok(())
}

async fn remove_from_claude_code(server: &crate::config::mcp::McpServerConfig) -> Result<()> {
    // Similar to Claude Desktop
    remove_from_claude_desktop(server).await
}

async fn remove_from_chatgpt_desktop(server: &crate::config::mcp::McpServerConfig) -> Result<()> {
    use crate::utils::client_detector;
    use tokio::fs;

    let config_path = client_detector::get_chatgpt_desktop_config_path()?;
    if !config_path.exists() {
        return Ok(());
    }

    let config_content = fs::read_to_string(&config_path).await?;
    let mut config: serde_json::Value = serde_json::from_str(&config_content)?;

    // Remove from mcp object
    if let Some(mcp) = config.get_mut("mcp") {
        if let Some(obj) = mcp.as_object_mut() {
            obj.remove(server.name.as_str());
        }
    }

    let updated_config = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, updated_config).await?;

    Ok(())
}

async fn remove_from_continue(server: &crate::config::mcp::McpServerConfig) -> Result<()> {
    use crate::utils::client_detector;
    use tokio::fs;

    let config_path = client_detector::get_continue_config_path()?;
    if !config_path.exists() {
        return Ok(());
    }

    let config_content = fs::read_to_string(&config_path).await?;
    let mut config: serde_json::Value = serde_json::from_str(&config_content)?;

    // Remove from mcp array
    if let Some(mcp_array) = config.get_mut("mcp") {
        if let Some(array) = mcp_array.as_array_mut() {
            array.retain(|item| {
                item.get("name")
                    .and_then(|n| n.as_str())
                    .map(|name| server.name != name)
                    .unwrap_or(true)
            });
        }
    }

    let updated_config = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, updated_config).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::mcp::McpServerConfig;
    use chrono::Utc;

    #[tokio::test]
    async fn test_remove_nonexistent_server() {
        let args = RemoveArgs {
            identifier: "nonexistent-server".to_string(),
            client: None,
            yes: true,
        };

        let cli = crate::Cli {
            verbose: false,
            quiet: true,
            force: false,
            command: crate::Commands::Mcp(crate::commands::McpArgs::Remove(args.clone())),
        };

        let result = execute(args, &cli).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No MCP server found"));
    }

    #[test]
    fn test_server_identification() {
        use crate::types::{CanisterId, Network, ServerName};

        let server = McpServerConfig {
            name: ServerName::new("test-server").unwrap(),
            canister_id: CanisterId::new("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap(),
            network: Network::Local,
            url: "http://localhost:3000/mcp".to_string(),
            client: "claude-desktop".to_string(),
            port: Some(3000),
            enabled: true,
            created_at: Utc::now(),
            last_updated: Utc::now(),
        };

        // Should be identifiable by name
        assert_eq!(server.name, "test-server");

        // Should be identifiable by canister ID
        assert_eq!(server.canister_id, "rdmx6-jaaaa-aaaaa-aaadq-cai");
    }
}
