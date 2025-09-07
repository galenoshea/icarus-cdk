//! Server trait and implementation for Icarus MCP servers

use crate::error::Result;
use crate::tool::IcarusTool;
use async_trait::async_trait;
use candid::Principal;
use rmcp::ServerHandler;

/// Version information for server upgrades
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Version {
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

/// Core trait for Icarus MCP servers that extends rmcp's ServerHandler
#[async_trait]
pub trait IcarusServer: ServerHandler + Send + Sync {
    /// Called when the canister is first initialized
    async fn on_canister_init(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called during canister upgrades
    async fn on_canister_upgrade(&mut self, _from_version: Version) -> Result<()> {
        Ok(())
    }

    /// Called before canister upgrade to save state
    async fn on_pre_upgrade(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called after canister upgrade to restore state
    async fn on_post_upgrade(&mut self) -> Result<()> {
        Ok(())
    }

    /// Get the canister principal
    fn canister_id(&self) -> Option<Principal> {
        None
    }

    /// Register a tool with the server
    fn register_tool(&mut self, tool: Box<dyn IcarusTool>) -> Result<()>;
}
