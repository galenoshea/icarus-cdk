//! RMCP-based MCP server for ICP canister bridge
//!
//! Uses the official rmcp crate to implement Model Context Protocol

use anyhow::Result;
use candid::Principal;
use ic_agent::Agent;
use rmcp::model::{
    CallToolRequestParam, CallToolResult, ListToolsResult, PaginatedRequestParam, ProtocolVersion,
    ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};
use serde::{Deserialize, Serialize};
use serde_json::Map as JsonObject;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{stdin, stdout};
use tokio::sync::RwLock;

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

/// ICP Canister Bridge service
#[derive(Clone)]
pub struct IcpBridge {
    canister_id: Principal,
    canister_client: Arc<RwLock<CanisterClient>>,
    tools: Arc<RwLock<Vec<CanisterTool>>>,
    current_identity: Arc<RwLock<Option<String>>>,
    agent_cache: Arc<RwLock<HashMap<String, Agent>>>,
}

// Implementation for ICP bridge
impl IcpBridge {
    pub async fn new(canister_id: Principal) -> Result<Self> {
        // Use dfx identity for authentication
        // This will detect the current dfx identity and use it for authentication
        // Falls back to error if no identity is available (no more anonymous)
        let is_mcp_mode = !is_terminal::is_terminal(std::io::stdin())
            && !is_terminal::is_terminal(std::io::stdout());
        Self::new_with_dfx_fallback(canister_id, is_mcp_mode).await
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
        let (identity_name, _principal, agent) = if let Some(ref dfx) = dfx_path {
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

                                (identity_name.clone(), principal, agent)
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

        let canister_client = Arc::new(RwLock::new(CanisterClient::new_with_agent(
            canister_id,
            agent.clone(),
        )));

        // Initialize agent cache with current identity
        let mut agent_cache = HashMap::new();
        agent_cache.insert(identity_name.clone(), agent);

        Ok(Self {
            canister_id,
            canister_client,
            tools: Arc::new(RwLock::new(Vec::new())),
            current_identity: Arc::new(RwLock::new(Some(identity_name))),
            agent_cache: Arc::new(RwLock::new(agent_cache)),
        })
    }

    /// Get the current dfx identity name
    fn get_current_dfx_identity(&self) -> Result<String> {
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

        if let Some(dfx) = dfx_path {
            match std::process::Command::new(&dfx)
                .args(&["identity", "whoami"])
                .output()
            {
                Ok(output) if output.status.success() => Ok(String::from_utf8(output.stdout)
                    .unwrap_or_default()
                    .trim()
                    .to_string()),
                _ => Err(anyhow::anyhow!("Failed to get current dfx identity")),
            }
        } else {
            Err(anyhow::anyhow!("dfx not found"))
        }
    }

    /// Load an agent for a specific identity
    async fn load_identity_agent(&self, identity_name: &str) -> Result<Agent> {
        let identity_path = format!(
            "{}/.config/dfx/identity/{}/identity.pem",
            std::env::var("HOME").unwrap_or_default(),
            identity_name
        );

        // Try to load the identity - try secp256k1 first (most common for dfx)
        let agent = if let Ok(identity) =
            ic_agent::identity::Secp256k1Identity::from_pem_file(&identity_path)
        {
            ic_agent::Agent::builder()
                .with_url("http://localhost:4943")
                .with_identity(identity)
                .build()?
        } else if let Ok(identity) =
            ic_agent::identity::BasicIdentity::from_pem_file(&identity_path)
        {
            ic_agent::Agent::builder()
                .with_url("http://localhost:4943")
                .with_identity(identity)
                .build()?
        } else {
            return Err(anyhow::anyhow!(
                "Could not load identity file at {}",
                identity_path
            ));
        };

        // Fetch root key for local development
        agent.fetch_root_key().await?;

        Ok(agent)
    }

    /// Ensure we're using the current dfx identity
    async fn ensure_current_identity(&self) -> Result<()> {
        // Get the current dfx identity
        let identity_name = match self.get_current_dfx_identity() {
            Ok(name) => name,
            Err(_) => {
                // If we can't get the current identity, just continue with the cached one
                return Ok(());
            }
        };

        let mut current = self.current_identity.write().await;

        // Check if identity has changed
        if current.as_ref() != Some(&identity_name) {
            // Identity changed, need to update agent
            let mut cache = self.agent_cache.write().await;

            // Check if we already have an agent for this identity
            if !cache.contains_key(&identity_name) {
                // Create new agent for this identity
                match self.load_identity_agent(&identity_name).await {
                    Ok(agent) => {
                        cache.insert(identity_name.clone(), agent);
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to load identity '{}': {}", identity_name, e);
                        return Err(e);
                    }
                }
            }

            // Update canister client with new agent
            let agent = cache.get(&identity_name).unwrap().clone();
            let mut client = self.canister_client.write().await;
            *client = CanisterClient::new_with_agent(self.canister_id, agent);

            *current = Some(identity_name.clone());

            eprintln!("🔑 Switched to identity: {}", identity_name);
        }

        Ok(())
    }

    /// Discover tools from canister metadata
    async fn discover_tools(&self) -> Result<()> {
        // Ensure we're using the current identity
        self.ensure_current_identity().await?;

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
                // Try to get canister tools list
                let client = self.canister_client.read().await;
                match client.list_tools().await {
                    Ok(metadata_str) => {
                        if debug {
                            eprintln!("[DEBUG] Got metadata: {}", metadata_str);
                        }

                        // Parse the metadata JSON
                        match serde_json::from_str::<CanisterMetadata>(&metadata_str) {
                            Ok(metadata) => {
                                let mut tools_lock = self.tools.write().await;
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

    /// Internal helper to call canister tools dynamically
    async fn call_dynamic_tool(&self, tool_name: &str, args: serde_json::Value) -> String {
        // Ensure we're using the current identity before making the call
        if let Err(e) = self.ensure_current_identity().await {
            return serde_json::json!({
                "success": false,
                "error": format!("Failed to verify identity: {}", e)
            })
            .to_string();
        }

        let debug = std::env::var("ICARUS_DEBUG").is_ok();
        if debug {
            eprintln!("[DEBUG] Calling dynamic tool: {}", tool_name);
        }

        // Check if this tool exists in discovered tools
        let tools = self.tools.read().await;
        let tool_exists = tools.iter().any(|t| t.name == tool_name);
        drop(tools);

        if !tool_exists {
            if debug {
                eprintln!(
                    "[DEBUG] Tool '{}' not found in discovered canister tools",
                    tool_name
                );
            }
            return serde_json::json!({
                "success": false,
                "error": format!("Tool '{}' not available in this canister", tool_name)
            })
            .to_string();
        }

        // Call the canister method
        let client = self.canister_client.read().await;
        match client.generic_call(tool_name, args, false).await {
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
}

// Implement the ServerHandler trait manually for custom tool listing
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

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListToolsResult, ErrorData> {
        // Return dynamically discovered tools instead of the single execute_tool
        let tools = self.tools.read().await;

        let tool_infos: Vec<Tool> = tools
            .iter()
            .map(|tool| {
                // Convert the input_schema Value to JsonObject
                let schema = if let serde_json::Value::Object(obj) = &tool.input_schema {
                    Arc::new(obj.clone())
                } else {
                    Arc::new(JsonObject::new())
                };

                Tool {
                    name: Cow::Owned(tool.name.clone()),
                    description: Some(Cow::Owned(tool.description.clone())),
                    input_schema: schema,
                    output_schema: None,
                    annotations: None,
                }
            })
            .collect();

        Ok(ListToolsResult {
            tools: tool_infos,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<CallToolResult, ErrorData> {
        // Convert Option<Map> to Value for arguments
        let args = request
            .arguments
            .map(|map| serde_json::Value::Object(map))
            .unwrap_or(serde_json::Value::Object(JsonObject::new()));

        // Route all tool calls through our dynamic handler
        let result = self.call_dynamic_tool(&request.name, args).await;

        // Parse the result and format as CallToolResult
        match serde_json::from_str::<serde_json::Value>(&result) {
            Ok(json_result) => Ok(CallToolResult {
                content: vec![rmcp::model::Content::text(json_result.to_string())],
                structured_content: Some(json_result.clone()),
                is_error: Some(
                    json_result
                        .get("success")
                        .and_then(|v| v.as_bool())
                        .map(|success| !success)
                        .unwrap_or(false),
                ),
                meta: Default::default(),
            }),
            Err(_) => {
                // If result is not JSON, return as plain text
                Ok(CallToolResult {
                    content: vec![rmcp::model::Content::text(result)],
                    structured_content: None,
                    is_error: Some(false),
                    meta: Default::default(),
                })
            }
        }
    }
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
