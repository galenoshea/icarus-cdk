//! Integration tests for the complete CDK workflow

use icarus_canister::core::tools::{ToolRegistration, ToolRegistry};
use icarus_core::error::ToolError;
use serde_json::json;

/// Test the complete workflow of creating and executing tools
#[tokio::test]
async fn test_complete_tool_workflow() {
    // Set up tools
    let mut tool_registry = ToolRegistry::new();

    let tool = ToolRegistration {
        name: "echo_tool".to_string(),
        description: "A tool that echoes input".to_string(),
        function: Box::new(|args| {
            Box::pin(async move {
                Ok(json!({
                    "echoed": args,
                    "processed": true
                }))
            })
        }),
    };

    tool_registry.register(tool);

    // Execute the tool
    let input = json!({"message": "test message"});
    let result = tool_registry.execute("echo_tool", input).await.unwrap();

    // Verify the results
    assert_eq!(result["echoed"]["message"], "test message");
    assert_eq!(result["processed"], true);
}

/// Test error handling across the CDK
#[tokio::test]
async fn test_error_handling() {
    let mut tool_registry = ToolRegistry::new();

    // Register a tool that always fails
    let failing_tool = ToolRegistration {
        name: "failing_tool".to_string(),
        description: "A tool that always fails".to_string(),
        function: Box::new(|_args| {
            Box::pin(async move { Err(ToolError::operation_failed("Intentional failure")) })
        }),
    };

    tool_registry.register(failing_tool);

    // Execute and expect failure
    let result = tool_registry.execute("failing_tool", json!({})).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ToolError::OperationFailed(msg) => {
            assert_eq!(msg, "Intentional failure");
        }
        _ => panic!("Expected OperationFailed error"),
    }
}

/// Test multiple tools working together
#[tokio::test]
async fn test_tool_composition() {
    let mut registry = ToolRegistry::new();

    // Tool 1: Preprocessor
    registry.register(ToolRegistration {
        name: "preprocess".to_string(),
        description: "Preprocesses data".to_string(),
        function: Box::new(|args| {
            Box::pin(async move {
                let text = args["text"].as_str().unwrap_or("");
                Ok(json!({
                    "processed": text.to_uppercase()
                }))
            })
        }),
    });

    // Tool 2: Analyzer
    registry.register(ToolRegistration {
        name: "analyze".to_string(),
        description: "Analyzes processed data".to_string(),
        function: Box::new(|args| {
            Box::pin(async move {
                let text = args["processed"].as_str().unwrap_or("");
                Ok(json!({
                    "length": text.len(),
                    "has_hello": text.contains("HELLO")
                }))
            })
        }),
    });

    // Execute tools in sequence
    let step1 = registry
        .execute("preprocess", json!({"text": "hello world"}))
        .await
        .unwrap();
    let step2 = registry.execute("analyze", step1).await.unwrap();

    assert_eq!(step2["length"], 11);
    assert_eq!(step2["has_hello"], true);
}
