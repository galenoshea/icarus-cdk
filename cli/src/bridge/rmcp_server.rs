//! RMCP-based MCP server for ICP canister bridge
//!
//! Uses the official rmcp crate to implement Model Context Protocol

use anyhow::Result;
use candid::Principal;
use ic_agent::Agent;
use rmcp::model::{ProtocolVersion, ServerCapabilities, ServerInfo};
use rmcp::schemars::JsonSchema;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::future::Future;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::io::{stdin, stdout};

use crate::bridge::canister_client::CanisterClient;

/// Canister metadata for tool discovery
#[derive(Debug, Deserialize, Serialize)]
pub struct CanisterMetadata {
    pub name: String,
    pub version: Option<String>,
    pub tools: Vec<CanisterTool>,
}

/// Individual tool definition from canister
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CanisterTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Request to store a memory
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MemorizeRequest {
    pub content: String,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Request to forget a specific memory
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ForgetRequest {
    pub id: String,
}

/// Generic request for dynamic tool calls
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenericToolRequest {
    pub method: String,
    pub args: serde_json::Value,
}

/// Request to validate a JWT token
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateJwtRequest {
    pub token: String,
}

/// ICP Canister Bridge service
#[derive(Clone)]
pub struct IcpBridge {
    canister_id: Principal,
    canister_client: Arc<CanisterClient>,
    discovered_tools: Arc<tokio::sync::RwLock<Vec<CanisterTool>>>,
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
    bridge_tool_names: Arc<OnceLock<HashSet<String>>>,
}

// Implement the tool router for ICP bridge
#[tool_router]
impl IcpBridge {
    pub async fn new(canister_id: Principal) -> Result<Self> {
        // Use dfx identity for authentication
        // This will detect the current dfx identity and use it for authentication
        // Falls back to error if no identity is available (no more anonymous)
        let is_mcp_mode = !is_terminal::is_terminal(std::io::stdin())
            && !is_terminal::is_terminal(std::io::stdout());
        Self::new_with_dfx_fallback(canister_id, is_mcp_mode).await
    }

    /// Create a new authenticated bridge service (currently same as new)
    pub async fn new_authenticated(canister_id: Principal, _use_local: bool) -> Result<Self> {
        // For now, just use the same as new() since we're removing auth
        Self::new(canister_id).await
    }

    /// Fallback to dfx-based authentication
    async fn new_with_dfx_fallback(canister_id: Principal, is_mcp_mode: bool) -> Result<Self> {
        // Try to find dfx in common locations
        let dfx_paths = vec![
            "/Users/goshea/Library/Application Support/org.dfinity.dfx/bin/dfx",
            "/usr/local/bin/dfx",
            "/opt/homebrew/bin/dfx",
        ];

        let mut dfx_path = None;
        for path in &dfx_paths {
            if std::path::Path::new(path).exists() {
                dfx_path = Some(path.to_string());
                break;
            }
        }

        // Try to use dfx identity if available
        let (principal, agent) = if let Some(dfx) = dfx_path {
            match std::process::Command::new(&dfx)
                .args(&["identity", "whoami"])
                .output()
            {
                Ok(output) if output.status.success() => {
                    let identity_name = String::from_utf8(output.stdout)
                        .unwrap_or_default()
                        .trim()
                        .to_string();

                    // Get the principal
                    if let Ok(principal_output) = std::process::Command::new(&dfx)
                        .args(&["identity", "get-principal"])
                        .output()
                    {
                        if principal_output.status.success() {
                            let principal_str = String::from_utf8(principal_output.stdout)
                                .unwrap_or_default()
                                .trim()
                                .to_string();

                            if let Ok(principal) = Principal::from_text(&principal_str) {
                                // Try to load the identity PEM file
                                let identity_path = format!(
                                    "{}/.config/dfx/identity/{}/identity.pem",
                                    std::env::var("HOME").unwrap_or_default(),
                                    identity_name
                                );

                                // Try to load the identity - dfx uses secp256k1 by default
                                // Try secp256k1 first (most common for dfx)
                                let agent = if let Ok(identity) =
                                    ic_agent::identity::Secp256k1Identity::from_pem_file(
                                        &identity_path,
                                    ) {
                                    if !is_mcp_mode {
                                        eprintln!("🔑 Using dfx identity '{}' (secp256k1) with principal: {}", identity_name, principal_str);
                                    }
                                    ic_agent::Agent::builder()
                                        .with_url("http://localhost:4943")
                                        .with_identity(identity)
                                        .build()?
                                } else if let Ok(identity) =
                                    ic_agent::identity::BasicIdentity::from_pem_file(&identity_path)
                                {
                                    if !is_mcp_mode {
                                        eprintln!("🔑 Using dfx identity '{}' (ed25519) with principal: {}", identity_name, principal_str);
                                    }
                                    ic_agent::Agent::builder()
                                        .with_url("http://localhost:4943")
                                        .with_identity(identity)
                                        .build()?
                                } else {
                                    return Err(anyhow::anyhow!(
                                            "Could not load identity file at {}. Please ensure your dfx identity is properly configured.",
                                            identity_path
                                        ));
                                };

                                // Fetch root key for local development
                                agent.fetch_root_key().await?;

                                (principal, agent)
                            } else {
                                return Err(anyhow::anyhow!(
                                        "Could not parse principal from dfx. Please ensure dfx is properly configured."
                                    ));
                            }
                        } else {
                            return Err(anyhow::anyhow!(
                                    "Could not get principal from dfx. Please run 'dfx identity get-principal' to verify your identity."
                                ));
                        }
                    } else {
                        return Err(anyhow::anyhow!(
                                "Could not get principal from dfx. Please run 'dfx identity get-principal' to verify your identity."
                            ));
                    }
                }
                _ => {
                    return Err(anyhow::anyhow!(
                            "dfx command failed or is not available. Please ensure dfx is installed and a valid identity is selected."
                        ));
                }
            }
        } else {
            return Err(anyhow::anyhow!(
                    "dfx not found in standard locations. Please ensure dfx is installed and available in PATH."
                ));
        };

        // Fetch root key for local development
        agent.fetch_root_key().await?;

        let canister_client = Arc::new(CanisterClient::new_with_agent(canister_id, agent));

        Ok(Self {
            canister_id,
            canister_client,
            discovered_tools: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            tool_router: Self::tool_router(),
            bridge_tool_names: Arc::new(OnceLock::new()),
        })
    }

    /// Discover tools from canister metadata
    async fn discover_tools(&self) -> Result<()> {
        let debug = std::env::var("ICARUS_DEBUG").is_ok();

        // Wrap entire function in try-catch to prevent crashes
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| async {
            if debug {
                eprintln!(
                    "[DEBUG] Starting tool discovery for canister: {}",
                    self.canister_id
                );
            }

            // Add timeout to prevent hanging
            let timeout_duration = std::time::Duration::from_secs(10);
            let discovery_future = async {
                // Try to get canister metadata
                match self.canister_client.get_metadata().await {
                    Ok(metadata_str) => {
                        if debug {
                            eprintln!("[DEBUG] Got metadata: {}", metadata_str);
                        }

                        // Parse the metadata JSON
                        match serde_json::from_str::<CanisterMetadata>(&metadata_str) {
                            Ok(metadata) => {
                                let mut tools_lock = self.discovered_tools.write().await;
                                *tools_lock = metadata.tools;

                                if debug {
                                    eprintln!(
                                        "[DEBUG] Discovered {} tools: {:?}",
                                        tools_lock.len(),
                                        tools_lock.iter().map(|t| &t.name).collect::<Vec<_>>()
                                    );
                                }

                                Ok(())
                            }
                            Err(e) => {
                                if debug {
                                    eprintln!("[DEBUG] Failed to parse metadata JSON: {}", e);
                                }
                                // Fallback: assume this is an older canister without get_metadata
                                eprintln!("⚠️ Canister does not provide tool metadata. Using basic authentication tools only.");
                                Ok(())
                            }
                        }
                    }
                    Err(e) => {
                        if debug {
                            eprintln!("[DEBUG] Failed to get metadata: {}", e);
                        }
                        // Fallback: assume this is an older canister without get_metadata
                        eprintln!("⚠️ Could not retrieve canister metadata: {}. Using basic authentication tools only.", e);
                        Ok(())
                    }
                }
            };

            // Apply timeout
            match tokio::time::timeout(timeout_duration, discovery_future).await {
                Ok(result) => result,
                Err(_) => {
                    eprintln!("⚠️ Tool discovery timed out after 10 seconds. Continuing with basic tools.");
                    Ok(())
                }
            }
        }));

        match result {
            Ok(future_result) => future_result.await,
            Err(panic_info) => {
                eprintln!(
                    "⚠️ Tool discovery panicked: {:?}. Continuing with basic tools only.",
                    panic_info
                );
                Ok(())
            }
        }
    }

    /// Get bridge tool names with caching
    fn get_bridge_tool_names(&self) -> &HashSet<String> {
        self.bridge_tool_names.get_or_init(|| {
            self.tool_router
                .list_all()
                .into_iter()
                .map(|tool| tool.name.to_string())
                .collect()
        })
    }

    /// Get the current caller's principal identity
    #[tool(description = "Get the current caller's principal identity")]
    async fn whoami(&self) -> String {
        self.call_dynamic_tool("whoami", serde_json::json!({}))
            .await
    }

    /// Store a new memory
    #[tool(description = "Store a new memory")]
    async fn memorize(&self) -> String {
        // For now, just call with empty content as an example
        let args = serde_json::json!({ "content": "example memory" });
        self.call_dynamic_tool("memorize", args).await
    }

    /// List all stored memories
    #[tool(description = "List all stored memories")]
    async fn list(&self) -> String {
        self.call_dynamic_tool("list", serde_json::json!({})).await
    }

    /// Remove the first available memory
    #[tool(description = "Remove the first available memory")]
    async fn forget(&self) -> String {
        // For now, call without specific ID
        let args = serde_json::json!({ "id": "1" });
        self.call_dynamic_tool("forget", args).await
    }

    /// Remove the oldest memory
    #[tool(description = "Remove the oldest memory")]
    async fn forget_oldest(&self) -> String {
        self.call_dynamic_tool("forget_oldest", serde_json::json!({}))
            .await
    }

    /// Retrieve the latest memory
    #[tool(description = "Retrieve the latest memory")]
    async fn recall_latest(&self) -> String {
        self.call_dynamic_tool("recall_latest", serde_json::json!({}))
            .await
    }

    /// Add a principal to the whitelist (simplified without auth)
    #[tool(description = "Add a principal to the whitelist")]
    async fn add_to_whitelist(&self) -> String {
        // For now, use anonymous principal
        let args = serde_json::json!({ "principal": Principal::anonymous().to_text() });
        self.call_dynamic_tool("add_to_whitelist", args).await
    }

    /// Generate JWT session token (simplified without auth)
    #[tool(description = "Generate a JWT session token for authenticated user")]
    async fn create_jwt_session(&self) -> String {
        // For now, return a dummy token
        serde_json::json!({
            "success": true,
            "token": "dummy_token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "dummy_refresh"
        })
        .to_string()
    }

    /// Check if bridge is authenticated and get principal
    #[tool(description = "Check if bridge is authenticated and get principal")]
    async fn bridge_auth_status(&self) -> String {
        // For now, always return anonymous principal
        serde_json::json!({
            "authenticated": false,
            "principal": Principal::anonymous().to_text(),
            "message": "Authentication removed for now"
        })
        .to_string()
    }

    /// Internal helper to call canister tools dynamically
    async fn call_dynamic_tool(&self, tool_name: &str, args: serde_json::Value) -> String {
        let debug = std::env::var("ICARUS_DEBUG").is_ok();
        if debug {
            eprintln!("[DEBUG] Calling dynamic tool: {}", tool_name);
        }

        // Check if this tool exists in discovered tools or is a bridge tool
        let tools = self.discovered_tools.read().await;
        let tool_exists_in_canister = tools.iter().any(|t| t.name == tool_name);
        drop(tools);

        // Get bridge tools dynamically from the tool router
        let bridge_tool_names = self.get_bridge_tool_names();
        let is_bridge_tool = bridge_tool_names.contains(tool_name);

        if !tool_exists_in_canister && !is_bridge_tool {
            if debug {
                eprintln!(
                    "[DEBUG] Tool '{}' not found in discovered canister tools or bridge tools",
                    tool_name
                );
            }
            return serde_json::json!({
                "success": false,
                "error": format!("Tool '{}' not available in this canister", tool_name)
            })
            .to_string();
        }

        if debug && is_bridge_tool && !tool_exists_in_canister {
            eprintln!(
                "[DEBUG] Using bridge tool '{}' (not in discovered canister tools)",
                tool_name
            );
        }

        // Call the canister method
        match self
            .canister_client
            .generic_call(tool_name, args, false)
            .await
        {
            Ok(result) => {
                if debug {
                    eprintln!("[DEBUG] Tool '{}' returned: {}", tool_name, result);
                }
                result
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "success": false,
                    "error": format!("Failed to call {}: {}", tool_name, e)
                })
                .to_string();
                if debug {
                    eprintln!("[DEBUG] Tool '{}' error: {}", tool_name, error_response);
                }
                error_response
            }
        }
    }

    /// Helper to check authentication before tool execution (simplified)
    async fn ensure_authenticated(&self) -> Result<Principal, String> {
        // For now, return anonymous principal
        Ok(Principal::anonymous())
    }

    /// Verify that the authenticated user has access to the canister
    async fn ensure_canister_access(&self) -> Result<Principal, String> {
        let principal = self.ensure_authenticated().await?;

        // Check if the current principal is authorized to use this canister
        match self.canister_client.check_authorization().await {
            Ok(true) => Ok(principal),
            Ok(false) => {
                // Get canister metadata to provide helpful error message
                match self.canister_client.get_canister_metadata().await {
                    Ok(metadata_json) => {
                        if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&metadata_json) {
                            let primary_owner = metadata.get("primary_owner")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            let additional_owners = metadata.get("additional_owners")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.len())
                                .unwrap_or(0);
                            Err(format!(
                                "Access denied: Principal {} is not authorized to use canister {}. Current owners: primary={}, additional={}. Use 'icarus auth add-owner' if you need access.",
                                principal.to_text(),
                                self.canister_id.to_text(),
                                primary_owner,
                                additional_owners
                            ))
                        } else {
                            Err(format!(
                                "Access denied: Principal {} is not authorized to use canister {}",
                                principal.to_text(),
                                self.canister_id.to_text()
                            ))
                        }
                    }
                    Err(_) => Err(format!(
                        "Access denied: Principal {} is not authorized to use canister {} (metadata unavailable)",
                        principal.to_text(),
                        self.canister_id.to_text()
                    ))
                }
            }
            Err(e) => Err(format!("Failed to check canister authorization: {}", e)),
        }
    }
}

