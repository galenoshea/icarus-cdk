//! Bridge service implementation for protocol translation
//!
//! Handles all MCP protocol complexity while canisters remain clean

// pub mod translator;  // Not used with RMCP
// pub mod server;      // Not used with RMCP
pub mod canister_client;
// pub mod mcp_stdio;   // Not used with RMCP
pub mod param_mapper;
pub mod rmcp_server;
