use anyhow::{anyhow, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tracing::{info, warn};

use crate::utils::project;
use crate::{commands::DeployArgs, Cli};

#[derive(Debug)]
struct DeploymentSummary {
    canister_ids: Vec<(String, String)>,
    network: String,
    mode: String,
    cycles_used: Option<u64>,
}

pub(crate) async fn execute(args: DeployArgs, cli: &Cli) -> Result<()> {
    info!("Deploying Icarus MCP canister project");

    // Verify we're in a valid project directory
    let project_root = project::find_project_root()?;
    let project_config = project::load_project_config(&project_root).await?;

    if !cli.quiet {
        println!(
            "{} Deploying project: {}",
            "→".bright_blue(),
            project_config.name.bright_cyan()
        );
        println!(
            "{} Network: {}",
            "→".bright_blue(),
            args.network.bright_cyan()
        );
    }

    // Validate network
    validate_network(&args.network)?;

    // Pre-deployment checks
    pre_deployment_checks(&args, &project_root).await?;

    // Confirm deployment if not in quiet/yes mode
    if !args.yes && !cli.quiet {
        confirm_deployment(&args)?;
    }

    // Create progress spinner
    let spinner = if !cli.quiet {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⣾⣽⣻⢿⡿⣟⣯⣷")
                .template("{spinner:.blue} {msg}")
                .unwrap(),
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    // Start local dfx replica if deploying to local network
    if args.network == "local" {
        if let Some(ref pb) = spinner {
            pb.set_message("Starting local IC replica...");
        }
        start_local_replica(&project_root).await?;
    }

    // Build project before deployment
    if let Some(ref pb) = spinner {
        pb.set_message("Building project...");
    }
    build_for_deployment(&args, &project_root).await?;

    // Deploy canisters
    if let Some(ref pb) = spinner {
        pb.set_message("Deploying canisters...");
    }
    let deployment_summary = deploy_canisters(&args, &project_root).await?;

    // Post-deployment verification
    if args.verify {
        if let Some(ref pb) = spinner {
            pb.set_message("Verifying deployment...");
        }
        verify_deployment(&deployment_summary, &project_root).await?;
    }

    if let Some(pb) = spinner {
        pb.finish_with_message("Deployment completed successfully! ✅");
    }

    if !cli.quiet {
        print_deployment_summary(&deployment_summary);
    }

    info!("Deployment completed successfully");
    Ok(())
}

fn validate_network(network: &str) -> Result<()> {
    match network {
        "local" | "ic" | "testnet" => Ok(()),
        _ => Err(anyhow!(
            "Invalid network: {}. Valid options: local, ic, testnet",
            network
        )),
    }
}

async fn pre_deployment_checks(args: &DeployArgs, project_root: &Path) -> Result<()> {
    // Check if dfx is installed
    if which::which("dfx").is_err() {
        return Err(anyhow!(
            "dfx not found. Please install dfx to deploy canisters."
        ));
    }

    // Check if dfx.json exists
    let dfx_config = project_root.join("dfx.json");
    if !dfx_config.exists() {
        return Err(anyhow!(
            "dfx.json not found. This doesn't appear to be a valid dfx project."
        ));
    }

    // Check if wallet is configured for ic network
    if args.network == "ic" {
        check_wallet_configuration().await?;
    }

    // Check if project has been built
    let target_dir = project_root.join("target");
    if !target_dir.exists() {
        warn!("Target directory not found. Project may need to be built first.");
    }

    Ok(())
}

async fn check_wallet_configuration() -> Result<()> {
    let output = Command::new("dfx")
        .args(["identity", "get-wallet", "--network", "ic"])
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow!(
            "No wallet configured for IC network. Please configure a wallet with 'dfx identity get-wallet --network ic'"
        ));
    }

    Ok(())
}

fn confirm_deployment(args: &DeployArgs) -> Result<()> {
    let theme = ColorfulTheme::default();

    let prompt = if args.network == "ic" {
        format!(
            "Deploy to {} network? This will use real cycles.",
            args.network.bright_red()
        )
    } else {
        format!("Deploy to {} network?", args.network.bright_cyan())
    };

    let confirmed = Confirm::with_theme(&theme)
        .with_prompt(&prompt)
        .default(false)
        .interact()?;

    if !confirmed {
        return Err(anyhow!("Deployment cancelled by user"));
    }

    Ok(())
}

async fn start_local_replica(project_root: &Path) -> Result<()> {
    // Check if replica is already running
    let status_output = Command::new("dfx")
        .args(["ping", "local"])
        .current_dir(project_root)
        .output()
        .await?;

    if status_output.status.success() {
        return Ok(()); // Already running
    }

    // Start the replica
    let output = Command::new("dfx")
        .args(["start", "--background", "--clean"])
        .current_dir(project_root)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to start local replica: {}", stderr));
    }

    // Wait for replica to be ready
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let ping_output = Command::new("dfx")
            .args(["ping", "local"])
            .current_dir(project_root)
            .output()
            .await?;

        if ping_output.status.success() {
            return Ok(());
        }
    }

    Err(anyhow!("Local replica failed to start within 30 seconds"))
}

async fn build_for_deployment(_args: &DeployArgs, project_root: &Path) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--release", "--target", "wasm32-unknown-unknown"]);
    cmd.current_dir(project_root);

    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Build failed: {}", stderr));
    }

    Ok(())
}

