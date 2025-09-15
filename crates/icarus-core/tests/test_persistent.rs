//! Tests for persistent state management

use icarus_core::persistent::{IcarusPersistentState, MemoryPersistentState, TypedPersistentState};
use serde::{Deserialize, Serialize};

/// Test data structure for typed persistence
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestData {
    id: u64,
    name: String,
    active: bool,
    tags: Vec<String>,
}

/// Test MemoryPersistentState creation
#[tokio::test]
async fn test_memory_persistent_state_creation() {
    let state = MemoryPersistentState::new();
    let size = state.size().await.unwrap();
    assert_eq!(size, 0);

    let default_state = MemoryPersistentState::default();
    let default_size = default_state.size().await.unwrap();
    assert_eq!(default_size, 0);
}

/// Test basic set and get operations
#[tokio::test]
async fn test_basic_set_get() {
    let mut state = MemoryPersistentState::new();
    let key = "test_key".to_string();
    let value = b"test_value".to_vec();

    // Test setting a value
    let result = state.set(key.clone(), value.clone()).await;
    assert!(result.is_ok());

    // Test getting the value
    let retrieved = state.get(&key).await.unwrap();
    assert_eq!(retrieved, Some(value));

    // Test getting non-existent key
    let missing = state.get("missing_key").await.unwrap();
    assert_eq!(missing, None);
}

/// Test multiple key-value pairs
#[tokio::test]
async fn test_multiple_keys() {
    let mut state = MemoryPersistentState::new();

    let pairs = vec![
        ("key1".to_string(), b"value1".to_vec()),
        ("key2".to_string(), b"value2".to_vec()),
        ("key3".to_string(), b"value3".to_vec()),
    ];

    // Set multiple values
    for (key, value) in &pairs {
        state.set(key.clone(), value.clone()).await.unwrap();
    }

    // Verify all values
    for (key, expected_value) in &pairs {
        let retrieved = state.get(key).await.unwrap();
        assert_eq!(retrieved, Some(expected_value.clone()));
    }
}

/// Test key listing functionality
#[tokio::test]
async fn test_list_keys() {
    let mut state = MemoryPersistentState::new();

    // Empty state should have no keys
    let keys = state.list_keys().await.unwrap();
    assert!(keys.is_empty());

    // Add some keys
    let test_keys = vec!["alpha", "beta", "gamma"];
    for key in &test_keys {
        state.set(key.to_string(), b"value".to_vec()).await.unwrap();
    }

    // List keys and verify
    let mut keys = state.list_keys().await.unwrap();
    keys.sort(); // HashMap order is not guaranteed
    let mut expected_keys: Vec<String> = test_keys.iter().map(|s| s.to_string()).collect();
    expected_keys.sort();

    assert_eq!(keys, expected_keys);
}

/// Test delete functionality
#[tokio::test]
async fn test_delete() {
    let mut state = MemoryPersistentState::new();
    let key = "delete_me".to_string();
    let value = b"temporary_value".to_vec();

    // Set a value
    state.set(key.clone(), value.clone()).await.unwrap();
    assert_eq!(state.get(&key).await.unwrap(), Some(value));

    // Delete the value
    state.delete(&key).await.unwrap();
    assert_eq!(state.get(&key).await.unwrap(), None);

    // Deleting non-existent key should not error
    state.delete("non_existent").await.unwrap();
}

/// Test clear functionality
#[tokio::test]
async fn test_clear() {
    let mut state = MemoryPersistentState::new();

    // Add multiple values
    for i in 0..5 {
        let key = format!("key_{}", i);
        let value = format!("value_{}", i).into_bytes();
        state.set(key, value).await.unwrap();
    }

    // Verify we have data
    let keys = state.list_keys().await.unwrap();
    assert_eq!(keys.len(), 5);

    // Clear all data
    state.clear().await.unwrap();

    // Verify everything is gone
    let keys = state.list_keys().await.unwrap();
    assert!(keys.is_empty());
    assert_eq!(state.size().await.unwrap(), 0);
}

/// Test size calculation
#[tokio::test]
async fn test_size() {
    let mut state = MemoryPersistentState::new();

    // Empty state
    assert_eq!(state.size().await.unwrap(), 0);

    // Add some data
    let data1 = b"hello".to_vec(); // 5 bytes
    let data2 = b"world!".to_vec(); // 6 bytes
    let data3 = b"test".to_vec(); // 4 bytes

    state.set("key1".to_string(), data1).await.unwrap();
    assert_eq!(state.size().await.unwrap(), 5);

    state.set("key2".to_string(), data2).await.unwrap();
    assert_eq!(state.size().await.unwrap(), 11);

    state.set("key3".to_string(), data3).await.unwrap();
    assert_eq!(state.size().await.unwrap(), 15);

    // Delete one item
    state.delete("key2").await.unwrap();
    assert_eq!(state.size().await.unwrap(), 9);
}

