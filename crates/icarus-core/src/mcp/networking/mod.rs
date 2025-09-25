//! Networking module for ICP canister communication
//!
//! This module provides networking capabilities including:
//! - ICP canister client with connection pooling
//! - Agent pool management for efficient connections
//! - HTTP transport abstraction

#[cfg(all(feature = "mcp", feature = "client"))]
pub mod client;
#[cfg(all(feature = "mcp", feature = "networking"))]
pub mod pool;

#[cfg(all(feature = "mcp", feature = "client"))]
pub use client::{CanisterClient, CanisterMetadata, ToolMetadata};
#[cfg(all(feature = "mcp", feature = "networking"))]
pub use pool::AgentPool;
