//! Core abstractions for building MCP servers on ICP
//! 
//! This crate provides the fundamental traits and types for creating
//! Model Context Protocol servers that run as Internet Computer canisters.

pub mod error;
pub mod protocol;
pub mod response;
pub mod server;
pub mod state;
pub mod tool;
pub mod resource;
pub mod lifecycle;
pub mod registry;
pub mod persistent;
pub mod prompts;
pub mod outcalls;
pub mod session;
pub mod certificate;

pub use error::{IcarusError, Result, ToolError};
pub use response::{ToolSuccess, ToolStatus, tool_success, tool_ok};
pub use server::IcarusServer;
pub use tool::IcarusTool;
pub use resource::IcarusResource;

// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        error::{IcarusError, Result, ToolError},
        response::{ToolSuccess, ToolStatus, tool_success, tool_ok},
        server::IcarusServer,
        tool::IcarusTool,
        resource::IcarusResource,
        lifecycle::IcarusServerLifecycle,
        registry::IcarusToolRegistry,
        persistent::{IcarusPersistentState, TypedPersistentState},
        prompts::{Prompt, PromptRegistry, PromptBuilder},
    };
}