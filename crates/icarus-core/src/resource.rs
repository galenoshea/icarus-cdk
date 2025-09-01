//! Resource abstraction for Icarus MCP servers

use crate::error::Result;
use crate::tool::StorageRequirements;
use async_trait::async_trait;
use rmcp::model::Resource;
use serde::{Deserialize, Serialize};

/// Information about a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
}

/// Core trait for Icarus resources that integrate with rmcp
#[async_trait]
pub trait IcarusResource: Send + Sync {
    /// Get resource information
    fn info(&self) -> ResourceInfo;

    /// Convert to rmcp resource representation
    fn to_rmcp_resource(&self) -> Resource;

    /// Get storage requirements for capacity planning
    fn storage_requirements(&self) -> StorageRequirements {
        StorageRequirements::default()
    }

    /// Read the resource content
    async fn read(&self) -> Result<Vec<u8>>;

    /// List available resources
    async fn list(&self) -> Result<Vec<ResourceInfo>>;
}
