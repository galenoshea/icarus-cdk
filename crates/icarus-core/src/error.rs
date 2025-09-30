//! Error types for the Icarus CDK following `rust_best_practices.md` patterns.
//!
//! This module provides comprehensive error handling using `thiserror` for libraries
//! (as recommended in `rust_best_practices.md`). All errors include rich context and
//! support error chaining.

use std::fmt;

use candid::{CandidType, Deserialize};
use serde::Serialize;
use thiserror::Error;

/// Main error type for the Icarus CDK.
///
/// This enum covers all possible errors that can occur during MCP server
/// operations, following the error handling patterns from `rust_best_practices.md`.
///
/// # Examples
///
/// ```rust
/// use icarus_core::{IcarusError, ToolId};
///
/// // Tool not found error
/// let error = IcarusError::ToolNotFound(ToolId::new("nonexistent")?);
/// println!("Error: {}", error);
///
/// // Chain errors for context
/// let chained = IcarusError::ToolExecutionFailed {
///     tool_id: ToolId::new("calculator.add")?,
///     source: Box::new(error),
/// };
/// # Ok::<(), IcarusError>(())
/// ```
#[derive(Error, Debug, Clone, CandidType, Deserialize, Serialize)]
#[non_exhaustive]
pub enum IcarusError {
    /// Tool with the specified ID was not found.
    #[error("Tool not found: {0}")]
    ToolNotFound(crate::ToolId),

    /// Tool execution failed with an error.
    #[error("Tool execution failed for {tool_id}")]
    ToolExecutionFailed {
        /// The tool that failed to execute.
        tool_id: crate::ToolId,
        /// The underlying error that caused the failure.
        #[source]
        source: Box<IcarusError>,
    },

    /// Invalid tool identifier provided.
    #[error("Invalid tool ID: {0}")]
    InvalidToolId(String),

    /// Invalid user identifier provided.
    #[error("Invalid user ID: {0}")]
    InvalidUserId(String),

    /// Invalid session identifier provided.
    #[error("Invalid session ID: {0}")]
    InvalidSessionId(String),

    /// JSON-RPC protocol error.
    #[error("JSON-RPC error: {0}")]
    JsonRpcError(#[from] JsonRpcError),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    JsonError(String),

    /// Candid serialization/deserialization error.
    #[error("Candid error: {0}")]
    CandidError(String),

    /// Tool parameter validation error.
    #[error("Invalid parameter '{parameter}' for tool '{tool_id}': {message}")]
    InvalidParameter {
        /// The tool ID where the parameter error occurred.
        tool_id: crate::ToolId,
        /// The parameter name that was invalid.
        parameter: String,
        /// Description of what was invalid.
        message: String,
    },

    /// Tool schema validation error.
    #[error("Invalid tool schema for '{tool_id}': {message}")]
    InvalidSchema {
        /// The tool ID with the invalid schema.
        tool_id: crate::ToolId,
        /// Description of the schema issue.
        message: String,
    },

    /// Authentication or authorization error.
    #[error("Access denied: {0}")]
    AccessDenied(String),

    /// Rate limiting error.
    #[error("Rate limit exceeded for user {user_id}: {message}")]
    RateLimitExceeded {
        /// The user who exceeded the rate limit.
        user_id: crate::UserId,
        /// Additional context about the rate limit.
        message: String,
    },

    /// Resource limit exceeded (memory, compute, etc.).
    #[error("Resource limit exceeded: {resource} ({message})")]
    ResourceLimitExceeded {
        /// The type of resource that was exceeded.
        resource: String,
        /// Additional context about the limit.
        message: String,
    },

    /// Internal server error (should be rare).
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Network or external service error.
    #[error("External service error: {service} - {message}")]
    ExternalServiceError {
        /// The external service that failed.
        service: String,
        /// Error message from the service.
        message: String,
    },

    /// Timeout error for long-running operations.
    #[error("Operation timed out after {timeout_ms}ms: {operation}")]
    Timeout {
        /// The operation that timed out.
        operation: String,
        /// Timeout duration in milliseconds.
        timeout_ms: u64,
    },

    /// Invalid version string provided.
    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    /// Context-enriched error for better debugging and observability.
    #[error("{message}")]
    WithContext {
        /// The primary error message.
        message: String,
        /// The underlying error that caused this one.
        #[source]
        source: Box<IcarusError>,
        /// Additional structured context for debugging (boxed to reduce size).
        context: Box<ErrorContext>,
    },
}

/// Error severity levels for categorizing errors by impact.
///
/// This follows the pattern from `rust_best_practices.md` for structured error handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub enum ErrorSeverity {
    /// Low impact - warnings, deprecation notices.
    Low,
    /// Medium impact - recoverable errors, retryable failures.
    Medium,
    /// High impact - operation failures, data validation errors.
    High,
    /// Critical impact - system failures, security breaches.
    Critical,
}

