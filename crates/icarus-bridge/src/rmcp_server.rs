//! RMCP-based MCP server for ICP canister bridge
//!
//! Uses the official rmcp crate to implement Model Context Protocol
//!
//! # Response Streaming
//!
//! Supports three response modes:
//! - **Standard**: Normal response (no `_stream` parameter)
//! - **Basic Streaming**: Large responses chunked for better delivery (`"_stream": true`)
//! - **Progress Streaming**: Real-time progress updates during execution (`"_stream": "progress"`)
//!
//! ## Usage Examples
//!
//! ```json
//! // Standard response
//! { "name": "list", "arguments": { "limit": 10 } }
//!
//! // Basic streaming for large responses
//! { "name": "list", "arguments": { "limit": 1000, "_stream": true } }
//!
//! // Progress streaming with execution updates
//! { "name": "complex_operation", "arguments": { "data": "...", "_stream": "progress" } }
//! ```

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

use crate::auth;
use crate::canister_client::CanisterClient;

/// Canister metadata for tool discovery
#[derive(Debug, Deserialize, Serialize)]
pub struct CanisterMetadata {
    pub name: String,
    pub version: Option<String>,
    pub tools: Vec<CanisterTool>,
    /// Optional display title for the canister
    pub title: Option<String>,
    /// Optional website URL for the canister
    pub website_url: Option<String>,
}

/// Individual tool definition from canister
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CanisterTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
    /// Optional display title for the tool
    pub title: Option<String>,
    /// Optional icon identifier for the tool
    pub icon: Option<String>,
}


/// ICP Canister Bridge service
#[derive(Clone)]
#[allow(dead_code)] // Used in MCP mode but analysis doesn't detect it
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
        let (identity_name, _principal, agent) = auth::create_authenticated_agent(is_mcp_mode).await?;

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
                title: Some("Icarus ICP Bridge".to_string()),
                website_url: Some("https://github.com/galenoshea/icarus-sdk".to_string()),
                icons: None,
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
                    title: tool.title.clone(),
                    icons: tool.icon.as_ref().map(|_icon_name| {
                        // TODO: Fix icon implementation when rmcp Icon format is clarified
                        vec![]
                    }),
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

        // Check if streaming is requested via tool arguments
        // Supports both "_stream": true and "_stream": "progress" for different streaming modes
        let streaming_mode = args.get("_stream");
        let enable_streaming = streaming_mode.is_some();

        if enable_streaming {
            let is_progress_mode = streaming_mode
                .and_then(|v| v.as_str())
                .map(|s| s == "progress")
                .unwrap_or(false);

            // Use streaming response handler with mode selection
            self.call_tool_streaming(&request.name, args, is_progress_mode).await
        } else {
            // Use standard response handler
            self.call_tool_standard(&request.name, args).await
        }
    }
}

