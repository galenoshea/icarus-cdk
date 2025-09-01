use anyhow::Result;
use colored::Colorize;
use serde_json::Value;
use std::fs;

use crate::utils::{print_info, print_warning};

pub async fn execute(verbose: bool) -> Result<()> {
    // Get Claude Desktop config path
    let config_path = get_claude_config_path()?;

    if !config_path.exists() {
        print_warning("No Claude Desktop configuration found");
        println!("Add canisters with: icarus bridge add <canister-id>");
        return Ok(());
    }

    // Read config
    let content = fs::read_to_string(&config_path)?;
    let config: Value = serde_json::from_str(&content)?;

    // Get MCP servers
    let servers = config.get("mcpServers").and_then(|s| s.as_object());

    if servers.is_none() || servers.unwrap().is_empty() {
        print_info("No MCP servers configured");
        println!("Add canisters with: icarus bridge add <canister-id>");
        return Ok(());
    }

    let servers = servers.unwrap();

    // Filter for Icarus servers
    let icarus_servers: Vec<_> = servers
        .iter()
        .filter(|(_, v)| {
            v.get("command")
                .and_then(|c| c.as_str())
                .map(|c| c == "icarus")
                .unwrap_or(false)
        })
        .collect();

    if icarus_servers.is_empty() {
        print_info("No Icarus canisters configured");
        println!("Add canisters with: icarus bridge add <canister-id>");
        return Ok(());
    }

    println!("{}", "Configured Icarus Canisters:".bold());
    println!("{}", "=".repeat(60));

    for (name, config) in &icarus_servers {
        println!("\n{}", name.green().bold());

        if let Some(args) = config.get("args").and_then(|a| a.as_array()) {
            // Extract canister ID from args
            if let Some(canister_id_index) = args
                .iter()
                .position(|v| v.as_str() == Some("--canister-id"))
            {
                if let Some(canister_id) = args.get(canister_id_index + 1).and_then(|v| v.as_str())
                {
                    println!("  Canister ID: {}", canister_id.yellow());
                }
            }
        }

        if let Some(desc) = config.get("description").and_then(|d| d.as_str()) {
            println!("  Description: {}", desc);
        }

        if verbose {
            println!(
                "  Command: {}",
                config.get("command").and_then(|c| c.as_str()).unwrap_or("")
            );
            if let Some(args) = config.get("args").and_then(|a| a.as_array()) {
                let args_str: Vec<String> = args
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect();
                println!("  Args: {}", args_str.join(" "));
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("Total: {} canister(s)", icarus_servers.len());

    if !verbose {
        println!("\nUse --verbose for more details");
    }

    Ok(())
}

fn get_claude_config_path() -> Result<std::path::PathBuf> {
    // Get the config directory based on the platform
    let config_dir = if cfg!(target_os = "macos") {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
            .join("Library")
            .join("Application Support")
            .join("Claude")
    } else if cfg!(target_os = "windows") {
        dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find config directory"))?
            .join("Claude")
    } else {
        // Linux
        dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find config directory"))?
            .join("claude")
    };

    Ok(config_dir.join("claude_desktop_config.json"))
}
