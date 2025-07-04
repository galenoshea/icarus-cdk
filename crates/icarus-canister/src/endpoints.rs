//! Canister endpoints for MCP protocol

use crate::state::STATE;
use icarus_core::protocol::{IcarusMcpRequest, IcarusMcpResponse, IcarusMcpError, IcarusServerCapabilities};
use candid::{CandidType, Deserialize};
use serde_json::{json, Value};
use std::cell::RefCell;

// Thread-local storage for tool handler
thread_local! {
    pub static TOOL_HANDLER: RefCell<Option<Box<dyn Fn(&str, &str) -> IcarusMcpResponse>>> = RefCell::new(None);
}

/// Set the tool handler for this canister
pub fn set_tool_handler<F>(handler: F) 
where
    F: Fn(&str, &str) -> IcarusMcpResponse + 'static
{
    TOOL_HANDLER.with(|h| {
        *h.borrow_mut() = Some(Box::new(handler));
    });
}

/// Main MCP request handler
pub async fn icarus_mcp_request(request: IcarusMcpRequest) -> IcarusMcpResponse {
    let request_id = request.id.clone();
    
    // Check if this is a direct tool call first
    let is_tool = STATE.with(|s| {
        if let Some(state) = s.borrow().as_ref() {
            state.tools.contains_key(&request.method)
        } else {
            false
        }
    });
    
    if is_tool {
        // Try the tool handler directly
        return TOOL_HANDLER.with(|h| {
            if let Some(handler) = h.borrow().as_ref() {
                let mut response = handler(&request.method, &request.params);
                response.id = request_id;
                response
            } else {
                IcarusMcpResponse {
                    result: None,
                    error: Some(IcarusMcpError {
                        code: -32603,
                        message: format!("Tool handler not initialized for: {}", request.method),
                        data: None,
                    }),
                    id: request_id,
                }
            }
        });
    }
    
    // Otherwise use standard MCP handling
    match handle_mcp_request(request).await {
        Ok(result) => IcarusMcpResponse {
            result: Some(result),
            error: None,
            id: request_id,
        },
        Err(e) => IcarusMcpResponse {
            result: None,
            error: Some(IcarusMcpError {
                code: -32603,
                message: e.to_string(),
                data: None,
            }),
            id: request_id,
        }
    }
}

/// Handle different MCP request methods
async fn handle_mcp_request(request: IcarusMcpRequest) -> Result<String, String> {
    // Handle empty params gracefully
    let params: Value = if request.params.is_empty() {
        json!({})
    } else {
        serde_json::from_str(&request.params)
            .map_err(|e| format!("Failed to parse params: {}", e))?
    };
    
    match request.method.as_str() {
        "initialize" => handle_initialize(params).await,
        "tools/list" => handle_tools_list().await,
        "tools/call" => handle_tool_call(params).await,
        "resources/list" => handle_resources_list().await,
        "resources/read" => handle_resource_read(params).await,
        method => {
            // Check if this is a direct tool call
            let is_tool = STATE.with(|s| {
                if let Some(state) = s.borrow().as_ref() {
                    state.tools.contains_key(&method.to_string())
                } else {
                    false
                }
            });
            
            if is_tool {
                // Try to dispatch to the tool handler
                TOOL_HANDLER.with(|h| {
                    if let Some(handler) = h.borrow().as_ref() {
                        let response = handler(method, &request.params);
                        // Convert response to JSON string
                        match serde_json::to_string(&response) {
                            Ok(json_str) => Ok(json_str),
                            Err(e) => Err(format!("Failed to serialize response: {}", e))
                        }
                    } else {
                        Err(format!("Tool handler not initialized for: {}", method))
                    }
                })
            } else {
                Err(format!("Unknown method: {}", method))
            }
        }
    }
}

/// Handle initialization request
async fn handle_initialize(_params: Value) -> Result<String, String> {
    Ok(json!({
        "protocolVersion": "1.0",
        "capabilities": {
            "tools": {},
            "resources": {}
        }
    }).to_string())
}

