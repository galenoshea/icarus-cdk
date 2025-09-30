//! Simple integration tests for icarus-macros crate.
//!
//! These tests verify that macros compile and generate working code,
//! focusing on functionality rather than testing generated internals.

use icarus_macros::tool;

/// Test that basic tool macro usage compiles and works
#[test]
fn test_basic_tool_functionality() {
    #[tool]
    fn simple_add(a: i32, b: i32) -> i32 {
        a + b
    }

    // Test the original function still works
    let result = simple_add(2, 3);
    assert_eq!(result, 5);
}

/// Test async tools compile and work
#[test]
fn test_async_tool_functionality() {
    use tokio_test;

    #[tool]
    async fn async_multiply(a: i32, b: i32) -> i32 {
        tokio::task::yield_now().await;
        a * b
    }

    tokio_test::block_on(async {
        let result = async_multiply(4, 5).await;
        assert_eq!(result, 20);
    });
}

/// Test tools with optional parameters
#[test]
fn test_optional_parameter_tools() {
    #[tool]
    fn greet(name: String, title: Option<String>) -> String {
        match title {
            Some(t) => format!("Hello, {} {}!", t, name),
            None => format!("Hello, {}!", name),
        }
    }

    let result1 = greet("Alice".to_string(), Some("Dr.".to_string()));
    assert_eq!(result1, "Hello, Dr. Alice!");

    let result2 = greet("Bob".to_string(), None);
    assert_eq!(result2, "Hello, Bob!");
}

/// Test tools with different return types
#[test]
fn test_different_return_types() {
    #[tool]
    fn get_flag() -> bool {
        true
    }

    #[tool]
    fn get_number() -> i64 {
        42
    }

    #[tool]
    fn get_text() -> String {
        "test".to_string()
    }

    assert!(get_flag());
    assert_eq!(get_number(), 42);
    assert_eq!(get_text(), "test");
}

/// Test tools with complex parameter types
#[test]
fn test_complex_parameters() {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Person {
        name: String,
        age: u32,
    }

    #[tool]
    fn process_person(person: Person) -> String {
        format!("{} is {} years old", person.name, person.age)
    }

    let person = Person {
        name: "Charlie".to_string(),
        age: 25,
    };

    let result = process_person(person);
    assert_eq!(result, "Charlie is 25 years old");
}

/// Test tools with Result return types
#[test]
fn test_result_return_types() {
    #[tool]
    fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
        if b == 0.0 {
            Err("Division by zero".to_string())
        } else {
            Ok(a / b)
        }
    }

    let success = safe_divide(10.0, 2.0);
    assert_eq!(success, Ok(5.0));

    let error = safe_divide(10.0, 0.0);
    assert_eq!(error, Err("Division by zero".to_string()));
}

/// Test tools with vector parameters
#[test]
fn test_vector_parameters() {
    #[tool]
    fn sum_numbers(numbers: Vec<i32>) -> i32 {
        numbers.iter().sum()
    }

    #[tool]
    fn join_strings(strings: Vec<String>) -> String {
        strings.join(", ")
    }

    let sum_result = sum_numbers(vec![1, 2, 3, 4, 5]);
    assert_eq!(sum_result, 15);

    let join_result = join_strings(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    assert_eq!(join_result, "a, b, c");
}

/// Test tools with tuple parameters
#[test]
fn test_tuple_parameters() {
    #[tool]
    fn process_coordinate(point: (f64, f64)) -> f64 {
        (point.0 * point.0 + point.1 * point.1).sqrt()
    }

    let distance = process_coordinate((3.0, 4.0));
    assert!((distance - 5.0).abs() < 0.001);
}

/// Test tools with multiple parameters
#[test]
fn test_multiple_parameters() {
    #[tool]
    fn create_message(
        prefix: String,
        number: i32,
        suffix: Option<String>,
        uppercase: bool,
    ) -> String {
        let mut message = format!("{}{}", prefix, number);
        if let Some(s) = suffix {
            message.push_str(&s);
        }
        if uppercase {
            message.to_uppercase()
        } else {
            message
        }
    }

    let result1 = create_message("Count: ".to_string(), 42, None, false);
    assert_eq!(result1, "Count: 42");

    let result2 = create_message("Item #".to_string(), 5, Some(" (final)".to_string()), true);
    assert_eq!(result2, "ITEM #5 (FINAL)");
}

/// Test that tool macros work with documentation
#[test]
fn test_tools_with_documentation() {
    /// This function calculates the square of a number.
    ///
    /// It takes an integer and returns its square.
    #[tool]
    fn square(n: i32) -> i32 {
        n * n
    }

    let result = square(7);
    assert_eq!(result, 49);
}

/// Test tools with various visibility modifiers
#[test]
fn test_tool_visibility() {
    #[tool]
    pub(crate) fn public_tool(x: i32) -> i32 {
        x * 2
    }

    #[tool]
    fn private_tool(x: i32) -> i32 {
        x + 1
    }

    assert_eq!(public_tool(5), 10);
    assert_eq!(private_tool(5), 6);
}

/// Test that multiple tools can coexist
#[test]
fn test_multiple_tools_coexistence() {
    #[tool]
    fn tool_a(x: i32) -> i32 {
        x + 10
    }

    #[tool]
    fn tool_b(x: i32) -> i32 {
        x * 10
    }

    #[tool]
    fn tool_c(a: i32, b: i32) -> i32 {
        tool_a(a) + tool_b(b)
    }

    assert_eq!(tool_a(5), 15);
    assert_eq!(tool_b(5), 50);
    assert_eq!(tool_c(3, 4), 53); // (3+10) + (4*10) = 13 + 40 = 53
}

/// Test edge cases
#[test]
fn test_edge_cases() {
    #[tool]
    fn handle_empty_string(s: String) -> usize {
        s.len()
    }

    #[tool]
    fn handle_zero(n: i32) -> bool {
        n == 0
    }

    assert_eq!(handle_empty_string(String::new()), 0);
    assert_eq!(handle_empty_string("hello".to_string()), 5);
    assert!(handle_zero(0));
    assert!(!handle_zero(1));
}

/// Test tools in modules
#[test]
fn test_tools_in_modules() {
    mod inner {
        use super::*;

        #[tool]
        pub(crate) fn module_tool(x: String) -> String {
            format!("module: {}", x)
        }

        pub(crate) mod nested {
            use super::*;

            #[tool]
            pub(crate) fn nested_tool(x: i32) -> String {
                format!("nested: {}", x)
            }
        }
    }

    let result1 = inner::module_tool("test".to_string());
    assert_eq!(result1, "module: test");

    let result2 = inner::nested::nested_tool(42);
    assert_eq!(result2, "nested: 42");
}
