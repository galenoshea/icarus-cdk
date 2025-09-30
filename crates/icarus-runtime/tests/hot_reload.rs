//! Integration tests for hot-reload capabilities.
//!
//! This tests that dynamic tool registration and management works correctly
//! for hot-reload scenarios without requiring recompilation.
//!
//! Note: These tests must run sequentially due to shared global registry state.

use icarus_core::{Tool, ToolId};
use icarus_runtime::ToolRegistry;
use serial_test::serial;
use std::sync::Arc;

/// Creates a test tool for hot-reload testing.
fn create_test_tool(name: &str, description: &str) -> Tool {
    Tool::new(
        name.to_string(),
        description.to_string(),
        Arc::new(serde_json::Map::new()),
    )
}

#[test]
#[serial]
fn test_dynamic_tool_registration() {
    // Clear any existing dynamic tools first
    let _ = ToolRegistry::clear_dynamic_tools();

    let tool = create_test_tool("dynamic_test_tool", "A dynamically registered test tool");

    // Register the tool dynamically
    assert!(ToolRegistry::register_dynamic_tool(tool).is_ok());

    // Verify it can be found
    let tool_id = ToolId::new("dynamic_test_tool").unwrap();
    let found_tool = ToolRegistry::find_by_id(&tool_id);

    assert!(found_tool.is_some());
    let found_tool = found_tool.unwrap();
    assert_eq!(found_tool.name.as_ref(), "dynamic_test_tool");
    assert_eq!(
        found_tool.description.as_deref(),
        Some("A dynamically registered test tool")
    );
}

#[test]
#[serial]
fn test_dynamic_tool_unregistration() {
    // Clear any existing dynamic tools first
    let _ = ToolRegistry::clear_dynamic_tools();

    let tool = create_test_tool("remove_test_tool", "Tool to be removed");

    // Register and then remove the tool
    assert!(ToolRegistry::register_dynamic_tool(tool).is_ok());

    let tool_id = ToolId::new("remove_test_tool").unwrap();
    let removed_tool = ToolRegistry::unregister_dynamic_tool(&tool_id);

    assert!(removed_tool.is_ok());
    let removed_tool = removed_tool.unwrap();
    assert!(removed_tool.is_some());
    assert_eq!(
        removed_tool.unwrap().description.as_deref(),
        Some("Tool to be removed")
    );

    // Verify it's no longer findable
    assert!(ToolRegistry::find_by_id(&tool_id).is_none());
}

#[test]
#[serial]
fn test_dynamic_tool_precedence() {
    // Clear any existing dynamic tools first
    let _ = ToolRegistry::clear_dynamic_tools();

    // Create a dynamic tool that could potentially conflict with a static tool
    let dynamic_tool = create_test_tool("precedence_test", "Dynamic tool with precedence");

    // Register the dynamic tool
    assert!(ToolRegistry::register_dynamic_tool(dynamic_tool).is_ok());

    // When looking up, dynamic tool should take precedence
    let tool_id = ToolId::new("precedence_test").unwrap();
    let found_tool = ToolRegistry::find_by_id(&tool_id);

    assert!(found_tool.is_some());
    let found_tool = found_tool.unwrap();
    assert_eq!(
        found_tool.description.as_deref(),
        Some("Dynamic tool with precedence")
    );
}

#[test]
#[serial]
fn test_list_dynamic_tools() {
    // Clear any existing dynamic tools first
    let _ = ToolRegistry::clear_dynamic_tools();

    // Register multiple dynamic tools
    let tool1 = create_test_tool("list_test_1", "First dynamic tool");
    let tool2 = create_test_tool("list_test_2", "Second dynamic tool");

    assert!(ToolRegistry::register_dynamic_tool(tool1).is_ok());
    assert!(ToolRegistry::register_dynamic_tool(tool2).is_ok());

    // List dynamic tools
    let dynamic_tools = ToolRegistry::list_dynamic_tools();

    assert_eq!(dynamic_tools.len(), 2);

    let tool_names: Vec<String> = dynamic_tools
        .iter()
        .map(|tool| tool.name.as_ref().to_string())
        .collect();

    assert!(tool_names.contains(&"list_test_1".to_string()));
    assert!(tool_names.contains(&"list_test_2".to_string()));
}

