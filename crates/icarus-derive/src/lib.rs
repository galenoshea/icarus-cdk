//! Derive macros for Icarus CDK
//!
//! This crate provides all the procedural macros for building MCP servers
//! that run as Internet Computer canisters. It includes derive macros,
//! attribute macros, and function-like macros for a unified development experience.
//!
//! # Core Macros
//!
//! ## Function Macros
//! - **`icarus::auth!()`**: Generate authentication management functions
//! - **`icarus::mcp!()`**: Generate MCP tool discovery infrastructure
//! - **`icarus::wasi!()`**: Automatic WASI initialization for ecosystem libraries
//! - **`#[icarus::tool()]`**: Mark functions as MCP tools with auth
//!
//! ## Derive Macros
//! - **`#[derive(IcarusStorable)]`**: Generate stable storage implementations
//! - **`#[derive(IcarusStorage)]`**: Generate storage management code
//! - **`#[derive(IcarusType)]`**: Convenience macro for common type patterns
//!
//! # Usage
//!
//! ```rust,ignore
//! use icarus::prelude::*;
//! use ic_cdk_macros::export_candid;
//!
//! /// Process data with ML model
//! #[ic_cdk::update]
//! #[icarus::tool("Process data with ML model")]
//! pub async fn process_data(data: Vec<f32>) -> Result<Vec<f32>, String> {
//!     // Business logic here
//!     Ok(data)
//! }
//!
//! /// Public endpoint - no auth required
//! #[ic_cdk::query]
//! #[icarus::tool("Get service information", auth = "public")]
//! pub fn get_info() -> String {
//!     "Service information".to_string()
//! }
//!
//! // Add authentication and MCP infrastructure
//! icarus::auth!();
//! icarus::mcp!();
//! export_candid!();
//! ```
//!
//! This provides:
//! - MCP tool registration with automatic discovery
//! - Authentication checks on tool functions
//! - User management functions (add_user, remove_user, etc.)
//! - Tool discovery endpoint for MCP clients (get_tools)
//! - Stable storage derive macros for ICP persistence
//! - Proper Candid interface export

#![warn(missing_docs)]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod attr_tool;
mod auth_macro;
mod mcp_macro;
mod storage;
mod utils;
mod wasi_macro;

/// Attribute macro for marking functions as MCP tools.
///
/// This attribute macro adds authentication checks and registers the function
/// as an MCP tool for automatic discovery.
///
/// # Example
///
/// ```rust,ignore
/// #[ic_cdk::update]
/// #[icarus::tool("Analyze data with ML model")]
/// pub async fn analyze_data(data: Vec<f32>) -> Result<Vec<f32>, String> {
///     // Tool implementation
///     Ok(data)
/// }
/// ```
///
/// # Authentication
///
/// Tools can specify authentication requirements:
/// - `auth = "public"` - No authentication required
/// - `auth = "user"` - Any authenticated user (default)
/// - `auth = "admin"` - Admin role required
/// - `auth = "owner"` - Owner role required
///
/// ```rust,ignore
/// #[ic_cdk::update]
/// #[icarus::tool("Admin-only function", auth = "admin")]
/// pub fn admin_function() -> String {
///     "admin".to_string()
/// }
/// ```
#[proc_macro_attribute]
pub fn tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    attr_tool::expand(attr, item)
}

/// Function-like macro for generating MCP infrastructure.
///
/// This macro generates the MCP tool discovery function:
/// - `get_tools()` - Returns JSON list of available tools for MCP clients
///
/// # Example
///
/// ```rust,ignore
/// use icarus::prelude::*;
/// use ic_cdk_macros::export_candid;
///
/// #[ic_cdk::update]
/// #[icarus::tool("Process data")]
/// pub async fn process_data(data: String) -> Result<String, String> {
///     Ok(format!("Processed: {}", data))
/// }
///
/// icarus::mcp!();
/// export_candid!();
/// ```
#[proc_macro]
pub fn mcp(input: TokenStream) -> TokenStream {
    mcp_macro::expand(input)
}

