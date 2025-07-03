//! Canister endpoints for MCP protocol

use crate::state::STATE;
use ic_cdk_macros::{query, update};
use icarus_core::protocol::{IcarusMcpRequest, IcarusMcpResponse, IcarusMcpError, IcarusServerCapabilities};
use serde_json::{json, Value};

/// Main MCP request handler
#[update]
pub async fn icarus_mcp_request(request: IcarusMcpRequest) -> IcarusMcpResponse {
    let request_id = request.id.clone();
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
    let params: Value = serde_json::from_str(&request.params)
        .map_err(|e| format!("Failed to parse params: {}", e))?;
    
    match request.method.as_str() {
        "initialize" => handle_initialize(params).await,
        "tools/list" => handle_tools_list().await,
        "tools/call" => handle_tool_call(params).await,
        "resources/list" => handle_resources_list().await,
        "resources/read" => handle_resource_read(params).await,
        _ => Err(format!("Unknown method: {}", request.method)),
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
    
    // TODO: Actually execute the tool
    // For now, return a placeholder response
    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Executed tool '{}' with args: {}", tool_name, args)
        }]
    }).to_string())
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
#[query]
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
    let params: Value = serde_json::from_str(&request.params)
        .map_err(|e| format!("Failed to parse params: {}", e))?;
    
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