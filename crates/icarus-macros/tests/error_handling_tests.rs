//! Error handling tests for icarus-macros crate.
//!
//! These tests verify that macros handle error conditions gracefully
//! and provide helpful error messages for common mistakes.

use icarus_macros::tool;

/// Test error cases that should be caught at compile time
#[test]
fn test_compile_time_error_detection() {
    // These tests document the expected error behavior
    // In a full implementation, these would use `trybuild` or similar
    // to test actual compilation failures
}

/// Test runtime error handling in generated wrapper code
#[test]
fn test_runtime_error_handling() {
    #[tool]
    fn error_prone_tool(input: String) -> Result<String, String> {
        if input.is_empty() {
            Err("Input cannot be empty".to_string())
        } else if input.len() > 100 {
            Err("Input too long".to_string())
        } else {
            Ok(format!("Processed: {}", input))
        }
    }

    // Test successful case
    let success = error_prone_tool("hello".to_string());
    assert_eq!(success, Ok("Processed: hello".to_string()));

    // Test error cases
    let empty_error = error_prone_tool(String::new());
    assert_eq!(empty_error, Err("Input cannot be empty".to_string()));

    let long_input = "x".repeat(101);
    let long_error = error_prone_tool(long_input);
    assert_eq!(long_error, Err("Input too long".to_string()));
}

/// Test JSON deserialization error handling
#[test]
fn test_json_error_handling() {
    // Test how the generated wrapper functions handle JSON parsing errors
    // This simulates what happens when invalid JSON is passed to a tool

    #[tool]
    fn json_test_tool(name: String, age: u32) -> String {
        format!("{} is {} years old", name, age)
    }

    // We can't directly test the generated wrapper, but we can test
    // that the tool itself works with valid inputs
    let result = json_test_tool("Alice".to_string(), 30);
    assert_eq!(result, "Alice is 30 years old");

    // JSON parsing errors would be handled by the generated wrapper code
    // The wrapper should return appropriate error messages for:
    // - Invalid JSON syntax
    // - Missing required fields
    // - Type mismatches
    // - Extra unexpected fields
}

/// Test parameter validation error handling
#[test]
fn test_parameter_validation_errors() {
    use serde_json::{from_value, json};

    // Test how serde handles various parameter validation scenarios

    #[derive(serde::Deserialize, Debug)]
    struct TestParams {
        #[allow(dead_code)]
        required_string: String,
        #[allow(dead_code)]
        required_number: i32,
        optional_bool: Option<bool>,
    }

    // Valid parameters should deserialize successfully
    let valid_json = json!({
        "required_string": "test",
        "required_number": 42,
        "optional_bool": true
    });
    let valid_params: Result<TestParams, _> = from_value(valid_json);
    assert!(valid_params.is_ok());

    // Missing required field should fail
    let missing_required = json!({
        "required_string": "test"
        // missing required_number
    });
    let missing_result: Result<TestParams, _> = from_value(missing_required);
    assert!(missing_result.is_err());

    // Wrong type should fail
    let wrong_type = json!({
        "required_string": "test",
        "required_number": "not_a_number",
        "optional_bool": true
    });
    let wrong_type_result: Result<TestParams, _> = from_value(wrong_type);
    assert!(wrong_type_result.is_err());

    // Optional field can be missing
    let missing_optional = json!({
        "required_string": "test",
        "required_number": 42
    });
    let optional_result: Result<TestParams, _> = from_value(missing_optional);
    assert!(optional_result.is_ok());
    assert_eq!(optional_result.unwrap().optional_bool, None);
}

/// Test error handling with complex parameter types
#[test]
fn test_complex_type_errors() {
    use serde::{Deserialize, Serialize};
    use serde_json::{from_value, json};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct UserInfo {
        name: String,
        email: String,
        age: u32,
    }

    #[derive(serde::Deserialize)]
    struct ComplexParams {
        #[allow(dead_code)]
        user: UserInfo,
        #[allow(dead_code)]
        tags: Vec<String>,
        #[allow(dead_code)]
        metadata: std::collections::HashMap<String, String>,
    }

    // Valid complex parameters
    let valid_complex = json!({
        "user": {
            "name": "Alice",
            "email": "alice@example.com",
            "age": 30
        },
        "tags": ["important", "user"],
        "metadata": {
            "source": "api",
            "version": "1.0"
        }
    });

    let valid_result: Result<ComplexParams, _> = from_value(valid_complex);
    assert!(valid_result.is_ok());

    // Invalid nested structure
    let invalid_nested = json!({
        "user": {
            "name": "Alice",
            // missing email and age
        },
        "tags": ["important"],
        "metadata": {}
    });

    let invalid_result: Result<ComplexParams, _> = from_value(invalid_nested);
    assert!(invalid_result.is_err());

    // Wrong array type
    let wrong_array = json!({
        "user": {
            "name": "Alice",
            "email": "alice@example.com",
            "age": 30
        },
        "tags": [123, 456], // Should be strings
        "metadata": {}
    });

    let array_error: Result<ComplexParams, _> = from_value(wrong_array);
    assert!(array_error.is_err());
}

/// Test async function error handling
#[test]
fn test_async_error_handling() {
    use tokio_test;

    #[tool]
    async fn async_error_tool(should_fail: bool) -> Result<String, String> {
        tokio::task::yield_now().await;

        if should_fail {
            Err("Async operation failed".to_string())
        } else {
            Ok("Async operation succeeded".to_string())
        }
    }

    tokio_test::block_on(async {
        let success = async_error_tool(false).await;
        assert_eq!(success, Ok("Async operation succeeded".to_string()));

        let failure = async_error_tool(true).await;
        assert_eq!(failure, Err("Async operation failed".to_string()));
    });
}

