//! MCP protocol handling and translation

use async_trait::async_trait;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tracing::debug;

use crate::networking::{CanisterClient, CanisterMetadata, ToolMetadata};

/// Trait for MCP protocol operations
///
/// This trait defines the core operations that any MCP protocol handler must implement.
/// It enables dependency injection, testing with mock implementations, and extensibility.
#[async_trait]
pub trait McpProtocol: Send + Sync {
    /// Error type for protocol operations
    type Error: std::fmt::Display + Send + Sync + 'static;

    /// List all available tools
    async fn list_tools(&self) -> Result<Vec<JsonValue>, Self::Error>;

    /// Call a specific tool with arguments
    async fn call_tool(
        &self,
        tool_name: &str,
        arguments: JsonValue,
    ) -> Result<JsonValue, Self::Error>;

    /// Get server information
    fn get_server_info(&self) -> JsonValue;

    /// Get the metadata for this protocol handler
    fn metadata(&self) -> &CanisterMetadata;
}

/// Trait for tool conversion operations
pub trait ToolConverter {
    /// Convert a tool metadata to MCP JSON representation
    fn convert_tool(tool: &ToolMetadata) -> JsonValue;

    /// Convert multiple tools with optional filtering
    fn convert_tools(tools: &[ToolMetadata]) -> Vec<JsonValue> {
        tools.iter().map(Self::convert_tool).collect()
    }

    /// Convert tools with filtering predicate
    fn convert_tools_filtered<F>(tools: &[ToolMetadata], predicate: F) -> Vec<JsonValue>
    where
        F: Fn(&ToolMetadata) -> bool,
    {
        tools
            .iter()
            .filter(|tool| predicate(tool))
            .map(Self::convert_tool)
            .collect()
    }
}

/// Trait for canister communication abstraction
#[async_trait]
pub trait CanisterBackend: Send + Sync {
    /// Error type for canister operations
    type Error: std::fmt::Display + Send + Sync + 'static;

    /// Get canister metadata
    async fn get_metadata(&self) -> Result<CanisterMetadata, Self::Error>;

    /// Call a method on the canister
    async fn call_method(
        &self,
        method_name: &str,
        args: JsonValue,
        is_query: bool,
    ) -> Result<JsonValue, Self::Error>;

    /// Check if the current principal is authorized
    async fn check_authorization(&self) -> Result<bool, Self::Error>;

    /// Get the canister ID as text
    fn canister_id_text(&self) -> String;
}

/// MCP protocol handler that translates between MCP and ICP
///
/// This is a generic handler that can work with any backend that implements CanisterBackend.
#[derive(Debug)]
pub struct McpProtocolHandler<B = Arc<CanisterClient>> {
    backend: B,
    metadata: CanisterMetadata,
}

impl<B> McpProtocolHandler<B> {
    /// Create a new protocol handler with a backend
    pub fn with_backend(backend: B, metadata: CanisterMetadata) -> Self {
        Self { backend, metadata }
    }

    /// Get the backend reference
    pub fn backend(&self) -> &B {
        &self.backend
    }
}

// Implementation for the default CanisterClient backend
impl McpProtocolHandler<Arc<CanisterClient>> {
    /// Create a new protocol handler (backward compatibility)
    pub fn new(client: Arc<CanisterClient>, canister_metadata: CanisterMetadata) -> Self {
        Self {
            backend: client,
            metadata: canister_metadata,
        }
    }

    /// Get the canister client (backward compatibility)
    pub fn client(&self) -> &Arc<CanisterClient> {
        &self.backend
    }
}

// Implement CanisterBackend for Arc<CanisterClient>
#[async_trait]
impl CanisterBackend for Arc<CanisterClient> {
    type Error = anyhow::Error;

    async fn get_metadata(&self) -> Result<CanisterMetadata, Self::Error> {
        self.get_canister_metadata().await
    }

    async fn call_method(
        &self,
        method_name: &str,
        args: JsonValue,
        is_query: bool,
    ) -> Result<JsonValue, Self::Error> {
        CanisterClient::call_method(self, method_name, args, is_query).await
    }

    async fn check_authorization(&self) -> Result<bool, Self::Error> {
        CanisterClient::check_authorization(self).await
    }

    fn canister_id_text(&self) -> String {
        self.canister_id().to_text()
    }
}

// Implement ToolConverter for McpProtocolHandler
impl<B> ToolConverter for McpProtocolHandler<B> {
    fn convert_tool(tool: &ToolMetadata) -> JsonValue {
        serde_json::json!({
            "name": tool.name,
            "description": tool.description,
            "inputSchema": tool.input_schema,
            "title": tool.title,
            "icon": tool.icon
        })
    }
}

