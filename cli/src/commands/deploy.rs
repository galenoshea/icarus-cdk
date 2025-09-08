use anyhow::Result;
use colored::Colorize;
use serde_json::json;
use std::path::Path;

use crate::config::cargo_config;
use crate::utils::{
    claude_desktop::{
        find_claude_config_path, generate_claude_server_config, update_claude_config,
    },
    print_info, print_success, print_warning, run_command,
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

    // Run dfx deploy with appropriate arguments
    let mut args = vec!["deploy", &project_name, "--network", &network];

    if let Some(canister_id) = &upgrade {
        args.push("--upgrade-unchanged");
        print_info(&format!("Upgrading canister {}", canister_id));
    }

    if force {
        args.push("--yes");
        print_info("Force deploying (will delete existing canister if present)");
    }

    // Run dfx deploy
    let output = run_command("dfx", &args, Some(&current_dir)).await?;

    // Parse the output to find the canister ID
    let canister_id = extract_canister_id(&output, &project_name).await?;

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

async fn extract_canister_id(output: &str, project_name: &str) -> Result<String> {
    // Look for patterns like:
    // "Installing code for canister project_name, with canister ID xxxxx-xxxxx-xxxxx-xxxxx-xxxxx"
    // or "Deployed canisters." followed by canister URLs

    // First try to find the canister ID from the deployment message
    if let Some(line) = output.lines().find(|l| l.contains("with canister ID")) {
        if let Some(id_part) = line.split("with canister ID").nth(1) {
            let id = id_part.trim();
            if !id.is_empty() && id.contains('-') {
                return Ok(id.to_string());
            }
        }
    }

    // Try to find from URLs section
    if let Some(url_section_start) = output.lines().position(|l| l.contains("URLs:")) {
        let lines: Vec<&str> = output.lines().collect();
        for i in url_section_start + 1..lines.len() {
            if lines[i].contains("http") {
                // Extract canister ID from URL
                if let Some(id) = extract_id_from_url(lines[i]) {
                    return Ok(id);
                }
            }
        }
    }

    // As a fallback, try to get it from dfx canister id
    match tokio::process::Command::new("dfx")
        .args(&["canister", "id", project_name])
        .current_dir(std::env::current_dir()?)
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => anyhow::bail!("Could not determine canister ID from deployment output"),
    }
}

fn extract_id_from_url(url_line: &str) -> Option<String> {
    // Extract from patterns like:
    // http://xxxxx-xxxxx-xxxxx-xxxxx-xxxxx.localhost:4943
    // or from query params like &id=xxxxx-xxxxx-xxxxx-xxxxx-xxxxx

    if let Some(id_part) = url_line.split("&id=").nth(1) {
        let id = id_part.split_whitespace().next()?;
        if id.contains('-') {
            return Some(id.to_string());
        }
    }

    if let Some(start) = url_line.find("http://") {
        let url_part = &url_line[start + 7..];
        if let Some(end) = url_part.find('.') {
            let id = &url_part[..end];
            if id.contains('-') {
                return Some(id.to_string());
            }
        }
    }

    None
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
