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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    // Mock implementation for testing
    struct MockServer {
        initialized: bool,
        pre_upgrade_called: bool,
        post_upgrade_called: bool,
        stopped: bool,
        heartbeat_count: u32,
    }

    impl MockServer {
        fn new() -> Self {
            Self {
                initialized: false,
                pre_upgrade_called: false,
                post_upgrade_called: false,
                stopped: false,
                heartbeat_count: 0,
            }
        }
    }

    #[async_trait]
    impl IcarusServerLifecycle for MockServer {
        async fn on_initialize(&mut self) -> Result<()> {
            self.initialized = true;
            Ok(())
        }

        async fn on_pre_upgrade(&self) -> Result<()> {
            // Note: pre_upgrade takes &self, not &mut self
            // We can't modify state here in the real implementation
            Ok(())
        }

        async fn on_post_upgrade(&mut self) -> Result<()> {
            self.post_upgrade_called = true;
            Ok(())
        }

        async fn on_stop(&self) -> Result<()> {
            // Note: on_stop takes &self, not &mut self
            // We can't modify state here in the real implementation
            Ok(())
        }

        async fn on_heartbeat(&mut self) -> Result<()> {
            self.heartbeat_count += 1;
            Ok(())
        }
    }

    // Test server that returns errors
    struct ErrorServer;

    #[async_trait]
    impl IcarusServerLifecycle for ErrorServer {
        async fn on_initialize(&mut self) -> Result<()> {
            Err(crate::error::IcarusError::State(
                "Initialization failed".to_string(),
            ))
        }

        async fn on_pre_upgrade(&self) -> Result<()> {
            Err(crate::error::IcarusError::State(
                "Pre-upgrade failed".to_string(),
            ))
        }

        async fn on_post_upgrade(&mut self) -> Result<()> {
            Err(crate::error::IcarusError::State(
                "Post-upgrade failed".to_string(),
            ))
        }

        async fn on_stop(&self) -> Result<()> {
            Err(crate::error::IcarusError::State("Stop failed".to_string()))
        }

        async fn on_heartbeat(&mut self) -> Result<()> {
            Err(crate::error::IcarusError::State(
                "Heartbeat failed".to_string(),
            ))
        }
    }

    #[tokio::test]
    async fn test_mock_server_initialization() {
        let mut server = MockServer::new();
        assert!(!server.initialized);

        let result = server.on_initialize().await;
        assert!(result.is_ok());
        assert!(server.initialized);
    }

    #[tokio::test]
    async fn test_mock_server_pre_upgrade() {
        let server = MockServer::new();
        let result = server.on_pre_upgrade().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_server_post_upgrade() {
        let mut server = MockServer::new();
        assert!(!server.post_upgrade_called);

        let result = server.on_post_upgrade().await;
        assert!(result.is_ok());
        assert!(server.post_upgrade_called);
    }

    #[tokio::test]
    async fn test_mock_server_stop() {
        let server = MockServer::new();
        let result = server.on_stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_server_heartbeat() {
        let mut server = MockServer::new();
        assert_eq!(server.heartbeat_count, 0);

        // Call heartbeat multiple times
        for i in 1..=5 {
            let result = server.on_heartbeat().await;
            assert!(result.is_ok());
            assert_eq!(server.heartbeat_count, i);
        }
    }

    #[tokio::test]
    async fn test_lifecycle_sequence() {
        let mut server = MockServer::new();

        // 1. Initialize
        server.on_initialize().await.unwrap();
        assert!(server.initialized);

        // 2. Heartbeat
        server.on_heartbeat().await.unwrap();
        assert_eq!(server.heartbeat_count, 1);

        // 3. Pre-upgrade
        server.on_pre_upgrade().await.unwrap();

        // 4. Post-upgrade
        server.on_post_upgrade().await.unwrap();
        assert!(server.post_upgrade_called);

        // 5. More heartbeats
        server.on_heartbeat().await.unwrap();
        assert_eq!(server.heartbeat_count, 2);

        // 6. Stop
        server.on_stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_error_server_initialization() {
        let mut server = ErrorServer;
        let result = server.on_initialize().await;
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(error_msg.contains("Initialization failed"));
    }

    #[tokio::test]
    async fn test_error_server_pre_upgrade() {
        let server = ErrorServer;
        let result = server.on_pre_upgrade().await;
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(error_msg.contains("Pre-upgrade failed"));
    }

    #[tokio::test]
    async fn test_error_server_post_upgrade() {
        let mut server = ErrorServer;
        let result = server.on_post_upgrade().await;
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(error_msg.contains("Post-upgrade failed"));
    }

    #[tokio::test]
    async fn test_error_server_stop() {
        let server = ErrorServer;
        let result = server.on_stop().await;
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(error_msg.contains("Stop failed"));
    }

    #[tokio::test]
    async fn test_error_server_heartbeat() {
        let mut server = ErrorServer;
        let result = server.on_heartbeat().await;
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(error_msg.contains("Heartbeat failed"));
    }

    #[test]
    fn test_lifecycle_config_creation() {
        let config = LifecycleConfig {
            heartbeat_interval: 60,
            auto_snapshot: true,
            max_snapshots: 10,
        };

        assert_eq!(config.heartbeat_interval, 60);
        assert_eq!(config.auto_snapshot, true);
        assert_eq!(config.max_snapshots, 10);
    }

    #[test]
    fn test_lifecycle_config_clone() {
        let config1 = LifecycleConfig {
            heartbeat_interval: 30,
            auto_snapshot: false,
            max_snapshots: 5,
        };

        let config2 = config1.clone();
        assert_eq!(config2.heartbeat_interval, 30);
        assert_eq!(config2.auto_snapshot, false);
        assert_eq!(config2.max_snapshots, 5);
    }

    #[test]
    fn test_lifecycle_config_debug() {
        let config = LifecycleConfig {
            heartbeat_interval: 120,
            auto_snapshot: true,
            max_snapshots: 15,
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("heartbeat_interval: 120"));
        assert!(debug_str.contains("auto_snapshot: true"));
        assert!(debug_str.contains("max_snapshots: 15"));
    }

    #[test]
    fn test_lifecycle_config_disabled_heartbeat() {
        let config = LifecycleConfig {
            heartbeat_interval: 0, // Disabled
            auto_snapshot: false,
            max_snapshots: 0,
        };

        assert_eq!(config.heartbeat_interval, 0);
        assert!(!config.auto_snapshot);
        assert_eq!(config.max_snapshots, 0);
    }

    #[test]
    fn test_lifecycle_config_extreme_values() {
        let config = LifecycleConfig {
            heartbeat_interval: u64::MAX,
            auto_snapshot: true,
            max_snapshots: u32::MAX,
        };

        assert_eq!(config.heartbeat_interval, u64::MAX);
        assert!(config.auto_snapshot);
        assert_eq!(config.max_snapshots, u32::MAX);
    }

    // Test default implementations work
    struct DefaultServer;

    #[async_trait]
    impl IcarusServerLifecycle for DefaultServer {
        // Use all default implementations
    }

    #[tokio::test]
    async fn test_default_implementations() {
        let mut server = DefaultServer;

        // All default implementations should return Ok(())
        assert!(server.on_initialize().await.is_ok());
        assert!(server.on_pre_upgrade().await.is_ok());
        assert!(server.on_post_upgrade().await.is_ok());
        assert!(server.on_stop().await.is_ok());
        assert!(server.on_heartbeat().await.is_ok());
    }

    // Test that we can call methods multiple times
    #[tokio::test]
    async fn test_multiple_calls() {
        let mut server = MockServer::new();

        // Initialize multiple times (shouldn't be an issue)
        server.on_initialize().await.unwrap();
        server.on_initialize().await.unwrap();
        assert!(server.initialized);

        // Multiple heartbeats
        for i in 1..=10 {
            server.on_heartbeat().await.unwrap();
            assert_eq!(server.heartbeat_count, i);
        }

        // Multiple upgrade cycles
        server.on_pre_upgrade().await.unwrap();
        server.on_post_upgrade().await.unwrap();
        server.on_pre_upgrade().await.unwrap();
        server.on_post_upgrade().await.unwrap();
        assert!(server.post_upgrade_called);

        // Multiple stops
        server.on_stop().await.unwrap();
        server.on_stop().await.unwrap();
    }

    // Test immutable reference methods (pre_upgrade, stop)
    #[tokio::test]
    async fn test_immutable_reference_methods() {
        let server = MockServer::new();

        // These methods take &self, not &mut self
        server.on_pre_upgrade().await.unwrap();
        server.on_stop().await.unwrap();

        // Should be able to call them multiple times on the same reference
        server.on_pre_upgrade().await.unwrap();
        server.on_stop().await.unwrap();
    }

    // Test async trait requirements
    #[tokio::test]
    async fn test_async_trait_requirements() {
        let mut server = MockServer::new();

        // All methods should be async and return a Future
        let init_future = server.on_initialize();
        let result = init_future.await;
        assert!(result.is_ok());

        let heartbeat_future = server.on_heartbeat();
        let result = heartbeat_future.await;
        assert!(result.is_ok());
    }
}
