//! Agent connection pool for improved performance
//!
//! Maintains a pool of reusable Agent connections to avoid expensive setup/teardown

use anyhow::Result;
use ic_agent::Agent;
use rustc_hash::FxHashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::config::McpConfig;

/// Global agent pool instance
static AGENT_POOL: OnceLock<Arc<AgentPool>> = OnceLock::new();

/// Connection pool for Agent instances
///
/// Maintains a pool of Agent connections keyed by (url, identity_name) to avoid
/// expensive connection setup for repeated calls to the same canister.
#[derive(Debug)]
pub struct AgentPool {
    /// Pool of agents keyed by (ic_url, identity_name)
    agents: RwLock<FxHashMap<(String, String), Arc<Agent>>>,
}

impl AgentPool {
    /// Create a new empty agent pool
    fn new() -> Self {
        Self {
            agents: RwLock::new(FxHashMap::default()),
        }
    }

    /// Get the global agent pool instance
    pub fn global() -> Arc<Self> {
        AGENT_POOL
            .get_or_init(|| Arc::new(AgentPool::new()))
            .clone()
    }

    /// Get or create an agent for the given configuration
    ///
    /// This method maintains a cache of agents to avoid expensive setup.
    /// Agents are keyed by IC URL and identity name.
    pub async fn get_or_create_agent(&self, config: &McpConfig) -> Result<Arc<Agent>> {
        let identity_name = self.get_current_identity_name().await?;
        let key = (config.ic_url.clone(), identity_name.clone());

        // First try to get existing agent (read lock)
        {
            let agents = self.agents.read().await;
            if let Some(agent) = agents.get(&key) {
                debug!(
                    "Reusing existing agent for {} with identity {}",
                    config.ic_url, identity_name
                );
                return Ok(agent.clone());
            }
        }

        // Need to create new agent (write lock)
        let mut agents = self.agents.write().await;

        // Double-check in case another thread created it while we waited
        if let Some(agent) = agents.get(&key) {
            return Ok(agent.clone());
        }

        info!(
            "Creating new agent for {} with identity {}",
            config.ic_url, identity_name
        );

        // Create new agent
        let mut agent = Agent::builder()
            .with_url(&config.ic_url)
            .with_ingress_expiry(config.timeout)
            .build()?;

        // Set identity if available
        if let Some(identity) = self.get_dfx_identity().await? {
            agent.set_identity(identity);
        }

        // Fetch root key for local development
        if config.fetch_root_key {
            debug!("Fetching root key for local development");
            agent.fetch_root_key().await?;
        }

        let agent = Arc::new(agent);
        agents.insert(key, agent.clone());

        Ok(agent)
    }

    /// Clear all cached agents
    ///
    /// Useful for testing or when identity changes require fresh connections
    pub async fn clear(&self) {
        let mut agents = self.agents.write().await;
        let count = agents.len();
        agents.clear();
        info!("Cleared {} cached agents", count);
    }

    /// Get the current dfx identity name
    async fn get_current_identity_name(&self) -> Result<String> {
        // Try to get current identity from dfx
        match tokio::process::Command::new("dfx")
            .args(["identity", "whoami"])
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                let identity = String::from_utf8(output.stdout)?.trim().to_string();
                Ok(identity)
            }
            _ => {
                // Fall back to "default" if dfx not available
                Ok("default".to_string())
            }
        }
    }

    /// Get the current dfx identity if available
    async fn get_dfx_identity(&self) -> Result<Option<Box<dyn ic_agent::Identity>>> {
        // Try to find dfx in common locations
        let dfx_paths = [
            "/Users/goshea/Library/Application Support/org.dfinity.dfx/bin/dfx",
            "/usr/local/bin/dfx",
            "dfx", // In PATH
        ];

        for dfx_path in &dfx_paths {
            if let Ok(output) = tokio::process::Command::new(dfx_path)
                .args(["identity", "export", "--identity-type=secp256k1"])
                .output()
                .await
            {
                if output.status.success() {
                    let pem_content = String::from_utf8(output.stdout)?;
                    if let Ok(identity) =
                        ic_agent::identity::Secp256k1Identity::from_pem(pem_content.as_bytes())
                    {
                        return Ok(Some(Box::new(identity)));
                    }
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;
    use std::time::Duration;

    #[tokio::test]
    async fn test_agent_pool_creation() {
        let pool = AgentPool::new();
        assert!(pool.agents.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_global_pool_singleton() {
        let pool1 = AgentPool::global();
        let pool2 = AgentPool::global();

        // Should be the same instance
        assert!(Arc::ptr_eq(&pool1, &pool2));
    }

    #[tokio::test]
    async fn test_agent_caching() {
        let pool = AgentPool::new();
        let config = McpConfig {
            canister_id: Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap(),
            ic_url: "http://localhost:4943".to_string(),
            timeout: Duration::from_secs(30),
            fetch_root_key: true,
            max_concurrent_requests: 10,
        };

        // First call should create new agent
        let agent1 = pool.get_or_create_agent(&config).await.unwrap();

        // Second call should reuse cached agent
        let agent2 = pool.get_or_create_agent(&config).await.unwrap();

        // Should be the same Arc instance
        assert!(Arc::ptr_eq(&agent1, &agent2));
    }

    #[tokio::test]
    async fn test_pool_clear() {
        let pool = AgentPool::new();
        let config = McpConfig {
            canister_id: Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap(),
            ic_url: "http://localhost:4943".to_string(),
            timeout: Duration::from_secs(30),
            fetch_root_key: true,
            max_concurrent_requests: 10,
        };

        // Create an agent
        let _agent = pool.get_or_create_agent(&config).await.unwrap();
        assert_eq!(pool.agents.read().await.len(), 1);

        // Clear the pool
        pool.clear().await;
        assert!(pool.agents.read().await.is_empty());
    }
}
