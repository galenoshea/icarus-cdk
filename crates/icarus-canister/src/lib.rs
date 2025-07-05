//! ICP canister integration for Icarus MCP servers
//! 
//! This crate provides the canister implementation details for running
//! MCP servers on the Internet Computer.

pub mod memory;
pub mod state;
pub mod storage;
pub mod tools;
pub mod endpoints;
pub mod lifecycle;
pub mod macros;

pub use state::IcarusCanisterState;
pub use storage::{StableMap, StableCounter};
pub use endpoints::{
    icarus_metadata,
    http_request,
    HttpRequest,
    HttpResponse,
};
pub use lifecycle::{init, post_upgrade, pre_upgrade};

// Re-export the icarus_tool, icarus_module, and icarus_canister attributes from icarus-derive
pub use icarus_derive::{icarus_tool, icarus_module, icarus_canister};