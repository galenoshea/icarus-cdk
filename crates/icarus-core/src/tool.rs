//! Tool abstraction for Icarus MCP servers

use crate::error::Result;
use async_trait::async_trait;
use rmcp::model::Tool;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Icon information for tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconInfo {
    /// Icon name or identifier
    pub name: String,
    /// Optional icon data (base64 encoded or URL)
    pub data: Option<String>,
}

/// Semantic category for tools
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolCategory {
    /// Data manipulation and storage
    Data,
    /// User interface and interaction
    Interface,
    /// Analysis and computation
    Analysis,
    /// Communication and external services
    Communication,
    /// System administration and monitoring
    System,
    /// Authentication and security
    Security,
    /// Utility and helper functions
    Utility,
    /// Custom category
    Custom(String),
}

/// Usage complexity indicator for tools
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComplexityLevel {
    /// Simple, single-parameter operations
    Simple,
    /// Moderate complexity with multiple parameters
    Moderate,
    /// Complex operations requiring careful configuration
    Complex,
}

/// Information about a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    /// Optional display title for better UX
    pub title: Option<String>,
    /// Optional icons for tool display
    pub icons: Option<Vec<IconInfo>>,
    /// Semantic category for discovery
    pub category: Option<ToolCategory>,
    /// Searchable tags for better discovery
    pub tags: Vec<String>,
    /// Complexity indicator for user guidance
    pub complexity: Option<ComplexityLevel>,
    /// Usage frequency counter (for suggestions)
    pub usage_count: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time: Option<u64>,
}

/// Storage requirements for a tool
#[derive(Debug, Clone, Default)]
pub struct StorageRequirements {
    /// Estimated stable memory usage in bytes
    pub stable_memory: Option<u64>,
    /// Whether the tool needs persistent state
    pub requires_persistence: bool,
}

/// Core trait for Icarus tools that integrate with rmcp
#[async_trait]
pub trait IcarusTool: Send + Sync {
    /// Get tool information
    fn info(&self) -> ToolInfo;

    /// Convert to rmcp tool representation
    fn to_rmcp_tool(&self) -> Tool {
        use std::borrow::Cow;
        use std::sync::Arc;

        let info = self.info();

        // Convert input_schema Value to JsonObject
        let schema = if let serde_json::Value::Object(obj) = &info.input_schema {
            Arc::new(obj.clone())
        } else {
            Arc::new(serde_json::Map::new())
        };

        Tool {
            name: Cow::Owned(info.name),
            description: Some(Cow::Owned(info.description)),
            input_schema: schema,
            output_schema: None,
            annotations: None,
            title: info.title,
            icons: None, // Icons will be handled later when we support them fully
        }
    }

    /// Check if the tool requires stable storage
    fn requires_stable_storage(&self) -> bool {
        false
    }

    /// Get storage requirements for capacity planning
    fn storage_requirements(&self) -> StorageRequirements {
        StorageRequirements::default()
    }

    /// Execute the tool with given arguments
    async fn execute(&self, args: Value) -> Result<Value>;
}
