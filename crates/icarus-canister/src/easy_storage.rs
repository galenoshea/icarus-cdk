//! Simplified storage patterns for common use cases

use crate::memory::get_memory;
use crate::result::IcarusResult;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::{StableBTreeMap, StableCell, Storable};
use std::cell::RefCell;

type Memory = VirtualMemory<DefaultMemoryImpl>;

/// A simplified storage pattern that reduces boilerplate
///
/// # Example
/// ```ignore
/// // This macro requires IC canister context and stable structures
/// use icarus_canister::prelude::*;
///
/// // Define your storage
/// icarus_storage! {
///     USERS: Map<String, User> = 0;
///     COUNTER: Cell<u64> = 1;
/// }
///
/// // Use it directly
/// fn add_user(id: String, user: User) -> IcarusResult<()> {
///     USERS.insert(id, user)?;
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! icarus_storage {
    (
        $($name:ident: Map<$key:ty, $value:ty> = $id:expr;)*
        $($counter:ident: Cell<$ctype:ty> = $cid:expr;)*
    ) => {
        thread_local! {
            $(
                static $name: $crate::easy_storage::StorageMap<$key, $value> =
                    $crate::easy_storage::StorageMap::new($id);
            )*
            $(
                static $counter: $crate::easy_storage::CounterCell =
                    $crate::easy_storage::CounterCell::new($cid);
            )*
        }

        // Generate convenience accessor structs
        $(
            #[allow(non_camel_case_types)]
            pub struct $name;

            impl $name {
                pub fn insert(key: $key, value: $value) -> $crate::result::IcarusResult<Option<$value>> {
                    $name.with(|s| s.insert(key, value))
                }

                pub fn get(key: &$key) -> Option<$value> {
                    $name.with(|s| s.get(key))
                }

                pub fn remove(key: &$key) -> Option<$value> {
                    $name.with(|s| s.remove(key))
                }

                pub fn contains(key: &$key) -> bool {
                    $name.with(|s| s.contains(key))
                }

                pub fn len() -> u64 {
                    $name.with(|s| s.len())
                }

                pub fn is_empty() -> bool {
                    $name.with(|s| s.is_empty())
                }

                pub fn clear() {
                    $name.with(|s| s.clear())
                }

                pub fn iter<F>(f: F)
                where
                    F: FnMut(&$key, &$value)
                {
                    $name.with(|s| s.iter(f))
                }

                pub fn values() -> Vec<$value> {
                    $name.with(|s| s.values())
                }
            }
        )*

        $(
            #[allow(non_camel_case_types)]
            pub struct $counter;

            impl $counter {
                pub fn get() -> $ctype {
                    $counter.with(|c| c.get())
                }

                pub fn set(value: $ctype) -> $crate::result::IcarusResult<()> {
                    $counter.with(|c| c.set(value))
                }

                pub fn increment() -> $ctype {
                    $counter.with(|c| c.increment())
                }

                pub fn decrement() -> $ctype {
                    $counter.with(|c| c.decrement())
                }
            }
        )*
    };
}

/// A thread-safe storage map with automatic memory management
pub struct StorageMap<K, V>
where
    K: Storable + Ord + Clone,
    V: Storable + Clone,
{
    inner: RefCell<StableBTreeMap<K, V, Memory>>,
}

impl<K, V> StorageMap<K, V>
where
    K: Storable + Ord + Clone,
    V: Storable + Clone,
{
    pub fn new(memory_id: u8) -> Self {
        Self {
            inner: RefCell::new(StableBTreeMap::init(get_memory(memory_id))),
        }
    }

    pub fn insert(&self, key: K, value: V) -> IcarusResult<Option<V>> {
        Ok(self.inner.borrow_mut().insert(key, value))
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.borrow().get(key)
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.inner.borrow_mut().remove(key)
    }

    pub fn contains(&self, key: &K) -> bool {
        self.inner.borrow().contains_key(key)
    }

    pub fn len(&self) -> u64 {
        self.inner.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }

    pub fn clear(&self) {
        self.inner.borrow_mut().clear_new()
    }

    pub fn iter<F>(&self, mut f: F)
    where
        F: FnMut(&K, &V),
    {
        for entry in self.inner.borrow().iter() {
            f(entry.key(), &entry.value());
        }
    }

    pub fn values(&self) -> Vec<V> {
        self.inner
            .borrow()
            .iter()
            .map(|entry| entry.value())
            .collect()
    }

    pub fn get_or_insert<F>(&self, key: K, f: F) -> V
    where
        F: FnOnce() -> V,
    {
        if let Some(v) = self.get(&key) {
            v
        } else {
            let value = f();
            self.inner.borrow_mut().insert(key, value.clone());
            value
        }
    }
}

/// A thread-safe storage cell for single values
pub struct StorageCell<T>
where
    T: Storable + Default + Clone,
{
    inner: RefCell<StableCell<T, Memory>>,
}

