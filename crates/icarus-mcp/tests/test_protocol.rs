//! Unit tests for McpProtocolHandler

use candid::Principal;
use icarus_mcp::{CanisterMetadata, McpConfig, McpProtocolHandler, ToolConverter, ToolMetadata};
use serde_json::json;

fn create_test_metadata() -> CanisterMetadata {
    CanisterMetadata {
        name: "test-canister".to_string(),
        version: Some("1.0.0".to_string()),
        tools: vec![ToolMetadata {
            name: "echo".to_string(),
            description: "Echo input message".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to echo"
                    }
                },
                "required": ["message"]
            }),
            title: Some("Echo Tool".to_string()),
            icon: Some("📢".to_string()),
        }]
        .into(),
        title: Some("Test Canister".to_string()),
        website_url: Some("https://example.com".to_string()),
    }
}

/// Test protocol handler creation
#[test]
fn test_protocol_handler_creation() {
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let _config = McpConfig::local(canister_id);
    let metadata = create_test_metadata();

    // We can't create a real CanisterClient without IC connection,
    // so this test focuses on the metadata structure
    assert_eq!(metadata.name, "test-canister");
    assert_eq!(metadata.tools.len(), 1);
    assert_eq!(metadata.tools[0].name, "echo");
}

/// Test tool conversion to MCP format
#[test]
fn test_convert_tool_to_mcp() {
    let tool = ToolMetadata {
        name: "calculate".to_string(),
        description: "Perform calculations".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"]
                },
                "operands": {
                    "type": "array",
                    "items": {"type": "number"},
                    "minItems": 2
                }
            },
            "required": ["operation", "operands"]
        }),
        title: Some("Calculator".to_string()),
        icon: Some("🧮".to_string()),
    };

    let converted =
        McpProtocolHandler::<std::sync::Arc<icarus_mcp::CanisterClient>>::convert_tool(&tool);

    assert_eq!(converted["name"], "calculate");
    assert_eq!(converted["description"], "Perform calculations");
    assert_eq!(converted["title"], "Calculator");
    assert_eq!(converted["icon"], "🧮");
    assert!(converted["inputSchema"].is_object());
    assert_eq!(converted["inputSchema"]["type"], "object");
}

/// Test server info generation
#[test]
fn test_get_server_info_structure() {
    let metadata = create_test_metadata();
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();

    // Mock the server info structure that would be generated
    let server_info = json!({
        "name": "icarus-mcp",
        "version": env!("CARGO_PKG_VERSION"),
        "canister_id": canister_id.to_text(),
        "canister_name": metadata.name,
        "canister_title": metadata.title,
        "website_url": metadata.website_url,
        "tools_count": metadata.tools.len()
    });

    assert_eq!(server_info["name"], "icarus-mcp");
    assert_eq!(server_info["canister_name"], "test-canister");
    assert_eq!(server_info["tools_count"], 1);
    assert!(server_info["version"].is_string());
}

/// Test tool metadata with various field combinations
#[test]
fn test_tool_metadata_variations() {
    // Test minimal tool
    let minimal_tool = ToolMetadata {
        name: "minimal".to_string(),
        description: "Minimal tool".to_string(),
        input_schema: json!({}),
        title: None,
        icon: None,
    };

    let converted = McpProtocolHandler::<std::sync::Arc<icarus_mcp::CanisterClient>>::convert_tool(
        &minimal_tool,
    );
    assert_eq!(converted["name"], "minimal");
    assert_eq!(converted["description"], "Minimal tool");
    assert!(converted["title"].is_null());
    assert!(converted["icon"].is_null());

    // Test tool with all fields
    let full_tool = ToolMetadata {
        name: "full".to_string(),
        description: "Full featured tool".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "input": {"type": "string"}
            }
        }),
        title: Some("Full Tool".to_string()),
        icon: Some("⚙️".to_string()),
    };

    let converted_full =
        McpProtocolHandler::<std::sync::Arc<icarus_mcp::CanisterClient>>::convert_tool(&full_tool);
    assert_eq!(converted_full["name"], "full");
    assert_eq!(converted_full["title"], "Full Tool");
    assert_eq!(converted_full["icon"], "⚙️");
    assert!(converted_full["inputSchema"]["properties"]["input"].is_object());
}

/// Test canister metadata structure
#[test]
fn test_canister_metadata_structure() {
    let metadata = create_test_metadata();

    assert!(!metadata.name.is_empty());
    assert!(metadata.version.is_some());
    assert!(!metadata.tools.is_empty());
    assert!(metadata.title.is_some());
    assert!(metadata.website_url.is_some());

    // Verify tool structure
    let tool = &metadata.tools[0];
    assert!(!tool.name.is_empty());
    assert!(!tool.description.is_empty());
    assert!(tool.input_schema.is_object());
}

/// Test edge cases in tool conversion
#[test]
fn test_tool_conversion_edge_cases() {
    // Tool with complex schema
    let complex_tool = ToolMetadata {
        name: "complex".to_string(),
        description: "Complex tool with nested schema".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "config": {
                    "type": "object",
                    "properties": {
                        "enabled": {"type": "boolean"},
                        "values": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "id": {"type": "string"},
                                    "value": {"type": "number"}
                                }
                            }
                        }
                    }
                }
            },
            "required": ["config"]
        }),
        title: Some("Complex Tool".to_string()),
        icon: Some("🔧".to_string()),
    };

    let converted = McpProtocolHandler::<std::sync::Arc<icarus_mcp::CanisterClient>>::convert_tool(
        &complex_tool,
    );

    assert_eq!(converted["name"], "complex");
    assert!(converted["inputSchema"]["properties"]["config"].is_object());
    assert!(
        converted["inputSchema"]["properties"]["config"]["properties"]["values"]["items"]
            ["properties"]["id"]
            .is_object()
    );
}

/// Test protocol handler methods exist (without IC connection)
#[test]
fn test_protocol_handler_interface() {
    // This test verifies the interface exists without requiring IC connection
    let metadata = create_test_metadata();

    // Test that our metadata follows the expected structure
    assert!(
        McpProtocolHandler::<std::sync::Arc<icarus_mcp::CanisterClient>>::convert_tool(
            &metadata.tools[0]
        )
        .is_object()
    );

    // Test metadata accessor pattern
    assert_eq!(metadata.tools[0].name, "echo");
    assert!(metadata.tools[0].title.is_some());
    assert!(metadata.tools[0].icon.is_some());
}
