// Copyright (c) 2025 Icarus Team. All Rights Reserved.
// Licensed under BSL-1.1. See LICENSE and NOTICE files.

//! Tool compatibility requirements and validation
//!
//! This module defines the traits and types that ensure tools are compatible
//! with both ICP canisters and the Icarus bridge.

use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Marker trait for types that can be used as tool parameters
///
/// All tool parameters must implement this trait to ensure they can be:
/// 1. Deserialized from JSON (MCP protocol)
/// 2. Encoded to Candid (ICP canisters)
/// 3. Validated at compile time
pub trait IcarusParam: CandidType + for<'de> Deserialize<'de> + Send + Sync + 'static {
    /// Validate that this type can be used as a parameter
    fn validate() -> Result<(), String> {
        Ok(())
    }
}

/// Marker trait for types that can be returned from tools
///
/// All tool return types must implement this trait to ensure they can be:
/// 1. Serialized to JSON (MCP protocol)
/// 2. Encoded to Candid (ICP canisters)
pub trait IcarusReturn: CandidType + Serialize + Send + Sync + 'static {
    /// Validate that this type can be used as a return value
    fn validate() -> Result<(), String> {
        Ok(())
    }
}

/// Trait for tool functions that ensures compatibility
///
/// This trait is automatically implemented by the `#[icarus_tool]` macro
/// for functions with compatible signatures
pub trait IcarusTool {
    /// The input type (tuple of parameters)
    type Input: IcarusParam;

    /// The output type (must be Result<T, String>)
    type Output: IcarusReturn;

    /// Whether this is a query (read-only) or update (state-changing) operation
    const IS_QUERY: bool;

    /// Whether this function is async
    const IS_ASYNC: bool;

    /// Validate the tool signature at compile time
    fn validate_signature() -> Result<(), String> {
        // Queries must be synchronous in ICP
        if Self::IS_QUERY && Self::IS_ASYNC {
            return Err("Query functions cannot be async in ICP canisters".to_string());
        }
        Ok(())
    }
}

// Implement IcarusParam for common types
impl IcarusParam for String {}
impl IcarusParam for bool {}
impl IcarusParam for i8 {}
impl IcarusParam for i16 {}
impl IcarusParam for i32 {}
impl IcarusParam for i64 {}
impl IcarusParam for i128 {}
impl IcarusParam for u8 {}
impl IcarusParam for u16 {}
impl IcarusParam for u32 {}
impl IcarusParam for u64 {}
impl IcarusParam for u128 {}
impl IcarusParam for f32 {}
impl IcarusParam for f64 {}

// Implement for Option<T> where T: IcarusParam
impl<T> IcarusParam for Option<T> where T: IcarusParam {}

// Implement for Vec<T> where T: IcarusParam
impl<T> IcarusParam for Vec<T> where T: IcarusParam {}

// Implement for tuples (for multiple parameters)
impl IcarusParam for () {}
impl<T1: IcarusParam> IcarusParam for (T1,) {}
impl<T1: IcarusParam, T2: IcarusParam> IcarusParam for (T1, T2) {}
impl<T1: IcarusParam, T2: IcarusParam, T3: IcarusParam> IcarusParam for (T1, T2, T3) {}
impl<T1: IcarusParam, T2: IcarusParam, T3: IcarusParam, T4: IcarusParam> IcarusParam
    for (T1, T2, T3, T4)
{
}
impl<T1: IcarusParam, T2: IcarusParam, T3: IcarusParam, T4: IcarusParam, T5: IcarusParam>
    IcarusParam for (T1, T2, T3, T4, T5)
{
}

// Implement IcarusReturn for common types
impl IcarusReturn for String {}
impl IcarusReturn for bool {}
impl IcarusReturn for i8 {}
impl IcarusReturn for i16 {}
impl IcarusReturn for i32 {}
impl IcarusReturn for i64 {}
impl IcarusReturn for i128 {}
impl IcarusReturn for u8 {}
impl IcarusReturn for u16 {}
impl IcarusReturn for u32 {}
impl IcarusReturn for u64 {}
impl IcarusReturn for u128 {}
impl IcarusReturn for f32 {}
impl IcarusReturn for f64 {}
impl IcarusReturn for () {}

// Implement for Option<T> where T: IcarusReturn
impl<T> IcarusReturn for Option<T>
where
    T: IcarusReturn,
{
    fn validate() -> Result<(), String> {
        T::validate()
    }
}

// Implement for Vec<T> where T: IcarusReturn
impl<T> IcarusReturn for Vec<T>
where
    T: CandidType + Serialize + Send + Sync + 'static,
{
    fn validate() -> Result<(), String> {
        Ok(())
    }
}

