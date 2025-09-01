//! Metadata types for tool discovery and canister introspection

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

/// Metadata about the Icarus canister for tool discovery
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct IcarusMetadata {
    /// Version of the Icarus protocol
    pub version: String,
    /// Canister ID where this server is deployed
    pub canister_id: Principal,
    /// List of available tools in this canister
    pub tools: Vec<ToolMetadata>,
}

/// Metadata about a single tool
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ToolMetadata {
    /// Name of the tool
    pub name: String,
    /// Candid method name to call on the canister
    pub candid_method: String,
    /// Whether this is a query call (read-only)
    pub is_query: bool,
    /// Human-readable description of what the tool does
    pub description: String,
    /// Parameters accepted by this tool
    pub parameters: Vec<ParameterMetadata>,
}

/// Metadata about a tool parameter
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ParameterMetadata {
    /// Parameter name
    pub name: String,
    /// Candid type of the parameter
    pub candid_type: String,
    /// Whether this parameter is required
    pub required: bool,
    /// Description of the parameter
    pub description: String,
}

/// Canister configuration
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct CanisterConfig {
    pub name: String,
    pub version: String,
    pub canister_id: Principal,
}

/// Types of ICP subnets (kept for canister info)
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum SubnetType {
    Application,
    System,
    Fiduciary,
}
