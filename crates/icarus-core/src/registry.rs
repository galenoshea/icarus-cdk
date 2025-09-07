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

    // Note: Full async tests with mock implementations would require
    // complex setup of rmcp types. The trait implementations are tested
    // through integration tests instead.
}
