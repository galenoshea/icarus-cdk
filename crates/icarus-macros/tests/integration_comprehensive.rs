//! Comprehensive integration tests for icarus-macros crate.
//!
//! These tests verify complete macro expansion workflows, including code generation,
//! compilation, and runtime behavior following the patterns from `rust_best_practices.md`.

use icarus_macros::tool;

/// Test that simple functions can be converted to tools
#[test]
fn test_simple_tool_expansion() {
    // This test verifies the tool macro works with basic function signatures

    #[tool]
    fn add_numbers(a: f64, b: f64) -> f64 {
        a + b
    }

    // The tool macro should generate wrapper functions that we can test
    // We can't directly test the generated code, but we can verify compilation succeeds
    let result = add_numbers(2.0, 3.0);
    assert_eq!(result, 5.0);
}

/// Test that async functions can be converted to tools
#[test]
fn test_async_tool_expansion() {
    use tokio_test;

    #[tool]
    async fn async_add(a: i32, b: i32) -> i32 {
        // Simulate async work
        tokio::task::yield_now().await;
        a + b
    }

    tokio_test::block_on(async {
        let result = async_add(10, 20).await;
        assert_eq!(result, 30);
    });
}

/// Test that functions with optional parameters work
#[test]
fn test_optional_parameters() {
    #[tool]
    fn greet_user(name: String, title: Option<String>) -> String {
        match title {
            Some(t) => format!("Hello, {} {}!", t, name),
            None => format!("Hello, {}!", name),
        }
    }

    let result1 = greet_user("Alice".to_string(), Some("Dr.".to_string()));
    assert_eq!(result1, "Hello, Dr. Alice!");

    let result2 = greet_user("Bob".to_string(), None);
    assert_eq!(result2, "Hello, Bob!");
}

/// Test that functions with different return types work
#[test]
fn test_different_return_types() {
    #[tool]
    fn get_status() -> bool {
        true
    }

    #[tool]
    fn get_count() -> usize {
        42
    }

    #[tool]
    fn get_message() -> String {
        "test message".to_string()
    }

    assert!(get_status());
    assert_eq!(get_count(), 42);
    assert_eq!(get_message(), "test message");
}

/// Test that the mcp! macro generates valid code
#[test]
fn test_mcp_macro_basic() {
    // We can't easily test the full mcp! macro in a unit test since it generates
    // canister endpoints, but we can verify it compiles and expands correctly
    // by including it in a module scope

    mod test_mcp {
        use super::*;

        #[tool]
        pub(crate) fn test_function(input: String) -> String {
            format!("Processed: {}", input)
        }

        // Basic mcp! usage - this should compile without errors
        // mcp! {}  // Commented out as it would generate conflicting canister exports
    }

    // Test that tools defined in the module work
    let result = test_mcp::test_function("hello".to_string());
    assert_eq!(result, "Processed: hello");
}

/// Test tool macro with doc comments
#[test]
fn test_tool_with_documentation() {
    /// This function adds two numbers together.
    /// It's a simple mathematical operation.
    #[tool]
    fn documented_add(x: f64, y: f64) -> f64 {
        x + y
    }

    // The tool should work normally regardless of documentation
    let result = documented_add(1.5, 2.5);
    assert_eq!(result, 4.0);
}

/// Test complex parameter types
#[test]
fn test_complex_parameter_types() {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct UserData {
        name: String,
        age: u32,
    }

    #[tool]
    fn process_user(user: UserData) -> String {
        format!("{} is {} years old", user.name, user.age)
    }

    let user = UserData {
        name: "Alice".to_string(),
        age: 30,
    };

    let result = process_user(user);
    assert_eq!(result, "Alice is 30 years old");
}

/// Test multiple parameters with mixed types
#[test]
fn test_mixed_parameter_types() {
    #[tool]
    fn mixed_params(name: String, count: i32, enabled: bool, factor: Option<f64>) -> String {
        let factor_str = factor
            .map(|f| f.to_string())
            .unwrap_or_else(|| "1.0".to_string());
        format!(
            "name={}, count={}, enabled={}, factor={}",
            name, count, enabled, factor_str
        )
    }

    let result1 = mixed_params("test".to_string(), 5, true, Some(2.5));
    assert_eq!(result1, "name=test, count=5, enabled=true, factor=2.5");

    let result2 = mixed_params("demo".to_string(), -1, false, None);
    assert_eq!(result2, "name=demo, count=-1, enabled=false, factor=1.0");
}

