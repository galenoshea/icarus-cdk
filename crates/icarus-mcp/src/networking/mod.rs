//! Networking module for ICP canister communication
//!
//! This module provides networking capabilities including:
//! - ICP canister client with connection pooling
//! - Agent pool management for efficient connections
//! - HTTP transport abstraction

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "networking")]
pub mod pool;

#[cfg(feature = "client")]
pub use client::{CanisterClient, CanisterMetadata, ToolMetadata};
#[cfg(feature = "networking")]
pub use pool::AgentPool;
