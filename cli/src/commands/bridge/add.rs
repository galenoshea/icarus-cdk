use anyhow::Result;
use candid::Principal;
use colored::Colorize;
use ic_agent::Agent;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use crate::utils::{create_spinner, print_error, print_info, print_success};

pub async fn execute(
    canister_id: String,
    name: Option<String>,
    description: Option<String>,
) -> Result<()> {
    // Validate canister ID format
    if !is_valid_canister_id(&canister_id) {
        print_error("Invalid canister ID format");
        return Err(anyhow::anyhow!("Canister ID must be a valid Principal"));
    }

    // Try to fetch metadata from the canister if name/description not provided
    let (name, description) = if name.is_none() || description.is_none() {
        match fetch_canister_metadata(&canister_id).await {
            Ok((fetched_name, fetched_desc)) => {
                print_success(&format!("✓ Fetched metadata from canister"));
                (
                    name.unwrap_or(fetched_name),
                    description.unwrap_or(fetched_desc),
                )
            }
            Err(e) => {
                print_info(&format!("Could not fetch metadata: {}", e));
                print_info("Using canister ID as name");
                (
                    name.unwrap_or_else(|| canister_id.clone()),
                    description.unwrap_or_else(|| format!("Icarus MCP tool: {}", canister_id)),
                )
            }
        }
    } else {
        (
            name.unwrap_or_else(|| canister_id.clone()),
            description.unwrap_or_else(|| format!("Icarus MCP tool")),
        )
    };

    // Get Claude Desktop config directory (platform-independent)
    let config_path = get_claude_config_path()?;

    // Ensure config directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Read existing config or create new
    let mut config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    // Ensure mcpServers section exists
    if !config.get("mcpServers").is_some() {
        config["mcpServers"] = json!({});
    }

    // Add the new canister configuration
    let server_name = format!("icarus-{}", name.to_lowercase().replace(' ', "-"));

    // Get the full path to icarus executable (installed via cargo)
    let icarus_command = get_icarus_executable_path();

    config["mcpServers"][&server_name] = json!({
        "command": icarus_command,
        "args": ["bridge", "start", "--canister-id", canister_id],
        "description": description
    });

    // Write updated config
    let pretty_json = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, pretty_json)?;

    print_success(&format!(
        "✅ Added canister {} to Claude Desktop",
        canister_id
    ));
    println!();
    println!("{}", "Configuration added:".bold());
    println!("  Name: {}", server_name.cyan());
    println!("  Canister: {}", canister_id.yellow());
    println!("  Config: {}", config_path.display());
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Restart Claude Desktop");
    println!("  2. The tool will appear in Claude's MCP servers");
    println!("  3. Claude will automatically start the bridge when needed");

    Ok(())
}

fn is_valid_canister_id(id: &str) -> bool {
    // Basic validation - should be a valid Principal ID
    // Format: xxxxx-xxxxx-xxxxx-xxxxx-cai or similar
    id.len() > 10 && (id.contains('-') || id.len() == 27)
}

async fn fetch_canister_metadata(canister_id: &str) -> Result<(String, String)> {
    let spinner = create_spinner("Fetching canister metadata");

    // Parse the canister ID as a Principal
    let principal = Principal::from_text(canister_id)
        .map_err(|e| anyhow::anyhow!("Invalid canister ID: {}", e))?;

    // Create an agent (try local first, then mainnet)
    let agent = create_agent().await?;

    // Call list_tools() on the canister
    let response = agent
        .query(&principal, "list_tools")
        .with_arg(candid::encode_args(()).unwrap())
        .call()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to call list_tools: {}", e))?;

    spinner.finish_and_clear();

    // Decode the response (expecting a JSON string)
    let metadata_json: String = candid::decode_one(&response)
        .map_err(|e| anyhow::anyhow!("Failed to decode metadata: {}", e))?;

    // Parse the JSON to extract name and description
    let metadata: serde_json::Value = serde_json::from_str(&metadata_json)
        .map_err(|e| anyhow::anyhow!("Failed to parse metadata JSON: {}", e))?;

    let name = metadata["name"].as_str().unwrap_or(canister_id).to_string();

    let description = metadata["description"]
        .as_str()
        .unwrap_or(&format!("MCP tool: {}", name))
        .to_string();

    Ok((name, description))
}

async fn create_agent() -> Result<Agent> {
    // Try local first
    let local_url = "http://localhost:4943";

    let agent = Agent::builder()
        .with_url(local_url)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create agent: {}", e))?;

    // For local network, fetch the root key
    match agent.fetch_root_key().await {
        Ok(_) => Ok(agent),
        Err(_) => {
            // Try mainnet
            let mainnet_url = "https://ic0.app";
            Agent::builder()
                .with_url(mainnet_url)
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to create mainnet agent: {}", e))
        }
    }
}

fn get_icarus_executable_path() -> String {
    // Get the full path to icarus executable in cargo bin directory
    let home = dirs::home_dir().expect("Cannot find home directory");
    let icarus_path = if cfg!(target_os = "windows") {
        home.join(".cargo").join("bin").join("icarus.exe")
    } else {
        home.join(".cargo").join("bin").join("icarus")
    };

    // Convert to string, using forward slashes even on Windows for consistency
    icarus_path.to_string_lossy().to_string()
}

fn get_claude_config_path() -> Result<PathBuf> {
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