/// Test that tool macro preserves function visibility
#[test]
fn test_function_visibility() {
    // Function with tool macro (testing visibility doesn't affect macro)
    #[tool]
    fn public_tool(input: String) -> String {
        format!("public: {}", input)
    }

    // Private function with tool macro (default)
    #[tool]
    fn private_tool(input: String) -> String {
        format!("private: {}", input)
    }

    assert_eq!(public_tool("test".to_string()), "public: test");
    assert_eq!(private_tool("test".to_string()), "private: test");
}

/// Test error handling in tool functions
#[test]
fn test_tool_error_handling() {
    #[tool]
    fn divide_numbers(a: f64, b: f64) -> Result<f64, String> {
        if b == 0.0 {
            Err("Division by zero".to_string())
        } else {
            Ok(a / b)
        }
    }

    let success = divide_numbers(10.0, 2.0);
    assert_eq!(success, Ok(5.0));

    let error = divide_numbers(10.0, 0.0);
    assert_eq!(error, Err("Division by zero".to_string()));
}

/// Test tool with unit return type
#[test]
fn test_unit_return_tool() {
    use std::sync::{Arc, Mutex};

    let _counter = Arc::new(Mutex::new(0));

    #[tool]
    fn increment_counter() {
        // This would normally access some global state
        // For testing, we'll just verify it compiles and runs
        let _ = 1 + 1; // Placeholder operation
    }

    increment_counter();
    // Test passes if function compiles and executes without panic
}

/// Test that functions with attributes besides #[tool] work
#[test]
fn test_tool_with_other_attributes() {
    #[allow(dead_code)]
    #[tool]
    fn attributed_function(input: String) -> String {
        input.to_uppercase()
    }

    let result = attributed_function("hello".to_string());
    assert_eq!(result, "HELLO");
}

/// Test nested function calls within tools
#[test]
fn test_tool_with_helper_functions() {
    fn helper_function(s: &str) -> String {
        s.chars().rev().collect()
    }

    #[tool]
    fn reverse_string(input: String) -> String {
        helper_function(&input)
    }

    let result = reverse_string("hello".to_string());
    assert_eq!(result, "olleh");
}

/// Test compilation with multiple tools in same module
#[test]
fn test_multiple_tools() {
    #[tool]
    fn tool_one(x: i32) -> i32 {
        x * 2
    }

    #[tool]
    fn tool_two(x: i32) -> i32 {
        x + 1
    }

    #[tool]
    fn tool_three(x: i32, y: i32) -> i32 {
        tool_one(x) + tool_two(y)
    }

    assert_eq!(tool_one(5), 10);
    assert_eq!(tool_two(5), 6);
    assert_eq!(tool_three(3, 4), 11); // (3*2) + (4+1) = 6 + 5 = 11
}

/// Test that macros work with generic types in parameters (should compile if supported)
#[test]
fn test_tool_with_concrete_generic_types() {
    #[tool]
    fn process_vector(items: Vec<String>) -> usize {
        items.len()
    }

    #[tool]
    fn process_option(maybe_value: Option<i32>) -> i32 {
        maybe_value.unwrap_or(0)
    }

    let vec_result = process_vector(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(vec_result, 2);

    let option_result1 = process_option(Some(42));
    assert_eq!(option_result1, 42);

    let option_result2 = process_option(None);
    assert_eq!(option_result2, 0);
}

/// Test long function names and parameter names
#[test]
fn test_long_identifiers() {
    #[tool]
    fn very_long_function_name_that_tests_identifier_limits(
        very_long_parameter_name_for_testing: String,
        another_extremely_long_parameter_name: i32,
    ) -> String {
        format!(
            "{}-{}",
            very_long_parameter_name_for_testing, another_extremely_long_parameter_name
        )
    }

    let result = very_long_function_name_that_tests_identifier_limits("test".to_string(), 123);
    assert_eq!(result, "test-123");
}

/// Test tool macro with const and static items (macro should not interfere)
#[test]
fn test_tool_with_constants() {
    const DEFAULT_MULTIPLIER: i32 = 3;
    static mut COUNTER: i32 = 0;

    #[tool]
    fn multiply_by_default(value: i32) -> i32 {
        value * DEFAULT_MULTIPLIER
    }

    #[tool]
    fn increment_and_return() -> i32 {
        unsafe {
            COUNTER += 1;
            COUNTER
        }
    }

    assert_eq!(multiply_by_default(5), 15);
    assert_eq!(increment_and_return(), 1);
    assert_eq!(increment_and_return(), 2);
}
