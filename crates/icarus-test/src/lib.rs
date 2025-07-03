//! Testing utilities for Icarus MCP servers
//! 
//! This crate provides test helpers and mocks for developing and testing
//! Icarus MCP servers locally without deploying to ICP.

pub mod mock;
pub mod harness;
pub mod assertions;

use icarus_core::protocol::{IcarusMcpRequest, IcarusMcpResponse};
use serde_json::Value;

/// Test context for running MCP server tests
pub struct TestContext {
    /// Mock canister environment
    pub env: mock::MockEnvironment,
}

impl TestContext {
    /// Create a new test context
    pub fn new() -> Self {
        Self {
            env: mock::MockEnvironment::new(),
        }
    }
    
    /// Execute an MCP request in the test context
    pub async fn execute_request(&self, request: IcarusMcpRequest) -> IcarusMcpResponse {
        // Simulate request handling
        icarus_canister::endpoints::icarus_mcp_request(request).await
    }
    
    /// Helper to create a tool call request
    pub fn tool_call_request(tool_name: &str, args: Value) -> IcarusMcpRequest {
        IcarusMcpRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": tool_name,
                "arguments": args
            })),
        }
    }
    
    /// Helper to create an initialize request
    pub fn initialize_request() -> IcarusMcpRequest {
        IcarusMcpRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "1.0.0",
                "capabilities": {}
            })),
        }
    }
}

/// Test utilities for assertions
pub mod test_utils {
    use super::*;
    
    /// Assert that a response is successful
    pub fn assert_success(response: &IcarusMcpResponse) {
        assert!(response.error.is_none(), "Expected success but got error: {:?}", response.error);
        assert!(response.result.is_some(), "Expected result but got none");
    }
    
    /// Assert that a response is an error
    pub fn assert_error(response: &IcarusMcpResponse) {
        assert!(response.error.is_some(), "Expected error but got success");
        assert!(response.result.is_none(), "Expected no result but got: {:?}", response.result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_context_creation() {
        let ctx = TestContext::new();
        
        // Test initialize request
        let init_req = TestContext::initialize_request();
        let response = ctx.execute_request(init_req).await;
        
        test_utils::assert_success(&response);
    }
}