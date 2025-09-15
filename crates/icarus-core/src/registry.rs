//! Tool registry for dynamic registration

use crate::error::Result;
use crate::tool::IcarusTool;
use async_trait::async_trait;
use std::collections::HashMap;

/// Registry for dynamically managing tools
#[async_trait]
pub trait IcarusToolRegistry: Send + Sync {
    /// Register a new tool
    async fn register_tool(&mut self, tool: Box<dyn IcarusTool>) -> Result<()>;

    /// Unregister a tool by name
    async fn unregister_tool(&mut self, name: &str) -> Result<()>;

    /// Get a tool by name
    async fn get_tool(&self, name: &str) -> Result<Option<&dyn IcarusTool>>;

    /// List all registered tools
    async fn list_tools(&self) -> Result<Vec<String>>;
}

/// Default implementation of tool registry
pub struct DefaultToolRegistry {
    tools: HashMap<String, Box<dyn IcarusTool>>,
}

impl DefaultToolRegistry {
    /// Create a new default registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }
}

impl Default for DefaultToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IcarusToolRegistry for DefaultToolRegistry {
    async fn register_tool(&mut self, tool: Box<dyn IcarusTool>) -> Result<()> {
        let info = tool.info();
        self.tools.insert(info.name.clone(), tool);
        Ok(())
    }

    async fn unregister_tool(&mut self, name: &str) -> Result<()> {
        self.tools.remove(name);
        Ok(())
    }

    async fn get_tool(&self, name: &str) -> Result<Option<&dyn IcarusTool>> {
        Ok(self.tools.get(name).map(|t| t.as_ref()))
    }

    async fn list_tools(&self) -> Result<Vec<String>> {
        Ok(self.tools.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{IconInfo, StorageRequirements, ToolInfo};
    use serde_json::json;

    // Mock tool implementation for testing
    struct MockTool {
        name: String,
        description: String,
        should_fail: bool,
    }

    impl MockTool {
        fn new(name: &str, description: &str) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
                should_fail: false,
            }
        }

        fn new_failing(name: &str, description: &str) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl IcarusTool for MockTool {
        fn info(&self) -> ToolInfo {
            ToolInfo {
                name: self.name.clone(),
                description: self.description.clone(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "input": {"type": "string"}
                    }
                }),
                title: Some(format!("Mock: {}", self.name)),
                icons: Some(vec![IconInfo {
                    name: "test-icon".to_string(),
                    data: Some("data:image/svg+xml,<svg></svg>".to_string()),
                }]),
            }
        }

        fn requires_stable_storage(&self) -> bool {
            self.name.contains("storage")
        }

        fn storage_requirements(&self) -> StorageRequirements {
            if self.requires_stable_storage() {
                StorageRequirements {
                    stable_memory: Some(1024 * 1024), // 1MB
                    requires_persistence: true,
                }
            } else {
                StorageRequirements::default()
            }
        }

