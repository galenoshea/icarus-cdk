//! Tool registry for dynamic registration

use crate::error::Result;
use crate::tool::{ComplexityLevel, IcarusTool, ToolCategory};
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

    /// Search tools by text query (name, description, tags)
    async fn search_tools(&self, query: &str) -> Result<Vec<String>>;

    /// Filter tools by category
    async fn filter_by_category(&self, category: &ToolCategory) -> Result<Vec<String>>;

    /// Filter tools by complexity level
    async fn filter_by_complexity(&self, complexity: &ComplexityLevel) -> Result<Vec<String>>;

    /// Get tools by tags (supports multiple tags with AND logic)
    async fn find_by_tags(&self, tags: &[String]) -> Result<Vec<String>>;

    /// Get tool suggestions based on usage patterns
    async fn suggest_tools(&self, context: &str, limit: usize) -> Result<Vec<String>>;

    /// Record tool usage for analytics and suggestions
    async fn record_usage(&mut self, tool_name: &str, execution_time_ms: u64) -> Result<()>;
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

    async fn search_tools(&self, query: &str) -> Result<Vec<String>> {
        let query_lower = query.to_lowercase();
        let mut matches = Vec::new();

        for (name, tool) in &self.tools {
            let info = tool.info();

            // Search in name, description, and tags
            let matches_name = name.to_lowercase().contains(&query_lower);
            let matches_desc = info.description.to_lowercase().contains(&query_lower);
            let matches_tags = info
                .tags
                .iter()
                .any(|tag| tag.to_lowercase().contains(&query_lower));

            if matches_name || matches_desc || matches_tags {
                matches.push(name.clone());
            }
        }

        // Sort by usage count (descending) then by name
        matches.sort_by(|a, b| {
            let tool_a = &self.tools[a];
            let tool_b = &self.tools[b];
            let usage_a = tool_a.info().usage_count;
            let usage_b = tool_b.info().usage_count;

            usage_b.cmp(&usage_a).then_with(|| a.cmp(b))
        });

        Ok(matches)
    }

    async fn filter_by_category(&self, category: &ToolCategory) -> Result<Vec<String>> {
        let matches: Vec<String> = self
            .tools
            .iter()
            .filter_map(|(name, tool)| {
                if tool.info().category.as_ref() == Some(category) {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(matches)
    }

    async fn filter_by_complexity(&self, complexity: &ComplexityLevel) -> Result<Vec<String>> {
        let matches: Vec<String> = self
            .tools
            .iter()
            .filter_map(|(name, tool)| {
                if tool.info().complexity.as_ref() == Some(complexity) {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(matches)
    }

    async fn find_by_tags(&self, tags: &[String]) -> Result<Vec<String>> {
        let search_tags: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();

        let matches: Vec<String> = self
            .tools
            .iter()
            .filter_map(|(name, tool)| {
                let tool_tags: Vec<String> =
                    tool.info().tags.iter().map(|t| t.to_lowercase()).collect();

                // Check if ALL search tags are present (AND logic)
                let has_all_tags = search_tags
                    .iter()
                    .all(|search_tag| tool_tags.contains(search_tag));

                if has_all_tags {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(matches)
    }

    async fn suggest_tools(&self, context: &str, limit: usize) -> Result<Vec<String>> {
        let context_lower = context.to_lowercase();
        let mut scored_tools = Vec::new();

        for (name, tool) in &self.tools {
            let info = tool.info();
            let mut score = 0f64;

            // Base score from usage count (normalized)
            score += (info.usage_count as f64 / 100.0).min(10.0);

            // Bonus for context matching
            if info.description.to_lowercase().contains(&context_lower) {
                score += 5.0;
            }
            if info
                .tags
                .iter()
                .any(|tag| context_lower.contains(&tag.to_lowercase()))
            {
                score += 3.0;
            }
            if name.to_lowercase().contains(&context_lower) {
                score += 4.0;
            }

            // Complexity penalty for complex tools (prefer simpler ones for suggestions)
            if let Some(ComplexityLevel::Complex) = info.complexity {
                score -= 2.0;
            }

            scored_tools.push((name.clone(), score));
        }

        // Sort by score (descending) and take top N
        scored_tools.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored_tools
            .into_iter()
            .take(limit)
            .map(|(name, _)| name)
            .collect())
    }

    async fn record_usage(&mut self, tool_name: &str, _execution_time_ms: u64) -> Result<()> {
        if let Some(_tool) = self.tools.get_mut(tool_name) {
            // Note: This is a conceptual implementation - in practice, we'd need mutable access
            // to the tool's info, which would require a different architecture
            // For now, this represents the intended functionality

            // In a real implementation, we might:
            // 1. Store usage data separately from tools
            // 2. Use interior mutability (RefCell/Mutex)
            // 3. Implement a usage tracking service

            // Placeholder implementation showing the concept
            Ok(())
        } else {
            Err(crate::error::IcarusError::Tool(
                crate::error::ToolError::operation_failed("Tool not found for usage recording"),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{ComplexityLevel, IconInfo, StorageRequirements, ToolCategory, ToolInfo};
    use serde_json::json;

    // Mock tool implementation for testing
    struct MockTool {
        name: String,
        description: String,
        should_fail: bool,
        category: Option<ToolCategory>,
        tags: Vec<String>,
        complexity: Option<ComplexityLevel>,
        usage_count: u64,
    }

    impl MockTool {
        fn new(name: &str, description: &str) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
                should_fail: false,
                category: None,
                tags: Vec::new(),
                complexity: None,
                usage_count: 0,
            }
        }

        fn new_failing(name: &str, description: &str) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
                should_fail: true,
                category: None,
                tags: Vec::new(),
                complexity: None,
                usage_count: 0,
            }
        }

        fn with_category(mut self, category: ToolCategory) -> Self {
            self.category = Some(category);
            self
        }

        fn with_tags(mut self, tags: Vec<&str>) -> Self {
            self.tags = tags.into_iter().map(|s| s.to_string()).collect();
            self
        }

        fn with_complexity(mut self, complexity: ComplexityLevel) -> Self {
            self.complexity = Some(complexity);
            self
        }

        fn with_usage_count(mut self, count: u64) -> Self {
            self.usage_count = count;
            self
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
                category: self.category.clone(),
                tags: self.tags.clone(),
                complexity: self.complexity.clone(),
                usage_count: self.usage_count,
                avg_execution_time: None,
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

    #[tokio::test]
    async fn test_semantic_search_by_text() {
        let mut registry = DefaultToolRegistry::new();

        // Create tools with different properties for search testing
        let data_tool = Box::new(
            MockTool::new("data_processor", "Process data files")
                .with_category(ToolCategory::Data)
                .with_tags(vec!["processing", "files"]),
        );

        let ui_tool = Box::new(
            MockTool::new("button_creator", "Create UI buttons")
                .with_category(ToolCategory::Interface)
                .with_tags(vec!["ui", "buttons"]),
        );

        registry.register_tool(data_tool).await.unwrap();
        registry.register_tool(ui_tool).await.unwrap();

        // Search by name
        let name_results = registry.search_tools("data").await.unwrap();
        assert_eq!(name_results.len(), 1);
        assert_eq!(name_results[0], "data_processor");

        // Search by description
        let desc_results = registry.search_tools("buttons").await.unwrap();
        assert_eq!(desc_results.len(), 1);
        assert_eq!(desc_results[0], "button_creator");

        // Search by tag
        let tag_results = registry.search_tools("ui").await.unwrap();
        assert_eq!(tag_results.len(), 1);
        assert_eq!(tag_results[0], "button_creator");

        // Search with no matches
        let no_matches = registry.search_tools("nonexistent").await.unwrap();
        assert_eq!(no_matches.len(), 0);
    }

    #[tokio::test]
    async fn test_filter_by_category() {
        let mut registry = DefaultToolRegistry::new();

        let data_tool1 = Box::new(
            MockTool::new("csv_reader", "Read CSV files").with_category(ToolCategory::Data),
        );
        let data_tool2 = Box::new(
            MockTool::new("json_parser", "Parse JSON data").with_category(ToolCategory::Data),
        );
        let ui_tool = Box::new(
            MockTool::new("modal_creator", "Create modals").with_category(ToolCategory::Interface),
        );

        registry.register_tool(data_tool1).await.unwrap();
        registry.register_tool(data_tool2).await.unwrap();
        registry.register_tool(ui_tool).await.unwrap();

        // Filter by Data category
        let data_tools = registry
            .filter_by_category(&ToolCategory::Data)
            .await
            .unwrap();
        assert_eq!(data_tools.len(), 2);
        assert!(data_tools.contains(&"csv_reader".to_string()));
        assert!(data_tools.contains(&"json_parser".to_string()));

        // Filter by Interface category
        let ui_tools = registry
            .filter_by_category(&ToolCategory::Interface)
            .await
            .unwrap();
        assert_eq!(ui_tools.len(), 1);
        assert_eq!(ui_tools[0], "modal_creator");

        // Filter by category with no matches
        let system_tools = registry
            .filter_by_category(&ToolCategory::System)
            .await
            .unwrap();
        assert_eq!(system_tools.len(), 0);
    }

    #[tokio::test]
    async fn test_filter_by_complexity() {
        let mut registry = DefaultToolRegistry::new();

        let simple_tool =
            Box::new(MockTool::new("echo", "Echo input").with_complexity(ComplexityLevel::Simple));
        let complex_tool = Box::new(
            MockTool::new("ml_trainer", "Train ML models")
                .with_complexity(ComplexityLevel::Complex),
        );

        registry.register_tool(simple_tool).await.unwrap();
        registry.register_tool(complex_tool).await.unwrap();

        let simple_tools = registry
            .filter_by_complexity(&ComplexityLevel::Simple)
            .await
            .unwrap();
        assert_eq!(simple_tools.len(), 1);
        assert_eq!(simple_tools[0], "echo");

        let complex_tools = registry
            .filter_by_complexity(&ComplexityLevel::Complex)
            .await
            .unwrap();
        assert_eq!(complex_tools.len(), 1);
        assert_eq!(complex_tools[0], "ml_trainer");
    }

    #[tokio::test]
    async fn test_find_by_tags() {
        let mut registry = DefaultToolRegistry::new();

        let tool1 = Box::new(
            MockTool::new("web_scraper", "Scrape web pages")
                .with_tags(vec!["web", "scraping", "data"]),
        );
        let tool2 = Box::new(
            MockTool::new("api_client", "Make API calls").with_tags(vec!["web", "api", "http"]),
        );
        let tool3 =
            Box::new(MockTool::new("file_reader", "Read files").with_tags(vec!["files", "io"]));

        registry.register_tool(tool1).await.unwrap();
        registry.register_tool(tool2).await.unwrap();
        registry.register_tool(tool3).await.unwrap();

        // Find tools with "web" tag
        let web_tools = registry.find_by_tags(&["web".to_string()]).await.unwrap();
        assert_eq!(web_tools.len(), 2);
        assert!(web_tools.contains(&"web_scraper".to_string()));
        assert!(web_tools.contains(&"api_client".to_string()));

        // Find tools with both "web" AND "data" tags (AND logic)
        let web_data_tools = registry
            .find_by_tags(&["web".to_string(), "data".to_string()])
            .await
            .unwrap();
        assert_eq!(web_data_tools.len(), 1);
        assert_eq!(web_data_tools[0], "web_scraper");

        // Find tools with non-existent tag
        let no_tools = registry
            .find_by_tags(&["nonexistent".to_string()])
            .await
            .unwrap();
        assert_eq!(no_tools.len(), 0);
    }

    #[tokio::test]
    async fn test_suggest_tools() {
        let mut registry = DefaultToolRegistry::new();

        // Create tools with different usage patterns
        let popular_tool = Box::new(
            MockTool::new("popular_tool", "Frequently used tool")
                .with_usage_count(100)
                .with_tags(vec!["data", "popular"]),
        );
        let simple_tool = Box::new(
            MockTool::new("simple_tool", "Simple data processor")
                .with_complexity(ComplexityLevel::Simple)
                .with_usage_count(50)
                .with_tags(vec!["data", "simple"]),
        );
        let complex_tool = Box::new(
            MockTool::new("complex_tool", "Complex data analysis")
                .with_complexity(ComplexityLevel::Complex)
                .with_usage_count(75)
                .with_tags(vec!["data", "analysis"]),
        );

        registry.register_tool(popular_tool).await.unwrap();
        registry.register_tool(simple_tool).await.unwrap();
        registry.register_tool(complex_tool).await.unwrap();

        // Test suggestions for "data" context
        let suggestions = registry.suggest_tools("data processing", 2).await.unwrap();
        assert_eq!(suggestions.len(), 2);

        // Popular tool should be suggested first due to high usage
        assert_eq!(suggestions[0], "popular_tool");

        // Simple tool should be preferred over complex tool despite lower usage
        assert_eq!(suggestions[1], "simple_tool");
    }

    #[tokio::test]
    async fn test_record_usage_placeholder() {
        let mut registry = DefaultToolRegistry::new();
        let tool = Box::new(MockTool::new("test_tool", "Test tool"));

        registry.register_tool(tool).await.unwrap();

        // Test usage recording (placeholder implementation)
        let result = registry.record_usage("test_tool", 250).await;
        assert!(result.is_ok());

        // Test recording usage for non-existent tool
        let error_result = registry.record_usage("nonexistent", 100).await;
        assert!(error_result.is_err());
    }

    #[tokio::test]
    async fn test_semantic_fields_in_tool_info() {
        let tool = MockTool::new("semantic_test", "Test semantic features")
            .with_category(ToolCategory::Analysis)
            .with_tags(vec!["test", "semantic", "analysis"])
            .with_complexity(ComplexityLevel::Moderate)
            .with_usage_count(42);

        let info = tool.info();

        assert_eq!(info.category, Some(ToolCategory::Analysis));
        assert_eq!(info.tags, vec!["test", "semantic", "analysis"]);
        assert_eq!(info.complexity, Some(ComplexityLevel::Moderate));
        assert_eq!(info.usage_count, 42);
        assert_eq!(info.avg_execution_time, None);
    }
}
