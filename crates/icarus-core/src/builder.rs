//! Builder pattern alternatives to macros for Icarus SDK
//!
//! This module provides programmatic alternatives to the macro-based approach
//! for cases where users prefer explicit builder patterns or need runtime configuration.

use crate::error::{IcarusError, Result};
use crate::tool::{IcarusTool, ToolInfo};
use async_trait::async_trait;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Builder for creating MCP tools programmatically
///
/// Alternative to the `#[icarus_tool]` macro for runtime or dynamic tool creation
///
/// # Examples
///
/// ```ignore
/// use icarus_core::builder::ToolBuilder;
///
/// let tool = ToolBuilder::new("my_tool")
///     .description("A custom tool")
///     .parameter("input", "string", "Input parameter", true)
///     .parameter("count", "integer", "Number of iterations", false)
///     .handler(|args| async move {
///         let input = args["input"].as_str().unwrap();
///         Ok(serde_json::json!({ "result": format!("Processed: {}", input) }))
///     })
///     .build();
/// ```
pub struct ToolBuilder {
    name: String,
    description: String,
    parameters: HashMap<String, ParameterSpec>,
    handler: Option<Box<dyn Fn(JsonValue) -> BoxedToolFuture + Send + Sync>>,
}

/// Future type for tool execution
pub type BoxedToolFuture =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<JsonValue>> + Send>>;

/// Parameter specification for tool schema
#[derive(Debug, Clone)]
pub struct ParameterSpec {
    pub param_type: String,
    pub description: String,
    pub required: bool,
    pub default: Option<JsonValue>,
    pub enum_values: Option<Vec<String>>,
}

impl ToolBuilder {
    /// Create a new tool builder with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            parameters: HashMap::new(),
            handler: None,
        }
    }

    /// Set the tool description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a parameter to the tool schema
    pub fn parameter(
        mut self,
        name: impl Into<String>,
        param_type: impl Into<String>,
        description: impl Into<String>,
        required: bool,
    ) -> Self {
        let spec = ParameterSpec {
            param_type: param_type.into(),
            description: description.into(),
            required,
            default: None,
            enum_values: None,
        };
        self.parameters.insert(name.into(), spec);
        self
    }

    /// Add a parameter with default value
    pub fn parameter_with_default(
        mut self,
        name: impl Into<String>,
        param_type: impl Into<String>,
        description: impl Into<String>,
        default: JsonValue,
    ) -> Self {
        let spec = ParameterSpec {
            param_type: param_type.into(),
            description: description.into(),
            required: false,
            default: Some(default),
            enum_values: None,
        };
        self.parameters.insert(name.into(), spec);
        self
    }

    /// Add an enum parameter
    pub fn enum_parameter(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        values: Vec<String>,
        required: bool,
    ) -> Self {
        let spec = ParameterSpec {
            param_type: "string".to_string(),
            description: description.into(),
            required,
            default: None,
            enum_values: Some(values),
        };
        self.parameters.insert(name.into(), spec);
        self
    }

    /// Set the tool execution handler
    pub fn handler<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(JsonValue) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<JsonValue>> + Send + 'static,
    {
        self.handler = Some(Box::new(move |args| Box::pin(handler(args))));
        self
    }

    /// Build the tool
    pub fn build(self) -> Result<BuiltTool> {
        if self.handler.is_none() {
            return Err(IcarusError::Other("Tool handler is required".to_string()));
        }

        Ok(BuiltTool {
            name: self.name,
            description: self.description,
            parameters: self.parameters,
            handler: self.handler.unwrap(),
        })
    }
}

/// A tool created using the builder pattern
pub struct BuiltTool {
    name: String,
    description: String,
    parameters: HashMap<String, ParameterSpec>,
    handler: Box<dyn Fn(JsonValue) -> BoxedToolFuture + Send + Sync>,
}

