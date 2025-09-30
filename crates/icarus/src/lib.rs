//! # Icarus CDK
//!
//! **Build MCP servers on Internet Computer canisters with ease.**
//!
//! Icarus CDK is a modern Rust framework for creating MCP (Model Context Protocol)
//! servers that run on Internet Computer canisters. It provides a declarative API
//! with automatic tool discovery, type-safe execution, and comprehensive error handling.
//!
//! ## Features
//!
//! - **🔧 Declarative API**: Use `#[tool]` to automatically expose Rust functions as MCP tools
//! - **🚀 Zero-overhead**: Compile-time tool registration with <10ms execution times
//! - **🛡️ Type Safety**: Comprehensive type system with domain newtypes and validation
//! - **⚡ Performance**: Zero-copy patterns, memory efficiency, and SIMD optimizations
//! - **🌐 IC Native**: Built specifically for Internet Computer canister architecture
//! - **📊 Observability**: Built-in metrics, tracing, and performance monitoring
//! - **🔄 Async Support**: Optional async execution for I/O-bound tools
//! - **🧪 Well Tested**: Comprehensive test suite with property-based testing
//!
//! ## Quick Start
//!
//! Add Icarus to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! icarus = "0.9.0"
//! ```
//!
//! Create your first MCP server:
//!
//! ```rust,ignore
//! use icarus::prelude::*;
//!
//! /// Add two numbers together
//! #[tool]
//! fn add(a: f64, b: f64) -> f64 {
//!     a + b
//! }
//!
//! /// Calculate the square of a number
//! #[tool]
//! fn square(x: f64) -> f64 {
//!     x * x
//! }
//!
//! // Generate the MCP server
//! mcp! {
//!     name = "calculator",
//!     description = "A simple calculator service",
//!     version = "1.0.0"
//! }
//! ```
//!
//! Deploy to Internet Computer:
//!
//! ```bash
//! dfx deploy
//! ```
//!
//! ## Architecture
//!
//! Icarus CDK follows a layered architecture:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     Your Application                        │
//! │                   #[tool] functions                         │
//! ├─────────────────────────────────────────────────────────────┤
//! │                    icarus (facade)                          │
//! │              Public API and re-exports                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │   icarus-macros     │    icarus-runtime    │  icarus-core   │
//! │  #[tool], mcp!{}    │   Tool execution     │  Core types    │
//! │  Proc macros        │   Registry, cache    │  Protocols     │
//! ├─────────────────────────────────────────────────────────────┤
//! │              Internet Computer (IC)                         │
//! │            Canister Runtime Environment                     │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Core Concepts
//!
//! ### Tools
//!
//! Tools are Rust functions decorated with `#[tool]` that become available as MCP tools:
//!
//! ```rust,ignore
//! use icarus::prelude::*;
//!
//! /// Greet a user with optional title
//! #[tool]
//! fn greet(name: String, title: Option<String>) -> String {
//!     match title {
//!         Some(t) => format!("Hello, {} {}!", t, name),
//!         None => format!("Hello, {}!", name),
//!     }
//! }
//! ```
//!
//! ### Type Safety
//!
//! Icarus uses domain-specific newtypes for enhanced type safety:
//!
//! ```rust
//! use icarus::{ToolId, UserId, SessionId};
//!
//! // These are all distinct types that prevent mixing up IDs
//! let tool_id = ToolId::new("calculator_add")?;
//! let user_id = UserId::new("user_12345")?;
//! let session_id = SessionId::new("session_abcdef")?;
//! # Ok::<(), icarus::IcarusError>(())
//! ```
//!
//! ### Error Handling
//!
//! Comprehensive error handling with rich context:
//!
//! ```rust,ignore
//! use icarus::prelude::*;
//!
//! #[tool]
//! fn divide(a: f64, b: f64) -> Result<f64, String> {
//!     if b == 0.0 {
//!         Err("Division by zero".to_string())
//!     } else {
//!         Ok(a / b)
//!     }
//! }
//! ```
//!
//! ### Performance
//!
//! Icarus is designed for high performance:
//!
//! - **Sub-10ms execution**: Most tools execute in under 10 milliseconds
//! - **Zero-copy patterns**: Using `Cow<str>` and references where possible
//! - **Memory efficiency**: `SmallVec` for small collections, pre-allocation
//! - **Compile-time optimization**: Tool registration happens at compile time
//!
//! ## Examples
//!
//! ### Basic Calculator
//!
//! ```rust,ignore
//! use icarus::prelude::*;
//!
//! #[tool]
//! fn add(a: f64, b: f64) -> f64 { a + b }
//!
//! #[tool]
//! fn multiply(a: f64, b: f64) -> f64 { a * b }
//!
//! mcp! {
//!     name = "calculator",
//!     description = "Basic arithmetic operations"
//! }
//! ```
//!
//! ### Async Tools
//!
//! ```rust,ignore
//! use icarus::prelude::*;
//!
//! #[tool]
//! async fn fetch_data(url: String) -> Result<String, String> {
//!     // Async operations are fully supported
//!     match reqwest::get(&url).await {
//!         Ok(response) => response.text().await.map_err(|e| e.to_string()),
//!         Err(e) => Err(e.to_string()),
//!     }
//! }
//! ```
//!
//! ### Stateful Tools
//!
//! ```rust,ignore
//! use icarus::prelude::*;
//! use std::collections::HashMap;
//!
//! thread_local! {
//!     static COUNTERS: std::cell::RefCell<HashMap<String, u64>> =
//!         std::cell::RefCell::new(HashMap::new());
//! }
//!
//! #[tool]
//! fn increment_counter(session: String) -> u64 {
//!     COUNTERS.with(|counters| {
//!         let mut map = counters.borrow_mut();
//!         let counter = map.entry(session).or_insert(0);
//!         *counter += 1;
//!         *counter
//!     })
//! }
//! ```
//!
//! ## Best Practices
//!
//! 1. **Use descriptive tool names**: Tool names become part of the MCP API
//! 2. **Include documentation**: Doc comments become tool descriptions
//! 3. **Handle errors gracefully**: Use `Result` types for fallible operations
//! 4. **Keep tools focused**: Each tool should do one thing well
//! 5. **Validate inputs**: Use newtypes and validation for safety
//! 6. **Test thoroughly**: Write unit tests for all tool functions
//!
//! ## Deployment
//!
//! Deploy your MCP server to Internet Computer:
//!
//! ```bash
//! # Initialize dfx project (if not already done)
//! dfx new my_mcp_server
//! cd my_mcp_server
//!
//! # Build and deploy
//! dfx build
//! dfx deploy
//!
//! # Test your tools
//! dfx canister call my_mcp_server mcp_list_tools
//! ```
//!
//! ## Performance Tuning
//!
//! For production deployments, consider these optimizations:
//!
//! ```toml
//! [profile.release]
//! opt-level = 3
//! lto = "fat"
//! codegen-units = 1
//! strip = true
//! panic = "abort"
//! ```
//!
//! ## Monitoring and Observability
//!
//! ```rust,ignore
//! use icarus::prelude::*;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut executor = ToolExecutor::new().with_cache();
//!
//! // Create a tool call
//! let tool_id = ToolId::new("my_tool")?;
//! let tool_call = ToolCall::new(tool_id);
//!
//! // Execute tools and get metrics
//! let result = executor.execute(tool_call).await?;
//! let metrics = executor.metrics();
//!
//! println!("Success rate: {:.2}%", metrics.success_rate());
//! println!("Average execution time: {:.2}ms", metrics.avg_execution_time_ms);
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(unsafe_code)]

