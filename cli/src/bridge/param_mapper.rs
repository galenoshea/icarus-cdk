//! Parameter mapping between MCP JSON and ICP Candid
//!
//! This module handles the intelligent translation of MCP protocol JSON arguments
//! to ICP Candid-encoded parameters, supporting various parameter patterns.

use anyhow::{anyhow, Result};
use candid::Encode;
use serde_json::Value;
use std::collections::HashMap;

/// How parameters are passed to the canister function
#[derive(Debug, Clone, PartialEq)]
pub enum ParamStyle {
    /// Multiple ordered parameters (e.g., fn(key: String, content: String))
    Positional {
        order: Vec<String>,
        types: Vec<String>,
    },
    /// Single struct/record parameter (e.g., fn(args: MyStruct))
    Record,
    /// No parameters (e.g., fn())
    Empty,
}

/// Tool definition with parameter information
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub input_schema: Value,
    pub param_style: ParamStyle,
}

/// Maps MCP JSON arguments to Candid encoding based on tool definitions
pub struct ParamMapper {
    tools: HashMap<String, ToolDefinition>,
}

impl ParamMapper {
    /// Create a new ParamMapper from the canister's list_tools response
    pub fn from_tools_list(tools_json: &str) -> Result<Self> {
        let data: Value = serde_json::from_str(tools_json)?;
        let mut tools = HashMap::new();

        let tools_array = data["tools"]
            .as_array()
            .ok_or_else(|| anyhow!("Expected 'tools' array in response"))?;

        for tool in tools_array {
            let name = tool["name"]
                .as_str()
                .ok_or_else(|| anyhow!("Tool missing 'name' field"))?
                .to_string();

            let input_schema = tool
                .get("inputSchema")
                .cloned()
                .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

            // Check for explicit Icarus parameter hints
            let param_style = if let Some(icarus_params) = input_schema.get("x-icarus-params") {
                Self::parse_explicit_style(icarus_params)?
            } else {
                // Auto-detect from schema structure
                Self::detect_param_style(&input_schema, &name)
            };

            tools.insert(
                name.clone(),
                ToolDefinition {
                    name: name.clone(),
                    input_schema,
                    param_style,
                },
            );
        }

        Ok(Self { tools })
    }