/// Test overwriting existing keys
#[tokio::test]
async fn test_overwrite() {
    let mut state = MemoryPersistentState::new();
    let key = "overwrite_key".to_string();

    // Set initial value
    let value1 = b"original_value".to_vec();
    state.set(key.clone(), value1.clone()).await.unwrap();
    assert_eq!(state.get(&key).await.unwrap(), Some(value1));

    // Overwrite with new value
    let value2 = b"new_value".to_vec();
    state.set(key.clone(), value2.clone()).await.unwrap();
    assert_eq!(state.get(&key).await.unwrap(), Some(value2));

    // Should still have only one key
    let keys = state.list_keys().await.unwrap();
    assert_eq!(keys.len(), 1);
}

/// Test typed storage with simple types
#[tokio::test]
async fn test_typed_simple() {
    let mut state = MemoryPersistentState::new();

    // Test string
    let string_key = "string_key";
    let string_value = "Hello, World!".to_string();
    state
        .set_typed(string_key.to_string(), &string_value)
        .await
        .unwrap();
    let retrieved_string: String = state.get_typed(string_key).await.unwrap().unwrap();
    assert_eq!(retrieved_string, string_value);

    // Test number
    let number_key = "number_key";
    let number_value = 12345u64;
    state
        .set_typed(number_key.to_string(), &number_value)
        .await
        .unwrap();
    let retrieved_number: u64 = state.get_typed(number_key).await.unwrap().unwrap();
    assert_eq!(retrieved_number, number_value);

    // Test boolean
    let bool_key = "bool_key";
    let bool_value = true;
    state
        .set_typed(bool_key.to_string(), &bool_value)
        .await
        .unwrap();
    let retrieved_bool: bool = state.get_typed(bool_key).await.unwrap().unwrap();
    assert_eq!(retrieved_bool, bool_value);
}

/// Test typed storage with complex structures
#[tokio::test]
async fn test_typed_complex() {
    let mut state = MemoryPersistentState::new();

    let test_data = TestData {
        id: 42,
        name: "Test Item".to_string(),
        active: true,
        tags: vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
    };

    let key = "complex_data";

    // Store complex data
    state.set_typed(key.to_string(), &test_data).await.unwrap();

    // Retrieve and verify
    let retrieved: TestData = state.get_typed(key).await.unwrap().unwrap();
    assert_eq!(retrieved, test_data);
}

/// Test typed storage with collections
#[tokio::test]
async fn test_typed_collections() {
    let mut state = MemoryPersistentState::new();

    // Test Vec
    let vec_key = "vector_data";
    let vec_data = vec![1, 2, 3, 4, 5];
    state
        .set_typed(vec_key.to_string(), &vec_data)
        .await
        .unwrap();
    let retrieved_vec: Vec<i32> = state.get_typed(vec_key).await.unwrap().unwrap();
    assert_eq!(retrieved_vec, vec_data);

    // Test HashMap
    let map_key = "map_data";
    let mut map_data = std::collections::HashMap::new();
    map_data.insert("key1".to_string(), 100);
    map_data.insert("key2".to_string(), 200);
    state
        .set_typed(map_key.to_string(), &map_data)
        .await
        .unwrap();
    let retrieved_map: std::collections::HashMap<String, i32> =
        state.get_typed(map_key).await.unwrap().unwrap();
    assert_eq!(retrieved_map, map_data);
}

/// Test typed storage with Option types
#[tokio::test]
async fn test_typed_options() {
    let mut state = MemoryPersistentState::new();

    // Test Some value
    let some_key = "some_value";
    let some_data: Option<String> = Some("exists".to_string());
    state
        .set_typed(some_key.to_string(), &some_data)
        .await
        .unwrap();
    let retrieved_some: Option<String> = state.get_typed(some_key).await.unwrap().unwrap();
    assert_eq!(retrieved_some, some_data);

    // Test None value
    let none_key = "none_value";
    let none_data: Option<String> = None;
    state
        .set_typed(none_key.to_string(), &none_data)
        .await
        .unwrap();
    let retrieved_none: Option<String> = state.get_typed(none_key).await.unwrap().unwrap();
    assert_eq!(retrieved_none, none_data);
}

/// Test getting non-existent typed data
#[tokio::test]
async fn test_typed_missing() {
    let state = MemoryPersistentState::new();

    let result: Option<String> = state.get_typed("missing_key").await.unwrap();
    assert_eq!(result, None);
}

