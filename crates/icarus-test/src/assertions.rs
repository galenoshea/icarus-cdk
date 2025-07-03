//! Test assertions for Icarus MCP servers

use icarus_core::protocol::IcarusMcpResponse;
use serde_json::Value;

/// Assertion helpers for MCP responses
pub struct ResponseAssertions<'a> {
    response: &'a IcarusMcpResponse,
}

impl<'a> ResponseAssertions<'a> {
    /// Create assertions for a response
    pub fn new(response: &'a IcarusMcpResponse) -> Self {
        Self { response }
    }
    
    /// Assert the response was successful
    pub fn assert_success(&self) -> &Self {
        assert!(
            self.response.error.is_none(),
            "Expected successful response but got error: {:?}",
            self.response.error
        );
        assert!(
            self.response.result.is_some(),
            "Expected result in successful response"
        );
        self
    }
    
    /// Assert the response was an error
    pub fn assert_error(&self) -> &Self {
        assert!(
            self.response.error.is_some(),
            "Expected error response but got success"
        );
        assert!(
            self.response.result.is_none(),
            "Expected no result in error response"
        );
        self
    }
    
    /// Assert the error code matches
    pub fn assert_error_code(&self, expected_code: i32) -> &Self {
        self.assert_error();
        if let Some(error) = &self.response.error {
            if let Some(code) = error.get("code").and_then(|c| c.as_i64()) {
                assert_eq!(
                    code, expected_code as i64,
                    "Expected error code {} but got {}",
                    expected_code, code
                );
            } else {
                panic!("Error response missing code field");
            }
        }
        self
    }
    
    /// Assert the error message contains a substring
    pub fn assert_error_contains(&self, substring: &str) -> &Self {
        self.assert_error();
        if let Some(error) = &self.response.error {
            if let Some(message) = error.get("message").and_then(|m| m.as_str()) {
                assert!(
                    message.contains(substring),
                    "Expected error message to contain '{}' but got: {}",
                    substring, message
                );
            } else {
                panic!("Error response missing message field");
            }
        }
        self
    }
    
    /// Assert a field exists in the result
    pub fn assert_result_has(&self, field: &str) -> &Self {
        self.assert_success();
        if let Some(result) = &self.response.result {
            assert!(
                result.get(field).is_some(),
                "Expected field '{}' in result but it was not found. Result: {:?}",
                field, result
            );
        }
        self
    }
    
    /// Get a value from the result for further assertions
    pub fn get_result_value(&self, path: &str) -> Option<&Value> {
        self.response.result.as_ref().and_then(|result| {
            let parts: Vec<&str> = path.split('.').collect();
            let mut current = result;
            
            for part in parts {
                current = current.get(part)?;
            }
            
            Some(current)
        })
    }
}

/// Extension trait for easy assertions on responses
pub trait ResponseAssertExt {
    fn assert(&self) -> ResponseAssertions;
}

impl ResponseAssertExt for IcarusMcpResponse {
    fn assert(&self) -> ResponseAssertions {
        ResponseAssertions::new(self)
    }
}

/// Macros for common assertions
#[macro_export]
macro_rules! assert_tool_list_contains {
    ($response:expr, $tool_name:expr) => {{
        use $crate::assertions::ResponseAssertExt;
        
        $response.assert()
            .assert_success()
            .assert_result_has("tools");
            
        let tools = $response.assert()
            .get_result_value("tools")
            .and_then(|v| v.as_array())
            .expect("Expected tools to be an array");
            
        let has_tool = tools.iter().any(|tool| {
            tool.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n == $tool_name)
                .unwrap_or(false)
        });
        
        assert!(
            has_tool,
            "Expected tool '{}' in tools list but it was not found",
            $tool_name
        );
    }};
}

/// Assert a resource list contains a specific resource
#[macro_export]
macro_rules! assert_resource_list_contains {
    ($response:expr, $resource_uri:expr) => {{
        use $crate::assertions::ResponseAssertExt;
        
        $response.assert()
            .assert_success()
            .assert_result_has("resources");
            
        let resources = $response.assert()
            .get_result_value("resources")
            .and_then(|v| v.as_array())
            .expect("Expected resources to be an array");
            
        let has_resource = resources.iter().any(|resource| {
            resource.get("uri")
                .and_then(|u| u.as_str())
                .map(|u| u == $resource_uri)
                .unwrap_or(false)
        });
        
        assert!(
            has_resource,
            "Expected resource '{}' in resources list but it was not found",
            $resource_uri
        );
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertions::ResponseAssertExt;
    
    #[test]
    fn test_success_assertions() {
        let response = IcarusMcpResponse {
            jsonrpc: "2.0".to_string(),
            id: 1,
            result: Some(serde_json::json!({
                "status": "ok",
                "data": {
                    "value": 42
                }
            })),
            error: None,
        };
        
        response.assert()
            .assert_success()
            .assert_result_has("status")
            .assert_result_has("data");
            
        let value = response.assert().get_result_value("data.value");
        assert_eq!(value, Some(&serde_json::json!(42)));
    }
    
    #[test]
    fn test_error_assertions() {
        let response = IcarusMcpResponse {
            jsonrpc: "2.0".to_string(),
            id: 1,
            result: None,
            error: Some(serde_json::json!({
                "code": -32602,
                "message": "Invalid params: missing field 'name'"
            })),
        };
        
        response.assert()
            .assert_error()
            .assert_error_code(-32602)
            .assert_error_contains("missing field");
    }
}