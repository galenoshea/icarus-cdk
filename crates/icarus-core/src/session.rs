//! Session management for stateful MCP interactions

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A user session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: String,
    /// User principal (ICP identity)
    pub principal: Option<String>,
    /// Session creation timestamp
    pub created_at: u64,
    /// Last activity timestamp
    pub last_activity: u64,
    /// Session metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Whether the session is active
    pub active: bool,
}

/// Trait for managing sessions
#[async_trait]
pub trait SessionManager: Send + Sync {
    /// Create a new session
    async fn create_session(&mut self, principal: Option<String>) -> Result<Session>;

    /// Get a session by ID
    async fn get_session(&self, session_id: &str) -> Result<Option<Session>>;

    /// Update a session
    async fn update_session(&mut self, session: Session) -> Result<()>;

    /// Delete a session
    async fn delete_session(&mut self, session_id: &str) -> Result<()>;

    /// List active sessions
    async fn list_active_sessions(&self) -> Result<Vec<Session>>;

    /// Clean up expired sessions
    async fn cleanup_expired(&mut self, max_age_secs: u64) -> Result<u32>;

    /// Touch a session to update last activity
    async fn touch_session(&mut self, session_id: &str) -> Result<()> {
        if let Some(mut session) = self.get_session(session_id).await? {
            session.last_activity = ic_cdk::api::time();
            self.update_session(session).await?;
        }
        Ok(())
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum session duration in seconds
    pub max_duration: u64,
    /// Session timeout (inactivity) in seconds
    pub timeout: u64,
    /// Maximum number of concurrent sessions per principal
    pub max_per_principal: u32,
    /// Whether to require authentication
    pub require_auth: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_duration: 86400, // 24 hours
            timeout: 3600,       // 1 hour
            max_per_principal: 10,
            require_auth: false,
        }
    }
}

/// In-memory session manager for testing
pub struct MemorySessionManager {
    sessions: HashMap<String, Session>,
    #[cfg(test)]
    mock_time: Option<u64>,
}

impl MemorySessionManager {
    /// Create a new memory session manager
    pub fn new(_config: SessionConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            #[cfg(test)]
            mock_time: None,
        }
    }

    #[cfg(test)]
    fn get_time(&self) -> u64 {
        self.mock_time.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64
        })
    }

    #[cfg(not(test))]
    fn get_time(&self) -> u64 {
        ic_cdk::api::time()
    }

    fn generate_session_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("session_{}", timestamp)
    }
}

#[async_trait]
impl SessionManager for MemorySessionManager {
    async fn create_session(&mut self, principal: Option<String>) -> Result<Session> {
        let now = self.get_time();
        let session = Session {
            id: Self::generate_session_id(),
            principal,
            created_at: now,
            last_activity: now,
            metadata: HashMap::new(),
            active: true,
        };

        self.sessions.insert(session.id.clone(), session.clone());
        Ok(session)
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
        Ok(self.sessions.get(session_id).cloned())
    }

    async fn update_session(&mut self, session: Session) -> Result<()> {
        self.sessions.insert(session.id.clone(), session);
        Ok(())
    }

    async fn delete_session(&mut self, session_id: &str) -> Result<()> {
        self.sessions.remove(session_id);
        Ok(())
    }

    async fn list_active_sessions(&self) -> Result<Vec<Session>> {
        Ok(self
            .sessions
            .values()
            .filter(|s| s.active)
            .cloned()
            .collect())
    }

    async fn cleanup_expired(&mut self, max_age_secs: u64) -> Result<u32> {
        let now = self.get_time();
        let cutoff = now.saturating_sub(max_age_secs * 1_000_000_000); // Convert to nanos

        let expired: Vec<String> = self
            .sessions
            .iter()
            .filter(|(_, session)| session.last_activity < cutoff)
            .map(|(id, _)| id.clone())
            .collect();

        let count = expired.len() as u32;
        for id in expired {
            self.sessions.remove(&id);
        }

        Ok(count)
    }

    /// Touch a session to update last activity (override to use mock time in tests)
    async fn touch_session(&mut self, session_id: &str) -> Result<()> {
        if let Some(mut session) = self.get_session(session_id).await? {
            session.last_activity = self.get_time();
            self.update_session(session).await?;
        }
        Ok(())
    }
}