/// Structured context information for enhanced error debugging.
///
/// This provides rich context following `rust_best_practices.md` patterns for
/// better error diagnosis and observability.
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct ErrorContext {
    /// Error severity level.
    pub severity: ErrorSeverity,
    /// Operation that was being performed when the error occurred.
    pub operation: Option<String>,
    /// Component or module where the error originated.
    pub component: Option<String>,
    /// Timestamp when the error occurred (milliseconds since Unix epoch).
    pub timestamp: u64,
    /// Additional key-value pairs for structured context.
    pub metadata: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    /// Creates a new error context with the specified severity.
    #[must_use]
    pub fn new(severity: ErrorSeverity) -> Self {
        Self {
            severity,
            operation: None,
            component: None,
            timestamp: crate::Timestamp::now().as_millis(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Sets the operation context.
    #[must_use]
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Sets the component context.
    #[must_use]
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = Some(component.into());
        self
    }

    /// Adds metadata key-value pair.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Adds multiple metadata entries.
    #[must_use]
    pub fn with_metadata_map(
        mut self,
        metadata: std::collections::HashMap<String, String>,
    ) -> Self {
        self.metadata.extend(metadata);
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new(ErrorSeverity::Medium)
    }
}

/// JSON-RPC specific error codes and messages.
///
/// Following the JSON-RPC 2.0 specification for error codes.
#[derive(Error, Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct JsonRpcError {
    /// Standard JSON-RPC error code.
    pub code: i32,
    /// Human-readable error message.
    pub message: String,
    /// Additional error data as JSON string (optional).
    pub data: Option<String>,
}

impl JsonRpcError {
    /// Creates a new JSON-RPC error.
    #[must_use]
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Creates a new JSON-RPC error with additional data.
    #[must_use]
    pub fn with_data(code: i32, message: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data.into()),
        }
    }

    /// Parse error (JSON-RPC -32700).
    #[must_use]
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(-32700, message)
    }

    /// Invalid request (JSON-RPC -32600).
    #[must_use]
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(-32600, message)
    }

    /// Method not found (JSON-RPC -32601).
    #[must_use]
    pub fn method_not_found(method: &str) -> Self {
        Self::new(-32601, format!("Method not found: {method}"))
    }

    /// Invalid parameters (JSON-RPC -32602).
    #[must_use]
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(-32602, message)
    }

    /// Internal error (JSON-RPC -32603).
    #[must_use]
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(-32603, message)
    }

    /// Server error (JSON-RPC -32000 to -32099).
    ///
    /// # Panics
    ///
    /// Panics if `code` is not in the range -32099 to -32000 (inclusive).
    #[must_use]
    pub fn server_error(code: i32, message: impl Into<String>) -> Self {
        assert!(
            (-32099..=-32000).contains(&code),
            "Invalid server error code"
        );
        Self::new(code, message)
    }
}