// Implement McpProtocol for McpProtocolHandler with CanisterBackend
#[async_trait]
impl<B> McpProtocol for McpProtocolHandler<B>
where
    B: CanisterBackend,
{
    type Error = B::Error;

    async fn list_tools(&self) -> Result<Vec<JsonValue>, Self::Error> {
        let metadata = self.backend.get_metadata().await?;
        let tools = Self::convert_tools(&metadata.tools);
        Ok(tools)
    }

    async fn call_tool(
        &self,
        tool_name: &str,
        arguments: JsonValue,
    ) -> Result<JsonValue, Self::Error> {
        debug!("Calling tool: {} with args: {}", tool_name, arguments);

        // For now, assume all calls are updates (not queries)
        let is_query = false;

        let result = self
            .backend
            .call_method(tool_name, arguments, is_query)
            .await?;
        debug!("Tool call successful for: {}", tool_name);
        Ok(result)
    }

    fn get_server_info(&self) -> JsonValue {
        serde_json::json!({
            "name": "icarus-mcp",
            "version": env!("CARGO_PKG_VERSION"),
            "canister_id": self.backend.canister_id_text(),
            "canister_name": self.metadata.name,
            "canister_title": self.metadata.title,
            "website_url": self.metadata.website_url,
            "tools_count": self.metadata.tools.len()
        })
    }

    fn metadata(&self) -> &CanisterMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::McpConfig;
    use candid::Principal;
    use serde_json::json;

    fn create_test_metadata() -> CanisterMetadata {
        CanisterMetadata {
            name: "test-canister".to_string(),
            version: Some("1.0.0".to_string()),
            tools: vec![
                ToolMetadata {
                    name: "echo".to_string(),
                    description: "Echo input".to_string(),
                    input_schema: json!({"type": "object", "properties": {"message": {"type": "string"}}}),
                    title: Some("Echo Tool".to_string()),
                    icon: Some("📢".to_string()),
                }
            ].into(),
            title: Some("Test Canister".to_string()),
            website_url: Some("https://example.com".to_string()),
        }
    }

    #[test]
    fn test_convert_tool() {
        let tool = ToolMetadata {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({"type": "object"}),
            title: Some("Test Tool".to_string()),
            icon: Some("🔧".to_string()),
        };

        let converted = McpProtocolHandler::<Arc<CanisterClient>>::convert_tool(&tool);

        assert_eq!(converted["name"], "test_tool");
        assert_eq!(converted["description"], "A test tool");
        assert_eq!(converted["title"], "Test Tool");
        assert_eq!(converted["icon"], "🔧");
    }

    #[test]
    fn test_get_server_info() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let _config = McpConfig::local(canister_id);
        let metadata = create_test_metadata();

        // We can't easily create a CanisterClient without IC connection,
        // but we can test the metadata structure
        assert_eq!(metadata.name, "test-canister");
        assert_eq!(metadata.tools.len(), 1);
        assert_eq!(metadata.tools[0].name, "echo");
    }

    #[test]
    fn test_tool_conversion_edge_cases() {
        // Test with minimal tool
        let minimal_tool = ToolMetadata {
            name: "minimal".to_string(),
            description: "".to_string(),
            input_schema: json!({}),
            title: None,
            icon: None,
        };

        let converted = McpProtocolHandler::<Arc<CanisterClient>>::convert_tool(&minimal_tool);
        assert_eq!(converted["name"], "minimal");
        assert_eq!(converted["description"], "");
        assert!(converted["title"].is_null());
        assert!(converted["icon"].is_null());
    }

    #[test]
    fn test_trait_implementations() {
        // Test that our traits work correctly

        let metadata = create_test_metadata();

        // Test ToolConverter trait
        let tools = &metadata.tools;
        let converted = McpProtocolHandler::<Arc<CanisterClient>>::convert_tools(tools);
        assert_eq!(converted.len(), 1);
        assert_eq!(converted[0]["name"], "echo");

        // Test filtering
        let filtered =
            McpProtocolHandler::<Arc<CanisterClient>>::convert_tools_filtered(tools, |tool| {
                tool.name.contains("echo")
            });
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0]["name"], "echo");

        let empty_filtered =
            McpProtocolHandler::<Arc<CanisterClient>>::convert_tools_filtered(tools, |tool| {
                tool.name.contains("nonexistent")
            });
        assert_eq!(empty_filtered.len(), 0);
    }
}