#[test]
#[serial]
fn test_list_all_includes_dynamic_tools() {
    // Clear any existing dynamic tools first
    let _ = ToolRegistry::clear_dynamic_tools();

    let initial_count = ToolRegistry::list_all().len();

    // Add a dynamic tool
    let dynamic_tool = create_test_tool("list_all_test", "Dynamic tool for list all test");
    assert!(ToolRegistry::register_dynamic_tool(dynamic_tool).is_ok());

    // Verify list_all includes the dynamic tool
    let all_tools = ToolRegistry::list_all();
    assert_eq!(all_tools.len(), initial_count + 1);

    let dynamic_tool = all_tools
        .iter()
        .find(|tool| tool.name.as_ref() == "list_all_test");

    assert!(dynamic_tool.is_some());
    assert_eq!(
        dynamic_tool.unwrap().description.as_deref(),
        Some("Dynamic tool for list all test")
    );
}

#[test]
#[serial]
fn test_build_index_includes_dynamic_tools() {
    // Clear any existing dynamic tools first
    let _ = ToolRegistry::clear_dynamic_tools();

    // Add a dynamic tool
    let dynamic_tool = create_test_tool("index_test", "Dynamic tool for index test");
    assert!(ToolRegistry::register_dynamic_tool(dynamic_tool).is_ok());

    // Build index and verify it includes the dynamic tool
    let index = ToolRegistry::build_index();

    let tool_id = ToolId::new("index_test").unwrap();
    assert!(index.contains_key(&tool_id));

    let indexed_tool = &index[&tool_id];
    assert_eq!(
        indexed_tool.description.as_deref(),
        Some("Dynamic tool for index test")
    );
}

#[test]
#[serial]
fn test_registry_stats_with_dynamic_tools() {
    // Clear any existing dynamic tools first
    let _ = ToolRegistry::clear_dynamic_tools();

    let initial_stats = ToolRegistry::stats();
    let initial_total = initial_stats.tool_count;
    let static_count = initial_stats.static_tool_count;

    // Add dynamic tools
    let tool1 = create_test_tool("stats_test_1", "First stats test tool");
    let tool2 = create_test_tool("stats_test_2", "Second stats test tool");

    assert!(ToolRegistry::register_dynamic_tool(tool1).is_ok());
    assert!(ToolRegistry::register_dynamic_tool(tool2).is_ok());

    // Check updated stats
    let updated_stats = ToolRegistry::stats();

    assert_eq!(updated_stats.tool_count, initial_total + 2);
    assert_eq!(updated_stats.static_tool_count, static_count); // Should remain the same
    assert_eq!(updated_stats.dynamic_tool_count, 2);

    // Verify summary includes the new information
    let summary = updated_stats.summary();
    assert!(summary.contains("static"));
    assert!(summary.contains("dynamic"));
}