/// Handle tools/list request
async fn handle_tools_list() -> Result<String, String> {
    STATE.with(|s| {
        let state = s.borrow();
        if let Some(state) = state.as_ref() {
            let tools: Vec<Value> = state.tools.iter()
                .map(|(name, _tool_state)| {
                    json!({
                        "name": name,
                        "description": format!("{} tool", name),
                        "inputSchema": {
                            "type": "object",
                            "properties": {},
                            "required": []
                        }
                    })
                })
                .collect();
            
            Ok(json!({ "tools": tools }).to_string())
        } else {
            Err("State not initialized".to_string())
        }
    })
}

/// Handle tools/call request
async fn handle_tool_call(params: Value) -> Result<String, String> {
    let tool_name = params.get("name")
        .and_then(|n| n.as_str())
        .ok_or("Missing tool name")?;
    
    let args = params.get("arguments")
        .cloned()
        .unwrap_or(json!({}));
    
    // Use the tool handler if available
    TOOL_HANDLER.with(|h| {
        if let Some(handler) = h.borrow().as_ref() {
            let response = handler(tool_name, &serde_json::to_string(&args).unwrap_or_else(|_| "{}".to_string()));
            if let Some(result) = response.result {
                Ok(result)
            } else if let Some(error) = response.error {
                Err(error.message)
            } else {
                Ok(json!({
                    "content": [{
                        "type": "text", 
                        "text": "Tool executed successfully"
                    }]
                }).to_string())
            }
        } else {
            // Fallback response
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Executed tool '{}' with args: {}", tool_name, args)
                }]
            }).to_string())
        }
    })
}

/// Handle resources/list request
async fn handle_resources_list() -> Result<String, String> {
    STATE.with(|s| {
        let state = s.borrow();
        if let Some(state) = state.as_ref() {
            let resources: Vec<Value> = state.resources.iter()
                .map(|(uri, _resource_state)| {
                    json!({
                        "uri": uri,
                        "name": uri,
                        "mimeType": "text/plain"
                    })
                })
                .collect();
            
            Ok(json!({ "resources": resources }).to_string())
        } else {
            Err("State not initialized".to_string())
        }
    })
}

/// Handle resources/read request
async fn handle_resource_read(params: Value) -> Result<String, String> {
    let uri = params.get("uri")
        .and_then(|u| u.as_str())
        .ok_or("Missing resource URI")?;
    
    // TODO: Actually read the resource
    // For now, return a placeholder response
    Ok(json!({
        "contents": [{
            "uri": uri,
            "mimeType": "text/plain",
            "text": format!("Content of resource: {}", uri)
        }]
    }).to_string())
}

/// Query server capabilities
pub fn icarus_capabilities() -> IcarusServerCapabilities {
    crate::state::IcarusCanisterState::with(|state| state.capabilities())
}

/// Main MCP request handler with custom tool registry
pub async fn icarus_mcp_request_with_registry<F, Fut>(
    request: IcarusMcpRequest,
    tool_executor: F,
) -> IcarusMcpResponse
where
    F: FnOnce(&str, Value) -> Fut,
    Fut: std::future::Future<Output = Result<Value, icarus_core::error::ToolError>>,
{
    let request_id = request.id.clone();
    match handle_mcp_request_with_registry(request, tool_executor).await {
        Ok(result) => IcarusMcpResponse {
            result: Some(result),
            error: None,
            id: request_id,
        },
        Err(e) => IcarusMcpResponse {
            result: None,
            error: Some(IcarusMcpError {
                code: -32603,
                message: e.to_string(),
                data: None,
            }),
            id: request_id,
        }
    }
}

