use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod bridge;
mod commands;
mod config;
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

    #[command(about = "Build the MCP server to WASM")]
    Build {
        #[arg(long, help = "Build profile: size, speed, or debug", value_parser = ["size", "speed", "debug"])]
        profile: Option<String>,

        #[arg(long, help = "Skip optimization step")]
        no_optimize: bool,

        #[arg(long, help = "Optimize for smallest size (uses wasm-opt -Oz)")]
        optimize_size: bool,

        #[arg(long, help = "Optimize for best performance (uses wasm-opt -O4)")]
        optimize_performance: bool,

        #[arg(
            long,
            help = "Skip gzip compression (compression is enabled by default)"
        )]
        no_compress: bool,

        #[arg(long, help = "Output directory for build artifacts")]
        output: Option<String>,
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

        #[arg(long, help = "Build profile: size, speed, or debug", value_parser = ["size", "speed", "debug"])]
        profile: Option<String>,
    },

    #[command(about = "Manage the Icarus bridge")]
    Bridge {
        #[command(subcommand)]
        command: BridgeCommands,
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

    #[command(about = "Test HTTP outcalls functionality")]
    TestHttp {
        #[arg(long, help = "Canister ID to test")]
        canister_id: String,

        #[arg(long, default_value = "local", help = "Network to use")]
        network: String,

        #[arg(
            long,
            default_value = "https://api.github.com/meta",
            help = "URL to fetch"
        )]
        url: String,

        #[arg(
            long,
            default_value = "get",
            help = "Test type: get, post, json, weather, crypto"
        )]
        test_type: String,
    },
}

#[derive(Subcommand)]
enum BridgeCommands {
    #[command(about = "Add a canister to Claude Desktop configuration")]
    Add {
        #[arg(help = "Canister ID to add")]
        canister_id: String,

        #[arg(long, help = "Name for the canister (defaults to canister ID)")]
        name: Option<String>,

        #[arg(long, help = "Description for the canister")]
        description: Option<String>,
    },

    #[command(about = "List configured canisters")]
    List {
        #[arg(long, help = "Show detailed information")]
        verbose: bool,
    },

    #[command(about = "Start the Icarus bridge")]
    Start {
        #[arg(long, help = "Canister ID to connect to")]
        canister_id: String,

        #[arg(
            short,
            long,
            default_value = "9090",
            help = "Port to run the bridge on"
        )]
        port: u16,

        #[arg(long, help = "Run in background as daemon")]
        daemon: bool,
    },

    #[command(about = "Check the status of the Icarus bridge")]
    Status {
        #[arg(short, long, help = "Show detailed status information")]
        verbose: bool,
    },

    #[command(about = "Stop the Icarus bridge")]
    Stop {
        #[arg(long, help = "Stop all running bridge instances")]
        all: bool,

        #[arg(short, long, help = "Port of the bridge to stop")]
        port: Option<u16>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check if we're in MCP mode (being run by Claude Desktop) or running auth commands
    let is_mcp_mode = if let Commands::Bridge { command } = &cli.command {
        matches!(command, BridgeCommands::Start { .. })
            && !is_terminal::is_terminal(std::io::stdin())
            && !is_terminal::is_terminal(std::io::stdout())
    } else {
        false
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
        Commands::New {
            name,
            path,
            local_sdk,
            with_tests,
        } => {
            info!("Creating new project: {}", name);
            commands::new::execute(name, path, local_sdk, with_tests).await?;
        }
        Commands::Build {
            profile,
            no_optimize,
            optimize_size,
            optimize_performance,
            no_compress,
            output,
        } => {
            info!("Building project");

            // Apply profile settings
            let (opt_skip, opt_size, opt_perf, compress) = match profile.as_deref() {
                Some("size") => (false, true, false, true), // Maximum size optimization
                Some("speed") => (false, false, true, false), // Maximum performance, no compression
                Some("debug") => (true, false, false, false), // Fast builds, no optimization
                _ => (
                    no_optimize,
                    optimize_size,
                    optimize_performance,
                    !no_compress,
                ), // Use individual flags
            };

            if let Err(e) =
                commands::build::execute(opt_skip, opt_size, opt_perf, compress, output).await
            {
                eprintln!("Build error: {:?}", e);
                return Err(e);
            }
        }
        Commands::Analyze { top, compressed } => {
            info!("Analyzing WASM binary");
            commands::analyze::execute(top, compressed).await?;
        }
        Commands::Deploy {
            network,
            force,
            upgrade,
            profile,
        } => {
            info!("Deploying to {}", network);
            commands::deploy::execute(network, force, upgrade, profile).await?;
        }
        Commands::Bridge { command } => match command {
            BridgeCommands::Add {
                canister_id,
                name,
                description,
            } => {
                info!("Adding canister to Claude Desktop config");
                commands::bridge::add::execute(canister_id, name, description).await?;
            }
            BridgeCommands::List { verbose } => {
                info!("Listing configured canisters");
                commands::bridge::list::execute(verbose).await?;
            }
            BridgeCommands::Start {
                canister_id,
                port,
                daemon,
            } => {
                if !is_mcp_mode {
                    info!("Starting bridge");
                }
                // Always authenticate (using local for development)
                commands::bridge::start::execute(canister_id, port, daemon).await?;
            }
            BridgeCommands::Status { verbose } => {
                info!("Checking bridge status");
                commands::bridge::status::execute(verbose).await?;
            }
            BridgeCommands::Stop { all, port } => {
                info!("Stopping bridge");
                commands::bridge::stop::execute(all, port).await?;
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
        Commands::TestHttp {
            canister_id,
            network,
            url,
            test_type,
        } => {
            info!("Testing HTTP outcalls");
            let cmd = commands::test_http::TestHttpCmd {
                canister_id,
                network,
                url,
                test_type,
            };
            cmd.execute().await?;
        }
    }

    Ok(())
}
