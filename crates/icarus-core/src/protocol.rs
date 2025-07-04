//! Metadata types for tool discovery and canister introspection

use serde::{Deserialize, Serialize};
use candid::{CandidType, Principal};

/// Metadata about the Icarus canister for tool discovery
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct IcarusMetadata {
    pub version: String,
    pub canister_id: Principal,
    pub tools: Vec<ToolMetadata>,
}

/// Metadata about a single tool
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ToolMetadata {
    pub name: String,
    pub candid_method: String,
    pub is_query: bool,
    pub description: String,
    pub parameters: Vec<ParameterMetadata>,
}

/// Metadata about a tool parameter
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ParameterMetadata {
    pub name: String,
    pub candid_type: String,
    pub required: bool,
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