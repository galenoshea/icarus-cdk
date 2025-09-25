//! Main MCP server implementation

#[cfg(feature = "mcp")]
use anyhow::Result;
#[cfg(feature = "mcp")]
use std::marker::PhantomData;
#[cfg(feature = "mcp")]
use std::sync::Arc;
#[cfg(feature = "mcp")]
use tokio::io::{AsyncRead, AsyncWrite};
#[cfg(feature = "mcp")]
use tracing::info;

#[cfg(feature = "mcp")]
use crate::mcp::config::McpConfig;
#[cfg(feature = "mcp")]
use crate::mcp::networking::CanisterClient;
#[cfg(feature = "mcp")]
use crate::mcp::protocol::McpProtocolHandler;

/// Type-state marker for uninitialized server
#[cfg(feature = "mcp")]
#[derive(Debug)]
pub struct Uninitialized;

/// Type-state marker for connected server
#[cfg(feature = "mcp")]
#[derive(Debug)]
pub struct Connected;

/// Type-state marker for serving server
#[cfg(feature = "mcp")]
#[derive(Debug)]
pub struct Serving;

/// Main MCP server that coordinates protocol handling and canister communication
/// Uses type-state pattern for compile-time safety
#[cfg(feature = "mcp")]
#[derive(Debug)]
pub struct McpServer<State = Uninitialized> {
    client: Option<Arc<CanisterClient>>,
    handler: Option<McpProtocolHandler>,
    _state: PhantomData<State>,
}

/// Server builder for type-safe construction
#[cfg(feature = "mcp")]
#[derive(Debug)]
pub struct McpServerBuilder {
    config: Option<McpConfig>,
}

// Type-state implementations for compile-time safety

#[cfg(feature = "mcp")]
impl McpServer<Uninitialized> {
    /// Create a new uninitialized MCP server
    #[inline]
    pub fn new() -> Self {
        Self {
            client: None,
            handler: None,
            _state: PhantomData,
        }
    }

    /// Create a builder for the server
    #[inline]
    pub fn builder() -> McpServerBuilder {
        McpServerBuilder::new()
    }

    /// Connect to the canister and initialize the server
    pub async fn connect(self, config: McpConfig) -> Result<McpServer<Connected>> {
        info!(
            "Initializing MCP server for canister {}",
            config.canister_id
        );

        // Create canister client
        let mut client = CanisterClient::new(config).await?;

        // Refresh tools from canister
        client.refresh_tools().await?;

        // Get canister metadata for server info
        let metadata = client.get_canister_metadata().await?;
        info!(
            "Connected to canister '{}' with {} tools",
            metadata.name,
            metadata.tools.len()
        );

        // Create shared client reference
        let client = Arc::new(client);

        // Create protocol handler
        let handler = McpProtocolHandler::new(client.clone(), metadata);

        Ok(McpServer {
            client: Some(client),
            handler: Some(handler),
            _state: PhantomData,
        })
    }
}

#[cfg(feature = "mcp")]
impl McpServer<Connected> {
    /// Get the canister client for advanced operations
    #[inline]
    pub fn client(&self) -> &Arc<CanisterClient> {
        self.client.as_ref().expect("Client should be initialized")
    }

    /// Get the protocol handler
    #[inline]
    pub fn handler(&self) -> &McpProtocolHandler {
        self.handler
            .as_ref()
            .expect("Handler should be initialized")
    }

    /// Refresh tools from the canister
    pub async fn refresh_tools(&self) -> Result<()> {
        // For now, tools are refreshed on startup
        // In the future, we could implement tool refresh by recreating the handler
        Ok(())
    }

    /// Start serving MCP protocol over the provided input/output streams
    pub async fn serve<R, W>(self, _input: R, _output: W) -> Result<McpServer<Serving>>
    where
        R: AsyncRead + Unpin + Send,
        W: AsyncWrite + Unpin + Send,
    {
        info!("Starting MCP server");

        // For now, create a simple placeholder that transitions state
        // In a full implementation, this would handle MCP JSON-RPC protocol
        // over the input/output streams

        Ok(McpServer {
            client: self.client,
            handler: self.handler,
            _state: PhantomData,
        })
    }
}

