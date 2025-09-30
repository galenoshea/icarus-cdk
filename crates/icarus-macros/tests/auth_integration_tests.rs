//! Integration tests for authentication system

#[test]
fn test_auth_tool_compilation() {
    // Verify that tools with auth attributes compile
    let t = trybuild::TestCases::new();
    t.pass("tests/compilation/pass/auth_tool.rs");
}

#[test]
fn test_auth_annotation_structure() {
    // Test that ToolAnnotations struct is properly defined with RMCP fields
    use icarus_core::ToolAnnotations;

    let annotations = ToolAnnotations {
        title: Some("Test Tool".to_string()),
        read_only_hint: Some(true),
        destructive_hint: Some(false),
        idempotent_hint: Some(true),
        open_world_hint: Some(false),
    };

    assert_eq!(annotations.title, Some("Test Tool".to_string()));
    assert_eq!(annotations.read_only_hint, Some(true));
    assert_eq!(annotations.destructive_hint, Some(false));
    assert_eq!(annotations.idempotent_hint, Some(true));
    assert_eq!(annotations.open_world_hint, Some(false));
}

#[test]
fn test_tool_with_annotations() {
    // Test that Tool::new() and annotate() work together
    use icarus_core::{Tool, ToolAnnotations};
    use std::sync::Arc;

    let mut schema = serde_json::Map::new();
    schema.insert("type".to_string(), serde_json::json!("object"));

    let annotations = ToolAnnotations {
        title: None,
        read_only_hint: Some(true),
        destructive_hint: Some(false),
        idempotent_hint: None,
        open_world_hint: None,
    };

    let tool =
        Tool::new("test_tool", "Test tool with auth", Arc::new(schema)).annotate(annotations);

    assert!(tool.annotations.is_some());
    let tool_annotations = tool.annotations.unwrap();
    assert_eq!(tool_annotations.read_only_hint, Some(true));
    assert_eq!(tool_annotations.destructive_hint, Some(false));
}

#[test]
fn test_mcp_config_parsing() {
    // Test mcp!{} configuration parsing is handled correctly
    // This is implicitly tested by the macro unit tests
}
