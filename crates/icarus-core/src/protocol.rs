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
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, PartialEq)]
pub enum SubnetType {
    Application,
    System,
    Fiduciary,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_metadata_creation() {
        let params = vec![ParameterMetadata {
            name: "input".to_string(),
            candid_type: "text".to_string(),
            required: true,
            description: "Input text".to_string(),
        }];

        let tool = ToolMetadata {
            name: "process".to_string(),
            candid_method: "process_text".to_string(),
            is_query: false,
            description: "Process text input".to_string(),
            parameters: params,
        };

        assert_eq!(tool.name, "process");
        assert_eq!(tool.candid_method, "process_text");
        assert!(!tool.is_query);
        assert_eq!(tool.parameters.len(), 1);
        assert_eq!(tool.parameters[0].name, "input");
    }

    #[test]
    fn test_icarus_metadata_serialization() {
        let metadata = IcarusMetadata {
            version: "0.2.6".to_string(),
            canister_id: Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap(),
            tools: vec![ToolMetadata {
                name: "tool1".to_string(),
                candid_method: "method1".to_string(),
                is_query: true,
                description: "First tool".to_string(),
                parameters: vec![],
            }],
        };

        // Test serialization round-trip
        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: IcarusMetadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.version, metadata.version);
        assert_eq!(deserialized.canister_id, metadata.canister_id);
        assert_eq!(deserialized.tools.len(), metadata.tools.len());
        assert_eq!(deserialized.tools[0].name, "tool1");
    }

    #[test]
    fn test_parameter_metadata() {
        let param = ParameterMetadata {
            name: "count".to_string(),
            candid_type: "nat32".to_string(),
            required: false,
            description: "Number of items".to_string(),
        };

        assert_eq!(param.name, "count");
        assert_eq!(param.candid_type, "nat32");
        assert!(!param.required);
        assert_eq!(param.description, "Number of items");
    }

    #[test]
    fn test_canister_config() {
        let config = CanisterConfig {
            name: "test_canister".to_string(),
            version: "1.0.0".to_string(),
            canister_id: Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(),
        };

        assert_eq!(config.name, "test_canister");
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.canister_id.to_text(), "ryjl3-tyaaa-aaaaa-aaaba-cai");
    }

    #[test]
    fn test_subnet_type() {
        let app = SubnetType::Application;
        let sys = SubnetType::System;
        let fid = SubnetType::Fiduciary;

        // Test serialization
        assert_eq!(serde_json::to_string(&app).unwrap(), "\"Application\"");
        assert_eq!(serde_json::to_string(&sys).unwrap(), "\"System\"");
        assert_eq!(serde_json::to_string(&fid).unwrap(), "\"Fiduciary\"");

        // Test deserialization
        let deserialized: SubnetType = serde_json::from_str("\"Application\"").unwrap();
        assert_eq!(deserialized, SubnetType::Application);
    }

    #[test]
    fn test_metadata_with_multiple_tools() {
        let tools = vec![
            ToolMetadata {
                name: "read".to_string(),
                candid_method: "read_data".to_string(),
                is_query: true,
                description: "Read data".to_string(),
                parameters: vec![],
            },
            ToolMetadata {
                name: "write".to_string(),
                candid_method: "write_data".to_string(),
                is_query: false,
                description: "Write data".to_string(),
                parameters: vec![ParameterMetadata {
                    name: "data".to_string(),
                    candid_type: "text".to_string(),
                    required: true,
                    description: "Data to write".to_string(),
                }],
            },
        ];

        let metadata = IcarusMetadata {
            version: "0.2.6".to_string(),
            canister_id: Principal::anonymous(),
            tools,
        };

        assert_eq!(metadata.tools.len(), 2);
        assert!(metadata.tools[0].is_query);
        assert!(!metadata.tools[1].is_query);
        assert_eq!(metadata.tools[1].parameters.len(), 1);
    }
}
