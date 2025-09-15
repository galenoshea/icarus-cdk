//! Extension traits for stable structures
//!
//! These traits provide convenience methods for common operations on stable structures,
//! making them easier to use while maintaining full access to the underlying API.

use ic_stable_structures::cell::ValueError;
use ic_stable_structures::{StableBTreeMap, StableCell};
use std::cell::RefCell;

/// Extension trait for StableBTreeMap in thread-local storage
pub trait StableBTreeMapExt<K, V> {
    /// Insert a key-value pair
    fn insert(&self, key: K, value: V) -> Option<V>;

    /// Get a value by key
    fn get(&self, key: &K) -> Option<V>;

    /// Remove a value by key
    fn remove(&self, key: &K) -> Option<V>;

    /// Check if a key exists
    fn contains_key(&self, key: &K) -> bool;

    /// Get the number of entries
    fn len(&self) -> u64;

    /// Check if the map is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over all entries
    fn iter(&self) -> Vec<(K, V)>;

    /// Clear all entries
    fn clear(&self);
}

/// Extension trait for StableCell in thread-local storage
pub trait StableCellExt<T> {
    /// Get the current value
    fn get(&self) -> T;

    /// Set a new value
    fn set(&self, value: T) -> Result<T, ValueError>;

    /// Update the value with a closure
    fn update<F>(&self, f: F) -> T
    where
        F: FnOnce(&mut T);
}