/// Test serialization error handling in typed storage
#[tokio::test]
async fn test_typed_serialization_errors() {
    let mut state = MemoryPersistentState::new();

    // Set invalid JSON data manually
    let key = "invalid_json";
    let invalid_data = b"{ invalid json }".to_vec();
    state.set(key.to_string(), invalid_data).await.unwrap();

    // Try to deserialize as typed data - should return error
    let result: Result<Option<TestData>, _> = state.get_typed(key).await;
    assert!(result.is_err());
}

/// Test edge cases with empty data
#[tokio::test]
async fn test_edge_cases() {
    let mut state = MemoryPersistentState::new();

    // Test empty key
    let empty_key = "";
    let value = b"value_for_empty_key".to_vec();
    state
        .set(empty_key.to_string(), value.clone())
        .await
        .unwrap();
    assert_eq!(state.get(empty_key).await.unwrap(), Some(value));

    // Test empty value
    let key = "empty_value_key";
    let empty_value = Vec::new();
    state
        .set(key.to_string(), empty_value.clone())
        .await
        .unwrap();
    assert_eq!(state.get(key).await.unwrap(), Some(empty_value));

    // Test very long key
    let long_key = "a".repeat(1000);
    let value = b"value_for_long_key".to_vec();
    state.set(long_key.clone(), value.clone()).await.unwrap();
    assert_eq!(state.get(&long_key).await.unwrap(), Some(value));

    // Test very large value
    let key = "large_value_key";
    let large_value = vec![0u8; 10000]; // 10KB of zeros
    state
        .set(key.to_string(), large_value.clone())
        .await
        .unwrap();
    assert_eq!(state.get(key).await.unwrap(), Some(large_value));
}

/// Test concurrent-like operations (sequential but mimics patterns)
#[tokio::test]
async fn test_multiple_operations() {
    let mut state = MemoryPersistentState::new();

    // Perform multiple operations in sequence
    let operations = vec![
        ("set", "key1", Some(b"value1".to_vec())),
        ("set", "key2", Some(b"value2".to_vec())),
        ("get", "key1", None),
        ("delete", "key1", None),
        ("get", "key1", None), // Should be None now
        ("set", "key3", Some(b"value3".to_vec())),
        ("clear", "", None),
        ("get", "key2", None), // Should be None after clear
        ("get", "key3", None), // Should be None after clear
    ];

    for (op, key, data) in operations {
        match op {
            "set" => {
                if let Some(value) = data {
                    state.set(key.to_string(), value).await.unwrap();
                }
            }
            "get" => {
                let _ = state.get(key).await.unwrap();
            }
            "delete" => {
                state.delete(key).await.unwrap();
            }
            "clear" => {
                state.clear().await.unwrap();
            }
            _ => panic!("Unknown operation: {}", op),
        }
    }

    // Final state should be empty
    let keys = state.list_keys().await.unwrap();
    assert!(keys.is_empty());
    assert_eq!(state.size().await.unwrap(), 0);
}

/// Test typed storage with nested structures
#[tokio::test]
async fn test_typed_nested() {
    let mut state = MemoryPersistentState::new();

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct NestedData {
        user: TestData,
        metadata: std::collections::HashMap<String, Vec<String>>,
        config: Option<Config>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Config {
        enabled: bool,
        timeout: u64,
        retries: u32,
    }

    let mut metadata = std::collections::HashMap::new();
    metadata.insert(
        "permissions".to_string(),
        vec!["read".to_string(), "write".to_string()],
    );
    metadata.insert(
        "groups".to_string(),
        vec!["admin".to_string(), "user".to_string()],
    );

    let nested_data = NestedData {
        user: TestData {
            id: 1,
            name: "Admin User".to_string(),
            active: true,
            tags: vec!["admin".to_string(), "power_user".to_string()],
        },
        metadata,
        config: Some(Config {
            enabled: true,
            timeout: 5000,
            retries: 3,
        }),
    };

    let key = "nested_data";
    state
        .set_typed(key.to_string(), &nested_data)
        .await
        .unwrap();

    let retrieved: NestedData = state.get_typed(key).await.unwrap().unwrap();
    assert_eq!(retrieved, nested_data);
}

/// Test trait implementation completeness
#[tokio::test]
async fn test_trait_implementation() {
    let mut state = MemoryPersistentState::new();

    // Test that MemoryPersistentState implements IcarusPersistentState
    let _: &dyn IcarusPersistentState = &state;

    // Test all IcarusPersistentState methods are accessible
    state
        .set("test".to_string(), b"data".to_vec())
        .await
        .unwrap();
    let _data = state.get("test").await.unwrap();
    state.delete("test").await.unwrap();
    let _keys = state.list_keys().await.unwrap();
    state.clear().await.unwrap();
    let _size = state.size().await.unwrap();

    // Test typed methods (TypedPersistentState is auto-implemented)
    state.set_typed("typed".to_string(), &42u64).await.unwrap();
    let _value: Option<u64> = state.get_typed("typed").await.unwrap();
}
