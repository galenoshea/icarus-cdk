//! ICP canister client for the MCP server

use anyhow::{anyhow, Result};
use candid::{Decode, Encode, Principal};
use ic_agent::Agent;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use smallvec::SmallVec;
use tracing::debug;

use crate::mcp::config::McpConfig;
use crate::mcp::networking::pool::AgentPool;

/// Client for communicating with ICP canisters
#[derive(Debug)]
pub struct CanisterClient {
    canister_id: Principal,
    agent: Agent,
    /// Tool schemas using FxHashMap for better performance with string keys
    tool_schemas: FxHashMap<String, JsonValue>,
}

/// Tool metadata from canister
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// The unique identifier for this tool
    pub name: String,
    /// Human-readable description of what the tool does
    pub description: String,
    /// JSON schema defining the input parameters for this tool
    #[serde(rename = "inputSchema")]
    pub input_schema: JsonValue,
    /// Optional display title for the tool (falls back to name if not provided)
    pub title: Option<String>,
    /// Optional icon identifier for UI display
    pub icon: Option<String>,
}

/// Canister metadata response
///
/// Uses SmallVec for tools to optimize for canisters with few tools (common case)
#[derive(Debug, Serialize, Deserialize)]
pub struct CanisterMetadata {
    /// The canister's name or identifier
    pub name: String,
    /// Optional version information for the canister
    pub version: Option<String>,
    /// Most canisters have <8 tools, so use stack allocation
    #[serde(with = "smallvec_serde")]
    pub tools: SmallVec<[ToolMetadata; 8]>,
    /// Optional display title for the canister (falls back to name if not provided)
    pub title: Option<String>,
    /// Optional website URL for additional information about the canister
    pub website_url: Option<String>,
}

mod smallvec_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(vec: &SmallVec<[ToolMetadata; 8]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        vec.as_slice().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SmallVec<[ToolMetadata; 8]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<ToolMetadata> = Vec::deserialize(deserializer)?;
        Ok(SmallVec::from_vec(vec))
    }
}

impl CanisterClient {
    /// Get the canister ID this client is connected to
    #[inline]
    pub fn canister_id(&self) -> Principal {
        self.canister_id
    }

    /// Create a new canister client from configuration
    ///
    /// Uses connection pooling for improved performance when connecting to the same canister
    pub async fn new(config: McpConfig) -> Result<Self> {
        debug!("Creating canister client for {}", config.canister_id);

        // Get agent from pool for better performance
        let pool = AgentPool::global();
        let agent = pool.get_or_create_agent(&config).await?;

        // Extract the agent from Arc for storage
        // Note: We clone the Arc here to share the connection
        let agent = (*agent).clone();

        Ok(Self {
            canister_id: config.canister_id,
            agent,
            tool_schemas: FxHashMap::default(),
        })
    }

    // Identity management is now handled by the AgentPool

    /// Refresh tool definitions from canister
    pub async fn refresh_tools(&mut self) -> Result<()> {
        debug!("Refreshing tool definitions from canister");

        let metadata = self.get_canister_metadata().await?;

        // Cache tool schemas for parameter mapping
        self.tool_schemas.clear();
        for tool in &metadata.tools {
            self.tool_schemas
                .insert(tool.name.clone(), tool.input_schema.clone());
        }

        debug!("Cached {} tool schemas", self.tool_schemas.len());
        Ok(())
    }

    /// Get canister metadata including tools
    pub async fn get_canister_metadata(&self) -> Result<CanisterMetadata> {
        debug!("Getting canister metadata");

        let result: Vec<u8> = self
            .agent
            .query(&self.canister_id, "list_tools")
            .call()
            .await?;

        // Try to decode as Result<String, String> first
        match Decode!(&result, Result<String, String>) {
            Ok(Ok(json_str)) => {
                let metadata: CanisterMetadata = serde_json::from_str(&json_str)?;
                Ok(metadata)
            }
            Ok(Err(error)) => {
                anyhow::bail!("Canister error: {}", error);
            }
            Err(_) => {
                // Fallback: try to decode as plain string
                match Decode!(&result, String) {
                    Ok(json_str) => {
                        let metadata: CanisterMetadata = serde_json::from_str(&json_str)?;
                        Ok(metadata)
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to decode metadata: {}", e);
                    }
                }
            }
        }
    }

    /// Call a canister method with JSON parameters
    pub async fn call_method(
        &self,
        method_name: &str,
        args: JsonValue,
        is_query: bool,
    ) -> Result<JsonValue> {
        debug!(
            "Calling canister method: {} (query: {})",
            method_name, is_query
        );

        // Convert JSON args to Candid
        let candid_args = self.json_to_candid(method_name, args)?;

        let result_bytes = if is_query {
            self.agent
                .query(&self.canister_id, method_name)
                .with_arg(candid_args)
                .call()
                .await?
        } else {
            self.agent
                .update(&self.canister_id, method_name)
                .with_arg(candid_args)
                .call_and_wait()
                .await?
        };

        // Decode result - handle both Result<T, String> and T patterns
        match Decode!(&result_bytes, Result<String, String>) {
            Ok(Ok(result_json)) => {
                let result: JsonValue = serde_json::from_str(&result_json)?;
                Ok(result)
            }
            Ok(Err(error)) => {
                anyhow::bail!("Canister method '{}' failed: {}", method_name, error);
            }
            Err(_) => {
                // Try to decode as direct value
                match Decode!(&result_bytes, String) {
                    Ok(result_json) => {
                        let result: JsonValue = serde_json::from_str(&result_json)?;
                        Ok(result)
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to decode result from '{}': {}", method_name, e);
                    }
                }
            }
        }
    }

