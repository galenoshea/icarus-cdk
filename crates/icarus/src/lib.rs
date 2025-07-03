//! Icarus SDK - Build MCP servers that run on the Internet Computer
//! 
//! Icarus SDK enables developers to create Model Context Protocol (MCP) servers
//! that run as Internet Computer Protocol (ICP) canisters, combining AI tool
//! integration with blockchain persistence.
//! 
//! # Quick Start
//! 
//! ```ignore
//! use icarus::prelude::*;
//! 
//! #[derive(IcarusTool)]
//! #[icarus_tool(name = "remember", description = "Store a fact")]
//! struct RememberTool;
//! 
//! #[icarus_server(name = "memory-server", version = "1.0.0")]
//! pub struct MemoryServer {
//!     facts: Vec<String>,
//! }
//! ```

// Re-export all subcrates
pub use icarus_core as core;
pub use icarus_derive as derive;
pub use icarus_canister as canister;
pub use icarus_types as types;

// Re-export commonly used items
pub use icarus_core::{IcarusError, IcarusServer, IcarusTool, IcarusResource};
pub use icarus_derive::{IcarusTool, icarus_server, icarus_tools, icarus_tool, IcarusStorable};

// Re-export key dependencies
pub use rmcp;
pub use ic_cdk;
pub use candid;

/// Prelude module for common imports
pub mod prelude {
    pub use crate::{
        IcarusError,
        IcarusServer,
        IcarusTool,
        IcarusResource,
        IcarusTool as DeriveTool,
        icarus_server,
        icarus_tools,
        icarus_tool,
        IcarusStorable,
    };
    
    // Common types needed for examples
    pub use serde::{Serialize, Deserialize as SerdeDeserialize};
    pub use candid::{CandidType, Deserialize};
    pub use ic_cdk::api;
    
    // Type aliases for common return types
    pub type ToolResult = Result<serde_json::Value, IcarusError>;
}

/// Generate Candid interface for the canister
#[macro_export]
macro_rules! export_candid {
    () => {
        #[ic_cdk::query(name = "__get_candid_interface_tmp_hack")]
        fn export_candid() -> String {
            ic_cdk::export_candid!();
            __export_service()
        }
    };
}