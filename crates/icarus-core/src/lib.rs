// Copyright (c) 2025 Icarus Team. All Rights Reserved.
// Licensed under BSL-1.1. See LICENSE and NOTICE files.
// Signature verification and telemetry must remain intact.

// Missing docs warnings disabled during active development

//! Core abstractions for building MCP servers on ICP
//!
//! This crate provides the fundamental traits and types for creating
//! Model Context Protocol servers that run as Internet Computer canisters.

pub mod builder;
pub mod compatibility;
pub mod error;
pub mod extensions;
pub mod lifecycle;
pub mod persistent;
pub mod protocol;
pub mod provider;
pub mod registry;
pub mod response;
pub mod server;
pub mod session;
pub mod state;
pub mod tool;

pub use builder::{ServerBuilder, StorageBuilder, ToolBuilder};
pub use compatibility::{
    IcarusParam, IcarusReturn, IcarusTool as IcarusToolCompatible, ToolResult,
};
pub use error::{IcarusError, Result, ToolError};
pub use extensions::{ExtensionProvider, InitError, InitRequirements, InitializationExtension};
pub use provider::{
    AuthConfig, ErrorConfig, GenerateServiceMetadata, IcarusToolMethod, IcarusToolProvider,
    ServiceConfig, ServiceMetadata, ToolMethodMetadata,
};
pub use response::{tool_ok, tool_success, ToolStatus, ToolSuccess};
pub use server::IcarusServer;
pub use tool::IcarusTool;

/// Prelude module for convenient imports
///
/// Import everything you need with:
/// ```
/// use icarus_core::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        builder::{ServerBuilder, StorageBuilder, ToolBuilder},
        error::{IcarusError, Result, ToolError},
        extensions::{ExtensionProvider, InitError, InitRequirements, InitializationExtension},
        lifecycle::IcarusServerLifecycle,
        persistent::{IcarusPersistentState, TypedPersistentState},
        provider::{
            AuthConfig, ErrorConfig, GenerateServiceMetadata, IcarusToolMethod, IcarusToolProvider,
            ServiceConfig, ServiceMetadata, ToolMethodMetadata,
        },
        registry::IcarusToolRegistry,
        response::{tool_ok, tool_success, ToolStatus, ToolSuccess},
        server::IcarusServer,
        tool::IcarusTool,
    };
}
