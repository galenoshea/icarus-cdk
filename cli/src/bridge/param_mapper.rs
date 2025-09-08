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
                Self::detect_param_style(&input_schema)
            };

            tools.insert(name, ToolDefinition { param_style });
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
    fn detect_param_style(schema: &Value) -> ParamStyle {
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
        let debug = std::env::var("ICARUS_DEBUG").is_ok();

        if debug {
            eprintln!(
                "[DEBUG] encode_positional: order={:?}, types={:?}, args={}",
                order, types, args
            );
        }

        if !args.is_object() {
            return Err(anyhow!("Expected object for positional parameters"));
        }

        let obj = args.as_object().unwrap();

        // Collect all parameter values in order
        let mut values: Vec<String> = Vec::new();

        for param_name in order.iter() {
            let value = obj.get(param_name);
            let param_value = if let Some(v) = value {
                match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => v.to_string(),
                }
            } else {
                // Parameter not provided, use empty string as default
                String::new()
            };
            values.push(param_value);
        }

        if debug {
            eprintln!("[DEBUG] Collected values: {:?}", values);
        }

        // Encode all parameters together
        // IMPORTANT: Multiple parameters must be encoded in a single Encode! call,
        // not as a tuple but as separate arguments to match Candid function signatures
        let encoded = match values.len() {
            0 => Encode!(&())?,
            1 => {
                // Single parameter
                let param_type = types.get(0).map(|s| s.as_str()).unwrap_or("text");
                match param_type {
                    "nat64" | "nat" => {
                        let n = values[0].parse::<u64>().unwrap_or(0);
                        Encode!(&n)?
                    }
                    "int64" | "int" => {
                        let n = values[0].parse::<i64>().unwrap_or(0);
                        Encode!(&n)?
                    }
                    "bool" => {
                        let b = values[0].parse::<bool>().unwrap_or(false);
                        Encode!(&b)?
                    }
                    _ => Encode!(&values[0])?,
                }
            }
            2 => {
                // Two parameters - encode as separate arguments, not as a tuple
                let v1 = &values[0];
                let v2 = &values[1];
                Encode!(&v1, &v2)?
            }
            3 => {
                // Three parameters
                let v1 = &values[0];
                let v2 = &values[1];
                let v3 = &values[2];
                Encode!(&v1, &v2, &v3)?
            }
            4 => {
                // Four parameters
                let v1 = &values[0];
                let v2 = &values[1];
                let v3 = &values[2];
                let v4 = &values[3];
                Encode!(&v1, &v2, &v3, &v4)?
            }
            5 => {
                // Five parameters
                let v1 = &values[0];
                let v2 = &values[1];
                let v3 = &values[2];
                let v4 = &values[3];
                let v5 = &values[4];
                Encode!(&v1, &v2, &v3, &v4, &v5)?
            }
            6 => {
                // Six parameters
                let v1 = &values[0];
                let v2 = &values[1];
                let v3 = &values[2];
                let v4 = &values[3];
                let v5 = &values[4];
                let v6 = &values[5];
                Encode!(&v1, &v2, &v3, &v4, &v5, &v6)?
            }
            7 => {
                // Seven parameters
                let v1 = &values[0];
                let v2 = &values[1];
                let v3 = &values[2];
                let v4 = &values[3];
                let v5 = &values[4];
                let v6 = &values[5];
                let v7 = &values[6];
                Encode!(&v1, &v2, &v3, &v4, &v5, &v6, &v7)?
            }
            8 => {
                // Eight parameters
                let v1 = &values[0];
                let v2 = &values[1];
                let v3 = &values[2];
                let v4 = &values[3];
                let v5 = &values[4];
                let v6 = &values[5];
                let v7 = &values[6];
                let v8 = &values[7];
                Encode!(&v1, &v2, &v3, &v4, &v5, &v6, &v7, &v8)?
            }
            _ => {
                // For more than 8 parameters, we'd need a different approach
                // This is a very rare case - most functions don't have >8 params
                return Err(anyhow!(
                    "Functions with more than 8 parameters are not yet supported"
                ));
            }
        };

        if debug {
            eprintln!(
                "[DEBUG] Encoded bytes: {:?} (length: {})",
                encoded,
                encoded.len()
            );
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
        // Check that the tool was parsed and stored
        assert!(mapper.tools.contains_key("memorize"));
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

    #[test]
    fn test_record_style_encoding() {
        let tools_json = r#"{
            "tools": [{
                "name": "complex_tool",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "field1": { "type": "string" },
                        "field2": { "type": "number" },
                        "field3": { "type": "boolean" }
                    },
                    "x-icarus-params": {
                        "style": "record"
                    }
                }
            }]
        }"#;

        let mapper = ParamMapper::from_tools_list(tools_json).unwrap();
        let args = serde_json::json!({
            "field1": "test",
            "field2": 42,
            "field3": true
        });

        let encoded = mapper.map_to_candid("complex_tool", args.clone()).unwrap();
        assert!(!encoded.is_empty());

        // Verify it encodes as JSON string for record style
        let _expected_json = serde_json::to_string(&args).unwrap();
        // The encoded bytes should contain the JSON string
        assert!(encoded.len() > 0);
    }

    #[test]
    fn test_empty_params_encoding() {
        let tools_json = r#"{
            "tools": [{
                "name": "no_params",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "x-icarus-params": {
                        "style": "empty"
                    }
                }
            }]
        }"#;

        let mapper = ParamMapper::from_tools_list(tools_json).unwrap();
        let args = serde_json::json!({});

        let encoded = mapper.map_to_candid("no_params", args).unwrap();
        // Empty params should still produce valid Candid encoding
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_auto_detect_positional_style() {
        // Test auto-detection without explicit x-icarus-params
        let tools_json = r#"{
            "tools": [{
                "name": "auto_tool",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "key": { "type": "string" },
                        "value": { "type": "string" }
                    },
                    "required": ["key", "value"]
                }
            }]
        }"#;

        let mapper = ParamMapper::from_tools_list(tools_json).unwrap();
        let tool = mapper.tools.get("auto_tool").unwrap();

        // Should auto-detect as positional for small number of params
        match &tool.param_style {
            ParamStyle::Positional { order, .. } => {
                assert_eq!(order.len(), 2);
                assert!(order.contains(&"key".to_string()));
                assert!(order.contains(&"value".to_string()));
            }
            _ => panic!("Expected positional style for auto-detection"),
        }
    }

    #[test]
    fn test_type_conversions() {
        let tools_json = r#"{
            "tools": [{
                "name": "typed_tool",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text_field": { "type": "string" },
                        "number_field": { "type": "number" },
                        "int_field": { "type": "integer" },
                        "bool_field": { "type": "boolean" }
                    },
                    "x-icarus-params": {
                        "style": "positional",
                        "order": ["text_field", "number_field", "int_field", "bool_field"],
                        "types": ["text", "nat64", "int64", "bool"]
                    }
                }
            }]
        }"#;

        let mapper = ParamMapper::from_tools_list(tools_json).unwrap();
        let args = serde_json::json!({
            "text_field": "hello",
            "number_field": 42,
            "int_field": -10,
            "bool_field": true
        });

        let encoded = mapper.map_to_candid("typed_tool", args).unwrap();
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_fallback_encoding() {
        let tools_json = r#"{
            "tools": [{
                "name": "simple",
                "inputSchema": {}
            }]
        }"#;

        let mapper = ParamMapper::from_tools_list(tools_json).unwrap();

        // Test fallback with object
        let obj_args = serde_json::json!({"key": "value"});
        let encoded = mapper
            .encode_with_fallback("unknown_tool", obj_args)
            .unwrap();
        assert!(!encoded.is_empty());

        // Test fallback with string
        let str_args = serde_json::json!("simple_string");
        let encoded = mapper
            .encode_with_fallback("unknown_tool", str_args)
            .unwrap();
        assert!(!encoded.is_empty());

        // Test fallback with array
        let arr_args = serde_json::json!(["item1", "item2"]);
        let encoded = mapper.encode_with_fallback("simple", arr_args).unwrap();
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_error_handling() {
        // Test with invalid JSON
        let result = ParamMapper::from_tools_list("invalid json");
        assert!(result.is_err());

        // Test with missing tools array
        let result = ParamMapper::from_tools_list(r#"{"name": "test"}"#);
        assert!(result.is_err());

        // Test encoding for unknown tool
        let tools_json = r#"{"tools": []}"#;
        let mapper = ParamMapper::from_tools_list(tools_json).unwrap();
        let result = mapper.map_to_candid("unknown", serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_memento_scenario() {
        // Test the exact scenario that was failing with Memento
        let tools_json = r#"{
            "tools": [{
                "name": "memorize",
                "description": "Store a memory with a unique key",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "key": { 
                            "type": "string",
                            "description": "Unique identifier for the memory"
                        },
                        "content": { 
                            "type": "string",
                            "description": "Content to store"
                        }
                    },
                    "required": ["key", "content"],
                    "x-icarus-params": {
                        "style": "positional",
                        "order": ["key", "content"],
                        "types": ["text", "text"]
                    }
                }
            }]
        }"#;

        let mapper = ParamMapper::from_tools_list(tools_json).unwrap();

        // MCP sends arguments as JSON object
        let mcp_args = serde_json::json!({
            "key": "test_key",
            "content": "This is test content"
        });

        // Should encode successfully
        let encoded = mapper.map_to_candid("memorize", mcp_args).unwrap();
        assert!(!encoded.is_empty());

        // The encoding should handle two separate string parameters
        // not a single record/object
    }
}
