//! Memory management for ICP stable storage

use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};
use std::cell::RefCell;

/// Memory IDs for different storage regions
pub const MEMORY_ID_CONFIG: u8 = 0;
pub const MEMORY_ID_TOOLS: u8 = 1;
pub const MEMORY_ID_RESOURCES: u8 = 2;
pub const MEMORY_ID_SESSIONS: u8 = 3;
pub const MEMORY_ID_AUDIT_LOG: u8 = 4;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    /// Global memory manager instance
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
}

/// Get a virtual memory instance for the given ID
pub fn get_memory(id: u8) -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(id)))
}
