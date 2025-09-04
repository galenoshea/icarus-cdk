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
        self.borrow().iter().collect()
    }

    fn clear(&self) {
        let keys: Vec<K> = self.borrow().iter().map(|(k, _)| k).collect();
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
        self.borrow_mut().set(value)
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
