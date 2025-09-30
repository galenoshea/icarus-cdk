use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod commands;
mod config;
mod templates;
mod types;
mod utils;

use commands::{BuildArgs, DeployArgs, McpArgs, NewArgs};

/// Icarus CLI - MCP canister framework for Internet Computer
#[derive(Parser)]
#[command(
    name = "icarus",
    version,
    about = "MCP canister framework for Internet Computer",
    long_about = "A powerful command-line interface for creating, building, and deploying MCP (Model Context Protocol) canisters on the Internet Computer network."
)]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Force operations without confirmation prompts
    #[arg(short, long, global = true)]
    pub force: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new MCP canister project
    New(NewArgs),

    /// Build the current project
    Build(BuildArgs),

    /// Deploy the canister to Internet Computer
    Deploy(DeployArgs),

    /// MCP server management commands
    #[command(subcommand)]
    Mcp(McpArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(&cli)?;

    // Display banner if not in quiet mode
    if !cli.quiet {
        display_banner();
    }

    // Execute the command
    match cli.command {
        Commands::New(ref args) => commands::new::execute(args.clone(), &cli).await,
        Commands::Build(ref args) => commands::build::execute(args.clone(), &cli).await,
        Commands::Deploy(ref args) => commands::deploy::execute(args.clone(), &cli).await,
        Commands::Mcp(ref mcp_args) => commands::mcp::execute(mcp_args.clone(), &cli).await,
    }
}

fn init_logging(cli: &Cli) -> Result<()> {
    let level = if cli.verbose {
        Level::DEBUG
    } else if cli.quiet {
        Level::ERROR
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

fn display_banner() {
    println!(
        "{}",
        "
██╗ ██████╗ █████╗ ██████╗ ██╗   ██╗███████╗
██║██╔════╝██╔══██╗██╔══██╗██║   ██║██╔════╝
██║██║     ███████║██████╔╝██║   ██║███████╗
██║██║     ██╔══██║██╔══██╗██║   ██║╚════██║
██║╚██████╗██║  ██║██║  ██║╚██████╔╝███████║
╚═╝ ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝
"
        .bright_red()
    );

    println!(
        "{} {}",
        "Icarus CLI".bright_white().bold(),
        "- MCP Canister Framework".bright_blue()
    );
    println!(
        "{}\n",
        "Building the future of AI-Internet Computer integration"
            .italic()
            .bright_black()
    );

    info!("Icarus CLI initialized successfully");
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert()
    }

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::try_parse_from(&["icarus", "--help"]);
        assert!(cli.is_err()); // Help should exit with error code
    }
}
