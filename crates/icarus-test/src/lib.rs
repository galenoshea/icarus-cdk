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

/// Helper to create a tool call request
pub fn tool_call_request(tool_name: &str, args: Value) -> IcarusMcpRequest {
    IcarusMcpRequest {
        id: Some("1".to_string()),
        method: "tools/call".to_string(),
        params: serde_json::to_string(&serde_json::json!({
            "name": tool_name,
            "arguments": args
        })).unwrap(),
    }
}

/// Assert that a response is successful
pub fn assert_success(response: &IcarusMcpResponse) {
    assert!(response.error.is_none(), "Expected success but got error: {:?}", response.error);
    assert!(response.result.is_some(), "Expected result but got none");
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
    
    /// Execute a tool with the given server instance
    pub async fn execute_tool<S>(&mut self, server: &mut S, request: IcarusMcpRequest) -> IcarusMcpResponse
    where
        S: icarus_core::server::IcarusServer,
    {
        // For now, just forward to execute_request
        // In a real implementation, this would use the server instance directly
        self.execute_request(request).await
    }
    
    /// Helper to create a tool call request
    pub fn tool_call_request(tool_name: &str, args: Value) -> IcarusMcpRequest {
        IcarusMcpRequest {
            id: Some("1".to_string()),
            method: "tools/call".to_string(),
            params: serde_json::to_string(&serde_json::json!({
                "name": tool_name,
                "arguments": args
            })).unwrap(),
        }
    }
    
    /// Helper to create an initialize request
    pub fn initialize_request() -> IcarusMcpRequest {
        IcarusMcpRequest {
            id: Some("1".to_string()),
            method: "initialize".to_string(),
            params: serde_json::to_string(&serde_json::json!({
                "protocolVersion": "1.0.0",
                "capabilities": {}
            })).unwrap(),
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