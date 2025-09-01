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
