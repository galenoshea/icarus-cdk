//! Property-based tests for icarus-core crate using proptest.
//!
//! These tests verify invariants and properties that should hold for all
//! possible inputs, following `rust_best_practices.md` patterns.

use icarus_core::{
    error::IcarusError,
    newtypes::{SessionId, Timestamp, ToolId, UserId},
    protocol::{JsonRpcRequest, ToolCall, ToolResult},
    tool::{Tool, ToolParameter, ToolSchema},
};
use proptest::prelude::*;
use std::borrow::Cow;

// Property test strategies for generating test data

/// Strategy for generating valid tool ID strings
fn valid_tool_id_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z][a-zA-Z0-9_.-]{0,50}").expect("Valid regex for tool IDs")
}

/// Strategy for generating invalid tool ID strings
fn invalid_tool_id_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Empty string
        Just(String::new()),
        // Contains spaces (ensure at least one space)
        prop::string::string_regex("[a-zA-Z]+ [a-zA-Z0-9 ]*").expect("Valid regex"),
        // Contains invalid characters (ensure at least one invalid char)
        prop::string::string_regex("[a-zA-Z]*[@#$%]+[a-zA-Z0-9]*").expect("Valid regex"),
        // Too long
        prop::string::string_regex("[a-zA-Z][a-zA-Z0-9_.-]{300,400}").expect("Valid regex"),
        // Start with numbers (should be invalid - must start with letter)
        prop::string::string_regex("[0-9][a-zA-Z0-9_.-]{1,10}").expect("Valid regex"),
    ]
}

/// Strategy for generating valid user ID strings
fn valid_user_id_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9_.-]{1,255}").expect("Valid regex for user IDs")
}

/// Strategy for generating valid session ID strings
fn valid_session_id_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9_.-]{1,128}").expect("Valid regex for session IDs")
}

/// Strategy for generating timestamp values
fn timestamp_strategy() -> impl Strategy<Value = u64> {
    0u64..=u64::MAX
}

/// Type alias for JSON-RPC request data
type JsonRpcRequestData = (String, String, Option<String>, Option<String>);

/// Strategy for generating JSON-RPC request data
fn json_rpc_request_strategy() -> impl Strategy<Value = JsonRpcRequestData> {
    (
        Just("2.0".to_string()),
        prop::string::string_regex("[a-zA-Z][a-zA-Z0-9_/.-]{1,50}").expect("Valid method regex"),
        prop::option::of(prop::string::string_regex(r"\{[^{}]*\}").expect("Valid JSON regex")),
        prop::option::of(prop::string::string_regex("[a-zA-Z0-9-]{1,20}").expect("Valid ID regex")),
    )
}

