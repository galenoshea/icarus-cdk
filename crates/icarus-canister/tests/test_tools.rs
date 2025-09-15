//! Tests for tool registry and dispatch system

use icarus_canister::tools::{ToolFunction, ToolRegistration, ToolRegistry};
use icarus_core::error::ToolError;
use serde_json::{json, Value};

/// Create a simple test tool function
fn create_test_tool(result: Result<Value, ToolError>) -> ToolFunction {
    Box::new(move |_args: Value| {
        let result = result.clone();
        Box::pin(async move { result })
    })
}

/// Create an echo tool that returns the input
fn create_echo_tool() -> ToolFunction {
    Box::new(move |args: Value| {
        Box::pin(async move { Ok(args) })
    })
}

/// Create a calculator tool for more complex testing
fn create_calculator_tool() -> ToolFunction {
    Box::new(move |args: Value| {
        Box::pin(async move {
            let obj = args.as_object().ok_or_else(|| {
                ToolError::invalid_input("args must be an object")
            })?;

            let a = obj.get("a")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| ToolError::invalid_input("'a' must be a number"))?;

            let b = obj.get("b")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| ToolError::invalid_input("'b' must be a number"))?;

            let operation = obj.get("operation")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::invalid_input("'operation' must be a string"))?;

            let result = match operation {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => {
                    if b == 0.0 {
                        return Err(ToolError::invalid_input("Cannot divide by zero"));
                    }
                    a / b
                },
                _ => return Err(ToolError::invalid_input("Unknown operation")),
            };

            Ok(json!({"result": result}))
        })
    })
}

/// Test ToolRegistry creation
#[tokio::test]
async fn test_tool_registry_creation() {
    let registry = ToolRegistry::new();
    assert_eq!(registry.list_tools().len(), 0);

    let default_registry = ToolRegistry::default();
    assert_eq!(default_registry.list_tools().len(), 0);
}

/// Test tool registration
#[tokio::test]
async fn test_tool_registration() {
    let mut registry = ToolRegistry::new();

    let registration = ToolRegistration {
        name: "test_tool".to_string(),
        description: "A test tool".to_string(),
        function: create_test_tool(Ok(json!({"success": true}))),
    };

    registry.register(registration);

    let tools = registry.list_tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].0, "test_tool");
    assert_eq!(tools[0].1, "A test tool");
}

/// Test multiple tool registration
#[tokio::test]
async fn test_multiple_tool_registration() {
    let mut registry = ToolRegistry::new();

    // Register first tool
    let registration1 = ToolRegistration {
        name: "tool1".to_string(),
        description: "First tool".to_string(),
        function: create_test_tool(Ok(json!({"tool": 1}))),
    };
    registry.register(registration1);

    // Register second tool
    let registration2 = ToolRegistration {
        name: "tool2".to_string(),
        description: "Second tool".to_string(),
        function: create_test_tool(Ok(json!({"tool": 2}))),
    };
    registry.register(registration2);

    let tools = registry.list_tools();
    assert_eq!(tools.len(), 2);

    // Check that both tools are present
    let tool_names: Vec<String> = tools.iter().map(|(name, _)| name.clone()).collect();
    assert!(tool_names.contains(&"tool1".to_string()));
    assert!(tool_names.contains(&"tool2".to_string()));
}

/// Test tool execution with successful result
#[tokio::test]
async fn test_tool_execution_success() {
    let mut registry = ToolRegistry::new();

    let expected_result = json!({"message": "Hello, World!"});
    let registration = ToolRegistration {
        name: "hello_tool".to_string(),
        description: "Says hello".to_string(),
        function: create_test_tool(Ok(expected_result.clone())),
    };

    registry.register(registration);

    let result = registry.execute("hello_tool", json!({})).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_result);
}

/// Test tool execution with error result
#[tokio::test]
async fn test_tool_execution_error() {
    let mut registry = ToolRegistry::new();

    let expected_error = ToolError::invalid_input("Invalid input");
    let registration = ToolRegistration {
        name: "error_tool".to_string(),
        description: "Always errors".to_string(),
        function: create_test_tool(Err(expected_error)),
    };

    registry.register(registration);

    let result = registry.execute("error_tool", json!({})).await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), "Invalid input: Invalid input");
}

/// Test tool execution with non-existent tool
#[tokio::test]
async fn test_tool_execution_not_found() {
    let registry = ToolRegistry::new();

    let result = registry.execute("nonexistent_tool", json!({})).await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Tool 'nonexistent_tool' not found"));
}

/// Test echo tool functionality
#[tokio::test]
async fn test_echo_tool() {
    let mut registry = ToolRegistry::new();

    let registration = ToolRegistration {
        name: "echo".to_string(),
        description: "Echoes input".to_string(),
        function: create_echo_tool(),
    };

    registry.register(registration);

    let input = json!({"test": "data", "number": 42});
    let result = registry.execute("echo", input.clone()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), input);
}