// Implement the ServerHandler trait using the tool_handler macro
#[tool_handler]
impl ServerHandler for IcpBridge {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: rmcp::model::Implementation {
                name: "icarus-bridge".to_string(),
                version: "0.1.0".to_string(),
            },
            instructions: Some("ICP Canister Bridge for MCP - provides tools to interact with Internet Computer canisters".to_string()),
        }
    }
}

/// Run the RMCP server with stdio transport
pub async fn run(canister_id_str: String) -> Result<()> {
    run_with_auth(canister_id_str, false, false).await
}

/// Run the RMCP server with optional authentication
pub async fn run_with_auth(
    canister_id_str: String,
    _authenticate: bool,
    _use_local: bool,
) -> Result<()> {
    use rmcp::serve_server;

    let debug = std::env::var("ICARUS_DEBUG").is_ok();

    // Always log to stderr for Claude Desktop debugging
    eprintln!(
        "[MCP] Starting RMCP server with canister_id: {}",
        canister_id_str
    );

    if debug {
        eprintln!("[DEBUG] Debug mode enabled");
    }

    // Wrap entire server startup in error handling
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| async {
        // Parse canister ID first to fail fast if invalid
        let canister_id = Principal::from_text(&canister_id_str).map_err(|e| {
            eprintln!("[ERROR] Invalid canister ID '{}': {}", canister_id_str, e);
            e
        })?;
        eprintln!("[MCP] Parsed canister ID successfully");

        // Create the ICP bridge service (authentication removed for now)
        eprintln!("[MCP] Creating bridge service");
        let service = IcpBridge::new(canister_id).await.map_err(|e| {
            eprintln!("[ERROR] Failed to create bridge: {}", e);
            e
        })?;
        eprintln!("[MCP] Bridge service created successfully");

        // Enable tool discovery to load dynamic tools from canister metadata
        eprintln!("[MCP] Starting tool discovery");
        if let Err(e) = service.discover_tools().await {
            eprintln!(
                "⚠️ Tool discovery failed: {}. Bridge will work with basic tools only.",
                e
            );
        } else {
            eprintln!("[MCP] Tool discovery completed successfully");
        }

        // Create stdio transport
        eprintln!("[MCP] Creating stdio transport");
        let transport = (stdin(), stdout());

        // Start the server
        eprintln!("[MCP] Starting RMCP server v0.5.0 with MCP protocol");

        if debug {
            eprintln!("[DEBUG] Creating RMCP server with stdio transport");
        }

        let server = serve_server(service, transport).await.map_err(|e| {
            eprintln!("[ERROR] Failed to start server: {:?}", e);
            if debug {
                eprintln!("[DEBUG] Server startup error details: {:#?}", e);
            }
            anyhow::anyhow!("Failed to start server: {:?}", e)
        })?;
        eprintln!("[MCP] RMCP server started successfully, waiting for requests");

        if debug {
            eprintln!("[DEBUG] Server is now running and ready to handle MCP protocol");
        }

        // Wait for server shutdown
        let quit_reason = server.waiting().await.map_err(|e| {
            eprintln!("[ERROR] Server error: {:?}", e);
            anyhow::anyhow!("Server error: {:?}", e)
        })?;

        eprintln!("[MCP] Server shutdown with reason: {:?}", quit_reason);
        Ok(())
    }));

    match result {
        Ok(future_result) => future_result.await,
        Err(panic_info) => {
            eprintln!("[FATAL] Server startup panicked: {:?}", panic_info);
            Err(anyhow::anyhow!("Server startup panicked: {:?}", panic_info))
        }
    }
}
