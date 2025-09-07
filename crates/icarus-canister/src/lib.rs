// Copyright (c) 2025 Icarus Team. All Rights Reserved.
// Licensed under BSL-1.1. See LICENSE and NOTICE files.
// Signature verification and telemetry must remain intact.

// #![warn(missing_docs)] // TODO: Enable after adding all documentation

//! ICP canister integration for Icarus MCP servers
//!
//! This crate provides the canister implementation details for running
//! MCP servers on the Internet Computer.

pub mod auth;
pub mod auth_tools;
pub mod easy_storage;
pub mod endpoints;
pub mod http;
pub mod lifecycle;
pub mod macros;
pub mod memory;
pub mod result;
pub mod stable_ext;
pub mod state;
pub mod storage;
pub mod timers;
pub mod tools;

pub use auth::{
    add_user, authenticate, get_auth_audit, get_auth_status, get_authorized_users, get_user,
    init_auth, list_users, remove_user, require_any_of_roles, require_exact_role,
    require_none_of_roles, require_role_or_higher, update_user_role, AuthAction, AuthAuditEntry,
    AuthInfo, AuthRole, User,
};
pub use endpoints::{
    get_owner as get_canister_owner, http_request, icarus_metadata, HttpRequest, HttpResponse,
};
pub use lifecycle::{init, init_with_caller, post_upgrade, pre_upgrade};
pub use stable_ext::{StableBTreeMapExt, StableCellExt};
pub use state::{assert_owner, get_owner, is_owner, IcarusCanisterState};
pub use storage::{StableCounter, StableMap};

// Re-export the macros from icarus-derive
pub use icarus_derive::{
    icarus_canister, icarus_module, icarus_tool, IcarusStorable, IcarusStorage, IcarusType,
};

// Re-export commonly used macros (defined with #[macro_export])
// Note: These macros are automatically available at the crate root due to #[macro_export]

/// Comprehensive prelude for Icarus canister development
///
/// This module contains all the commonly used imports, so developers only need:
/// `use icarus_canister::prelude::*;`
pub mod prelude {
    // Core IC CDK types and macros
    pub use candid::{CandidType, Principal};
    pub use ic_cdk::{api, caller, print, storage, trap};
    pub use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};

    // Stable structures for persistence
    pub use ic_stable_structures::{
        memory_manager::{MemoryId, MemoryManager, VirtualMemory},
        storable::Bound as StorableBound,
        DefaultMemoryImpl, StableBTreeMap, StableCell, Storable,
    };

    // Serde for serialization
    pub use serde::{Deserialize, Serialize};
    pub use serde_json;

    // Icarus core functionality
    pub use crate::{
        // Authentication system
        auth::{
            add_user, authenticate, get_auth_audit, get_auth_status, get_authorized_users,
            get_user, init_auth, list_users, remove_user, require_any_of_roles, require_exact_role,
            require_none_of_roles, require_role_or_higher, update_user_role, AuthAction,
            AuthAuditEntry, AuthInfo, AuthRole, User,
        },
        // Easy storage patterns
        easy_storage::{CounterCell, StorageCell, StorageMap},

        // HTTP endpoints - exclude get_owner to avoid conflict
        endpoints::{
            get_owner as get_canister_owner, http_request, icarus_metadata, HttpRequest,
            HttpResponse,
        },

        // HTTP outcalls for external data
        http,

        icarus_module,
        icarus_storage,
        // Macros for tool definition
        icarus_tool,
        init_memory,
        // Lifecycle hooks
        lifecycle::*,
        // Memory management
        memory::{get_memory, MEMORY_MANAGER},
        // Memory management macros
        memory_id,
        // Result types and error handling
        result::{IcarusError, IcarusResult, TrapExt},

        stable_storage,
        // State management
        state::*,
        // Storage utilities
        storage::*,
        // Timer system for autonomous operations
        timers,

        tool_metadata,
        IcarusStorable,
        IcarusStorage,
        IcarusType,
    };

    // Common type aliases
    pub type Memory = VirtualMemory<DefaultMemoryImpl>;
    pub type Map<K, V> = StableBTreeMap<K, V, Memory>;
    pub type Cell<T> = StableCell<T, Memory>;

    // Common Result type for tools
    pub type ToolResult<T> = Result<T, String>;
}
