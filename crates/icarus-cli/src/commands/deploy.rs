use anyhow::Result;
use colored::Colorize;
use serde_json::json;
use std::path::Path;

use crate::config::cargo_config;
use crate::utils::{
    build_utils::get_project_name, claude_desktop::generate_claude_server_config, print_info,
    print_success, print_warning, run_command, run_command_streaming,
};

pub async fn execute(
    network: String,
    force: bool,
    _upgrade: Option<String>,
    _enable_simd: bool,
    _skip_build: bool,
) -> Result<()> {
    // Validate network
    if !["local", "ic"].contains(&network.as_str()) {
        anyhow::bail!("Invalid network. Use 'local' or 'ic'");
    }

    // Check if we're in an Icarus project
    let current_dir = std::env::current_dir()?;
    if !is_icarus_project(&current_dir) {
        anyhow::bail!("Not in an Icarus project directory. Run this command from a project created with 'icarus new'.");
    }

    // Get project name
    let project_name = get_project_name(&current_dir)?;

    print_info(&format!(
        "Deploying {} to {} network...",
        project_name, network
    ));

    // Get the current principal to use as the init argument
    let principal_output =
        run_command("dfx", &["identity", "get-principal"], Some(&current_dir)).await?;
    let principal = principal_output.trim().to_string();
    print_info(&format!("Using principal: {}", principal));

    // Build args with owned strings for the init argument
    let init_arg = format!("(principal \"{}\")", principal);

    // Deploy via dfx (leverages dfx's battle-tested deployment logic)
    deploy_with_dfx(&current_dir, &project_name, &network, &init_arg, force).await?;

    // Get the canister ID after deployment
    let canister_id_output = run_command(
        "dfx",
        &["canister", "id", &project_name, "--network", &network],
        Some(&current_dir),
    )
    .await?;
    let canister_id = canister_id_output.trim().to_string();

    print_success(&format!(
        "Successfully deployed! Canister ID: {}",
        canister_id
    ));

    // Save canister ID for future reference
    save_canister_id(&current_dir, &network, &canister_id)?;

    // Handle MCP client configuration using existing MCP command infrastructure
    handle_mcp_client_config(&current_dir, &project_name, &canister_id).await?;

    if network == "ic" {
        println!();
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
        );
        println!("{}", "📦 Marketplace Publishing".bold().cyan());
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
        );
        println!();
        println!("Ready to share your MCP server? Publish it:");
        println!("  {}", "icarus publish --network ic".bright_yellow());
    }

    println!();

    Ok(())
}

fn is_icarus_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists() && path.join("dfx.json").exists()
}

fn save_canister_id(project_dir: &Path, network: &str, canister_id: &str) -> Result<()> {
    let canister_ids_path = project_dir.join("canister_ids.json");

    let mut ids: serde_json::Value = if canister_ids_path.exists() {
        let content = std::fs::read_to_string(&canister_ids_path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    let project_name = get_project_name(project_dir)?;

    if !ids.is_object() {
        ids = serde_json::json!({});
    }

    let obj = ids.as_object_mut().unwrap();

    if !obj.contains_key(&project_name) {
        obj.insert(project_name.clone(), serde_json::json!({}));
    }

    obj[&project_name][network] = serde_json::json!(canister_id);

    let content = serde_json::to_string_pretty(&ids)?;
    std::fs::write(&canister_ids_path, content)?;

    Ok(())
}

async fn handle_mcp_client_config(
    project_dir: &Path,
    project_name: &str,
    canister_id: &str,
) -> Result<()> {
    // Check for Cargo.toml metadata and handle MCP client auto-configuration
    if let Some(icarus_metadata) = cargo_config::load_from_cargo_toml(project_dir)? {
        let mut clients_to_update = Vec::new();

        // Check which clients have auto_update enabled
        if icarus_metadata.claude_desktop.auto_update {
            clients_to_update.push("claude".to_string());
        }
        if icarus_metadata.chatgpt_desktop.auto_update {
            clients_to_update.push("chatgpt".to_string());
        }
        if icarus_metadata.claude_code.auto_update {
            clients_to_update.push("claude-code".to_string());
        }

        if !clients_to_update.is_empty() {
            println!();
            print_info(&format!(
                "Auto-updating MCP client configurations: {}",
                clients_to_update.join(", ")
            ));

            // Use the existing MCP add command for all enabled clients
            match crate::commands::mcp::add::execute(
                canister_id.to_string(),
                Some(project_name.to_string()),
                Some(clients_to_update),
                false, // not all clients
                None,  // no custom config path
                true,  // skip confirmation for auto-deployment
            )
            .await
            {
                Ok(_) => {
                    print_success(
                        "✨ MCP server automatically configured for all enabled clients!",
                    );
                }
                Err(e) => {
                    print_warning(&format!("Could not auto-configure MCP clients: {}", e));
                    print_info("Manual configuration required - see instructions below");
                    show_manual_mcp_instructions(project_name, canister_id)?;
                }
            }
        } else {
            show_manual_mcp_instructions(project_name, canister_id)?;
        }
    } else {
        show_manual_mcp_instructions(project_name, canister_id)?;
    }

    Ok(())
}

fn show_manual_mcp_instructions(project_name: &str, canister_id: &str) -> Result<()> {
    // Show manual configuration instructions
    println!();
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
    );
    println!("{}", "🔌 MCP Client Integration".bold().cyan());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
    );
    println!();

    // Generate configuration for all supported clients
    let server_config = generate_claude_server_config(project_name, canister_id);
    let claude_config = json!({
        "mcpServers": server_config
    });

    println!("Configure your AI clients:");
    println!();
    println!("📋 Claude Desktop configuration:");
    println!(
        "{}",
        serde_json::to_string_pretty(&claude_config)?.bright_blue()
    );
    println!();

    println!("Or use the MCP command for guided setup:");
    println!(
        "  {}",
        format!("icarus mcp add {} --name {}", canister_id, project_name).bright_yellow()
    );

    Ok(())
}

async fn deploy_with_dfx(
    current_dir: &Path,
    project_name: &str,
    network: &str,
    init_arg: &str,
    force: bool,
) -> Result<()> {
    print_info("🚀 Starting deployment with dfx (showing build progress)...");

    let mut args = vec!["deploy", project_name, "--network", network];

    // Add init argument for new canisters
    args.push("--argument");
    args.push(init_arg);

    // Use auto mode for smart install/upgrade logic
    if force {
        args.push("--mode");
        args.push("reinstall");
        args.push("--yes"); // Auto-approve reinstalls
        print_info("⚡ Force reinstall mode enabled");
    } else {
        args.push("--mode");
        args.push("auto"); // Auto-detect install vs upgrade
        print_info("🔄 Using auto-detect mode (install vs upgrade)");
    }

    println!(); // Add spacing before dfx output
    run_command_streaming("dfx", &args, Some(current_dir)).await?;
    println!(); // Add spacing after dfx output

    Ok(())
}
