use anyhow::Result;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Table};

use crate::config::mcp::McpConfig;
use crate::{
    commands::mcp::{ListArgs, OutputFormat},
    Cli,
};

pub(crate) async fn execute(args: ListArgs, cli: &Cli) -> Result<()> {
    let mcp_config = McpConfig::load().await.unwrap_or_default();

    // Apply filters using iterators (avoid cloning entire vector)
    let filtered_servers: Vec<&crate::config::mcp::McpServerConfig> = mcp_config
        .servers
        .iter()
        .filter(|server| {
            // Client filter
            if let Some(ref client_filter) = args.client {
                if server.client != client_filter.to_string() {
                    return false;
                }
            }
            // Active filter
            if args.active && !server.enabled {
                return false;
            }
            true
        })
        .collect();

    // Output based on format
    match args.format {
        OutputFormat::Table => print_table(&filtered_servers, args.detailed, cli),
        OutputFormat::Json => print_json(&filtered_servers)?,
        OutputFormat::Yaml => print_yaml(&filtered_servers)?,
        OutputFormat::Plain => print_plain(&filtered_servers),
    }

    Ok(())
}

fn print_table(servers: &[&crate::config::mcp::McpServerConfig], detailed: bool, cli: &Cli) {
    if servers.is_empty() {
        if !cli.quiet {
            println!("{}", "No MCP servers registered.".yellow());
            println!("Use 'icarus mcp add <canister-id> --client <client>' to register a server.");
        }
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    // Set headers based on detail level
    if detailed {
        table.set_header(vec![
            "Name".bright_white().bold(),
            "Canister ID".bright_white().bold(),
            "Network".bright_white().bold(),
            "Client".bright_white().bold(),
            "URL".bright_white().bold(),
            "Status".bright_white().bold(),
            "Created".bright_white().bold(),
            "Updated".bright_white().bold(),
        ]);
    } else {
        table.set_header(vec![
            "Name".bright_white().bold(),
            "Canister ID".bright_white().bold(),
            "Network".bright_white().bold(),
            "Client".bright_white().bold(),
            "Status".bright_white().bold(),
        ]);
    }

    // Add rows
    for server in servers {
        let status = if server.enabled {
            "✅ Active".green()
        } else {
            "❌ Disabled".red()
        };

        if detailed {
            table.add_row(vec![
                server.name.to_string().bright_cyan().to_string(),
                server.canister_id.to_string().bright_blue().to_string(),
                server.network.to_string().bright_yellow().to_string(),
                server.client.bright_magenta().to_string(),
                server.url.bright_white().to_string(),
                status.to_string(),
                format_timestamp(&server.created_at),
                format_timestamp(&server.last_updated),
            ]);
        } else {
            table.add_row(vec![
                server.name.to_string().bright_cyan().to_string(),
                server.canister_id.to_string().bright_blue().to_string(),
                server.network.to_string().bright_yellow().to_string(),
                server.client.bright_magenta().to_string(),
                status.to_string(),
            ]);
        }
    }

    if !cli.quiet {
        println!("\n{}", "📋 Registered MCP Servers".bright_white().bold());
        println!("{}", table);
        println!(
            "\n{} servers total",
            servers.len().to_string().bright_cyan()
        );
    } else {
        println!("{}", table);
    }
}

fn print_json(servers: &[&crate::config::mcp::McpServerConfig]) -> Result<()> {
    let json = serde_json::to_string_pretty(servers)?;
    println!("{}", json);
    Ok(())
}

fn print_yaml(servers: &[&crate::config::mcp::McpServerConfig]) -> Result<()> {
    // For now, we'll convert to JSON and then to YAML format manually
    // In a real implementation, you might want to use a YAML library
    let _json_value: serde_json::Value = serde_json::to_value(servers)?;

    // Simple YAML-like output (basic implementation)
    println!("servers:");
    for (i, server) in servers.iter().enumerate() {
        println!("  - name: {}", server.name);
        println!("    canister_id: {}", server.canister_id);
        println!("    network: {}", server.network);
        println!("    client: {}", server.client);
        println!("    url: {}", server.url);
        println!("    enabled: {}", server.enabled);
        println!("    created_at: {}", server.created_at.to_rfc3339());
        println!("    last_updated: {}", server.last_updated.to_rfc3339());
        if i < servers.len() - 1 {
            println!();
        }
    }

    Ok(())
}

fn print_plain(servers: &[&crate::config::mcp::McpServerConfig]) {
    for server in servers {
        println!("{}", server.name);
    }
}

fn format_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(*timestamp);

    if duration.num_days() > 0 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} minutes ago", duration.num_minutes())
    } else {
        "Just now".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_format_timestamp() {
        let now = Utc::now();

        // Test "just now"
        assert_eq!(format_timestamp(&now), "Just now");

        // Test minutes ago
        let minutes_ago = now - chrono::Duration::minutes(5);
        assert_eq!(format_timestamp(&minutes_ago), "5 minutes ago");

        // Test hours ago
        let hours_ago = now - chrono::Duration::hours(2);
        assert_eq!(format_timestamp(&hours_ago), "2 hours ago");

        // Test days ago
        let days_ago = now - chrono::Duration::days(3);
        assert_eq!(format_timestamp(&days_ago), "3 days ago");
    }

    #[tokio::test]
    async fn test_empty_server_list() {
        let args = ListArgs {
            client: None,
            detailed: false,
            active: false,
            format: OutputFormat::Table,
        };

        let cli = crate::Cli {
            verbose: false,
            quiet: true,
            force: false,
            command: crate::Commands::Mcp(crate::commands::McpArgs::List(args.clone())),
        };

        // This should not panic with empty server list
        let result = execute(args, &cli).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_serialization() {
        use crate::config::mcp::McpServerConfig;
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

        let servers = vec![server];
        let server_refs: Vec<&McpServerConfig> = servers.iter().collect();
        let result = print_json(&server_refs);
        assert!(result.is_ok());
    }
}
