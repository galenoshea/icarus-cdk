//! Common result types and error handling for Icarus canisters

use candid::{CandidType, Deserialize};
use serde::Serialize;

/// Standard result type for Icarus tools
pub type IcarusResult<T> = Result<T, IcarusError>;

/// Standard error type for Icarus operations
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum IcarusError {
    /// Authentication or authorization error
    Unauthorized(String),
    /// Validation error with field and message
    ValidationError { field: String, message: String },
    /// Resource not found
    NotFound(String),
    /// Resource already exists
    AlreadyExists(String),
    /// Storage operation failed
    StorageError(String),
    /// Generic error with message
    Other(String),
}

impl IcarusError {
    /// Create an unauthorized error
    pub fn unauthorized() -> Self {
        Self::Unauthorized("Unauthorized access".to_string())
    }

    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound(format!("{} not found", resource.into()))
    }

    /// Create an already exists error
    pub fn already_exists(resource: impl Into<String>) -> Self {
        Self::AlreadyExists(format!("{} already exists", resource.into()))
    }

    /// Create a storage error
    pub fn storage(message: impl Into<String>) -> Self {
        Self::StorageError(message.into())
    }

    /// Convert to a trap message for compatibility
    pub fn trap(self) -> ! {
        ic_cdk::trap(&self.to_string())
    }
}

impl std::fmt::Display for IcarusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Self::ValidationError { field, message } => {
                write!(f, "Validation error on '{}': {}", field, message)
            }
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::AlreadyExists(msg) => write!(f, "Already exists: {}", msg),
            Self::StorageError(msg) => write!(f, "Storage error: {}", msg),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for IcarusError {}

impl From<String> for IcarusError {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}

impl From<&str> for IcarusError {
    fn from(s: &str) -> Self {
        Self::Other(s.to_string())
    }
}

/// Extension trait for Result types to convert to trap
pub trait TrapExt<T> {
    /// Unwrap the result or trap with the error message
    fn unwrap_or_trap(self) -> T;
}

impl<T> TrapExt<T> for Result<T, IcarusError> {
    fn unwrap_or_trap(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => e.trap(),
        }
    }
}
