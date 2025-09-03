//! Response types for idiomatic tool return values
//!
//! These types provide a bridge between Rust's type system and MCP's JSON requirements.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// A successful tool response with optional data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSuccess<T = Value> {
    /// The main data payload
    #[serde(flatten)]
    pub data: T,

    /// Optional status message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T> ToolSuccess<T> {
    /// Create a new success response with data
    pub fn new(data: T) -> Self {
        Self {
            data,
            message: None,
        }
    }

    /// Add a message to the response
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

impl ToolSuccess<Value> {
    /// Create an empty success response
    pub fn empty() -> Self {
        Self {
            data: json!({}),
            message: None,
        }
    }
}

/// A simple status response for operations without data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    /// Whether the operation succeeded
    pub success: bool,

    /// Status message
    pub message: String,

    /// Optional additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl ToolStatus {
    /// Create a success status
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            details: None,
        }
    }

    /// Create an error status
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            details: None,
        }
    }

    /// Add details to the status
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Helper function to create a successful response with data
pub fn tool_success<T: Serialize>(data: T) -> Result<Value, crate::error::ToolError> {
    serde_json::to_value(data)
        .map_err(|e| crate::error::ToolError::internal(format!("Serialization failed: {}", e)))
}

/// Helper function to create a simple success status
pub fn tool_ok(message: impl Into<String>) -> Result<Value, crate::error::ToolError> {
    Ok(json!({
        "success": true,
        "message": message.into()
    }))
}

// Removed tool_error - use Err(ToolError::...) instead for proper error handling

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_success_new() {
        let data = json!({"count": 42});
        let success = ToolSuccess::new(data.clone());

        assert_eq!(success.data, data);
        assert_eq!(success.message, None);
    }

    #[test]
    fn test_tool_success_with_message() {
        let data = json!({"result": "ok"});
        let success = ToolSuccess::new(data.clone()).with_message("Operation completed");

        assert_eq!(success.data, data);
        assert_eq!(success.message, Some("Operation completed".to_string()));
    }

    #[test]
    fn test_tool_success_empty() {
        let success = ToolSuccess::empty();
        assert_eq!(success.data, json!({}));
        assert_eq!(success.message, None);
    }

    #[test]
    fn test_tool_success_serialization() {
        let success = ToolSuccess::new(json!({"value": 100})).with_message("Done");

        let serialized = serde_json::to_value(&success).unwrap();
        assert_eq!(serialized["value"], 100);
        assert_eq!(serialized["message"], "Done");
    }

    #[test]
    fn test_tool_success_no_message_serialization() {
        let success = ToolSuccess::new(json!({"key": "value"}));
        let serialized = serde_json::to_value(&success).unwrap();

        assert_eq!(serialized["key"], "value");
        assert!(!serialized.as_object().unwrap().contains_key("message"));
    }

    #[test]
    fn test_tool_status_success() {
        let status = ToolStatus::success("All good");

        assert!(status.success);
        assert_eq!(status.message, "All good");
        assert_eq!(status.details, None);
    }

    #[test]
    fn test_tool_status_error() {
        let status = ToolStatus::error("Something went wrong");

        assert!(!status.success);
        assert_eq!(status.message, "Something went wrong");
        assert_eq!(status.details, None);
    }

    #[test]
    fn test_tool_status_with_details() {
        let details = json!({"code": 404, "reason": "not found"});
        let status = ToolStatus::error("Failed").with_details(details.clone());

        assert!(!status.success);
        assert_eq!(status.message, "Failed");
        assert_eq!(status.details, Some(details));
    }

    #[test]
    fn test_tool_status_serialization() {
        let status = ToolStatus::success("Complete").with_details(json!({"items": 5}));

        let serialized = serde_json::to_value(&status).unwrap();
        assert_eq!(serialized["success"], true);
        assert_eq!(serialized["message"], "Complete");
        assert_eq!(serialized["details"]["items"], 5);
    }

    #[test]
    fn test_tool_success_helper() {
        #[derive(Serialize)]
        struct TestData {
            name: String,
            age: u32,
        }

        let data = TestData {
            name: "Alice".to_string(),
            age: 30,
        };

        let result = tool_success(data).unwrap();
        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
    }

    #[test]
    fn test_tool_ok_helper() {
        let result = tool_ok("Task completed").unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["message"], "Task completed");
    }

    #[test]
    fn test_tool_success_with_struct() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct CustomData {
            id: u64,
            value: String,
        }

        let data = CustomData {
            id: 123,
            value: "test".to_string(),
        };

        let success = ToolSuccess::new(data.clone());
        let serialized = serde_json::to_string(&success).unwrap();
        let deserialized: ToolSuccess<CustomData> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.data, data);
    }
}