#[test]
#[serial]
fn test_clear_dynamic_tools() {
    // Clear any existing dynamic tools first
    let _ = ToolRegistry::clear_dynamic_tools();

    // Add multiple dynamic tools
    let tool1 = create_test_tool("clear_test_1", "First clear test tool");
    let tool2 = create_test_tool("clear_test_2", "Second clear test tool");
    let tool3 = create_test_tool("clear_test_3", "Third clear test tool");

    assert!(ToolRegistry::register_dynamic_tool(tool1).is_ok());
    assert!(ToolRegistry::register_dynamic_tool(tool2).is_ok());
    assert!(ToolRegistry::register_dynamic_tool(tool3).is_ok());

    // Verify they're registered
    assert_eq!(ToolRegistry::list_dynamic_tools().len(), 3);

    // Clear all dynamic tools
    let cleared_count = ToolRegistry::clear_dynamic_tools();
    assert!(cleared_count.is_ok());
    assert_eq!(cleared_count.unwrap(), 3);

    // Verify they're all gone
    assert_eq!(ToolRegistry::list_dynamic_tools().len(), 0);

    // Verify none can be found
    let tool_id_1 = ToolId::new("clear_test_1").unwrap();
    let tool_id_2 = ToolId::new("clear_test_2").unwrap();
    let tool_id_3 = ToolId::new("clear_test_3").unwrap();

    assert!(ToolRegistry::find_by_id(&tool_id_1).is_none());
    assert!(ToolRegistry::find_by_id(&tool_id_2).is_none());
    assert!(ToolRegistry::find_by_id(&tool_id_3).is_none());
}

#[test]
#[serial]
fn test_hot_reload_workflow() {
    // Clear any existing dynamic tools first
    let _ = ToolRegistry::clear_dynamic_tools();

    // Simulate a complete hot-reload workflow
    let initial_stats = ToolRegistry::stats();

    // 1. Register initial dynamic tool
    let v1_tool = create_test_tool("hot_reload_tool", "Version 1.0 of the tool");
    assert!(ToolRegistry::register_dynamic_tool(v1_tool).is_ok());

    // 2. Verify it's available
    let tool_id = ToolId::new("hot_reload_tool").unwrap();
    let found_tool = ToolRegistry::find_by_id(&tool_id);
    assert!(found_tool.is_some());
    assert_eq!(
        found_tool.unwrap().description.as_deref(),
        Some("Version 1.0 of the tool")
    );

    // 3. Hot-reload with updated version (unregister old, register new)
    let removed_tool = ToolRegistry::unregister_dynamic_tool(&tool_id);
    assert!(removed_tool.is_ok() && removed_tool.unwrap().is_some());

    let v2_tool = create_test_tool(
        "hot_reload_tool",
        "Version 2.0 of the tool with new features",
    );
    assert!(ToolRegistry::register_dynamic_tool(v2_tool).is_ok());

    // 4. Verify the updated tool is now available
    let updated_tool = ToolRegistry::find_by_id(&tool_id);
    assert!(updated_tool.is_some());
    assert_eq!(
        updated_tool.unwrap().description.as_deref(),
        Some("Version 2.0 of the tool with new features")
    );

    // 5. Verify stats are consistent
    let final_stats = ToolRegistry::stats();
    assert_eq!(final_stats.tool_count, initial_stats.tool_count + 1);
    assert_eq!(final_stats.dynamic_tool_count, 1);
}

#[test]
#[serial]
fn test_concurrent_dynamic_tool_access() {
    use std::thread;

    // Clear any existing dynamic tools first
    let _ = ToolRegistry::clear_dynamic_tools();

    // Test thread-safety of dynamic tool operations
    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                let tool_name = format!("concurrent_tool_{}", i);
                let tool_desc = format!("Concurrent test tool {}", i);
                let tool = create_test_tool(&tool_name, &tool_desc);

                // Register tool
                assert!(ToolRegistry::register_dynamic_tool(tool).is_ok());

                // Verify it can be found
                let tool_id = ToolId::new(&tool_name).unwrap();
                let found_tool = ToolRegistry::find_by_id(&tool_id);
                assert!(found_tool.is_some());
                assert_eq!(
                    found_tool.unwrap().description.as_deref(),
                    Some(tool_desc.as_str())
                );

                // Remove tool
                let removed = ToolRegistry::unregister_dynamic_tool(&tool_id);
                assert!(removed.is_ok() && removed.unwrap().is_some());
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread should complete successfully");
    }

    // Verify no tools remain
    assert_eq!(ToolRegistry::list_dynamic_tools().len(), 0);
}
