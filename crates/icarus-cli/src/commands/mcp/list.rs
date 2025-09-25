//! List configured MCP servers across AI clients

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;

use crate::utils::{
    mcp_clients::{ClientRegistry, ClientType},
    ui,
};

/// List configured MCP servers across AI clients
pub async fn execute(client_filter: Option<String>) -> Result<()> {
    ui::display_header("🚀 MCP Server Configuration Overview");

    let spinner =
        ui::create_beautiful_spinner("Scanning AI clients for MCP server configurations...");

    let registry = ClientRegistry::new();
    let all_client_info = registry.get_all_client_info();

    // Filter clients if specified
    let clients_to_check = if let Some(filter) = client_filter {
        let client_type = match filter.to_lowercase().as_str() {
            "claude" | "claude-desktop" | "claude_desktop" => Some(ClientType::ClaudeDesktop),
            "chatgpt" | "chatgpt-desktop" | "chatgpt_desktop" => Some(ClientType::ChatGptDesktop),
            "claude-code" | "claude_code" | "cline" => Some(ClientType::ClaudeCode),
            _ => {
                ui::display_warning_styled(&format!("Unknown client type: {}", filter));
                return Ok(());
            }
        };

        if let Some(ct) = client_type {
            all_client_info
                .iter()
                .filter(|info| info.client_type == ct)
                .cloned()
                .collect()
        } else {
            vec![]
        }
    } else {
        all_client_info.clone()
    };

    spinner.finish_and_clear();

    let mut found_any_servers = false;
    let mut client_results: HashMap<ClientType, Vec<String>> = HashMap::new();

    // Check each client for configured servers with progress tracking
    let installed_clients: Vec<_> = clients_to_check
        .iter()
        .filter(|info| info.is_installed)
        .collect();

    if !installed_clients.is_empty() {
        let progress = ui::create_progress_bar(
            installed_clients.len() as u64,
            "Checking client configurations",
        );

        for (i, client_info) in installed_clients.iter().enumerate() {
            progress.set_message(format!(
                "Checking {} {}...",
                client_info.client_type.emoji(),
                client_info.client_type.display_name()
            ));

            let client = registry
                .get_client(client_info.client_type)
                .expect("Failed to get client implementation");

            match client.list_servers(&client_info.config_path) {
                Ok(servers) => {
                    if !servers.is_empty() {
                        found_any_servers = true;
                    }
                    client_results.insert(client_info.client_type, servers);
                }
                Err(e) => {
                    ui::display_warning_styled(&format!(
                        "Could not read {} configuration: {}",
                        client_info.client_type.display_name(),
                        e
                    ));
                    client_results.insert(client_info.client_type, vec![]);
                }
            }

            progress.set_position(i as u64 + 1);
        }

        progress.finish_and_clear();
    }

    // Display results using beautiful tree visualization
    let tree_data: Vec<_> = client_results
        .iter()
        .map(|(client_type, servers)| {
            let client_info = clients_to_check
                .iter()
                .find(|info| info.client_type == *client_type);
            let is_installed = client_info.map(|info| info.is_installed).unwrap_or(false);
            (*client_type, is_installed, servers.clone())
        })
        .collect();

    ui::display_config_tree(&tree_data);

    // Show installation status for clients not configured
    let unconfigured_clients: Vec<_> = all_client_info
        .iter()
        .filter(|info| !client_results.contains_key(&info.client_type))
        .collect();

    if !unconfigured_clients.is_empty() {
        ui::display_section("Client Installation Status");

        for client_info in unconfigured_clients {
            let (emoji, status_msg, color_fn): (&str, &str, fn(&str) -> colored::ColoredString) =
                if client_info.is_installed {
                    ("✅", "installed, no servers configured", |s| s.yellow())
                } else {
                    ("❌", "not installed", |s| s.red())
                };
            println!(
                "  {} {} {} ({})",
                emoji,
                client_info.client_type.emoji(),
                client_info.client_type.display_name(),
                color_fn(status_msg)
            );
        }
    }

    println!();

    if !found_any_servers {
        ui::display_info_styled("💡 No MCP servers are currently configured.");
        ui::display_info_styled("💡 Use 'icarus mcp add <canister-id>' to add an MCP server.");
    } else {
        let active_clients = client_results
            .iter()
            .filter(|(_, servers)| !servers.is_empty())
            .count();
        ui::display_success_animated(&format!(
            "Found MCP servers in {} client(s)! 🎉",
            active_clients
        ));
    }

    Ok(())
}
