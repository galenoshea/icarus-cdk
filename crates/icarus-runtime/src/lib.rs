//! # Icarus Runtime
//!
//! Runtime and execution engine for the Icarus CDK.
//!
//! This crate provides the core runtime infrastructure for executing MCP tools
//! within Internet Computer canisters. It manages tool registration, discovery,
//! and execution through a distributed slice registry pattern.
//!
//! # Features
//!
//! - **Tool Registry**: Automatic tool discovery using `linkme` distributed slices
//! - **Execution Engine**: Type-safe tool execution with comprehensive error handling
//! - **Async Support**: Optional async execution for I/O-bound tools (feature `async`)
//! - **Performance**: Zero-allocation registry access with <10ms execution times
//! - **Memory Safety**: RAII resource management with proper cleanup
//!
//! # Architecture
//!
//! The runtime follows a distributed registry pattern where tools are automatically
//! registered at compile time using the `linkme` crate. This allows for:
//!
//! - Zero-runtime registration overhead
//! - Compile-time tool discovery
//! - Type-safe tool execution
//! - Memory-efficient tool storage
//!
//! # Examples
//!
//! ## Tool Registration
//!
//! Tools are automatically registered when using the `#[tool]` macro:
//!
//! ```rust,ignore
//! use icarus_macros::tool;
//!
//! #[tool]
//! fn add(a: f64, b: f64) -> f64 {
//!     a + b
//! }
//! ```
//!
//! ## Tool Execution
//!
//! ```rust
//! use icarus_runtime::execute_tool;
//! use icarus_core::{LegacyToolCall as ToolCall, ToolId};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let tool_call = ToolCall::new(ToolId::new("add")?)
//!     .with_arguments(r#"{"a": 5.0, "b": 3.0}"#);
//!
//! let result = execute_tool(tool_call).await?;
//! if let Ok(success_value) = result.into_success() {
//!     println!("Result: {}", success_value);
//! }
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(unsafe_code)]

mod error;
mod executor;
mod registry;

pub use error::{ErrorSeverity, RuntimeError, RuntimeResult};
pub use executor::{execute_tool, ExecutionMetrics, ToolExecutor, ToolExecutorTrait};
pub use registry::{find_tool, list_tools, RegistryStats, SyncToolExecutor, ToolRegistry};

#[cfg(feature = "async")]
pub use registry::AsyncToolExecutor;

// Re-export core types for convenience
pub use icarus_core::{IcarusError, Tool, ToolId};
pub use icarus_core::{LegacyToolCall as ToolCall, LegacyToolResult as ToolResult};

/// Runtime version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Distributed slice for tool registry.
///
/// This slice is populated at compile time by the `#[tool]` attribute macro.
/// Each tool function automatically registers itself in this slice through
/// the `linkme` crate, enabling zero-overhead tool discovery.
///
/// # Safety
///
/// This slice is safe to access from multiple threads as tool functions
/// are immutable once registered at compile time.
#[linkme::distributed_slice]
pub static TOOL_REGISTRY: [fn() -> Tool] = [..];

/// Distributed slice for executor initialization functions.
///
/// This slice is populated at compile time by the `#[tool]` attribute macro.
/// Each registration function in this slice is called during runtime initialization
/// to register tool executors with the `ToolRegistry`.
///
/// # Safety
///
/// This slice is safe to access from multiple threads as registration functions
/// are immutable once compiled.
#[linkme::distributed_slice]
pub static EXECUTOR_INIT: [fn()] = [..];

/// Initializes all tool executors by calling their registration functions.
///
/// This function should be called once during canister initialization or before
/// the first tool execution. It iterates through all registered executor initialization
/// functions and calls them to register executors with the `ToolRegistry`.
///
/// # Examples
///
/// ```rust
/// use icarus_runtime::initialize_executors;
///
/// // Call during canister initialization
/// initialize_executors();
/// ```
pub fn initialize_executors() {
    // Ensure the executor registry is initialized
    ToolRegistry::initialize_executors();

    // Call all executor registration functions
    for init_fn in EXECUTOR_INIT {
        init_fn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_set() {
        // VERSION is set by Cargo and guaranteed to be non-empty
        assert!(VERSION.contains('.'));
    }

    #[test]
    fn test_registry_exists() {
        // The registry should exist even if empty
        #[allow(clippy::type_complexity, clippy::no_effect_underscore_binding)]
        let _tools: &[fn() -> Tool] = &TOOL_REGISTRY;
    }
}
