use clap::{Args, Subcommand};

pub(crate) mod build;
pub(crate) mod deploy;
pub(crate) mod mcp;
pub(crate) mod new;

/// Arguments for the `new` command
#[derive(Args, Clone)]
pub struct NewArgs {
    /// Name of the project to create
    pub name: String,

    /// Directory to create the project in (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<std::path::PathBuf>,

    /// Skip git repository initialization
    #[arg(long)]
    pub no_git: bool,

    /// Skip dependency installation
    #[arg(long)]
    pub no_install: bool,
}

/// Arguments for the `build` command
#[derive(Args, Clone)]
pub struct BuildArgs {
    /// Build target (wasm32-unknown-unknown, x86_64-unknown-linux-gnu)
    #[arg(short, long)]
    pub target: Option<String>,

    /// Build mode (debug, release)
    #[arg(short, long, default_value = "release")]
    pub mode: String,

    /// Features to enable
    #[arg(long)]
    pub features: Vec<String>,

    /// Run tests after build
    #[arg(long)]
    pub test: bool,

    /// Generate canister declarations
    #[arg(long, default_value = "true")]
    pub generate_declarations: bool,

    /// Output directory for build artifacts
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}

/// Arguments for the `deploy` command
#[derive(Args, Clone)]
pub struct DeployArgs {
    /// Network to deploy to (local, ic, testnet)
    #[arg(short, long, default_value = "local")]
    pub network: String,

    /// Canister name to deploy (if not specified, deploys all)
    #[arg(short, long)]
    pub canister: Option<String>,

    /// Deploy with arguments
    #[arg(long)]
    pub with_cycles: Option<u64>,

    /// Skip confirmation prompts
    #[arg(short, long)]
    pub yes: bool,

    /// Upgrade mode (install, reinstall, upgrade)
    #[arg(long, default_value = "upgrade")]
    pub mode: String,

    /// Post-deployment verification
    #[arg(long, default_value = "true")]
    pub verify: bool,
}

/// MCP server management commands
#[derive(Subcommand, Clone)]
pub enum McpArgs {
    /// Register MCP server with AI clients
    Add(mcp::AddArgs),

    /// List registered MCP servers
    List(mcp::ListArgs),

    /// Remove MCP server registration
    Remove(mcp::RemoveArgs),

    /// Check MCP server status
    Status(mcp::StatusArgs),

    /// Start MCP bridge server
    Start(mcp::StartArgs),

    /// Stop MCP bridge server
    Stop(mcp::StopArgs),
}