    /// Parse explicit parameter style from x-icarus-params
    fn parse_explicit_style(icarus_params: &Value) -> Result<ParamStyle> {
        match icarus_params["style"].as_str() {
            Some("positional") => {
                let order: Vec<String> = icarus_params["order"]
                    .as_array()
                    .ok_or_else(|| anyhow!("Missing 'order' for positional params"))?
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();

                let types: Vec<String> = icarus_params["types"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();

                Ok(ParamStyle::Positional { order, types })
            }
            Some("record") => Ok(ParamStyle::Record),
            Some("empty") | None => Ok(ParamStyle::Empty),
            Some(other) => Err(anyhow!("Unknown param style: {}", other)),
        }
    }

    /// Auto-detect parameter style from schema structure
    fn detect_param_style(schema: &Value, _tool_name: &str) -> ParamStyle {
        // Check if schema has properties
        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            if properties.is_empty() {
                return ParamStyle::Empty;
            }

            // Get required fields to determine order
            let required = schema
                .get("required")
                .and_then(|r| r.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            // For now, assume positional if we have multiple simple properties
            // This heuristic can be improved based on actual usage patterns
            if properties.len() <= 5 {
                // Assume positional for small number of params
                let mut order = required.clone();

                // Add optional params after required ones
                for key in properties.keys() {
                    if !order.contains(key) {
                        order.push(key.clone());
                    }
                }

                // Try to infer types from schema
                let types = order
                    .iter()
                    .map(|param_name| {
                        properties
                            .get(param_name)
                            .and_then(|p| p.get("type"))
                            .and_then(|t| t.as_str())
                            .map(|t| match t {
                                "string" => "text",
                                "number" => "nat64",
                                "integer" => "int64",
                                "boolean" => "bool",
                                _ => "text",
                            })
                            .unwrap_or("text")
                            .to_string()
                    })
                    .collect();

                return ParamStyle::Positional { order, types };
            }
        }

        // Default to record style for complex objects
        ParamStyle::Record
    }

    /// Convert MCP JSON arguments to Candid bytes
    pub fn map_to_candid(&self, tool_name: &str, mcp_args: Value) -> Result<Vec<u8>> {
        let tool = self
            .tools
            .get(tool_name)
            .ok_or_else(|| anyhow!("Unknown tool: {}", tool_name))?;

        match &tool.param_style {
            ParamStyle::Positional { order, types } => {
                self.encode_positional(order, types, mcp_args)
            }
            ParamStyle::Record => self.encode_record(mcp_args),
            ParamStyle::Empty => Ok(Encode!(&())?),
        }
    }

    /// Encode positional parameters from JSON object
    fn encode_positional(
        &self,
        order: &[String],
        types: &[String],
        args: Value,
    ) -> Result<Vec<u8>> {
        if !args.is_object() {
            return Err(anyhow!("Expected object for positional parameters"));
        }

        let obj = args.as_object().unwrap();
        let mut encoded = Vec::new();

        for (i, param_name) in order.iter().enumerate() {
            let value = obj.get(param_name);
            let param_type = types.get(i).map(|s| s.as_str()).unwrap_or("text");

            // Encode based on type
            let param_bytes = match param_type {
                "text" => {
                    let s = value.and_then(|v| v.as_str()).unwrap_or("");
                    Encode!(&s)?
                }
                "nat64" | "nat" => {
                    let n = value.and_then(|v| v.as_u64()).unwrap_or(0);
                    Encode!(&n)?
                }
                "int64" | "int" => {
                    let n = value.and_then(|v| v.as_i64()).unwrap_or(0);
                    Encode!(&n)?
                }
                "bool" => {
                    let b = value.and_then(|v| v.as_bool()).unwrap_or(false);
                    Encode!(&b)?
                }
                _ => {
                    // Default to string representation
                    let s = value.map(|v| v.to_string()).unwrap_or_default();
                    Encode!(&s)?
                }
            };

            // For multiple parameters, we need to encode them as a tuple
            // This is a simplified approach - may need refinement
            if i == 0 {
                encoded = param_bytes;
            } else {
                // This is tricky - Candid tuple encoding needs special handling
                // For now, we'll append, but this needs proper tuple encoding
                encoded.extend(param_bytes);
            }
        }

        Ok(encoded)
    }

    /// Encode as a single record/struct parameter
    fn encode_record(&self, args: Value) -> Result<Vec<u8>> {
        // For record types, we encode the entire JSON object
        // This assumes the canister expects a single parameter that is a record
        let json_str = serde_json::to_string(&args)?;
        Ok(Encode!(&json_str)?)
    }

    /// Try to encode with fallback strategies
    pub fn encode_with_fallback(&self, tool_name: &str, args: Value) -> Result<Vec<u8>> {
        // Try primary strategy based on tool definition
        if let Ok(encoded) = self.map_to_candid(tool_name, args.clone()) {
            return Ok(encoded);
        }

        // Fallback 1: Try as positional if object with properties
        if args.is_object() {
            let obj = args.as_object().unwrap();
            if !obj.is_empty() && obj.len() <= 5 {
                // Try to extract values in alphabetical order as a guess
                let mut keys: Vec<_> = obj.keys().cloned().collect();
                keys.sort();

                let types = vec!["text".to_string(); keys.len()];
                if let Ok(encoded) = self.encode_positional(&keys, &types, args.clone()) {
                    return Ok(encoded);
                }
            }
        }

        // Fallback 2: Try as single string if simple value
        if let Some(s) = args.as_str() {
            return Ok(Encode!(&s)?);
        }

        // Fallback 3: Try as JSON string
        let json_str = serde_json::to_string(&args)?;
        Ok(Encode!(&json_str)?)
    }

    /// Get tool definition by name
    pub fn get_tool(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    /// Check if a tool exists
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tools_list() {
        let tools_json = r#"{
            "name": "test-canister",
            "version": "0.1.0",
            "tools": [
                {
                    "name": "memorize",
                    "description": "Store a memory",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "key": { "type": "string" },
                            "content": { "type": "string" }
                        },
                        "required": ["key", "content"]
                    }
                }
            ]
        }"#;

        let mapper = ParamMapper::from_tools_list(tools_json).unwrap();
        assert!(mapper.has_tool("memorize"));
    }

    #[test]
    fn test_positional_encoding() {
        let tools_json = r#"{
            "tools": [{
                "name": "test",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "a": { "type": "string" },
                        "b": { "type": "string" }
                    },
                    "required": ["a", "b"],
                    "x-icarus-params": {
                        "style": "positional",
                        "order": ["a", "b"],
                        "types": ["text", "text"]
                    }
                }
            }]
        }"#;

        let mapper = ParamMapper::from_tools_list(tools_json).unwrap();
        let args = serde_json::json!({
            "a": "value1",
            "b": "value2"
        });

        let encoded = mapper.map_to_candid("test", args).unwrap();
        assert!(!encoded.is_empty());
    }
}
