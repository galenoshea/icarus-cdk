//! Claude Code/Cline VS Code extension MCP client configuration

use anyhow::Result;
use serde_json::{json, Value};
use std::path::PathBuf;

use super::{client_detector, ClientInfo, ClientType, McpClient};

pub struct ClaudeCodeClient;

impl ClaudeCodeClient {
    pub fn new() -> Self {
        Self
    }

    /// Find the Claude Code/Cline configuration directory
    pub fn find_claude_code_directory() -> Result<PathBuf> {
        let vscode_dir = client_detector::get_vscode_extensions_dir()?;

        // Claude Code/Cline extension ID in VS Code
        let extension_dir = vscode_dir.join("saoudrizwan.claude-dev");

        Ok(extension_dir)
    }

    /// Find the Claude Code/Cline configuration file
    pub fn find_claude_code_config_path() -> Result<PathBuf> {
        let extension_dir = Self::find_claude_code_directory()?;
        let config_path = extension_dir.join("settings/cline_mcp_settings.json");

        Ok(config_path)
    }

    /// Generate Claude Code/Cline MCP server configuration
    pub fn generate_claude_code_server_config(name: &str, canister_id: &str) -> Value {
        // Get the full path to icarus binary
        let icarus_path = which::which("icarus")
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "icarus".to_string());

        json!({
            name: {
                "command": icarus_path,
                "args": ["mcp", "start", canister_id],
                "env": {},
                "type": "stdio"
            }
        })
    }

    /// Update Claude Code/Cline configuration with new MCP server
    pub fn update_claude_code_config(
        config_path: &PathBuf,
        _server_name: &str,
        server_config: Value,
    ) -> Result<()> {
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

        Ok(())
    }

    /// Remove an MCP server from Claude Code/Cline configuration
    pub fn remove_claude_code_config(config_path: &PathBuf, server_name: &str) -> Result<()> {
        if !config_path.exists() {
            return Ok(()); // Nothing to remove
        }

        let content = std::fs::read_to_string(config_path)?;
        let mut config: Value = serde_json::from_str(&content)?;

        if let Some(servers) = config.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
            servers.remove(server_name);
        }

        let pretty_json = serde_json::to_string_pretty(&config)?;
        std::fs::write(config_path, pretty_json)?;

        Ok(())
    }

    /// List configured MCP servers in Claude Code/Cline
    pub fn list_claude_code_servers(config_path: &PathBuf) -> Result<Vec<String>> {
        if !config_path.exists() {
            return Ok(vec![]);
        }

        let content = std::fs::read_to_string(config_path)?;
        let config: Value = serde_json::from_str(&content)?;

        let servers = config
            .get("mcpServers")
            .and_then(|s| s.as_object())
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default();

        Ok(servers)
    }

    /// Check if VS Code is installed
    pub fn is_vscode_installed() -> bool {
        client_detector::check_app_installed("code")
            || client_detector::check_app_installed("Visual Studio Code")
    }

    /// Check if Claude Code/Cline extension is installed
    pub fn is_claude_code_extension_installed() -> bool {
        if let Ok(extension_dir) = Self::find_claude_code_directory() {
            extension_dir.exists()
        } else {
            false
        }
    }
}

impl McpClient for ClaudeCodeClient {
    fn client_type(&self) -> ClientType {
        ClientType::ClaudeCode
    }

    fn detect_installation(&self) -> Result<ClientInfo> {
        // Check if VS Code and Claude Code/Cline extension are installed
        let vscode_installed = Self::is_vscode_installed();
        let extension_installed = Self::is_claude_code_extension_installed();
        let is_installed = vscode_installed && extension_installed;

        // Try to find config path
        let config_path = if is_installed {
            Self::find_claude_code_config_path()
                .unwrap_or_else(|_| PathBuf::from("~/.vscode/extensions/saoudrizwan.claude-dev"))
        } else {
            PathBuf::from("~/.vscode/extensions/saoudrizwan.claude-dev")
        };

        Ok(ClientInfo {
            client_type: ClientType::ClaudeCode,
            config_path,
            is_installed,
        })
    }

    fn find_config_path(&self) -> Result<PathBuf> {
        Self::find_claude_code_config_path()
    }

    fn generate_server_config(&self, name: &str, canister_id: &str) -> Value {
        Self::generate_claude_code_server_config(name, canister_id)
    }

    fn update_config(
        &self,
        config_path: &PathBuf,
        server_name: &str,
        server_config: Value,
    ) -> Result<()> {
        Self::update_claude_code_config(config_path, server_name, server_config)
    }

    fn remove_config(&self, config_path: &PathBuf, server_name: &str) -> Result<()> {
        Self::remove_claude_code_config(config_path, server_name)
    }

    fn list_servers(&self, config_path: &PathBuf) -> Result<Vec<String>> {
        Self::list_claude_code_servers(config_path)
    }

    fn validate_config(&self, config_path: &PathBuf) -> Result<bool> {
        if !config_path.exists() {
            return Ok(false);
        }

        let content = std::fs::read_to_string(config_path)?;
        let config: Value = serde_json::from_str(&content)?;

        // Check if it has mcpServers key
        Ok(config.get("mcpServers").is_some())
    }
}