proptest! {
    /// Test that valid tool IDs can always be created and round-trip correctly
    #[test]
    fn test_tool_id_roundtrip(tool_id_str in valid_tool_id_strategy()) {
        // Creating a valid tool ID should succeed
        let tool_id = ToolId::new(&tool_id_str);
        prop_assert!(tool_id.is_ok(), "Failed to create ToolId from: {}", tool_id_str);

        let tool_id = tool_id.expect("valid tool ID creation");

        // The tool ID should preserve the original string
        prop_assert_eq!(tool_id.as_str(), &tool_id_str);

        // Display should match the original string
        prop_assert_eq!(tool_id.to_string(), tool_id_str.clone());

        // Into string should preserve the value
        prop_assert_eq!(tool_id.clone().into_string(), tool_id_str.clone());

        // FromStr should work
        let parsed: Result<ToolId, _> = tool_id_str.parse();
        prop_assert!(parsed.is_ok());
        let parsed_tool_id = parsed.expect("valid parsing");
        prop_assert_eq!(parsed_tool_id.as_str(), &tool_id_str);

        // Cloning should preserve equality
        let cloned = tool_id.clone();
        prop_assert_eq!(tool_id.clone(), cloned);

        // Hash should be consistent
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        tool_id.hash(&mut hasher1);
        tool_id.hash(&mut hasher2);
        prop_assert_eq!(hasher1.finish(), hasher2.finish());
    }

    /// Test that invalid tool IDs always fail validation
    #[test]
    fn test_tool_id_validation_rejects_invalid(invalid_str in invalid_tool_id_strategy()) {
        let result = ToolId::new(&invalid_str);
        prop_assert!(result.is_err(), "Should reject invalid tool ID: {}", invalid_str);

        // The error should be the correct type
        match result.expect_err("should fail") {
            IcarusError::InvalidToolId(_) => {}, // Expected
            other => prop_assert!(false, "Wrong error type: {:?}", other),
        }
    }

    /// Test user ID invariants
    #[test]
    fn test_user_id_invariants(user_id_str in valid_user_id_strategy()) {
        let user_id = UserId::new(&user_id_str)?;

        // Basic invariants
        prop_assert!(!user_id.as_str().is_empty());
        prop_assert!(user_id.as_str().len() <= 255);
        prop_assert_eq!(user_id.as_str(), &user_id_str);

        // Serialization invariants
        let serialized = serde_json::to_string(&user_id)?;
        let deserialized: UserId = serde_json::from_str(&serialized)?;
        prop_assert_eq!(user_id, deserialized);
    }

    /// Test session ID invariants
    #[test]
    fn test_session_id_invariants(session_id_str in valid_session_id_strategy()) {
        let session_id = SessionId::new(&session_id_str)?;

        // Basic invariants
        prop_assert!(!session_id.as_str().is_empty());
        prop_assert!(session_id.as_str().len() <= 128);
        prop_assert_eq!(session_id.as_str(), &session_id_str);

        // Serialization invariants
        let serialized = serde_json::to_string(&session_id)?;
        let deserialized: SessionId = serde_json::from_str(&serialized)?;
        prop_assert_eq!(session_id, deserialized);
    }

    /// Test timestamp invariants and operations
    #[test]
    fn test_timestamp_invariants(nanos in timestamp_strategy()) {
        let timestamp = Timestamp::from_nanos(nanos);

        // Value preservation
        prop_assert_eq!(timestamp.as_nanos(), nanos);

        // Unit conversions should be consistent
        let secs = timestamp.as_secs();
        let millis = timestamp.as_millis();

        prop_assert_eq!(secs, nanos / 1_000_000_000);
        prop_assert_eq!(millis, nanos / 1_000_000);

        // Conversions should round-trip
        let from_u64: Timestamp = nanos.into();
        prop_assert_eq!(from_u64, timestamp);

        let back_to_u64: u64 = timestamp.into();
        prop_assert_eq!(back_to_u64, nanos);

        // Ordering should be consistent with the underlying value
        let other_nanos = if nanos == u64::MAX { nanos - 1 } else { nanos + 1 };
        let other_timestamp = Timestamp::from_nanos(other_nanos);

        use std::cmp::Ordering;
        match nanos.cmp(&other_nanos) {
            Ordering::Less => prop_assert!(timestamp < other_timestamp),
            Ordering::Greater => prop_assert!(timestamp > other_timestamp),
            Ordering::Equal => prop_assert_eq!(timestamp, other_timestamp),
        }

        // Display should not panic
        let _display = timestamp.to_string();
    }

    /// Test JSON-RPC request invariants
    #[test]
    fn test_json_rpc_request_invariants(
        (jsonrpc, method, params, id) in json_rpc_request_strategy()
    ) {
        let request = JsonRpcRequest::new(
            &jsonrpc,
            &method,
            params.as_deref().map(Cow::Borrowed),
            id.as_deref().map(Cow::Borrowed),
        )?;

        // Version should always be 2.0
        prop_assert_eq!(request.jsonrpc.as_ref(), "2.0");

        // Method should be preserved
        prop_assert_eq!(request.method.as_ref(), &method);

        // Notification detection should be consistent
        prop_assert_eq!(request.is_notification(), id.is_none());

        // Serialization should round-trip
        let serialized = serde_json::to_string(&request)?;
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized)?;
        prop_assert_eq!(request.method.as_ref(), deserialized.method.as_ref());
        prop_assert_eq!(request.jsonrpc.as_ref(), deserialized.jsonrpc.as_ref());
    }

    /// Test tool call invariants
    #[test]
    fn test_tool_call_invariants(
        tool_id_str in valid_tool_id_strategy(),
        session_id_str in valid_session_id_strategy(),
        args in prop::string::string_regex(r"\{[^{}]*\}").expect("Valid JSON regex")
    ) {
        let tool_id = ToolId::new(tool_id_str)?;
        let session_id = SessionId::new(session_id_str)?;

        let tool_call = ToolCall::new(tool_id.clone())
            .with_arguments(&args)
            .with_session(session_id.clone());

        // Tool ID should be preserved
        prop_assert_eq!(tool_call.name.clone(), tool_id);

        // Arguments should be preserved
        prop_assert_eq!(tool_call.arguments.as_ref(), &args);

        // Session ID should be preserved
        prop_assert_eq!(tool_call.session_id.clone(), Some(session_id));

        // Serialization should round-trip
        let serialized = serde_json::to_string(&tool_call)?;
        let deserialized: ToolCall = serde_json::from_str(&serialized)?;
        prop_assert_eq!(tool_call.name, deserialized.name);
        prop_assert_eq!(tool_call.session_id, deserialized.session_id);
    }

    /// Test tool result properties
    #[test]
    fn test_tool_result_properties(
        success_data in "[a-zA-Z0-9 ]{1,100}",
        error_message in "[a-zA-Z0-9 ]{1,100}",
        progress in 0u8..=100
    ) {
        // Success result properties
        let success_result = ToolResult::success(&success_data);
        prop_assert!(success_result.is_success());
        prop_assert!(!success_result.is_error());
        prop_assert!(!success_result.is_pending());

        // Error result properties
        let error_result = ToolResult::error(&error_message);
        prop_assert!(!error_result.is_success());
        prop_assert!(error_result.is_error());
        prop_assert!(!error_result.is_pending());

        // Pending result properties
        let pending_result = ToolResult::pending_with_progress(progress, "Working...");
        prop_assert!(!pending_result.is_success());
        prop_assert!(!pending_result.is_error());
        prop_assert!(pending_result.is_pending());

        // Conversion from Result<T, E>
        let ok_result: Result<String, String> = Ok(success_data.clone());
        let converted_success = ToolResult::from_result(ok_result);
        prop_assert!(converted_success.is_success());

        let err_result: Result<String, String> = Err(error_message.clone());
        let converted_error = ToolResult::from_result(err_result);
        prop_assert!(converted_error.is_error());
    }

    /// Test tool schema validation properties
    #[test]
    fn test_tool_schema_validation_properties(
        min_length in 0usize..=50,
        max_length in 51usize..=200,
        min_number in -1000.0f64..=0.0,
        max_number in 1.0f64..=1000.0
    ) {
        // String schema with valid ranges should validate
        let string_schema = ToolSchema::string_with_length(Some(min_length), Some(max_length));
        prop_assert!(string_schema.validate().is_ok());

        // Number schema with valid ranges should validate
        let number_schema = ToolSchema::number_range(Some(min_number), Some(max_number));
        prop_assert!(number_schema.validate().is_ok());

        // Array schema should validate if item schema is valid
        let array_schema = ToolSchema::array(ToolSchema::string());
        prop_assert!(array_schema.validate().is_ok());

        // Boolean schema should always validate
        let boolean_schema = ToolSchema::boolean();
        prop_assert!(boolean_schema.validate().is_ok());
    }

    /// Test session ID generation properties
    #[test]
    fn test_session_id_generation_properties(_seed in any::<u32>()) {
        // Generated session IDs should always be valid
        let session_id = SessionId::generate();

        // Should not be empty
        prop_assert!(!session_id.as_str().is_empty());

        // Should have the correct prefix
        prop_assert!(session_id.as_str().starts_with("sess_"));

        // Should be within length limits
        prop_assert!(session_id.as_str().len() <= 128);

        // Should be parseable
        let parsed = SessionId::new(session_id.as_str());
        prop_assert!(parsed.is_ok());
        prop_assert_eq!(parsed.expect("valid session ID"), session_id.clone());

        // Should serialize/deserialize correctly
        let serialized = serde_json::to_string(&session_id)?;
        let deserialized: SessionId = serde_json::from_str(&serialized)?;
        prop_assert_eq!(session_id, deserialized);
    }

    /// Test error message properties
    #[test]
    fn test_error_message_properties(
        tool_id_str in valid_tool_id_strategy(),
        user_id_str in valid_user_id_strategy(),
        message in "[a-zA-Z0-9 ]{1,100}"
    ) {
        let tool_id = ToolId::new(tool_id_str)?;
        let user_id = UserId::new(user_id_str)?;

        // Error messages should not be empty
        let tool_not_found = IcarusError::tool_not_found(tool_id.clone());
        prop_assert!(!tool_not_found.to_string().is_empty());

        let access_denied = IcarusError::access_denied(&message);
        prop_assert!(!access_denied.to_string().is_empty());

        let rate_limited = IcarusError::rate_limit_exceeded(user_id, &message);
        prop_assert!(!rate_limited.to_string().is_empty());

        // User messages should be user-friendly
        let user_message = tool_not_found.to_string();
        prop_assert!(!user_message.is_empty());
        prop_assert!(!user_message.contains("debug"));
        prop_assert!(!user_message.contains("stack"));
    }

    /// Test that zero-copy optimizations work correctly
    #[test]
    fn test_zero_copy_properties(
        test_str in "[a-zA-Z0-9 ]{1,50}"
    ) {
        // Test that Cow types preserve borrowing when possible
        let borrowed_cow = Cow::Borrowed(&test_str);
        let owned_cow: Cow<str> = Cow::Owned(test_str.clone());

        // Both should have the same logical content
        prop_assert_eq!(borrowed_cow.as_ref(), &test_str);
        prop_assert_eq!(owned_cow.as_ref(), &test_str);

        // Test in ToolCall context - both should produce the same serialization
        let tool_id = ToolId::new("test_tool")?;

        let borrowed_call = ToolCall::new(tool_id.clone())
            .with_arguments(borrowed_cow.as_ref());
        let owned_call = ToolCall::new(tool_id)
            .with_arguments(owned_cow.as_ref());

        // Both should serialize to the same JSON structure
        let borrowed_json = serde_json::to_string(&borrowed_call)?;
        let owned_json = serde_json::to_string(&owned_call)?;

        // Parse both back to compare structure
        let borrowed_parsed: serde_json::Value = serde_json::from_str(&borrowed_json)?;
        let owned_parsed: serde_json::Value = serde_json::from_str(&owned_json)?;

        // The arguments field should be the same since they contain the same content
        prop_assert_eq!(
            borrowed_parsed.get("arguments"),
            owned_parsed.get("arguments")
        );
    }
}

