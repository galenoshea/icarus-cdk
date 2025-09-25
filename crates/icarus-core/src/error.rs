//! Error types for the Icarus CDK

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

    #[test]
    fn test_serde_json_error_conversion() {
        // Test automatic conversion from serde_json::Error
        let bad_json = r#"{"invalid": json"#;
        let parse_result: std::result::Result<serde_json::Value, serde_json::Error> =
            serde_json::from_str(bad_json);

        match parse_result {
            Err(json_err) => {
                let icarus_err: IcarusError = json_err.into();
                assert!(icarus_err.to_string().contains("Serialization error:"));
            }
            Ok(_) => panic!("Expected JSON parsing to fail"),
        }
    }

    #[test]
    fn test_error_debug_formatting() {
        let err = IcarusError::Tool(ToolError::invalid_input("test"));
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Tool"));
        assert!(debug_str.contains("InvalidInput"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_result_type_alias() {
        // Test that our Result type alias works correctly
        let value = 42;
        let success: Result<i32> = Ok(value);
        let failure: Result<i32> = Err(IcarusError::Other("failed".to_string()));

        assert!(success.is_ok());
        assert_eq!(success.unwrap(), value);

        assert!(failure.is_err());
        assert_eq!(failure.unwrap_err().to_string(), "failed");
    }

    #[test]
    fn test_tool_error_variants_coverage() {
        // Ensure all ToolError variants are properly tested
        let errors = vec![
            ToolError::InvalidInput("test".to_string()),
            ToolError::NotFound("test".to_string()),
            ToolError::PermissionDenied("test".to_string()),
            ToolError::OperationFailed("test".to_string()),
            ToolError::InternalError("test".to_string()),
        ];

        for error in errors {
            // Test Debug trait
            let debug_str = format!("{:?}", error);
            assert!(!debug_str.is_empty());

            // Test Display trait
            let display_str = error.to_string();
            assert!(!display_str.is_empty());

            // Test Clone trait
            let cloned = error.clone();
            assert_eq!(error.to_string(), cloned.to_string());
        }
    }

    #[test]
    fn test_icarus_error_variants_coverage() {
        // Test all IcarusError variants
        let tool_error = ToolError::invalid_input("test");
        let json_error =
            serde_json::from_str::<serde_json::Value>(r#"{"invalid": json"#).unwrap_err();

        let errors = vec![
            IcarusError::Mcp("mcp error".to_string()),
            IcarusError::Canister("canister error".to_string()),
            IcarusError::Serialization(json_error),
            IcarusError::State("state error".to_string()),
            IcarusError::Protocol("protocol error".to_string()),
            IcarusError::Tool(tool_error),
            IcarusError::Other("other error".to_string()),
        ];

        for error in errors {
            // Test Debug trait
            let debug_str = format!("{:?}", error);
            assert!(!debug_str.is_empty());

            // Test Display trait
            let display_str = error.to_string();
            assert!(!display_str.is_empty());
        }
    }

    #[test]
    fn test_tool_error_constructor_variants() {
        // Test all constructor methods accept different string types
        let static_str = "static string";
        let owned_string = String::from("owned string");
        let string_slice = &owned_string[..];

        // Test with &str
        let err1 = ToolError::invalid_input(static_str);
        assert!(err1.to_string().contains("static string"));

        // Test with String
        let err2 = ToolError::not_found(owned_string.clone());
        assert!(err2.to_string().contains("owned string"));

        // Test with string slice
        let err3 = ToolError::permission_denied(string_slice);
        assert!(err3.to_string().contains("owned string"));

        // Test with format! result
        let err4 = ToolError::operation_failed(format!("formatted {}", 42));
        assert!(err4.to_string().contains("formatted 42"));

        // Test with owned literal
        let err5 = ToolError::internal("literal".to_string());
        assert!(err5.to_string().contains("literal"));
    }

    #[test]
    fn test_error_chain_propagation() {
        // Test that error chains work correctly through conversions
        let tool_err = ToolError::operation_failed("database connection failed");
        let icarus_err: IcarusError = tool_err.into();

        // Verify the error message contains both levels
        let full_message = icarus_err.to_string();
        assert!(full_message.contains("Tool error:"));
        assert!(full_message.contains("Operation failed:"));
        assert!(full_message.contains("database connection failed"));
    }

    #[test]
    fn test_error_equality_through_serialization() {
        // Test that ToolError round-trip serialization preserves equality
        let original_errors = vec![
            ToolError::InvalidInput("input".to_string()),
            ToolError::NotFound("resource".to_string()),
            ToolError::PermissionDenied("access".to_string()),
            ToolError::OperationFailed("operation".to_string()),
            ToolError::InternalError("internal".to_string()),
        ];

        for original in original_errors {
            let json = serde_json::to_string(&original).unwrap();
            let deserialized: ToolError = serde_json::from_str(&json).unwrap();

            // Since ToolError doesn't implement PartialEq, we compare string representations
            assert_eq!(original.to_string(), deserialized.to_string());
            assert_eq!(format!("{:?}", original), format!("{:?}", deserialized));
        }
    }
}