        async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value> {
            if self.should_fail {
                return Err(crate::error::IcarusError::Tool(
                    crate::error::ToolError::operation_failed("Mock tool failure"),
                ));
            }

            Ok(json!({
                "tool": self.name,
                "input": args,
                "result": "success"
            }))
        }
    }

    #[test]
    fn test_default_registry() {
        let registry = DefaultToolRegistry::default();
        assert_eq!(registry.tools.len(), 0);
    }

    #[test]
    fn test_new_registry() {
        let registry = DefaultToolRegistry::new();
        assert_eq!(registry.tools.len(), 0);
    }

    #[tokio::test]
    async fn test_register_tool() {
        let mut registry = DefaultToolRegistry::new();
        let tool = Box::new(MockTool::new("test_tool", "A test tool"));

        let result = registry.register_tool(tool).await;
        assert!(result.is_ok());
        assert_eq!(registry.tools.len(), 1);
    }

    #[tokio::test]
    async fn test_register_multiple_tools() {
        let mut registry = DefaultToolRegistry::new();

        let tool1 = Box::new(MockTool::new("tool1", "First tool"));
        let tool2 = Box::new(MockTool::new("tool2", "Second tool"));
        let tool3 = Box::new(MockTool::new("tool3", "Third tool"));

        registry.register_tool(tool1).await.unwrap();
        registry.register_tool(tool2).await.unwrap();
        registry.register_tool(tool3).await.unwrap();

        assert_eq!(registry.tools.len(), 3);
    }

    #[tokio::test]
    async fn test_register_duplicate_tool_overwrites() {
        let mut registry = DefaultToolRegistry::new();

        let tool1 = Box::new(MockTool::new("duplicate", "First version"));
        let tool2 = Box::new(MockTool::new("duplicate", "Second version"));

        registry.register_tool(tool1).await.unwrap();
        assert_eq!(registry.tools.len(), 1);

        registry.register_tool(tool2).await.unwrap();
        assert_eq!(registry.tools.len(), 1); // Still 1, but overwritten

        let retrieved = registry.get_tool("duplicate").await.unwrap().unwrap();
        assert_eq!(retrieved.info().description, "Second version");
    }

    #[tokio::test]
    async fn test_unregister_tool() {
        let mut registry = DefaultToolRegistry::new();
        let tool = Box::new(MockTool::new("removable", "Tool to remove"));

        registry.register_tool(tool).await.unwrap();
        assert_eq!(registry.tools.len(), 1);

        let result = registry.unregister_tool("removable").await;
        assert!(result.is_ok());
        assert_eq!(registry.tools.len(), 0);
    }

    #[tokio::test]
    async fn test_unregister_nonexistent_tool() {
        let mut registry = DefaultToolRegistry::new();

        // Should not fail even if tool doesn't exist
        let result = registry.unregister_tool("nonexistent").await;
        assert!(result.is_ok());
        assert_eq!(registry.tools.len(), 0);
    }

    #[tokio::test]
    async fn test_get_tool() {
        let mut registry = DefaultToolRegistry::new();
        let tool = Box::new(MockTool::new("getter", "Tool for getting"));

        registry.register_tool(tool).await.unwrap();

        let retrieved = registry.get_tool("getter").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().info().name, "getter");
    }

    #[tokio::test]
    async fn test_get_nonexistent_tool() {
        let registry = DefaultToolRegistry::new();

        let retrieved = registry.get_tool("nonexistent").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_tools_empty() {
        let registry = DefaultToolRegistry::new();

        let tools = registry.list_tools().await.unwrap();
        assert_eq!(tools.len(), 0);
    }

    #[tokio::test]
    async fn test_list_tools_populated() {
        let mut registry = DefaultToolRegistry::new();

        let tool1 = Box::new(MockTool::new("alpha", "Alpha tool"));
        let tool2 = Box::new(MockTool::new("beta", "Beta tool"));
        let tool3 = Box::new(MockTool::new("gamma", "Gamma tool"));

        registry.register_tool(tool1).await.unwrap();
        registry.register_tool(tool2).await.unwrap();
        registry.register_tool(tool3).await.unwrap();

        let mut tools = registry.list_tools().await.unwrap();
        tools.sort(); // HashMap order is not guaranteed

        assert_eq!(tools, vec!["alpha", "beta", "gamma"]);
    }

    #[tokio::test]
    async fn test_registry_lifecycle() {
        let mut registry = DefaultToolRegistry::new();

        // Start empty
        let tools = registry.list_tools().await.unwrap();
        assert_eq!(tools.len(), 0);

        // Add some tools
        registry
            .register_tool(Box::new(MockTool::new("tool1", "First")))
            .await
            .unwrap();
        registry
            .register_tool(Box::new(MockTool::new("tool2", "Second")))
            .await
            .unwrap();

        assert_eq!(registry.list_tools().await.unwrap().len(), 2);

        // Get a tool
        let tool = registry.get_tool("tool1").await.unwrap().unwrap();
        assert_eq!(tool.info().description, "First");

        // Remove a tool
        registry.unregister_tool("tool1").await.unwrap();
        assert_eq!(registry.list_tools().await.unwrap().len(), 1);

        // Verify the right tool remains
        let remaining = registry.get_tool("tool2").await.unwrap().unwrap();
        assert_eq!(remaining.info().description, "Second");

        // Clear all
        registry.unregister_tool("tool2").await.unwrap();
        assert_eq!(registry.list_tools().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_tool_info_preservation() {
        let mut registry = DefaultToolRegistry::new();
        let tool = Box::new(MockTool::new("info_test", "Tool with rich info"));

        registry.register_tool(tool).await.unwrap();

        let retrieved = registry.get_tool("info_test").await.unwrap().unwrap();
        let info = retrieved.info();

        assert_eq!(info.name, "info_test");
        assert_eq!(info.description, "Tool with rich info");
        assert!(info.title.is_some());
        assert_eq!(info.title.unwrap(), "Mock: info_test");
        assert!(info.icons.is_some());
        assert_eq!(info.icons.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_storage_requirements_preserved() {
        let mut registry = DefaultToolRegistry::new();

        // Tool that requires storage
        let storage_tool = Box::new(MockTool::new("storage_tool", "Needs storage"));
        // Tool that doesn't require storage
        let simple_tool = Box::new(MockTool::new("simple", "No storage needed"));

        registry.register_tool(storage_tool).await.unwrap();
        registry.register_tool(simple_tool).await.unwrap();

        let storage_retrieved = registry.get_tool("storage_tool").await.unwrap().unwrap();
        let simple_retrieved = registry.get_tool("simple").await.unwrap().unwrap();

        assert!(storage_retrieved.requires_stable_storage());
        assert!(!simple_retrieved.requires_stable_storage());

        let storage_reqs = storage_retrieved.storage_requirements();
        let simple_reqs = simple_retrieved.storage_requirements();

        assert!(storage_reqs.requires_persistence);
        assert!(storage_reqs.stable_memory.is_some());
        assert_eq!(storage_reqs.stable_memory.unwrap(), 1024 * 1024);

        assert!(!simple_reqs.requires_persistence);
        assert!(simple_reqs.stable_memory.is_none());
    }

    #[tokio::test]
    async fn test_rmcp_tool_conversion() {
        let mut registry = DefaultToolRegistry::new();
        let tool = Box::new(MockTool::new("rmcp_test", "RMCP conversion test"));

        registry.register_tool(tool).await.unwrap();

        let retrieved = registry.get_tool("rmcp_test").await.unwrap().unwrap();
        let rmcp_tool = retrieved.to_rmcp_tool();

        assert_eq!(rmcp_tool.name, "rmcp_test");
        assert!(rmcp_tool.description.is_some());
        assert_eq!(rmcp_tool.description.unwrap(), "RMCP conversion test");
        assert!(!rmcp_tool.input_schema.is_empty());
    }
}
