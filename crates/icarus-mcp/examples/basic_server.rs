//! Basic MCP Server Example
//!
//! This example shows how to create a simple MCP server using the icarus-mcp crate.
//! Run with: cargo run --example basic_server --features=all

use anyhow::Result;
use candid::Principal;
use icarus_mcp::{McpConfig, McpServer};
use std::str::FromStr;
use tokio::io::{stdin, stdout};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("icarus_mcp=debug,basic_server=debug")
        .init();

    // Parse canister ID from command line args or use a default
    let canister_id = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "rdmx6-jaaaa-aaaaa-aaadq-cai".to_string());

    let principal = Principal::from_str(&canister_id).expect("Invalid canister ID format");

    println!("🚀 Starting MCP server for canister: {}", canister_id);
    println!("💡 Connect your MCP client to this process via stdin/stdout");

    // Create configuration for local IC replica
    let config = McpConfig::local(principal);

    // Create and connect the server
    let server = McpServer::from_config(config)
        .await
        .expect("Failed to create MCP server");

    // Serve over stdio (for MCP client connection)
    let serving_server = server
        .serve(stdin(), stdout())
        .await
        .expect("Failed to start MCP server");

    // Run the server
    serving_server.run().await.expect("MCP server failed");

    Ok(())
}
