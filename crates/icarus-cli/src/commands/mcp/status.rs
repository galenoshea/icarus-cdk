use anyhow::{anyhow, Result};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Table};
use std::time::Duration;
use tokio::time::timeout;
use tracing::info;

use crate::config::mcp::McpConfig;
use crate::{commands::mcp::StatusArgs, Cli};

#[derive(Debug)]
struct ServerStatus {
    name: String,
    canister_id: String,
    network: String,
    #[allow(dead_code)]
    url: String,
    health: HealthStatus,
    response_time: Option<Duration>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
enum HealthStatus {
    Healthy,
    Unhealthy,
    Unreachable,
    Timeout,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(not(test))]
        {
            match self {
                HealthStatus::Healthy => write!(f, "{}", "✅ Healthy".green()),
                HealthStatus::Unhealthy => write!(f, "{}", "⚠️  Unhealthy".yellow()),
                HealthStatus::Unreachable => write!(f, "{}", "❌ Unreachable".red()),
                HealthStatus::Timeout => write!(f, "{}", "⏰ Timeout".red()),
            }
        }

        #[cfg(test)]
        {
            match self {
                HealthStatus::Healthy => write!(f, "✅ Healthy"),
                HealthStatus::Unhealthy => write!(f, "⚠️  Unhealthy"),
                HealthStatus::Unreachable => write!(f, "❌ Unreachable"),
                HealthStatus::Timeout => write!(f, "⏰ Timeout"),
            }
        }
    }
}

pub(crate) async fn execute(args: StatusArgs, cli: &Cli) -> Result<()> {
    let mcp_config = McpConfig::load().await.unwrap_or_default();

    if mcp_config.servers.is_empty() {
        if !cli.quiet {
            println!("{}", "No MCP servers registered.".yellow());
            println!("Use 'icarus mcp add <canister-id> --client <client>' to register a server.");
        }
        // If looking for a specific server but none exist, that's an error
        if args.identifier.is_some() {
            return Err(anyhow!("No MCP server found"));
        }
        // If checking all servers (--all) and none exist, that's OK
        return Ok(());
    }

    let servers_to_check = if args.all {
        mcp_config.servers.clone()
    } else if let Some(ref identifier) = args.identifier {
        let server = mcp_config
            .servers
            .iter()
            .find(|s| s.name == *identifier || s.canister_id == *identifier)
            .cloned();

        match server {
            Some(s) => vec![s],
            None => {
                return Err(anyhow!(
                    "No MCP server found with identifier: {}",
                    identifier
                ));
            }
        }
    } else {
        // Show status for all servers if no specific identifier
        mcp_config.servers.clone()
    };

    if !cli.quiet {
        println!("{} Checking MCP server status...", "→".bright_blue());
    }

    let mut statuses = Vec::new();
    for server in servers_to_check {
        if !cli.quiet && !args.all {
            println!(
                "  {} Checking {}...",
                "→".bright_blue(),
                server.name.to_string().bright_cyan()
            );
        }

        let status = check_server_health(&server, args.timeout).await;
        statuses.push(status);
    }

    if !cli.quiet {
        print_status_table(&statuses);
        print_status_summary(&statuses);
    }

    // Exit with error code if any servers are unhealthy
    let unhealthy_count = statuses
        .iter()
        .filter(|s| !matches!(s.health, HealthStatus::Healthy))
        .count();

    if unhealthy_count > 0 {
        info!(
            "{} out of {} servers are unhealthy",
            unhealthy_count,
            statuses.len()
        );
        std::process::exit(1);
    }

    info!("All MCP servers are healthy");
    Ok(())
}

async fn check_server_health(
    server: &crate::config::mcp::McpServerConfig,
    timeout_seconds: u64,
) -> ServerStatus {
    let start_time = std::time::Instant::now();

    // Try to connect to the server
    let health_result = timeout(
        Duration::from_secs(timeout_seconds),
        perform_health_check(server),
    )
    .await;

    let response_time = start_time.elapsed();

    match health_result {
        Ok(Ok(())) => ServerStatus {
            name: server.name.to_string(),
            canister_id: server.canister_id.to_string(),
            network: server.network.to_string(),
            url: server.url.clone(),
            health: HealthStatus::Healthy,
            response_time: Some(response_time),
            error: None,
        },
        Ok(Err(error)) => {
            let error_string = error.to_string().to_lowercase();
            let health = if error_string.contains("connection")
                || error_string.contains("refused")
                || error_string.contains("unreachable")
                || error_string.contains("timeout")
            {
                HealthStatus::Unreachable
            } else {
                HealthStatus::Unhealthy
            };

            ServerStatus {
                name: server.name.to_string(),
                canister_id: server.canister_id.to_string(),
                network: server.network.to_string(),
                url: server.url.clone(),
                health,
                response_time: Some(response_time),
                error: Some(error.to_string()),
            }
        }
        Err(_) => ServerStatus {
            name: server.name.to_string(),
            canister_id: server.canister_id.to_string(),
            network: server.network.to_string(),
            url: server.url.clone(),
            health: HealthStatus::Timeout,
            response_time: None,
            error: Some(format!("Timeout after {}s", timeout_seconds)),
        },
    }
}

