//! Unit tests for tools module
//!
//! These tests verify the tool registry, registration, and dispatch system
//! without requiring actual IC canister context.

#[cfg(feature = "canister")]
use icarus_core::error::ToolError;
#[cfg(feature = "canister")]
use icarus_core::tools::{create_schema_for, ToolInfo, ToolRegistration, ToolRegistry};
#[cfg(feature = "canister")]
use serde_json::{json, Value};
#[cfg(feature = "canister")]
use std::future::Future;
#[cfg(feature = "canister")]
use std::pin::Pin;

// Helper function to create a mock tool that always succeeds
#[cfg(feature = "canister")]
fn create_mock_success_tool(
    result: Value,
) -> Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value, ToolError>> + 'static>> + 'static>
{
    Box::new(move |_args: Value| {
        let result = result.clone();
        Box::pin(async move { Ok(result) })
    })
}

// Helper function to create a mock tool that always fails
#[cfg(feature = "canister")]
fn create_mock_error_tool(
    error_msg: &str,
) -> Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value, ToolError>> + 'static>> + 'static>
{
    let error_msg = error_msg.to_string();
    Box::new(move |_args: Value| {
        let error_msg = error_msg.clone();
        Box::pin(async move { Err(ToolError::operation_failed(error_msg)) })
    })
}

// Helper function to create a mock tool that echoes its input
#[cfg(feature = "canister")]
fn create_mock_echo_tool(
) -> Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value, ToolError>> + 'static>> + 'static>
{
    Box::new(move |args: Value| Box::pin(async move { Ok(args) }))
}

#[cfg(all(test, feature = "canister"))]
mod tools_module_tests {
    use super::*;

