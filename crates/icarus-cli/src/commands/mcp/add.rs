//! Add MCP server to AI client configurations

use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;

use crate::utils::{
    mcp_clients::{ClientRegistry, ClientType, McpClient},
    print_info, print_warning, ui,
};

/// Add MCP server to AI client configurations
pub async fn execute(
    canister_id: String,
    name: Option<String>,
    clients: Option<Vec<String>>,
    all_clients: bool,
    config_path: Option<String>,
    skip_confirmation: bool,
) -> Result<()> {
    // Validate canister ID format
    if candid::Principal::from_text(&canister_id).is_err() {
        anyhow::bail!("Invalid canister ID format: {}", canister_id);
    }

    let project_name = name.unwrap_or_else(|| {
        // Try to derive name from canister ID or use a default
        format!("icarus-{}", &canister_id[..8])
    });

    ui::display_header(&format!("🚀 Adding MCP Server '{}'", project_name));
    ui::display_info_styled(&format!("Canister ID: {}", canister_id));

    let registry = ClientRegistry::new();
    let all_client_info = registry.get_all_client_info();

    // Filter to only installed clients
    let installed_clients: Vec<_> = all_client_info
        .iter()
        .filter(|info| info.is_installed)
        .collect();

    if installed_clients.is_empty() {
        ui::display_warning_styled("No supported AI clients detected on this system.");
        println!();
        ui::display_section("Supported Clients");
        for client_type in ClientType::all() {
            println!("  {} {}", client_type.emoji(), client_type.display_name());
        }
        return Ok(());
    }

    // Determine which clients to configure
    let clients_to_configure = if all_clients {
        // Configure all installed clients
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
        // Interactive client selection with beautiful UI
        let selections = ui::select_clients_interactive_beautiful(&installed_clients)?;
        selections
            .into_iter()
            .map(|i| installed_clients[i])
            .collect()
    };

    if clients_to_configure.is_empty() {
        print_warning("No clients selected for configuration.");
        return Ok(());
    }

    // Show configuration preview
    ui::display_section("Configuration Preview");
    println!("  📋 Server Name: {}", project_name.cyan().bold());
    println!("  🆔 Canister ID: {}", canister_id.yellow().bold());
    println!("  🎯 Target Clients:");

    for client_info in &clients_to_configure {
        let client = registry
            .get_client(client_info.client_type)
            .context("Failed to get client implementation")?;

        // Generate preview of server configuration
        let server_config = client.generate_server_config(&project_name, &canister_id);

        // Determine config path to show user
        let preview_config_path = if let Some(custom_path) = config_path.as_ref() {
            custom_path.clone()
        } else {
            match client.find_config_path() {
                Ok(path) => path.display().to_string(),
                Err(_) => client_info.config_path.display().to_string(),
            }
        };

        println!(
            "    {} {} {}",
            client_info.client_type.emoji(),
            client_info.client_type.display_name().bold(),
            format!("({})", preview_config_path).dimmed()
        );

        // Show a snippet of what will be added
        if let Some(command_obj) = server_config.get("command") {
            if let Some(command_arr) = command_obj.as_array() {
                if let Some(first_cmd) = command_arr.first() {
                    println!(
                        "      └── Command: {}",
                        first_cmd.as_str().unwrap_or("").dimmed()
                    );
                }
            }
        }
    }

    println!();

    // Ask for confirmation unless skipped
    if !skip_confirmation
        && !ui::confirm_beautiful(&format!(
            "Proceed with configuring {} client(s)?",
            clients_to_configure.len()
        ))?
    {
        ui::display_info_styled("Configuration cancelled by user.");
        return Ok(());
    }

    // Configure each selected client with progress tracking
    let mut results = HashMap::new();
    let progress =
        ui::create_progress_bar(clients_to_configure.len() as u64, "Configuring clients");

    for (i, client_info) in clients_to_configure.iter().enumerate() {
        let client = registry
            .get_client(client_info.client_type)
            .context("Failed to get client implementation")?;

        progress.set_message(format!(
            "Configuring {} {}...",
            client_info.client_type.emoji(),
            client_info.client_type.display_name()
        ));

        let result = configure_client(
            client,
            &project_name,
            &canister_id,
            client_info,
            config_path.as_ref(),
        )
        .await;
        results.insert(client_info.client_type, result);

        progress.set_position(i as u64 + 1);
    }

    progress.finish_and_clear();

    // Report results with beautiful UI
    ui::display_section("Configuration Results");

    let mut success_count = 0;
    let mut total_count = 0;

    for (client_type, result) in results {
        total_count += 1;
        match result {
            Ok(_) => {
                println!(
                    "  {} {} {}",
                    "✅".green(),
                    client_type.emoji(),
                    client_type.display_name().green().bold()
                );
                success_count += 1;
            }
            Err(e) => {
                println!(
                    "  {} {} {}: {}",
                    "❌".red(),
                    client_type.emoji(),
                    client_type.display_name().red().bold(),
                    e.to_string().red()
                );
            }
        }
    }

    println!();
    if success_count == total_count {
        ui::display_success_animated(&format!(
            "Successfully configured '{}' in {} client(s)!",
            project_name, success_count
        ));
        ui::display_info_styled("💡 Restart the AI clients to load the new MCP server.");
        ui::display_info_styled(&format!(
            "🚀 Test your server: icarus mcp start {}",
            canister_id
        ));
    } else if success_count > 0 {
        ui::display_warning_styled(&format!(
            "Partially successful: {}/{} clients configured",
            success_count, total_count
        ));
        ui::display_info_styled("Check the errors above and try again for failed clients.");
        ui::display_info_styled(&format!(
            "🚀 Test your server: icarus mcp start {}",
            canister_id
        ));
    } else {
        ui::display_error_styled("Failed to configure any clients.");
        anyhow::bail!("All client configurations failed");
    }

    Ok(())
}

async fn configure_client(
    client: &dyn McpClient,
    project_name: &str,
    canister_id: &str,
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

    // Generate server configuration
    let server_config = client.generate_server_config(project_name, canister_id);

    // Update the configuration
    client
        .update_config(&config_path, project_name, &server_config)
        .context("Failed to update client configuration")?;

    // Validate the configuration
    client
        .validate_config(&config_path)
        .context("Configuration validation failed")?;

    Ok(())
}