/// Session context for request handling
#[derive(Debug, Clone)]
pub struct SessionContext {
    pub session: Session,
    pub authenticated: bool,
}

impl SessionContext {
    /// Create a new session context
    pub fn new(session: Session, authenticated: bool) -> Self {
        Self {
            session,
            authenticated,
        }
    }

    /// Get session metadata value
    pub fn get_metadata<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.session
            .metadata
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Set session metadata value
    pub fn set_metadata<T: Serialize>(&mut self, key: String, value: T) -> Result<()> {
        let json_value =
            serde_json::to_value(value).map_err(crate::error::IcarusError::Serialization)?;
        self.session.metadata.insert(key, json_value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    impl MemorySessionManager {
        /// Set mock time for testing
        pub fn set_mock_time(&mut self, time: u64) {
            self.mock_time = Some(time);
        }

        /// Clear mock time
        pub fn clear_mock_time(&mut self) {
            self.mock_time = None;
        }
    }

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.max_duration, 86400);
        assert_eq!(config.timeout, 3600);
        assert_eq!(config.max_per_principal, 10);
        assert!(!config.require_auth);
    }

    #[test]
    fn test_session_config_serialization() {
        let config = SessionConfig {
            max_duration: 7200,
            timeout: 1800,
            max_per_principal: 5,
            require_auth: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SessionConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.max_duration, deserialized.max_duration);
        assert_eq!(config.timeout, deserialized.timeout);
        assert_eq!(config.max_per_principal, deserialized.max_per_principal);
        assert_eq!(config.require_auth, deserialized.require_auth);
    }

