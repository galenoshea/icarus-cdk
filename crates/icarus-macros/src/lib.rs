//! # Icarus Macros
//!
//! Procedural macros for the Icarus CDK following `rust_best_practices.md` patterns.
//!
//! This crate provides the core procedural macros that enable the declarative API
//! for building MCP servers on Internet Computer canisters:
//!
//! - `#[tool]` - Attribute macro for automatically generating MCP tool wrappers
//! - `mcp!{}` - Declarative macro for generating canister initialization code
//!
//! # Examples
//!
//! ```rust,ignore
//! use icarus_macros::{tool, mcp};
//!
//! /// Add two numbers together
//! #[tool]
//! fn add(a: f64, b: f64) -> f64 {
//!     a + b
//! }
//!
//! mcp! {}
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(unsafe_code)]

mod error;
mod mcp;
mod tool;
mod utils;

use proc_macro::TokenStream;

/// Attribute macro that converts a function into an MCP tool.
///
/// This macro automatically generates the necessary boilerplate to expose
/// a Rust function as an MCP tool, including parameter validation, JSON-RPC
/// handling, and error conversion.
///
/// # Examples
///
/// ```rust
/// use icarus_macros::tool;
///
/// /// Adds two numbers together
/// #[tool]
/// fn add(a: f64, b: f64) -> f64 {
///     a + b
/// }
///
/// /// Greets a user with optional title
/// #[tool]
/// fn greet(name: String, title: Option<String>) -> String {
///     match title {
///         Some(t) => format!("Hello, {} {}!", t, name),
///         None => format!("Hello, {}!", name),
///     }
/// }
/// ```
///
/// # Generated Code
///
/// The macro generates:
/// - Parameter validation structures
/// - JSON schema generation
/// - Error handling and conversion
/// - MCP protocol compliance wrappers
///
/// # Restrictions
///
/// - Functions must have simple parameter types that implement `serde::Deserialize`
/// - Return types must implement `serde::Serialize` or be convertible to `String`
/// - Async functions are supported
/// - Generic functions are not currently supported
#[proc_macro_attribute]
pub fn tool(args: TokenStream, input: TokenStream) -> TokenStream {
    tool::tool_impl(args.into(), input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Declarative macro for generating MCP server initialization code.
///
/// This macro generates all the necessary canister endpoints and infrastructure
/// to run an MCP server on the Internet Computer, including:
///
/// - Tool discovery endpoints
/// - Tool execution endpoints
/// - JSON-RPC request/response handling
/// - Candid interface generation
///
/// # Examples
///
/// ```rust,ignore
/// use icarus_macros::{tool, mcp};
///
/// #[tool]
/// fn add(a: f64, b: f64) -> f64 {
///     a + b
/// }
///
/// // Minimal configuration - generates all required endpoints
/// mcp! {}
///
/// // With optional configuration
/// mcp! {
///     name = "calculator",
///     description = "A simple calculator service",
///     version = "1.0.0"
/// }
/// ```
///
/// # Configuration Options
///
/// - `name`: Service name (defaults to crate name)
/// - `description`: Service description
/// - `version`: Service version (defaults to crate version)
/// - `auth`: Enable authentication (optional)
/// - `rate_limit`: Enable rate limiting (optional)
///
/// # Generated Endpoints
///
/// The macro generates these IC canister endpoints:
/// - `mcp_list_tools() -> String` (query)
/// - `mcp_call_tool(request: String) -> String` (update)
/// - `mcp_server_info() -> String` (query)
#[proc_macro]
pub fn mcp(input: TokenStream) -> TokenStream {
    mcp::mcp_impl(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

// Note: VERSION constant removed as proc-macro crates cannot export non-proc-macro items
