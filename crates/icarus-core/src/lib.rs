// Copyright (c) 2025 Icarus Team. All Rights Reserved.
// Licensed under BSL-1.1. See LICENSE and NOTICE files.
// Signature verification and telemetry must remain intact.

// #![warn(missing_docs)] // TODO: Enable after adding all documentation

//! Core abstractions for building MCP servers on ICP
//!
//! This crate provides the fundamental traits and types for creating
//! Model Context Protocol servers that run as Internet Computer canisters.

pub mod certificate;
pub mod error;
pub mod lifecycle;
pub mod outcalls;
pub mod persistent;
pub mod prompts;
pub mod protocol;
pub mod registry;
pub mod resource;
pub mod response;
pub mod server;
pub mod session;
pub mod state;
pub mod tool;

pub use error::{IcarusError, Result, ToolError};
pub use resource::IcarusResource;
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
        error::{IcarusError, Result, ToolError},
        lifecycle::IcarusServerLifecycle,
        persistent::{IcarusPersistentState, TypedPersistentState},
        prompts::{Prompt, PromptBuilder, PromptRegistry},
        registry::IcarusToolRegistry,
        resource::IcarusResource,
        response::{tool_ok, tool_success, ToolStatus, ToolSuccess},
        server::IcarusServer,
        tool::IcarusTool,
    };
}
