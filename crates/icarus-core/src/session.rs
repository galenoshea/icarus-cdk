//! Session management for stateful MCP interactions

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::Result;

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
            max_duration: 86400,     // 24 hours
            timeout: 3600,           // 1 hour
            max_per_principal: 10,
            require_auth: false,
        }
    }
}

/// In-memory session manager for testing
pub struct MemorySessionManager {
    sessions: HashMap<String, Session>,
    #[allow(dead_code)]
    config: SessionConfig,
    #[cfg(test)]
    mock_time: Option<u64>,
}

impl MemorySessionManager {
    /// Create a new memory session manager
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            config,
            #[cfg(test)]
            mock_time: None,
        }
    }
    
    #[cfg(test)]
    fn get_time(&self) -> u64 {
        self.mock_time.unwrap_or_else(|| std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64)
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
        Ok(self.sessions.values()
            .filter(|s| s.active)
            .cloned()
            .collect())
    }
    
    async fn cleanup_expired(&mut self, max_age_secs: u64) -> Result<u32> {
        let now = self.get_time();
        let cutoff = now.saturating_sub(max_age_secs * 1_000_000_000); // Convert to nanos
        
        let expired: Vec<String> = self.sessions.iter()
            .filter(|(_, session)| session.last_activity < cutoff)
            .map(|(id, _)| id.clone())
            .collect();
            
        let count = expired.len() as u32;
        for id in expired {
            self.sessions.remove(&id);
        }
        
        Ok(count)
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
        self.session.metadata.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    
    /// Set session metadata value
    pub fn set_metadata<T: Serialize>(&mut self, key: String, value: T) -> Result<()> {
        let json_value = serde_json::to_value(value)
            .map_err(crate::error::IcarusError::Serialization)?;
        self.session.metadata.insert(key, json_value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_session_manager() {
        let config = SessionConfig::default();
        let mut manager = MemorySessionManager::new(config);
        
        // Create session
        let session = manager.create_session(Some("test-principal".to_string())).await.unwrap();
        assert!(session.active);
        assert_eq!(session.principal, Some("test-principal".to_string()));
        
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
}