// Additional manual property tests for more complex scenarios

#[test]
fn test_tool_parameter_schema_consistency() {
    use proptest::test_runner::TestRunner;

    let mut runner = TestRunner::default();

    let strategy = (
        valid_tool_id_strategy(),
        prop::string::string_regex("[a-zA-Z ]{5,100}").expect("Valid description regex"),
        prop::collection::vec(
            (
                prop::string::string_regex("[a-zA-Z][a-zA-Z0-9_]{1,20}").expect("Valid param name"),
                prop::string::string_regex("[a-zA-Z ]{5,50}").expect("Valid param description"),
                prop_oneof![
                    Just(ToolSchema::string()),
                    Just(ToolSchema::number()),
                    Just(ToolSchema::integer()),
                    Just(ToolSchema::boolean()),
                ],
            ),
            0..10,
        ),
    );

    runner
        .run(&strategy, |(tool_id_str, description, params)| {
            let tool_id = ToolId::new(tool_id_str)?;
            let mut builder = Tool::builder().name(tool_id).description(description);

            for (param_name, param_desc, schema) in params {
                builder = builder.parameter(ToolParameter::new(param_name, param_desc, schema));
            }

            let tool = builder.build()?;

            // Tool should validate successfully
            tool.validate()?;

            // Should serialize/deserialize correctly
            let serialized = serde_json::to_string(&tool)?;
            let deserialized: Tool = serde_json::from_str(&serialized)?;

            assert_eq!(tool.name, deserialized.name);
            assert_eq!(tool.description, deserialized.description);
            assert_eq!(tool.parameters.len(), deserialized.parameters.len());

            Ok(())
        })
        .expect("property test should succeed");
}

