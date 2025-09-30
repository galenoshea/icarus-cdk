use clap::{Args, Subcommand};

pub(crate) mod add;
pub(crate) mod list;
pub(crate) mod remove;
pub(crate) mod start;
pub(crate) mod status;
pub(crate) mod stop;

use crate::Cli;
use anyhow::Result;

/// MCP server management commands
#[derive(Subcommand)]
pub(crate) enum McpCommand {
    /// Register MCP server with AI clients
    Add(AddArgs),
    /// List registered MCP servers
    List(ListArgs),
    /// Remove MCP server registration
    Remove(RemoveArgs),
    /// Check MCP server status
    Status(StatusArgs),
    /// Start MCP bridge server
    Start(StartArgs),
    /// Stop MCP bridge server
    Stop(StopArgs),
}

/// Arguments for the `mcp add` command
#[derive(Args, Clone)]
pub struct AddArgs {
    /// Canister ID to register
    pub canister_id: String,

    /// AI client to register with
    #[arg(long, value_enum)]
    pub client: McpClient,

    /// Custom client name (overrides detected client)
    #[arg(long)]
    pub client_name: Option<String>,

    /// MCP server port (auto-detected if not specified)
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Network the canister is deployed on
    #[arg(short, long, default_value = "local")]
    pub network: String,

    /// Custom server name
    #[arg(long)]
    pub name: Option<String>,

    /// Skip verification of canister accessibility
    #[arg(long)]
    pub skip_verify: bool,
}

/// Arguments for the `mcp list` command
#[derive(Args, Clone)]
pub struct ListArgs {
    /// Filter by AI client
    #[arg(long, value_enum)]
    pub client: Option<McpClient>,

    /// Show detailed information
    #[arg(short, long)]
    pub detailed: bool,

    /// Show only active servers
    #[arg(long)]
    pub active: bool,

    /// Output format
    #[arg(long, value_enum, default_value = "table")]
    pub format: OutputFormat,
}

/// Arguments for the `mcp remove` command
#[derive(Args, Clone)]
pub struct RemoveArgs {
    /// Canister ID or server name to remove
    pub identifier: String,

    /// AI client to remove from (if not specified, removes from all)
    #[arg(long, value_enum)]
    pub client: Option<McpClient>,

    /// Remove without confirmation
    #[arg(short, long)]
    pub yes: bool,
}

/// Arguments for the `mcp status` command
#[derive(Args, Clone)]
pub struct StatusArgs {
    /// Canister ID or server name to check
    pub identifier: Option<String>,

    /// Check health of all registered servers
    #[arg(long)]
    pub all: bool,

    /// Timeout for health checks in seconds
    #[arg(long, default_value = "10")]
    pub timeout: u64,
}

/// Arguments for the `mcp start` command
#[derive(Args, Clone)]
pub struct StartArgs {
    /// Port to run the MCP bridge server on
    #[arg(short, long, default_value = "3000")]
    pub port: u16,

    /// Host to bind to
    #[arg(long, default_value = "localhost")]
    pub host: String,

    /// Run in background/daemon mode
    #[arg(short, long)]
    pub daemon: bool,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<std::path::PathBuf>,
}

/// Arguments for the `mcp stop` command
#[derive(Args, Clone)]
pub struct StopArgs {
    /// Force stop without graceful shutdown
    #[arg(short, long)]
    pub force: bool,

    /// Stop all MCP processes
    #[arg(long)]
    pub all: bool,
}

/// Supported AI clients
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum McpClient {
    /// Claude Desktop application
    ClaudeDesktop,
    /// Claude Code VS Code extension
    ClaudeCode,
    /// ChatGPT Desktop application
    ChatgptDesktop,
    /// Continue VS Code extension
    Continue,
    /// Custom client configuration
    Custom,
}

impl std::fmt::Display for McpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpClient::ClaudeDesktop => write!(f, "claude-desktop"),
            McpClient::ClaudeCode => write!(f, "claude-code"),
            McpClient::ChatgptDesktop => write!(f, "chatgpt-desktop"),
            McpClient::Continue => write!(f, "continue"),
            McpClient::Custom => write!(f, "custom"),
        }
    }
}

/// Output formats for list command
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table
    Table,
    /// JSON output
    Json,
    /// YAML output
    Yaml,
    /// Plain text (one per line)
    Plain,
}

pub(crate) async fn execute(mcp_args: crate::commands::McpArgs, cli: &Cli) -> Result<()> {
    match mcp_args {
        crate::commands::McpArgs::Add(args) => add::execute(args, cli).await,
        crate::commands::McpArgs::List(args) => list::execute(args, cli).await,
        crate::commands::McpArgs::Remove(args) => remove::execute(args, cli).await,
        crate::commands::McpArgs::Status(args) => status::execute(args, cli).await,
        crate::commands::McpArgs::Start(args) => start::execute(args, cli).await,
        crate::commands::McpArgs::Stop(args) => stop::execute(args, cli).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_client_display() {
        assert_eq!(McpClient::ClaudeDesktop.to_string(), "claude-desktop");
        assert_eq!(McpClient::ClaudeCode.to_string(), "claude-code");
        assert_eq!(McpClient::ChatgptDesktop.to_string(), "chatgpt-desktop");
        assert_eq!(McpClient::Continue.to_string(), "continue");
        assert_eq!(McpClient::Custom.to_string(), "custom");
    }
}
