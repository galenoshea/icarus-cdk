//! Error types for the Icarus SDK

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main error type for Icarus operations
#[derive(Error, Debug)]
pub enum IcarusError {
    /// Error from the underlying MCP implementation
    #[error("MCP error: {0}")]
    Mcp(String),

    /// Error from ICP canister operations
    #[error("Canister error: {0}")]
    Canister(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// State management errors
    #[error("State error: {0}")]
    State(String),

    /// Protocol translation errors
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Tool execution errors
    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),

    /// Generic errors
    #[error("{0}")]
    Other(String),
}

/// Result type alias for Icarus operations
pub type Result<T> = std::result::Result<T, IcarusError>;

impl From<candid::Error> for IcarusError {
    fn from(err: candid::Error) -> Self {
        IcarusError::Canister(err.to_string())
    }
}

/// Specialized error type for tool execution
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ToolError {
    /// Invalid input provided to the tool
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Requested resource was not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Permission denied for the operation
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Operation failed for a specific reason
    #[error("Operation failed: {0}")]
    OperationFailed(String),

    /// Internal error occurred
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl ToolError {
    /// Create an InvalidInput error
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Create a NotFound error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create a PermissionDenied error
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }

    /// Create an OperationFailed error
    pub fn operation_failed(msg: impl Into<String>) -> Self {
        Self::OperationFailed(msg.into())
    }

    /// Create an InternalError
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::InternalError(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icarus_error_display() {
        let err = IcarusError::Mcp("connection failed".to_string());
        assert_eq!(err.to_string(), "MCP error: connection failed");

        let err = IcarusError::Canister("out of cycles".to_string());
        assert_eq!(err.to_string(), "Canister error: out of cycles");

        let err = IcarusError::State("invalid state".to_string());
        assert_eq!(err.to_string(), "State error: invalid state");

        let err = IcarusError::Protocol("invalid message".to_string());
        assert_eq!(err.to_string(), "Protocol error: invalid message");

        let err = IcarusError::Other("something went wrong".to_string());
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn test_tool_error_helpers() {
        let err = ToolError::invalid_input("bad input");
        assert_eq!(err.to_string(), "Invalid input: bad input");

        let err = ToolError::not_found("resource missing");
        assert_eq!(err.to_string(), "Not found: resource missing");

        let err = ToolError::permission_denied("unauthorized");
        assert_eq!(err.to_string(), "Permission denied: unauthorized");

        let err = ToolError::operation_failed("failed to process");
        assert_eq!(err.to_string(), "Operation failed: failed to process");

        let err = ToolError::internal("internal error");
        assert_eq!(err.to_string(), "Internal error: internal error");
    }

    #[test]
    fn test_tool_error_conversion() {
        let tool_err = ToolError::invalid_input("bad data");
        let icarus_err: IcarusError = tool_err.into();
        assert_eq!(
            icarus_err.to_string(),
            "Tool error: Invalid input: bad data"
        );
    }

    #[test]
    fn test_candid_error_conversion() {
        // We can't easily create a real candid::Error, but we can test the conversion exists
        // by checking that the From trait is implemented
        fn _test_conversion_exists() {
            let _: fn(candid::Error) -> IcarusError = IcarusError::from;
        }
    }

    #[test]
    fn test_tool_error_serialization() {
        let err = ToolError::not_found("item");
        let serialized = serde_json::to_string(&err).unwrap();
        let deserialized: ToolError = serde_json::from_str(&serialized).unwrap();
        assert_eq!(err.to_string(), deserialized.to_string());
    }

    #[test]
    fn test_tool_error_clone() {
        let err1 = ToolError::permission_denied("access denied");
        let err2 = err1.clone();
        assert_eq!(err1.to_string(), err2.to_string());
    }
}
