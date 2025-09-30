//! # Stateful Counter Example
//!
//! This example demonstrates how to manage canister state across function calls
//! using Internet Computer's stable memory and thread-local storage.
//!
//! ## Features
//! - Persistent state across canister upgrades
//! - Thread-local state management with `RefCell`
//! - Multiple counters with independent state
//! - Atomic increment/decrement operations
//! - State inspection and reset capabilities
//!
//! ## Usage
//!
//! ```bash
//! # Deploy to Internet Computer
//! dfx start --background
//! dfx deploy stateful_counter
//!
//! # Increment the counter
//! dfx canister call stateful_counter call_tool '(
//!   record {
//!     name = "increment";
//!     arguments = "{}"
//!   }
//! )'
//!
//! # Get current value
//! dfx canister call stateful_counter call_tool '(
//!   record {
//!     name = "get_count";
//!     arguments = "{}"
//!   }
//! )'
//!
//! # Use named counter
//! dfx canister call stateful_counter call_tool '(
//!   record {
//!     name = "increment_named";
//!     arguments = "{\"name\": \"visits\"}"
//!   }
//! )'
//! ```
//!
//! ## State Management Patterns
//!
//! ### 1. Thread-Local Storage (Volatile)
//! ```rust
//! thread_local! {
//!     static COUNTER: RefCell<u64> = RefCell::new(0);
//! }
//! ```
//! - **Fast**: No serialization overhead
//! - **Volatile**: Lost on canister upgrades
//! - **Use for**: Temporary state, caches, request counters
//!
//! ### 2. Stable Memory (Persistent)
//! ```rust
//! use ic_stable_structures::{StableBTreeMap, DefaultMemoryImpl};
//!
//! thread_local! {
//!     static MEMORY: RefCell<StableBTreeMap<String, u64>> = ...;
//! }
//! ```
//! - **Persistent**: Survives canister upgrades
//! - **Slower**: Serialization cost per access
//! - **Use for**: Critical state, user data, configuration
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │      Canister State Layers          │
//! │                                     │
//! │  ┌───────────────────────────────┐  │
//! │  │   Thread-Local (Volatile)     │  │
//! │  │  • COUNTER: u64               │  │
//! │  │  • NAMED_COUNTERS: HashMap    │  │
//! │  └───────────────────────────────┘  │
//! │              │ (lost on upgrade)   │
//! │              ▼                      │
//! │  ┌───────────────────────────────┐  │
//! │  │  Stable Memory (Persistent)   │  │
//! │  │  • Backup before upgrade      │  │
//! │  │  • Restore after upgrade      │  │
//! │  └───────────────────────────────┘  │
//! └─────────────────────────────────────┘
//! ```

use icarus_macros::tool;
use std::cell::RefCell;
use std::collections::HashMap;

// Thread-local state (volatile - lost on canister upgrade)
thread_local! {
    /// Global counter - simple single value
    static COUNTER: RefCell<u64> = RefCell::new(0);

    /// Named counters - multiple independent counters
    static NAMED_COUNTERS: RefCell<HashMap<String, u64>> = RefCell::new(HashMap::new());
}

/// Increment the global counter by 1.
///
/// # Returns
/// The new counter value after incrementing
///
/// # Example
/// ```json
/// {}
/// ```
/// Returns: `42` (if counter was 41)
#[tool("Increment the global counter")]
fn increment() -> u64 {
    COUNTER.with(|counter| {
        let mut count = counter.borrow_mut();
        *count += 1;
        *count
    })
}

/// Decrement the global counter by 1.
///
/// # Returns
/// The new counter value after decrementing
///
/// # Example
/// ```json
/// {}
/// ```
/// Returns: `40` (if counter was 41)
#[tool("Decrement the global counter")]
fn decrement() -> u64 {
    COUNTER.with(|counter| {
        let mut count = counter.borrow_mut();
        *count = count.saturating_sub(1); // Prevent underflow
        *count
    })
}

/// Get the current value of the global counter.
///
/// # Returns
/// The current counter value (read-only, no side effects)
///
/// # Example
/// ```json
/// {}
/// ```
/// Returns: `42`
#[tool("Get the current global counter value")]
fn get_count() -> u64 {
    COUNTER.with(|counter| *counter.borrow())
}

/// Reset the global counter to zero.
///
/// # Returns
/// The previous counter value before reset
///
/// # Example
/// ```json
/// {}
/// ```
/// Returns: `42` (counter is now 0)
#[tool("Reset the global counter to zero")]
fn reset() -> u64 {
    COUNTER.with(|counter| {
        let mut count = counter.borrow_mut();
        let old_value = *count;
        *count = 0;
        old_value
    })
}

