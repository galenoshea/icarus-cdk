use anyhow::Result;
use colored::Colorize;
use serde_json::json;
use std::path::Path;

use crate::config::cargo_config;
use crate::utils::{
    claude_desktop::{
        find_claude_config_path, generate_claude_server_config, update_claude_config,
    },
    print_info, print_success, print_warning, run_command, run_command_interactive,
};

pub async fn execute(network: String, force: bool, upgrade: Option<String>) -> Result<()> {
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

    // Build the WASM first (with visible output)
    print_info("Building WASM...");
    run_command_interactive(
        "cargo",
        &["build", "--target", "wasm32-unknown-unknown", "--release"],
        Some(&current_dir),
    )
    .await?;

    // Extract and update the Candid interface from the built WASM
    print_info("Updating Candid interface from WASM...");
    update_candid_from_wasm(&current_dir, &project_name).await?;

    // Get the current principal to use as the init argument
    let principal_output =
        run_command("dfx", &["identity", "get-principal"], Some(&current_dir)).await?;
    let principal = principal_output.trim().to_string();
    print_info(&format!("Using principal: {}", principal));

    // Build args with owned strings for the init argument
    let init_arg = format!("(principal \"{}\")", principal);

    // Build the command args as owned strings
    let mut cmd_args = vec![
        "deploy".to_string(),
        project_name.clone(),
        "--network".to_string(),
        network.clone(),
        "--argument".to_string(),
        init_arg,
    ];

    if let Some(canister_id) = &upgrade {
        cmd_args.push("--upgrade-unchanged".to_string());
        print_info(&format!("Upgrading canister {}", canister_id));
    }

    if force {
        cmd_args.push("--yes".to_string());
        print_info("Force deploying (will delete existing canister if present)");
    }

    // Convert to &str for run_command
    let args: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();

    // Run dfx deploy with visible output and interactive prompts
    run_command_interactive("dfx", &args, Some(&current_dir)).await?;

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

    // Get Candid UI canister ID for local network
    let candid_ui_id = if network == "local" {
        get_candid_ui_canister_id().await.ok()
    } else {
        None
    };

    // Display URLs
    println!("\n{}", "URLs:".bold());
    println!("  Backend canister via Candid interface:");

    if network == "local" {
        if let Some(ui_id) = candid_ui_id {
            println!(
                "    {}: {}",
                project_name,
                format!(
                    "http://127.0.0.1:4943/?canisterId={}&id={}",
                    ui_id, canister_id
                )
                .bright_blue()
                .underline()
            );
        } else {
            println!(
                "    {}: {}",
                project_name,
                format!("http://{}.localhost:4943", canister_id)
                    .bright_blue()
                    .underline()
            );
        }
    } else {
        println!(
            "    {}: {}",
            project_name,
            format!("https://{}.icp0.io", canister_id)
                .bright_blue()
                .underline()
        );
    }
    println!();

    // Save canister ID for future reference
    save_canister_id(&current_dir, &network, &canister_id)?;

    // Handle Claude Desktop configuration
    handle_claude_desktop_config(&current_dir, &project_name, &canister_id).await?;

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

fn get_project_name(project_dir: &Path) -> Result<String> {
    let cargo_toml = project_dir.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml)?;
    let toml: toml::Value = toml::from_str(&content)?;

    toml.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Could not find package name in Cargo.toml"))
}

async fn get_candid_ui_canister_id() -> Result<String> {
    let output = tokio::process::Command::new("dfx")
        .args(&["canister", "id", "__Candid_UI"])
        .output()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        anyhow::bail!("Candid UI canister not found")
    }
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

