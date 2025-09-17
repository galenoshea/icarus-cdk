use anyhow::Result;
use colored::Colorize;
use serde_json::json;
use std::path::Path;

use crate::config::cargo_config;
use crate::utils::{
    build_utils::{build_canister_wasi_native, get_project_name},
    claude_desktop::generate_claude_server_config,
    print_info, print_success, print_warning, run_command, run_command_interactive,
};

pub async fn execute(network: String, force: bool, _upgrade: Option<String>) -> Result<()> {
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

    // Build with WASI-Native architecture (auto-handles conversion)
    print_info("Building WASM with WASI-Native architecture...");

    // Step 1: Compile (WASI by default, pure WASM if configured)
    let target = "wasm32-wasip1"; // WASI-Native default

    run_command_interactive(
        "cargo",
        &["build", "--target", target, "--release"],
        Some(&current_dir),
    )
    .await?;

    // Step 2: Check if manual .did file exists (for WASI projects that can't use export_candid!)
    let did_path = current_dir
        .join("src")
        .join(format!("{}.did", project_name));
    if did_path.exists() {
        print_info("Using existing manual .did file (WASI-compatible approach)");
    } else {
        // Step 2a: Extract Candid interface BEFORE WASI conversion (for non-WASI projects)
        let pre_conversion_wasm_path =
            crate::utils::build_utils::get_wasm_path(&current_dir, &project_name, target);
        print_info("Extracting Candid interface from WASM...");
        update_candid_from_wasm_at_path(&current_dir, &project_name, &pre_conversion_wasm_path)
            .await?;
    }

    // Step 3: Convert to IC-compatible WASM using WASI-Native pipeline
    let final_wasm_path = build_canister_wasi_native(&current_dir, false).await?;

    // Optimize WASM with ic-wasm if available
    optimize_wasm_at_path(&final_wasm_path).await?;

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

async fn optimize_wasm_at_path(wasm_path: &Path) -> Result<()> {
    use std::fs;

    // Check if ic-wasm is available
    if which::which("ic-wasm").is_err() {
        print_info(
            "ic-wasm not found. Skipping optimization (install with: cargo install ic-wasm)",
        );
        return Ok(());
    }

    if !wasm_path.exists() {
        print_warning("WASM file not found, skipping optimization");
        return Ok(());
    }

    // Get original size
    let original_size = fs::metadata(wasm_path)?.len();

    print_info("Optimizing WASM with ic-wasm...");

    // Run ic-wasm shrink
    let result = tokio::process::Command::new("ic-wasm")
        .arg(wasm_path)
        .arg("-o")
        .arg(wasm_path)
        .arg("shrink")
        .output()
        .await;

    match result {
        Ok(output) if output.status.success() => {
            // Get new size
            let new_size = fs::metadata(wasm_path)?.len();
            let reduction = ((original_size - new_size) as f64 / original_size as f64) * 100.0;

            print_success(&format!(
                "WASM optimized: {} → {} (reduced by {:.1}%)",
                format_size(original_size),
                format_size(new_size),
                reduction
            ));
        }
        Ok(output) => {
            let error = String::from_utf8_lossy(&output.stderr);
            print_warning(&format!("WASM optimization failed: {}", error));
        }
        Err(e) => {
            print_warning(&format!("Could not run ic-wasm: {}", e));
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

async fn update_candid_from_wasm_at_path(
    project_dir: &Path,
    project_name: &str,
    wasm_path: &Path,
) -> Result<()> {
    use std::fs;

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
        .arg(wasm_path)
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
    print_info("Deploying via dfx (leverages dfx's smart upgrade logic)...");

    let mut args = vec!["deploy", project_name, "--network", network];

    // Add init argument for new canisters
    args.push("--argument");
    args.push(init_arg);

    // Use auto mode for smart install/upgrade logic
    if force {
        args.push("--mode");
        args.push("reinstall");
        args.push("--yes"); // Auto-approve reinstalls
    } else {
        args.push("--mode");
        args.push("auto"); // Auto-detect install vs upgrade
    }

    run_command_interactive("dfx", &args, Some(current_dir)).await?;
    Ok(())
}
