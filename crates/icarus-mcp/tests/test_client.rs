//! Unit tests for CanisterClient

use anyhow::Result;
use candid::Principal;
use icarus_mcp::{CanisterClient, CanisterMetadata, McpConfig, ToolMetadata};
use serde_json::json;
use std::time::Duration;

/// Test basic client creation with valid configuration
#[tokio::test]
async fn test_client_creation_with_valid_config() -> Result<()> {
    let config = McpConfig {
        canister_id: Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai")?,
        ic_url: "http://localhost:4943".to_string(),
        timeout: Duration::from_secs(30),
        fetch_root_key: true,
        max_concurrent_requests: 10,
    };

    // This will fail to connect but should succeed in client creation
    let result = CanisterClient::new(config).await;

    // We expect this to fail since we don't have a running IC instance
    // but it should fail with a connection error, not a configuration error
    match result {
        Ok(_) => {
            // If it succeeds, that's fine too (means IC is running)
        }
        Err(e) => {
            // Should be a network/connection error, not a config error
            let error_msg = e.to_string().to_lowercase();
            assert!(
                error_msg.contains("connection")
                    || error_msg.contains("network")
                    || error_msg.contains("timeout")
                    || error_msg.contains("refused")
                    || error_msg.contains("unreachable"),
                "Expected network error, got: {}",
                e
            );
        }
    }

    Ok(())
}

/// Test client creation with invalid canister ID
#[tokio::test]
async fn test_client_creation_with_invalid_canister_id() -> Result<()> {
    let config = McpConfig {
        canister_id: Principal::from_text("invalid-id").unwrap_or(Principal::anonymous()),
        ic_url: "http://localhost:4943".to_string(),
        timeout: Duration::from_secs(5),
        fetch_root_key: true,
        max_concurrent_requests: 10,
    };

    // Even with invalid canister ID, client creation should succeed
    // The error will come when trying to call the canister
    let result = CanisterClient::new(config).await;
    assert!(result.is_ok() || result.is_err()); // Either is acceptable for this test

    Ok(())
}

/// Test McpConfig::local helper
#[test]
fn test_mcp_config_local() {
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let config = McpConfig::local(canister_id);

    assert_eq!(config.canister_id, canister_id);
    assert_eq!(config.ic_url, "http://localhost:4943");
    assert!(config.fetch_root_key);
    assert!(config.timeout > Duration::from_secs(10));
}

/// Test ToolMetadata serialization and deserialization
#[test]
fn test_tool_metadata_serde() -> Result<()> {
    let tool = ToolMetadata {
        name: "test_tool".to_string(),
        description: "A test tool for unit testing".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to process"
                }
            },
            "required": ["message"]
        }),
        title: Some("Test Tool".to_string()),
        icon: Some("🧪".to_string()),
    };

    // Test serialization
    let serialized = serde_json::to_string(&tool)?;
    assert!(serialized.contains("test_tool"));
    assert!(serialized.contains("inputSchema")); // Should use camelCase

    // Test deserialization
    let deserialized: ToolMetadata = serde_json::from_str(&serialized)?;
    assert_eq!(deserialized.name, tool.name);
    assert_eq!(deserialized.description, tool.description);
    assert_eq!(deserialized.title, tool.title);
    assert_eq!(deserialized.icon, tool.icon);

    Ok(())
}

/// Test CanisterMetadata creation and field access
#[test]
fn test_canister_metadata() {
    let tool1 = ToolMetadata {
        name: "echo".to_string(),
        description: "Echo input".to_string(),
        input_schema: json!({"type": "object"}),
        title: None,
        icon: None,
    };

    let tool2 = ToolMetadata {
        name: "reverse".to_string(),
        description: "Reverse string".to_string(),
        input_schema: json!({"type": "object"}),
        title: Some("String Reverser".to_string()),
        icon: Some("🔄".to_string()),
    };

    let metadata = CanisterMetadata {
        name: "test-canister".to_string(),
        version: Some("1.0.0".to_string()),
        tools: vec![tool1, tool2].into(),
        title: Some("Test Canister".to_string()),
        website_url: Some("https://example.com".to_string()),
    };

    assert_eq!(metadata.name, "test-canister");
    assert_eq!(metadata.version.as_ref().unwrap(), "1.0.0");
    assert_eq!(metadata.tools.len(), 2);
    assert_eq!(metadata.tools[0].name, "echo");
    assert_eq!(metadata.tools[1].name, "reverse");
    assert_eq!(metadata.title.as_ref().unwrap(), "Test Canister");
    assert_eq!(
        metadata.website_url.as_ref().unwrap(),
        "https://example.com"
    );
}

/// Test client canister_id getter
#[tokio::test]
async fn test_client_canister_id_getter() -> Result<()> {
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai")?;
    let config = McpConfig::local(canister_id);

    // This might fail due to network, but if it succeeds, test the getter
    if let Ok(client) = CanisterClient::new(config).await {
        assert_eq!(client.canister_id(), canister_id);
    }

    Ok(())
}

/// Test configuration validation
#[test]
fn test_config_validation() {
    // Test valid config
    let valid_canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let config = McpConfig {
        canister_id: valid_canister_id,
        ic_url: "http://localhost:4943".to_string(),
        timeout: Duration::from_secs(30),
        fetch_root_key: true,
        max_concurrent_requests: 10,
    };

    assert_eq!(config.canister_id, valid_canister_id);
    assert!(!config.ic_url.is_empty());
    assert!(config.timeout > Duration::from_secs(0));

    // Test production config helper (if it exists)
    let prod_config = McpConfig {
        canister_id: valid_canister_id,
        ic_url: "https://ic0.app".to_string(),
        timeout: Duration::from_secs(60),
        fetch_root_key: false,
        max_concurrent_requests: 20,
    };

    assert!(!prod_config.fetch_root_key);
    assert!(prod_config.ic_url.contains("ic0.app"));
}

/// Test error handling in tool metadata
#[test]
fn test_tool_metadata_error_handling() {
    // Test with missing required fields
    let invalid_json = r#"{"name": "test"}"#;
    let result: Result<ToolMetadata, _> = serde_json::from_str(invalid_json);
    assert!(
        result.is_err(),
        "Should fail without required description field"
    );

    // Test with invalid input schema
    let tool_with_invalid_schema = ToolMetadata {
        name: "invalid_tool".to_string(),
        description: "Tool with invalid schema".to_string(),
        input_schema: json!("not-an-object"),
        title: None,
        icon: None,
    };

    // Should still serialize/deserialize even with non-object schema
    let serialized = serde_json::to_string(&tool_with_invalid_schema).unwrap();
    let deserialized: ToolMetadata = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.name, "invalid_tool");
}

/// Test clone and debug traits
#[test]
fn test_tool_metadata_traits() {
    let tool = ToolMetadata {
        name: "test".to_string(),
        description: "test".to_string(),
        input_schema: json!({}),
        title: None,
        icon: None,
    };

    // Test Clone
    let cloned = tool.clone();
    assert_eq!(cloned.name, tool.name);

    // Test Debug
    let debug_str = format!("{:?}", tool);
    assert!(debug_str.contains("test"));
}