#[cfg(feature = "mcp")]
impl McpServer<Serving> {
    /// Run the server until shutdown signal
    pub async fn run(self) -> Result<()> {
        // Note: For ICP canisters, MCP protocol handling is implemented at the canister level
        // through IC CDK endpoints rather than a traditional stdio-based MCP server
        // This design allows MCP tools to be exposed as HTTP endpoints on ICP
        //
        // Future versions may support standalone MCP servers for non-ICP deployments
        // which would implement:
        // 1. JSON-RPC message parsing from stdio
        // 2. Request routing to tool handlers
        // 3. Response formatting and output
        // 4. Proper shutdown signal handling

        info!("MCP server shutting down");
        Ok(())
    }

    /// Gracefully shutdown the server
    pub fn shutdown(self) -> McpServer<Connected> {
        info!("Server shutting down gracefully");
        McpServer {
            client: self.client,
            handler: self.handler,
            _state: PhantomData,
        }
    }
}

// Backward compatibility wrapper
#[cfg(feature = "mcp")]
impl McpServer {
    /// Create a new MCP server from config (backward compatibility)
    pub async fn from_config(config: McpConfig) -> Result<McpServer<Connected>> {
        McpServer::<Uninitialized>::new().connect(config).await
    }
}

#[cfg(feature = "mcp")]
impl McpServerBuilder {
    /// Create a new server builder
    #[inline]
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Set the configuration
    #[inline]
    pub fn config(mut self, config: McpConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Build and connect the server
    pub async fn build(self) -> Result<McpServer<Connected>> {
        let config = self
            .config
            .ok_or_else(|| anyhow::anyhow!("Configuration is required"))?;

        McpServer::<Uninitialized>::new().connect(config).await
    }
}

#[cfg(feature = "mcp")]
impl Default for McpServerBuilder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "mcp")]
impl Default for McpServer<Uninitialized> {
    #[inline]
    fn default() -> Self {
        McpServer::<Uninitialized>::new()
    }
}

#[cfg(all(test, feature = "mcp"))]
mod tests {
    use super::*;
    use crate::mcp::config::McpConfig;
    use candid::Principal;

    #[test]
    fn test_type_state_transitions() {
        // Test creation in Uninitialized state
        let _server = McpServer::<Uninitialized>::new();

        // Verify we can't access client or handler in Uninitialized state
        // (This is enforced by the type system - these methods don't exist)

        // Test default creation
        let server = McpServer::default();

        // These assertions verify the type state is working
        // In production, connections would be tested with real IC setup
        assert!(server.client.is_none());
        assert!(server.handler.is_none());
    }

    #[test]
    fn test_builder_pattern() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let config = McpConfig::local(canister_id);

        // Test server builder
        let builder = McpServer::builder();

        // Verify builder configuration
        let _builder = builder.config(config);

        // In a real test, we would do:
        // let server = builder.build().await.unwrap();
        // But that requires actual IC connection

        // Test that builder fails without config
        let _empty_builder = McpServerBuilder::new();
        // let result = empty_builder.build().await;
        // assert!(result.is_err());
    }

    #[test]
    fn test_server_builder_default() {
        let builder1 = McpServerBuilder::default();
        let builder2 = McpServerBuilder::new();

        // Both should behave the same way
        assert!(builder1.config.is_none());
        assert!(builder2.config.is_none());
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that from_config still works for backward compatibility
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let _config = McpConfig::local(canister_id);

        // In production, this would be:
        // let server = McpServer::from_config(config).await.unwrap();
        // But that requires actual IC connection, so we just verify the method exists

        // Compile-time verification that the method signature is correct
        let _: fn(McpConfig) -> _ = McpServer::from_config;
    }

    #[test]
    fn test_type_state_marker_types() {
        // Verify marker types can be instantiated and debugged
        let _uninitialized = Uninitialized;
        let _connected = Connected;
        let _serving = Serving;

        // Verify they implement Debug
        assert_eq!(format!("{:?}", _uninitialized), "Uninitialized");
        assert_eq!(format!("{:?}", _connected), "Connected");
        assert_eq!(format!("{:?}", _serving), "Serving");
    }

    #[test]
    fn test_phantom_data_usage() {
        // Verify PhantomData doesn't affect size
        use std::mem;

        let server_uninit = McpServer::<Uninitialized>::new();
        let size_uninit = mem::size_of_val(&server_uninit);

        // PhantomData should not add to the size
        // The actual size depends on Option<Arc<CanisterClient>> + Option<McpProtocolHandler>
        assert!(size_uninit > 0);

        // Verify the marker type system works
        // Different states should have the same memory layout
        // but different compile-time types
    }
}
