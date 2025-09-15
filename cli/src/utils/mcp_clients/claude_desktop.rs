//! Claude Desktop MCP client configuration

use anyhow::Result;
use serde_json::{json, Value};
use std::path::PathBuf;

use super::{client_detector, ClientInfo, ClientType, McpClient};

pub struct ClaudeDesktopClient;

impl ClaudeDesktopClient {
    pub fn new() -> Self {
        Self
    }

    /// Find the Claude Desktop configuration file
    pub fn find_claude_config_path() -> Result<PathBuf> {
        // First check for .client-env file in Claude config directory
        let claude_dir = Self::find_claude_directory()?;
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
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

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
        // Get the full path to icarus binary
        let icarus_path = which::which("icarus")
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "icarus".to_string());

        json!({
            name: {
                "command": icarus_path,
                "args": ["mcp", "start", canister_id],
                "env": {}
            }
        })
    }

    /// Update Claude Desktop configuration with new MCP server
    pub fn update_claude_config(
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

    /// Remove an MCP server from Claude Desktop configuration
    pub fn remove_claude_config(config_path: &PathBuf, server_name: &str) -> Result<()> {
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

    /// List configured MCP servers in Claude Desktop
    pub fn list_claude_servers(config_path: &PathBuf) -> Result<Vec<String>> {
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

impl McpClient for ClaudeDesktopClient {
    fn client_type(&self) -> ClientType {
        ClientType::ClaudeDesktop
    }

    fn detect_installation(&self) -> Result<ClientInfo> {
        // Check if Claude Desktop app is installed
        let is_installed = client_detector::check_app_installed("Claude")
            || client_detector::check_app_installed("Claude Desktop");

        // Try to find config directory
        let config_path = if is_installed {
            Self::find_claude_config_path().unwrap_or_else(|_| {
                Self::find_claude_directory()
                    .unwrap_or_else(|_| PathBuf::from("/"))
                    .join("claude_desktop_config.json")
            })
        } else {
            Self::find_claude_directory()
                .unwrap_or_else(|_| PathBuf::from("/"))
                .join("claude_desktop_config.json")
        };

        Ok(ClientInfo {
            client_type: ClientType::ClaudeDesktop,
            config_path,
            is_installed,
        })
    }

    fn find_config_path(&self) -> Result<PathBuf> {
        Self::find_claude_config_path()
    }

    fn generate_server_config(&self, name: &str, canister_id: &str) -> Value {
        Self::generate_claude_server_config(name, canister_id)
    }

    fn update_config(
        &self,
        config_path: &PathBuf,
        server_name: &str,
        server_config: Value,
    ) -> Result<()> {
        Self::update_claude_config(config_path, server_name, server_config)
    }

    fn remove_config(&self, config_path: &PathBuf, server_name: &str) -> Result<()> {
        Self::remove_claude_config(config_path, server_name)
    }

    fn list_servers(&self, config_path: &PathBuf) -> Result<Vec<String>> {
        Self::list_claude_servers(config_path)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_claude_desktop_client_creation() {
        let client = ClaudeDesktopClient::new();
        assert_eq!(client.client_type(), ClientType::ClaudeDesktop);
    }

    #[test]
    fn test_generate_claude_server_config() {
        let config = ClaudeDesktopClient::generate_claude_server_config(
            "test-server",
            "rdmx6-jaaaa-aaaah-qcaiq-cai",
        );

        // Verify structure
        assert!(config.is_object());
        let config_obj = config.as_object().unwrap();
        assert!(config_obj.contains_key("test-server"));

        let server_config = &config_obj["test-server"];
        assert!(server_config.is_object());

        let server_obj = server_config.as_object().unwrap();
        assert!(server_obj.contains_key("command"));
        assert!(server_obj.contains_key("args"));
        assert!(server_obj.contains_key("env"));

        // Verify command and args
        assert!(server_obj["command"].as_str().unwrap().ends_with("icarus"));
        let args = server_obj["args"].as_array().unwrap();
        assert_eq!(args.len(), 3);
        assert_eq!(args[0].as_str().unwrap(), "mcp");
        assert_eq!(args[1].as_str().unwrap(), "start");
        assert_eq!(args[2].as_str().unwrap(), "rdmx6-jaaaa-aaaah-qcaiq-cai");
    }

    #[test]
    fn test_update_claude_config_new_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path().to_path_buf();

        // Remove the temp file so we test creation
        drop(temp_file);

        let server_config = serde_json::json!({
            "test-server": {
                "command": "icarus",
                "args": ["mcp", "start", "test-canister"],
                "env": {}
            }
        });

        let result =
            ClaudeDesktopClient::update_claude_config(&config_path, "test-server", server_config);
        assert!(result.is_ok());

        // Verify file was created and has correct content
        assert!(config_path.exists());
        let content = std::fs::read_to_string(&config_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(parsed.get("mcpServers").is_some());
        let servers = parsed["mcpServers"].as_object().unwrap();
        assert!(servers.contains_key("test-server"));
    }

    #[test]
    fn test_update_claude_config_existing_file() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // Write initial config
        let initial_config = serde_json::json!({
            "mcpServers": {
                "existing-server": {
                    "command": "existing",
                    "args": []
                }
            }
        });
        temp_file
            .write_all(
                serde_json::to_string_pretty(&initial_config)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();
        temp_file.flush().unwrap();

        let config_path = temp_file.path().to_path_buf();

        let server_config = serde_json::json!({
            "new-server": {
                "command": "icarus",
                "args": ["mcp", "start", "new-canister"],
                "env": {}
            }
        });

        let result =
            ClaudeDesktopClient::update_claude_config(&config_path, "new-server", server_config);
        assert!(result.is_ok());

        // Verify both servers exist
        let content = std::fs::read_to_string(&config_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        let servers = parsed["mcpServers"].as_object().unwrap();
        assert!(servers.contains_key("existing-server"));
        assert!(servers.contains_key("new-server"));
    }

    #[test]
    fn test_remove_claude_config() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // Write config with multiple servers
        let initial_config = serde_json::json!({
            "mcpServers": {
                "server1": {"command": "cmd1"},
                "server2": {"command": "cmd2"}
            }
        });
        temp_file
            .write_all(
                serde_json::to_string_pretty(&initial_config)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();
        temp_file.flush().unwrap();

        let config_path = temp_file.path().to_path_buf();

        let result = ClaudeDesktopClient::remove_claude_config(&config_path, "server1");
        assert!(result.is_ok());

        // Verify server1 was removed but server2 remains
        let content = std::fs::read_to_string(&config_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        let servers = parsed["mcpServers"].as_object().unwrap();
        assert!(!servers.contains_key("server1"));
        assert!(servers.contains_key("server2"));
    }

    #[test]
    fn test_list_claude_servers() {
        let mut temp_file = NamedTempFile::new().unwrap();

        let config = serde_json::json!({
            "mcpServers": {
                "server1": {"command": "cmd1"},
                "server2": {"command": "cmd2"},
                "server3": {"command": "cmd3"}
            }
        });
        temp_file
            .write_all(serde_json::to_string_pretty(&config).unwrap().as_bytes())
            .unwrap();
        temp_file.flush().unwrap();

        let config_path = temp_file.path().to_path_buf();
        let servers = ClaudeDesktopClient::list_claude_servers(&config_path).unwrap();

        assert_eq!(servers.len(), 3);
        assert!(servers.contains(&"server1".to_string()));
        assert!(servers.contains(&"server2".to_string()));
        assert!(servers.contains(&"server3".to_string()));
    }

    #[test]
    fn test_list_claude_servers_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();

        let config = serde_json::json!({
            "mcpServers": {}
        });
        temp_file
            .write_all(serde_json::to_string_pretty(&config).unwrap().as_bytes())
            .unwrap();
        temp_file.flush().unwrap();

        let config_path = temp_file.path().to_path_buf();
        let servers = ClaudeDesktopClient::list_claude_servers(&config_path).unwrap();

        assert_eq!(servers.len(), 0);
    }

    #[test]
    fn test_validate_config() {
        let mut temp_file = NamedTempFile::new().unwrap();

        let config = serde_json::json!({
            "mcpServers": {
                "server1": {"command": "cmd1"}
            }
        });
        temp_file
            .write_all(serde_json::to_string_pretty(&config).unwrap().as_bytes())
            .unwrap();
        temp_file.flush().unwrap();

        let config_path = temp_file.path().to_path_buf();
        let client = ClaudeDesktopClient::new();

        assert!(client.validate_config(&config_path).unwrap());
    }

    #[test]
    fn test_validate_config_invalid() {
        let mut temp_file = NamedTempFile::new().unwrap();

        let config = serde_json::json!({
            "otherField": "value"
        });
        temp_file
            .write_all(serde_json::to_string_pretty(&config).unwrap().as_bytes())
            .unwrap();
        temp_file.flush().unwrap();

        let config_path = temp_file.path().to_path_buf();
        let client = ClaudeDesktopClient::new();

        assert!(!client.validate_config(&config_path).unwrap());
    }
}