#[test]
fn test_json_rpc_error_code_consistency() {
    use icarus_core::error::JsonRpcError;
    use proptest::test_runner::TestRunner;

    let mut runner = TestRunner::default();

    let strategy = (
        prop::string::string_regex("[a-zA-Z0-9 ]{1,100}").expect("Valid message regex"),
        prop::option::of(prop::string::string_regex(r"\{[^{}]*\}").expect("Valid data regex")),
        -32099i32..=-32000i32, // Server error range
    );

    runner
        .run(&strategy, |(message, data, error_code)| {
            // Test standard error codes
            let parse_error = JsonRpcError::parse_error(&message);
            assert_eq!(parse_error.code, -32700);
            assert!(!parse_error.message.is_empty());

            let invalid_request = JsonRpcError::invalid_request(&message);
            assert_eq!(invalid_request.code, -32600);

            let method_not_found = JsonRpcError::method_not_found("test_method");
            assert_eq!(method_not_found.code, -32601);
            assert!(method_not_found.message.contains("test_method"));

            let invalid_params = JsonRpcError::invalid_params(&message);
            assert_eq!(invalid_params.code, -32602);

            let internal_error = JsonRpcError::internal_error(&message);
            assert_eq!(internal_error.code, -32603);

            // Test server error with custom code
            let server_error = JsonRpcError::server_error(error_code, &message);
            assert_eq!(server_error.code, error_code);
            assert!((-32099..=-32000).contains(&server_error.code));

            // Test with data
            if let Some(data_str) = data {
                let error_with_data = JsonRpcError::with_data(error_code, &message, data_str);
                assert!(error_with_data.data.is_some());
                assert_eq!(error_with_data.code, error_code);
            }

            Ok(())
        })
        .expect("property test should succeed");
}