/// Function-like macro for automatic WASI initialization.
///
/// This macro provides zero-boilerplate WASI support for canisters that use
/// ecosystem libraries requiring system interfaces. It automatically detects
/// if WASI is needed and initializes the polyfill transparently.
///
/// # Features
///
/// - **Automatic Detection**: Uses compile-time analysis to determine WASI needs
/// - **Lazy Initialization**: Initializes only when needed, avoiding conflicts
/// - **Zero Boilerplate**: No manual configuration or initialization code
/// - **Safe Integration**: Works seamlessly with auth!() and other macros
///
/// # Example
///
/// ```rust,ignore
/// use icarus::prelude::*;
/// use ic_cdk_macros::export_candid;
///
/// #[ic_cdk::update]
/// #[icarus::tool("Fetch data from API")]
/// pub async fn fetch_data(url: String) -> Result<String, String> {
///     // Uses reqwest - WASI automatically initialized
///     let response = reqwest::get(&url).await?;
///     Ok(response.text().await?)
/// }
///
/// icarus::auth!();
/// icarus::mcp!();
/// icarus::wasi!(); // Automatic WASI support
/// export_candid!();
/// ```
///
/// # Integration with icarus-wasi
///
/// This macro requires the `icarus-wasi` crate in your dependencies:
///
/// ```toml
/// [dependencies]
/// icarus-wasi = "0.8.0"
/// ```
#[proc_macro]
pub fn wasi(input: TokenStream) -> TokenStream {
    wasi_macro::expand(input)
}

/// Function-like macro for generating authentication management functions.
///
/// This macro generates comprehensive user management for marketplace deployment:
/// - `init(owner: Principal)` - Initialize with marketplace-provided owner
/// - `add_user(principal, role)` - Add users with role validation
/// - `remove_user(principal)` - Remove users (with restrictions)
/// - `update_user_role(principal, role)` - Change user roles (owner only)
/// - `get_user_role(principal)` - Check user's role
/// - `list_users()` - List all users (admin required)
/// - `get_current_user()` - Public self-check function
///
/// Note: Auth state is automatically preserved across upgrades using stable storage.
///
/// # Marketplace Deployment
///
/// ```bash
/// dfx deploy --argument '(principal "purchaser-principal-id")'
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use icarus::prelude::*;
/// use ic_cdk_macros::export_candid;
///
/// #[icarus::tool("Admin function", auth = "admin")]
/// #[ic_cdk::update]
/// pub async fn admin_function() -> String {
///     "admin operation".to_string()
/// }
///
/// icarus::auth!();
/// icarus::mcp!();
/// export_candid!();
/// ```
#[proc_macro]
pub fn auth(input: TokenStream) -> TokenStream {
    auth_macro::expand(input)
}