impl fmt::Display for JsonRpcError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON-RPC Error {}: {}", self.code, self.message)?;
        if let Some(data) = &self.data {
            write!(f, " (data: {data})")?;
        }
        Ok(())
    }
}

// Implement From for common error types to provide automatic conversion

impl From<serde_json::Error> for IcarusError {
    #[inline]
    fn from(err: serde_json::Error) -> Self {
        Self::JsonError(err.to_string())
    }
}

impl From<candid::Error> for IcarusError {
    #[inline]
    fn from(err: candid::Error) -> Self {
        Self::CandidError(err.to_string())
    }
}

// Additional From implementations for better ergonomics
impl From<std::io::Error> for IcarusError {
    #[inline]
    fn from(err: std::io::Error) -> Self {
        Self::InternalError(format!("IO error: {err}"))
    }
}

impl From<std::num::ParseIntError> for IcarusError {
    #[inline]
    fn from(err: std::num::ParseIntError) -> Self {
        Self::JsonError(format!("Parse error: {err}"))
    }
}

impl From<std::num::ParseFloatError> for IcarusError {
    #[inline]
    fn from(err: std::num::ParseFloatError) -> Self {
        Self::JsonError(format!("Parse error: {err}"))
    }
}

impl From<std::string::FromUtf8Error> for IcarusError {
    #[inline]
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::InternalError(format!("UTF-8 error: {err}"))
    }
}

// Additional From implementations following rust_best_practices.md patterns
impl From<fmt::Error> for IcarusError {
    #[inline]
    fn from(err: fmt::Error) -> Self {
        Self::InternalError(format!("Format error: {err}"))
    }
}

impl From<std::str::Utf8Error> for IcarusError {
    #[inline]
    fn from(err: std::str::Utf8Error) -> Self {
        Self::InternalError(format!("UTF-8 validation error: {err}"))
    }
}

impl From<std::array::TryFromSliceError> for IcarusError {
    #[inline]
    fn from(err: std::array::TryFromSliceError) -> Self {
        Self::InternalError(format!("Array conversion error: {err}"))
    }
}

impl From<std::collections::TryReserveError> for IcarusError {
    #[inline]
    fn from(err: std::collections::TryReserveError) -> Self {
        Self::ResourceLimitExceeded {
            resource: "memory".to_string(),
            message: format!("Memory allocation failed: {err}"),
        }
    }
}

impl From<std::time::SystemTimeError> for IcarusError {
    #[inline]
    fn from(err: std::time::SystemTimeError) -> Self {
        Self::InternalError(format!("System time error: {err}"))
    }
}

/// Result type alias for convenience.
///
/// This follows the pattern recommended in `rust_best_practices.md` for libraries.
pub type Result<T> = std::result::Result<T, IcarusError>;

/// Helper functions for creating common errors.
impl IcarusError {
    /// Creates a tool not found error.
    #[must_use]
    pub fn tool_not_found(tool_id: crate::ToolId) -> Self {
        Self::ToolNotFound(tool_id)
    }

    /// Creates a tool execution failed error with source.
    #[must_use]
    pub fn tool_execution_failed(tool_id: crate::ToolId, source: impl Into<Self>) -> Self {
        Self::ToolExecutionFailed {
            tool_id,
            source: Box::new(source.into()),
        }
    }

    /// Creates an invalid parameter error.
    #[must_use]
    pub fn invalid_parameter(
        tool_id: crate::ToolId,
        parameter: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::InvalidParameter {
            tool_id,
            parameter: parameter.into(),
            message: message.into(),
        }
    }

    /// Creates an access denied error.
    #[must_use]
    pub fn access_denied(message: impl Into<String>) -> Self {
        Self::AccessDenied(message.into())
    }

    /// Creates a rate limit exceeded error.
    #[must_use]
    pub fn rate_limit_exceeded(user_id: crate::UserId, message: impl Into<String>) -> Self {
        Self::RateLimitExceeded {
            user_id,
            message: message.into(),
        }
    }