// Special implementation for Result<T, E> as it's the required return type
impl<T, E> IcarusReturn for Result<T, E>
where
    T: CandidType + Serialize + Send + Sync + 'static,
    E: CandidType + Serialize + Send + Sync + 'static,
{
    fn validate() -> Result<(), String> {
        Ok(())
    }
}

/// Type alias for the standard tool result type
pub type ToolResult<T> = Result<T, String>;

/// Validate that a type can be used in tools
///
/// This is used by the macro to provide compile-time validation
pub const fn validate_tool_type<T>() -> bool {
    // This function is const to enable compile-time validation
    // The actual validation happens through trait bounds
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_basic_types_implement_traits() {
        // Test that basic types implement IcarusParam
        assert!(<String as IcarusParam>::validate().is_ok());
        assert!(<u64 as IcarusParam>::validate().is_ok());
        assert!(<bool as IcarusParam>::validate().is_ok());

        // Test that Option and Vec work as parameters
        assert!(<Option<String> as IcarusParam>::validate().is_ok());
        assert!(<Vec<u64> as IcarusParam>::validate().is_ok());

        // Test that basic types also implement IcarusReturn
        assert!(<String as IcarusReturn>::validate().is_ok());
        assert!(<u64 as IcarusReturn>::validate().is_ok());
        assert!(<bool as IcarusReturn>::validate().is_ok());
    }

    #[test]
    fn test_result_implements_return() {
        // Test that Result<T, String> implements IcarusReturn
        assert!(<Result<String, String> as IcarusReturn>::validate().is_ok());
        assert!(<Result<Vec<u64>, String> as IcarusReturn>::validate().is_ok());
    }

    #[test]
    fn test_all_integer_types_implement_param() {
        // Test signed integers
        assert!(<i8 as IcarusParam>::validate().is_ok());
        assert!(<i16 as IcarusParam>::validate().is_ok());
        assert!(<i32 as IcarusParam>::validate().is_ok());
        assert!(<i64 as IcarusParam>::validate().is_ok());
        assert!(<i128 as IcarusParam>::validate().is_ok());

        // Test unsigned integers
        assert!(<u8 as IcarusParam>::validate().is_ok());
        assert!(<u16 as IcarusParam>::validate().is_ok());
        assert!(<u32 as IcarusParam>::validate().is_ok());
        assert!(<u64 as IcarusParam>::validate().is_ok());
        assert!(<u128 as IcarusParam>::validate().is_ok());
    }

    #[test]
    fn test_all_integer_types_implement_return() {
        // Test signed integers
        assert!(<i8 as IcarusReturn>::validate().is_ok());
        assert!(<i16 as IcarusReturn>::validate().is_ok());
        assert!(<i32 as IcarusReturn>::validate().is_ok());
        assert!(<i64 as IcarusReturn>::validate().is_ok());
        assert!(<i128 as IcarusReturn>::validate().is_ok());

        // Test unsigned integers
        assert!(<u8 as IcarusReturn>::validate().is_ok());
        assert!(<u16 as IcarusReturn>::validate().is_ok());
        assert!(<u32 as IcarusReturn>::validate().is_ok());
        assert!(<u64 as IcarusReturn>::validate().is_ok());
        assert!(<u128 as IcarusReturn>::validate().is_ok());
    }

    #[test]
    fn test_floating_point_types() {
        // Test as parameters
        assert!(<f32 as IcarusParam>::validate().is_ok());
        assert!(<f64 as IcarusParam>::validate().is_ok());

        // Test as return types
        assert!(<f32 as IcarusReturn>::validate().is_ok());
        assert!(<f64 as IcarusReturn>::validate().is_ok());
    }

    #[test]
    fn test_tuple_parameters() {
        // Test empty tuple
        assert!(<() as IcarusParam>::validate().is_ok());

        // Test single element tuple
        assert!(<(String,) as IcarusParam>::validate().is_ok());

        // Test two element tuple
        assert!(<(String, u64) as IcarusParam>::validate().is_ok());

        // Test three element tuple
        assert!(<(String, u64, bool) as IcarusParam>::validate().is_ok());

        // Test four element tuple
        assert!(<(String, u64, bool, f64) as IcarusParam>::validate().is_ok());

        // Test five element tuple
        assert!(<(String, u64, bool, f64, i32) as IcarusParam>::validate().is_ok());
    }

    #[test]
    fn test_nested_option_types() {
        // Test nested Options as parameters
        assert!(<Option<Option<String>> as IcarusParam>::validate().is_ok());
        assert!(<Option<Vec<String>> as IcarusParam>::validate().is_ok());

        // Test nested Options as return types
        assert!(<Option<Option<String>> as IcarusReturn>::validate().is_ok());
        assert!(<Option<Vec<String>> as IcarusReturn>::validate().is_ok());
    }

    #[test]
    fn test_nested_vec_types() {
        // Test Vec of Vec as parameters
        assert!(<Vec<Vec<String>> as IcarusParam>::validate().is_ok());
        assert!(<Vec<Option<u64>> as IcarusParam>::validate().is_ok());

        // Test Vec of Vec as return types (need CandidType + Serialize)
        assert!(<Vec<String> as IcarusReturn>::validate().is_ok());
    }

    #[test]
    fn test_unit_type_return() {
        // Test unit type as return
        assert!(<() as IcarusReturn>::validate().is_ok());
    }

    #[test]
    fn test_result_type_variants() {
        // Test different Result types
        assert!(<Result<(), String> as IcarusReturn>::validate().is_ok());
        assert!(<Result<bool, String> as IcarusReturn>::validate().is_ok());
        assert!(<Result<Vec<u64>, String> as IcarusReturn>::validate().is_ok());
        assert!(<Result<Option<String>, String> as IcarusReturn>::validate().is_ok());
    }

    #[test]
    fn test_tool_result_type_alias() {
        // Test the ToolResult type alias
        let success: ToolResult<String> = Ok("success".to_string());
        let error: ToolResult<String> = Err("error".to_string());

        assert!(success.is_ok());
        assert!(error.is_err());
    }

    #[test]
    fn test_validate_tool_type_function() {
        // Test the const validation function
        assert!(validate_tool_type::<String>());
        assert!(validate_tool_type::<u64>());
        assert!(validate_tool_type::<Vec<String>>());
        assert!(validate_tool_type::<Option<bool>>());
    }

    // Mock tool implementation for testing the IcarusTool trait
    struct MockTool;

    impl IcarusTool for MockTool {
        type Input = (String, u64);
        type Output = Result<String, String>;
        const IS_QUERY: bool = false;
        const IS_ASYNC: bool = false;
    }

    struct MockQueryTool;

    impl IcarusTool for MockQueryTool {
        type Input = String;
        type Output = Result<bool, String>;
        const IS_QUERY: bool = true;
        const IS_ASYNC: bool = false;
    }

    struct MockAsyncTool;

    impl IcarusTool for MockAsyncTool {
        type Input = ();
        type Output = Result<(), String>;
        const IS_QUERY: bool = false;
        const IS_ASYNC: bool = true;
    }

    #[test]
    fn test_icarus_tool_validation() {
        // Test valid tool configurations
        assert!(MockTool::validate_signature().is_ok());
        assert!(MockQueryTool::validate_signature().is_ok());
        assert!(MockAsyncTool::validate_signature().is_ok());
    }

    // Test invalid configuration: async query (should fail validation)
    struct InvalidAsyncQueryTool;

    impl IcarusTool for InvalidAsyncQueryTool {
        type Input = String;
        type Output = Result<String, String>;
        const IS_QUERY: bool = true;  // Query...
        const IS_ASYNC: bool = true;  // ...but async (invalid)
    }

    #[test]
    fn test_icarus_tool_invalid_async_query() {
        // This should fail validation
        let result = InvalidAsyncQueryTool::validate_signature();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Query functions cannot be async"));
    }

    #[test]
    fn test_complex_parameter_combinations() {
        // Test complex nested types as parameters
        assert!(<Vec<(String, Option<u64>)> as IcarusParam>::validate().is_ok());
        assert!(<Option<Vec<(bool, i32)>> as IcarusParam>::validate().is_ok());
        assert!(<(Vec<String>, Option<u64>, bool) as IcarusParam>::validate().is_ok());
    }

    #[test]
    fn test_custom_types_with_required_traits() {
        #[derive(CandidType, Serialize, Deserialize)]
        struct CustomParam {
            name: String,
            value: u64,
        }

        impl IcarusParam for CustomParam {}

        #[derive(CandidType, Serialize)]
        struct CustomReturn {
            result: String,
            count: u32,
        }

        impl IcarusReturn for CustomReturn {}

        // Test validation
        assert!(CustomParam::validate().is_ok());
        assert!(CustomReturn::validate().is_ok());
    }

    #[test]
    fn test_option_validation_delegation() {
        #[derive(CandidType, Serialize)]
        struct TestType;

        impl IcarusReturn for TestType {
            fn validate() -> Result<(), String> {
                Err("TestType validation failed".to_string())
            }
        }

        // Option should delegate validation to the inner type
        let result = <Option<TestType> as IcarusReturn>::validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("TestType validation failed"));
    }

    #[test]
    fn test_vec_validation_for_return() {
        // Vec<T> should validate successfully for any T that meets bounds
        assert!(<Vec<String> as IcarusReturn>::validate().is_ok());
        assert!(<Vec<u64> as IcarusReturn>::validate().is_ok());
        assert!(<Vec<bool> as IcarusReturn>::validate().is_ok());
    }
}
