use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod commands;
mod config;
mod interactive;
mod utils;

#[derive(Parser)]
#[command(
    name = "icarus",
    version,
    about = "Icarus CLI - Build and deploy MCP servers to the Internet Computer",
    long_about = "The Icarus CLI helps developers create, build, test, and deploy Model Context Protocol (MCP) servers as Internet Computer Protocol (ICP) canisters."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true, help = "Enable verbose output")]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "🧙 Interactive wizard for guided operations")]
    Wizard,

    #[command(about = "Create a new Icarus MCP server project")]
    New {
        #[arg(help = "Name of the project")]
        name: String,

        #[arg(short, long, help = "Directory to create the project in")]
        path: Option<String>,

        #[arg(long, help = "Use local SDK for development (path to icarus-sdk)")]
        local_sdk: Option<String>,

        #[arg(long, help = "Include test files and dependencies")]
        with_tests: bool,
    },

    #[command(about = "Analyze WASM binary size and optimization opportunities")]
    Analyze {
        #[arg(long, help = "Show top N size contributors", default_value = "20")]
        top: usize,

        #[arg(long, help = "Analyze compressed size if .wasm.gz exists")]
        compressed: bool,
    },

    #[command(about = "Deploy the MCP server to ICP")]
    Deploy {
        #[arg(short, long, default_value = "local", help = "Network to deploy to")]
        network: String,

        #[arg(long, help = "Force new deployment (deletes existing canister first)")]
        force: bool,

        #[arg(
            long,
            help = "Explicitly upgrade specific canister ID (default: auto-upgrade if exists)"
        )]
        upgrade: Option<String>,
    },

    #[command(about = "Manage the Icarus MCP server")]
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },

    #[command(about = "Validate WASM file for marketplace compatibility")]
    Validate {
        #[arg(long, help = "Path to WASM file to validate")]
        wasm_path: Option<String>,

        #[arg(
            long,
            help = "Network to use for test deployment",
            default_value = "local"
        )]
        network: Option<String>,

        #[arg(short, long, help = "Show detailed validation output")]
        verbose: bool,
    },

    #[command(about = "Performance profiling and benchmarking")]
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },

    #[command(about = "Local development mode with enhanced tooling")]
    Dev {
        #[command(subcommand)]
        command: DevCommands,
    },

    #[command(about = "Monitor canister health and performance")]
    Monitor {
        #[arg(long, help = "Canister ID to monitor")]
        canister_id: Option<String>,

        #[arg(long, default_value = "5", help = "Refresh interval in seconds")]
        interval: u64,

        #[arg(long, help = "Show detailed metrics")]
        detailed: bool,

        #[arg(long, default_value = "table", help = "Output format (table, json)")]
        format: String,

        #[arg(long, value_delimiter = ',', help = "Monitor specific metrics only")]
        metrics: Vec<String>,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    #[command(about = "Run performance benchmarks")]
    Bench {
        #[arg(long, help = "Filter benchmarks by name pattern")]
        filter: Option<String>,

        #[arg(long, help = "Save benchmark results to file")]
        output: Option<String>,

        #[arg(long, help = "Generate HTML report")]
        html: bool,
    },

    #[command(about = "Profile canister performance")]
    Canister {
        #[arg(help = "Canister ID to profile")]
        canister_id: String,

        #[arg(
            short,
            long,
            default_value = "30",
            help = "Duration to profile in seconds"
        )]
        duration: u64,

        #[arg(long, help = "Network to connect to", default_value = "local")]
        network: String,

        #[arg(long, help = "Number of concurrent requests", default_value = "10")]
        concurrency: usize,
    },

    #[command(about = "Analyze WASM binary performance characteristics")]
    Analyze {
        #[arg(long, help = "Path to WASM file to analyze")]
        wasm_path: Option<String>,

        #[arg(long, help = "Show memory usage analysis")]
        memory: bool,

        #[arg(long, help = "Show instruction count analysis")]
        instructions: bool,
    },
}

#[derive(Subcommand)]
enum DevCommands {
    #[command(about = "Start development server with auto-reload")]
    Start {
        #[arg(long, help = "Port for development server", default_value = "8080")]
        port: u16,

        #[arg(long, help = "Enable hot reload on file changes")]
        hot_reload: bool,

        #[arg(long, help = "Skip initial deployment")]
        skip_deploy: bool,

        #[arg(long, help = "Enable debug logging")]
        debug: bool,
    },

