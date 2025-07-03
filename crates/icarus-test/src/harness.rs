//! Test harness for running integration tests

use icarus_core::protocol::{IcarusMcpRequest, IcarusMcpResponse, IcarusMcpError};
use icarus_core::tool::IcarusTool;
use icarus_core::resource::IcarusResource;
use std::collections::HashMap;
use serde_json;

/// Test harness for running MCP server tests
pub struct TestHarness {
    tools: HashMap<String, Box<dyn IcarusTool>>,
    resources: HashMap<String, Box<dyn IcarusResource>>,
}

impl TestHarness {
    /// Create a new test harness
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            resources: HashMap::new(),
        }
    }
    
    /// Register a tool with the harness
    pub fn register_tool(&mut self, tool: Box<dyn IcarusTool>) {
        let info = tool.info();
        self.tools.insert(info.name, tool);
    }
    
    /// Register a resource with the harness
    pub fn register_resource(&mut self, resource: Box<dyn IcarusResource>) {
        let info = resource.info();
        self.resources.insert(info.uri, resource);
    }
    
    /// Execute a request in the test harness
    pub async fn execute(&self, request: IcarusMcpRequest) -> IcarusMcpResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request),
            "tools/list" => self.handle_tools_list(request),
            "tools/call" => self.handle_tool_call(request).await,
            "resources/list" => self.handle_resources_list(request),
            "resources/read" => self.handle_resource_read(request).await,
            _ => IcarusMcpResponse {
                id: request.id,
                result: None,
                error: Some(IcarusMcpError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            },
        }
    }
    
    fn handle_initialize(&self, request: IcarusMcpRequest) -> IcarusMcpResponse {
        IcarusMcpResponse {
            id: request.id,
            result: Some(serde_json::to_string(&serde_json::json!({
                "protocolVersion": "1.0.0",
                "serverInfo": {
                    "name": "icarus-test-harness",
                    "version": "0.1.0"
                },
                "capabilities": {
                    "tools": !self.tools.is_empty(),
                    "resources": !self.resources.is_empty()
                }
            })).unwrap()),
            error: None,
        }
    }
    
    fn handle_tools_list(&self, request: IcarusMcpRequest) -> IcarusMcpResponse {
        let tools: Vec<_> = self.tools.values()
            .map(|tool| tool.to_rmcp_tool())
            .collect();
            
        IcarusMcpResponse {
            id: request.id,
            result: Some(serde_json::to_string(&serde_json::json!({
                "tools": tools
            })).unwrap()),
            error: None,
        }
    }
    
    async fn handle_tool_call(&self, request: IcarusMcpRequest) -> IcarusMcpResponse {
        // Parse params from JSON string
        let params: serde_json::Value = match serde_json::from_str(&request.params) {
            Ok(p) => p,
            Err(_) => return IcarusMcpResponse {
                id: request.id,
                result: None,
                error: Some(IcarusMcpError {
                    code: -32602,
                    message: "Invalid params".to_string(),
                    data: None,
                }),
            },
        };
        
        let tool_name = match params.get("name").and_then(|n| n.as_str()) {
            Some(name) => name,
            None => return IcarusMcpResponse {
                id: request.id,
                result: None,
                error: Some(IcarusMcpError {
                    code: -32602,
                    message: "Missing tool name".to_string(),
                    data: None,
                }),
            },
        };
        
        let args = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));
        
        match self.tools.get(tool_name) {
            Some(tool) => {
                match tool.execute(args).await {
                    Ok(result) => IcarusMcpResponse {
                        id: request.id,
                        result: Some(serde_json::to_string(&serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": result.to_string()
                            }]
                        })).unwrap()),
                        error: None,
                    },
                    Err(e) => IcarusMcpResponse {
                        id: request.id,
                        result: None,
                        error: Some(IcarusMcpError {
                            code: -32603,
                            message: e.to_string(),
                            data: None,
                        }),
                    },
                }
            }
            None => IcarusMcpResponse {
                id: request.id,
                result: None,
                error: Some(IcarusMcpError {
                    code: -32602,
                    message: format!("Tool not found: {}", tool_name),
                    data: None,
                }),
            },
        }
    }
    
    fn handle_resources_list(&self, request: IcarusMcpRequest) -> IcarusMcpResponse {
        let resources: Vec<_> = self.resources.values()
            .map(|resource| resource.to_rmcp_resource())
            .collect();
            
        IcarusMcpResponse {
            id: request.id,
            result: Some(serde_json::to_string(&serde_json::json!({
                "resources": resources
            })).unwrap()),
            error: None,
        }
    }
    
    async fn handle_resource_read(&self, request: IcarusMcpRequest) -> IcarusMcpResponse {
        // Parse params from JSON string
        let params: serde_json::Value = match serde_json::from_str(&request.params) {
            Ok(p) => p,
            Err(_) => return IcarusMcpResponse {
                id: request.id,
                result: None,
                error: Some(IcarusMcpError {
                    code: -32602,
                    message: "Invalid params".to_string(),
                    data: None,
                }),
            },
        };
        
        let uri = match params.get("uri").and_then(|u| u.as_str()) {
            Some(uri) => uri,
            None => return IcarusMcpResponse {
                id: request.id,
                result: None,
                error: Some(IcarusMcpError {
                    code: -32602,
                    message: "Missing resource URI".to_string(),
                    data: None,
                }),
            },
        };
        
        match self.resources.get(uri) {
            Some(resource) => {
                match resource.read().await {
                    Ok(content) => IcarusMcpResponse {
                        id: request.id,
                        result: Some(serde_json::to_string(&serde_json::json!({
                            "contents": [{
                                "uri": uri,
                                "mimeType": resource.info().mime_type,
                                "text": String::from_utf8_lossy(&content)
                            }]
                        })).unwrap()),
                        error: None,
                    },
                    Err(e) => IcarusMcpResponse {
                        id: request.id,
                        result: None,
                        error: Some(IcarusMcpError {
                            code: -32603,
                            message: e.to_string(),
                            data: None,
                        }),
                    },
                }
            }
            None => IcarusMcpResponse {
                id: request.id,
                result: None,
                error: Some(IcarusMcpError {
                    code: -32602,
                    message: format!("Resource not found: {}", uri),
                    data: None,
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_harness_initialize() {
        let harness = TestHarness::new();
        let request = IcarusMcpRequest {
            id: Some("1".to_string()),
            method: "initialize".to_string(),
            params: serde_json::to_string(&serde_json::json!({
                "protocolVersion": "1.0.0"
            })).unwrap(),
        };
        
        let response = harness.execute(request).await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }
}