#[async_trait]
impl IcarusTool for BuiltTool {
    fn info(&self) -> ToolInfo {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for (name, spec) in &self.parameters {
            let mut param_schema = serde_json::Map::new();
            param_schema.insert(
                "type".to_string(),
                JsonValue::String(spec.param_type.clone()),
            );
            param_schema.insert(
                "description".to_string(),
                JsonValue::String(spec.description.clone()),
            );

            if let Some(default) = &spec.default {
                param_schema.insert("default".to_string(), default.clone());
            }

            if let Some(enum_values) = &spec.enum_values {
                param_schema.insert(
                    "enum".to_string(),
                    JsonValue::Array(
                        enum_values
                            .iter()
                            .map(|v| JsonValue::String(v.clone()))
                            .collect(),
                    ),
                );
            }

            properties.insert(name.clone(), JsonValue::Object(param_schema));

            if spec.required {
                required.push(JsonValue::String(name.clone()));
            }
        }

        let schema = serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required
        });

        ToolInfo {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: schema,
            title: None,
            icons: None,
            category: None,
            tags: Vec::new(),
            complexity: None,
            usage_count: 0,
            avg_execution_time: None,
        }
    }

    fn to_rmcp_tool(&self) -> rmcp::model::Tool {
        use std::borrow::Cow;
        use std::sync::Arc;

        let info = self.info();
        rmcp::model::Tool {
            name: Cow::Owned(info.name),
            description: Some(Cow::Owned(info.description)),
            input_schema: Arc::new(
                info.input_schema
                    .as_object()
                    .expect("Generated JSON schema should be an object")
                    .clone(),
            ),
            title: info.title,
            icons: info.icons.map(|icons| {
                icons
                    .into_iter()
                    .map(|icon| rmcp::model::Icon {
                        src: icon.data.unwrap_or_default(),
                        mime_type: None,
                        sizes: None,
                    })
                    .collect()
            }),
            output_schema: None,
            annotations: None,
        }
    }

    async fn execute(&self, args: JsonValue) -> Result<JsonValue> {
        (self.handler)(args).await
    }
}

/// Builder for creating MCP servers programmatically
///
/// Alternative to the `#[icarus_module]` macro for runtime server configuration
///
/// # Examples
///
/// ```ignore
/// use icarus_core::builder::ServerBuilder;
///
/// let server = ServerBuilder::new("My MCP Server")
///     .version("1.0.0")
///     .description("A custom MCP server")
///     .add_tool(tool1)
///     .add_tool(tool2)
///     .build();
/// ```
pub struct ServerBuilder {
    name: String,
    version: String,
    description: String,
    tools: Vec<Box<dyn IcarusTool>>,
    metadata: HashMap<String, JsonValue>,
}

impl ServerBuilder {
    /// Create a new server builder with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "1.0.0".to_string(),
            description: String::new(),
            tools: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the server version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set the server description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a tool to the server
    pub fn add_tool(mut self, tool: impl IcarusTool + 'static) -> Self {
        self.tools.push(Box::new(tool));
        self
    }

    /// Add multiple tools to the server
    pub fn add_tools(mut self, tools: Vec<Box<dyn IcarusTool>>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Add custom metadata
    pub fn metadata(mut self, key: impl Into<String>, value: JsonValue) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Build the server configuration
    pub fn build(self) -> ServerConfig {
        ServerConfig {
            name: self.name,
            version: self.version,
            description: self.description,
            tools: self.tools,
            metadata: self.metadata,
        }
    }
}

/// Built server configuration
pub struct ServerConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub tools: Vec<Box<dyn IcarusTool>>,
    pub metadata: HashMap<String, JsonValue>,
}

impl ServerConfig {
    /// Get server information as JSON
    pub fn info(&self) -> JsonValue {
        let tools: Vec<JsonValue> = self
            .tools
            .iter()
            .map(|tool| {
                let info = tool.info();
                serde_json::json!({
                    "name": info.name,
                    "description": info.description,
                    "inputSchema": info.input_schema
                })
            })
            .collect();

        let mut result = serde_json::json!({
            "name": self.name,
            "version": self.version,
            "description": self.description,
            "tools": tools
        });

        // Add custom metadata
        if let Some(result_obj) = result.as_object_mut() {
            for (key, value) in &self.metadata {
                result_obj.insert(key.clone(), value.clone());
            }
        }

        result
    }

    /// Get all tools
    pub fn tools(&self) -> &[Box<dyn IcarusTool>] {
        &self.tools
    }

    /// Find a tool by name
    pub fn find_tool(&self, name: &str) -> Option<&dyn IcarusTool> {
        self.tools
            .iter()
            .find(|tool| tool.info().name == name)
            .map(|tool| tool.as_ref())
    }
}

