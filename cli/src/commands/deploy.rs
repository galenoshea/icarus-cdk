use anyhow::Result;
use colored::Colorize;
use serde_json::json;
use std::path::Path;

use crate::config::cargo_config;
use crate::utils::{
    claude_desktop::{
        find_claude_config_path, generate_claude_server_config, update_claude_config,
    },
    create_spinner,
    dfx::{self, deploy_canister, get_canister_id, install_canister},
    print_info, print_success, print_warning,
};

pub async fn execute(
    network: String,
    force: bool,
    upgrade: Option<String>,
    profile: Option<String>,
) -> Result<()> {
    // Validate network
    if !["local", "ic"].contains(&network.as_str()) {
        anyhow::bail!("Invalid network. Use 'local' or 'ic'");
    }

    // Check if we're in an Icarus project
    let current_dir = std::env::current_dir()?;
    if !is_icarus_project(&current_dir) {
        anyhow::bail!("Not in an Icarus project directory");
    }

    // Get project name
    let project_name = get_project_name(&current_dir)?;

    // Always build before deploying
    let build_msg = match profile.as_deref() {
        Some(p) => format!("Building project with {} profile...", p),
        None => "Building project with default profile...".to_string(),
    };
    print_info(&build_msg);

    // Apply profile settings
    let (opt_skip, opt_size, opt_perf, compress) = match profile.as_deref() {
        Some("size") => (false, true, false, true), // Maximum size optimization
        Some("speed") => (false, false, true, false), // Maximum performance, no compression
        Some("debug") => (true, false, false, false), // Fast builds, no optimization
        _ => (false, false, false, true),           // Default: balanced with compression
    };

    crate::commands::build::execute(opt_skip, opt_size, opt_perf, compress).await?;
    print_success("Build completed!");

    // Ensure dfx is running for local deployment
    if network == "local" {
        dfx::ensure_dfx_running().await?;
    }

    let spinner = create_spinner(&format!("Deploying to {}", network));

    // Deploy or upgrade with smart behavior
    let (canister_id, was_deployed) = if let Some(existing_id) = upgrade {
        // Explicit upgrade request
        spinner.set_message(format!("Upgrading canister {}", existing_id));

        install_canister(&project_name, "upgrade", &network).await?;
        print_success("Canister upgraded successfully! 🎉");
        (existing_id, true)
    } else {
        // Smart deploy: auto-upgrade if exists, create if not
        match get_canister_id(&project_name, &network).await {
            Ok(id) => {
                if force {
                    // Force new deployment - delete existing first
                    spinner.set_message(format!(
                        "Force deploying - deleting existing canister {}",
                        id
                    ));

                    // Delete existing canister
                    tokio::process::Command::new("dfx")
                        .args(&[
                            "canister",
                            "delete",
                            &project_name,
                            "--network",
                            &network,
                            "--yes",
                        ])
                        .output()
                        .await?;

                    // Deploy new canister
                    spinner.set_message("Creating new canister".to_string());
                    let cycles = if network == "ic" {
                        Some(1_000_000_000_000) // 1T cycles
                    } else {
                        None
                    };

                    let new_id = deploy_canister(&project_name, &network, cycles).await?;
                    print_success("Force deployed successfully! 🎉");
                    (new_id, true)
                } else {
                    // Auto-upgrade existing canister (new smart behavior)
                    spinner.set_message(format!("Auto-upgrading existing canister {}", id));
                    print_info(&format!(
                        "Found existing canister {}, upgrading with latest code...",
                        id
                    ));

                    install_canister(&project_name, "upgrade", &network).await?;
                    print_success("Canister upgraded successfully! 🎉");
                    (id, true)
                }
            }
            Err(_) => {
                // Deploy new canister
                spinner.set_message("Creating new canister".to_string());

                let cycles = if network == "ic" {
                    Some(1_000_000_000_000) // 1T cycles
                } else {
                    None
                };

                let id = deploy_canister(&project_name, &network, cycles).await?;
                print_success("Deployed successfully! 🎉");
                (id, true)
            }
        }
    };

    spinner.finish_and_clear();

    // Extract and save Candid interface if it doesn't exist or if forced
    let project_name = get_project_name(&current_dir)?;
    let candid_path = current_dir
        .join("src")
        .join(format!("{}.did", project_name));

    if force || !candid_path.exists() {
        // Try to update from canister, but don't fail if it doesn't work
        // (e.g., if the canister doesn't have the __get_candid_interface_tmp_hack method)
        if let Err(e) = update_candid_from_canister(&current_dir, &canister_id, &network).await {
            print_warning(&format!("Could not extract Candid from canister: {}", e));
            print_info("Using Candid file generated during build");
        }
    }

    // Get Candid UI canister ID for local network
    let candid_ui_id = if network == "local" {
        get_candid_ui_canister_id().await.ok()
    } else {
        None
    };

    // Display URLs in dfx format
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
            // Fallback to direct canister URL if Candid UI not found
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

    // Check for Cargo.toml metadata and handle Claude Desktop configuration
    let mut claude_auto_updated = false;
    if let Some(icarus_metadata) = cargo_config::load_from_cargo_toml(&current_dir)? {
        if icarus_metadata.claude_desktop.auto_update {
            println!();
            print_info("Auto-updating Claude Desktop configuration...");

            // Determine config path
            let claude_config_path = if let Some(custom_path) =
                cargo_config::get_claude_config_path(&icarus_metadata, &current_dir)
            {
                custom_path
            } else {
                find_claude_config_path()?
            };

            // Generate and update config
            let mut server_config = generate_claude_server_config(&project_name, &canister_id);

            // Update the command to use full path
            if let Some(servers) = server_config.as_object_mut() {
                if let Some(server) = servers.get_mut(&project_name) {
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

            match update_claude_config(&claude_config_path, &project_name, server_config) {
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
        let claude_config = generate_claude_desktop_config(&project_name, &canister_id);

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

async fn get_candid_ui_canister_id() -> Result<String> {
    // Try to get the Candid UI canister ID
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

async fn update_candid_from_canister(
    project_dir: &Path,
    canister_id: &str,
    network: &str,
) -> Result<()> {
    print_info("Extracting Candid interface from deployed canister...");

    // Call the __get_candid_interface_tmp_hack method
    // Use Tokio's Command directly to better control stdout/stderr
    let output = tokio::process::Command::new("dfx")
        .args(&[
            "canister",
            "call",
            canister_id,
            "__get_candid_interface_tmp_hack",
            "--query",
            "--network",
            network,
        ])
        .current_dir(project_dir)
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to extract Candid interface: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = String::from_utf8_lossy(&output.stdout);

    // Parse the output - it's wrapped in parentheses and quotes
    let output_trimmed = output.trim();

    if let Some(candid_str) = output_trimmed
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
    {
        // The content is wrapped in quotes with escape sequences
        let candid_str = candid_str.trim();

        // Parse as a quoted string - need to handle the comma at the end too
        let candid_content = if candid_str.starts_with('"') {
            // Find the last quote (may have trailing comma)
            let end_quote_pos = candid_str.rfind('"').unwrap_or(candid_str.len());
            let quoted_content = &candid_str[1..end_quote_pos];

            // Replace escape sequences
            quoted_content.replace("\\n", "\n").replace("\\\"", "\"")
        } else {
            candid_str.to_string()
        };

        // Save to the .did file
        let project_name = get_project_name(project_dir)?;
        let candid_path = project_dir
            .join("src")
            .join(format!("{}.did", project_name));
        std::fs::write(&candid_path, &candid_content)?;

        // Also update the .dfx directory
        let dfx_candid_path = project_dir
            .join(".dfx")
            .join(network)
            .join("canisters")
            .join(&project_name)
            .join("service.did");

        if let Some(parent) = dfx_candid_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dfx_candid_path, &candid_content)?;

        print_success("Candid interface updated successfully!");
    }

    Ok(())
}
