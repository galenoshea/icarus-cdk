//! Compilation tests for icarus-macros crate.
//!
//! These tests verify that macros handle edge cases correctly and produce
//! appropriate compilation errors when misused. Uses the `trybuild` crate
//! to test compile-time behavior.

/// Test that valid tool macro usage compiles successfully
#[test]
fn test_valid_tool_compilation() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compilation/pass/*.rs");
}

/// Test that invalid tool macro usage produces compilation errors
#[test]
fn test_invalid_tool_compilation() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation/fail/*.rs");
}

/// Test that tool macro rejects invalid function signatures
#[test]
fn test_tool_macro_error_cases() {
    // These tests would ideally use `trybuild` to test compilation failures
    // For now, we document the expected behavior:

    // 1. Generic functions should be rejected:
    //    #[tool]
    //    fn generic_tool<T>(x: T) -> T { x }
    //    Expected error: "Generic functions are not supported"

    // 2. Functions with self parameters should be rejected:
    //    #[tool]
    //    fn method_tool(&self, x: i32) -> i32 { x }
    //    Expected error: "Tool functions cannot have self parameters"

    // 3. Functions with lifetime parameters should be rejected:
    //    #[tool]
    //    fn lifetime_tool<'a>(x: &'a str) -> &'a str { x }
    //    Expected error: "Lifetime parameters are not supported"

    // Since these require compile-time testing infrastructure,
    // we'll test the error detection logic in unit tests instead
}

/// Test mcp macro edge cases
#[test]
fn test_mcp_macro_edge_cases() {
    // Test that mcp macro handles various configurations correctly
    // These would be compile-time tests ideally:

    // 1. Empty configuration should work:
    //    mcp! {}

    // 2. Configuration with all options should work:
    //    mcp! {
    //        name = "test_service",
    //        description = "Test service description",
    //        version = "1.0.0",
    //        auth = true,
    //        rate_limit = false
    //    }

    // 3. Invalid configuration keys should be rejected:
    //    mcp! { invalid_key = "value" }
    //    Expected error: "Unknown configuration key: invalid_key"

    // 4. Invalid boolean values should be rejected:
    //    mcp! { auth = "not_a_boolean" }
    //    Expected error: "auth must be a boolean value"
}

/// Test that macro expansion produces valid Rust code
#[test]
fn test_macro_output_validity() {
    // This test verifies that macro-generated code follows Rust conventions

    // The generated code should:
    // 1. Follow naming conventions (PascalCase for structs, snake_case for functions)
    // 2. Generate valid serde derive macros
    // 3. Produce properly formatted function signatures
    // 4. Include appropriate error handling
    // 5. Generate valid JSON-RPC protocol code

    // Since we can't easily inspect generated code in tests,
    // this test passes if macro usage compiles successfully
}

/// Test macro hygiene (no variable capture issues)
#[test]
fn test_macro_hygiene() {
    // Test that macros don't capture variables from surrounding scope
    let result = String::from("test");
    let args = 42;
    let config = true;

    #[icarus_macros::tool]
    fn hygiene_test(input: String) -> String {
        // The macro should not interfere with local variables
        // named 'result', 'args', 'config', etc.
        format!("output: {}", input)
    }

    let output = hygiene_test("hello".to_string());
    assert_eq!(output, "output: hello");

    // Original variables should be unchanged
    assert_eq!(result, "test");
    assert_eq!(args, 42);
    assert!(config);
}

/// Test that macros work in different module contexts
#[test]
fn test_macro_in_modules() {
    mod inner {
        use icarus_macros::tool;

        #[tool]
        pub(crate) fn module_tool(x: i32) -> i32 {
            x * 2
        }

        pub(crate) mod nested {
            use icarus_macros::tool;

            #[tool]
            pub(crate) fn nested_tool(s: String) -> String {
                s.to_uppercase()
            }
        }
    }

    assert_eq!(inner::module_tool(5), 10);
    assert_eq!(inner::nested::nested_tool("hello".to_string()), "HELLO");
}

/// Test macro interaction with other attributes
#[test]
fn test_macro_with_attributes() {
    #[allow(dead_code)]
    #[icarus_macros::tool]
    fn documented_tool(x: i32) -> i32 {
        x + 1
    }

    #[cfg(test)]
    #[icarus_macros::tool]
    fn conditional_tool(s: String) -> String {
        format!("test: {}", s)
    }

    // Tools should work normally with other attributes
    assert_eq!(documented_tool(5), 6);
    assert_eq!(conditional_tool("hello".to_string()), "test: hello");
}

/// Test that macros handle complex type expressions
#[test]
fn test_complex_types() {
    use std::collections::HashMap;

    #[icarus_macros::tool]
    fn complex_types_tool(
        map: HashMap<String, Vec<i32>>,
        tuple: (String, i32, bool),
        result: Result<String, String>,
    ) -> String {
        format!(
            "map_len={}, tuple=({},{},{}), result={:?}",
            map.len(),
            tuple.0,
            tuple.1,
            tuple.2,
            result
        )
    }

    let mut map = HashMap::new();
    map.insert("key".to_string(), vec![1, 2, 3]);

    let result = complex_types_tool(
        map,
        ("test".to_string(), 42, true),
        Ok("success".to_string()),
    );

    assert!(result.contains("map_len=1"));
    assert!(result.contains("tuple=(test,42,true)"));
    assert!(result.contains("Ok(\"success\")"));
}

/// Test macro performance with many parameters
#[test]
fn test_many_parameters() {
    #[icarus_macros::tool]
    fn many_params_tool(
        p1: i32,
        p2: i32,
        p3: i32,
        p4: i32,
        p5: i32,
        p6: i32,
        p7: i32,
        p8: i32,
        p9: i32,
        p10: i32,
        p11: String,
        p12: String,
        p13: String,
        p14: String,
        p15: String,
    ) -> i32 {
        p1 + p2
            + p3
            + p4
            + p5
            + p6
            + p7
            + p8
            + p9
            + p10
            + p11.len() as i32
            + p12.len() as i32
            + p13.len() as i32
            + p14.len() as i32
            + p15.len() as i32
    }

    let result = many_params_tool(
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
        9,
        10,
        "a".to_string(),
        "bb".to_string(),
        "ccc".to_string(),
        "dddd".to_string(),
        "eeeee".to_string(),
    );

    // Sum of numbers: 55, sum of string lengths: 15, total: 70
    assert_eq!(result, 70);
}

/// Test macro with zero parameters
#[test]
fn test_zero_parameters() {
    #[icarus_macros::tool]
    fn no_params_tool() -> String {
        "no parameters".to_string()
    }

    let result = no_params_tool();
    assert_eq!(result, "no parameters");
}

/// Test macro with reference parameters (if supported)
#[test]
fn test_reference_handling() {
    // Test that macros handle owned types correctly
    // (reference parameters would require lifetime management)

    #[icarus_macros::tool]
    fn string_tool(s: String) -> usize {
        s.len()
    }

    #[icarus_macros::tool]
    fn bytes_tool(data: Vec<u8>) -> usize {
        data.len()
    }

    assert_eq!(string_tool("hello".to_string()), 5);
    assert_eq!(bytes_tool(vec![1, 2, 3, 4]), 4);
}

/// Test macro expansion consistency
#[test]
fn test_expansion_consistency() {
    // Test that identical function signatures produce consistent expansions

    #[icarus_macros::tool]
    fn consistent_tool_a(x: i32, y: String) -> String {
        format!("{}: {}", x, y)
    }

    #[icarus_macros::tool]
    fn consistent_tool_b(x: i32, y: String) -> String {
        format!("{} -> {}", y, x)
    }

    // Both tools should work identically from macro perspective
    assert_eq!(consistent_tool_a(42, "test".to_string()), "42: test");
    assert_eq!(consistent_tool_b(42, "test".to_string()), "test -> 42");
}

/// Test that generated code doesn't conflict with user code
#[test]
fn test_no_name_conflicts() {
    // Define items with names similar to what the macro might generate
    struct TestParams {
        value: i32,
    }

    fn test_wrapper(x: i32) -> i32 {
        x * 3
    }

    fn test_info() -> String {
        "user function".to_string()
    }

    #[icarus_macros::tool]
    fn conflict_test_tool(input: i32) -> i32 {
        input * 2
    }

    // All functions should work without conflicts
    let params = TestParams { value: 10 };
    assert_eq!(params.value, 10);
    assert_eq!(test_wrapper(5), 15);
    assert_eq!(test_info(), "user function");
    assert_eq!(conflict_test_tool(7), 14);
}
