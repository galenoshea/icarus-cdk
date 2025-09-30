//! # Icarus Core
//!
//! Core types and MCP protocol integration for the Icarus CDK.
//!
//! This crate provides the foundational types for building MCP (Model Context Protocol)
//! servers on Internet Computer canisters, following the patterns from `rust_best_practices.md`:
//!
//! - **Type Safety**: Extensive use of newtype pattern for domain concepts
//! - **Performance**: Zero-copy patterns, const functions, pre-allocation
//! - **Error Handling**: Comprehensive error types with context using `thiserror`
//! - **Testing**: Property-based tests for invariants
//!
//! # Examples
//!
//! ```rust
//! use icarus_core::{IcarusError, Tool, ToolId};
//! use std::sync::Arc;
//!
//! # fn main() -> Result<(), IcarusError> {
//! // Type-safe tool identifiers
//! let tool_id = ToolId::new("calculator_add")?;
//!
//! // RMCP-native tool definition
//! let tool = Tool::new(
//!     "calculator_add",
//!     "Adds two numbers",
//!     Arc::new(serde_json::Map::new()),
//! );
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(unsafe_code)]

pub mod error;
pub mod newtypes;
pub mod protocol;
pub mod rmcp_types;
pub mod tool;
pub mod version;

/// Authentication and authorization module with stable memory persistence
pub mod auth;

/// Legacy types for backward compatibility (deprecated in 0.9.0)
///
/// All types in this module have RMCP-native replacements and will be removed
/// in a future major version. See module documentation for migration guide.
pub mod legacy;

// Re-export commonly used types for convenience
pub use error::IcarusError;
pub use newtypes::{SessionId, Timestamp, ToolId, UserId};
pub use version::{Version, VersionReq};

// Re-export RMCP types for RMCP-native protocol support
pub use rmcp_types::{
    CallToolResult, CanisterId, Content, JsonRpcError, JsonRpcRequest, JsonRpcResponse, MethodName,
    Tool, ToolAnnotations,
};

// Re-export legacy types with deprecation warnings for backward compatibility
#[allow(deprecated)]
pub use legacy::{
    LegacyTool, LegacyToolCall, LegacyToolResult, SmallParameters, ToolBuilder, ToolParameter,
    ToolSchema,
};

/// Type alias for convenience - follows `rust_best_practices.md`
pub type Result<T> = std::result::Result<T, IcarusError>;

/// Version information for the core library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum tool name length for validation
pub const MAX_TOOL_NAME_LENGTH: usize = 255;

/// Maximum description length for validation
pub const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// Maximum parameter count per tool
pub const MAX_PARAMETER_COUNT: usize = 50;