    /// Creates an internal error.
    #[must_use]
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::InternalError(message.into())
    }

    /// Adds rich context to any error, following `rust_best_practices.md` patterns.
    ///
    /// This is similar to anyhow's `Context` trait but maintains type safety
    /// and IC compatibility with `CandidType`.
    #[must_use]
    pub fn with_context(self, message: impl Into<String>) -> Self {
        Self::WithContext {
            message: message.into(),
            source: Box::new(self),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Adds rich context with detailed error context information.
    #[must_use]
    pub fn with_rich_context(self, message: impl Into<String>, context: ErrorContext) -> Self {
        Self::WithContext {
            message: message.into(),
            source: Box::new(self),
            context: Box::new(context),
        }
    }

    /// Adds operation context for better debugging.
    #[must_use]
    pub fn with_operation(self, operation: impl Into<String>) -> Self {
        let context = ErrorContext::default().with_operation(operation);
        self.with_rich_context("Operation failed", context)
    }

    /// Adds component context for module-level error tracking.
    #[must_use]
    pub fn with_component(self, component: impl Into<String>) -> Self {
        let context = ErrorContext::default().with_component(component);
        self.with_rich_context("Component error", context)
    }

    /// Gets the error severity level for prioritization.
    #[must_use]
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::WithContext { context, .. } => context.severity,
            Self::AccessDenied(_) | Self::InvalidParameter { .. } | Self::InvalidSchema { .. } => {
                ErrorSeverity::High
            }
            Self::InternalError(_) | Self::ExternalServiceError { .. } | Self::Timeout { .. } => {
                ErrorSeverity::Critical
            }
            Self::RateLimitExceeded { .. } | Self::ResourceLimitExceeded { .. } => {
                ErrorSeverity::Medium
            }
            _ => ErrorSeverity::Medium,
        }
    }

    /// Extracts structured context information if available.
    #[must_use]
    pub fn context(&self) -> Option<&ErrorContext> {
        if let Self::WithContext { context, .. } = self {
            Some(context)
        } else {
            None
        }
    }

    /// Checks if this error is retryable based on its type.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ExternalServiceError { .. }
                | Self::Timeout { .. }
                | Self::RateLimitExceeded { .. }
                | Self::JsonRpcError(_)
        )
    }
}

/// Extension trait for Result types to add context chaining.
///
/// This provides the same ergonomics as anyhow's Context trait while
/// maintaining type safety and IC compatibility.
pub trait ResultExt<T> {
    /// Adds context to an error result.
    ///
    /// # Errors
    ///
    /// Returns an error with the original error as source and the provided context message.
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;

    /// Adds rich context to an error result.
    ///
    /// # Errors
    ///
    /// Returns an error with the original error as source, the provided message, and rich context.
    fn with_rich_context<F>(self, message: String, f: F) -> Result<T>
    where
        F: FnOnce() -> ErrorContext;
}