    #[test]
    fn test_tool_info_creation() {
        let tool_info = ToolInfo {
            name: "test_tool".to_string(),
            description: "A tool for testing".to_string(),
            input_schema: r#"{"type": "object"}"#.to_string(),
            output_schema: r#"{"type": "string"}"#.to_string(),
        };

        assert_eq!(tool_info.name, "test_tool");
        assert_eq!(tool_info.description, "A tool for testing");
        assert_eq!(tool_info.input_schema, r#"{"type": "object"}"#);
        assert_eq!(tool_info.output_schema, r#"{"type": "string"}"#);
    }

    #[test]
    fn test_tool_info_clone() {
        let tool_info1 = ToolInfo {
            name: "original".to_string(),
            description: "Original description".to_string(),
            input_schema: "{}".to_string(),
            output_schema: "{}".to_string(),
        };

        let tool_info2 = tool_info1.clone();

        assert_eq!(tool_info1.name, tool_info2.name);
        assert_eq!(tool_info1.description, tool_info2.description);
        assert_eq!(tool_info1.input_schema, tool_info2.input_schema);
        assert_eq!(tool_info1.output_schema, tool_info2.output_schema);
    }

    #[test]
    fn test_tool_info_serialization() {
        let tool_info = ToolInfo {
            name: "serializable_tool".to_string(),
            description: "Tool for testing serialization".to_string(),
            input_schema: r#"{"type": "object", "properties": {"param": {"type": "string"}}}"#
                .to_string(),
            output_schema: r#"{"type": "object", "properties": {"result": {"type": "boolean"}}}"#
                .to_string(),
        };

        // Test serialization
        let serialized = serde_json::to_string(&tool_info).unwrap();
        assert!(!serialized.is_empty());
        assert!(serialized.contains("serializable_tool"));
        assert!(serialized.contains("testing serialization"));

        // Test deserialization
        let deserialized: ToolInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.name, tool_info.name);
        assert_eq!(deserialized.description, tool_info.description);
        assert_eq!(deserialized.input_schema, tool_info.input_schema);
        assert_eq!(deserialized.output_schema, tool_info.output_schema);
    }

    #[test]
    fn test_create_schema_for() {
        let schema = create_schema_for::<String>();

        // Parse the schema JSON
        let parsed: Value = serde_json::from_str(&schema).unwrap();

        assert_eq!(parsed["type"], "object");
        assert_eq!(parsed["description"], "Type schema (auto-generated)");
    }

    #[test]
    fn test_tool_registry_new() {
        let registry = ToolRegistry::new();
        let tools = registry.list_tools();
        assert!(tools.is_empty());
    }

    #[test]
    fn test_tool_registry_default() {
        let registry = ToolRegistry::default();
        let tools = registry.list_tools();
        assert!(tools.is_empty());
    }

    #[test]
    fn test_tool_registry_register_single_tool() {
        let mut registry = ToolRegistry::new();

        let registration = ToolRegistration {
            name: "hello_world".to_string(),
            description: "Says hello to the world".to_string(),
            function: create_mock_success_tool(json!("Hello, World!")),
        };

        registry.register(registration);

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].0, "hello_world");
        assert_eq!(tools[0].1, "Says hello to the world");
    }

    #[test]
    fn test_tool_registry_register_multiple_tools() {
        let mut registry = ToolRegistry::new();

        let tools_data = vec![
            ("echo", "Echoes the input", json!(null)),
            ("add", "Adds two numbers", json!(42)),
            ("greet", "Greets a user", json!("Hello!")),
        ];

        for (name, desc, result) in tools_data {
            let registration = ToolRegistration {
                name: name.to_string(),
                description: desc.to_string(),
                function: create_mock_success_tool(result),
            };
            registry.register(registration);
        }

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 3);

        let tool_names: Vec<String> = tools.iter().map(|(name, _)| name.clone()).collect();
        assert!(tool_names.contains(&"echo".to_string()));
        assert!(tool_names.contains(&"add".to_string()));
        assert!(tool_names.contains(&"greet".to_string()));
    }

    #[tokio::test]
    async fn test_tool_registry_execute_success() {
        let mut registry = ToolRegistry::new();

        let registration = ToolRegistration {
            name: "success_tool".to_string(),
            description: "Always succeeds".to_string(),
            function: create_mock_success_tool(json!({"status": "success", "value": 123})),
        };

        registry.register(registration);

        let args = json!({"input": "test"});
        let result = registry.execute("success_tool", args).await;

        assert!(result.is_ok());
        let result_value = result.unwrap();
        assert_eq!(result_value["status"], "success");
        assert_eq!(result_value["value"], 123);
    }

    #[tokio::test]
    async fn test_tool_registry_execute_error() {
        let mut registry = ToolRegistry::new();

        let registration = ToolRegistration {
            name: "error_tool".to_string(),
            description: "Always fails".to_string(),
            function: create_mock_error_tool("Simulated error"),
        };

        registry.register(registration);

        let args = json!({"input": "test"});
        let result = registry.execute("error_tool", args).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Simulated error"));
    }

    #[tokio::test]
    async fn test_tool_registry_execute_not_found() {
        let registry = ToolRegistry::new();

        let args = json!({"input": "test"});
        let result = registry.execute("nonexistent_tool", args).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error
            .to_string()
            .contains("Tool 'nonexistent_tool' not found"));
    }

    #[tokio::test]
    async fn test_tool_registry_execute_echo() {
        let mut registry = ToolRegistry::new();

        let registration = ToolRegistration {
            name: "echo".to_string(),
            description: "Echoes input".to_string(),
            function: create_mock_echo_tool(),
        };

        registry.register(registration);

        let test_inputs = vec![
            json!("simple string"),
            json!(42),
            json!({"complex": "object", "with": [1, 2, 3]}),
            json!(true),
            json!(null),
        ];

        for input in test_inputs {
            let result = registry.execute("echo", input.clone()).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), input);
        }
    }

    #[test]
    fn test_tool_registry_list_tools_empty() {
        let registry = ToolRegistry::new();
        let tools = registry.list_tools();
        assert!(tools.is_empty());
    }

    #[test]
    fn test_tool_registry_list_tools_populated() {
        let mut registry = ToolRegistry::new();

        // Add tools with different descriptions
        let tools_data = vec![
            ("calculator", "Performs mathematical calculations"),
            ("weather", "Gets weather information"),
            ("file_reader", "Reads file contents"),
        ];

        for (name, desc) in &tools_data {
            let registration = ToolRegistration {
                name: name.to_string(),
                description: desc.to_string(),
                function: create_mock_success_tool(json!(null)),
            };
            registry.register(registration);
        }

        let listed_tools = registry.list_tools();
        assert_eq!(listed_tools.len(), 3);

        // Verify all tools are listed with correct descriptions
        for (expected_name, expected_desc) in tools_data {
            let found = listed_tools.iter().find(|(name, _)| name == expected_name);
            assert!(
                found.is_some(),
                "Tool '{}' not found in list",
                expected_name
            );
            assert_eq!(found.unwrap().1, expected_desc);
        }
    }

    #[test]
    fn test_tool_registry_overwrite_tool() {
        let mut registry = ToolRegistry::new();

        // Register initial tool
        let registration1 = ToolRegistration {
            name: "test_tool".to_string(),
            description: "First version".to_string(),
            function: create_mock_success_tool(json!("first")),
        };
        registry.register(registration1);

        // Register tool with same name (should overwrite)
        let registration2 = ToolRegistration {
            name: "test_tool".to_string(),
            description: "Second version".to_string(),
            function: create_mock_success_tool(json!("second")),
        };
        registry.register(registration2);

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].0, "test_tool");
        assert_eq!(tools[0].1, "Second version"); // Should have new description
    }

    #[tokio::test]
    async fn test_tool_registry_overwrite_execution() {
        let mut registry = ToolRegistry::new();

        // Register initial tool
        let registration1 = ToolRegistration {
            name: "dynamic_tool".to_string(),
            description: "Dynamic tool".to_string(),
            function: create_mock_success_tool(json!("original")),
        };
        registry.register(registration1);

        // Execute original tool
        let result1 = registry.execute("dynamic_tool", json!({})).await.unwrap();
        assert_eq!(result1, "original");

        // Register new version of same tool
        let registration2 = ToolRegistration {
            name: "dynamic_tool".to_string(),
            description: "Updated dynamic tool".to_string(),
            function: create_mock_success_tool(json!("updated")),
        };
        registry.register(registration2);

        // Execute updated tool
        let result2 = registry.execute("dynamic_tool", json!({})).await.unwrap();
        assert_eq!(result2, "updated");
    }

    #[test]
    fn test_tool_edge_cases() {
        let mut registry = ToolRegistry::new();

        // Test with empty tool name
        let registration1 = ToolRegistration {
            name: "".to_string(),
            description: "Empty name tool".to_string(),
            function: create_mock_success_tool(json!(null)),
        };
        registry.register(registration1);

        // Test with empty description
        let registration2 = ToolRegistration {
            name: "empty_desc".to_string(),
            description: "".to_string(),
            function: create_mock_success_tool(json!(null)),
        };
        registry.register(registration2);

        // Test with special characters in name
        let registration3 = ToolRegistration {
            name: "tool-with_special.chars!".to_string(),
            description: "Special characters in name".to_string(),
            function: create_mock_success_tool(json!(null)),
        };
        registry.register(registration3);

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 3);

        let tool_names: Vec<String> = tools.iter().map(|(name, _)| name.clone()).collect();
        assert!(tool_names.contains(&"".to_string()));
        assert!(tool_names.contains(&"empty_desc".to_string()));
        assert!(tool_names.contains(&"tool-with_special.chars!".to_string()));
    }

    #[tokio::test]
    async fn test_tool_with_complex_arguments() {
        let mut registry = ToolRegistry::new();

        // Tool that processes complex arguments
        let registration = ToolRegistration {
            name: "complex_processor".to_string(),
            description: "Processes complex arguments".to_string(),
            function: Box::new(|args: Value| {
                Box::pin(async move {
                    // Extract and process arguments
                    if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
                        if let Some(count) = args.get("count").and_then(|v| v.as_u64()) {
                            return Ok(json!({
                                "greeting": format!("Hello {} ({})", name, count),
                                "processed": true
                            }));
                        }
                    }
                    Ok(json!({"error": "Invalid arguments"}))
                })
            }),
        };

        registry.register(registration);

        // Test with valid complex arguments
        let valid_args = json!({
            "name": "Alice",
            "count": 42,
            "extra": "ignored"
        });

        let result = registry
            .execute("complex_processor", valid_args)
            .await
            .unwrap();
        assert_eq!(result["greeting"], "Hello Alice (42)");
        assert_eq!(result["processed"], true);

        // Test with invalid arguments
        let invalid_args = json!({
            "wrong": "arguments"
        });

        let result = registry
            .execute("complex_processor", invalid_args)
            .await
            .unwrap();
        assert_eq!(result["error"], "Invalid arguments");
    }

    #[test]
    fn test_tool_info_with_complex_schemas() {
        let complex_input_schema = json!({
            "type": "object",
            "properties": {
                "user": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "minLength": 1},
                        "age": {"type": "integer", "minimum": 0}
                    },
                    "required": ["name"]
                },
                "options": {
                    "type": "array",
                    "items": {"type": "string"}
                }
            },
            "required": ["user"]
        })
        .to_string();

        let complex_output_schema = json!({
            "type": "object",
            "properties": {
                "result": {
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"},
                        "message": {"type": "string"},
                        "data": {"type": "array"}
                    }
                }
            }
        })
        .to_string();

        let tool_info = ToolInfo {
            name: "complex_tool".to_string(),
            description: "Tool with complex schemas".to_string(),
            input_schema: complex_input_schema.clone(),
            output_schema: complex_output_schema.clone(),
        };

        // Verify schemas are stored correctly
        assert_eq!(tool_info.input_schema, complex_input_schema);
        assert_eq!(tool_info.output_schema, complex_output_schema);

        // Verify schemas are valid JSON
        let _: Value = serde_json::from_str(&tool_info.input_schema).unwrap();
        let _: Value = serde_json::from_str(&tool_info.output_schema).unwrap();
    }

    #[test]
    fn test_schema_generation() {
        // Test schema generation for different types
        let string_schema = create_schema_for::<String>();
        let int_schema = create_schema_for::<i32>();
        let bool_schema = create_schema_for::<bool>();

        // All should be valid JSON
        let _: Value = serde_json::from_str(&string_schema).unwrap();
        let _: Value = serde_json::from_str(&int_schema).unwrap();
        let _: Value = serde_json::from_str(&bool_schema).unwrap();

        // All should have the same structure (simplified implementation)
        assert!(string_schema.contains("object"));
        assert!(int_schema.contains("object"));
        assert!(bool_schema.contains("object"));
    }

    #[tokio::test]
    async fn test_large_tool_registry() {
        let mut registry = ToolRegistry::new();

        // Register many tools
        for i in 0..100 {
            let registration = ToolRegistration {
                name: format!("tool_{:03}", i),
                description: format!("Auto-generated tool number {}", i),
                function: create_mock_success_tool(json!(i)),
            };
            registry.register(registration);
        }

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 100);

        // Test execution of a few random tools
        for i in &[0, 25, 50, 75, 99] {
            let tool_name = format!("tool_{:03}", i);
            let result = registry.execute(&tool_name, json!({})).await.unwrap();
            assert_eq!(result, *i);
        }

        // Test non-existent tool
        let result = registry.execute("tool_999", json!({})).await;
        assert!(result.is_err());
    }
}
