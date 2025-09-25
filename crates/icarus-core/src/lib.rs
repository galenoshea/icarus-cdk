// Copyright (c) 2025 Icarus Team. All Rights Reserved.
// Licensed under BSL-1.1. See LICENSE and NOTICE files.
// Signature verification and telemetry must remain intact.

// Missing docs warnings disabled during active development

//! Core abstractions for building MCP servers on ICP
//!
//! This crate provides the fundamental traits and types for creating
//! Model Context Protocol servers that run as Internet Computer canisters.

pub mod auth;
pub mod compatibility;
pub mod dual_protocol;
pub mod error;
pub mod macros;
pub mod memory;
pub mod protocol;
pub mod provider;
pub mod response;
pub mod server;
pub mod stable_ext;
pub mod storage;
pub mod tool;
pub mod tools;

// Canister-specific modules (feature-gated)
#[cfg(feature = "canister")]
pub mod http;
#[cfg(feature = "canister")]
pub mod timers;

// MCP modules (feature-gated)
#[cfg(feature = "mcp")]
pub mod mcp;

#[cfg(feature = "canister")]
pub use auth::{is_authenticated, is_owner, AuthRole};
pub use compatibility::{
    IcarusParam, IcarusReturn, IcarusTool as IcarusToolCompatible, ToolResult,
};
pub use dual_protocol::{
    detect_format, parse_input, serialize_output, CandidOrJson, MemoryStats, ProtocolError,
    ProtocolFormat,
};
pub use error::{IcarusError, Result, ToolError};
pub use provider::{
    AuthConfig, ErrorConfig, GenerateServiceMetadata, IcarusToolMethod, IcarusToolProvider,
    ServiceConfig, ServiceMetadata, ToolMethodMetadata,
};
pub use response::{tool_ok, tool_success, ToolStatus, ToolSuccess};
pub use server::IcarusServer;
#[cfg(feature = "canister")]
pub use storage::IcarusStorable;
pub use tool::IcarusTool;
#[cfg(feature = "canister")]
pub use tools::{create_schema_for, ToolInfo};

// MCP re-exports (feature-gated)
#[cfg(feature = "mcp")]
pub use mcp::{
    ConfigError, Connected, McpConfig, McpConfigBuilder, McpServer, McpServerBuilder, Serving,
    Uninitialized,
};

// Re-export canister-specific types (feature-gated)
#[cfg(feature = "canister")]
pub use http::{HttpConfig, HttpError, HttpResult};
#[cfg(feature = "canister")]
pub use timers::{TimerError, TimerInfo, TimerType};

/// Prelude module for convenient imports
///
/// Import everything you need with:
/// ```
/// use icarus_core::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        error::{IcarusError, Result, ToolError},
        provider::{
            AuthConfig, ErrorConfig, GenerateServiceMetadata, IcarusToolMethod, IcarusToolProvider,
            ServiceConfig, ServiceMetadata, ToolMethodMetadata,
        },
        response::{tool_ok, tool_success, ToolStatus, ToolSuccess},
        server::IcarusServer,
        tool::IcarusTool,
    };

    // Canister-specific prelude (feature-gated)
    #[cfg(feature = "canister")]
    pub use crate::{
        auth::{is_authenticated, is_owner, AuthRole},
        tools::{create_schema_for, ToolInfo},
    };

    // MCP prelude (feature-gated)
    #[cfg(feature = "mcp")]
    pub use crate::mcp::{
        ConfigError, Connected, McpConfig, McpConfigBuilder, McpServer, McpServerBuilder, Serving,
        Uninitialized,
    };
}