impl<T> ResultExt<T> for Result<T> {
    fn with_context<F>(self, f: F) -> Self
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| e.with_context(f()))
    }

    fn with_rich_context<F>(self, message: String, f: F) -> Self
    where
        F: FnOnce() -> ErrorContext,
    {
        self.map_err(|e| e.with_rich_context(message, f()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ToolId, UserId};

    #[test]
    fn test_error_creation() -> Result<()> {
        let tool_id = ToolId::new("test_tool")?;
        let user_id = UserId::new("test_user")?;

        // Test various error types
        let _tool_not_found = IcarusError::tool_not_found(tool_id.clone());
        let _access_denied = IcarusError::access_denied("Insufficient permissions");
        let _rate_limited = IcarusError::rate_limit_exceeded(user_id, "Too many requests");

        Ok(())
    }

    #[test]
    fn test_json_rpc_errors() {
        let parse_error = JsonRpcError::parse_error("Invalid JSON");
        assert_eq!(parse_error.code, -32700);

        let method_not_found = JsonRpcError::method_not_found("unknown_method");
        assert_eq!(method_not_found.code, -32601);
        assert!(method_not_found.message.contains("unknown_method"));
    }

    #[test]
    fn test_error_conversion() {
        let json_error =
            serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, "test"));
        let icarus_error: IcarusError = json_error.into();

        match icarus_error {
            IcarusError::JsonError(_) => (),
            _ => panic!("Expected JsonError"),
        }
    }

    #[test]
    fn test_error_chaining() -> Result<()> {
        let tool_id = ToolId::new("failing_tool")?;
        let inner_error = IcarusError::internal_error("Something went wrong");
        let outer_error = IcarusError::tool_execution_failed(tool_id, inner_error);

        // Test that the error displays correctly
        let error_string = outer_error.to_string();
        assert!(error_string.contains("failing_tool"));

        Ok(())
    }

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new(ErrorSeverity::High)
            .with_operation("test_operation")
            .with_component("test_component")
            .with_metadata("key1", "value1")
            .with_metadata("key2", "value2");

        assert_eq!(context.severity, ErrorSeverity::High);
        assert_eq!(context.operation, Some("test_operation".to_string()));
        assert_eq!(context.component, Some("test_component".to_string()));
        assert_eq!(context.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(context.metadata.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_with_context_chaining() {
        let base_error = IcarusError::internal_error("Base error");
        let contextualized = base_error
            .with_context("First context layer")
            .with_context("Second context layer");

        let error_string = contextualized.to_string();
        assert!(error_string.contains("Second context layer"));

        // Test that we can extract context
        assert!(contextualized.context().is_some());
        assert_eq!(contextualized.severity(), ErrorSeverity::Medium);
    }

    #[test]
    fn test_rich_context_chaining() {
        let context = ErrorContext::new(ErrorSeverity::Critical)
            .with_operation("critical_operation")
            .with_component("auth_module")
            .with_metadata("user_id", "12345")
            .with_metadata("session_id", "abcdef");

        let base_error = IcarusError::access_denied("Permission denied");
        let rich_error = base_error.with_rich_context("Authentication failed", context);

        // Test context extraction
        let extracted_context = rich_error
            .context()
            .expect("rich error should have context");
        assert_eq!(extracted_context.severity, ErrorSeverity::Critical);
        assert_eq!(
            extracted_context.operation,
            Some("critical_operation".to_string())
        );
        assert_eq!(extracted_context.component, Some("auth_module".to_string()));
        assert_eq!(
            extracted_context.metadata.get("user_id"),
            Some(&"12345".to_string())
        );
    }

    #[test]
    fn test_operation_and_component_context() {
        let base_error = IcarusError::internal_error("Something failed");
        let operation_error = base_error.clone().with_operation("database_query");
        let component_error = base_error.with_component("storage_module");

        // Test operation context
        assert!(operation_error.to_string().contains("Operation failed"));
        let op_context = operation_error
            .context()
            .expect("operation error should have context");
        assert_eq!(op_context.operation, Some("database_query".to_string()));

        // Test component context
        assert!(component_error.to_string().contains("Component error"));
        let comp_context = component_error
            .context()
            .expect("component error should have context");
        assert_eq!(comp_context.component, Some("storage_module".to_string()));
    }

    #[test]
    fn test_error_severity_levels() -> Result<()> {
        let tool_id = ToolId::new("test_tool")?;
        let user_id = UserId::new("test_user")?;

        // Test various error severity levels
        assert_eq!(
            IcarusError::access_denied("test").severity(),
            ErrorSeverity::High
        );
        assert_eq!(
            IcarusError::internal_error("test").severity(),
            ErrorSeverity::Critical
        );
        assert_eq!(
            IcarusError::rate_limit_exceeded(user_id, "test").severity(),
            ErrorSeverity::Medium
        );
        assert_eq!(
            IcarusError::invalid_parameter(tool_id, "param", "message").severity(),
            ErrorSeverity::High
        );

        Ok(())
    }

    #[test]
    fn test_retryable_errors() -> Result<()> {
        let user_id = UserId::new("test_user")?;

        // Test retryable errors
        assert!(IcarusError::ExternalServiceError {
            service: "test".to_string(),
            message: "test".to_string(),
        }
        .is_retryable());

        assert!(IcarusError::Timeout {
            operation: "test".to_string(),
            timeout_ms: 1000,
        }
        .is_retryable());

        assert!(IcarusError::rate_limit_exceeded(user_id, "test").is_retryable());

        // Test non-retryable errors
        assert!(!IcarusError::access_denied("test").is_retryable());
        assert!(!IcarusError::internal_error("test").is_retryable());

        Ok(())
    }

    #[test]
    fn test_result_ext_context() {
        use super::ResultExt;

        // Test with_context extension
        let result: std::result::Result<i32, IcarusError> =
            Err(IcarusError::internal_error("test"));
        let contextualized = result.with_context(|| "Additional context".to_string());

        assert!(contextualized.is_err());
        let error = contextualized.unwrap_err();
        assert!(error.to_string().contains("Additional context"));
    }

    #[test]
    fn test_additional_from_implementations() {
        // Test new From implementations
        let fmt_error = fmt::Error;
        let icarus_error: IcarusError = fmt_error.into();
        matches!(icarus_error, IcarusError::InternalError(_));

        let slice_error = <[u8; 4]>::try_from([1u8, 2u8, 3u8].as_slice()).unwrap_err();
        let icarus_error: IcarusError = slice_error.into();
        matches!(icarus_error, IcarusError::InternalError(_));
    }

    #[test]
    fn test_error_context_serialization() -> Result<()> {
        let context = ErrorContext::new(ErrorSeverity::High)
            .with_operation("test_op")
            .with_metadata("key", "value");

        // Test JSON serialization
        let json = serde_json::to_string(&context)?;
        let deserialized: ErrorContext = serde_json::from_str(&json)?;

        assert_eq!(deserialized.severity, ErrorSeverity::High);
        assert_eq!(deserialized.operation, Some("test_op".to_string()));
        assert_eq!(deserialized.metadata.get("key"), Some(&"value".to_string()));

        Ok(())
    }

    #[test]
    fn test_comprehensive_error_workflow() -> Result<()> {
        use super::ResultExt;

        let tool_id = ToolId::new("complex_tool")?;

        // Simulate a complex error scenario with rich context
        let process_data = || -> Result<String> {
            // This would fail
            Err(IcarusError::internal_error("Database connection failed"))
        };

        let enhanced_result = process_data()
            .with_context(|| "Failed to process user data".to_string())
            .map_err(|e| {
                e.with_rich_context(
                    "Critical system failure",
                    ErrorContext::new(ErrorSeverity::Critical)
                        .with_operation("user_data_processing")
                        .with_component("data_pipeline")
                        .with_metadata("tool_id", tool_id.as_str())
                        .with_metadata("retry_count", "3"),
                )
            });

        assert!(enhanced_result.is_err());
        let error = enhanced_result.unwrap_err();

        // Verify rich context is preserved
        let context = error
            .context()
            .expect("error should have context after adding rich context");
        assert_eq!(context.severity, ErrorSeverity::Critical);
        assert_eq!(context.operation, Some("user_data_processing".to_string()));
        assert_eq!(context.component, Some("data_pipeline".to_string()));
        assert_eq!(context.metadata.get("tool_id"), Some(&tool_id.to_string()));
        assert_eq!(context.metadata.get("retry_count"), Some(&"3".to_string()));

        // Test error is correctly categorized
        assert_eq!(error.severity(), ErrorSeverity::Critical);
        assert!(!error.is_retryable());

        Ok(())
    }
}
