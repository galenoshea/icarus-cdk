//! Server lifecycle management traits

use crate::error::Result;
use async_trait::async_trait;

/// Lifecycle events for MCP servers
#[async_trait]
pub trait IcarusServerLifecycle: Send + Sync {
    /// Called when the server is initialized
    async fn on_initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called before the server is upgraded
    async fn on_pre_upgrade(&self) -> Result<()> {
        Ok(())
    }

    /// Called after the server is upgraded
    async fn on_post_upgrade(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when the server is about to be stopped
    async fn on_stop(&self) -> Result<()> {
        Ok(())
    }

    /// Called periodically for maintenance tasks
    async fn on_heartbeat(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Configuration for server lifecycle
#[derive(Debug, Clone)]
pub struct LifecycleConfig {
    /// Heartbeat interval in seconds (0 to disable)
    pub heartbeat_interval: u64,
    /// Whether to enable automatic state snapshots
    pub auto_snapshot: bool,
    /// Maximum number of snapshots to keep
    pub max_snapshots: u32,
}
