use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use serde_json::Value;
use std::path::PathBuf;
use tracing::{info, warn};

use crate::config::mcp::{McpConfig, McpServerConfig};
use crate::utils::client_detector;
use crate::{commands::mcp::AddArgs, Cli};

pub(crate) async fn execute(args: AddArgs, cli: &Cli) -> Result<()> {
    info!(
        "Adding MCP server registration for canister: {}",
        args.canister_id
    );

    if !cli.quiet {
        println!("{} Registering MCP server", "→".bright_blue());
        println!(
            "  {} {}",
            "Canister ID:".bright_white(),
            args.canister_id.bright_cyan()
        );
        println!(
            "  {} {}",
            "Client:".bright_white(),
            args.client.to_string().bright_cyan()
        );
        println!(
            "  {} {}",
            "Network:".bright_white(),
            args.network.bright_cyan()
        );
    }

    // Validate canister ID format
    validate_canister_id(&args.canister_id)?;

    // Detect or validate client configuration
    let client_config = detect_client_config(&args, cli).await?;

    // Verify canister accessibility if not skipped
    if !args.skip_verify {
        verify_canister_accessibility(&args).await?;
    }

    // Load existing MCP configuration
    let mut mcp_config = McpConfig::load().await.unwrap_or_default();

    // Create server configuration
    let server_config = create_server_config(&args, &client_config)?;

    // Check for existing registration
    if mcp_config.has_server(server_config.name.as_str()) {
        if !cli.force {
            let theme = ColorfulTheme::default();
            let overwrite = Confirm::with_theme(&theme)
                .with_prompt(&format!(
                    "Server '{}' already exists. Overwrite?",
                    server_config.name
                ))
                .default(false)
                .interact()?;

            if !overwrite {
                return Err(anyhow!("Registration cancelled"));
            }
        }
    }

    // Add server to configuration
    mcp_config.add_server(server_config.clone())?;

    // Register with AI client
    register_with_client(&server_config, &client_config, &args.client).await?;

    // Save updated configuration
    mcp_config.save().await?;

    if !cli.quiet {
        print_success_message(&server_config, &client_config);
    }

    info!("MCP server registered successfully");
    Ok(())
}

fn validate_canister_id(canister_id: &str) -> Result<()> {
    // Basic canister ID format validation
    if canister_id.is_empty() {
        return Err(anyhow!("Canister ID cannot be empty"));
    }

    // Check for IC canister ID format (basic validation)
    if !canister_id.contains('-') || canister_id.len() < 20 {
        return Err(anyhow!(
            "Invalid canister ID format. Expected format: xxxxx-xxxxx-xxxxx-xxxxx-xxx"
        ));
    }

    Ok(())
}

#[derive(Debug)]
struct ClientConfig {
    name: String,
    config_path: PathBuf,
    #[allow(dead_code)]
    install_path: Option<PathBuf>,
}

async fn detect_client_config(args: &AddArgs, cli: &Cli) -> Result<ClientConfig> {
    let client_name = args
        .client_name
        .as_ref()
        .unwrap_or(&args.client.to_string())
        .clone();

    match args.client {
        crate::commands::mcp::McpClient::ClaudeDesktop => {
            let config_path = client_detector::get_claude_desktop_config_path()?;
            Ok(ClientConfig {
                name: client_name,
                config_path,
                install_path: client_detector::get_claude_desktop_install_path(),
            })
        }
        crate::commands::mcp::McpClient::ClaudeCode => {
            let config_path = client_detector::get_claude_code_config_path()?;
            Ok(ClientConfig {
                name: client_name,
                config_path,
                install_path: None, // VS Code extension
            })
        }
        crate::commands::mcp::McpClient::ChatgptDesktop => {
            let config_path = client_detector::get_chatgpt_desktop_config_path()?;
            Ok(ClientConfig {
                name: client_name,
                config_path,
                install_path: client_detector::get_chatgpt_desktop_install_path(),
            })
        }
        crate::commands::mcp::McpClient::Continue => {
            let config_path = client_detector::get_continue_config_path()?;
            Ok(ClientConfig {
                name: client_name,
                config_path,
                install_path: None, // VS Code extension
            })
        }
        crate::commands::mcp::McpClient::Custom => {
            if !cli.quiet {
                warn!("Custom client selected. Manual configuration required.");
            }
            Ok(ClientConfig {
                name: client_name,
                config_path: PathBuf::from("."), // Placeholder
                install_path: None,
            })
        }
    }
}