    #[test]
    fn test_session_serialization() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), json!("value1"));
        metadata.insert("key2".to_string(), json!(42));

        let session = Session {
            id: "test-session".to_string(),
            principal: Some("test-principal".to_string()),
            created_at: 1000000000,
            last_activity: 1000000000,
            metadata,
            active: true,
        };

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.principal, deserialized.principal);
        assert_eq!(session.created_at, deserialized.created_at);
        assert_eq!(session.last_activity, deserialized.last_activity);
        assert_eq!(session.active, deserialized.active);
        assert_eq!(session.metadata.len(), deserialized.metadata.len());
    }

    #[tokio::test]
    async fn test_memory_session_manager_basic() {
        let config = SessionConfig::default();
        let mut manager = MemorySessionManager::new(config);

        // Create session
        let session = manager
            .create_session(Some("test-principal".to_string()))
            .await
            .unwrap();
        assert!(session.active);
        assert_eq!(session.principal, Some("test-principal".to_string()));
        assert!(session.id.starts_with("session_"));

        // Get session
        let retrieved = manager.get_session(&session.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, session.id);

        // List active sessions
        let active = manager.list_active_sessions().await.unwrap();
        assert_eq!(active.len(), 1);

        // Delete session
        manager.delete_session(&session.id).await.unwrap();
        let deleted = manager.get_session(&session.id).await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_create_session_anonymous() {
        let config = SessionConfig::default();
        let mut manager = MemorySessionManager::new(config);

        let session = manager.create_session(None).await.unwrap();
        assert!(session.principal.is_none());
        assert!(session.active);
        assert_eq!(session.created_at, session.last_activity);
    }

    #[tokio::test]
    async fn test_create_multiple_sessions() {
        let config = SessionConfig::default();
        let mut manager = MemorySessionManager::new(config);

        let session1 = manager
            .create_session(Some("principal1".to_string()))
            .await
            .unwrap();
        let session2 = manager
            .create_session(Some("principal2".to_string()))
            .await
            .unwrap();
        let session3 = manager.create_session(None).await.unwrap();

        // All should have unique IDs
        assert_ne!(session1.id, session2.id);
        assert_ne!(session1.id, session3.id);
        assert_ne!(session2.id, session3.id);

        // Should be able to retrieve all
        assert!(manager.get_session(&session1.id).await.unwrap().is_some());
        assert!(manager.get_session(&session2.id).await.unwrap().is_some());
        assert!(manager.get_session(&session3.id).await.unwrap().is_some());

        let active = manager.list_active_sessions().await.unwrap();
        assert_eq!(active.len(), 3);
    }

    #[tokio::test]
    async fn test_update_session() {
        let config = SessionConfig::default();
        let mut manager = MemorySessionManager::new(config);

        let mut session = manager
            .create_session(Some("test-principal".to_string()))
            .await
            .unwrap();

        // Modify session
        session.active = false;
        session.metadata.insert("key".to_string(), json!("value"));

        manager.update_session(session.clone()).await.unwrap();

        // Verify update
        let updated = manager.get_session(&session.id).await.unwrap().unwrap();
        assert!(!updated.active);
        assert_eq!(updated.metadata.get("key"), Some(&json!("value")));
    }

    #[tokio::test]
    async fn test_get_nonexistent_session() {
        let config = SessionConfig::default();
        let manager = MemorySessionManager::new(config);

        let result = manager.get_session("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_session() {
        let config = SessionConfig::default();
        let mut manager = MemorySessionManager::new(config);

        // Should not fail
        let result = manager.delete_session("nonexistent").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_active_sessions_filter() {
        let config = SessionConfig::default();
        let mut manager = MemorySessionManager::new(config);

        let mut session1 = manager
            .create_session(Some("p1".to_string()))
            .await
            .unwrap();
        let session2 = manager
            .create_session(Some("p2".to_string()))
            .await
            .unwrap();
        let mut session3 = manager
            .create_session(Some("p3".to_string()))
            .await
            .unwrap();

        // Make session1 and session3 inactive
        session1.active = false;
        session3.active = false;
        manager.update_session(session1).await.unwrap();
        manager.update_session(session3).await.unwrap();

        let active = manager.list_active_sessions().await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, session2.id);
    }

    #[tokio::test]
    async fn test_cleanup_expired_sessions() {
        let config = SessionConfig::default();
        let mut manager = MemorySessionManager::new(config);

        let base_time = 1000000000u64; // 1 second in nanoseconds
        manager.set_mock_time(base_time);

        // Create sessions at different times
        let session1 = manager
            .create_session(Some("p1".to_string()))
            .await
            .unwrap();

        manager.set_mock_time(base_time + 2_000_000_000); // +2 seconds
        let session2 = manager
            .create_session(Some("p2".to_string()))
            .await
            .unwrap();

        manager.set_mock_time(base_time + 4_000_000_000); // +4 seconds
        let session3 = manager
            .create_session(Some("p3".to_string()))
            .await
            .unwrap();

        // Move to +7 seconds and cleanup sessions older than 3 seconds
        // This means anything with last_activity < (7000000000 - 3000000000) = 4000000000 should be cleaned
        // session1: last_activity = 1000000000 (should be cleaned)
        // session2: last_activity = 3000000000 (should be cleaned)
        // session3: last_activity = 5000000000 (should remain)
        manager.set_mock_time(base_time + 7_000_000_000);
        let cleaned = manager.cleanup_expired(3).await.unwrap();

        assert_eq!(cleaned, 2); // session1 and session2 should be cleaned
        assert!(manager.get_session(&session1.id).await.unwrap().is_none());
        assert!(manager.get_session(&session2.id).await.unwrap().is_none());
        assert!(manager.get_session(&session3.id).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_cleanup_expired_no_sessions() {
        let config = SessionConfig::default();
        let mut manager = MemorySessionManager::new(config);

        let cleaned = manager.cleanup_expired(3600).await.unwrap();
        assert_eq!(cleaned, 0);
    }

    #[tokio::test]
    async fn test_touch_session() {
        let config = SessionConfig::default();
        let mut manager = MemorySessionManager::new(config);

        let base_time = 1000000000u64;
        manager.set_mock_time(base_time);

        let session = manager
            .create_session(Some("test-principal".to_string()))
            .await
            .unwrap();
        assert_eq!(session.last_activity, base_time);

        // Touch session at a later time
        manager.set_mock_time(base_time + 1_000_000_000); // +1 second
        manager.touch_session(&session.id).await.unwrap();

        let updated = manager.get_session(&session.id).await.unwrap().unwrap();
        assert_eq!(updated.last_activity, base_time + 1_000_000_000);
    }

    #[tokio::test]
    async fn test_touch_nonexistent_session() {
        let config = SessionConfig::default();
        let mut manager = MemorySessionManager::new(config);

        // Should not fail
        let result = manager.touch_session("nonexistent").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_session_context_creation() {
        let session = Session {
            id: "test-session".to_string(),
            principal: Some("test-principal".to_string()),
            created_at: 1000000000,
            last_activity: 1000000000,
            metadata: HashMap::new(),
            active: true,
        };

        let context = SessionContext::new(session.clone(), true);
        assert_eq!(context.session.id, session.id);
        assert!(context.authenticated);

        let context2 = SessionContext::new(session, false);
        assert!(!context2.authenticated);
    }

    #[test]
    fn test_session_context_metadata() {
        let session = Session {
            id: "test-session".to_string(),
            principal: Some("test-principal".to_string()),
            created_at: 1000000000,
            last_activity: 1000000000,
            metadata: HashMap::new(),
            active: true,
        };

        let mut context = SessionContext::new(session, true);

        // Set various types of metadata
        context
            .set_metadata("string_val".to_string(), "hello")
            .unwrap();
        context.set_metadata("number_val".to_string(), 42).unwrap();
        context.set_metadata("bool_val".to_string(), true).unwrap();
        context
            .set_metadata("object_val".to_string(), json!({"key": "value"}))
            .unwrap();

        // Get metadata with correct types
        let string_val: String = context.get_metadata("string_val").unwrap();
        assert_eq!(string_val, "hello");

        let number_val: i32 = context.get_metadata("number_val").unwrap();
        assert_eq!(number_val, 42);

        let bool_val: bool = context.get_metadata("bool_val").unwrap();
        assert!(bool_val);

        let object_val: serde_json::Value = context.get_metadata("object_val").unwrap();
        assert_eq!(object_val, json!({"key": "value"}));

        // Try to get non-existent metadata
        let missing: Option<String> = context.get_metadata("missing");
        assert!(missing.is_none());

        // Try to get metadata with wrong type
        let wrong_type: Option<Vec<String>> = context.get_metadata("string_val");
        assert!(wrong_type.is_none());
    }

    #[test]
    fn test_session_id_generation() {
        let id1 = MemorySessionManager::generate_session_id();
        // Add a small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id2 = MemorySessionManager::generate_session_id();

        assert!(id1.starts_with("session_"));
        assert!(id2.starts_with("session_"));
        assert_ne!(id1, id2); // Should be unique (timestamp-based)
    }

    #[tokio::test]
    async fn test_session_manager_concurrent_operations() {
        let config = SessionConfig::default();
        let mut manager = MemorySessionManager::new(config);

        // Create multiple sessions
        let mut sessions = Vec::new();
        for i in 0..10 {
            let session = manager
                .create_session(Some(format!("principal{}", i)))
                .await
                .unwrap();
            sessions.push(session);
        }

        // Verify all sessions exist
        for session in &sessions {
            let retrieved = manager.get_session(&session.id).await.unwrap();
            assert!(retrieved.is_some());
        }

        // Update some sessions
        for (i, session) in sessions.iter_mut().enumerate() {
            if i % 2 == 0 {
                session.active = false;
                manager.update_session(session.clone()).await.unwrap();
            }
        }

        // Check active sessions count before deletion
        let active = manager.list_active_sessions().await.unwrap();
        assert_eq!(active.len(), 5); // Half should be active (indices 1,3,5,7,9)

        // Delete some sessions
        for (i, session) in sessions.iter().enumerate() {
            if i < 3 {
                manager.delete_session(&session.id).await.unwrap();
            }
        }

        // Check active sessions count after deletion
        let active_after_delete = manager.list_active_sessions().await.unwrap();
        assert_eq!(active_after_delete.len(), 4); // Should have 4 active (indices 3,5,7,9)

        // Verify deletion
        for (i, session) in sessions.iter().enumerate() {
            let exists = manager.get_session(&session.id).await.unwrap().is_some();
            if i < 3 {
                assert!(!exists);
            } else {
                assert!(exists);
            }
        }
    }

    #[tokio::test]
    async fn test_session_edge_cases() {
        let config = SessionConfig::default();
        let mut manager = MemorySessionManager::new(config);

        // Test empty principal string
        let session = manager.create_session(Some("".to_string())).await.unwrap();
        assert_eq!(session.principal, Some("".to_string()));

        // Test very long principal
        let long_principal = "a".repeat(1000);
        let session2 = manager
            .create_session(Some(long_principal.clone()))
            .await
            .unwrap();
        assert_eq!(session2.principal, Some(long_principal));

        // Test session with empty metadata initially
        assert!(session.metadata.is_empty());
        assert!(session2.metadata.is_empty());
    }
}
