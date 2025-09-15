//! Multi-client MCP configuration support
//!
//! This module provides a unified interface for configuring MCP servers
//! across different AI clients (Claude Desktop, ChatGPT Desktop, Claude Code, etc.)

use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;

pub mod claude_desktop;
pub mod chatgpt_desktop;
pub mod claude_code;
pub mod client_detector;

/// Represents an AI client that can be configured with MCP servers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClientType {
    ClaudeDesktop,
    ChatGptDesktop,
    ClaudeCode,
}

impl ClientType {
    /// Get the display name for this client type
    pub fn display_name(&self) -> &'static str {
        match self {
            ClientType::ClaudeDesktop => "Claude Desktop",
            ClientType::ChatGptDesktop => "ChatGPT Desktop",
            ClientType::ClaudeCode => "Claude Code/Cline",
        }
    }

    /// Get the emoji icon for this client type
    pub fn emoji(&self) -> &'static str {
        match self {
            ClientType::ClaudeDesktop => "🤖",
            ClientType::ChatGptDesktop => "💬",
            ClientType::ClaudeCode => "🎨",
        }
    }


    /// Get all available client types
    pub fn all() -> Vec<ClientType> {
        vec![
            ClientType::ClaudeDesktop,
            ClientType::ChatGptDesktop,
            ClientType::ClaudeCode,
        ]
    }
}

/// Information about a detected AI client installation
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub client_type: ClientType,
    pub config_path: PathBuf,
    pub is_installed: bool,
}

/// Trait for AI client configuration management
pub trait McpClient {
    /// Get the client type
    fn client_type(&self) -> ClientType;


    /// Detect if this client is installed on the system
    fn detect_installation(&self) -> Result<ClientInfo>;

    /// Find the configuration file path for this client
    fn find_config_path(&self) -> Result<PathBuf>;

    /// Generate MCP server configuration for this client
    fn generate_server_config(&self, name: &str, canister_id: &str) -> Value;

    /// Update the client configuration with new MCP server
    fn update_config(&self, config_path: &PathBuf, server_name: &str, server_config: Value) -> Result<()>;

    /// Remove an MCP server from the client configuration
    fn remove_config(&self, config_path: &PathBuf, server_name: &str) -> Result<()>;

    /// List configured MCP servers for this client
    fn list_servers(&self, config_path: &PathBuf) -> Result<Vec<String>>;

    /// Validate that the configuration is correct
    fn validate_config(&self, config_path: &PathBuf) -> Result<bool>;
}

/// Registry for all supported MCP clients
pub struct ClientRegistry {
    clients: Vec<Box<dyn McpClient>>,
}

impl ClientRegistry {
    /// Create a new client registry with all supported clients
    pub fn new() -> Self {
        Self {
            clients: vec![
                Box::new(claude_desktop::ClaudeDesktopClient::new()),
                Box::new(chatgpt_desktop::ChatGptDesktopClient::new()),
                Box::new(claude_code::ClaudeCodeClient::new()),
            ],
        }
    }


    /// Get a specific client by type
    pub fn get_client(&self, client_type: ClientType) -> Option<&Box<dyn McpClient>> {
        self.clients
            .iter()
            .find(|client| client.client_type() == client_type)
    }


    /// Get all available clients (installed or not)
    pub fn get_all_client_info(&self) -> Vec<ClientInfo> {
        self.clients
            .iter()
            .filter_map(|client| client.detect_installation().ok())
            .collect()
    }
}

impl Default for ClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_type_display_name() {
        assert_eq!(ClientType::ClaudeDesktop.display_name(), "Claude Desktop");
        assert_eq!(ClientType::ChatGptDesktop.display_name(), "ChatGPT Desktop");
        assert_eq!(ClientType::ClaudeCode.display_name(), "Claude Code/Cline");
    }

    #[test]
    fn test_client_type_emoji() {
        assert_eq!(ClientType::ClaudeDesktop.emoji(), "🤖");
        assert_eq!(ClientType::ChatGptDesktop.emoji(), "💬");
        assert_eq!(ClientType::ClaudeCode.emoji(), "🎨");
    }

    #[test]
    fn test_client_type_all() {
        let all_types = ClientType::all();
        assert_eq!(all_types.len(), 3);
        assert!(all_types.contains(&ClientType::ClaudeDesktop));
        assert!(all_types.contains(&ClientType::ChatGptDesktop));
        assert!(all_types.contains(&ClientType::ClaudeCode));
    }

    #[test]
    fn test_client_registry_creation() {
        let registry = ClientRegistry::new();
        assert_eq!(registry.clients.len(), 3);
    }

    #[test]
    fn test_client_registry_get_client() {
        let registry = ClientRegistry::new();

        let claude_client = registry.get_client(ClientType::ClaudeDesktop);
        assert!(claude_client.is_some());
        assert_eq!(claude_client.unwrap().client_type(), ClientType::ClaudeDesktop);

        let chatgpt_client = registry.get_client(ClientType::ChatGptDesktop);
        assert!(chatgpt_client.is_some());
        assert_eq!(chatgpt_client.unwrap().client_type(), ClientType::ChatGptDesktop);

        let claude_code_client = registry.get_client(ClientType::ClaudeCode);
        assert!(claude_code_client.is_some());
        assert_eq!(claude_code_client.unwrap().client_type(), ClientType::ClaudeCode);
    }

    #[test]
    fn test_client_registry_get_all_client_info() {
        let registry = ClientRegistry::new();
        let all_info = registry.get_all_client_info();

        // Should return info for all 3 clients
        assert_eq!(all_info.len(), 3);

        // Verify we have all client types
        let client_types: Vec<ClientType> = all_info.iter().map(|info| info.client_type.clone()).collect();
        assert!(client_types.contains(&ClientType::ClaudeDesktop));
        assert!(client_types.contains(&ClientType::ChatGptDesktop));
        assert!(client_types.contains(&ClientType::ClaudeCode));
    }

    #[test]
    fn test_client_info_structure() {
        let registry = ClientRegistry::new();
        let all_info = registry.get_all_client_info();

        for info in all_info {
            // Each ClientInfo should have valid fields
            assert!(!info.config_path.as_os_str().is_empty());
            // is_installed can be true or false depending on system
        }
    }
}