/// Handle different MCP request methods with custom tool registry
async fn handle_mcp_request_with_registry<F, Fut>(
    request: IcarusMcpRequest,
    tool_executor: F,
) -> Result<String, String>
where
    F: FnOnce(&str, Value) -> Fut,
    Fut: std::future::Future<Output = Result<Value, icarus_core::error::ToolError>>,
{
    // Handle empty params gracefully
    let params: Value = if request.params.is_empty() {
        json!({})
    } else {
        serde_json::from_str(&request.params)
            .map_err(|e| format!("Failed to parse params: {}", e))?
    };
    
    match request.method.as_str() {
        "initialize" => handle_initialize(params).await,
        "tools/list" => handle_tools_list().await,
        "tools/call" => handle_tool_call_with_registry(params, tool_executor).await,
        "resources/list" => handle_resources_list().await,
        "resources/read" => handle_resource_read(params).await,
        _ => Err(format!("Unknown method: {}", request.method)),
    }
}

/// Handle tools/call request with custom registry
async fn handle_tool_call_with_registry<F, Fut>(
    params: Value,
    tool_executor: F,
) -> Result<String, String>
where
    F: FnOnce(&str, Value) -> Fut,
    Fut: std::future::Future<Output = Result<Value, icarus_core::error::ToolError>>,
{
    let tool_name = params.get("name")
        .and_then(|n| n.as_str())
        .ok_or("Missing tool name")?;
    
    let args = params.get("arguments")
        .cloned()
        .unwrap_or(json!({}));
    
    match tool_executor(tool_name, args).await {
        Ok(result) => Ok(json!({
            "content": [{
                "type": "text",
                "text": result.to_string()
            }]
        }).to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// HTTP request type for canister HTTP gateway
#[derive(CandidType, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// HTTP response type for canister HTTP gateway
#[derive(CandidType)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Handle HTTP requests from the IC HTTP gateway
pub fn http_request(req: HttpRequest) -> HttpResponse {
    let _path = req.url.as_str();
    
    // Simple HTML response showing canister info
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Icarus MCP Server</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        .container {{
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            margin-bottom: 10px;
        }}
        .info {{
            background: #e7f3ff;
            padding: 15px;
            border-radius: 5px;
            border-left: 4px solid #2196F3;
            margin: 20px 0;
        }}
        code {{
            background: #f0f0f0;
            padding: 2px 5px;
            border-radius: 3px;
            font-family: "Courier New", monospace;
        }}
        .endpoint {{
            margin: 10px 0;
            padding: 10px;
            background: #f9f9f9;
            border-radius: 5px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Icarus MCP Server</h1>
        <p>This is an MCP (Model Context Protocol) server running on the Internet Computer.</p>
        
        <div class="info">
            <strong>Canister ID:</strong> <code>{}</code><br>
            <strong>Status:</strong> <span style="color: green;">✓ Running</span>
        </div>
        
        <h2>Available Endpoints</h2>
        <div class="endpoint">
            <strong>MCP Request:</strong><br>
            <code>POST /icarus_mcp_request</code><br>
            <small>Main endpoint for MCP protocol communication</small>
        </div>
        
        <div class="endpoint">
            <strong>Capabilities:</strong><br>
            <code>GET /icarus_capabilities</code><br>
            <small>Query server capabilities and available tools</small>
        </div>
        
        <h2>Connect with Claude Desktop</h2>
        <p>To use this MCP server with Claude Desktop, run:</p>
        <code>icarus connect --canister-id {}</code>
        
        <hr style="margin-top: 40px; border: none; border-top: 1px solid #eee;">
        <p style="text-align: center; color: #666; font-size: 14px;">
            Powered by <a href="https://icarus.dev" style="color: #2196F3;">Icarus SDK</a>
        </p>
    </div>
</body>
</html>"#,
        ic_cdk::id().to_text(),
        ic_cdk::id().to_text()
    );
    
    HttpResponse {
        status_code: 200,
        headers: vec![
            ("Content-Type".to_string(), "text/html; charset=UTF-8".to_string()),
            ("Cache-Control".to_string(), "no-cache".to_string()),
        ],
        body: html.into_bytes(),
    }
}