async fn verify_canister_accessibility(args: &AddArgs) -> Result<()> {
    // Construct canister URL based on network
    let base_url = match args.network.as_str() {
        "local" => "http://127.0.0.1:4943",
        "ic" => "https://ic0.app",
        "testnet" => "https://testnet.dfinity.network",
        _ => return Err(anyhow!("Unsupported network: {}", args.network)),
    };

    let candid_url = format!("{}/?canisterId={}", base_url, args.canister_id);

    // Try to fetch Candid interface
    let client = reqwest::Client::new();
    let response = client
        .get(&candid_url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .with_context(|| format!("Failed to access canister at {}", candid_url))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Canister not accessible. HTTP status: {}",
            response.status()
        ));
    }

    Ok(())
}

fn create_server_config(args: &AddArgs, client_config: &ClientConfig) -> Result<McpServerConfig> {
    let server_name = args
        .name
        .clone()
        .unwrap_or_else(|| format!("icarus-{}", &args.canister_id[..5]));

    let server_url = match args.network.as_str() {
        "local" => format!("http://127.0.0.1:{}/mcp", args.port.unwrap_or(3000)),
        "ic" => format!("https://{}.icp0.io/mcp", args.canister_id),
        "testnet" => format!("https://{}.testnet.dfinity.network/mcp", args.canister_id),
        _ => return Err(anyhow!("Unsupported network: {}", args.network)),
    };

    use crate::types::{CanisterId, Network, ServerName};

    Ok(McpServerConfig {
        name: ServerName::new(server_name)?,
        canister_id: CanisterId::new(args.canister_id.clone())?,
        network: args.network.parse()?,
        url: server_url,
        client: client_config.name.clone(),
        port: args.port,
        enabled: true,
        created_at: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
    })
}

async fn register_with_client(
    server_config: &McpServerConfig,
    client_config: &ClientConfig,
    client_type: &crate::commands::mcp::McpClient,
) -> Result<()> {
    match client_type {
        crate::commands::mcp::McpClient::ClaudeDesktop => {
            register_claude_desktop(server_config, client_config).await
        }
        crate::commands::mcp::McpClient::ClaudeCode => {
            register_claude_code(server_config, client_config).await
        }
        crate::commands::mcp::McpClient::ChatgptDesktop => {
            register_chatgpt_desktop(server_config, client_config).await
        }
        crate::commands::mcp::McpClient::Continue => {
            register_continue(server_config, client_config).await
        }
        crate::commands::mcp::McpClient::Custom => {
            // Custom clients require manual configuration
            Ok(())
        }
    }
}

async fn register_claude_desktop(
    server_config: &McpServerConfig,
    client_config: &ClientConfig,
) -> Result<()> {
    use tokio::fs;

    // Load existing Claude Desktop configuration
    let config_content = if client_config.config_path.exists() {
        fs::read_to_string(&client_config.config_path).await?
    } else {
        "{}".to_string()
    };

    let mut config: Value = serde_json::from_str(&config_content)?;

    // Ensure mcpServers object exists
    if config.get("mcpServers").is_none() {
        config["mcpServers"] = serde_json::json!({});
    }

    // Add our server configuration
    config["mcpServers"][server_config.name.as_str()] = serde_json::json!({
        "command": "icarus",
        "args": ["mcp", "start", "--port", server_config.port.unwrap_or(3000)],
        "env": {
            "ICARUS_CANISTER_ID": server_config.canister_id.as_str(),
            "ICARUS_NETWORK": server_config.network.as_str()
        }
    });

    // Write updated configuration
    let updated_config = serde_json::to_string_pretty(&config)?;
    fs::write(&client_config.config_path, updated_config).await?;

    Ok(())
}

async fn register_claude_code(
    server_config: &McpServerConfig,
    client_config: &ClientConfig,
) -> Result<()> {
    // Similar to Claude Desktop but with different configuration format
    register_claude_desktop(server_config, client_config).await
}