// Implementation for thread-local StableBTreeMap
impl<K, V, M> StableBTreeMapExt<K, V> for RefCell<StableBTreeMap<K, V, M>>
where
    K: ic_stable_structures::Storable + Clone + Ord,
    V: ic_stable_structures::Storable + Clone,
    M: ic_stable_structures::Memory,
{
    fn insert(&self, key: K, value: V) -> Option<V> {
        self.borrow_mut().insert(key, value)
    }

    fn get(&self, key: &K) -> Option<V> {
        self.borrow().get(key)
    }

    fn remove(&self, key: &K) -> Option<V> {
        self.borrow_mut().remove(key)
    }

    fn contains_key(&self, key: &K) -> bool {
        self.borrow().contains_key(key)
    }

    fn len(&self) -> u64 {
        self.borrow().len()
    }

    fn iter(&self) -> Vec<(K, V)> {
        self.borrow()
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    fn clear(&self) {
        let keys: Vec<K> = self
            .borrow()
            .iter()
            .map(|entry| entry.key().clone())
            .collect();
        for key in keys {
            self.borrow_mut().remove(&key);
        }
    }
}

// Implementation for thread-local StableCell
impl<T, M> StableCellExt<T> for RefCell<StableCell<T, M>>
where
    T: ic_stable_structures::Storable + Clone,
    M: ic_stable_structures::Memory,
{
    fn get(&self) -> T {
        self.borrow().get().clone()
    }

    fn set(&self, value: T) -> Result<T, ValueError> {
        Ok(self.borrow_mut().set(value))
    }

    fn update<F>(&self, f: F) -> T
    where
        F: FnOnce(&mut T),
    {
        let mut value = self.get();
        f(&mut value);
        let _ = self.set(value.clone());
        value
    }
}

/// Helper macro to work with thread-local stable storage
///
/// This macro provides a convenient way to access thread-local stable storage
/// without having to write the `.with(|x| x.method())` pattern every time.
///
/// # Example
/// ```ignore
/// // This example requires IC stable structures and macros
/// stable_storage! {
///     MEMORIES: StableBTreeMap<String, Data> = StableBTreeMap::new();
/// }
///
/// // Instead of:
/// MEMORIES.with(|m| m.borrow_mut().insert("key".to_string(), data));
///
/// // You can write:
/// stable_with!(MEMORIES.insert("key".to_string(), data));
/// ```
#[macro_export]
macro_rules! stable_with {
    ($storage:ident.$method:ident($($args:expr),*)) => {
        $storage.with(|s| s.$method($($args),*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::get_memory;
    use ic_stable_structures::{StableBTreeMap, StableCell, Storable};
    use std::borrow::Cow;
    use std::cell::RefCell;

    // Test types for extension traits
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

        const BOUND: ic_stable_structures::storable::Bound =
            ic_stable_structures::storable::Bound::Unbounded;
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TestValue {
        data: String,
        number: u32,
    }

    impl Storable for TestValue {
        fn to_bytes(&self) -> Cow<[u8]> {
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
            let data_len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
            let data = String::from_utf8(bytes[4..4 + data_len].to_vec()).unwrap();
            let number_start = 4 + data_len;
            let number = u32::from_le_bytes([
                bytes[number_start],
                bytes[number_start + 1],
                bytes[number_start + 2],
                bytes[number_start + 3],
            ]);
            TestValue { data, number }
        }

        fn into_bytes(self) -> Vec<u8> {
            let data_bytes = self.data.as_bytes();
            let data_len = data_bytes.len() as u32;
            let mut result = Vec::new();
            result.extend_from_slice(&data_len.to_le_bytes());
            result.extend_from_slice(data_bytes);
            result.extend_from_slice(&self.number.to_le_bytes());
            result
        }

        const BOUND: ic_stable_structures::storable::Bound =
            ic_stable_structures::storable::Bound::Unbounded;
    }

    #[derive(Debug, Clone, PartialEq, Default)]
    struct TestCellValue {
        value: String,
        count: u32,
    }

    impl Storable for TestCellValue {
        fn to_bytes(&self) -> Cow<[u8]> {
            let value_bytes = self.value.as_bytes();
            let value_len = value_bytes.len() as u32;
            let mut result = Vec::new();
            result.extend_from_slice(&value_len.to_le_bytes());
            result.extend_from_slice(value_bytes);
            result.extend_from_slice(&self.count.to_le_bytes());
            Cow::Owned(result)
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            let bytes = bytes.as_ref();
            if bytes.len() < 8 {
                return TestCellValue::default();
            }
            let value_len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
            if bytes.len() < 4 + value_len + 4 {
                return TestCellValue::default();
            }
            let value = String::from_utf8(bytes[4..4 + value_len].to_vec()).unwrap_or_default();
            let count_start = 4 + value_len;
            let count = u32::from_le_bytes([
                bytes[count_start],
                bytes[count_start + 1],
                bytes[count_start + 2],
                bytes[count_start + 3],
            ]);
            TestCellValue { value, count }
        }

        fn into_bytes(self) -> Vec<u8> {
            let value_bytes = self.value.as_bytes();
            let value_len = value_bytes.len() as u32;
            let mut result = Vec::new();
            result.extend_from_slice(&value_len.to_le_bytes());
            result.extend_from_slice(value_bytes);
            result.extend_from_slice(&self.count.to_le_bytes());
            result
        }

        const BOUND: ic_stable_structures::storable::Bound =
            ic_stable_structures::storable::Bound::Unbounded;
    }

    // StableBTreeMapExt tests
    #[test]
    fn test_stable_btree_map_ext_insert_and_get() {
        let map = RefCell::new(StableBTreeMap::init(get_memory(70)));
        let key = TestKey("test_key".to_string());
        let value = TestValue {
            data: "test_data".to_string(),
            number: 42,
        };

        // Test insert
        let old_value = map.insert(key.clone(), value.clone());
        assert!(old_value.is_none());

        // Test get
        let retrieved = map.get(&key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), value);
    }

    #[test]
    fn test_stable_btree_map_ext_contains_key() {
        let map = RefCell::new(StableBTreeMap::init(get_memory(71)));
        let key = TestKey("test_key".to_string());
        let value = TestValue {
            data: "test_data".to_string(),
            number: 42,
        };

        // Key should not exist initially
        assert!(!map.contains_key(&key));

        // Insert and check again
        map.insert(key.clone(), value);
        assert!(map.contains_key(&key));
    }

    #[test]
    fn test_stable_btree_map_ext_remove() {
        let map = RefCell::new(StableBTreeMap::init(get_memory(72)));
        let key = TestKey("test_key".to_string());
        let value = TestValue {
            data: "test_data".to_string(),
            number: 42,
        };

        // Insert first
        map.insert(key.clone(), value.clone());
        assert_eq!(map.len(), 1);

        // Remove and verify
        let removed = map.remove(&key);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), value);
        assert_eq!(map.len(), 0);
        assert!(!map.contains_key(&key));
    }

    #[test]
    fn test_stable_btree_map_ext_len_and_is_empty() {
        let map = RefCell::new(StableBTreeMap::init(get_memory(73)));

        // Initially empty
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());

        // Add items
        for i in 0..3 {
            let key = TestKey(format!("key_{}", i));
            let value = TestValue {
                data: format!("data_{}", i),
                number: i,
            };
            map.insert(key, value);
        }

        assert_eq!(map.len(), 3);
        assert!(!map.is_empty());
    }

    #[test]
    fn test_stable_btree_map_ext_iter() {
        let map = RefCell::new(StableBTreeMap::init(get_memory(74)));

        // Insert test data
        for i in 0..3 {
            let key = TestKey(format!("key_{}", i));
            let value = TestValue {
                data: format!("data_{}", i),
                number: i,
            };
            map.insert(key, value);
        }

        // Test iteration
        let items = map.iter();
        assert_eq!(items.len(), 3);

        // Verify all items are present (order may vary due to BTreeMap)
        let mut found_numbers = Vec::new();
        for (key, value) in items {
            found_numbers.push(value.number);
            assert!(key.0.starts_with("key_"));
            assert!(value.data.starts_with("data_"));
        }
        found_numbers.sort();
        assert_eq!(found_numbers, vec![0, 1, 2]);
    }

    #[test]
    fn test_stable_btree_map_ext_clear() {
        let map = RefCell::new(StableBTreeMap::init(get_memory(75)));

        // Insert multiple items
        for i in 0..5 {
            let key = TestKey(format!("key_{}", i));
            let value = TestValue {
                data: format!("data_{}", i),
                number: i,
            };
            map.insert(key, value);
        }

        assert_eq!(map.len(), 5);

        // Clear and verify
        map.clear();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());

        // Verify all items are gone
        for i in 0..5 {
            let key = TestKey(format!("key_{}", i));
            assert!(!map.contains_key(&key));
        }
    }

    // StableCellExt tests
    #[test]
    fn test_stable_cell_ext_get_and_set() {
        let cell = RefCell::new(StableCell::init(get_memory(80), TestCellValue::default()));

        // Initial value should be default
        let initial_value = cell.get();
        assert_eq!(initial_value, TestCellValue::default());

        // Set a new value
        let test_value = TestCellValue {
            value: "test_string".to_string(),
            count: 42,
        };

        let result = cell.set(test_value.clone());
        assert!(result.is_ok());

        // Get and verify
        let retrieved = cell.get();
        assert_eq!(retrieved, test_value);
    }

    #[test]
    fn test_stable_cell_ext_update() {
        let cell = RefCell::new(StableCell::init(
            get_memory(81),
            TestCellValue {
                value: "initial".to_string(),
                count: 0,
            },
        ));

        // Update the value using closure
        let updated_value = cell.update(|val| {
            val.value = "updated".to_string();
            val.count += 10;
        });

        assert_eq!(updated_value.value, "updated");
        assert_eq!(updated_value.count, 10);

        // Verify the cell was actually updated
        let stored_value = cell.get();
        assert_eq!(stored_value, updated_value);
    }

    #[test]
    fn test_stable_cell_ext_update_multiple_times() {
        let cell = RefCell::new(StableCell::init(
            get_memory(82),
            TestCellValue {
                value: "start".to_string(),
                count: 1,
            },
        ));

        // First update
        cell.update(|val| {
            val.count *= 2;
        });

        // Second update
        let final_value = cell.update(|val| {
            val.value = "final".to_string();
            val.count += 5;
        });

        assert_eq!(final_value.value, "final");
        assert_eq!(final_value.count, 7); // (1 * 2) + 5

        // Verify persistence
        let stored_value = cell.get();
        assert_eq!(stored_value, final_value);
    }

    #[test]
    fn test_stable_cell_ext_set_returns_old_value() {
        let initial_value = TestCellValue {
            value: "original".to_string(),
            count: 100,
        };
        let cell = RefCell::new(StableCell::init(get_memory(83), initial_value.clone()));

        let new_value = TestCellValue {
            value: "new".to_string(),
            count: 200,
        };

        // Set should return the old value
        let old_value = cell.set(new_value.clone()).unwrap();
        assert_eq!(old_value, initial_value);

        // Verify new value is stored
        let stored_value = cell.get();
        assert_eq!(stored_value, new_value);
    }

    #[test]
    fn test_extension_trait_independence() {
        // Test that different maps and cells are independent
        let map1 = RefCell::new(StableBTreeMap::init(get_memory(90)));
        let map2 = RefCell::new(StableBTreeMap::init(get_memory(91)));
        let cell1 = RefCell::new(StableCell::init(get_memory(92), TestCellValue::default()));
        let cell2 = RefCell::new(StableCell::init(get_memory(93), TestCellValue::default()));

        // Operate on different instances
        let key = TestKey("same_key".to_string());
        let value1 = TestValue {
            data: "map1".to_string(),
            number: 1,
        };
        let value2 = TestValue {
            data: "map2".to_string(),
            number: 2,
        };

        map1.insert(key.clone(), value1.clone());
        map2.insert(key.clone(), value2.clone());

        // Maps should be independent
        assert_eq!(map1.get(&key).unwrap(), value1);
        assert_eq!(map2.get(&key).unwrap(), value2);
        assert_eq!(map1.len(), 1);
        assert_eq!(map2.len(), 1);

        // Cells should be independent
        let cell_value1 = TestCellValue {
            value: "cell1".to_string(),
            count: 10,
        };
        let cell_value2 = TestCellValue {
            value: "cell2".to_string(),
            count: 20,
        };

        cell1.set(cell_value1.clone()).unwrap();
        cell2.set(cell_value2.clone()).unwrap();

        assert_eq!(cell1.get(), cell_value1);
        assert_eq!(cell2.get(), cell_value2);
    }

    #[test]
    fn test_extension_trait_overwrite_behavior() {
        let map = RefCell::new(StableBTreeMap::init(get_memory(95)));
        let key = TestKey("test_key".to_string());

        let value1 = TestValue {
            data: "first".to_string(),
            number: 1,
        };
        let value2 = TestValue {
            data: "second".to_string(),
            number: 2,
        };

        // First insert should return None
        let old_value1 = map.insert(key.clone(), value1.clone());
        assert!(old_value1.is_none());

        // Second insert should return the old value
        let old_value2 = map.insert(key.clone(), value2.clone());
        assert!(old_value2.is_some());
        assert_eq!(old_value2.unwrap(), value1);

        // Map should contain the new value
        assert_eq!(map.get(&key).unwrap(), value2);
        assert_eq!(map.len(), 1); // Still only one entry
    }

    #[test]
    fn test_extension_trait_large_dataset() {
        let map = RefCell::new(StableBTreeMap::init(get_memory(96)));
        let count = 100;

        // Insert large dataset
        for i in 0..count {
            let key = TestKey(format!("key_{:03}", i));
            let value = TestValue {
                data: format!("data_{}", i),
                number: i,
            };
            map.insert(key, value);
        }

        assert_eq!(map.len(), count as u64);
        assert!(!map.is_empty());

        // Test random access
        let mid_key = TestKey(format!("key_{:03}", count / 2));
        let mid_value = map.get(&mid_key).unwrap();
        assert_eq!(mid_value.number, count / 2);

        // Test iteration over large dataset
        let all_items = map.iter();
        assert_eq!(all_items.len(), count as usize);

        // Verify all items are present
        let mut found_numbers: Vec<u32> = all_items.into_iter().map(|(_, v)| v.number).collect();
        found_numbers.sort();
        let expected_numbers: Vec<u32> = (0..count).collect();
        assert_eq!(found_numbers, expected_numbers);
    }
}