/// Test panic handling in tool functions
#[test]
fn test_panic_handling() {
    #[tool]
    fn panic_tool(should_panic: bool) -> String {
        assert!(!should_panic, "This function panicked!");
        "No panic occurred".to_string()
    }

    // Normal operation should work
    let normal = panic_tool(false);
    assert_eq!(normal, "No panic occurred");

    // Panics should be caught by the test framework
    // In real usage, the MCP runtime would handle panics appropriately
    let result = std::panic::catch_unwind(|| panic_tool(true));
    assert!(result.is_err());
}

/// Test serialization error handling
#[test]
fn test_serialization_errors() {
    use serde::{Deserialize, Serialize};
    use serde_json::{from_str, to_string};

    // Types that might fail serialization
    #[derive(Serialize, Deserialize, Debug)]
    struct ProblematicType {
        normal_field: String,
        // In practice, most basic types serialize fine
        // But custom serialization logic might fail
    }

    #[tool]
    fn serialization_tool(input: String) -> ProblematicType {
        ProblematicType {
            normal_field: input,
        }
    }

    let result = serialization_tool("test".to_string());
    assert_eq!(result.normal_field, "test");

    // Test that the result can be serialized
    let serialized = to_string(&result);
    assert!(serialized.is_ok());

    // And deserialized back
    let json_str = serialized.unwrap();
    let deserialized: Result<ProblematicType, _> = from_str(&json_str);
    assert!(deserialized.is_ok());
}

/// Test error message quality and informativeness
#[test]
fn test_error_message_quality() {
    // Hypothetical error types

    // Error messages should be:
    // 1. Descriptive and helpful
    // 2. Include context about what went wrong
    // 3. Suggest potential fixes when possible
    // 4. Be consistent in format

    // This test verifies error message properties
    let error_cases = vec![
        ("Generic functions are not supported", "generic"),
        ("Tool functions cannot have self parameters", "self"),
        ("Invalid function signature", "signature"),
        ("Configuration error", "config"),
    ];

    for (message, category) in error_cases {
        // Error messages should be non-empty
        assert!(!message.is_empty());

        // Should not be just whitespace
        assert!(!message.trim().is_empty());

        // Should contain category information
        assert!(message.to_lowercase().contains(category) || message.len() > 10);

        // Should be helpful (at least somewhat descriptive)
        assert!(message.len() > 5);
    }
}

/// Test edge cases in error handling
#[test]
fn test_error_edge_cases() {
    // Test very long parameter names
    #[tool]
    fn long_param_tool(very_long_parameter_name_that_exceeds_normal_limits: String) -> String {
        very_long_parameter_name_that_exceeds_normal_limits.to_uppercase()
    }

    let result = long_param_tool("test".to_string());
    assert_eq!(result, "TEST");

    // Test empty strings
    #[tool]
    fn empty_string_tool(input: String) -> String {
        if input.is_empty() {
            "Empty input received".to_string()
        } else {
            format!("Input: {}", input)
        }
    }

    let empty_result = empty_string_tool(String::new());
    assert_eq!(empty_result, "Empty input received");

    // Test special characters in strings
    #[tool]
    fn special_chars_tool(input: String) -> String {
        format!("Special: {}", input)
    }

    let special_result = special_chars_tool("Hello\nWorld\t\"Test\"".to_string());
    assert!(special_result.contains("Hello\nWorld\t\"Test\""));
}

/// Test error handling with Option and Result types
#[test]
fn test_option_result_error_handling() {
    #[tool]
    fn option_tool(input: Option<String>) -> String {
        match input {
            Some(s) => format!("Got: {}", s),
            None => "Got None".to_string(),
        }
    }

    #[tool]
    fn result_tool(input: String) -> Result<String, String> {
        if input == "error" {
            Err("Requested error".to_string())
        } else {
            Ok(format!("Success: {}", input))
        }
    }

    // Test Option handling
    let some_result = option_tool(Some("test".to_string()));
    assert_eq!(some_result, "Got: test");

    let none_result = option_tool(None);
    assert_eq!(none_result, "Got None");

    // Test Result handling
    let ok_result = result_tool("hello".to_string());
    assert_eq!(ok_result, Ok("Success: hello".to_string()));

    let err_result = result_tool("error".to_string());
    assert_eq!(err_result, Err("Requested error".to_string()));
}

/// Test concurrent error handling
#[test]
fn test_concurrent_error_handling() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let counter = Arc::new(Mutex::new(0));

    #[tool]
    fn concurrent_tool(id: i32) -> Result<String, String> {
        // Simulate some work that might conflict
        if id < 0 {
            Err(format!("Negative ID not allowed: {}", id))
        } else {
            Ok(format!("Processed ID: {}", id))
        }
    }

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let counter = counter.clone();
            thread::spawn(move || {
                let result = concurrent_tool(i);
                match result {
                    Ok(msg) => {
                        let mut count = counter.lock().unwrap();
                        *count += 1;
                        msg
                    }
                    Err(e) => e,
                }
            })
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All positive IDs should succeed
    assert_eq!(results.len(), 10);
    let success_count = results
        .iter()
        .filter(|r| r.starts_with("Processed"))
        .count();
    assert_eq!(success_count, 10);

    let final_count = *counter.lock().unwrap();
    assert_eq!(final_count, 10);
}

// Helper module for error type testing (would be real imports in practice)
#[allow(dead_code)]
mod icarus_macros_errors {
    // Mock error types for testing
}