/// Test calculator tool with addition
#[tokio::test]
async fn test_calculator_tool_addition() {
    let mut registry = ToolRegistry::new();

    let registration = ToolRegistration {
        name: "calculator".to_string(),
        description: "Performs calculations".to_string(),
        function: create_calculator_tool(),
    };

    registry.register(registration);

    let input = json!({"a": 5.0, "b": 3.0, "operation": "add"});
    let result = registry.execute("calculator", input).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!({"result": 8.0}));
}

/// Test calculator tool with division
#[tokio::test]
async fn test_calculator_tool_division() {
    let mut registry = ToolRegistry::new();

    let registration = ToolRegistration {
        name: "calculator".to_string(),
        description: "Performs calculations".to_string(),
        function: create_calculator_tool(),
    };

    registry.register(registration);

    let input = json!({"a": 10.0, "b": 2.0, "operation": "divide"});
    let result = registry.execute("calculator", input).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!({"result": 5.0}));
}

/// Test calculator tool with division by zero
#[tokio::test]
async fn test_calculator_tool_division_by_zero() {
    let mut registry = ToolRegistry::new();

    let registration = ToolRegistration {
        name: "calculator".to_string(),
        description: "Performs calculations".to_string(),
        function: create_calculator_tool(),
    };

    registry.register(registration);

    let input = json!({"a": 10.0, "b": 0.0, "operation": "divide"});
    let result = registry.execute("calculator", input).await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Cannot divide by zero"));
}

/// Test calculator tool with invalid operation
#[tokio::test]
async fn test_calculator_tool_invalid_operation() {
    let mut registry = ToolRegistry::new();

    let registration = ToolRegistration {
        name: "calculator".to_string(),
        description: "Performs calculations".to_string(),
        function: create_calculator_tool(),
    };

    registry.register(registration);

    let input = json!({"a": 5.0, "b": 3.0, "operation": "invalid"});
    let result = registry.execute("calculator", input).await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Unknown operation"));
}

/// Test calculator tool with missing arguments
#[tokio::test]
async fn test_calculator_tool_missing_args() {
    let mut registry = ToolRegistry::new();

    let registration = ToolRegistration {
        name: "calculator".to_string(),
        description: "Performs calculations".to_string(),
        function: create_calculator_tool(),
    };

    registry.register(registration);

    let input = json!({"a": 5.0}); // Missing 'b' and 'operation'
    let result = registry.execute("calculator", input).await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("must be"));
}

/// Test tool overwriting (same name)
#[tokio::test]
async fn test_tool_overwrite() {
    let mut registry = ToolRegistry::new();

    // Register first version
    let registration1 = ToolRegistration {
        name: "test_tool".to_string(),
        description: "First version".to_string(),
        function: create_test_tool(Ok(json!({"version": 1}))),
    };
    registry.register(registration1);

    // Register second version with same name
    let registration2 = ToolRegistration {
        name: "test_tool".to_string(),
        description: "Second version".to_string(),
        function: create_test_tool(Ok(json!({"version": 2}))),
    };
    registry.register(registration2);

    // Should have only one tool with the latest description
    let tools = registry.list_tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].1, "Second version");

    // Should execute the latest version
    let result = registry.execute("test_tool", json!({})).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!({"version": 2}));
}

/// Test empty tool list
#[tokio::test]
async fn test_empty_tool_list() {
    let registry = ToolRegistry::new();
    let tools = registry.list_tools();
    assert!(tools.is_empty());
}

/// Test large number of tools
#[tokio::test]
async fn test_many_tools() {
    let mut registry = ToolRegistry::new();

    // Register 100 tools
    for i in 0..100 {
        let registration = ToolRegistration {
            name: format!("tool_{}", i),
            description: format!("Tool number {}", i),
            function: create_test_tool(Ok(json!({"id": i}))),
        };
        registry.register(registration);
    }

    let tools = registry.list_tools();
    assert_eq!(tools.len(), 100);

    // Test execution of a few random tools
    for &i in &[0, 25, 50, 75, 99] {
        let result = registry.execute(&format!("tool_{}", i), json!({})).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!({"id": i}));
    }
}

/// Test tool registry async execution with simpler function
#[tokio::test]
async fn test_async_tool_execution() {
    let mut registry = ToolRegistry::new();

    // Use a simple async tool that doesn't require complex type annotations
    let registration = ToolRegistration {
        name: "async_tool".to_string(),
        description: "Simple async tool".to_string(),
        function: create_echo_tool(),
    };

    registry.register(registration);

    let input = json!({"async_test": true});
    let result = registry.execute("async_tool", input.clone()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), input);
}

/// Test ToolRegistration structure
#[test]
fn test_tool_registration_structure() {
    let registration = ToolRegistration {
        name: "test".to_string(),
        description: "test tool".to_string(),
        function: create_test_tool(Ok(json!({}))),
    };

    assert_eq!(registration.name, "test");
    assert_eq!(registration.description, "test tool");
    // Function is harder to test directly, but we know it exists
}