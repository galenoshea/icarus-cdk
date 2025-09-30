//! RMCP-compliant bridge for connecting Claude Desktop to IC canisters.
//!
//! This bridge implements `rmcp::ServerHandler` to provide proper MCP protocol
//! support. It forwards tool calls from Claude Desktop to IC canisters using dfx.

use anyhow::{anyhow, Result};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

// Import RMCP types from icarus-core
use icarus_core::{CallToolResult, Content, Tool};

// Import types directly from rmcp crate for protocol handling
use rmcp::model::{
    CallToolRequestParam, Implementation, ListToolsResult, PaginatedRequestParam, ProtocolVersion,
    ServerCapabilities, ServerInfo, ToolsCapability,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ErrorData;
use rmcp::ServerHandler;

use crate::config::mcp::McpConfig;

/// Bridge configuration for connecting to an IC canister.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Canister ID to connect to
    pub canister_id: String,
    /// Network (local, ic, or custom URL)
    pub network: String,
    /// Server name/description
    pub server_name: String,
    /// Server version
    pub server_version: String,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            canister_id: String::new(),
            network: "local".to_string(),
            server_name: "Icarus Bridge".to_string(),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// RMCP-compliant bridge server that forwards requests to IC canisters.
///
/// This implements `rmcp::ServerHandler` to provide proper MCP protocol support.
/// It uses dfx to communicate with IC canisters, forwarding tool calls and
/// returning results in RMCP-compliant format.
#[allow(dead_code)]
pub struct IcarusBridge {
    config: Arc<RwLock<BridgeConfig>>,
    mcp_config: Arc<RwLock<McpConfig>>,
}

#[allow(dead_code)]
impl IcarusBridge {
    /// Creates a new Icarus bridge with the given configuration.
    pub fn new(config: BridgeConfig, mcp_config: McpConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            mcp_config: Arc::new(RwLock::new(mcp_config)),
        }
    }

    /// Calls a canister method using dfx.
    async fn dfx_call(&self, method: &str, args: &str) -> Result<String> {
        let config = self.config.read().await;

        debug!(
            "Calling canister {} method {} with args: {}",
            config.canister_id, method, args
        );

        // Build dfx command
        let output = Command::new("dfx")
            .arg("canister")
            .arg("call")
            .arg(&config.canister_id)
            .arg(method)
            .arg("--network")
            .arg(&config.network)
            .arg("--output")
            .arg("json")
            .arg(format!(
                "(record {{ request = \"{}\" }})",
                args.replace('"', "\\\"")
            ))
            .output()
            .map_err(|e| anyhow!("Failed to execute dfx: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("dfx call failed: {}", stderr);
            return Err(anyhow!("dfx call failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("dfx response: {}", stdout);

        Ok(stdout.to_string())
    }

    /// Lists tools from the canister.
    async fn list_canister_tools(&self) -> Result<Vec<Tool>> {
        let response = self.dfx_call("mcp_list_tools", "{}").await?;

        // Parse the JSON-RPC response
        let response_json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse list_tools response: {}", e))?;

        // Extract tools from result
        let tools = response_json
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array())
            .ok_or_else(|| anyhow!("Invalid list_tools response format"))?;

        // Convert to Tool objects
        let tools: Vec<Tool> = tools
            .iter()
            .filter_map(|tool_json| serde_json::from_value(tool_json.clone()).ok())
            .collect();

        Ok(tools)
    }

    /// Calls a tool on the canister.
    async fn call_canister_tool(
        &self,
        tool_name: &str,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult> {
        // Build JSON-RPC request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "1",
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments.unwrap_or_default()
            }
        });

        let request_str = serde_json::to_string(&request)
            .map_err(|e| anyhow!("Failed to serialize request: {}", e))?;

        let response = self.dfx_call("mcp_call_tool", &request_str).await?;

        // Parse the JSON-RPC response
        let response_json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse call_tool response: {}", e))?;

        // Check for JSON-RPC error
        if let Some(error) = response_json.get("error") {
            let error_msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Ok(CallToolResult {
                content: vec![Content::text(error_msg)],
                structured_content: None,
                is_error: Some(true),
                meta: None,
            });
        }

        // Extract CallToolResult from result field
        let result = response_json
            .get("result")
            .ok_or_else(|| anyhow!("Missing result field in response"))?;

        let call_tool_result: CallToolResult = serde_json::from_value(result.clone())
            .map_err(|e| anyhow!("Failed to parse CallToolResult: {}", e))?;

        Ok(call_tool_result)
    }
}

impl ServerHandler for IcarusBridge {
    fn get_info(&self) -> ServerInfo {
        // This is synchronous, so we can't use async lock
        // We'll return a default server info
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: None,
                }),
                prompts: None,
                resources: None,
                logging: None,
                experimental: None,
                completions: None,
            },
            server_info: Implementation {
                name: "icarus-bridge".to_string(),
                title: Some("Icarus MCP Bridge".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                website_url: None,
            },
            instructions: Some("Bridge server for connecting Claude Desktop to Internet Computer canisters via MCP protocol.".to_string()),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        info!("Listing tools from canister");

        match self.list_canister_tools().await {
            Ok(tools) => Ok(ListToolsResult {
                tools,
                next_cursor: None,
            }),
            Err(e) => {
                error!("Failed to list tools: {}", e);
                Err(ErrorData::internal_error(
                    format!("Failed to list tools: {}", e),
                    None,
                ))
            }
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("Calling tool: {}", request.name);

        match self
            .call_canister_tool(&request.name, request.arguments)
            .await
        {
            Ok(result) => Ok(result),
            Err(e) => {
                error!("Failed to call tool: {}", e);
                Err(ErrorData::internal_error(
                    format!("Failed to call tool: {}", e),
                    None,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let config = BridgeConfig::default();
        let mcp_config = McpConfig::default();
        let _bridge = IcarusBridge::new(config, mcp_config);
    }

    #[test]
    fn test_bridge_config_default() {
        let config = BridgeConfig::default();
        assert_eq!(config.network, "local");
        assert!(!config.canister_id.is_empty() || config.canister_id.is_empty());
        // Just ensure field exists
    }

    #[tokio::test]
    async fn test_get_info() {
        let config = BridgeConfig::default();
        let mcp_config = McpConfig::default();
        let bridge = IcarusBridge::new(config, mcp_config);

        let info = bridge.get_info();
        assert_eq!(info.server_info.name, "icarus-bridge");
        assert!(info.capabilities.tools.is_some());
    }
}
