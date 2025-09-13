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
