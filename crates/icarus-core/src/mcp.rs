//! Icarus MCP Server - Standalone MCP server for Internet Computer canisters
//!
//! This module provides a Model Context Protocol (MCP) server that connects to
//! Internet Computer Protocol (ICP) canisters, enabling AI tools like Claude
//! to interact with blockchain-deployed MCP servers.
//!
//! ## Features
//!
//! This module supports several optional features:
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
#[cfg(all(feature = "mcp", any(feature = "client", feature = "networking")))]
pub mod networking;

#[cfg(all(feature = "mcp", feature = "protocol"))]
pub mod protocol;

#[cfg(all(feature = "mcp", feature = "server"))]
pub mod server;

#[cfg(all(feature = "mcp", any(feature = "streaming", feature = "storage")))]
pub mod storage;

#[cfg(feature = "mcp")]
pub mod config;

// Feature-gated re-exports
#[cfg(all(feature = "mcp", feature = "client"))]
pub use networking::{CanisterClient, CanisterMetadata, ToolMetadata};

#[cfg(all(feature = "mcp", feature = "networking"))]
pub use networking::AgentPool;

#[cfg(feature = "mcp")]
pub use config::{ConfigError, McpConfig, McpConfigBuilder};

#[cfg(all(feature = "mcp", feature = "protocol"))]
pub use protocol::{CanisterBackend, McpProtocol, McpProtocolHandler, ToolConverter};

#[cfg(all(feature = "mcp", feature = "server"))]
pub use server::{Connected, McpServer, McpServerBuilder, Serving, Uninitialized};

#[cfg(all(feature = "mcp", feature = "streaming"))]
pub use storage::{
    collect_stream, write_stream_to, BufferSize, CustomSize, DefaultBuffer, Large, ResponseStream,
    Small, StreamingResponse, DEFAULT_CHUNK_SIZE, LARGE_BUFFER_SIZE, SMALL_BUFFER_SIZE,
};

#[cfg(all(feature = "mcp", any(feature = "simd", feature = "streaming")))]
pub use storage::SimdProcessor;

/// Result type for MCP operations
#[cfg(feature = "mcp")]
pub type Result<T> = anyhow::Result<T>;
