//! Utilities for Claude Desktop configuration management

use serde_json::{json, Value};

/// Generate Claude Desktop MCP server configuration
pub fn generate_claude_server_config(name: &str, canister_id: &str) -> Value {
    json!({
        name: {
            "command": "icarus",
            "args": ["bridge", "start", "--canister-id", canister_id],
            "env": {}
        }
    })
}
