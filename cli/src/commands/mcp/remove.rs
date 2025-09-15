//! Remove MCP server from AI client configurations

use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use std::collections::HashMap;

use crate::utils::{
    mcp_clients::{ClientRegistry, ClientType, McpClient},
    print_error, print_info, print_success, print_warning,
};

/// Remove MCP server from AI client configurations
pub async fn execute(
    server_name: String,
    clients: Option<Vec<String>>,
    all_clients: bool,
    config_path: Option<String>,
) -> Result<()> {
    print_info(&format!(
        "Removing MCP server '{}' from AI clients...",
        server_name
    ));

    let registry = ClientRegistry::new();
    let all_client_info = registry.get_all_client_info();

    // Filter to only installed clients
    let installed_clients: Vec<_> = all_client_info
        .iter()
        .filter(|info| info.is_installed)
        .collect();

    if installed_clients.is_empty() {
        print_warning("No supported AI clients detected on this system.");
        return Ok(());
    }

    // Determine which clients to check/remove from
    let clients_to_check = if all_clients {
        // Check all installed clients
        installed_clients.clone()
    } else if let Some(client_names) = clients {
        // Use specified clients
        let mut selected_clients = Vec::new();
        for client_name in client_names {
            let client_type = match client_name.to_lowercase().as_str() {
                "claude" | "claude-desktop" | "claude_desktop" => ClientType::ClaudeDesktop,
                "chatgpt" | "chatgpt-desktop" | "chatgpt_desktop" => ClientType::ChatGptDesktop,
                "claude-code" | "claude_code" | "cline" => ClientType::ClaudeCode,
                _ => {
                    print_warning(&format!("Unknown client type: {}", client_name));
                    continue;
                }
            };

            if let Some(info) = installed_clients
                .iter()
                .find(|info| info.client_type == client_type)
            {
                selected_clients.push(*info);
            } else {
                print_warning(&format!(
                    "{} is not installed or not detected",
                    client_type.display_name()
                ));
            }
        }
        selected_clients
    } else {
        // Interactive client selection - but first check which clients have this server
        find_clients_with_server(&registry, &installed_clients, &server_name).await?
    };

    if clients_to_check.is_empty() {
        print_warning(&format!(
            "Server '{}' not found in any client configuration.",
            server_name
        ));
        return Ok(());
    }

    // Remove from each selected client
    let mut results = HashMap::new();
    for client_info in &clients_to_check {
        let client = registry
            .get_client(client_info.client_type.clone())
            .context("Failed to get client implementation")?;

        print_info(&format!(
            "Removing from {}...",
            client_info.client_type.display_name()
        ));

        let result = remove_from_client(
            client.as_ref(),
            &server_name,
            client_info,
            config_path.as_ref(),
        )
        .await;
        results.insert(client_info.client_type.clone(), result);
    }

    // Report results
    println!();
    println!("{}", "Removal Results:".bold().cyan());
    println!("{}", "─".repeat(50).cyan());

    let mut success_count = 0;
    let mut total_count = 0;

    for (client_type, result) in results {
        total_count += 1;
        match result {
            Ok(_) => {
                print_success(&format!("✓ {}", client_type.display_name()));
                success_count += 1;
            }
            Err(e) => {
                print_error(&format!("✗ {}: {}", client_type.display_name(), e));
            }
        }
    }

    println!();
    if success_count == total_count {
        print_success(&format!(
            "Successfully removed '{}' from {} client(s)!",
            server_name, success_count
        ));
        print_info("Restart the AI clients to apply the changes.");
    } else if success_count > 0 {
        print_warning(&format!(
            "Partially successful: removed from {}/{} clients",
            success_count, total_count
        ));
    } else {
        print_error("Failed to remove from any clients.");
        anyhow::bail!("All removal operations failed");
    }

    Ok(())
}

async fn find_clients_with_server<'a>(
    registry: &'a ClientRegistry,
    installed_clients: &'a [&'a crate::utils::mcp_clients::ClientInfo],
    server_name: &'a str,
) -> Result<Vec<&'a crate::utils::mcp_clients::ClientInfo>> {
    let mut clients_with_server = Vec::new();

    // Check each client to see if it has this server configured
    for client_info in installed_clients {
        let client = registry
            .get_client(client_info.client_type.clone())
            .context("Failed to get client implementation")?;

        if let Ok(servers) = client.list_servers(&client_info.config_path) {
            if servers.contains(&server_name.to_string()) {
                clients_with_server.push(*client_info);
            }
        }
    }

    if clients_with_server.is_empty() {
        return Ok(vec![]);
    }

    // If multiple clients have the server, ask user which ones to remove from
    if clients_with_server.len() == 1 {
        // Only one client has it, ask for confirmation
        let client = clients_with_server[0];
        println!();
        print_info(&format!(
            "Found server '{}' in {}",
            server_name,
            client.client_type.display_name()
        ));

        let theme = ColorfulTheme::default();
        let confirmation = dialoguer::Confirm::with_theme(&theme)
            .with_prompt("Remove from this client?")
            .default(true)
            .interact()?;

        if confirmation {
            Ok(clients_with_server)
        } else {
            Ok(vec![])
        }
    } else {
        // Multiple clients have it, show selection menu
        println!();
        print_info(&format!(
            "Found server '{}' in multiple clients:",
            server_name
        ));

        let client_options: Vec<String> = clients_with_server
            .iter()
            .map(|info| info.client_type.display_name().to_string())
            .collect();

        let theme = ColorfulTheme::default();

        // Ask if they want to remove from all or select specific ones
        let choices = vec!["Remove from all clients", "Select specific clients"];
        let selection = Select::with_theme(&theme)
            .with_prompt("How would you like to proceed?")
            .items(&choices)
            .default(0)
            .interact()?;

        if selection == 0 {
            // Remove from all
            Ok(clients_with_server)
        } else {
            // Select specific clients
            let selections = MultiSelect::with_theme(&theme)
                .with_prompt(
                    "Select clients to remove from (use space to select, enter to confirm)",
                )
                .items(&client_options)
                .interact()?;

            let selected_clients: Vec<_> = selections
                .into_iter()
                .map(|i| clients_with_server[i])
                .collect();

            Ok(selected_clients)
        }
    }
}

async fn remove_from_client(
    client: &dyn McpClient,
    server_name: &str,
    client_info: &crate::utils::mcp_clients::ClientInfo,
    custom_config_path: Option<&String>,
) -> Result<()> {
    // Determine config path: custom path > default detection > fallback to client info
    let config_path = if let Some(custom_path) = custom_config_path {
        std::path::PathBuf::from(custom_path)
    } else {
        match client.find_config_path() {
            Ok(path) => path,
            Err(_) => {
                // Use the detected config path from client info as fallback
                client_info.config_path.clone()
            }
        }
    };

    print_info(&format!("Using config path: {}", config_path.display()));

    // Remove the server configuration
    client
        .remove_config(&config_path, server_name)
        .context("Failed to remove server from client configuration")?;

    Ok(())
}