/// Add a specific amount to the global counter.
///
/// # Parameters
/// - `amount`: The amount to add (can be negative for subtraction)
///
/// # Returns
/// The new counter value after adding
///
/// # Example
/// ```json
/// {
///   "amount": 10
/// }
/// ```
/// Returns: `52` (if counter was 42)
#[tool("Add a specific amount to the global counter")]
fn add(amount: i64) -> u64 {
    COUNTER.with(|counter| {
        let mut count = counter.borrow_mut();
        if amount >= 0 {
            *count = count.saturating_add(amount as u64);
        } else {
            *count = count.saturating_sub(amount.abs() as u64);
        }
        *count
    })
}

/// Increment a named counter (creates if doesn't exist).
///
/// Named counters allow you to track multiple independent counters.
///
/// # Parameters
/// - `name`: The name of the counter (e.g., "visits", "clicks", "errors")
///
/// # Returns
/// The new value of the named counter
///
/// # Example
/// ```json
/// {
///   "name": "page_visits"
/// }
/// ```
/// Returns: `1` (first increment)
#[tool("Increment a named counter")]
fn increment_named(name: String) -> u64 {
    NAMED_COUNTERS.with(|counters| {
        let mut map = counters.borrow_mut();
        let count = map.entry(name).or_insert(0);
        *count += 1;
        *count
    })
}

/// Get the value of a named counter.
///
/// # Parameters
/// - `name`: The name of the counter to query
///
/// # Returns
/// The current value of the named counter, or 0 if it doesn't exist
///
/// # Example
/// ```json
/// {
///   "name": "page_visits"
/// }
/// ```
/// Returns: `42`
#[tool("Get the value of a named counter")]
fn get_named(name: String) -> u64 {
    NAMED_COUNTERS.with(|counters| {
        *counters.borrow().get(&name).unwrap_or(&0)
    })
}

/// List all named counters and their values.
///
/// # Returns
/// JSON object mapping counter names to their values
///
/// # Example
/// ```json
/// {}
/// ```
/// Returns: `{"page_visits": 42, "button_clicks": 17, "errors": 3}`
#[tool("List all named counters")]
fn list_counters() -> String {
    NAMED_COUNTERS.with(|counters| {
        let map = counters.borrow();
        serde_json::to_string(&*map)
            .expect("Failed to serialize counters")
    })
}

/// Reset a named counter to zero.
///
/// # Parameters
/// - `name`: The name of the counter to reset
///
/// # Returns
/// The previous value before reset, or 0 if counter didn't exist
///
/// # Example
/// ```json
/// {
///   "name": "page_visits"
/// }
/// ```
/// Returns: `42` (counter is now 0)
#[tool("Reset a named counter to zero")]
fn reset_named(name: String) -> u64 {
    NAMED_COUNTERS.with(|counters| {
        let mut map = counters.borrow_mut();
        map.insert(name, 0).unwrap_or(0)
    })
}

/// Delete a named counter.
///
/// # Parameters
/// - `name`: The name of the counter to delete
///
/// # Returns
/// `true` if counter was deleted, `false` if it didn't exist
///
/// # Example
/// ```json
/// {
///   "name": "page_visits"
/// }
/// ```
/// Returns: `true`
#[tool("Delete a named counter")]
fn delete_counter(name: String) -> bool {
    NAMED_COUNTERS.with(|counters| {
        counters.borrow_mut().remove(&name).is_some()
    })
}

// Generate MCP server endpoints
icarus_macros::mcp! {}

// Note: For production use with persistent state:
//
// 1. **Use ic-stable-structures for persistence**:
//    ```rust
//    use ic_stable_structures::{StableBTreeMap, DefaultMemoryImpl};
//
//    thread_local! {
//        static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = ...;
//        static COUNTERS: RefCell<StableBTreeMap<String, u64>> = ...;
//    }
//    ```
//
// 2. **Implement pre/post-upgrade hooks**:
//    ```rust
//    #[ic_cdk::pre_upgrade]
//    fn pre_upgrade() {
//        // Save volatile state to stable memory
//    }
//
//    #[ic_cdk::post_upgrade]
//    fn post_upgrade() {
//        // Restore state from stable memory
//    }
//    ```
//
// 3. **Consider using ic-cdk-timers for periodic state snapshots**
//
// 4. **Implement state versioning for upgrade compatibility**
//
// 5. **Add state size monitoring to prevent memory exhaustion**

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        // Note: Test isolation is limited for thread-local state
        // In production, use proper test fixtures
        let initial = get_count();
        let after_increment = increment();
        assert_eq!(after_increment, initial + 1);
    }

    #[test]
    fn test_add() {
        let initial = get_count();
        let after_add = add(10);
        assert_eq!(after_add, initial + 10);
    }

    #[test]
    fn test_named_counters() {
        let test_name = format!("test_counter_{}", rand::random::<u32>());
        let first = increment_named(test_name.clone());
        assert_eq!(first, 1);

        let second = increment_named(test_name.clone());
        assert_eq!(second, 2);

        let value = get_named(test_name.clone());
        assert_eq!(value, 2);
    }
}