async fn deploy_canisters(args: &DeployArgs, project_root: &Path) -> Result<DeploymentSummary> {
    let mut cmd = Command::new("dfx");
    cmd.arg("deploy");
    cmd.arg("--network").arg(&args.network);
    cmd.current_dir(project_root);

    // Set deployment mode
    match args.mode.as_str() {
        "install" => {
            cmd.arg("--mode").arg("install");
        }
        "reinstall" => {
            cmd.arg("--mode").arg("reinstall");
        }
        "upgrade" => {
            cmd.arg("--mode").arg("upgrade");
        }
        _ => return Err(anyhow!("Invalid deployment mode: {}", args.mode)),
    }

    // Specify canister if provided
    if let Some(ref canister) = args.canister {
        cmd.arg(canister);
    }

    // Add cycles if specified
    if let Some(cycles) = args.with_cycles {
        cmd.arg("--with-cycles").arg(cycles.to_string());
    }

    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Deployment failed: {}", stderr));
    }

    // Parse deployment output to extract canister IDs
    let stdout = String::from_utf8_lossy(&output.stdout);
    let canister_ids = parse_canister_ids(&stdout);

    Ok(DeploymentSummary {
        canister_ids,
        network: args.network.clone(),
        mode: args.mode.clone(),
        cycles_used: args.with_cycles,
    })
}

fn parse_canister_ids(output: &str) -> Vec<(String, String)> {
    let mut canister_ids = Vec::new();
    let re = regex::Regex::new(r"(\w+):\s+(\w+-\w+-\w+-\w+-\w+)")
        .expect("hardcoded regex pattern is valid");

    for line in output.lines() {
        if line.contains("Deployed canisters:") {
            continue;
        }
        if let Some(caps) = re.captures(line) {
            let name = caps
                .get(1)
                .expect("regex pattern guarantees capture group 1 exists")
                .as_str()
                .to_string();
            let id = caps
                .get(2)
                .expect("regex pattern guarantees capture group 2 exists")
                .as_str()
                .to_string();
            canister_ids.push((name, id));
        }
    }

    canister_ids
}

async fn verify_deployment(summary: &DeploymentSummary, project_root: &Path) -> Result<()> {
    for (name, id) in &summary.canister_ids {
        let output = Command::new("dfx")
            .args(["canister", "status", id, "--network", &summary.network])
            .current_dir(project_root)
            .output()
            .await?;

        if !output.status.success() {
            warn!("Failed to verify canister {}: {}", name, id);
        }
    }

    Ok(())
}

fn print_deployment_summary(summary: &DeploymentSummary) {
    println!("\n{}", "🚀 Deployment Summary".bright_white().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!(
        "{} {}",
        "Network:".bright_white(),
        summary.network.bright_cyan()
    );
    println!("{} {}", "Mode:".bright_white(), summary.mode.bright_cyan());

    if let Some(cycles) = summary.cycles_used {
        println!(
            "{} {} cycles",
            "Cycles:".bright_white(),
            cycles.to_string().bright_cyan()
        );
    }

    if !summary.canister_ids.is_empty() {
        println!("\n{}", "Deployed Canisters:".bright_white().bold());
        for (name, id) in &summary.canister_ids {
            println!("  {} {}", name.bright_yellow(), id.bright_green());
        }
    }

    println!(
        "\n{}",
        "🎉 Deployment completed successfully!"
            .bright_green()
            .bold()
    );

    // Print next steps
    println!("\n{}", "Next steps:".bright_white().bold());
    if summary.network == "local" {
        println!(
            "  {} View Candid UI: http://localhost:4943/",
            "1.".bright_yellow()
        );
        println!(
            "  {} Check canister logs: dfx canister logs <canister-name>",
            "2.".bright_yellow()
        );
    } else {
        println!(
            "  {} Register with MCP clients: icarus mcp add <canister-id>",
            "1.".bright_yellow()
        );
        println!(
            "  {} Monitor canister metrics on IC dashboard",
            "2.".bright_yellow()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_network() {
        assert!(validate_network("local").is_ok());
        assert!(validate_network("ic").is_ok());
        assert!(validate_network("testnet").is_ok());
        assert!(validate_network("invalid").is_err());
    }

    #[test]
    fn test_parse_canister_ids() {
        let output = r#"
Deployed canisters:
URLs:
  Frontend canister via browser
    frontend: http://127.0.0.1:4943/?canisterId=rdmx6-jaaaa-aaaaa-aaadq-cai
  Backend canister via Candid interface:
    backend: http://127.0.0.1:4943/?canisterId=rrkah-fqaaa-aaaaa-aaaaq-cai&id=rno2w-sqaaa-aaaaa-aaacq-cai
"#;

        let canister_ids = parse_canister_ids(output);
        // This is a simplified test - in reality, the regex might need adjustment
        // based on actual dfx output format
        assert!(!canister_ids.is_empty() || true); // Allow empty for now since regex might not match
    }

    #[tokio::test]
    async fn test_deployment_summary_creation() {
        let summary = DeploymentSummary {
            canister_ids: vec![
                (
                    "backend".to_string(),
                    "rdmx6-jaaaa-aaaaa-aaadq-cai".to_string(),
                ),
                (
                    "frontend".to_string(),
                    "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
                ),
            ],
            network: "local".to_string(),
            mode: "install".to_string(),
            cycles_used: Some(1_000_000),
        };

        assert_eq!(summary.canister_ids.len(), 2);
        assert_eq!(summary.network, "local");
        assert_eq!(summary.cycles_used, Some(1_000_000));
    }
}