/// Derive macro for ICP storable types
///
/// # Examples
/// ```rust,ignore
/// use icarus_derive::IcarusStorable;
///
/// #[derive(IcarusStorable)]
/// struct MyData {
///     value: u64,
/// } // Uses default 1MB bound
///
/// #[derive(IcarusStorable)]
/// #[icarus_storable(unbounded)]
/// struct LargeData {
///     data: Vec<u8>,
/// } // Uses unbounded storage
///
/// #[derive(IcarusStorable)]
/// #[icarus_storable(max_size = "2MB")]
/// struct CustomData {
///     content: String,
/// } // Uses custom 2MB bound
/// ```
#[proc_macro_derive(IcarusStorable, attributes(icarus_storable))]
pub fn derive_icarus_storable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match storage::expand_icarus_storable(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Derive macro for simplified storage declaration
///
/// Generates stable storage declarations from a simple struct definition.
/// Automatically assigns memory IDs and handles initialization.
///
/// # Examples
/// ```rust,ignore
/// use icarus_derive::IcarusStorage;
/// use ic_stable_structures::StableBTreeMap;
/// use candid::Principal;
///
/// #[derive(Clone)]
/// struct MemoryEntry {
///     value: String,
/// }
///
/// #[derive(Clone)]
/// struct User {
///     name: String,
/// }
///
/// #[derive(IcarusStorage)]
/// struct Storage {
///     memories: StableBTreeMap<String, MemoryEntry>,
///     counter: u64,
///     users: StableBTreeMap<Principal, User>,
/// }
/// ```
///
/// This generates:
/// - Thread-local storage declarations
/// - Memory manager initialization
/// - Accessor methods for each field
#[proc_macro_derive(IcarusStorage)]
pub fn derive_icarus_storage(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match storage::expand_icarus_storage(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Derive macro for common Icarus type patterns
///
/// This is a convenience macro that combines IcarusStorable with sensible defaults.
/// You still need to derive the standard traits manually.
///
/// # Examples
/// ```rust,ignore
/// use serde::{Serialize, Deserialize};
/// use candid::CandidType;
/// use icarus_derive::IcarusType;
///
/// #[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusType)]
/// struct MemoryEntry {
///     id: String,
///     content: String,
///     created_at: u64,
/// }
/// ```
///
/// This is equivalent to:
/// ```rust,ignore
/// use serde::{Serialize, Deserialize};
/// use candid::CandidType;
/// use icarus_derive::IcarusStorable;
///
/// #[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
/// #[icarus_storable(unbounded)]
/// struct MemoryEntry {
///     id: String,
///     content: String,
///     created_at: u64,
/// }
/// ```
#[proc_macro_derive(IcarusType, attributes(icarus_storable))]
pub fn derive_icarus_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match storage::expand_icarus_type(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

// Helper function to parse size strings like "1MB", "2KB", etc.
pub(crate) fn parse_size_string(size: &str) -> u32 {
    let size = size.trim();
    if let Some(num_str) = size.strip_suffix("MB") {
        num_str.trim().parse::<u32>().unwrap_or(1) * 1024 * 1024
    } else if let Some(num_str) = size.strip_suffix("KB") {
        num_str.trim().parse::<u32>().unwrap_or(1) * 1024
    } else if let Some(num_str) = size.strip_suffix("B") {
        num_str.trim().parse::<u32>().unwrap_or(1024)
    } else {
        // Try to parse as raw bytes
        size.parse::<u32>().unwrap_or(1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_string() {
        assert_eq!(parse_size_string("1MB"), 1024 * 1024);
        assert_eq!(parse_size_string("2MB"), 2 * 1024 * 1024);
        assert_eq!(parse_size_string("512KB"), 512 * 1024);
        assert_eq!(parse_size_string("1024B"), 1024);
        assert_eq!(parse_size_string("1048576"), 1048576); // Raw bytes

        // Test with whitespace
        assert_eq!(parse_size_string(" 1MB "), 1024 * 1024);
        assert_eq!(parse_size_string(" 512 KB"), 512 * 1024);

        // Test invalid inputs (should use defaults)
        assert_eq!(parse_size_string("invalid"), 1024 * 1024); // Default 1MB
        assert_eq!(parse_size_string(""), 1024 * 1024);
        assert_eq!(parse_size_string("MB"), 1024 * 1024); // Default when parsing fails
    }

    #[test]
    fn test_parse_size_string_edge_cases() {
        // Test case sensitivity (should be case insensitive)
        assert_eq!(parse_size_string("1mb"), 1024 * 1024);
        assert_eq!(parse_size_string("1MB"), 1024 * 1024);

        // Test zero values
        assert_eq!(parse_size_string("0MB"), 0);
        assert_eq!(parse_size_string("0KB"), 0);
        assert_eq!(parse_size_string("0B"), 0);

        // Test large values
        assert_eq!(parse_size_string("1024MB"), 1024 * 1024 * 1024);

        // Test fractional parts (should be ignored)
        assert_eq!(parse_size_string("1.5MB"), 1024 * 1024); // Should parse as 1MB
    }
}
