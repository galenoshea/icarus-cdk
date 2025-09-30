//! Integration tests for linkme distributed slice functionality.
//!
//! This tests that the linkme-based tool registry correctly discovers and
//! manages tools registered at compile time through distributed slices.

use icarus_core::{Tool, ToolId};
use icarus_runtime::{ToolRegistry, TOOL_REGISTRY};
use std::sync::Arc;

/// Test helper function that creates a mock tool for linkme integration testing.
fn create_test_tool() -> Tool {
    Tool::new(
        "test_linkme_tool".to_string(),
        "A test tool for linkme integration".to_string(),
        Arc::new(serde_json::Map::new()),
    )
}

/// Manually register a test tool using linkme distributed slice syntax.
///
/// This simulates what the #[tool] macro generates for actual tools.
#[linkme::distributed_slice(TOOL_REGISTRY)]
static TEST_TOOL_REGISTRY: fn() -> Tool = create_test_tool;

#[test]
fn test_linkme_tool_discovery() {
    // Test that the distributed slice mechanism works
    let tools = ToolRegistry::list_all();

    // Verify that our test tool is discovered
    let test_tool = tools
        .iter()
        .find(|tool| tool.name.as_ref() == "test_linkme_tool");

    assert!(
        test_tool.is_some(),
        "Test tool should be discovered through linkme"
    );

    let test_tool = test_tool.unwrap();
    assert_eq!(
        test_tool.description.as_deref(),
        Some("A test tool for linkme integration")
    );
    // RMCP Tool has input_schema instead of parameters field
}

#[test]
fn test_linkme_tool_lookup() {
    // Test finding specific tools by ID
    let tool_id = ToolId::new("test_linkme_tool").unwrap();
    let found_tool = ToolRegistry::find_by_id(&tool_id);

    assert!(found_tool.is_some(), "Tool should be findable by ID");

    let found_tool = found_tool.unwrap();
    assert_eq!(found_tool.name.as_ref(), tool_id.as_str());
    assert_eq!(
        found_tool.description.as_deref(),
        Some("A test tool for linkme integration")
    );
}

#[test]
fn test_linkme_registry_stats() {
    // Test that registry statistics include linkme-registered tools
    let stats = ToolRegistry::stats();

    // Should have at least our test tool
    assert!(
        stats.tool_count >= 1,
        "Registry should contain at least the test tool"
    );
    assert!(
        stats.validation_status,
        "Registry should validate successfully"
    );
    assert!(
        !stats.has_duplicates,
        "Registry should not have duplicate tools"
    );
}

#[test]
fn test_linkme_registry_index() {
    // Test that the registry index includes linkme-registered tools
    let index = ToolRegistry::build_index();

    let tool_id = ToolId::new("test_linkme_tool").unwrap();
    assert!(
        index.contains_key(&tool_id),
        "Index should contain the test tool"
    );

    let indexed_tool = &index[&tool_id];
    assert_eq!(
        indexed_tool.description.as_deref(),
        Some("A test tool for linkme integration")
    );
}

#[test]
fn test_linkme_direct_slice_access() {
    // Test direct access to the distributed slice
    let tool_functions: &[fn() -> Tool] = &TOOL_REGISTRY;

    // Should have at least our test tool function
    assert!(
        !tool_functions.is_empty(),
        "Tool registry slice should not be empty"
    );

    // Verify that calling the functions produces valid tools
    let tools: Vec<Tool> = tool_functions.iter().map(|f| f()).collect();

    let test_tool = tools
        .iter()
        .find(|tool| tool.name.as_ref() == "test_linkme_tool");

    assert!(
        test_tool.is_some(),
        "Test tool should be accessible through direct slice access"
    );
}

#[test]
fn test_linkme_zero_cost_abstraction() {
    // Test that linkme registration has zero runtime cost
    use std::time::Instant;

    let start = Instant::now();

    // Perform multiple registry operations
    for _ in 0..1000 {
        let _tools = ToolRegistry::list_all();
    }

    let duration = start.elapsed();

    // Should complete very quickly (less than 10ms total for 1000 operations)
    assert!(
        duration.as_millis() < 10,
        "Registry operations should be near zero-cost"
    );
}