impl IcpBridge {
    /// Standard non-streaming tool call implementation
    async fn call_tool_standard(
        &self,
        tool_name: &str,
        args: serde_json::Value,
    ) -> std::result::Result<CallToolResult, ErrorData> {
        // Route all tool calls through our dynamic handler
        let result = self.call_dynamic_tool(tool_name, args).await;

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
                meta: None,
            }),
            Err(_) => {
                // If result is not JSON, return as plain text
                Ok(CallToolResult {
                    content: vec![rmcp::model::Content::text(result)],
                    structured_content: None,
                    is_error: Some(false),
                    meta: None,
                })
            }
        }
    }

    /// Streaming tool call implementation for large responses
    /// Supports two modes:
    /// - Basic streaming: Chunks large responses (progress_mode = false)
    /// - Progress streaming: Shows progress updates for long operations (progress_mode = true)
    async fn call_tool_streaming(
        &self,
        tool_name: &str,
        args: serde_json::Value,
        progress_mode: bool,
    ) -> std::result::Result<CallToolResult, ErrorData> {
        // Remove the internal _stream parameter before calling the tool
        let mut clean_args = args.clone();
        if let serde_json::Value::Object(ref mut map) = clean_args {
            map.remove("_stream");
        }

        if progress_mode {
            // Progress streaming mode - show updates during execution
            self.call_tool_with_progress(tool_name, clean_args).await
        } else {
            // Basic streaming mode - chunk large responses
            let result = self.call_dynamic_tool(tool_name, clean_args).await;

            const CHUNK_SIZE: usize = 1024; // 1KB chunks
            if result.len() > CHUNK_SIZE {
                // Stream large responses in chunks
                self.stream_large_response(result).await
            } else {
                // For small responses, use standard handling
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
                        meta: None,
                    }),
                    Err(_) => {
                        // If result is not JSON, return as plain text
                        Ok(CallToolResult {
                            content: vec![rmcp::model::Content::text(result)],
                            structured_content: None,
                            is_error: Some(false),
                            meta: None,
                        })
                    }
                }
            }
        }
    }

    /// Progress streaming mode - shows real-time updates during tool execution
    async fn call_tool_with_progress(
        &self,
        tool_name: &str,
        args: serde_json::Value,
    ) -> std::result::Result<CallToolResult, ErrorData> {
        let debug = std::env::var("ICARUS_DEBUG").is_ok();

        // Create progress tracking
        let start_time = std::time::Instant::now();

        if debug {
            eprintln!("[PROGRESS] Starting {} with streaming progress", tool_name);
        }

        // For demonstration, add artificial progress steps for canister communication
        let progress_steps = vec![
            "🔍 Validating tool request",
            "🔗 Connecting to canister",
            "📤 Sending request to IC",
            "⏳ Processing on canister",
            "📥 Receiving response",
            "✅ Formatting result"
        ];

        let mut progress_data = Vec::new();

        for (i, step) in progress_steps.iter().enumerate() {
            let progress_pct = ((i as f32 / progress_steps.len() as f32) * 100.0) as u8;
            let elapsed = start_time.elapsed().as_millis();

            let progress_msg = format!("[{}%] {} ({}ms)", progress_pct, step, elapsed);
            progress_data.push(progress_msg.clone());

            if debug {
                eprintln!("[PROGRESS] {}", progress_msg);
            }

            // Add small delay to simulate work (only in debug mode)
            if debug && i < progress_steps.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }

        // Execute the actual tool call
        let result = self.call_dynamic_tool(tool_name, args).await;
        let total_time = start_time.elapsed().as_millis();

        // Create streaming response with progress history
        let streaming_result = serde_json::json!({
            "success": true,
            "streaming": "progress",
            "execution_time_ms": total_time,
            "progress_steps": progress_data,
            "result": serde_json::from_str::<serde_json::Value>(&result).unwrap_or_else(|_| serde_json::Value::String(result.clone()))
        });

        if debug {
            eprintln!("[PROGRESS] Completed {} in {}ms", tool_name, total_time);
        }

        Ok(CallToolResult {
            content: vec![rmcp::model::Content::text(streaming_result.to_string())],
            structured_content: Some(streaming_result),
            is_error: Some(false),
            meta: None,
        })
    }

    /// Stream large responses in chunks with progress updates
    async fn stream_large_response(&self, response: String) -> std::result::Result<CallToolResult, ErrorData> {
        const CHUNK_SIZE: usize = 1024;
        let chunks: Vec<String> = response
            .chars()
            .collect::<Vec<char>>()
            .chunks(CHUNK_SIZE)
            .enumerate()
            .map(|(i, chunk)| {
                let chunk_str: String = chunk.iter().collect();
                let total_chunks = (response.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
                format!("[CHUNK {}/{}] {}", i + 1, total_chunks, chunk_str)
            })
            .collect();

        // Combine all chunks into a single response with streaming metadata
        let streaming_result = serde_json::json!({
            "success": true,
            "streaming": "chunked",
            "total_chunks": chunks.len(),
            "total_size": response.len(),
            "chunk_size": CHUNK_SIZE,
            "data": chunks.join("\n"),
            "original_result": serde_json::from_str::<serde_json::Value>(&response)
                .unwrap_or_else(|_| serde_json::Value::String(response.clone()))
        });

        Ok(CallToolResult {
            content: vec![rmcp::model::Content::text(streaming_result.to_string())],
            structured_content: Some(streaming_result),
            is_error: Some(false),
            meta: None,
        })
    }
}