async fn register_chatgpt_desktop(
    server_config: &McpServerConfig,
    client_config: &ClientConfig,
) -> Result<()> {
    // ChatGPT Desktop specific configuration
    use tokio::fs;

    let config_content = if client_config.config_path.exists() {
        fs::read_to_string(&client_config.config_path).await?
    } else {
        "{}".to_string()
    };

    let mut config: Value = serde_json::from_str(&config_content)?;

    // ChatGPT Desktop uses different configuration structure
    if config.get("mcp").is_none() {
        config["mcp"] = serde_json::json!({});
    }

    config["mcp"][server_config.name.as_str()] = serde_json::json!({
        "url": server_config.url,
        "canister_id": server_config.canister_id.as_str(),
        "network": server_config.network.as_str()
    });

    let updated_config = serde_json::to_string_pretty(&config)?;
    fs::write(&client_config.config_path, updated_config).await?;

    Ok(())
}

async fn register_continue(
    server_config: &McpServerConfig,
    client_config: &ClientConfig,
) -> Result<()> {
    // Continue VS Code extension configuration
    use tokio::fs;

    let config_content = if client_config.config_path.exists() {
        fs::read_to_string(&client_config.config_path).await?
    } else {
        "{}".to_string()
    };

    let mut config: Value = serde_json::from_str(&config_content)?;

    if config.get("mcp").is_none() {
        config["mcp"] = serde_json::json!([]);
    }

    // Continue uses array format
    if let Some(mcp_array) = config["mcp"].as_array_mut() {
        mcp_array.push(serde_json::json!({
            "name": server_config.name,
            "url": server_config.url,
            "canister_id": server_config.canister_id,
            "network": server_config.network
        }));
    }

    let updated_config = serde_json::to_string_pretty(&config)?;
    fs::write(&client_config.config_path, updated_config).await?;

    Ok(())
}

fn print_success_message(server_config: &McpServerConfig, client_config: &ClientConfig) {
    println!(
        "\n{}",
        "✅ MCP Server Registered Successfully!"
            .bright_green()
            .bold()
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!(
        "{} {}",
        "Server Name:".bright_white(),
        server_config.name.to_string().bright_cyan()
    );
    println!(
        "{} {}",
        "Canister ID:".bright_white(),
        server_config.canister_id.to_string().bright_cyan()
    );
    println!(
        "{} {}",
        "Network:".bright_white(),
        server_config.network.to_string().bright_cyan()
    );
    println!(
        "{} {}",
        "URL:".bright_white(),
        server_config.url.bright_cyan()
    );
    println!(
        "{} {}",
        "Client:".bright_white(),
        client_config.name.bright_cyan()
    );

    println!("\n{}", "Next steps:".bright_white().bold());
    println!(
        "  {} Restart your AI client to load the new MCP server",
        "1.".bright_yellow()
    );
    println!(
        "  {} Start the MCP bridge: icarus mcp start",
        "2.".bright_yellow()
    );
    println!(
        "  {} Test the connection: icarus mcp status {}",
        "3.".bright_yellow(),
        server_config.name
    );

    println!(
        "\n{}",
        "🎉 Your canister is now ready for AI integration!".bright_green()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_canister_id() {
        // Valid canister IDs
        assert!(validate_canister_id("rdmx6-jaaaa-aaaaa-aaadq-cai").is_ok());
        assert!(validate_canister_id("rrkah-fqaaa-aaaaa-aaaaq-cai").is_ok());

        // Invalid canister IDs
        assert!(validate_canister_id("").is_err());
        assert!(validate_canister_id("invalid").is_err());
        assert!(validate_canister_id("too-short").is_err());
    }

    #[test]
    fn test_create_server_config() {
        let args = AddArgs {
            canister_id: "rdmx6-jaaaa-aaaaa-aaadq-cai".to_string(),
            client: crate::commands::mcp::McpClient::ClaudeDesktop,
            client_name: None,
            port: Some(3000),
            network: "local".to_string(),
            name: Some("test-server".to_string()),
            skip_verify: false,
        };

        let client_config = ClientConfig {
            name: "claude-desktop".to_string(),
            config_path: PathBuf::from("/tmp/config.json"),
            install_path: None,
        };

        let server_config = create_server_config(&args, &client_config).unwrap();

        assert_eq!(server_config.name, "test-server");
        assert_eq!(server_config.canister_id, "rdmx6-jaaaa-aaaaa-aaadq-cai");
        assert_eq!(server_config.network, "local");
        assert_eq!(server_config.port, Some(3000));
    }
}
