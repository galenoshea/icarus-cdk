//! Example memory server using Icarus SDK
//! 
//! This example demonstrates how to build an MCP server that remembers facts
//! and can recall them later, running as an ICP canister.

use icarus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::json;

/// A memory stored in the server
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
pub struct Memory {
    id: String,
    content: String,
    created_at: u64,
    tags: Vec<String>,
}

/// Our memory server
#[icarus_server(name = "memory-server", version = "1.0.0")]
pub struct MemoryServer {
    memories: HashMap<String, Memory>,
}

impl MemoryServer {
    pub fn new() -> Self {
        Self {
            memories: HashMap::new(),
        }
    }
}

#[icarus_tools]
impl MemoryServer {
    /// Store a new memory with optional tags
    #[icarus_tool(name = "memorize")]
    pub async fn memorize(&mut self, content: String, tags: Option<Vec<String>>) -> ToolResult {
        let id = format!("mem_{}", self.memories.len());
        let memory = Memory {
            id: id.clone(),
            content,
            created_at: ic_cdk::api::time(),
            tags: tags.unwrap_or_default(),
        };
        
        self.memories.insert(id.clone(), memory);
        
        Ok(json!({
            "status": "success",
            "message": "Memory stored successfully",
            "id": id
        }))
    }
    
    /// Remove a specific memory by ID
    #[icarus_tool(name = "forget")]
    pub async fn forget(&mut self, id: String) -> ToolResult {
        match self.memories.remove(&id) {
            Some(_) => Ok(json!({
                "status": "success",
                "message": "Memory forgotten"
            })),
            None => Ok(json!({
                "status": "error",
                "message": "Memory not found"
            }))
        }
    }
    
    /// Retrieve memories matching a query
    #[icarus_tool(name = "recall")]
    pub async fn recall(&self, query: String) -> ToolResult {
        let matches: Vec<&Memory> = self.memories
            .values()
            .filter(|memory| {
                memory.content.contains(&query) ||
                memory.tags.iter().any(|tag| tag.contains(&query))
            })
            .collect();
            
        Ok(json!({
            "matches": matches,
            "count": matches.len()
        }))
    }
    
    /// List all stored memories with optional limit
    #[icarus_tool(name = "list")]
    pub async fn list(&self, limit: Option<usize>) -> ToolResult {
        let mut memories: Vec<&Memory> = self.memories.values().collect();
        memories.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        let limited = if let Some(max) = limit {
            memories.into_iter().take(max).collect()
        } else {
            memories
        };
        
        Ok(json!({
            "memories": limited,
            "total": self.memories.len()
        }))
    }
}

// Generate Candid interface
icarus::export_candid!();