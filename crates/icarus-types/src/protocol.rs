//! Protocol types for bridge communication

use serde::{Deserialize, Serialize};

/// Request from Claude Desktop to the bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcarusBridgeRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: Option<serde_json::Value>,
}

/// Response from the bridge to Claude Desktop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcarusBridgeResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<BridgeError>,
    pub id: Option<serde_json::Value>,
}

/// Error structure for bridge responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Bridge session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeSession {
    pub canister_id: String,
    pub created_at: u64,
    pub last_active: u64,
}