//! Runtime error types and handling.

use std::borrow::Cow;
use thiserror::Error;

/// Runtime-specific error types.
///
/// These errors occur during tool execution, registry operations,
/// and runtime management. All errors implement proper error chains
/// and provide rich context for debugging.
#[derive(Error, Debug)]
pub enum RuntimeError {
    /// Tool not found in registry
    #[error("Tool not found: {tool_id}")]
    ToolNotFound {
        /// The tool ID that was not found
        tool_id: String,
    },

    /// Tool execution failed
    #[error("Tool execution failed for '{tool_id}': {reason}")]
    ExecutionFailed {
        /// The tool ID that failed
        tool_id: String,
        /// Reason for failure
        reason: String,
    },

    /// Invalid tool arguments
    #[error("Invalid arguments for tool '{tool_id}': {details}")]
    InvalidArguments {
        /// The tool ID with invalid arguments
        tool_id: String,
        /// Details about the argument validation failure
        details: String,
    },

    /// JSON parsing error
    #[error("JSON parsing error in tool '{tool_id}': {source}")]
    JsonError {
        /// The tool ID where JSON parsing failed
        tool_id: String,
        /// The underlying JSON error
        #[source]
        source: serde_json::Error,
    },

    /// Registry corruption or inconsistency
    #[error("Registry error: {message}")]
    RegistryError {
        /// Error message describing the registry issue
        message: String,
    },

    /// Core library error
    #[error("Core error: {source}")]
    CoreError {
        /// The underlying core error
        #[from]
        source: Box<icarus_core::IcarusError>,
    },

    /// Async runtime error (only available with async feature)
    #[cfg(feature = "async")]
    #[error("Async runtime error: {message}")]
    AsyncError {
        /// Error message describing the async issue
        message: String,
    },
}

impl RuntimeError {
    /// Creates a new tool not found error.
    #[inline]
    pub fn tool_not_found(tool_id: impl Into<String>) -> Self {
        Self::ToolNotFound {
            tool_id: tool_id.into(),
        }
    }

    /// Creates a new execution failed error.
    #[inline]
    pub fn execution_failed(tool_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ExecutionFailed {
            tool_id: tool_id.into(),
            reason: reason.into(),
        }
    }

    /// Creates a new invalid arguments error.
    #[inline]
    pub fn invalid_arguments(tool_id: impl Into<String>, details: impl Into<String>) -> Self {
        Self::InvalidArguments {
            tool_id: tool_id.into(),
            details: details.into(),
        }
    }

    /// Creates a new JSON error.
    #[inline]
    pub fn json_error(tool_id: impl Into<String>, source: serde_json::Error) -> Self {
        Self::JsonError {
            tool_id: tool_id.into(),
            source,
        }
    }

    /// Creates a new registry error.
    #[inline]
    pub fn registry_error(message: impl Into<String>) -> Self {
        Self::RegistryError {
            message: message.into(),
        }
    }

    /// Creates a new async error (only available with async feature).
    #[cfg(feature = "async")]
    pub fn async_error(message: impl Into<String>) -> Self {
        Self::AsyncError {
            message: message.into(),
        }
    }

    /// Returns the tool ID associated with this error, if any.
    #[inline]
    #[must_use]
    pub fn tool_id(&self) -> Option<&str> {
        match self {
            Self::ToolNotFound { tool_id } => Some(tool_id),
            Self::ExecutionFailed { tool_id, .. } => Some(tool_id),
            Self::InvalidArguments { tool_id, .. } => Some(tool_id),
            Self::JsonError { tool_id, .. } => Some(tool_id),
            Self::RegistryError { .. } => None,
            Self::CoreError { .. } => None,
            #[cfg(feature = "async")]
            Self::AsyncError { .. } => None,
        }
    }

    /// Returns a user-friendly error message.
    #[must_use]
    pub fn user_message(&self) -> Cow<'static, str> {
        match self {
            Self::ToolNotFound { .. } => "The requested tool is not available".into(),
            Self::ExecutionFailed { .. } => "Tool execution failed".into(),
            Self::InvalidArguments { .. } => "Invalid arguments provided to tool".into(),
            Self::JsonError { .. } => "Failed to parse tool arguments".into(),
            Self::RegistryError { .. } => "Internal registry error".into(),
            Self::CoreError { .. } => "Internal system error".into(),
            #[cfg(feature = "async")]
            Self::AsyncError { .. } => "Async operation failed".into(),
        }
    }

    /// Returns the error severity level.
    #[inline]
    #[must_use]
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::ToolNotFound { .. } => ErrorSeverity::Warning,
            Self::ExecutionFailed { .. } => ErrorSeverity::Error,
            Self::InvalidArguments { .. } => ErrorSeverity::Warning,
            Self::JsonError { .. } => ErrorSeverity::Warning,
            Self::RegistryError { .. } => ErrorSeverity::Critical,
            Self::CoreError { .. } => ErrorSeverity::Error,
            #[cfg(feature = "async")]
            Self::AsyncError { .. } => ErrorSeverity::Error,
        }
    }
}

/// Error severity levels for categorizing runtime errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational - no action required
    Info,
    /// Warning - tool may work with different inputs
    Warning,
    /// Error - tool execution failed but system is stable
    Error,
    /// Critical - system integrity may be compromised
    Critical,
}

impl ErrorSeverity {
    /// Returns the numeric severity level (higher = more severe).
    #[must_use]
    pub const fn level(self) -> u8 {
        match self {
            Self::Info => 0,
            Self::Warning => 1,
            Self::Error => 2,
            Self::Critical => 3,
        }
    }

    /// Returns the severity name as a string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }
}

/// Convenience type alias for `Result<T, RuntimeError>`.
pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// Conversion from `IcarusError` to `RuntimeError`.
impl From<icarus_core::IcarusError> for RuntimeError {
    fn from(error: icarus_core::IcarusError) -> Self {
        RuntimeError::CoreError {
            source: Box::new(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = RuntimeError::tool_not_found("test_tool");
        assert_eq!(error.tool_id(), Some("test_tool"));
        assert_eq!(error.severity(), ErrorSeverity::Warning);
    }

    #[test]
    fn test_execution_failed_error() {
        let error = RuntimeError::execution_failed("test_tool", "timeout");
        assert_eq!(error.tool_id(), Some("test_tool"));
        assert_eq!(error.severity(), ErrorSeverity::Error);
        assert!(error.to_string().contains("timeout"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(ErrorSeverity::Critical > ErrorSeverity::Error);
        assert!(ErrorSeverity::Error > ErrorSeverity::Warning);
        assert!(ErrorSeverity::Warning > ErrorSeverity::Info);
    }

    #[test]
    fn test_severity_levels() {
        assert_eq!(ErrorSeverity::Info.level(), 0);
        assert_eq!(ErrorSeverity::Warning.level(), 1);
        assert_eq!(ErrorSeverity::Error.level(), 2);
        assert_eq!(ErrorSeverity::Critical.level(), 3);
    }

    #[test]
    fn test_user_messages() {
        let error = RuntimeError::tool_not_found("test");
        assert!(!error.user_message().is_empty());

        let error = RuntimeError::invalid_arguments("test", "bad input");
        assert!(!error.user_message().is_empty());
    }
}
