//! Protocol translation between MCP and Candid
//! 
//! This is the core of the clean architecture - all protocol complexity lives here

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use candid::Principal;

use crate::bridge::canister_client::CanisterClient;

/// MCP Request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Value,
}

/// MCP Response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

/// MCP Error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Tool metadata from canister
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub name: String,
    pub candid_method: String,
    pub is_query: bool,
    pub description: String,
    pub parameters: Vec<ParameterMetadata>,
}

/// Parameter metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterMetadata {
    pub name: String,
    pub candid_type: String,
    pub required: bool,
    pub description: String,
}

/// Canister metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcarusMetadata {
    pub version: String,
    pub canister_id: String,
    pub tools: Vec<ToolMetadata>,
}

/// Core protocol translator
pub struct ProtocolTranslator {
    pub canister_client: CanisterClient,
    pub metadata: IcarusMetadata,
}

impl ProtocolTranslator {
    /// Initialize by fetching tool metadata from canister
    pub async fn new(canister_id: Principal) -> Result<Self> {
        let client = CanisterClient::new(canister_id);
        
        // Discover available tools via metadata endpoint
        let metadata_json = client
            .query("__icarus_metadata", serde_json::json!({}))
            .await?;
        let metadata: IcarusMetadata = serde_json::from_str(&metadata_json)?;
            
        Ok(Self {
            canister_client: client,
            metadata,
        })
    }
    
    /// Handle MCP request by translating to Candid calls
    pub async fn handle_mcp_request(&self, request: McpRequest) -> McpResponse {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize().await,
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => self.handle_tool_call(request.params).await,
            _ => Err(anyhow!("Unknown method: {}", request.method)),
        };
        
        match result {
            Ok(value) => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: Some(value),
                error: None,
            },
            Err(e) => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(json!({
                    "code": -32603,
                    "message": e.to_string(),
                    "data": null
                })),
            },
        }
    }
    
    /// Handle initialize request
    async fn handle_initialize(&self) -> Result<Value> {
        Ok(json!({
            "protocolVersion": "1.0.0",
            "capabilities": {
                "tools": {},
                "resources": {}
            }
        }))
    }
    
    /// Handle tools/list request
    async fn handle_list_tools(&self) -> Result<Value> {
        let tools: Vec<Value> = self.metadata.tools.iter()
            .map(|tool| json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": {
                    "type": "object",
                    "properties": self.parameters_to_schema(&tool.parameters),
                    "required": self.required_parameters(&tool.parameters)
                }
            }))
            .collect();
            
        Ok(json!({ "tools": tools }))
    }
    
    /// Handle tools/call request
    async fn handle_tool_call(&self, params: Value) -> Result<Value> {
        let tool_name = params.get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| anyhow!("Missing tool name"))?;
        
        let arguments = params.get("arguments")
            .cloned()
            .unwrap_or(json!({}));
        
        // Find tool metadata
        let tool = self.metadata.tools.iter()
            .find(|t| t.name == tool_name)
            .ok_or_else(|| anyhow!("Tool not found: {}", tool_name))?;
        
        // Convert MCP arguments to Candid parameters
        let candid_args = self.mcp_to_candid(&arguments, &tool.parameters)?;
        
        // Call the actual Candid method
        let result_json = if tool.is_query {
            self.canister_client
                .query(&tool.candid_method, candid_args)
                .await?
        } else {
            self.canister_client
                .update(&tool.candid_method, candid_args)
                .await?
        };
        
        // Parse the result
        let result: Value = serde_json::from_str(&result_json)?;
        
        // Convert to MCP response format
        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&result)?
            }]
        }))
    }
    
    /// Convert parameters to JSON schema
    fn parameters_to_schema(&self, params: &[ParameterMetadata]) -> Value {
        let mut properties = serde_json::Map::new();
        
        for param in params {
            properties.insert(param.name.clone(), json!({
                "type": self.candid_type_to_json_type(&param.candid_type),
                "description": param.description
            }));
        }
        
        Value::Object(properties)
    }
    
    /// Get required parameters
    fn required_parameters(&self, params: &[ParameterMetadata]) -> Vec<String> {
        params.iter()
            .filter(|p| p.required)
            .map(|p| p.name.clone())
            .collect()
    }
    
    /// Convert Candid type to JSON schema type
    fn candid_type_to_json_type(&self, candid_type: &str) -> &'static str {
        match candid_type {
            "text" => "string",
            "nat" | "nat8" | "nat16" | "nat32" | "nat64" => "number",
            "int" | "int8" | "int16" | "int32" | "int64" => "number",
            "bool" => "boolean",
            _ => "object"
        }
    }
    
    /// Convert MCP arguments to Candid format
    fn mcp_to_candid(&self, args: &Value, _params: &[ParameterMetadata]) -> Result<Value> {
        // For MVP, just pass through the arguments
        // In a full implementation, this would convert based on parameter types
        Ok(args.clone())
    }
}