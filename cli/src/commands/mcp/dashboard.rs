//! MCP status dashboard - comprehensive overview of MCP configuration and health

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;

use crate::utils::{
    mcp_clients::{ClientRegistry, ClientType},
    ui,
};

/// Show comprehensive MCP status dashboard
pub async fn execute() -> Result<()> {
    ui::display_header("🎛️ MCP Status Dashboard");

    let spinner = ui::create_beautiful_spinner("Analyzing MCP ecosystem...");

    let registry = ClientRegistry::new();
    let all_client_info = registry.get_all_client_info();

    spinner.finish_and_clear();

    // Collect comprehensive status data
    let mut status_data = HashMap::new();
    let mut total_servers = 0;
    let mut active_clients = 0;
    let total_clients = all_client_info.len();

    // Progress bar for health checks
    let installed_client_infos: Vec<_> = all_client_info
        .iter()
        .filter(|info| info.is_installed)
        .collect();
    let installed_clients = installed_client_infos.len();

    if !installed_client_infos.is_empty() {
        let progress = ui::create_progress_bar(
            installed_client_infos.len() as u64,
            "Performing health checks",
        );

        for (i, client_info) in installed_client_infos.iter().enumerate() {
            progress.set_message(format!(
                "Checking {} {}...",
                client_info.client_type.emoji(),
                client_info.client_type.display_name()
            ));

            let client = registry
                .get_client(client_info.client_type.clone())
                .expect("Failed to get client implementation");

            let servers = match client.list_servers(&client_info.config_path) {
                Ok(servers) => {
                    total_servers += servers.len();
                    if !servers.is_empty() {
                        active_clients += 1;
                    }
                    servers
                }
                Err(_) => vec![],
            };

            status_data.insert(
                client_info.client_type.clone(),
                (true, servers, client_info.config_path.clone()),
            );
            progress.set_position(i as u64 + 1);
        }

        progress.finish_and_clear();
    }

    // Add uninstalled clients to status data
    for client_info in &all_client_info {
        if !client_info.is_installed {
            status_data
                .entry(client_info.client_type.clone())
                .or_insert((false, vec![], client_info.config_path.clone()));
        }
    }

    // Display comprehensive status overview
    ui::display_section("System Overview");
    println!(
        "  📊 Total AI Clients: {}",
        total_clients.to_string().cyan().bold()
    );
    println!(
        "  ✅ Installed Clients: {}",
        installed_clients.to_string().green().bold()
    );
    println!(
        "  🎯 Active Clients: {}",
        active_clients.to_string().blue().bold()
    );
    println!(
        "  🚀 Total MCP Servers: {}",
        total_servers.to_string().yellow().bold()
    );

    // Calculate health score
    let health_score = if total_clients > 0 {
        ((active_clients as f64 / total_clients as f64) * 100.0) as u32
    } else {
        0
    };

    let (health_emoji, health_color): (&str, fn(&str) -> colored::ColoredString) =
        match health_score {
            90..=100 => ("🟢", |s| s.green()),
            70..=89 => ("🟡", |s| s.yellow()),
            50..=69 => ("🟠", |s| s.truecolor(255, 165, 0)),
            _ => ("🔴", |s| s.red()),
        };

    println!(
        "  {} System Health: {}",
        health_emoji,
        health_color(&format!("{}%", health_score)).bold()
    );

    // Detailed client status table
    ui::display_section("Client Status Details");

    for client_type in ClientType::all() {
        if let Some((is_installed, servers, config_path)) = status_data.get(&client_type) {
            let status_emoji = if *is_installed {
                if servers.is_empty() {
                    "🟡" // Installed but no servers
                } else {
                    "🟢" // Installed with servers
                }
            } else {
                "⚫" // Not installed
            };

            let status_text = if *is_installed {
                if servers.is_empty() {
                    "Ready (no servers)".yellow()
                } else {
                    format!("Active ({} servers)", servers.len()).green()
                }
            } else {
                "Not installed".dimmed()
            };

            println!(
                "  {} {} {} {}",
                status_emoji,
                client_type.emoji(),
                client_type.display_name().bold(),
                status_text
            );

            // Show servers if any
            for (i, server) in servers.iter().enumerate() {
                let server_prefix = if i == servers.len() - 1 {
                    "    └──"
                } else {
                    "    ├──"
                };
                println!("{} 🚀 {}", server_prefix.dimmed(), server.cyan());
            }

            // Show config path
            println!(
                "    {} Config: {}",
                "📁".dimmed(),
                config_path.display().to_string().dimmed()
            );
        }
        println!();
    }

    // System recommendations
    ui::display_section("Recommendations");

    if installed_clients == 0 {
        ui::display_warning_styled("⚠️ No AI clients are installed on this system");
        ui::display_info_styled(
            "💡 Install Claude Desktop, ChatGPT Desktop, or Claude Code to get started",
        );
    } else if active_clients == 0 {
        ui::display_info_styled("💡 No MCP servers are configured");
        ui::display_info_styled("💡 Use 'icarus mcp add <canister-id>' to add an MCP server");
    } else if active_clients < installed_clients {
        let inactive_count = installed_clients - active_clients;
        ui::display_info_styled(&format!(
            "💡 {} installed client(s) don't have MCP servers configured",
            inactive_count
        ));
        ui::display_info_styled("💡 Consider adding MCP servers to maximize your AI capabilities");
    } else {
        ui::display_success_animated("🎉 All installed clients are configured with MCP servers!");
        ui::display_info_styled("💡 Your MCP ecosystem is optimally configured");
    }

    // Quick actions
    ui::display_section("Quick Actions");
    println!(
        "  {} Add MCP server:     {}",
        "⚡".yellow(),
        "icarus mcp add <canister-id>".cyan()
    );
    println!(
        "  {} List configurations: {}",
        "📋".blue(),
        "icarus mcp list".cyan()
    );
    println!(
        "  {} Remove server:      {}",
        "🗑️".red(),
        "icarus mcp remove <server-name>".cyan()
    );

    Ok(())
}
