//! Integration tests for the complete SDK workflow

use icarus_core::prompts::{PromptBuilder, PromptRegistry};
use icarus_canister::tools::{ToolRegistry, ToolRegistration};
use icarus_core::error::ToolError;
use serde_json::json;
use std::collections::HashMap;

/// Test the complete workflow of creating a tool with prompts
#[tokio::test]
async fn test_complete_tool_workflow() {
    // Step 1: Set up prompts
    let mut prompt_registry = PromptRegistry::new();
    
    let system_prompt = PromptBuilder::new("system")
        .description("System prompt for the tool")
        .template("You are {{role}} helping with {{task}}")
        .arg_with_default("role", "The role of the assistant", "an assistant")
        .arg("task", "The task to help with", true)
        .build();
    
    prompt_registry.register(system_prompt);
    
    // Step 2: Set up tools
    let mut tool_registry = ToolRegistry::new();
    
    let tool = ToolRegistration {
        name: "prompt_tool".to_string(),
        description: "A tool that uses prompts".to_string(),
        function: Box::new(move |args| {
            let prompt_registry_clone = prompt_registry.clone();
            Box::pin(async move {
                let mut prompt_args = HashMap::new();
                prompt_args.insert("task".to_string(), "testing".to_string());
                
                let rendered = prompt_registry_clone
                    .render("system", &prompt_args)
                    .map_err(|e| ToolError::operation_failed(e))?;
                
                Ok(json!({
                    "prompt": rendered,
                    "input": args
                }))
            })
        }),
    };
    
    tool_registry.register(tool);
    
    // Step 3: Execute the tool
    let input = json!({"message": "test message"});
    let result = tool_registry.execute("prompt_tool", input).await.unwrap();
    
    // Step 4: Verify the results
    assert!(result["prompt"].as_str().unwrap().contains("an assistant"));
    assert!(result["prompt"].as_str().unwrap().contains("testing"));
    assert_eq!(result["input"]["message"], "test message");
}

/// Test error handling across the SDK
#[tokio::test]
async fn test_error_handling() {
    let mut tool_registry = ToolRegistry::new();
    
    // Register a tool that always fails
    let failing_tool = ToolRegistration {
        name: "failing_tool".to_string(),
        description: "A tool that always fails".to_string(),
        function: Box::new(|_args| {
            Box::pin(async move {
                Err(ToolError::operation_failed("Intentional failure"))
            })
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
    let step1 = registry.execute("preprocess", json!({"text": "hello world"})).await.unwrap();
    let step2 = registry.execute("analyze", step1).await.unwrap();
    
    assert_eq!(step2["length"], 11);
    assert_eq!(step2["has_hello"], true);
}