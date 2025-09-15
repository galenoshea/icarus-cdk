//! Icarus MCP Server - Standalone MCP server for Internet Computer canisters
//!
//! This crate provides a Model Context Protocol (MCP) server that connects to
//! Internet Computer Protocol (ICP) canisters, enabling AI tools like Claude
//! to interact with blockchain-deployed MCP servers.
//!
//! ## Features
//!
//! This crate supports several optional features:
//!
//! - `client` (default): ICP canister client functionality
//! - `server` (default): MCP server implementation
//! - `streaming` (default): Large response streaming support
//! - `protocol`: MCP protocol handling
//! - `networking`: Connection pooling and HTTP transport
//! - `storage`: Efficient data storage and streaming
//! - `cli`: Command-line interface utilities

#![warn(missing_docs)]

// Feature-gated modules
#[cfg(any(feature = "client", feature = "networking"))]
pub mod networking;

#[cfg(feature = "protocol")]
pub mod protocol;

#[cfg(feature = "server")]
pub mod server;

#[cfg(any(feature = "streaming", feature = "storage"))]
pub mod storage;

pub mod config;

// Feature-gated re-exports
#[cfg(feature = "client")]
pub use networking::{CanisterClient, CanisterMetadata, ToolMetadata};

#[cfg(feature = "networking")]
pub use networking::AgentPool;

pub use config::{ConfigError, McpConfig, McpConfigBuilder};

#[cfg(feature = "protocol")]
pub use protocol::{CanisterBackend, McpProtocol, McpProtocolHandler, ToolConverter};

#[cfg(feature = "server")]
pub use server::{Connected, McpServer, McpServerBuilder, Serving, Uninitialized};

#[cfg(feature = "streaming")]
pub use storage::{
    collect_stream, write_stream_to, BufferSize, CustomSize, DefaultBuffer, Large, ResponseStream,
    Small, StreamingResponse, DEFAULT_CHUNK_SIZE, LARGE_BUFFER_SIZE, SMALL_BUFFER_SIZE,
};

#[cfg(any(feature = "simd", feature = "streaming"))]
pub use storage::SimdProcessor;

/// Result type for MCP operations
pub type Result<T> = anyhow::Result<T>;