/// Storage builder for creating stable storage configurations programmatically
///
/// Alternative to the `stable_storage!` macro for runtime storage setup
///
/// # Examples
///
/// ```ignore
/// use icarus_core::builder::StorageBuilder;
/// use ic_stable_structures::StableBTreeMap;
///
/// let storage = StorageBuilder::new()
///     .add_map::<String, MyData>("users", 0)
///     .add_cell::<u64>("counter", 1, 0)
///     .build();
/// ```
pub struct StorageBuilder {
    maps: Vec<StorageMapConfig>,
    cells: Vec<StorageCellConfig>,
    next_memory_id: u8,
}

/// Configuration for a stable map
#[derive(Debug, Clone, serde::Serialize)]
pub struct StorageMapConfig {
    pub name: String,
    pub memory_id: u8,
    pub key_type: String,
    pub value_type: String,
}

/// Configuration for a stable cell
#[derive(Debug, Clone, serde::Serialize)]
pub struct StorageCellConfig {
    pub name: String,
    pub memory_id: u8,
    pub value_type: String,
    pub default_value: JsonValue,
}

impl StorageBuilder {
    /// Create a new storage builder
    pub fn new() -> Self {
        Self {
            maps: Vec::new(),
            cells: Vec::new(),
            next_memory_id: 0,
        }
    }

    /// Add a stable map with automatic memory ID assignment
    pub fn add_map<K, V>(mut self, name: impl Into<String>) -> Self
    where
        K: 'static,
        V: 'static,
    {
        let config = StorageMapConfig {
            name: name.into(),
            memory_id: self.next_memory_id,
            key_type: std::any::type_name::<K>().to_string(),
            value_type: std::any::type_name::<V>().to_string(),
        };

        self.maps.push(config);
        self.next_memory_id += 1;
        self
    }

    /// Add a stable map with explicit memory ID
    pub fn add_map_with_id<K, V>(mut self, name: impl Into<String>, memory_id: u8) -> Self
    where
        K: 'static,
        V: 'static,
    {
        let config = StorageMapConfig {
            name: name.into(),
            memory_id,
            key_type: std::any::type_name::<K>().to_string(),
            value_type: std::any::type_name::<V>().to_string(),
        };

        self.maps.push(config);
        self.next_memory_id = self.next_memory_id.max(memory_id + 1);
        self
    }

    /// Add a stable cell with automatic memory ID assignment
    pub fn add_cell<T>(mut self, name: impl Into<String>, default_value: JsonValue) -> Self
    where
        T: 'static,
    {
        let config = StorageCellConfig {
            name: name.into(),
            memory_id: self.next_memory_id,
            value_type: std::any::type_name::<T>().to_string(),
            default_value,
        };

        self.cells.push(config);
        self.next_memory_id += 1;
        self
    }

    /// Add a stable cell with explicit memory ID
    pub fn add_cell_with_id<T>(
        mut self,
        name: impl Into<String>,
        memory_id: u8,
        default_value: JsonValue,
    ) -> Self
    where
        T: 'static,
    {
        let config = StorageCellConfig {
            name: name.into(),
            memory_id,
            value_type: std::any::type_name::<T>().to_string(),
            default_value,
        };

        self.cells.push(config);
        self.next_memory_id = self.next_memory_id.max(memory_id + 1);
        self
    }

    /// Build the storage configuration
    pub fn build(self) -> StorageConfig {
        StorageConfig {
            maps: self.maps,
            cells: self.cells,
        }
    }
}

/// Built storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub maps: Vec<StorageMapConfig>,
    pub cells: Vec<StorageCellConfig>,
}

