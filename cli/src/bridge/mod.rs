//! Bridge service implementation for protocol translation
//!
//! Handles all MCP protocol complexity while canisters remain clean

// pub mod translator;  // Not used with RMCP
// pub mod server;      // Not used with RMCP
pub mod auth;
pub mod canister_client;
// pub mod mcp_stdio;   // Not used with RMCP
pub mod oauth2;
pub mod rmcp_server;
