//! MCP configuration types and management
//! These functions are used throughout the CLI but cargo's dead code analysis
//! doesn't always detect usage within the same crate

#![allow(dead_code)] // Methods are used but cargo may not detect cross-module usage

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use crate::types::{CanisterId, Network, ServerName};

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Server name (unique identifier)
    pub name: ServerName,
    /// Canister ID
    pub canister_id: CanisterId,
    /// Network (local, ic, testnet)
    pub network: Network,
    /// Server URL
    pub url: String,
    /// AI client name
    pub client: String,
    /// Port (optional)
    pub port: Option<u16>,
    /// Whether the server is enabled
    pub enabled: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// MCP configuration container
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    /// Registered MCP servers
    pub servers: Vec<McpServerConfig>,
    /// Configuration metadata
    pub metadata: McpConfigMetadata,
}

/// Configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfigMetadata {
    /// Configuration version
    pub version: String,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
    /// CLI version that created this config
    pub cli_version: String,
}

impl Default for McpConfigMetadata {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            last_updated: Utc::now(),
            cli_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl McpConfig {
    /// Load configuration from file
    pub(crate) async fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .await
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let mut config: McpConfig =
            serde_json::from_str(&content).with_context(|| "Failed to parse MCP configuration")?;

        // Update metadata
        config.metadata.last_updated = Utc::now();

        Ok(config)
    }

    /// Save configuration to file
    pub(crate) async fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize MCP configuration")?;

        fs::write(&config_path, content)
            .await
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        Ok(())
    }

    /// Get the configuration file path
    pub(crate) fn config_path() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow!("Could not determine config directory"))?;

        Ok(config_dir.join("icarus").join("mcp.json"))
    }

    /// Check if a server with the given name exists
    pub fn has_server(&self, name: &str) -> bool {
        self.servers.iter().any(|s| s.name == name)
    }

    /// Add a new server configuration
    pub fn add_server(&mut self, mut server: McpServerConfig) -> Result<()> {
        // Check for duplicate names
        if self.has_server(server.name.as_str()) {
            return Err(anyhow!("Server with name '{}' already exists", server.name));
        }

        // Update timestamps
        server.created_at = Utc::now();
        server.last_updated = Utc::now();

        self.servers.push(server);
        self.metadata.last_updated = Utc::now();

        Ok(())
    }

    /// Remove a server by name
    pub fn remove_server(&mut self, name: &str) -> Result<()> {
        let initial_len = self.servers.len();
        self.servers.retain(|s| s.name != name);

        if self.servers.len() == initial_len {
            return Err(anyhow!("Server with name '{}' not found", name));
        }

        self.metadata.last_updated = Utc::now();
        Ok(())
    }

    /// Update a server configuration
    pub fn update_server(
        &mut self,
        name: &str,
        mut updater: impl FnMut(&mut McpServerConfig),
    ) -> Result<()> {
        let server = self
            .servers
            .iter_mut()
            .find(|s| s.name == name)
            .ok_or_else(|| anyhow!("Server with name '{}' not found", name))?;

        updater(server);
        server.last_updated = Utc::now();
        self.metadata.last_updated = Utc::now();

        Ok(())
    }

    /// Get a server by name
    pub fn get_server(&self, name: &str) -> Option<&McpServerConfig> {
        self.servers.iter().find(|s| s.name == name)
    }

    /// Get all enabled servers
    pub fn enabled_servers(&self) -> Vec<&McpServerConfig> {
        self.servers.iter().filter(|s| s.enabled).collect()
    }

    /// Get servers for a specific client
    pub(crate) fn servers_for_client(&self, client: &str) -> Vec<&McpServerConfig> {
        self.servers.iter().filter(|s| s.client == client).collect()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Check for duplicate names
        let mut names = std::collections::HashSet::new();
        for server in &self.servers {
            if !names.insert(&server.name) {
                return Err(anyhow!("Duplicate server name: {}", server.name));
            }
        }

        // Validate each server
        for server in &self.servers {
            validate_server_config(server)?;
        }

        Ok(())
    }

    /// Cleanup old or invalid entries
    pub fn cleanup(&mut self) -> usize {
        let initial_count = self.servers.len();

        // Remove servers with invalid configurations
        self.servers
            .retain(|server| validate_server_config(server).is_ok());

        let removed_count = initial_count - self.servers.len();
        if removed_count > 0 {
            self.metadata.last_updated = Utc::now();
        }

        removed_count
    }

    /// Get configuration statistics
    pub fn stats(&self) -> McpConfigStats {
        let total_servers = self.servers.len();
        let enabled_servers = self.servers.iter().filter(|s| s.enabled).count();
        let disabled_servers = total_servers - enabled_servers;

        let mut clients = std::collections::HashMap::new();
        let mut networks = std::collections::HashMap::new();

        for server in &self.servers {
            *clients.entry(server.client.clone()).or_insert(0) += 1;
            *networks.entry(server.network.to_string()).or_insert(0) += 1;
        }

        McpConfigStats {
            total_servers,
            enabled_servers,
            disabled_servers,
            clients,
            networks,
        }
    }
}

