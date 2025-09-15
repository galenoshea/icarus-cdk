//! Builder pattern for creating and configuring MCP bridges

use anyhow::Result;
use candid::Principal;

/// Builder for creating MCP bridge configurations
///
/// Provides a fluent interface for configuring bridge settings before starting.
///
/// # Example
///
/// ```no_run
/// use icarus_bridge::BridgeBuilder;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let bridge = BridgeBuilder::new()
///         .canister_id("rdmx6-jaaaa-aaaah-qcaiq-cai")
///         .with_authentication(true)
///         .with_local_network(true)
///         .build()
///         .await?;
///
///     bridge.start().await?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct BridgeBuilder {
    canister_id: Option<String>,
    authenticate: bool,
    use_local: bool,
}

impl Default for BridgeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BridgeBuilder {
    /// Create a new bridge builder with default settings
    pub fn new() -> Self {
        Self {
            canister_id: None,
            authenticate: true,
            use_local: true,
        }
    }

    /// Set the canister ID to connect to
    pub fn canister_id<S: Into<String>>(mut self, id: S) -> Self {
        self.canister_id = Some(id.into());
        self
    }

    /// Enable or disable authentication
    ///
    /// When enabled, the bridge will use dfx identity for authentication.
    /// When disabled, anonymous authentication will be used.
    pub fn with_authentication(mut self, auth: bool) -> Self {
        self.authenticate = auth;
        self
    }

    /// Enable or disable local network mode
    ///
    /// When enabled, connects to local IC network (http://localhost:4943).
    /// When disabled, connects to mainnet.
    pub fn with_local_network(mut self, local: bool) -> Self {
        self.use_local = local;
        self
    }

    /// Build the bridge configuration
    ///
    /// This validates the configuration and creates a `Bridge` instance
    /// ready to be started.
    pub async fn build(self) -> Result<Bridge> {
        let canister_id = self
            .canister_id
            .ok_or_else(|| anyhow::anyhow!("Canister ID is required"))?;

        // Validate canister ID format
        Principal::from_text(&canister_id)
            .map_err(|e| anyhow::anyhow!("Invalid canister ID '{}': {}", canister_id, e))?;

        Ok(Bridge {
            canister_id,
            authenticate: self.authenticate,
            use_local: self.use_local,
        })
    }
}

/// A configured MCP bridge ready to start
#[derive(Debug, Clone)]
pub struct Bridge {
    canister_id: String,
    authenticate: bool,
    use_local: bool,
}

impl Bridge {
    /// Start the bridge and begin accepting MCP connections
    ///
    /// This method runs indefinitely, handling MCP protocol messages
    /// and translating them to canister calls.
    pub async fn start(self) -> Result<()> {
        crate::rmcp_server::run_with_auth(self.canister_id, self.authenticate, self.use_local).await
    }

    /// Get the canister ID this bridge is configured for
    pub fn canister_id(&self) -> &str {
        &self.canister_id
    }

    /// Check if authentication is enabled
    pub fn is_authenticated(&self) -> bool {
        self.authenticate
    }

    /// Check if local network mode is enabled
    pub fn is_local_network(&self) -> bool {
        self.use_local
    }
}
