//! Legacy types for backward compatibility.
//!
//! This module contains types from earlier versions of Icarus that have been replaced
//! with RMCP-native equivalents. These types are maintained for backward compatibility
//! but will be removed in a future major version.
//!
//! # Migration Guide
//!
//! The following types have been superseded by RMCP-native equivalents:
//!
//! - `LegacyToolCall` → Use `JsonRpcRequest` from rmcp_types
//! - `LegacyToolResult` → Use `CallToolResult` from rmcp_types
//! - `LegacyTool` → Use `Tool` from rmcp_types
//!
//! # Examples
//!
//! **Before (Legacy)**:
//! ```ignore
//! use icarus_core::{LegacyTool, ToolBuilder};
//!
//! let tool = ToolBuilder::new()
//!     .name("add")
//!     .description("Adds two numbers")
//!     .build()
//!     .expect("Valid tool");
//! ```
//!
//! **After (RMCP-Native)**:
//! ```rust
//! use icarus_core::Tool;
//! use std::sync::Arc;
//!
//! let tool = Tool::new(
//!     "add",
//!     "Adds two numbers",
//!     Arc::new(serde_json::Map::new()),
//! );
//! ```

#[deprecated(
    since = "0.9.0",
    note = "Use `JsonRpcRequest` from `rmcp_types` module instead"
)]
pub use crate::protocol::ToolCall as LegacyToolCall;

#[deprecated(
    since = "0.9.0",
    note = "Use `CallToolResult` from `rmcp_types` module instead"
)]
pub use crate::protocol::ToolResult as LegacyToolResult;

#[deprecated(since = "0.9.0", note = "Use `Tool` from `rmcp_types` module instead")]
pub use crate::tool::Tool as LegacyTool;

#[deprecated(
    since = "0.9.0",
    note = "Use `Tool::new()` directly instead of builder pattern"
)]
pub use crate::tool::ToolBuilder;

#[deprecated(
    since = "0.9.0",
    note = "Internal type - use `Tool` from `rmcp_types` module"
)]
pub use crate::tool::ToolParameter;

#[deprecated(
    since = "0.9.0",
    note = "Internal type - use `Tool` from `rmcp_types` module"
)]
pub use crate::tool::ToolSchema;

#[deprecated(
    since = "0.9.0",
    note = "Internal type - use inline parameter definitions with `Tool::new()`"
)]
pub use crate::tool::SmallParameters;
