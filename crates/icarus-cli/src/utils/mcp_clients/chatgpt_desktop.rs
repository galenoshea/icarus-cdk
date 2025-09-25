//! ChatGPT Desktop MCP client configuration

use anyhow::Result;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

use super::{client_detector, ClientInfo, ClientType, McpClient};

pub struct ChatGptDesktopClient;

impl ChatGptDesktopClient {
    pub fn new() -> Self {
        Self
    }

    /// Find the ChatGPT Desktop directory based on the platform
    pub fn find_chatgpt_directory() -> Result<PathBuf> {
        let app_support = client_detector::get_app_support_dir()?;

        // ChatGPT Desktop is currently only available on macOS
        #[cfg(target_os = "macos")]
        {
            let chatgpt_dir = app_support.join("ChatGPT");
            Ok(chatgpt_dir)
        }

        #[cfg(not(target_os = "macos"))]
        {
            // ChatGPT Desktop is not available on other platforms yet
            anyhow::bail!("ChatGPT Desktop is currently only available on macOS")
        }
    }

    /// Find the ChatGPT Desktop configuration file
    pub fn find_chatgpt_config_path() -> Result<PathBuf> {
        let chatgpt_dir = Self::find_chatgpt_directory()?;

        // Estimated config file path - this may need adjustment when
        // ChatGPT Desktop MCP support is officially released
        let config_path = chatgpt_dir.join("chatgpt_config.json");

        Ok(config_path)
    }

    /// Generate ChatGPT Desktop MCP server configuration
    pub fn generate_chatgpt_server_config(name: &str, canister_id: &str) -> Value {
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

    /// Update ChatGPT Desktop configuration with new MCP server
    pub fn update_chatgpt_config(
        config_path: &Path,
        _server_name: &str,
        server_config: &Value,
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

    /// Remove an MCP server from ChatGPT Desktop configuration
    pub fn remove_chatgpt_config(config_path: &Path, server_name: &str) -> Result<()> {
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

    /// List configured MCP servers in ChatGPT Desktop
    pub fn list_chatgpt_servers(config_path: &Path) -> Result<Vec<String>> {
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
}

impl McpClient for ChatGptDesktopClient {
    fn client_type(&self) -> ClientType {
        ClientType::ChatGptDesktop
    }

    fn detect_installation(&self) -> Result<ClientInfo> {
        // Check if ChatGPT Desktop app is installed (macOS only)
        #[cfg(target_os = "macos")]
        let is_installed = client_detector::check_app_installed("ChatGPT");

        #[cfg(not(target_os = "macos"))]
        let is_installed = false;

        // Try to find config path
        let config_path = if is_installed {
            Self::find_chatgpt_config_path()
                .unwrap_or_else(|_| PathBuf::from("/Applications/ChatGPT.app"))
        } else {
            PathBuf::from("/Applications/ChatGPT.app")
        };

        Ok(ClientInfo {
            client_type: ClientType::ChatGptDesktop,
            config_path,
            is_installed,
        })
    }

    fn find_config_path(&self) -> Result<PathBuf> {
        Self::find_chatgpt_config_path()
    }

    fn generate_server_config(&self, name: &str, canister_id: &str) -> Value {
        Self::generate_chatgpt_server_config(name, canister_id)
    }

    fn update_config(
        &self,
        config_path: &Path,
        server_name: &str,
        server_config: &Value,
    ) -> Result<()> {
        Self::update_chatgpt_config(config_path, server_name, server_config)
    }

    fn remove_config(&self, config_path: &Path, server_name: &str) -> Result<()> {
        Self::remove_chatgpt_config(config_path, server_name)
    }

    fn list_servers(&self, config_path: &Path) -> Result<Vec<String>> {
        Self::list_chatgpt_servers(config_path)
    }

    fn validate_config(&self, config_path: &Path) -> Result<bool> {
        if !config_path.exists() {
            return Ok(false);
        }

        let content = std::fs::read_to_string(config_path)?;
        let config: Value = serde_json::from_str(&content)?;

        // Check if it has mcpServers key
        Ok(config.get("mcpServers").is_some())
    }
}