impl StorageConfig {
    /// Generate storage initialization code as a string
    /// This can be used for code generation or documentation
    pub fn generate_code(&self) -> String {
        let mut code = String::new();

        code.push_str("thread_local! {\n");
        code.push_str("    static MEMORY_MANAGER: std::cell::RefCell<\n");
        code.push_str("        ic_stable_structures::memory_manager::MemoryManager<\n");
        code.push_str("            ic_stable_structures::DefaultMemoryImpl\n");
        code.push_str("        >\n");
        code.push_str("    > = std::cell::RefCell::new(\n");
        code.push_str("        ic_stable_structures::memory_manager::MemoryManager::init(\n");
        code.push_str("            ic_stable_structures::DefaultMemoryImpl::default()\n");
        code.push_str("        )\n");
        code.push_str("    );\n\n");

        // Generate map declarations
        for map in &self.maps {
            code.push_str(&format!(
                "    static {}: std::cell::RefCell<ic_stable_structures::StableBTreeMap<{}, {}, _>> = std::cell::RefCell::new(\n",
                map.name.to_uppercase(),
                map.key_type,
                map.value_type
            ));
            code.push_str("        ic_stable_structures::StableBTreeMap::init(\n");
            code.push_str(&format!(
                "            MEMORY_MANAGER.with(|m| m.borrow().get(ic_stable_structures::memory_manager::MemoryId::new({})))\n",
                map.memory_id
            ));
            code.push_str("        )\n");
            code.push_str("    );\n\n");
        }

        // Generate cell declarations
        for cell in &self.cells {
            code.push_str(&format!(
                "    static {}: std::cell::RefCell<ic_stable_structures::StableCell<{}, _>> = std::cell::RefCell::new(\n",
                cell.name.to_uppercase(),
                cell.value_type
            ));
            code.push_str("        ic_stable_structures::StableCell::init(\n");
            code.push_str(&format!(
                "            MEMORY_MANAGER.with(|m| m.borrow().get(ic_stable_structures::memory_manager::MemoryId::new({}))),\n",
                cell.memory_id
            ));
            code.push_str(&format!(
                "            {}\n",
                if cell.default_value.is_null() {
                    "Default::default()".to_string()
                } else {
                    cell.default_value.to_string()
                }
            ));
            code.push_str("        ).expect(\"Failed to initialize StableCell\")\n");
            code.push_str("    );\n\n");
        }

        code.push_str("}\n");

        code
    }

    /// Get configuration summary
    pub fn summary(&self) -> JsonValue {
        serde_json::json!({
            "maps": self.maps.len(),
            "cells": self.cells.len(),
            "total_memory_ids": self.maps.len() + self.cells.len(),
            "map_configs": self.maps,
            "cell_configs": self.cells
        })
    }
}

impl Default for StorageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_tool_builder_basic() {
        let tool = ToolBuilder::new("test_tool")
            .description("A test tool")
            .parameter("input", "string", "Test input", true)
            .parameter("count", "integer", "Count parameter", false)
            .handler(|args| async move {
                Ok(serde_json::json!({
                    "result": format!("Got: {}", args.get("input").unwrap_or(&JsonValue::Null))
                }))
            })
            .build()
            .expect("Should build successfully");

        let info = tool.info();
        assert_eq!(info.name, "test_tool");
        assert_eq!(info.description, "A test tool");

        let result = tool
            .execute(serde_json::json!({"input": "hello"}))
            .await
            .expect("Should execute successfully");

