//! Protocol translation between MCP and ICP

use crate::protocol::{IcarusBridgeRequest, IcarusBridgeResponse, BridgeError};
use icarus_core::protocol::{IcarusMcpRequest, IcarusMcpResponse};
use serde_json::{json, Value};

/// Translates between MCP JSON-RPC and ICP canister calls
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
        // Remove canister_id from params as it's handled separately
        let mut cleaned_params = params.clone();
        if let Some(obj) = cleaned_params.as_object_mut() {
            obj.remove("canister_id");
            obj.remove("canisterId");
        }
        
        // Create ICP-compatible request
        let icp_request = IcarusMcpRequest {
            method,
            params: cleaned_params.to_string(),
            id: None, // ID is handled at bridge level
        };
        
        // Convert to JSON for canister call
        Ok(serde_json::to_value(icp_request)?)
    }
    
    /// Convert ICP response back to MCP format
    pub fn icp_to_mcp(&self, response: Value) -> Result<Value, Box<dyn std::error::Error>> {
        // Parse ICP response
        let icp_response: IcarusMcpResponse = serde_json::from_value(response)?;
        
        // Convert to MCP format
        if let Some(result) = icp_response.result {
            // Parse the JSON string result
            let result_value: Value = serde_json::from_str(&result)?;
            Ok(result_value)
        } else if let Some(error) = icp_response.error {
            Ok(json!({
                "error": {
                    "code": error.code,
                    "message": error.message,
                    "data": error.data
                }
            }))
        } else {
            Ok(json!(null))
        }
    }
    
    /// Convert bridge request to canister request
    pub fn to_canister_request(&self, bridge_req: IcarusBridgeRequest) -> IcarusMcpRequest {
        IcarusMcpRequest {
            method: bridge_req.method,
            params: bridge_req.params.to_string(),
            id: bridge_req.id.map(|v| v.to_string()),
        }
    }
    
    /// Convert canister response to bridge response
    pub fn from_canister_response(&self, canister_resp: IcarusMcpResponse, id: Option<serde_json::Value>) -> IcarusBridgeResponse {
        IcarusBridgeResponse {
            jsonrpc: "2.0".to_string(),
            result: canister_resp.result.and_then(|s| serde_json::from_str(&s).ok()),
            error: canister_resp.error.map(|e| BridgeError {
                code: e.code,
                message: e.message,
                data: e.data.and_then(|s| serde_json::from_str(&s).ok()),
            }),
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
        
        // Should have removed canister_id
        let result_str = result.get("params").unwrap().as_str().unwrap();
        assert!(!result_str.contains("canister_id"));
    }
    
    #[test]
    fn test_validate_method() {
        let translator = ProtocolTranslator::new();
        
        assert!(translator.validate_method("initialize"));
        assert!(translator.validate_method("tools/list"));
        assert!(!translator.validate_method("invalid/method"));
    }
    
    #[test]
    fn test_bridge_request_conversion() {
        let translator = ProtocolTranslator::new();
        
        let bridge_req = IcarusBridgeRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: json!({}),
            id: Some(json!(1)),
        };
        
        let canister_req = translator.to_canister_request(bridge_req);
        assert_eq!(canister_req.method, "tools/list");
        assert_eq!(canister_req.params, "{}");
        assert_eq!(canister_req.id, Some("1".to_string()));
    }
}