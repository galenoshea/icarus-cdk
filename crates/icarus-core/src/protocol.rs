//! MCP (Model Context Protocol) types and JSON-RPC integration.
//!
//! This module provides type-safe wrappers for the MCP protocol following
//! `rust_best_practices.md` patterns. It includes JSON-RPC request/response
//! handling with proper validation and error handling.

use std::borrow::Cow;

use candid::{CandidType, Deserialize};
use serde::Serialize;

use crate::{error::JsonRpcError, IcarusError, SessionId, ToolId};

/// JSON-RPC 2.0 request wrapper with validation and zero-copy optimization.
///
/// Provides type-safe handling of JSON-RPC requests following the specification.
/// Uses `Cow<str>` for zero-copy when possible, falling back to owned strings when needed.
/// Parameters and ID are stored as JSON strings for Candid compatibility.
///
/// # Examples
///
/// ```rust
/// use icarus_core::protocol::JsonRpcRequest;
/// use std::borrow::Cow;
///
/// let request = JsonRpcRequest::new(
///     "2.0",
///     "tools/call",
///     Some(Cow::Borrowed(r#"{"name": "add", "arguments": {"a": 1, "b": 2}}"#)),
///     Some(Cow::Borrowed("request-123"))
/// )?;
/// # Ok::<(), icarus_core::IcarusError>(())
/// ```
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct JsonRpcRequest<'a> {
    /// JSON-RPC version (must be "2.0").
    #[serde(borrow)]
    pub jsonrpc: Cow<'a, str>,
    /// Method name to call.
    #[serde(borrow)]
    pub method: Cow<'a, str>,
    /// Method parameters as JSON string (optional).
    #[serde(skip_serializing_if = "Option::is_none", borrow)]
    pub params: Option<Cow<'a, str>>,
    /// Request ID for correlation (optional for notifications).
    #[serde(skip_serializing_if = "Option::is_none", borrow)]
    pub id: Option<Cow<'a, str>>,
}

impl<'a> JsonRpcRequest<'a> {
    /// Creates a new JSON-RPC request with validation.
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::JsonRpcError` if the version is not "2.0".
    pub fn new(
        jsonrpc: impl Into<Cow<'a, str>>,
        method: impl Into<Cow<'a, str>>,
        params: impl Into<Option<Cow<'a, str>>>,
        id: impl Into<Option<Cow<'a, str>>>,
    ) -> Result<Self, IcarusError> {
        let jsonrpc = jsonrpc.into();
        if jsonrpc != "2.0" {
            return Err(JsonRpcError::invalid_request("JSON-RPC version must be '2.0'").into());
        }

        let method = method.into();
        if method.is_empty() {
            return Err(JsonRpcError::invalid_request("Method name cannot be empty").into());
        }

        Ok(Self {
            jsonrpc,
            method,
            params: params.into(),
            id: id.into(),
        })
    }

    /// Returns true if this is a notification (no ID).
    #[must_use]
    #[inline]
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }

    /// Extracts typed parameters from the request.
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::JsonError` if deserialization fails.
    pub fn extract_params<T>(&self) -> Result<T, IcarusError>
    where
        T: for<'de> Deserialize<'de>,
    {
        match &self.params {
            Some(params_str) => Ok(serde_json::from_str(params_str)?),
            None => Err(JsonRpcError::invalid_params("Missing parameters").into()),
        }
    }
}

/// JSON-RPC 2.0 response wrapper with zero-copy optimization.
///
/// Represents either a successful result or an error response.
/// Result is stored as JSON string for Candid compatibility.
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct JsonRpcResponse<'a> {
    /// JSON-RPC version (always "2.0").
    #[serde(borrow)]
    pub jsonrpc: Cow<'a, str>,
    /// Successful result as JSON string (mutually exclusive with error).
    #[serde(skip_serializing_if = "Option::is_none", borrow)]
    pub result: Option<Cow<'a, str>>,
    /// Error information (mutually exclusive with result).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Request ID for correlation (must match request).
    #[serde(borrow)]
    pub id: Cow<'a, str>,
}

impl<'a> JsonRpcResponse<'a> {
    /// Creates a successful response.
    #[must_use]
    #[inline]
    pub fn success(result: impl Into<Cow<'a, str>>, id: impl Into<Cow<'a, str>>) -> Self {
        Self {
            jsonrpc: Cow::Borrowed("2.0"),
            result: Some(result.into()),
            error: None,
            id: id.into(),
        }
    }

    /// Creates an error response.
    #[must_use]
    #[inline]
    pub fn error(error: JsonRpcError, id: impl Into<Cow<'a, str>>) -> Self {
        Self {
            jsonrpc: Cow::Borrowed("2.0"),
            result: None,
            error: Some(error),
            id: id.into(),
        }
    }

