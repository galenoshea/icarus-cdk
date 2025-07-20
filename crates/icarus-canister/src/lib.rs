//! ICP canister integration for Icarus MCP servers
//! 
//! This crate provides the canister implementation details for running
//! MCP servers on the Internet Computer.

pub mod memory;
pub mod state;
pub mod storage;
pub mod stable_ext;
pub mod tools;
pub mod endpoints;
pub mod lifecycle;
pub mod macros;

pub use state::{IcarusCanisterState, assert_owner, is_owner, get_owner};
pub use storage::{StableMap, StableCounter};
pub use stable_ext::{StableBTreeMapExt, StableCellExt};
pub use endpoints::{
    icarus_metadata,
    http_request,
    HttpRequest,
    HttpResponse,
    get_owner as get_canister_owner,
};
pub use lifecycle::{init, init_with_caller, post_upgrade, pre_upgrade};

// Re-export the icarus_tool, icarus_module, icarus_canister, and IcarusStorable from icarus-derive
pub use icarus_derive::{icarus_tool, icarus_module, icarus_canister, IcarusStorable};