    /// Convert JSON parameters to Candid encoding using tool schema
    fn json_to_candid(&self, method_name: &str, args: JsonValue) -> Result<Vec<u8>> {
        // Use tool schema for intelligent mapping if available
        if let Some(schema) = self.tool_schemas.get(method_name) {
            self.json_to_candid_with_schema(&args, schema)
        } else {
            // Fallback to best-effort mapping
            self.json_to_candid_best_effort(args)
        }
    }

    /// Convert JSON to Candid using tool schema for precise type mapping
    fn json_to_candid_with_schema(&self, args: &JsonValue, schema: &JsonValue) -> Result<Vec<u8>> {
        // Extract schema properties
        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            match args {
                JsonValue::Object(obj) => {
                    // Convert based on schema type information
                    let candid_values = self.convert_object_with_schema(obj, properties)?;
                    Ok(candid_values)
                }
                _ => {
                    // Single argument - use first property schema
                    if let Some((_, prop_schema)) = properties.iter().next() {
                        self.convert_value_with_schema(args, prop_schema)
                    } else {
                        self.json_to_candid_best_effort(args.clone())
                    }
                }
            }
        } else {
            // Schema doesn't have properties, use best-effort
            self.json_to_candid_best_effort(args.clone())
        }
    }

    /// Convert object properties based on schema types
    fn convert_object_with_schema(
        &self,
        obj: &serde_json::Map<String, JsonValue>,
        _schema_props: &serde_json::Map<String, JsonValue>,
    ) -> Result<Vec<u8>> {
        // For simplicity, encode as a struct-like format
        // In a full implementation, we'd need to handle more complex Candid types
        let args_str = serde_json::to_string(obj)?;
        Ok(Encode!(&args_str)?)
    }

    /// Convert single value based on schema type
    fn convert_value_with_schema(&self, value: &JsonValue, schema: &JsonValue) -> Result<Vec<u8>> {
        let type_name = schema
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("string");

        match (value, type_name) {
            (JsonValue::String(s), "string") => Ok(Encode!(s)?),
            (JsonValue::Number(n), "integer") => {
                if let Some(i) = n.as_i64() {
                    Ok(Encode!(&i)?)
                } else {
                    anyhow::bail!("Expected integer, got float")
                }
            }
            (JsonValue::Number(n), "number") => {
                if let Some(f) = n.as_f64() {
                    Ok(Encode!(&f)?)
                } else {
                    anyhow::bail!("Invalid number format")
                }
            }
            (JsonValue::Bool(b), "boolean") => Ok(Encode!(b)?),
            (JsonValue::Array(arr), "array") => {
                // Encode array as JSON string for now
                let arr_str = serde_json::to_string(arr)?;
                Ok(Encode!(&arr_str)?)
            }
            _ => {
                // Type mismatch or unknown type, fallback to string representation
                let value_str = serde_json::to_string(value)?;
                Ok(Encode!(&value_str)?)
            }
        }
    }

    /// Best-effort JSON to Candid conversion without schema
    fn json_to_candid_best_effort(&self, args: JsonValue) -> Result<Vec<u8>> {
        match args {
            JsonValue::Object(obj) => {
                // Convert object to key-value pairs and encode
                let args_str = serde_json::to_string(&obj)?;
                Ok(Encode!(&args_str)?)
            }
            JsonValue::String(s) => Ok(Encode!(&s)?),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Encode!(&i)?)
                } else if let Some(f) = n.as_f64() {
                    Ok(Encode!(&f)?)
                } else {
                    anyhow::bail!("Invalid number format");
                }
            }
            JsonValue::Bool(b) => Ok(Encode!(&b)?),
            JsonValue::Array(_) => {
                let args_str = serde_json::to_string(&args)?;
                Ok(Encode!(&args_str)?)
            }
            JsonValue::Null => Ok(Encode!(&())?),
        }
    }

    /// Check if current principal is authorized
    pub async fn check_authorization(&self) -> Result<bool> {
        let principal = self
            .agent
            .get_principal()
            .map_err(|e| anyhow!("Failed to get principal: {}", e))?;

        let result: Vec<u8> = self
            .agent
            .query(&self.canister_id, "is_authorized")
            .with_arg(Encode!(&principal)?)
            .call()
            .await?;

        let authorized = Decode!(&result, bool)?;
        Ok(authorized)
    }
}
