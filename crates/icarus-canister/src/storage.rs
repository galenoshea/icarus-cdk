//! Simplified stable storage utilities

use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, Storable,
};
use std::cell::RefCell;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
}

/// A stable map that automatically manages its memory
pub struct StableMap<K: Storable + Ord + Clone, V: Storable> {
    inner: RefCell<StableBTreeMap<K, V, VirtualMemory<DefaultMemoryImpl>>>,
}

impl<K: Storable + Ord + Clone, V: Storable> StableMap<K, V> {
    /// Create a new stable map with the given memory ID
    pub fn new(memory_id: u8) -> Self {
        let memory = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(memory_id)));
        Self {
            inner: RefCell::new(StableBTreeMap::init(memory)),
        }
    }
    
    /// Insert a key-value pair
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.inner.borrow_mut().insert(key, value)
    }
    
    /// Get a value by key
    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.borrow().get(key)
    }
    
    /// Remove a value by key
    pub fn remove(&self, key: &K) -> Option<V> {
        self.inner.borrow_mut().remove(key)
    }
    
    /// Get the number of entries
    pub fn len(&self) -> u64 {
        self.inner.borrow().len()
    }
    
    /// Clear all entries
    pub fn clear(&self) {
        self.inner.borrow_mut().clear_new()
    }
    
    /// Iterate over all entries
    pub fn iter<F>(&self, f: F) 
    where 
        F: FnMut(&K, &V)
    {
        let map = self.inner.borrow();
        let mut func = f;
        for (k, v) in map.iter() {
            func(&k, &v);
        }
    }
    
    /// Collect all values into a Vec
    pub fn values(&self) -> Vec<V> {
        self.inner.borrow().iter().map(|(_, v)| v).collect()
    }
    
    /// Filter and collect values
    pub fn filter<F>(&self, predicate: F) -> Vec<V>
    where
        F: Fn(&K, &V) -> bool
    {
        self.inner.borrow()
            .iter()
            .filter(|(k, v)| predicate(k, v))
            .map(|(_, v)| v)
            .collect()
    }
}

/// Helper for persistent counters
pub struct StableCounter {
    map: StableMap<String, u64>,
}

impl StableCounter {
    pub fn new(memory_id: u8) -> Self {
        Self {
            map: StableMap::new(memory_id),
        }
    }
    
    /// Get the next value and increment
    pub fn next(&self) -> u64 {
        let current = self.map.get(&"counter".to_string()).unwrap_or(0);
        let next = current + 1;
        self.map.insert("counter".to_string(), next);
        next
    }
    
    /// Get current value without incrementing
    pub fn current(&self) -> u64 {
        self.map.get(&"counter".to_string()).unwrap_or(0)
    }
}

/// Macro to declare stable storage
#[macro_export]
macro_rules! stable_storage {
    (
        $($name:ident: StableMap<$key:ty, $value:ty> = $memory_id:expr),* $(,)?
    ) => {
        thread_local! {
            $(
                static $name: $crate::storage::StableMap<$key, $value> = 
                    $crate::storage::StableMap::new($memory_id);
            )*
        }
    };
}