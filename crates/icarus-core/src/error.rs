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
