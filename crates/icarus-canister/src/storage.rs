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
pub struct StableMap<K: Storable + Ord + Clone, V: Storable + Clone> {
    inner: RefCell<StableBTreeMap<K, V, VirtualMemory<DefaultMemoryImpl>>>,
}

impl<K: Storable + Ord + Clone, V: Storable + Clone> StableMap<K, V> {
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

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.inner.borrow_mut().clear_new()
    }

    /// Iterate over all entries
    pub fn iter<F>(&self, f: F)
    where
        F: FnMut(&K, &V),
    {
        let map = self.inner.borrow();
        let mut func = f;
        for entry in map.iter() {
            func(entry.key(), &entry.value());
        }
    }

    /// Collect all values into a Vec
    pub fn values(&self) -> Vec<V> {
        self.inner
            .borrow()
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Filter and collect values
    pub fn filter<F>(&self, predicate: F) -> Vec<V>
    where
        F: Fn(&K, &V) -> bool,
    {
        self.inner
            .borrow()
            .iter()
            .filter(|entry| predicate(entry.key(), &entry.value()))
            .map(|entry| entry.value().clone())
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

/// Simplified macro to declare stable storage  
///
/// This macro provides a cleaner syntax for declaring stable storage.
/// For now, it uses the existing init_memory! macro under the hood.
///
/// # Example
/// ```ignore
/// // This macro requires IC stable structures context
/// stable_storage! {
///     MEMORIES: StableBTreeMap<String, MemoryEntry> = memory_id!(0);
///     COUNTER: u64 = 0;
/// }
/// ```
#[macro_export]
macro_rules! stable_storage {
    (
        $($name:ident: $type:ty = $init:expr;)*
    ) => {
        $crate::init_memory! {
            $($name: $type = $init;)*
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_stable_structures::Storable;
    use std::borrow::Cow;

    // Test types that implement Storable
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct TestKey(String);

    impl Storable for TestKey {
        fn to_bytes(&self) -> Cow<[u8]> {
            Cow::Borrowed(self.0.as_bytes())
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            TestKey(String::from_utf8(bytes.into_owned()).unwrap())
        }

        fn into_bytes(self) -> Vec<u8> {
            self.0.into_bytes()
        }

        const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TestValue {
        data: String,
        number: u32,
    }

    impl Storable for TestValue {
        fn to_bytes(&self) -> Cow<[u8]> {
            // Use length-prefixed encoding to avoid delimiter conflicts
            let data_bytes = self.data.as_bytes();
            let data_len = data_bytes.len() as u32;
            let mut result = Vec::new();
            result.extend_from_slice(&data_len.to_le_bytes());
            result.extend_from_slice(data_bytes);
            result.extend_from_slice(&self.number.to_le_bytes());
            Cow::Owned(result)
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            let bytes = bytes.as_ref();
            if bytes.len() < 8 {
                panic!("Invalid TestValue format: too short");
            }

            // Read data length
            let data_len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
            if bytes.len() < 4 + data_len + 4 {
                panic!("Invalid TestValue format: inconsistent length");
            }

            // Read data
            let data = String::from_utf8(bytes[4..4+data_len].to_vec()).unwrap();

            // Read number
            let number_start = 4 + data_len;
            let number = u32::from_le_bytes([
                bytes[number_start],
                bytes[number_start + 1],
                bytes[number_start + 2],
                bytes[number_start + 3]
            ]);

            TestValue { data, number }
        }

        fn into_bytes(self) -> Vec<u8> {
            // Use length-prefixed encoding to avoid delimiter conflicts
            let data_bytes = self.data.as_bytes();
            let data_len = data_bytes.len() as u32;
            let mut result = Vec::new();
            result.extend_from_slice(&data_len.to_le_bytes());
            result.extend_from_slice(data_bytes);
            result.extend_from_slice(&self.number.to_le_bytes());
            result
        }

        const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
    }

    #[test]
    fn test_stable_map_new() {
        let map: StableMap<TestKey, TestValue> = StableMap::new(0);
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_stable_map_insert_and_get() {
        let map = StableMap::new(1);
        let key = TestKey("test_key".to_string());
        let value = TestValue {
            data: "test_data".to_string(),
            number: 42,
        };

        // Insert and verify return value
        let old_value = map.insert(key.clone(), value.clone());
        assert!(old_value.is_none());

        // Get and verify
        let retrieved = map.get(&key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), value);

        // Verify map is no longer empty
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_stable_map_insert_overwrite() {
        let map = StableMap::new(2);
        let key = TestKey("test_key".to_string());
        let value1 = TestValue {
            data: "first".to_string(),
            number: 1,
        };
        let value2 = TestValue {
            data: "second".to_string(),
            number: 2,
        };

        // Insert first value
        map.insert(key.clone(), value1.clone());

        // Insert second value with same key
        let old_value = map.insert(key.clone(), value2.clone());
        assert!(old_value.is_some());
        assert_eq!(old_value.unwrap(), value1);

        // Verify new value is stored
        let retrieved = map.get(&key);
        assert_eq!(retrieved.unwrap(), value2);

        // Verify map still has only one entry
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_stable_map_remove() {
        let map = StableMap::new(3);
        let key = TestKey("test_key".to_string());
        let value = TestValue {
            data: "test_data".to_string(),
            number: 42,
        };

        // Insert value
        map.insert(key.clone(), value.clone());
        assert_eq!(map.len(), 1);

        // Remove value
        let removed = map.remove(&key);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), value);

        // Verify map is empty
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);

        // Try to get removed value
        assert!(map.get(&key).is_none());
    }

    #[test]
    fn test_stable_map_remove_nonexistent() {
        let map: StableMap<TestKey, TestValue> = StableMap::new(4);
        let key = TestKey("nonexistent".to_string());

        // Try to remove nonexistent key
        let removed = map.remove(&key);
        assert!(removed.is_none());
    }

    #[test]
    fn test_stable_map_multiple_entries() {
        let map = StableMap::new(5);

        // Insert multiple entries
        for i in 0..5 {
            let key = TestKey(format!("key_{}", i));
            let value = TestValue {
                data: format!("data_{}", i),
                number: i,
            };
            map.insert(key, value);
        }

        // Verify all entries
        assert_eq!(map.len(), 5);
        assert!(!map.is_empty());

        // Test get for each entry
        for i in 0..5 {
            let key = TestKey(format!("key_{}", i));
            let retrieved = map.get(&key).unwrap();
            assert_eq!(retrieved.data, format!("data_{}", i));
            assert_eq!(retrieved.number, i);
        }
    }

    #[test]
    fn test_stable_map_clear() {
        let map = StableMap::new(6);

        // Insert some entries
        for i in 0..3 {
            let key = TestKey(format!("key_{}", i));
            let value = TestValue {
                data: format!("data_{}", i),
                number: i,
            };
            map.insert(key, value);
        }

        assert_eq!(map.len(), 3);

        // Clear the map
        map.clear();

        // Verify map is empty
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);

        // Verify entries are gone
        for i in 0..3 {
            let key = TestKey(format!("key_{}", i));
            assert!(map.get(&key).is_none());
        }
    }

    #[test]
    fn test_stable_map_values() {
        let map = StableMap::new(7);
        let mut expected_values = Vec::new();

        // Insert test data
        for i in 0..3 {
            let key = TestKey(format!("key_{}", i));
            let value = TestValue {
                data: format!("data_{}", i),
                number: i,
            };
            expected_values.push(value.clone());
            map.insert(key, value);
        }

        // Get all values
        let mut values = map.values();
        values.sort_by(|a, b| a.number.cmp(&b.number));
        expected_values.sort_by(|a, b| a.number.cmp(&b.number));

        assert_eq!(values, expected_values);
    }

    #[test]
    fn test_stable_map_filter() {
        let map = StableMap::new(8);

        // Insert test data
        for i in 0..5 {
            let key = TestKey(format!("key_{}", i));
            let value = TestValue {
                data: format!("data_{}", i),
                number: i,
            };
            map.insert(key, value);
        }

        // Filter for even numbers
        let even_values = map.filter(|_key, value| value.number % 2 == 0);
        assert_eq!(even_values.len(), 3); // 0, 2, 4

        // Filter for specific data pattern
        let filtered = map.filter(|_key, value| value.data.contains("data_1"));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].number, 1);
    }

    #[test]
    fn test_stable_map_iter() {
        let map = StableMap::new(9);

        // Insert test data
        for i in 0..3 {
            let key = TestKey(format!("key_{}", i));
            let value = TestValue {
                data: format!("data_{}", i),
                number: i,
            };
            map.insert(key, value);
        }

        // Collect via iterator
        let mut collected = Vec::new();
        map.iter(|key, value| {
            collected.push((key.clone(), value.clone()));
        });

        assert_eq!(collected.len(), 3);

        // Verify all entries were iterated
        collected.sort_by(|a, b| a.1.number.cmp(&b.1.number));
        for (i, (key, value)) in collected.iter().enumerate() {
            assert_eq!(key.0, format!("key_{}", i));
            assert_eq!(value.number, i as u32);
        }
    }

    #[test]
    fn test_stable_counter_new() {
        let counter = StableCounter::new(10);
        assert_eq!(counter.current(), 0);
    }

    #[test]
    fn test_stable_counter_next() {
        let counter = StableCounter::new(11);

        // Test initial next
        assert_eq!(counter.next(), 1);
        assert_eq!(counter.current(), 1);

        // Test subsequent next calls
        assert_eq!(counter.next(), 2);
        assert_eq!(counter.current(), 2);

        assert_eq!(counter.next(), 3);
        assert_eq!(counter.current(), 3);
    }

    #[test]
    fn test_stable_counter_current() {
        let counter = StableCounter::new(12);

        // Current should not increment
        assert_eq!(counter.current(), 0);
        assert_eq!(counter.current(), 0);
        assert_eq!(counter.current(), 0);

        // After next, current should reflect new value
        counter.next();
        assert_eq!(counter.current(), 1);
        assert_eq!(counter.current(), 1);
    }

    #[test]
    fn test_stable_counter_multiple_instances() {
        let counter1 = StableCounter::new(13);
        let counter2 = StableCounter::new(14);

        // Each counter should be independent
        assert_eq!(counter1.next(), 1);
        assert_eq!(counter2.next(), 1);

        assert_eq!(counter1.next(), 2);
        assert_eq!(counter2.current(), 1);

        assert_eq!(counter2.next(), 2);
        assert_eq!(counter1.current(), 2);
        assert_eq!(counter2.current(), 2);
    }

    #[test]
    fn test_stable_counter_large_numbers() {
        let counter = StableCounter::new(15);

        // Test with large increments
        for i in 1..=100 {
            assert_eq!(counter.next(), i);
        }

        assert_eq!(counter.current(), 100);

        // Continue incrementing
        assert_eq!(counter.next(), 101);
        assert_eq!(counter.current(), 101);
    }

    #[test]
    fn test_stable_map_edge_cases() {
        let map = StableMap::new(16);

        // Test with empty strings
        let empty_key = TestKey("".to_string());
        let empty_value = TestValue {
            data: "".to_string(),
            number: 0,
        };

        map.insert(empty_key.clone(), empty_value.clone());
        let retrieved = map.get(&empty_key);
        assert_eq!(retrieved.unwrap(), empty_value);

        // Test with special characters
        let special_key = TestKey("key with spaces and !@#$%".to_string());
        let special_value = TestValue {
            data: "special:chars:here".to_string(),
            number: u32::MAX,
        };

        map.insert(special_key.clone(), special_value.clone());
        let retrieved = map.get(&special_key);
        assert_eq!(retrieved.unwrap(), special_value);
    }

    #[test]
    fn test_stable_map_large_dataset() {
        let map = StableMap::new(17);
        let count = 1000;

        // Insert large dataset
        for i in 0..count {
            let key = TestKey(format!("large_key_{:04}", i));
            let value = TestValue {
                data: format!("large_data_entry_{}", i),
                number: i,
            };
            map.insert(key, value);
        }

        assert_eq!(map.len(), count as u64);

        // Verify random access
        let mid_key = TestKey(format!("large_key_{:04}", count / 2));
        let mid_value = map.get(&mid_key).unwrap();
        assert_eq!(mid_value.number, count / 2);

        // Test filter on large dataset
        let filtered = map.filter(|_key, value| value.number % 100 == 0);
        assert_eq!(filtered.len(), 10); // 0, 100, 200, ..., 900

        // Clear large dataset
        map.clear();
        assert!(map.is_empty());
    }

    #[test]
    fn test_memory_id_separation() {
        let map1 = StableMap::new(18);
        let map2 = StableMap::new(19);

        let key = TestKey("same_key".to_string());
        let value1 = TestValue {
            data: "value1".to_string(),
            number: 1,
        };
        let value2 = TestValue {
            data: "value2".to_string(),
            number: 2,
        };

        // Insert different values with same key in different maps
        map1.insert(key.clone(), value1.clone());
        map2.insert(key.clone(), value2.clone());

        // Verify maps are independent
        assert_eq!(map1.get(&key).unwrap(), value1);
        assert_eq!(map2.get(&key).unwrap(), value2);

        // Remove from one map shouldn't affect the other
        map1.remove(&key);
        assert!(map1.get(&key).is_none());
        assert_eq!(map2.get(&key).unwrap(), value2);
    }
}
