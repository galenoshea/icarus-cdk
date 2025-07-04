//! Protocol translation between MCP and ICP
//! 
//! This module is kept for backward compatibility but is not used
//! in the clean architecture where bridge handles all translation

use crate::protocol::{IcarusBridgeRequest, IcarusBridgeResponse, BridgeError};
use serde_json::{json, Value};

/// Translates between MCP JSON-RPC and ICP canister calls
/// Deprecated: Bridge handles all protocol translation now
pub struct ProtocolTranslator;

impl ProtocolTranslator {
    pub fn new() -> Self {
        Self
    }
    
    /// Convert MCP method and params to ICP request format
    pub fn mcp_to_icp(
        &self,
        method: String,
        params: Value
    ) -> Result<Value, Box<dyn std::error::Error>> {
        // In clean architecture, this is handled by bridge
        Ok(json!({
            "method": method,
            "params": params
        }))
    }
    
    /// Convert ICP response back to MCP format
    pub fn icp_to_mcp(&self, response: Value) -> Result<Value, Box<dyn std::error::Error>> {
        // In clean architecture, this is handled by bridge
        Ok(response)
    }
    
    /// Convert bridge request to canister request
    pub fn to_canister_request(&self, bridge_req: IcarusBridgeRequest) -> Value {
        json!({
            "method": bridge_req.method,
            "params": bridge_req.params
        })
    }
    
    /// Convert canister response to bridge response
    pub fn from_canister_response(&self, response: Value, id: Option<serde_json::Value>) -> IcarusBridgeResponse {
        IcarusBridgeResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(response),
            error: None,
            id,
        }
    }
    
    /// Validate MCP method names
    pub fn validate_method(&self, method: &str) -> bool {
        matches!(method,
            "initialize" |
            "tools/list" |
            "tools/call" |
            "resources/list" |
            "resources/read" |
            "prompts/list" |
            "prompts/get" |
            "ping"
        )
    }
    
    /// Transform tool call parameters
    pub fn transform_tool_call(&self, params: &Value) -> Result<Value, Box<dyn std::error::Error>> {
        let name = params.get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing tool name")?;
            
        let arguments = params.get("arguments")
            .cloned()
            .unwrap_or(json!({}));
            
        Ok(json!({
            "name": name,
            "arguments": arguments
        }))
    }
    
    /// Transform resource read parameters
    pub fn transform_resource_read(&self, params: &Value) -> Result<Value, Box<dyn std::error::Error>> {
        let uri = params.get("uri")
            .and_then(|v| v.as_str())
            .ok_or("Missing resource URI")?;
            
        Ok(json!({
            "uri": uri
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mcp_to_icp_translation() {
        let translator = ProtocolTranslator::new();
        
        let params = json!({
            "canister_id": "test-canister",
            "name": "test-tool",
            "arguments": {"key": "value"}
        });
        
        let result = translator.mcp_to_icp("tools/call".to_string(), params).unwrap();
        assert!(result.get("method").is_some());
    }
    
    #[test]
    fn test_validate_method() {
        let translator = ProtocolTranslator::new();
        
        assert!(translator.validate_method("initialize"));
        assert!(translator.validate_method("tools/list"));
        assert!(!translator.validate_method("invalid/method"));
    }
}