//! # Icarus Bridge
//!
//! MCP-to-ICP bridge for the Icarus SDK that enables Model Context Protocol servers
//! to run on Internet Computer canisters.
//!
//! ## Features
//!
//! - **RMCP Protocol Support**: Full Model Context Protocol implementation using RMCP
//! - **Dynamic Authentication**: Automatic dfx identity detection and switching
//! - **Canister Communication**: Optimized communication with ICP canisters
//! - **Response Streaming**: Support for basic and progress streaming
//! - **Tool Discovery**: Automatic discovery of canister tools and metadata
//!
//! ## Quick Start
//!
//! ```no_run
//! use icarus_bridge::start_bridge;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Start bridge with a canister ID
//!     start_bridge("rdmx6-jaaaa-aaaah-qcaiq-cai").await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! ```no_run
//! use icarus_bridge::BridgeBuilder;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let bridge = BridgeBuilder::new()
//!         .canister_id("rdmx6-jaaaa-aaaah-qcaiq-cai")
//!         .with_authentication(true)
//!         .with_local_network(true)
//!         .build()
//!         .await?;
//!
//!     bridge.start().await?;
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod builder;
pub mod canister_client;
pub mod param_mapper;
pub mod rmcp_server;

// Re-export main types for convenience
pub use builder::BridgeBuilder;
pub use canister_client::CanisterClient;
pub use rmcp_server::{run_with_auth, CanisterMetadata, CanisterTool};

use anyhow::Result;

/// Start a bridge with the given canister ID using default settings.
///
/// This is a convenience function that creates a bridge with authentication
/// enabled and connects to the local IC network.
///
/// # Arguments
///
/// * `canister_id` - The canister ID to connect to
///
/// # Example
///
/// ```no_run
/// use icarus_bridge::start_bridge;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     start_bridge("rdmx6-jaaaa-aaaah-qcaiq-cai").await?;
///     Ok(())
/// }
/// ```
pub async fn start_bridge(canister_id: &str) -> Result<()> {
    run_with_auth(canister_id.to_string(), true, true).await
}
