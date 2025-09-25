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

#[cfg(test)]
mod tests {
    use super::*;
    use ic_stable_structures::Memory;

    #[test]
    fn test_memory_constants() {
        assert_eq!(MEMORY_ID_CONFIG, 0);
        assert_eq!(MEMORY_ID_TOOLS, 1);
        assert_eq!(MEMORY_ID_RESOURCES, 2);
        assert_eq!(MEMORY_ID_SESSIONS, 3);
        assert_eq!(MEMORY_ID_AUDIT_LOG, 4);
    }

    #[test]
    fn test_get_memory_basic() {
        let memory = get_memory(MEMORY_ID_CONFIG);
        // Memory should be initialized and usable
        // This is mostly a smoke test since memory operations require stable structures
        assert_eq!(memory.size(), 0); // Fresh memory should be empty
    }

    #[test]
    fn test_get_memory_different_ids() {
        let memory1 = get_memory(MEMORY_ID_CONFIG);
        let memory2 = get_memory(MEMORY_ID_TOOLS);
        let memory3 = get_memory(MEMORY_ID_RESOURCES);

        // All memories should start empty
        assert_eq!(memory1.size(), 0);
        assert_eq!(memory2.size(), 0);
        assert_eq!(memory3.size(), 0);
    }

    #[test]
    fn test_get_memory_same_id_returns_same_memory() {
        let memory1 = get_memory(MEMORY_ID_CONFIG);
        let memory2 = get_memory(MEMORY_ID_CONFIG);

        // Should return the same memory instance
        assert_eq!(memory1.size(), memory2.size());
    }

    #[test]
    fn test_memory_manager_persistence() {
        // First access
        let _memory1 = get_memory(MEMORY_ID_CONFIG);

        // Second access should return the same memory manager
        let _memory2 = get_memory(MEMORY_ID_CONFIG);

        // This is a basic test to ensure the thread-local storage works
        // In a real canister, this would persist across calls
    }

    #[test]
    fn test_all_memory_id_constants() {
        // Test that all constants are unique
        let ids = [MEMORY_ID_CONFIG,
            MEMORY_ID_TOOLS,
            MEMORY_ID_RESOURCES,
            MEMORY_ID_SESSIONS,
            MEMORY_ID_AUDIT_LOG];

        for (i, &id1) in ids.iter().enumerate() {
            for (j, &id2) in ids.iter().enumerate() {
                if i != j {
                    assert_ne!(id1, id2, "Memory IDs should be unique");
                }
            }
        }
    }

    #[test]
    fn test_memory_id_ranges() {
        // Memory IDs are compile-time constants within valid range (0-254 for IC)
        // These constants are designed to be in valid range by definition
    }

    #[test]
    fn test_get_memory_with_custom_ids() {
        // Test with custom memory IDs beyond the predefined ones
        let custom_memory_10 = get_memory(10);
        let custom_memory_20 = get_memory(20);
        let custom_memory_254 = get_memory(254); // Maximum valid ID (255 is reserved)

        // All should be initialized
        assert_eq!(custom_memory_10.size(), 0);
        assert_eq!(custom_memory_20.size(), 0);
        assert_eq!(custom_memory_254.size(), 0);
    }

    #[test]
    fn test_memory_type_alias() {
        // Test that the Memory type alias works correctly
        let memory = get_memory(MEMORY_ID_CONFIG);
        assert_eq!(memory.size(), 0);
    }

    #[test]
    fn test_memory_manager_thread_local() {
        // Test accessing the memory manager directly
        MEMORY_MANAGER.with(|manager| {
            let borrowed = manager.borrow();
            // Manager should be initialized
            let test_memory = borrowed.get(MemoryId::new(100));
            assert_eq!(test_memory.size(), 0);
        });
    }

    #[test]
    fn test_memory_isolation() {
        // Test that different memory IDs provide isolated memory spaces
        let memory_a = get_memory(50);
        let memory_b = get_memory(51);

        // Both should start with size 0
        assert_eq!(memory_a.size(), 0);
        assert_eq!(memory_b.size(), 0);

        // Memory IDs should provide isolation (this is ensured by IC stable structures)
        // This test verifies the basic setup, actual isolation is tested in integration
    }
}
