//! ICP canister integration for Icarus MCP servers
//! 
//! This crate provides the canister implementation details for running
//! MCP servers on the Internet Computer.

pub mod auth;
pub mod memory;
pub mod state;
pub mod storage;
pub mod stable_ext;
pub mod tools;
pub mod endpoints;
pub mod lifecycle;
pub mod macros;

pub use auth::{
    User, AuthRole, AuthInfo, AuthAuditEntry, AuthAction,
    init_auth, authenticate, require_role, add_user, remove_user, 
    update_user_role, get_authorized_users, get_auth_audit,
    get_auth_status, list_users, get_user,
};
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

// Re-export the macros from icarus-derive
pub use icarus_derive::{icarus_tool, icarus_module, icarus_canister, IcarusStorable, IcarusStorage, IcarusType};

// Re-export commonly used macros (defined with #[macro_export])
// Note: These macros are automatically available at the crate root due to #[macro_export]

/// Comprehensive prelude for Icarus canister development
/// 
/// This module contains all the commonly used imports, so developers only need:
/// `use icarus_canister::prelude::*;`
pub mod prelude {
    // Core IC CDK types and macros
    pub use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
    pub use ic_cdk::{api, caller, print, trap, storage};
    pub use candid::{CandidType, Principal};
    
    // Stable structures for persistence
    pub use ic_stable_structures::{
        memory_manager::{MemoryId, MemoryManager, VirtualMemory},
        DefaultMemoryImpl, StableBTreeMap, StableCell, Storable,
        storable::Bound as StorableBound,
    };
    
    // Serde for serialization
    pub use serde::{Serialize, Deserialize};
    pub use serde_json;
    
    // Icarus core functionality
    pub use crate::{
        // Authentication system
        auth::*,
        // State management
        state::*,
        // Storage utilities
        storage::*,
        // Memory management
        memory::{get_memory, MEMORY_MANAGER},
        // Lifecycle hooks
        lifecycle::*,
        // HTTP endpoints - exclude get_owner to avoid conflict
        endpoints::{icarus_metadata, http_request, HttpRequest, HttpResponse, get_owner as get_canister_owner},
        
        // Macros for tool definition
        icarus_tool, icarus_module, IcarusStorable, IcarusStorage, IcarusType,
        
        // Memory management macros
        memory_id, init_memory, tool_metadata, stable_storage,
    };
    
    // Common type aliases
    pub type Memory = VirtualMemory<DefaultMemoryImpl>;
    pub type Map<K, V> = StableBTreeMap<K, V, Memory>;
    pub type Cell<T> = StableCell<T, Memory>;
    
    // Common Result type for tools
    pub type ToolResult<T> = Result<T, String>;
}