    #[command(about = "Initialize development environment")]
    Init {
        #[arg(long, help = "Skip dependency checks")]
        skip_checks: bool,

        #[arg(long, help = "Force reinitialize existing environment")]
        force: bool,
    },

    #[command(about = "Watch files and auto-redeploy on changes")]
    Watch {
        #[arg(long, help = "File patterns to watch (glob)", value_delimiter = ',')]
        patterns: Option<Vec<String>>,

        #[arg(
            long,
            help = "Delay before triggering redeploy (ms)",
            default_value = "500"
        )]
        delay: u64,

        #[arg(long, help = "Enable verbose file watching output")]
        verbose: bool,
    },

    #[command(about = "Show development environment status")]
    Status {
        #[arg(long, help = "Include detailed canister information")]
        detailed: bool,
    },

    #[command(about = "Reset development environment")]
    Reset {
        #[arg(long, help = "Remove all canisters")]
        clean: bool,

        #[arg(long, help = "Skip confirmation prompt")]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum McpCommands {
    #[command(about = "Add MCP server to AI client configurations")]
    Add {
        #[arg(help = "Canister ID to connect to")]
        canister_id: String,

        #[arg(long, help = "Name for the MCP server (defaults to canister ID)")]
        name: Option<String>,

        #[arg(
            long,
            help = "Specific AI clients to configure (claude, chatgpt, claude-code)",
            value_delimiter = ','
        )]
        clients: Option<Vec<String>>,

        #[arg(long, help = "Configure all available AI clients")]
        all: bool,

        #[arg(long, help = "Custom path to configuration file")]
        config_path: Option<String>,
    },

    #[command(about = "Show comprehensive MCP status dashboard")]
    Dashboard,

    #[command(about = "List configured MCP servers across AI clients")]
    List {
        #[arg(long, help = "Filter by specific client type")]
        client: Option<String>,
    },

    #[command(about = "Remove MCP server from AI client configurations")]
    Remove {
        #[arg(help = "Name of the MCP server to remove")]
        server_name: String,

        #[arg(
            long,
            help = "Specific AI clients to remove from (claude, chatgpt, claude-code)",
            value_delimiter = ','
        )]
        clients: Option<Vec<String>>,

        #[arg(long, help = "Remove from all available AI clients")]
        all: bool,

        #[arg(long, help = "Custom path to configuration file")]
        config_path: Option<String>,
    },

    #[command(about = "Start the Icarus MCP server")]
    Start {
        #[arg(help = "Canister ID to connect to")]
        canister_id: String,

        #[arg(long, help = "Run in background as daemon")]
        daemon: bool,
    },

    #[command(about = "Check the status of the Icarus MCP server")]
    Status {
        #[arg(short, long, help = "Show detailed status information")]
        verbose: bool,
    },

    #[command(about = "Stop the Icarus MCP server")]
    Stop {
        #[arg(long, help = "Stop all running MCP server instances")]
        all: bool,

        #[arg(long, help = "Stop MCP server for specific canister")]
        canister_id: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check if we're in MCP mode (being run by Claude Desktop)
    let is_mcp_mode = match &cli.command {
        Commands::Mcp { command } => {
            matches!(command, McpCommands::Start { .. })
                && !is_terminal::is_terminal(std::io::stdin())
                && !is_terminal::is_terminal(std::io::stdout())
        }
        _ => false,
    };

    // Skip tracing for MCP mode to avoid hanging issues
    let skip_tracing = is_mcp_mode;

    // Only initialize logging if not skipping tracing
    if !skip_tracing {
        // Initialize logging
        let filter = if cli.verbose {
            EnvFilter::new("icarus=debug,info")
        } else {
            EnvFilter::new("icarus=info,warn")
        };

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .init();
    }

    // In MCP mode, disable all colored output
    if is_mcp_mode {
        colored::control::set_override(false);
    }

    // ASCII art banner - only show for new command and not in MCP mode
    let show_banner = matches!(cli.command, Commands::New { .. }) && !cli.verbose && !is_mcp_mode;

    if show_banner {
        println!(
            "{}",
            r#"
  ___                              
 |_ _|___ __ _ _ __ _   _ ___     
  | |/ __/ _` | '__| | | / __|    
  | | (_| (_| | |  | |_| \__ \    
 |___\___\__,_|_|   \__,_|___/    
                                   
"#
            .cyan()
            .bold()
        );
    }

    match cli.command {
        Commands::Wizard => {
            info!("Starting interactive wizard");
            let wizard = interactive::InteractiveWizard::new();
            wizard.run().await?;
        }
        Commands::New {
            name,
            path,
            local_sdk,
            with_tests,
        } => {
            info!("Creating new project: {}", name);
            commands::new::execute(name, path, local_sdk, with_tests).await?;
        }
        Commands::Analyze { top, compressed } => {
            info!("Analyzing WASM binary");
            commands::analyze::execute(top, compressed).await?;
        }
        Commands::Deploy {
            network,
            force,
            upgrade,
        } => {
            info!("Deploying to {}", network);
            commands::deploy::execute(network, force, upgrade).await?;
        }
        Commands::Mcp { command } => match command {
            McpCommands::Add {
                canister_id,
                name,
                clients,
                all,
                config_path,
            } => {
                info!("Adding MCP server to AI clients");
                commands::mcp::add::execute(canister_id, name, clients, all, config_path).await?;
            }
            McpCommands::Dashboard => {
                info!("Showing MCP status dashboard");
                commands::mcp::dashboard::execute().await?;
            }
            McpCommands::List { client } => {
                info!("Listing configured MCP servers");
                commands::mcp::list::execute(client).await?;
            }
            McpCommands::Remove {
                server_name,
                clients,
                all,
                config_path,
            } => {
                info!("Removing MCP server from AI clients");
                commands::mcp::remove::execute(server_name, clients, all, config_path).await?;
            }
            McpCommands::Start {
                canister_id,
                daemon,
            } => {
                if !is_mcp_mode {
                    info!("Starting MCP server");
                }
                commands::mcp::start::execute(canister_id, daemon).await?;
            }
            McpCommands::Status { verbose } => {
                info!("Checking MCP server status");
                commands::mcp::status::execute(verbose).await?;
            }
            McpCommands::Stop { all, canister_id } => {
                info!("Stopping MCP server");
                commands::mcp::stop::execute(all, canister_id).await?;
            }
        },
        Commands::Validate {
            wasm_path,
            network,
            verbose,
        } => {
            info!("Validating WASM for marketplace");
            commands::validate::execute(wasm_path, network, verbose).await?;
        }
        Commands::Profile { command } => match command {
            ProfileCommands::Bench {
                filter,
                output,
                html,
            } => {
                info!("Running performance benchmarks");
                commands::profile::bench::execute(filter, output, html).await?;
            }
            ProfileCommands::Canister {
                canister_id,
                duration,
                network,
                concurrency,
            } => {
                info!("Profiling canister performance");
                commands::profile::canister::execute(canister_id, duration, network, concurrency)
                    .await?;
            }
            ProfileCommands::Analyze {
                wasm_path,
                memory,
                instructions,
            } => {
                info!("Analyzing WASM performance");
                commands::profile::analyze::execute(wasm_path, memory, instructions).await?;
            }
        },
        Commands::Dev { command } => match command {
            DevCommands::Start {
                port,
                hot_reload,
                skip_deploy,
                debug,
            } => {
                info!("Starting development server");
                icarus_dev::start::execute(port, hot_reload, skip_deploy, debug).await?;
            }
            DevCommands::Init { skip_checks, force } => {
                info!("Initializing development environment");
                icarus_dev::init::execute(skip_checks, force).await?;
            }
            DevCommands::Watch {
                patterns,
                delay,
                verbose,
            } => {
                info!("Starting file watcher");
                icarus_dev::watch::execute(patterns.clone(), delay, verbose).await?;
            }
            DevCommands::Status { detailed } => {
                info!("Showing development status");
                icarus_dev::status::execute(detailed).await?;
            }
            DevCommands::Reset { clean, yes } => {
                info!("Resetting development environment");
                icarus_dev::reset::execute(clean, yes).await?;
            }
        },

        Commands::Monitor {
            canister_id,
            interval,
            detailed,
            format,
            metrics,
        } => {
            info!("Starting monitoring dashboard");
            let monitor_cmd = commands::monitor::MonitorCommand {
                canister_id,
                interval,
                detailed,
                format,
                metrics,
            };
            monitor_cmd.run().await?;
        }
    }

    Ok(())
}
