//! RMCP protocol types integration for Icarus CDK.
//!
//! This module re-exports types from the `rmcp` crate and provides IC-specific
//! newtype wrappers for type safety, following patterns from `rust_best_practices.md`.
//!
//! # Architecture
//!
//! Canisters speak RMCP protocol natively - this means:
//! - Tools are defined using `rmcp::model::Tool` structs
//! - Responses use `rmcp::model::CallToolResult`
//! - Protocol uses `rmcp::model::JsonRpcRequest` and `JsonRpcResponse`
//!
//! The bridge layer is a thin wrapper that forwards these RMCP-compliant messages
//! between Claude Desktop (MCP client) and IC canisters.

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::fmt;

use crate::error::IcarusError;

// Re-export RMCP model types for convenience
pub use rmcp::model::{
    CallToolResult, Content, JsonRpcError, JsonRpcRequest, JsonRpcResponse, Tool, ToolAnnotations,
};

/// Newtype wrapper for IC canister Principal with type safety.
///
/// This provides a domain-specific wrapper around `candid::Principal` to prevent
/// mixing up canister IDs with other identifiers, following the newtype pattern
/// from `rust_best_practices.md` Section 3.
///
/// # Examples
///
/// ```rust
/// use icarus_core::CanisterId;
///
/// // From text representation
/// let canister_id = CanisterId::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai")?;
///
/// // Access underlying Principal
/// let principal = canister_id.as_principal();
/// # Ok::<(), icarus_core::IcarusError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, CandidType, Deserialize, Serialize)]
pub struct CanisterId(Principal);

impl CanisterId {
    /// Creates a new `CanisterId` from a text representation.
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::InvalidParameter` if the text is not a valid Principal.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::CanisterId;
    ///
    /// let canister_id = CanisterId::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai")?;
    /// # Ok::<(), icarus_core::IcarusError>(())
    /// ```
    pub fn from_text(text: &str) -> Result<Self, IcarusError> {
        Principal::from_text(text)
            .map(Self)
            .map_err(|e| IcarusError::InvalidParameter {
                tool_id: crate::ToolId::new("system").unwrap_or_else(|_| unreachable!()),
                parameter: "canister_id".to_string(),
                message: format!("Invalid canister ID: {e}"),
            })
    }

    /// Creates a new `CanisterId` from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::InvalidParameter` if the bytes don't form a valid Principal.
    pub fn from_slice(slice: &[u8]) -> Result<Self, IcarusError> {
        Principal::try_from_slice(slice)
            .map(Self)
            .map_err(|e| IcarusError::InvalidParameter {
                tool_id: crate::ToolId::new("system").unwrap_or_else(|_| unreachable!()),
                parameter: "canister_id".to_string(),
                message: format!("Invalid canister ID bytes: {e}"),
            })
    }

    /// Creates a new `CanisterId` from a Principal.
    ///
    /// This is a const constructor that wraps a Principal without validation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::CanisterId;
    /// use candid::Principal;
    ///
    /// // When you already have a Principal
    /// let principal = Principal::management_canister();
    /// let canister_id = CanisterId::from_principal(principal);
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_principal(principal: Principal) -> Self {
        Self(principal)
    }

    /// Returns a reference to the underlying Principal.
    #[must_use]
    #[inline]
    pub const fn as_principal(&self) -> &Principal {
        &self.0
    }

    /// Converts to the underlying Principal, consuming self.
    #[must_use]
    #[inline]
    pub const fn into_principal(self) -> Principal {
        self.0
    }

    /// Returns the text representation of this canister ID.
    #[must_use]
    pub fn to_text(&self) -> String {
        self.0.to_text()
    }

    /// Returns the raw bytes of the Principal.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl fmt::Display for CanisterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Principal> for CanisterId {
    fn from(principal: Principal) -> Self {
        Self(principal)
    }
}

impl From<CanisterId> for Principal {
    fn from(canister_id: CanisterId) -> Self {
        canister_id.0
    }
}

/// Newtype wrapper for IC method names with validation.
///
/// Method names must be valid Candid method identifiers.
///
/// # Examples
///
/// ```rust
/// use icarus_core::MethodName;
///
/// let method = MethodName::new("get_user")?;
/// assert_eq!(method.as_str(), "get_user");
/// # Ok::<(), icarus_core::IcarusError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, CandidType, Deserialize, Serialize)]
pub struct MethodName(String);

impl MethodName {
    /// Creates a new `MethodName` with validation.
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::InvalidParameter` if the method name is invalid:
    /// - Empty string
    /// - Contains whitespace
    /// - Contains invalid characters
    pub fn new(name: impl Into<String>) -> Result<Self, IcarusError> {
        let name = name.into();

        if name.is_empty() {
            return Err(IcarusError::InvalidParameter {
                tool_id: crate::ToolId::new("system").unwrap_or_else(|_| unreachable!()),
                parameter: "method_name".to_string(),
                message: "Method name cannot be empty".to_string(),
            });
        }

        if name.contains(char::is_whitespace) {
            return Err(IcarusError::InvalidParameter {
                tool_id: crate::ToolId::new("system").unwrap_or_else(|_| unreachable!()),
                parameter: "method_name".to_string(),
                message: "Method name cannot contain whitespace".to_string(),
            });
        }

        // Method names should be valid identifiers
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(IcarusError::InvalidParameter {
                tool_id: crate::ToolId::new("system").unwrap_or_else(|_| unreachable!()),
                parameter: "method_name".to_string(),
                message: "Method name contains invalid characters".to_string(),
            });
        }

        Ok(Self(name))
    }

    /// Returns the method name as a string slice.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes self and returns the inner String.
    #[must_use]
    #[inline]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for MethodName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for MethodName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canister_id_from_text() {
        // Valid canister ID
        let canister_id = CanisterId::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai");
        assert!(canister_id.is_ok());

        // Invalid canister ID
        let invalid = CanisterId::from_text("invalid");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_canister_id_display() {
        let principal = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        let canister_id = CanisterId::from(principal);

        assert_eq!(canister_id.to_string(), "rrkah-fqaaa-aaaaa-aaaaq-cai");
    }

    #[test]
    fn test_canister_id_conversions() {
        let principal = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        let canister_id = CanisterId::from(principal);

        // Test reference
        assert_eq!(canister_id.as_principal(), &principal);

        // Test into
        let back_to_principal: Principal = canister_id.into();
        assert_eq!(back_to_principal, principal);
    }

    #[test]
    fn test_method_name_validation() {
        // Valid method names
        assert!(MethodName::new("get_user").is_ok());
        assert!(MethodName::new("mcp_list_tools").is_ok());
        assert!(MethodName::new("process_data_v2").is_ok());

        // Invalid method names
        assert!(MethodName::new("").is_err());
        assert!(MethodName::new("invalid name").is_err());
        assert!(MethodName::new("invalid-name").is_err());
        assert!(MethodName::new("invalid.name").is_err());
    }

    #[test]
    fn test_method_name_display() {
        let method = MethodName::new("get_user").unwrap();
        assert_eq!(method.to_string(), "get_user");
        assert_eq!(method.as_str(), "get_user");
    }

    #[test]
    fn test_method_name_as_ref() {
        let method = MethodName::new("process_data").unwrap();
        let s: &str = method.as_ref();
        assert_eq!(s, "process_data");
    }

    #[test]
    fn test_canister_id_from_slice() {
        let principal = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        let bytes = principal.as_slice();

        let canister_id = CanisterId::from_slice(bytes);
        assert!(canister_id.is_ok());
        assert_eq!(canister_id.unwrap().as_principal(), &principal);
    }
}
