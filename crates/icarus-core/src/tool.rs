//! Tool abstraction for Icarus MCP servers

use crate::error::Result;
use async_trait::async_trait;
use rmcp::model::Tool;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Information about a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
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
    fn to_rmcp_tool(&self) -> Tool;

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