async fn update_candid_from_wasm(project_dir: &Path, project_name: &str) -> Result<()> {
    use std::fs;

    // Construct paths
    let wasm_name = project_name.replace('-', "_");
    let wasm_path = project_dir
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join(format!("{}.wasm", wasm_name));

    let did_path = project_dir
        .join("src")
        .join(format!("{}.did", project_name));

    // Check if WASM exists
    if !wasm_path.exists() {
        anyhow::bail!(
            "WASM file not found at {:?}. Build may have failed.",
            wasm_path
        );
    }

    // Check if candid-extractor is available
    if which::which("candid-extractor").is_err() {
        print_warning("candid-extractor not found. Install with: cargo install candid-extractor");
        print_warning("Continuing with existing .did file...");
        return Ok(());
    }

    // Extract Candid from WASM
    let output = tokio::process::Command::new("candid-extractor")
        .arg(&wasm_path)
        .output()
        .await?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        print_warning(&format!("Could not extract Candid: {}", error));
        print_warning("Continuing with existing .did file...");
        return Ok(());
    }

    // Get the extracted Candid
    let candid_content = String::from_utf8_lossy(&output.stdout);

    // Only update if we got valid content
    if candid_content.contains("service") {
        fs::write(&did_path, candid_content.as_ref())?;
        print_success("Candid interface updated with actual tool functions");
    } else {
        print_warning("Extracted Candid seems invalid, keeping existing .did file");
    }

    Ok(())
}

async fn handle_claude_desktop_config(
    project_dir: &Path,
    project_name: &str,
    canister_id: &str,
) -> Result<()> {
    // Check for Cargo.toml metadata and handle Claude Desktop configuration
    let mut claude_auto_updated = false;
    if let Some(icarus_metadata) = cargo_config::load_from_cargo_toml(project_dir)? {
        if icarus_metadata.claude_desktop.auto_update {
            println!();
            print_info("Auto-updating Claude Desktop configuration...");

            // Determine config path
            let claude_config_path = if let Some(custom_path) =
                cargo_config::get_claude_config_path(&icarus_metadata, project_dir)
            {
                custom_path
            } else {
                find_claude_config_path()?
            };

            // Generate and update config
            let mut server_config = generate_claude_server_config(project_name, canister_id);

            // Update the command to use full path
            if let Some(servers) = server_config.as_object_mut() {
                if let Some(server) = servers.get_mut(project_name) {
                    if let Some(server_obj) = server.as_object_mut() {
                        // Get the full path to icarus binary
                        let icarus_path = which::which("icarus")
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|_| "icarus".to_string());
                        server_obj.insert(
                            "command".to_string(),
                            serde_json::Value::String(icarus_path),
                        );
                    }
                }
            }

            match update_claude_config(&claude_config_path, project_name, server_config) {
                Ok(_) => {
                    print_success("Claude Desktop configuration updated automatically!");
                    claude_auto_updated = true;
                }
                Err(e) => {
                    print_warning(&format!("Could not update Claude Desktop config: {}", e));
                    print_info("Manual configuration required - see instructions below");
                }
            }
        }
    }

    // Only show manual configuration if auto-update didn't happen or failed
    if !claude_auto_updated {
        // Claude Desktop integration section
        println!();
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
        );
        println!("{}", "🔌 Claude Desktop Integration".bold().cyan());
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
        );
        println!();

        // Generate Claude Desktop configuration
        let claude_config = generate_claude_desktop_config(project_name, canister_id);

        println!("Add this to your Claude Desktop configuration:");
        println!();
        println!(
            "{}",
            serde_json::to_string_pretty(&claude_config)?.bright_blue()
        );
        println!();

        println!("Or use the connect command for guided setup:");
        println!(
            "  {}",
            format!("icarus connect --canister-id {}", canister_id).bright_yellow()
        );
    }

    Ok(())
}

fn generate_claude_desktop_config(project_name: &str, canister_id: &str) -> serde_json::Value {
    // Get the full path to icarus binary
    let icarus_path = which::which("icarus")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "icarus".to_string());

    json!({
        "mcpServers": {
            project_name: {
                "command": icarus_path,
                "args": ["bridge", "start", "--canister-id", canister_id],
                "env": {}
            }
        }
    })
}