    /// Returns true if this response indicates success.
    #[must_use]
    #[inline]
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Extracts the result value if successful.
    ///
    /// # Errors
    ///
    /// Returns the error if the response indicates failure.
    #[inline]
    pub fn into_result(self) -> Result<Cow<'a, str>, JsonRpcError> {
        match (self.result, self.error) {
            (Some(result), None) => Ok(result),
            (None, Some(error)) => Err(error),
            _ => Err(JsonRpcError::internal_error("Invalid response state")),
        }
    }
}

/// MCP tool call request parameters with zero-copy optimization.
///
/// Represents a request to execute a specific tool with provided arguments.
/// Arguments and metadata are stored as JSON strings for Candid compatibility.
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct ToolCall<'a> {
    /// Name of the tool to call.
    pub name: ToolId,
    /// Arguments to pass to the tool as JSON string.
    #[serde(default, borrow)]
    pub arguments: Cow<'a, str>,
    /// Optional session context for stateful tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
    /// Optional metadata for the call as JSON string.
    #[serde(skip_serializing_if = "Option::is_none", borrow)]
    pub metadata: Option<Cow<'a, str>>,
}

impl<'a> ToolCall<'a> {
    /// Creates a new tool call.
    #[must_use]
    #[inline]
    pub fn new(name: ToolId) -> Self {
        Self {
            name,
            arguments: Cow::Borrowed("{}"),
            session_id: None,
            metadata: None,
        }
    }

    /// Sets the arguments as a JSON string.
    #[must_use]
    #[inline]
    pub fn with_arguments(mut self, arguments: impl Into<Cow<'a, str>>) -> Self {
        self.arguments = arguments.into();
        self
    }

    /// Sets the session ID for the tool call.
    #[must_use]
    #[inline]
    pub fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Sets metadata as a JSON string.
    #[must_use]
    #[inline]
    pub fn with_metadata(mut self, metadata: impl Into<Cow<'a, str>>) -> Self {
        self.metadata = Some(metadata.into());
        self
    }

