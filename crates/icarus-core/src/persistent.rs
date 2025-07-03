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
    async fn set_typed<T: Serialize>(&mut self, key: String, value: &T) -> Result<()> {
        let bytes = serde_json::to_vec(value)
            .map_err(|e| crate::error::IcarusError::Serialization(e))?;
        self.set(key, bytes).await
    }
    
    /// Get a typed value
    async fn get_typed<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        match self.get(key).await? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)
                    .map_err(|e| crate::error::IcarusError::Serialization(e))?;
                Ok(Some(value))
            }
            None => Ok(None),
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