/// Configuration statistics
#[derive(Debug)]
pub struct McpConfigStats {
    pub total_servers: usize,
    pub enabled_servers: usize,
    pub disabled_servers: usize,
    pub clients: std::collections::HashMap<String, usize>,
    pub networks: std::collections::HashMap<String, usize>,
}

fn validate_server_config(server: &McpServerConfig) -> Result<()> {
    // ServerName, CanisterId, and Network constructors already validate non-empty
    // and correct format, so name/canister_id/network checks are redundant.

    if server.url.is_empty() {
        return Err(anyhow!("URL cannot be empty"));
    }

    if server.client.is_empty() {
        return Err(anyhow!("Client cannot be empty"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CanisterId, Network, ServerName};

    fn create_test_server() -> McpServerConfig {
        McpServerConfig {
            name: ServerName::new("test-server").unwrap(),
            canister_id: CanisterId::new("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap(),
            network: Network::Local,
            url: "http://localhost:3000/mcp".to_string(),
            client: "claude-desktop".to_string(),
            port: Some(3000),
            enabled: true,
            created_at: Utc::now(),
            last_updated: Utc::now(),
        }
    }

    #[test]
    fn test_server_validation() {
        let server = create_test_server();
        assert!(validate_server_config(&server).is_ok());

        // Test invalid canister ID - validation now happens at construction
        assert!(CanisterId::new("invalid").is_err());

        // Test invalid network - validation now happens at construction
        assert!("invalid".parse::<Network>().is_err());

        // Test empty name - validation now happens at construction
        assert!(ServerName::new("").is_err());
    }

    #[test]
    fn test_config_operations() {
        let mut config = McpConfig::default();
        let server = create_test_server();

        // Test adding server
        assert!(config.add_server(server.clone()).is_ok());
        assert!(config.has_server("test-server"));

        // Test duplicate server
        assert!(config.add_server(server.clone()).is_err());

        // Test getting server
        assert!(config.get_server("test-server").is_some());
        assert!(config.get_server("nonexistent").is_none());

        // Test removing server
        assert!(config.remove_server("test-server").is_ok());
        assert!(!config.has_server("test-server"));

        // Test removing nonexistent server
        assert!(config.remove_server("nonexistent").is_err());
    }

    #[tokio::test]
    async fn test_config_serialization() {
        let mut config = McpConfig::default();
        let server = create_test_server();
        config.add_server(server).unwrap();

        // Test serialization
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("test-server"));
        assert!(json.contains("rdmx6-jaaaa-aaaaa-aaadq-cai"));

        // Test deserialization
        let deserialized: McpConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.servers.len(), 1);
        assert_eq!(deserialized.servers[0].name, "test-server");
    }

    #[test]
    fn test_config_stats() {
        let mut config = McpConfig::default();

        // Add multiple servers
        for i in 0..3 {
            let mut server = create_test_server();
            server.name = ServerName::new(format!("server-{}", i)).unwrap();
            server.enabled = i % 2 == 0; // Enable every other server
            if i == 1 {
                server.client = "claude-code".to_string();
                server.network = Network::Ic;
            }
            config.add_server(server).unwrap();
        }

        let stats = config.stats();
        assert_eq!(stats.total_servers, 3);
        assert_eq!(stats.enabled_servers, 2);
        assert_eq!(stats.disabled_servers, 1);
        assert_eq!(stats.clients.len(), 2);
        assert_eq!(stats.networks.len(), 2);
    }
}
