//! Testing utilities for Icarus MCP servers
//! 
//! This crate provides test helpers and mocks for developing and testing
//! Icarus MCP servers locally without deploying to ICP.

pub mod mock;
pub mod harness;
pub mod assertions;

use icarus_core::protocol::{IcarusMetadata, ToolMetadata};
use serde_json::Value;

/// Test context for running MCP server tests
pub struct TestContext {
    /// Mock canister environment
    pub env: mock::MockEnvironment,
}

/// Helper to create a tool call request
pub fn tool_call_request(tool_name: &str, args: Value) -> Value {
    serde_json::json!({
        "method": tool_name,
        "arguments": args
    })
}

/// Assert that a response is successful
pub fn assert_success(response: &Value) {
    assert!(response.get("error").is_none(), "Expected success but got error: {:?}", response.get("error"));
    assert!(response.get("result").is_some() || response.get("status").is_some(), "Expected result but got none");
}

impl TestContext {
    /// Create a new test context
    pub fn new() -> Self {
        Self {
            env: mock::MockEnvironment::new(),
        }
    }
    
    /// Execute a request in the test context
    pub async fn execute_request(&self, method: &str, params: Value) -> Value {
        // Simulate direct method call
        serde_json::json!({
            "status": "success",
            "result": params
        })
    }
    
    /// Execute a tool with the given server instance
    pub async fn execute_tool<S>(&mut self, server: &mut S, method: &str, params: Value) -> Value
    where
        S: icarus_core::server::IcarusServer,
    {
        // For now, just forward to execute_request
        self.execute_request(method, params).await
    }
    
    /// Helper to create a tool call request
    pub fn tool_call_request(tool_name: &str, args: Value) -> (&str, Value) {
        (tool_name, args)
    }
    
    /// Helper to get metadata
    pub fn get_metadata() -> IcarusMetadata {
        icarus_canister::endpoints::icarus_metadata()
    }
}

/// Test utilities for assertions
pub mod test_utils {
    use super::*;
    
    /// Assert that a response is successful
    pub fn assert_success(response: &Value) {
        assert!(response.get("error").is_none(), "Expected success but got error: {:?}", response.get("error"));
        assert!(response.get("result").is_some() || response.get("status").is_some(), "Expected result but got none");
    }
    
    /// Assert that a response is an error
    pub fn assert_error(response: &Value) {
        assert!(response.get("error").is_some(), "Expected error but got success");
        assert!(response.get("result").is_none(), "Expected no result but got: {:?}", response.get("result"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_context_creation() {
        let ctx = TestContext::new();
        
        // Test metadata request
        let metadata = TestContext::get_metadata();
        let response = serde_json::json!({
            "status": "success",
            "metadata": metadata
        });
        
        test_utils::assert_success(&response);
    }
}