// Re-export all public APIs from core crates
pub use icarus_core::{
    // Errors
    IcarusError,
    JsonRpcError,

    // Protocol types
    JsonRpcRequest,
    JsonRpcResponse,
    LegacyToolCall as ToolCall,
    LegacyToolResult as ToolResult,

    SessionId,
    Timestamp,

    // Core types
    Tool,
    ToolBuilder,
    // Domain newtypes
    ToolId,
    ToolParameter,
    ToolSchema,

    UserId,
    MAX_DESCRIPTION_LENGTH,
    MAX_PARAMETER_COUNT,
    // Constants
    VERSION,
};

pub use icarus_runtime::{
    execute_tool,

    find_tool,

    // Registry operations
    list_tools,
    // Runtime errors
    RuntimeError,
    RuntimeResult,
    // Runtime execution
    ToolExecutor,
};

// Re-export procedural macros
pub use icarus_macros::{mcp, tool};

/// Prelude module for convenient imports.
///
/// This module contains the most commonly used types and traits.
/// Import it with `use icarus::prelude::*;` to get started quickly.
///
/// # Examples
///
/// ```rust,ignore
/// use icarus::prelude::*;
///
/// #[tool]
/// fn hello(name: String) -> String {
///     format!("Hello, {}!", name)
/// }
///
/// mcp! {}
/// ```
pub mod prelude {
    pub use crate::{
        // Common execution functions
        execute_tool,
        list_tools,

        mcp,

        // Essential macros
        tool,
        // Error types
        IcarusError,
        RuntimeError,
        // Core types everyone needs
        Tool,
        ToolCall,
        ToolId,

        ToolResult,
    };
}

/// Runtime version of the Icarus CDK.
pub const CDK_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_set() {
        assert!(!CDK_VERSION.is_empty());
        assert!(CDK_VERSION.contains('.'));
    }

    #[test]
    fn test_prelude_exports() {
        // Test that prelude exports are accessible
        use prelude::*;

        // Should be able to create a ToolId
        let tool_id = ToolId::new("test").unwrap();
        assert_eq!(tool_id.as_str(), "test");
    }

    #[test]
    fn test_core_types_available() {
        // Test that core types are re-exported correctly
        let tool_id = ToolId::new("test_tool").unwrap();
        let user_id = UserId::new("test_user").unwrap();
        let session_id = SessionId::new("test_session").unwrap();

        assert_eq!(tool_id.as_str(), "test_tool");
        assert_eq!(user_id.as_str(), "test_user");
        assert_eq!(session_id.as_str(), "test_session");
    }

    #[test]
    fn test_tool_call_creation() {
        let tool_id = ToolId::new("test").unwrap();
        let tool_call = ToolCall::new(tool_id).with_arguments(r#"{"key": "value"}"#);

        assert_eq!(tool_call.name.as_str(), "test");
        assert!(tool_call.arguments.contains("value"));
    }

    #[test]
    fn test_tool_result_creation() {
        let success = ToolResult::success("test result");
        let error = ToolResult::error("test error");

        match success {
            ToolResult::Success { result, .. } => assert_eq!(result, "test result"),
            _ => panic!("Expected success result"),
        }

        match error {
            ToolResult::Error { message, .. } => assert_eq!(message, "test error"),
            _ => panic!("Expected error result"),
        }
    }
}