    /// Extracts typed arguments from the tool call.
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::JsonError` if deserialization fails.
    pub fn extract_arguments<T>(&self) -> Result<T, IcarusError>
    where
        T: for<'de> Deserialize<'de>,
    {
        Ok(serde_json::from_str(&self.arguments)?)
    }
}

/// MCP tool execution result with zero-copy optimization.
///
/// Represents the outcome of executing a tool, including both success and error cases.
/// All structured data is stored as JSON strings for Candid compatibility.
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ToolResult<'a> {
    /// Successful tool execution with result data.
    Success {
        /// The result value from the tool as JSON string.
        #[serde(borrow)]
        result: Cow<'a, str>,
        /// Optional metadata about the execution as JSON string.
        #[serde(skip_serializing_if = "Option::is_none", borrow)]
        metadata: Option<Cow<'a, str>>,
    },
    /// Tool execution failed with an error.
    Error {
        /// Error message.
        #[serde(borrow)]
        message: Cow<'a, str>,
        /// Optional error code.
        #[serde(skip_serializing_if = "Option::is_none", borrow)]
        code: Option<Cow<'a, str>>,
        /// Optional additional error details as JSON string.
        #[serde(skip_serializing_if = "Option::is_none", borrow)]
        details: Option<Cow<'a, str>>,
    },
    /// Tool execution is still in progress (for async tools).
    Pending {
        /// Estimated completion percentage (0-100).
        #[serde(skip_serializing_if = "Option::is_none")]
        progress: Option<u8>,
        /// Human-readable status message.
        #[serde(skip_serializing_if = "Option::is_none", borrow)]
        status: Option<Cow<'a, str>>,
    },
}

impl<'a> ToolResult<'a> {
    /// Creates a successful result.
    #[must_use]
    #[inline]
    pub fn success(result: impl Into<Cow<'a, str>>) -> Self {
        Self::Success {
            result: result.into(),
            metadata: None,
        }
    }

    /// Creates a successful result with metadata.
    #[must_use]
    #[inline]
    pub fn success_with_metadata(
        result: impl Into<Cow<'a, str>>,
        metadata: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self::Success {
            result: result.into(),
            metadata: Some(metadata.into()),
        }
    }

    /// Creates an error result.
    #[must_use]
    #[inline]
    pub fn error(message: impl Into<Cow<'a, str>>) -> Self {
        Self::Error {
            message: message.into(),
            code: None,
            details: None,
        }
    }

    /// Creates an error result with additional details.
    #[must_use]
    #[inline]
    pub fn error_with_details(
        message: impl Into<Cow<'a, str>>,
        code: impl Into<Cow<'a, str>>,
        details: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self::Error {
            message: message.into(),
            code: Some(code.into()),
            details: Some(details.into()),
        }
    }

    /// Creates a pending result.
    #[must_use]
    #[inline]
    pub fn pending() -> Self {
        Self::Pending {
            progress: None,
            status: None,
        }
    }

    /// Creates a pending result with progress.
    #[must_use]
    #[inline]
    pub fn pending_with_progress(progress: u8, status: impl Into<Cow<'a, str>>) -> Self {
        Self::Pending {
            progress: Some(progress.min(100)),
            status: Some(status.into()),
        }
    }

    /// Returns true if the result indicates success.
    #[must_use]
    #[inline]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Returns true if the result indicates an error.
    #[must_use]
    #[inline]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Returns true if the result indicates pending execution.
    #[must_use]
    #[inline]
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    /// Extracts the success value if available.
    ///
    /// # Errors
    ///
    /// Returns `IcarusError` if the result is not a success.
    #[inline]
    pub fn into_success(self) -> Result<Cow<'a, str>, IcarusError> {
        match self {
            Self::Success { result, .. } => Ok(result),
            Self::Error { message, code, .. } => Err(IcarusError::InternalError(format!(
                "Tool failed: {} (code: {:?})",
                message,
                code.unwrap_or(Cow::Borrowed("unknown"))
            ))),
            Self::Pending { .. } => Err(IcarusError::InternalError(
                "Tool execution still pending".to_string(),
            )),
        }
    }
}

/// Helper functions for converting between different representations.
impl<'a> ToolResult<'a> {
    /// Converts from a Result type.
    #[must_use]
    #[inline]
    pub fn from_result<T, E>(result: Result<T, E>) -> Self
    where
        T: Into<Cow<'a, str>>,
        E: std::fmt::Display,
    {
        match result {
            Ok(value) => Self::success(value.into()),
            Err(error) => Self::error(Cow::Owned(error.to_string())),
        }
    }
}

impl<'a, T, E> From<Result<T, E>> for ToolResult<'a>
where
    T: Into<Cow<'a, str>>,
    E: std::fmt::Display,
{
    #[inline]
    fn from(result: Result<T, E>) -> Self {
        Self::from_result(result)
    }
}

impl From<IcarusError> for ToolResult<'_> {
    #[inline]
    fn from(error: IcarusError) -> Self {
        Self::error(Cow::Owned(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn test_json_rpc_request_creation() -> Result<(), IcarusError> {
        let request = JsonRpcRequest::new(
            "2.0",
            "test_method",
            Some(Cow::Borrowed(r#"{"param": "value"}"#)),
            Some(Cow::Borrowed("test-id")),
        )?;

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "test_method");
        assert!(!request.is_notification());

        Ok(())
    }

    #[test]
    fn test_json_rpc_request_validation() {
        // Invalid version
        assert!(JsonRpcRequest::new("1.0", "test", None, None).is_err());

        // Empty method
        assert!(JsonRpcRequest::new("2.0", "", None, None).is_err());
    }

    #[test]
    fn test_json_rpc_response() {
        let success_response = JsonRpcResponse::success("result", "id");
        assert!(success_response.is_success());

        let error_response =
            JsonRpcResponse::error(JsonRpcError::internal_error("Test error"), "id".to_string());
        assert!(!error_response.is_success());
    }

    #[test]
    fn test_tool_call() -> Result<(), IcarusError> {
        let tool_id = ToolId::new("test_tool")?;
        let session_id = SessionId::new("sess_test_session")?;

        let call = ToolCall::new(tool_id)
            .with_arguments(r#"{"param1": "value1", "param2": "42"}"#)
            .with_session(session_id)
            .with_metadata(r#"{"source": "test"}"#);

        assert!(!call.arguments.is_empty());
        assert!(call.session_id.is_some());
        assert!(call.metadata.is_some());

        Ok(())
    }

    #[test]
    fn test_tool_result_variants() {
        let success = ToolResult::success("result");
        assert!(success.is_success());
        assert!(!success.is_error());
        assert!(!success.is_pending());

        let error = ToolResult::error("Test error");
        assert!(!error.is_success());
        assert!(error.is_error());
        assert!(!error.is_pending());

        let pending = ToolResult::pending_with_progress(50, "Processing");
        assert!(!pending.is_success());
        assert!(!pending.is_error());
        assert!(pending.is_pending());
    }

    #[test]
    fn test_tool_result_from_result() {
        let ok_result: Result<String, &str> = Ok("success".to_string());
        let tool_result = ToolResult::from_result(ok_result);
        assert!(tool_result.is_success());

        let err_result: Result<String, &str> = Err("error");
        let tool_result = ToolResult::from_result(err_result);
        assert!(tool_result.is_error());
    }
}