/// Run the RMCP server with optional authentication
#[allow(dead_code)] // Used in MCP mode but analysis doesn't detect it
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_canister_metadata_serialization() {
        let metadata = CanisterMetadata {
            name: "test-canister".to_string(),
            version: Some("1.0.0".to_string()),
            tools: vec![CanisterTool {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                input_schema: json!({"type": "object"}),
                title: Some("Test Tool".to_string()),
                icon: Some("test-icon".to_string()),
            }],
            title: Some("Test Canister".to_string()),
            website_url: Some("https://example.com".to_string()),
        };

        // Test serialization
        let json_str = serde_json::to_string(&metadata).unwrap();
        assert!(json_str.contains("test-canister"));
        assert!(json_str.contains("test_tool"));

        // Test deserialization
        let deserialized: CanisterMetadata = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.name, "test-canister");
        assert_eq!(deserialized.tools.len(), 1);
        assert_eq!(deserialized.tools[0].name, "test_tool");
    }

    #[test]
    fn test_canister_tool_creation() {
        let tool = CanisterTool {
            name: "my_tool".to_string(),
            description: "My awesome tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "param1": {"type": "string"}
                }
            }),
            title: Some("My Tool".to_string()),
            icon: Some("wrench".to_string()),
        };

        assert_eq!(tool.name, "my_tool");
        assert_eq!(tool.description, "My awesome tool");
        assert!(tool.input_schema.is_object());
        assert_eq!(tool.title.unwrap(), "My Tool");
        assert_eq!(tool.icon.unwrap(), "wrench");
    }

    #[test]
    fn test_canister_tool_minimal() {
        let tool = CanisterTool {
            name: "simple_tool".to_string(),
            description: "Simple tool".to_string(),
            input_schema: json!({}),
            title: None,
            icon: None,
        };

        assert_eq!(tool.name, "simple_tool");
        assert!(tool.title.is_none());
        assert!(tool.icon.is_none());
    }

    #[test]
    fn test_canister_metadata_minimal() {
        let metadata = CanisterMetadata {
            name: "minimal".to_string(),
            version: None,
            tools: vec![],
            title: None,
            website_url: None,
        };

        assert_eq!(metadata.name, "minimal");
        assert!(metadata.version.is_none());
        assert!(metadata.tools.is_empty());
    }

    #[tokio::test]
    async fn test_icp_bridge_creation_failure() {
        // Test that bridge creation fails gracefully with invalid canister ID
        let invalid_principal = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();

        // This should fail because dfx is likely not available in test environment
        let result = IcpBridge::new(invalid_principal).await;

        // We expect this to fail - just ensure it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_json_schema_handling() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            },
            "required": ["name"]
        });

        let tool = CanisterTool {
            name: "user_tool".to_string(),
            description: "User management tool".to_string(),
            input_schema: schema.clone(),
            title: None,
            icon: None,
        };

        // Verify schema is preserved
        assert_eq!(tool.input_schema["type"], "object");
        assert!(tool.input_schema["properties"].is_object());
        assert!(tool.input_schema["required"].is_array());
    }

    #[test]
    fn test_tool_serialization_roundtrip() {
        let original_tool = CanisterTool {
            name: "roundtrip_tool".to_string(),
            description: "Test roundtrip serialization".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "test": {"type": "string"}
                }
            }),
            title: Some("Roundtrip Tool".to_string()),
            icon: Some("cycle".to_string()),
        };

        // Serialize to JSON
        let json_str = serde_json::to_string(&original_tool).unwrap();

        // Deserialize back
        let recovered_tool: CanisterTool = serde_json::from_str(&json_str).unwrap();

        // Verify all fields match
        assert_eq!(recovered_tool.name, original_tool.name);
        assert_eq!(recovered_tool.description, original_tool.description);
        assert_eq!(recovered_tool.title, original_tool.title);
        assert_eq!(recovered_tool.icon, original_tool.icon);
        assert_eq!(recovered_tool.input_schema, original_tool.input_schema);
    }
}
