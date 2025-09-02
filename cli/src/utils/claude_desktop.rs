//! Utilities for Claude Desktop configuration management

use anyhow::Result;
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::utils::{print_info, print_success};

/// Find the Claude Desktop configuration file
pub fn find_claude_config_path() -> Result<PathBuf> {
    // First check for .client-env file in Claude config directory
    let claude_dir = find_claude_directory()?;
    let client_env_path = claude_dir.join(".client-env");

    if client_env_path.exists() {
        // Read the .client-env file for custom config location
        let contents = std::fs::read_to_string(&client_env_path)?;
        for line in contents.lines() {
            if let Some(path) = line.strip_prefix("CLAUDE_CONFIG_PATH=") {
                let config_path = PathBuf::from(path.trim());
                if config_path.exists() {
                    return Ok(config_path);
                }
            }
        }
    }

    // Default to claude_desktop_config.json in Claude directory
    let default_config = claude_dir.join("claude_desktop_config.json");

    // Create .client-env file with default if it doesn't exist
    if !client_env_path.exists() {
        let env_content = format!("CLAUDE_CONFIG_PATH={}\n", default_config.display());
        std::fs::write(&client_env_path, env_content).ok();
    }

    Ok(default_config)
}

/// Find the Claude directory based on the platform
pub fn find_claude_directory() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    // Check common locations for Claude directory
    let possible_paths = vec![
        home.join("Library/Application Support/Claude"), // macOS
        home.join(".config/claude"),                     // Linux
        home.join("AppData/Roaming/Claude"),             // Windows
    ];

    for path in &possible_paths {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    // Return the most likely path for the current platform
    #[cfg(target_os = "macos")]
    return Ok(possible_paths[0].clone());

    #[cfg(target_os = "linux")]
    return Ok(possible_paths[1].clone());

    #[cfg(target_os = "windows")]
    return Ok(possible_paths[2].clone());
}

/// Generate Claude Desktop MCP server configuration
pub fn generate_claude_server_config(name: &str, canister_id: &str) -> Value {
    json!({
        name: {
            "command": "icarus",
            "args": ["bridge", "start", "--canister-id", canister_id],
            "env": {}
        }
    })
}

/// Update Claude Desktop configuration with new MCP server
pub fn update_claude_config(
    config_path: &PathBuf,
    server_name: &str,
    server_config: Value,
) -> Result<()> {
    print_info(&format!(
        "Updating Claude Desktop configuration at: {}",
        config_path.display()
    ));

    // Read existing config or create new one
    let mut config = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        serde_json::from_str::<Value>(&content)?
    } else {
        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        json!({
            "mcpServers": {}
        })
    };

    // Add or update the server configuration
    if let Some(servers) = config.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
        if let Some(server_obj) = server_config.as_object() {
            for (key, value) in server_obj {
                servers.insert(key.clone(), value.clone());
            }
        }
    } else {
        // Create mcpServers object if it doesn't exist
        config["mcpServers"] = json!({});
        if let Some(servers) = config.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
            if let Some(server_obj) = server_config.as_object() {
                for (key, value) in server_obj {
                    servers.insert(key.clone(), value.clone());
                }
            }
        }
    }

    // Write the updated config
    let pretty_json = serde_json::to_string_pretty(&config)?;
    std::fs::write(config_path, pretty_json)?;

    print_success(&format!(
        "Added '{}' to Claude Desktop configuration",
        server_name
    ));
    print_info("Restart Claude Desktop to load the new MCP server");

    Ok(())
}