        assert_eq!(result["result"], "Got: \"hello\"");
    }

    #[test]
    fn test_tool_builder_no_handler_fails() {
        let result = ToolBuilder::new("incomplete_tool")
            .description("Missing handler")
            .parameter("input", "string", "Test input", true)
            .build();

        assert!(result.is_err());
        match result {
            Err(err) => assert!(err.to_string().contains("handler is required")),
            Ok(_) => panic!("Expected build to fail without handler"),
        }
    }

    #[tokio::test]
    async fn test_tool_builder_with_defaults() {
        let tool = ToolBuilder::new("tool_with_defaults")
            .description("Tool with default parameters")
            .parameter_with_default("name", "string", "User name", json!("anonymous"))
            .parameter_with_default("age", "integer", "User age", json!(18))
            .handler(|args| async move {
                let default_name = json!("unknown");
                let default_age = json!(0);
                let name = args.get("name").unwrap_or(&default_name);
                let age = args.get("age").unwrap_or(&default_age);
                Ok(json!({"greeting": format!("Hello {}, age {}", name, age)}))
            })
            .build()
            .unwrap();

        let info = tool.info();
        let properties = info.input_schema["properties"].as_object().unwrap();

        assert!(properties.contains_key("name"));
        assert!(properties.contains_key("age"));
        assert_eq!(properties["name"]["default"], json!("anonymous"));
        assert_eq!(properties["age"]["default"], json!(18));

        // Test execution with defaults
        let result = tool.execute(json!({})).await.unwrap();
        assert!(result["greeting"].as_str().unwrap().contains("unknown"));
    }

    #[tokio::test]
    async fn test_tool_builder_enum_parameter() {
        let tool = ToolBuilder::new("enum_tool")
            .description("Tool with enum parameter")
            .enum_parameter(
                "color",
                "Choose a color",
                vec!["red".to_string(), "green".to_string(), "blue".to_string()],
                true,
            )
            .parameter("intensity", "number", "Color intensity", false)
            .handler(|args| async move {
                let color = args["color"].as_str().unwrap_or("unknown");
                Ok(json!({"selected_color": color}))
            })
            .build()
            .unwrap();

        let info = tool.info();
        let properties = info.input_schema["properties"].as_object().unwrap();
        let color_param = &properties["color"];

        assert_eq!(color_param["type"], "string");
        assert!(color_param.get("enum").is_some());
        assert_eq!(color_param["enum"], json!(["red", "green", "blue"]));

        let required = info.input_schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("color")));

        let result = tool.execute(json!({"color": "red"})).await.unwrap();
        assert_eq!(result["selected_color"], "red");
    }

    #[test]
    fn test_tool_builder_fluent_interface() {
        // Test that all builder methods return Self for chaining
        let builder = ToolBuilder::new("fluent_test")
            .description("Testing fluent interface")
            .parameter("param1", "string", "First param", true)
            .parameter_with_default("param2", "integer", "Second param", json!(42))
            .enum_parameter(
                "param3",
                "Third param",
                vec!["a".to_string(), "b".to_string()],
                false,
            );

        // Should be able to add handler and build
        let tool = builder
            .handler(|_| async move { Ok(json!({"success": true})) })
            .build()
            .unwrap();

        assert_eq!(tool.info().name, "fluent_test");
    }

    #[tokio::test]
    async fn test_built_tool_rmcp_conversion() {
        let tool = ToolBuilder::new("rmcp_test")
            .description("Test RMCP conversion")
            .parameter("input", "string", "Test input", true)
            .handler(|_| async move { Ok(json!({"result": "ok"})) })
            .build()
            .unwrap();

        let rmcp_tool = tool.to_rmcp_tool();
        assert_eq!(rmcp_tool.name, "rmcp_test");
        assert_eq!(rmcp_tool.description.unwrap(), "Test RMCP conversion");
        assert!(!rmcp_tool.input_schema.is_empty());
        assert!(rmcp_tool.input_schema.contains_key("properties"));
    }

    #[tokio::test]
    async fn test_built_tool_execution_error() {
        let tool = ToolBuilder::new("error_tool")
            .description("Tool that fails")
            .parameter("should_fail", "boolean", "Whether to fail", false)
            .handler(|args| async move {
                if args
                    .get("should_fail")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    Err(IcarusError::Tool(
                        crate::error::ToolError::operation_failed("Test failure"),
                    ))
                } else {
                    Ok(json!({"success": true}))
                }
            })
            .build()
            .unwrap();

        // Test success case
        let result = tool.execute(json!({"should_fail": false})).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["success"], true);

        // Test error case
        let result = tool.execute(json!({"should_fail": true})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Test failure"));
    }

    #[test]
    fn test_parameter_spec_creation() {
        let spec = ParameterSpec {
            param_type: "string".to_string(),
            description: "Test parameter".to_string(),
            required: true,
            default: Some(json!("default_value")),
            enum_values: Some(vec!["a".to_string(), "b".to_string()]),
        };

        assert_eq!(spec.param_type, "string");
        assert_eq!(spec.description, "Test parameter");
        assert!(spec.required);
        assert_eq!(spec.default, Some(json!("default_value")));
        assert_eq!(
            spec.enum_values,
            Some(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn test_server_builder_basic() {
        let server = ServerBuilder::new("Test Server")
            .version("2.0.0")
            .description("A test server")
            .metadata(
                "custom_field",
                JsonValue::String("custom_value".to_string()),
            )
            .build();

        assert_eq!(server.name, "Test Server");
        assert_eq!(server.version, "2.0.0");
        assert_eq!(server.description, "A test server");

        let info = server.info();
        assert_eq!(info["name"], "Test Server");
        assert_eq!(info["custom_field"], "custom_value");
    }

    #[test]
    fn test_server_builder_with_tools() {
        // Create a simple tool for testing
        let tool1 = ToolBuilder::new("tool1")
            .description("First tool")
            .parameter("input", "string", "Input", true)
            .handler(|_| async move { Ok(json!({"result": "tool1"})) })
            .build()
            .unwrap();

        let tool2 = ToolBuilder::new("tool2")
            .description("Second tool")
            .parameter("data", "object", "Data", false)
            .handler(|_| async move { Ok(json!({"result": "tool2"})) })
            .build()
            .unwrap();

        let server = ServerBuilder::new("Multi-tool Server")
            .description("Server with multiple tools")
            .add_tool(tool1)
            .add_tool(tool2)
            .build();

        assert_eq!(server.tools.len(), 2);
        assert_eq!(server.name, "Multi-tool Server");

        let info = server.info();
        let tools_array = info["tools"].as_array().unwrap();
        assert_eq!(tools_array.len(), 2);
        assert_eq!(tools_array[0]["name"], "tool1");
        assert_eq!(tools_array[1]["name"], "tool2");
    }

    #[test]
    fn test_server_builder_add_multiple_tools() {
        let tool1 = ToolBuilder::new("batch_tool1")
            .description("Batch tool 1")
            .handler(|_| async move { Ok(json!({"result": "batch1"})) })
            .build()
            .unwrap();

        let tool2 = ToolBuilder::new("batch_tool2")
            .description("Batch tool 2")
            .handler(|_| async move { Ok(json!({"result": "batch2"})) })
            .build()
            .unwrap();

        let tools: Vec<Box<dyn IcarusTool>> = vec![Box::new(tool1), Box::new(tool2)];

        let server = ServerBuilder::new("Batch Server").add_tools(tools).build();

        assert_eq!(server.tools.len(), 2);
    }

    #[test]
    fn test_server_config_find_tool() {
        let tool = ToolBuilder::new("findable_tool")
            .description("Tool that can be found")
            .handler(|_| async move { Ok(json!({"found": true})) })
            .build()
            .unwrap();

        let server = ServerBuilder::new("Searchable Server")
            .add_tool(tool)
            .build();

        // Test finding existing tool
        let found = server.find_tool("findable_tool");
        assert!(found.is_some());
        assert_eq!(found.unwrap().info().name, "findable_tool");

        // Test finding non-existing tool
        let not_found = server.find_tool("missing_tool");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_server_config_tools_access() {
        let tool = ToolBuilder::new("access_test")
            .description("Tool for access testing")
            .handler(|_| async move { Ok(json!({"accessed": true})) })
            .build()
            .unwrap();

        let server = ServerBuilder::new("Access Server").add_tool(tool).build();

        let tools = server.tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].info().name, "access_test");
    }

    #[test]
    fn test_storage_builder_basic() {
        let storage = StorageBuilder::new()
            .add_map::<String, u64>("users")
            .add_cell::<u64>("counter", JsonValue::Number(0.into()))
            .build();

        assert_eq!(storage.maps.len(), 1);
        assert_eq!(storage.cells.len(), 1);

        let code = storage.generate_code();
        assert!(code.contains("MEMORY_MANAGER"));
        assert!(code.contains("USERS"));
        assert!(code.contains("COUNTER"));
    }

    #[test]
    fn test_storage_builder_auto_memory_ids() {
        let storage = StorageBuilder::new()
            .add_map::<String, String>("map1")
            .add_map::<u64, String>("map2")
            .add_cell::<bool>("cell1", json!(false))
            .add_cell::<i32>("cell2", json!(42))
            .build();

        assert_eq!(storage.maps.len(), 2);
        assert_eq!(storage.cells.len(), 2);

        // Check auto-assigned memory IDs
        assert_eq!(storage.maps[0].memory_id, 0);
        assert_eq!(storage.maps[1].memory_id, 1);
        assert_eq!(storage.cells[0].memory_id, 2);
        assert_eq!(storage.cells[1].memory_id, 3);
    }

    #[test]
    fn test_storage_builder_explicit_memory_ids() {
        let storage = StorageBuilder::new()
            .add_map_with_id::<String, u64>("high_priority", 10)
            .add_map::<String, String>("auto_assigned")
            .add_cell_with_id::<bool>("special_cell", 5, json!(true))
            .add_cell::<i32>("normal_cell", json!(0))
            .build();

        assert_eq!(storage.maps.len(), 2);
        assert_eq!(storage.cells.len(), 2);

        // Check explicit IDs
        assert_eq!(storage.maps[0].memory_id, 10);
        assert_eq!(storage.cells[0].memory_id, 5);

        // Check auto-assigned IDs (should be after the highest explicit ID)
        assert_eq!(storage.maps[1].memory_id, 11);
        assert_eq!(storage.cells[1].memory_id, 12);
    }

    #[test]
    fn test_storage_builder_type_names() {
        let storage = StorageBuilder::new()
            .add_map::<String, u64>("users")
            .add_map::<u64, Vec<String>>("permissions")
            .add_cell::<bool>("enabled", json!(true))
            .build();

        assert_eq!(storage.maps[0].key_type, std::any::type_name::<String>());
        assert_eq!(storage.maps[0].value_type, std::any::type_name::<u64>());
        assert_eq!(storage.maps[1].key_type, std::any::type_name::<u64>());
        assert_eq!(
            storage.maps[1].value_type,
            std::any::type_name::<Vec<String>>()
        );
        assert_eq!(storage.cells[0].value_type, std::any::type_name::<bool>());
    }

    #[test]
    fn test_storage_config_code_generation() {
        let storage = StorageBuilder::new()
            .add_map::<String, u64>("test_map")
            .add_cell::<bool>("test_cell", json!(false))
            .build();

        let code = storage.generate_code();

        // Check essential parts of generated code
        assert!(code.contains("thread_local!"));
        assert!(code.contains("MEMORY_MANAGER"));
        assert!(code.contains("TEST_MAP"));
        assert!(code.contains("TEST_CELL"));
        assert!(code.contains("StableBTreeMap"));
        assert!(code.contains("StableCell"));
        assert!(code.contains("MemoryId::new(0)"));
        assert!(code.contains("MemoryId::new(1)"));
    }

    #[test]
    fn test_storage_config_summary() {
        let storage = StorageBuilder::new()
            .add_map::<String, u64>("map1")
            .add_map::<i32, String>("map2")
            .add_cell::<bool>("cell1", json!(true))
            .build();

        let summary = storage.summary();
        assert_eq!(summary["maps"], 2);
        assert_eq!(summary["cells"], 1);
        assert_eq!(summary["total_memory_ids"], 3);

        let map_configs = summary["map_configs"].as_array().unwrap();
        assert_eq!(map_configs.len(), 2);
        assert_eq!(map_configs[0]["name"], "map1");
        assert_eq!(map_configs[1]["name"], "map2");

        let cell_configs = summary["cell_configs"].as_array().unwrap();
        assert_eq!(cell_configs.len(), 1);
        assert_eq!(cell_configs[0]["name"], "cell1");
    }

    #[test]
    fn test_storage_builder_default() {
        let storage1 = StorageBuilder::default();
        let storage2 = StorageBuilder::new();

        // Both should start with same state
        assert_eq!(storage1.next_memory_id, storage2.next_memory_id);
        assert_eq!(storage1.maps.len(), storage2.maps.len());
        assert_eq!(storage1.cells.len(), storage2.cells.len());
    }

    #[test]
    fn test_storage_config_null_default_handling() {
        let storage = StorageBuilder::new()
            .add_cell::<Option<String>>("optional_value", json!(null))
            .build();

        let code = storage.generate_code();
        assert!(code.contains("Default::default()"));
    }

    #[test]
    fn test_storage_map_config_serialization() {
        let config = StorageMapConfig {
            name: "test_map".to_string(),
            memory_id: 5,
            key_type: "String".to_string(),
            value_type: "u64".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("test_map"));
        assert!(json.contains("\"memory_id\":5"));
        assert!(json.contains("String"));
        assert!(json.contains("u64"));
    }

    #[test]
    fn test_storage_cell_config_serialization() {
        let config = StorageCellConfig {
            name: "test_cell".to_string(),
            memory_id: 3,
            value_type: "bool".to_string(),
            default_value: json!(true),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("test_cell"));
        assert!(json.contains("\"memory_id\":3"));
        assert!(json.contains("bool"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_server_builder_defaults() {
        let server = ServerBuilder::new("Minimal Server").build();

        assert_eq!(server.name, "Minimal Server");
        assert_eq!(server.version, "1.0.0"); // Default version
        assert!(server.description.is_empty()); // Default empty description
        assert_eq!(server.tools.len(), 0);
        assert_eq!(server.metadata.len(), 0);
    }

    #[test]
    fn test_server_config_empty_tools() {
        let server = ServerBuilder::new("Empty Server").build();
        let info = server.info();

        assert_eq!(info["tools"].as_array().unwrap().len(), 0);
        assert!(server.find_tool("any_tool").is_none());
    }
}
