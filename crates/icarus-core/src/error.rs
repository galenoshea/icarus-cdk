//! Error types for the Icarus SDK

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