impl<T> StorageCell<T>
where
    T: Storable + Default + Clone,
{
    pub fn new(memory_id: u8) -> Self {
        Self {
            inner: RefCell::new(StableCell::init(get_memory(memory_id), T::default())),
        }
    }

    pub fn get(&self) -> T {
        self.inner.borrow().get().clone()
    }

    pub fn set(&self, value: T) -> IcarusResult<()> {
        let _old_value = self.inner.borrow_mut().set(value);
        Ok(())
    }
}

// Specialized wrapper for u64 counters with increment/decrement
pub struct CounterCell {
    inner: RefCell<StableCell<u64, Memory>>,
}

impl CounterCell {
    pub fn new(memory_id: u8) -> Self {
        Self {
            inner: RefCell::new(StableCell::init(get_memory(memory_id), 0u64)),
        }
    }

    pub fn get(&self) -> u64 {
        *self.inner.borrow().get()
    }

    pub fn set(&self, value: u64) -> IcarusResult<()> {
        let _old_value = self.inner.borrow_mut().set(value);
        Ok(())
    }

    pub fn increment(&self) -> u64 {
        let current = self.get();
        let next = current + 1;
        let _ = self.set(next);
        next
    }

    pub fn decrement(&self) -> u64 {
        let current = self.get();
        let next = current.saturating_sub(1);
        let _ = self.set(next);
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_stable_structures::Storable;
    use std::borrow::Cow;

    // Test types for StorageMap
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
            let data = String::from_utf8(bytes[4..4+data_len].to_vec()).unwrap();
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

    // Test type for StorageCell
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
            let value = String::from_utf8(bytes[4..4+value_len].to_vec()).unwrap_or_default();
            let count_start = 4 + value_len;
            let count = u32::from_le_bytes([
                bytes[count_start],
                bytes[count_start + 1],
                bytes[count_start + 2],
                bytes[count_start + 3]
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

        const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
    }

    // StorageMap tests
    #[test]
    fn test_storage_map_new() {
        let map = StorageMap::<TestKey, TestValue>::new(30);
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_storage_map_insert_and_get() {
        let map = StorageMap::new(31);
        let key = TestKey("test_key".to_string());
        let value = TestValue {
            data: "test_data".to_string(),
            number: 42,
        };

        let result = map.insert(key.clone(), value.clone());
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        let retrieved = map.get(&key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), value);

        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_storage_map_contains() {
        let map = StorageMap::new(32);
        let key = TestKey("test_key".to_string());
        let value = TestValue {
            data: "test_data".to_string(),
            number: 42,
        };

        assert!(!map.contains(&key));

        map.insert(key.clone(), value).unwrap();
        assert!(map.contains(&key));
    }

    #[test]
    fn test_storage_map_remove() {
        let map = StorageMap::new(33);
        let key = TestKey("test_key".to_string());
        let value = TestValue {
            data: "test_data".to_string(),
            number: 42,
        };

        map.insert(key.clone(), value.clone()).unwrap();
        assert_eq!(map.len(), 1);

        let removed = map.remove(&key);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), value);

        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert!(!map.contains(&key));
    }

    #[test]
    fn test_storage_map_clear() {
        let map = StorageMap::new(34);

        // Insert multiple items
        for i in 0..5 {
            let key = TestKey(format!("key_{}", i));
            let value = TestValue {
                data: format!("data_{}", i),
                number: i,
            };
            map.insert(key, value).unwrap();
        }

        assert_eq!(map.len(), 5);

        map.clear();

        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_storage_map_iter() {
        let map = StorageMap::new(35);

        // Insert test data
        for i in 0..3 {
            let key = TestKey(format!("key_{}", i));
            let value = TestValue {
                data: format!("data_{}", i),
                number: i,
            };
            map.insert(key, value).unwrap();
        }

        let mut collected = Vec::new();
        map.iter(|key, value| {
            collected.push((key.clone(), value.clone()));
        });

        assert_eq!(collected.len(), 3);
        collected.sort_by(|a, b| a.1.number.cmp(&b.1.number));

        for (i, (key, value)) in collected.iter().enumerate() {
            assert_eq!(key.0, format!("key_{}", i));
            assert_eq!(value.number, i as u32);
        }
    }

    #[test]
    fn test_storage_map_values() {
        let map = StorageMap::new(36);
        let mut expected_values = Vec::new();

        // Insert test data
        for i in 0..3 {
            let key = TestKey(format!("key_{}", i));
            let value = TestValue {
                data: format!("data_{}", i),
                number: i,
            };
            expected_values.push(value.clone());
            map.insert(key, value).unwrap();
        }

        let mut values = map.values();
        values.sort_by(|a, b| a.number.cmp(&b.number));
        expected_values.sort_by(|a, b| a.number.cmp(&b.number));

        assert_eq!(values, expected_values);
    }

    #[test]
    fn test_storage_map_get_or_insert() {
        let map = StorageMap::new(37);
        let key = TestKey("test_key".to_string());
        let value = TestValue {
            data: "default_data".to_string(),
            number: 100,
        };

        // First call should insert and return the value
        let result = map.get_or_insert(key.clone(), || value.clone());
        assert_eq!(result, value);
        assert_eq!(map.len(), 1);

        // Second call should return existing value without calling closure
        let result2 = map.get_or_insert(key.clone(), || TestValue {
            data: "should_not_be_used".to_string(),
            number: 999,
        });
        assert_eq!(result2, value);
        assert_eq!(map.len(), 1);
    }

    // StorageCell tests
    #[test]
    fn test_storage_cell_new() {
        let cell = StorageCell::<TestCellValue>::new(40);
        let value = cell.get();
        assert_eq!(value, TestCellValue::default());
    }

    #[test]
    fn test_storage_cell_set_and_get() {
        let cell = StorageCell::new(41);
        let test_value = TestCellValue {
            value: "test_string".to_string(),
            count: 42,
        };

        let result = cell.set(test_value.clone());
        assert!(result.is_ok());

        let retrieved = cell.get();
        assert_eq!(retrieved, test_value);
    }

    #[test]
    fn test_storage_cell_overwrite() {
        let cell = StorageCell::new(42);
        let value1 = TestCellValue {
            value: "first".to_string(),
            count: 1,
        };
        let value2 = TestCellValue {
            value: "second".to_string(),
            count: 2,
        };

        cell.set(value1).unwrap();
        cell.set(value2.clone()).unwrap();

        let retrieved = cell.get();
        assert_eq!(retrieved, value2);
    }

    // CounterCell tests
    #[test]
    fn test_counter_cell_new() {
        let counter = CounterCell::new(50);
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_counter_cell_set_and_get() {
        let counter = CounterCell::new(51);

        let result = counter.set(42);
        assert!(result.is_ok());
        assert_eq!(counter.get(), 42);
    }

    #[test]
    fn test_counter_cell_increment() {
        let counter = CounterCell::new(52);

        // Start at 0
        assert_eq!(counter.get(), 0);

        // Increment multiple times
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.get(), 1);

        assert_eq!(counter.increment(), 2);
        assert_eq!(counter.get(), 2);

        assert_eq!(counter.increment(), 3);
        assert_eq!(counter.get(), 3);
    }

    #[test]
    fn test_counter_cell_decrement() {
        let counter = CounterCell::new(53);

        // Set initial value
        counter.set(5).unwrap();
        assert_eq!(counter.get(), 5);

        // Decrement multiple times
        assert_eq!(counter.decrement(), 4);
        assert_eq!(counter.get(), 4);

        assert_eq!(counter.decrement(), 3);
        assert_eq!(counter.get(), 3);
    }

    #[test]
    fn test_counter_cell_decrement_saturating() {
        let counter = CounterCell::new(54);

        // Start at 0
        assert_eq!(counter.get(), 0);

        // Decrement should saturate at 0
        assert_eq!(counter.decrement(), 0);
        assert_eq!(counter.get(), 0);

        // Multiple decrements should still be 0
        assert_eq!(counter.decrement(), 0);
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_counter_cell_increment_decrement_sequence() {
        let counter = CounterCell::new(55);

        // Mixed increment/decrement operations
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
        assert_eq!(counter.decrement(), 1);
        assert_eq!(counter.increment(), 2);
        assert_eq!(counter.decrement(), 1);
        assert_eq!(counter.decrement(), 0);
        assert_eq!(counter.decrement(), 0); // Should saturate
    }

    #[test]
    fn test_counter_cell_large_values() {
        let counter = CounterCell::new(56);

        let large_value = u64::MAX - 1;
        counter.set(large_value).unwrap();
        assert_eq!(counter.get(), large_value);

        // Test increment near max value
        assert_eq!(counter.increment(), u64::MAX);
        assert_eq!(counter.get(), u64::MAX);
    }

    #[test]
    fn test_multiple_storage_instances_independence() {
        let map1 = StorageMap::<TestKey, TestValue>::new(60);
        let map2 = StorageMap::<TestKey, TestValue>::new(61);
        let counter1 = CounterCell::new(62);
        let counter2 = CounterCell::new(63);

        let key = TestKey("same_key".to_string());
        let value1 = TestValue {
            data: "value1".to_string(),
            number: 1,
        };
        let value2 = TestValue {
            data: "value2".to_string(),
            number: 2,
        };

        // Test map independence
        map1.insert(key.clone(), value1.clone()).unwrap();
        map2.insert(key.clone(), value2.clone()).unwrap();

        assert_eq!(map1.get(&key).unwrap(), value1);
        assert_eq!(map2.get(&key).unwrap(), value2);

        // Test counter independence
        counter1.set(10).unwrap();
        counter2.set(20).unwrap();

        assert_eq!(counter1.get(), 10);
        assert_eq!(counter2.get(), 20);

        counter1.increment();
        assert_eq!(counter1.get(), 11);
        assert_eq!(counter2.get(), 20); // Should not change
    }
}
