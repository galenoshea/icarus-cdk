//! Protocol types for MCP-ICP communication

use serde::{Deserialize, Serialize};
use candid::{CandidType, Principal};

/// ICP-specific protocol extensions
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct IcarusProtocol {
    pub canister_id: Principal,
    pub subnet_type: SubnetType,
    pub cycles_balance: u128,
}

/// Types of ICP subnets
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum SubnetType {
    Application,
    System,
    Fiduciary,
}

/// Request wrapper for MCP calls to canisters
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct IcarusMcpRequest {
    pub method: String,
    pub params: String, // JSON string instead of Value for Candid compatibility
    pub id: Option<String>,
}

/// Response wrapper for MCP calls from canisters
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct IcarusMcpResponse {
    pub result: Option<String>, // JSON string instead of Value
    pub error: Option<IcarusMcpError>,
    pub id: Option<String>,
}

/// Error structure for MCP responses
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct IcarusMcpError {
    pub code: i32,
    pub message: String,
    pub data: Option<String>, // JSON string instead of Value
}

/// Server capabilities with ICP extensions
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct IcarusServerCapabilities {
    pub tools: Vec<String>,
    pub resources: Vec<String>,
    pub icarus_version: String,
    pub canister_id: Principal,
}