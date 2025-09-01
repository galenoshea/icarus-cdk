//! Persistent state management for servers

use async_trait::async_trait;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for managing persistent state
#[async_trait]
pub trait IcarusPersistentState: Send + Sync {
    /// Save a value to persistent storage
    async fn set(&mut self, key: String, value: Vec<u8>) -> Result<()>;
    
    /// Retrieve a value from persistent storage
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// Delete a value from persistent storage
    async fn delete(&mut self, key: &str) -> Result<()>;
    
    /// List all keys in persistent storage
    async fn list_keys(&self) -> Result<Vec<String>>;
    
    /// Clear all persistent storage
    async fn clear(&mut self) -> Result<()>;
    
    /// Get storage size in bytes
    async fn size(&self) -> Result<u64>;
}

/// Helper methods for typed storage
pub trait TypedPersistentState: IcarusPersistentState {
    /// Save a typed value
    fn set_typed<T: Serialize + Send + Sync>(&mut self, key: String, value: &T) -> impl std::future::Future<Output = Result<()>> + Send
    where
        Self: Send,
    {
        async move {
            let bytes = serde_json::to_vec(value)
                .map_err(crate::error::IcarusError::Serialization)?;
            self.set(key, bytes).await
        }
    }
    
    /// Get a typed value
    fn get_typed<T: for<'de> Deserialize<'de> + Send>(&self, key: &str) -> impl std::future::Future<Output = Result<Option<T>>> + Send
    where
        Self: Send + Sync,
    {
        async move {
            match self.get(key).await? {
                Some(bytes) => {
                    let value = serde_json::from_slice(&bytes)
                        .map_err(crate::error::IcarusError::Serialization)?;
                    Ok(Some(value))
                }
                None => Ok(None),
            }
        }
    }
}

/// Automatically implement TypedPersistentState for all IcarusPersistentState
impl<T: IcarusPersistentState + ?Sized> TypedPersistentState for T {}

/// In-memory implementation for testing
pub struct MemoryPersistentState {
    data: HashMap<String, Vec<u8>>,
}

impl MemoryPersistentState {
    /// Create new in-memory persistent state
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Default for MemoryPersistentState {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IcarusPersistentState for MemoryPersistentState {
    async fn set(&mut self, key: String, value: Vec<u8>) -> Result<()> {
        self.data.insert(key, value);
        Ok(())
    }
    
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.data.get(key).cloned())
    }
    
    async fn delete(&mut self, key: &str) -> Result<()> {
        self.data.remove(key);
        Ok(())
    }
    
    async fn list_keys(&self) -> Result<Vec<String>> {
        Ok(self.data.keys().cloned().collect())
    }
    
    async fn clear(&mut self) -> Result<()> {
        self.data.clear();
        Ok(())
    }
    
    async fn size(&self) -> Result<u64> {
        let size: usize = self.data.values().map(|v| v.len()).sum();
        Ok(size as u64)
    }
}