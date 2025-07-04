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

pub use state::IcarusCanisterState;
pub use storage::{StableMap, StableCounter};
pub use endpoints::{
    icarus_mcp_request, 
    icarus_capabilities,
    http_request,
    HttpRequest,
    HttpResponse,
};
pub use lifecycle::{init, post_upgrade, pre_upgrade};