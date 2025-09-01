#![warn(missing_docs)]

//! # Icarus SDK
//!
//! Build MCP (Model Context Protocol) servers that run as Internet Computer canisters.
//!
//! ## Overview
//!
//! Icarus SDK enables developers to create persistent AI tools by combining:
//! - **MCP**: The Model Context Protocol for AI assistant tools
//! - **ICP**: The Internet Computer's blockchain-based compute platform
//!
//! Write your MCP servers in Rust, deploy them to ICP, and they run forever with built-in persistence.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use icarus::prelude::*;
//! use candid::{CandidType, Deserialize};
//! use serde::Serialize;
//!
//! // Define your data structures
//! #[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
//! pub struct MemoryEntry {
//!     id: String,
//!     content: String,
//!     created_at: u64,
//! }
//!
//! // Define your tools with automatic metadata generation
//! #[icarus_module]
//! mod tools {
//!     use super::*;
//!     
//!     #[update]
//!     #[icarus_tool("Store a new memory")]
//!     pub fn memorize(content: String) -> Result<String, String> {
//!         Ok(format!("Stored: {}", content))
//!     }
//! }
//! ```

// Re-export core functionality
pub use icarus_core as core;

// Re-export derive macros
pub use icarus_derive as derive;

// Re-export canister functionality
pub use icarus_canister as canister;

// Re-export commonly used items
pub use icarus_derive::{icarus_module, icarus_tool};

// Prelude module for convenient imports
pub mod prelude {
    pub use crate::canister::prelude::*;
    pub use crate::derive::{icarus_module, icarus_tool};
}