async fn perform_health_check(server: &crate::config::mcp::McpServerConfig) -> Result<()> {
    let client = reqwest::Client::new();

    // Try different health check endpoints
    let endpoints = vec![
        format!("{}/health", server.url),
        format!("{}/status", server.url),
        server.url.clone(), // Try the main endpoint
    ];

    for endpoint in endpoints {
        let response_result = client.get(&endpoint).send().await;
        match response_result {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(());
                }
            }
            Err(_) => continue,
        }
    }

    // If none of the endpoints work, try a simple canister call
    if server.network == "local" {
        check_local_canister_health(server.canister_id.as_str()).await
    } else {
        check_ic_canister_health(server.canister_id.as_str(), server.network.as_str()).await
    }
}

async fn check_local_canister_health(canister_id: &str) -> Result<()> {
    // Try to ping the local canister
    let candid_url = format!("http://127.0.0.1:4943/?canisterId={}", canister_id);

    let client = reqwest::Client::new();
    let response = client.get(&candid_url).send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow!("Canister returned status: {}", response.status()))
    }
}

async fn check_ic_canister_health(canister_id: &str, network: &str) -> Result<()> {
    let base_url = match network {
        "ic" => "https://ic0.app",
        "testnet" => "https://testnet.dfinity.network",
        _ => return Err(anyhow!("Unsupported network: {}", network)),
    };

    let candid_url = format!("{}/?canisterId={}", base_url, canister_id);

    let client = reqwest::Client::new();
    let response = client.get(&candid_url).send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow!("Canister returned status: {}", response.status()))
    }
}

fn print_status_table(statuses: &[ServerStatus]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    table.set_header(vec![
        "Name".bright_white().bold(),
        "Canister ID".bright_white().bold(),
        "Network".bright_white().bold(),
        "Status".bright_white().bold(),
        "Response Time".bright_white().bold(),
        "Error".bright_white().bold(),
    ]);

    for status in statuses {
        let response_time = status
            .response_time
            .map(|rt| format!("{}ms", rt.as_millis()))
            .unwrap_or_else(|| "-".to_string());

        let error = status
            .error
            .as_ref()
            .map(|e| {
                if e.len() > 50 {
                    format!("{}...", &e[..47])
                } else {
                    e.clone()
                }
            })
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            status.name.bright_cyan().to_string(),
            status.canister_id.bright_blue().to_string(),
            status.network.bright_yellow().to_string(),
            status.health.to_string(),
            response_time.bright_white().to_string(),
            error.bright_red().to_string(),
        ]);
    }

    println!("\n{}", "🔍 MCP Server Status".bright_white().bold());
    println!("{}", table);
}

fn print_status_summary(statuses: &[ServerStatus]) {
    let healthy = statuses
        .iter()
        .filter(|s| matches!(s.health, HealthStatus::Healthy))
        .count();
    let total = statuses.len();

    println!("\n{}", "📊 Summary".bright_white().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if healthy == total {
        println!(
            "{} {} out of {} servers are healthy",
            "✅".green(),
            healthy.to_string().bright_green(),
            total.to_string().bright_white()
        );
    } else {
        println!(
            "{} {} out of {} servers are healthy",
            "⚠️".yellow(),
            healthy.to_string().bright_yellow(),
            total.to_string().bright_white()
        );

        let unhealthy = total - healthy;
        println!(
            "{} {} servers need attention",
            "❌".red(),
            unhealthy.to_string().bright_red()
        );
    }

    // Calculate average response time for healthy servers
    let healthy_response_times: Vec<_> = statuses
        .iter()
        .filter(|s| matches!(s.health, HealthStatus::Healthy))
        .filter_map(|s| s.response_time)
        .collect();

    if !healthy_response_times.is_empty() {
        let avg_response_time = healthy_response_times
            .iter()
            .sum::<Duration>()
            .div_f64(healthy_response_times.len() as f64);

        println!(
            "{} Average response time: {}ms",
            "📈".bright_blue(),
            avg_response_time.as_millis().to_string().bright_cyan()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::mcp::McpServerConfig;
    use chrono::Utc;

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "✅ Healthy");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "⚠️  Unhealthy");
        assert_eq!(HealthStatus::Unreachable.to_string(), "❌ Unreachable");
        assert_eq!(HealthStatus::Timeout.to_string(), "⏰ Timeout");
    }

    #[tokio::test]
    async fn test_status_check_nonexistent_server() {
        let args = StatusArgs {
            identifier: Some("nonexistent-server".to_string()),
            all: false,
            timeout: 10,
        };

        let cli = crate::Cli {
            verbose: false,
            quiet: true,
            force: false,
            command: crate::Commands::Mcp(crate::commands::McpArgs::Status(args.clone())),
        };

        let result = execute(args, &cli).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No MCP server found"));
    }

    #[tokio::test]
    async fn test_server_status_creation() {
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

        // This should create a timeout status with 0 second timeout
        let status = check_server_health(&server, 0).await;

        assert_eq!(status.name, "test-server");
        assert_eq!(status.canister_id, "rdmx6-jaaaa-aaaaa-aaadq-cai");
        // Should timeout with 0 second timeout
        assert!(matches!(
            status.health,
            HealthStatus::Timeout | HealthStatus::Unreachable
        ));
    }
}
