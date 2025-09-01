//! Unit tests for the tools module

use icarus_canister::tools::{ToolRegistry, ToolRegistration, ToolFunction};
use icarus_core::error::ToolError;
use serde_json::{json, Value};

#[test]
fn test_tool_registry_creation() {
    let registry = ToolRegistry::new();
    assert!(registry.list_tools().is_empty());
}

#[test]
fn test_tool_registration() {
    let mut registry = ToolRegistry::new();
    
    let tool = ToolRegistration {
        name: "test_tool".to_string(),
        description: "A test tool".to_string(),
        function: ToolFunction::Query(Box::new(|_args| {
            Box::pin(async move {
                Ok(json!({"result": "success"}))
            })
        })),
    };
    
    registry.register(tool);
    
    let tools = registry.list_tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].0, "test_tool");
    assert_eq!(tools[0].1, "A test tool");
}

#[test]
fn test_default_tool_registry() {
    let registry = ToolRegistry::default();
    assert!(registry.list_tools().is_empty());
}

#[tokio::test]
async fn test_tool_execution() {
    let mut registry = ToolRegistry::new();
    
    let tool = ToolRegistration {
        name: "echo".to_string(),
        description: "Echoes input".to_string(),
        function: ToolFunction::Query(Box::new(|args| {
            Box::pin(async move {
                Ok(args.clone())
            })
        })),
    };
    
    registry.register(tool);
    
    let input = json!({"message": "hello"});
    let result = registry.execute("echo", input.clone()).await.unwrap();
    assert_eq!(result, input);
}

#[tokio::test]
async fn test_tool_not_found() {
    let registry = ToolRegistry::new();
    let result = registry.execute("nonexistent", json!({})).await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        ToolError::NotFound(name) => {
            assert_eq!(name, "nonexistent");
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_multiple_tools() {
    let mut registry = ToolRegistry::new();
    
    // Register first tool
    registry.register(ToolRegistration {
        name: "add".to_string(),
        description: "Adds two numbers".to_string(),
        function: ToolFunction::Query(Box::new(|args| {
            Box::pin(async move {
                let a = args["a"].as_i64().unwrap_or(0);
                let b = args["b"].as_i64().unwrap_or(0);
                Ok(json!({"result": a + b}))
            })
        })),
    });
    
    // Register second tool
    registry.register(ToolRegistration {
        name: "multiply".to_string(),
        description: "Multiplies two numbers".to_string(),
        function: ToolFunction::Query(Box::new(|args| {
            Box::pin(async move {
                let a = args["a"].as_i64().unwrap_or(1);
                let b = args["b"].as_i64().unwrap_or(1);
                Ok(json!({"result": a * b}))
            })
        })),
    });
    
    // Test both tools
    let add_result = registry.execute("add", json!({"a": 5, "b": 3})).await.unwrap();
    assert_eq!(add_result["result"], 8);
    
    let mul_result = registry.execute("multiply", json!({"a": 4, "b": 7})).await.unwrap();
    assert_eq!(mul_result["result"], 28);
    
    // Check listing
    let tools = registry.list_tools();
    assert_eq!(tools.len(), 2);
}