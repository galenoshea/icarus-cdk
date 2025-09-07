//! Basic Memory Server Example
//!
//! This example demonstrates a simple MCP server that stores and retrieves
//! text memories with tags. It showcases:
//! - Using stable storage for persistence
//! - Defining MCP tools with the icarus_module macro
//! - Implementing CRUD operations

use candid::{CandidType, Deserialize};
use ic_cdk::api::time;
use icarus::prelude::*;
use serde::Serialize;

/// A memory entry that persists across canister upgrades
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub created_at: u64,
    pub tags: Vec<String>,
}

// Declare stable storage that persists across upgrades
stable_storage! {
    // BTree map for efficient key-value storage
    MEMORIES: StableBTreeMap<String, MemoryEntry, Memory> = memory_id!(0);
    // Simple counter for generating unique IDs
    COUNTER: u64 = 0;
}

// Helper function to generate unique IDs
fn generate_id() -> String {
    COUNTER.with(|c| {
        let mut counter = c.borrow_mut();
        *counter += 1;
        format!("mem_{}", *counter)
    })
}

/// MCP module containing all tool functions
#[icarus_module]
mod tools {
    use super::*;

    /// Store a new memory with optional tags
    #[update]
    #[icarus_tool("Store a new memory with optional tags")]
    pub fn memorize(content: String, tags: Option<Vec<String>>) -> Result<String, String> {
        if content.is_empty() {
            return Err("Content cannot be empty".to_string());
        }

        let id = generate_id();
        let memory = MemoryEntry {
            id: id.clone(),
            content,
            created_at: time(),
            tags: tags.unwrap_or_default(),
        };

        MEMORIES.with(|m| {
            m.borrow_mut().insert(id.clone(), memory);
        });

        Ok(id)
    }

    /// Retrieve a specific memory by ID
    #[query]
    #[icarus_tool("Retrieve a specific memory by ID")]
    pub fn recall(id: String) -> Result<MemoryEntry, String> {
        MEMORIES.with(|m| {
            m.borrow()
                .get(&id)
                .ok_or_else(|| format!("Memory with ID {} not found", id))
        })
    }

    /// List all stored memories with optional limit
    #[query]
    #[icarus_tool("List all stored memories with optional limit")]
    pub fn list(limit: Option<u64>) -> Result<Vec<MemoryEntry>, String> {
        Ok(MEMORIES.with(|m| {
            let memories = m.borrow();
            let iter = memories.iter();

            match limit {
                Some(n) => iter.take(n as usize).map(|(_, v)| v).collect(),
                None => iter.map(|(_, v)| v).collect(),
            }
        }))
    }

    /// Search memories by tag
    #[query]
    #[icarus_tool("Search memories by tag")]
    pub fn search_by_tag(tag: String) -> Result<Vec<MemoryEntry>, String> {
        Ok(MEMORIES.with(|m| {
            m.borrow()
                .iter()
                .filter(|(_, memory)| memory.tags.contains(&tag))
                .map(|(_, v)| v)
                .collect()
        }))
    }

    /// Delete a memory by ID
    #[update]
    #[icarus_tool("Delete a memory by ID")]
    pub fn forget(id: String) -> Result<bool, String> {
        MEMORIES.with(|m| match m.borrow_mut().remove(&id) {
            Some(_) => Ok(true),
            None => Err(format!("Memory with ID {} not found", id)),
        })
    }

    /// Get total number of stored memories
    #[query]
    #[icarus_tool("Get total number of stored memories")]
    pub fn count() -> Result<u64, String> {
        Ok(MEMORIES.with(|m| m.borrow().len()))
    }

    /// Clear all memories (use with caution!)
    #[update]
    #[icarus_tool("Clear all memories - use with caution")]
    pub fn clear_all() -> Result<u64, String> {
        MEMORIES.with(|m| {
            let mut memories = m.borrow_mut();
            let count = memories.len();

            // Clear all entries
            let keys: Vec<String> = memories.iter().map(|(k, _)| k).collect();
            for key in keys {
                memories.remove(&key);
            }

            Ok(count)
        })
    }
}

// Export the Candid interface for the canister
ic_cdk::export_candid!();
