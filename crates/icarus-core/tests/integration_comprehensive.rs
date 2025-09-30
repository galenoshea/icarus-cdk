//! Comprehensive integration tests for icarus-core crate.
//!
//! These tests verify that all components work together correctly and follow
//! the patterns from `rust_best_practices.md`.

use icarus_core::{
    error::{IcarusError, JsonRpcError},
    newtypes::{SessionId, Timestamp, ToolId, UserId},
    protocol::{JsonRpcRequest, JsonRpcResponse, ToolCall, ToolResult},
    tool::{Tool, ToolParameter, ToolSchema},
    Result,
};
use std::borrow::Cow;

#[test]
fn test_complete_tool_workflow() -> Result<()> {
    // Create a tool with various parameter types
    let tool = Tool::builder()
        .name(ToolId::new("calculator.add")?)
        .description("Adds two numbers together")
        .parameter(ToolParameter::new(
            "a",
            "First number",
            ToolSchema::number(),
        ))
        .parameter(ToolParameter::new(
            "b",
            "Second number",
            ToolSchema::number(),
        ))
        .parameter(ToolParameter::optional(
            "precision",
            "Decimal precision",
            ToolSchema::integer(),
        ))
        .metadata(r#"{"version": "1.0", "category": "math"}"#)
        .build()?;

    // Validate the tool
    tool.validate()?;

    // Create a tool call
    let session_id = SessionId::generate();
    let tool_call = ToolCall::new(tool.name.clone())
        .with_arguments(r#"{"a": 5.5, "b": 3.2, "precision": 2}"#)
        .with_session(session_id.clone())
        .with_metadata(r#"{"source": "test"}"#);

    // Create JSON-RPC request
    let request = JsonRpcRequest::new(
        "2.0",
        "tools/call",
        Some(serde_json::to_string(&tool_call)?.into()),
        Some("req-123".into()),
    )?;

    // Test parameter extraction
    if let Some(params) = &request.params {
        let extracted_call: ToolCall<'_> = serde_json::from_str(params)?;
        assert_eq!(extracted_call.name, tool.name);
        assert_eq!(extracted_call.session_id, Some(session_id));
    }

    // Test successful result
    let success_result = ToolResult::success_with_metadata("8.70", r#"{"execution_time_ms": 2.3}"#);

    let response = JsonRpcResponse::success(serde_json::to_string(&success_result)?, "req-123");

    assert!(response.is_success());

    Ok(())
}

#[test]
fn test_error_handling_integration() -> Result<()> {
    // Test various error scenarios and their propagation

    // Invalid tool ID
    let invalid_tool = ToolId::new("");
    assert!(invalid_tool.is_err());
    match invalid_tool.expect_err("should fail") {
        IcarusError::InvalidToolId(_) => (),
        _ => panic!("Expected InvalidToolId error"),
    }

    // Invalid user ID
    let invalid_user = UserId::new("");
    assert!(invalid_user.is_err());

    // Invalid JSON-RPC request
    let invalid_request = JsonRpcRequest::new("1.0", "test", None, None);
    assert!(invalid_request.is_err());

    // Tool with invalid schema
    let invalid_tool = Tool::builder()
        .name(ToolId::new("test")?)
        .description("Test tool")
        .parameter(ToolParameter::new(
            "param",
            "Test param",
            ToolSchema::string_with_length(Some(10), Some(5)),
        ))
        .build();

    assert!(invalid_tool.is_err());

    // Test error chaining
    let tool_id = ToolId::new("failing_tool")?;
    let inner_error = IcarusError::internal_error("Something went wrong");
    let outer_error = IcarusError::tool_execution_failed(tool_id, inner_error);

    assert!(outer_error.to_string().contains("failing_tool"));

    Ok(())
}

#[test]
fn test_json_rpc_protocol_compliance() -> Result<()> {
    // Test JSON-RPC 2.0 specification compliance

    // Valid request with all fields
    let request = JsonRpcRequest::new(
        "2.0",
        "test_method",
        Some(r#"{"param1": "value1"}"#.into()),
        Some("req-001".into()),
    )?;

    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "test_method");
    assert!(!request.is_notification());

    // Notification (no ID)
    let notification = JsonRpcRequest::new("2.0", "notification_method", Some(r"{}".into()), None)?;

    assert!(notification.is_notification());

    // Success response
    let success_response = JsonRpcResponse::success("result_data", "req-001");
    assert!(success_response.is_success());
    assert_eq!(success_response.jsonrpc, "2.0");

    // Error response with all JSON-RPC error codes
    let parse_error = JsonRpcError::parse_error("Invalid JSON");
    assert_eq!(parse_error.code, -32700);

    let invalid_request = JsonRpcError::invalid_request("Invalid request format");
    assert_eq!(invalid_request.code, -32600);

    let method_not_found = JsonRpcError::method_not_found("unknown_method");
    assert_eq!(method_not_found.code, -32601);

    let invalid_params = JsonRpcError::invalid_params("Missing required params");
    assert_eq!(invalid_params.code, -32602);

    let internal_error = JsonRpcError::internal_error("Server error");
    assert_eq!(internal_error.code, -32603);

    // Server error range
    let server_error = JsonRpcError::server_error(-32099, "Custom server error");
    assert_eq!(server_error.code, -32099);

    Ok(())
}

#[test]
fn test_tool_schema_validation() -> Result<()> {
    // Test comprehensive schema validation

    // String schema with constraints
    let string_schema = ToolSchema::string_with_length(Some(5), Some(100));
    string_schema.validate()?;

    // Number schema with range
    let number_schema = ToolSchema::number_range(Some(0.0), Some(100.0));
    number_schema.validate()?;

    // Integer schema with range
    let integer_schema = ToolSchema::integer_range(Some(1), Some(10));
    integer_schema.validate()?;

    // Array schema
    let array_schema = ToolSchema::array(ToolSchema::string());
    array_schema.validate()?;

    // Object schema with required and optional properties
    let object_schema = ToolSchema::object(
        [
            ("name".to_string(), ToolSchema::string()),
            ("age".to_string(), ToolSchema::integer()),
            ("active".to_string(), ToolSchema::boolean()),
        ],
        ["name", "age"],
    );
    object_schema.validate()?;

    // Nested schemas
    let nested_schema = ToolSchema::array(ToolSchema::object(
        [("id".to_string(), ToolSchema::integer())],
        ["id"],
    ));
    nested_schema.validate()?;

    // Enum schema
    let enum_schema = ToolSchema::string_enum(["red", "green", "blue"]);
    enum_schema.validate()?;

    Ok(())
}

#[test]
fn test_tool_parameter_combinations() -> Result<()> {
    // Test various parameter combinations

    let tool = Tool::builder()
        .name(ToolId::new("complex_tool")?)
        .description("A tool with various parameter types")
        .parameter(ToolParameter::new(
            "required_string",
            "Required string param",
            ToolSchema::string(),
        ))
        .parameter(ToolParameter::new(
            "required_number",
            "Required number param",
            ToolSchema::number(),
        ))
        .parameter(ToolParameter::optional(
            "optional_integer",
            "Optional integer param",
            ToolSchema::integer(),
        ))
        .parameter(ToolParameter::with_default(
            "default_boolean",
            "Boolean with default",
            ToolSchema::boolean(),
            "true",
        ))
        .parameter(ToolParameter::new(
            "string_enum",
            "String enum param",
            ToolSchema::string_enum(["option1", "option2", "option3"]),
        ))
        .parameter(ToolParameter::new(
            "number_range",
            "Number with range",
            ToolSchema::number_range(Some(0.0), Some(100.0)),
        ))
        .parameter(ToolParameter::new(
            "array_param",
            "Array parameter",
            ToolSchema::array(ToolSchema::string()),
        ))
        .parameter(ToolParameter::new(
            "object_param",
            "Object parameter",
            ToolSchema::object([("key1".to_string(), ToolSchema::string())], ["key1"]),
        ))
        .build()?;

    // Validate the tool
    tool.validate()?;

    // Check parameter categorization
    let required_params = tool.required_parameters();
    let optional_params = tool.optional_parameters();

    assert_eq!(required_params.len(), 6); // All except the optional ones
    assert_eq!(optional_params.len(), 2); // optional_integer and default_boolean

    // Test parameter lookup
    assert!(tool.find_parameter("required_string").is_some());
    assert!(tool.find_parameter("nonexistent").is_none());

    Ok(())
}

#[test]
fn test_zero_copy_optimization() -> Result<()> {
    // Test that Cow types work correctly for zero-copy optimization

    // Test with borrowed strings
    let borrowed_request = JsonRpcRequest::new(
        "2.0",
        "borrowed_method",
        Some(Cow::Borrowed(r#"{"borrowed": true}"#)),
        Some(Cow::Borrowed("borrowed-id")),
    )?;

    assert!(matches!(borrowed_request.method, Cow::Borrowed(_)));
    assert!(matches!(borrowed_request.params, Some(Cow::Borrowed(_))));

    // Test with owned strings
    let owned_request = JsonRpcRequest::new(
        "2.0".to_string(),
        "owned_method".to_string(),
        Some(Cow::Owned(r#"{"owned": true}"#.to_string())),
        Some(Cow::Owned("owned-id".to_string())),
    )?;

    assert!(matches!(owned_request.method, Cow::Owned(_)));
    assert!(matches!(owned_request.params, Some(Cow::Owned(_))));

    // Test tool call with zero-copy
    let tool_id = ToolId::new("zero_copy_tool")?;
    let tool_call = ToolCall::new(tool_id)
        .with_arguments(Cow::Borrowed(r#"{"test": "borrowed"}"#))
        .with_metadata(Cow::Borrowed(r#"{"meta": "borrowed"}"#));

    assert!(matches!(tool_call.arguments, Cow::Borrowed(_)));
    assert!(matches!(tool_call.metadata, Some(Cow::Borrowed(_))));

    Ok(())
}

#[test]
fn test_tool_result_conversions() -> Result<()> {
    // Test various ToolResult conversion patterns

    // From Result<T, E>
    let ok_result: std::result::Result<String, &str> = Ok("success".to_string());
    let tool_result = ToolResult::from_result(ok_result);
    assert!(tool_result.is_success());

    let err_result: std::result::Result<String, &str> = Err("failure");
    let tool_result = ToolResult::from_result(err_result);
    assert!(tool_result.is_error());

    // From IcarusError
    let icarus_error = IcarusError::internal_error("test error");
    let tool_result: ToolResult = icarus_error.into();
    assert!(tool_result.is_error());

    // Test result extraction
    let success_result = ToolResult::success("test_value");
    let extracted = success_result.into_success()?;
    assert_eq!(extracted, "test_value");

    // Test pending results
    let pending = ToolResult::pending_with_progress(75, "Processing...");
    assert!(pending.is_pending());
    assert!(!pending.is_success());
    assert!(!pending.is_error());

    Ok(())
}

#[test]
fn test_timestamp_precision_and_ordering() {
    // Test timestamp precision and ordering behavior

    let ts1 = Timestamp::from_nanos(1_000_000_000); // 1 second
    let ts2 = Timestamp::from_nanos(1_500_000_000); // 1.5 seconds
    let ts3 = Timestamp::from_nanos(2_000_000_000); // 2 seconds

    // Test ordering
    assert!(ts1 < ts2);
    assert!(ts2 < ts3);
    assert!(ts1 < ts3);

    // Test conversions
    assert_eq!(ts2.as_secs(), 1);
    assert_eq!(ts2.as_millis(), 1500);
    assert_eq!(ts2.as_nanos(), 1_500_000_000);

    // Test display format
    let display_str = ts1.to_string();
    assert!(display_str.contains("1970")); // Unix epoch start

    // Test current time generation
    let now1 = Timestamp::now();
    let now2 = Timestamp::now();
    assert!(now1 <= now2); // Should be monotonic or equal
}

#[test]
fn test_session_id_uniqueness_and_format() {
    // Test session ID uniqueness and format consistency

    let mut session_ids = std::collections::HashSet::new();

    // Generate multiple session IDs and ensure uniqueness
    for _ in 0..100 {
        let session_id = SessionId::generate();
        let id_str = session_id.as_str();

        // Check format
        assert!(id_str.starts_with("sess_"));
        assert!(id_str.len() > 10); // Should be reasonably long

        // Check uniqueness
        assert!(
            session_ids.insert(session_id),
            "Duplicate session ID generated"
        );
    }
}

#[test]
fn test_comprehensive_serialization() -> Result<()> {
    // Test that all types can be serialized and deserialized correctly

    // Test tool serialization
    let tool = Tool::builder()
        .name(ToolId::new("serialization_test")?)
        .description("Test tool for serialization")
        .parameter(ToolParameter::new(
            "param1",
            "Parameter 1",
            ToolSchema::string(),
        ))
        .build()?;

    let serialized = serde_json::to_string(&tool)?;
    let deserialized: Tool = serde_json::from_str(&serialized)?;
    assert_eq!(tool.name, deserialized.name);
    assert_eq!(tool.description, deserialized.description);

    // Test JSON-RPC request/response serialization
    let request = JsonRpcRequest::new(
        "2.0",
        "test_method",
        Some(r#"{"test": true}"#.into()),
        Some("test-id".into()),
    )?;

    let serialized = serde_json::to_string(&request)?;
    let deserialized: JsonRpcRequest = serde_json::from_str(&serialized)?;
    assert_eq!(request.method, deserialized.method);

    // Test tool call serialization
    let tool_call = ToolCall::new(ToolId::new("test_tool")?)
        .with_arguments(r#"{"arg": "value"}"#)
        .with_session(SessionId::new("test_session")?);

    let serialized = serde_json::to_string(&tool_call)?;
    let deserialized: ToolCall = serde_json::from_str(&serialized)?;
    assert_eq!(tool_call.name, deserialized.name);

    Ok(())
}

#[test]
fn test_error_severity_and_classification() {
    use icarus_core::error::IcarusError;

    // Test different error types and their user messages
    let errors = vec![
        IcarusError::tool_not_found(ToolId::new("missing").expect("valid tool ID")),
        IcarusError::access_denied("Insufficient permissions"),
        IcarusError::internal_error("System failure"),
        IcarusError::rate_limit_exceeded(
            UserId::new("user1").expect("valid user ID"),
            "Too many requests",
        ),
    ];

    for error in errors {
        let user_message = error.to_string();
        assert!(!user_message.is_empty());

        // User messages should not contain technical details
        assert!(!user_message.contains("stack trace"));
        assert!(!user_message.contains("internal"));
    }
}

#[test]
fn test_tool_builder_validation() -> Result<()> {
    // Test tool builder validation and error handling

    // Missing required fields
    let incomplete_tool = Tool::builder().build();
    assert!(incomplete_tool.is_err());

    // Tool with too many parameters
    let mut builder = Tool::builder()
        .name(ToolId::new("many_params")?)
        .description("Tool with many parameters");

    // Add maximum allowed parameters
    for i in 0..icarus_core::MAX_PARAMETER_COUNT {
        builder = builder.parameter(ToolParameter::new(
            format!("param_{i}"),
            format!("Parameter {i}"),
            ToolSchema::string(),
        ));
    }

    let tool_with_max_params = builder.build();
    assert!(tool_with_max_params.is_ok());

    // Test by creating a builder with too many params directly
    let mut builder_with_too_many = Tool::builder()
        .name(ToolId::new("too_many_params")?)
        .description("Tool with too many parameters");

    for i in 0..=icarus_core::MAX_PARAMETER_COUNT {
        builder_with_too_many = builder_with_too_many.parameter(ToolParameter::new(
            format!("param_{i}"),
            format!("Parameter {i}"),
            ToolSchema::string(),
        ));
    }

    let result = builder_with_too_many.build();
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_concurrent_operations() -> Result<()> {
    use std::sync::Arc;
    use std::thread;

    // Test that types are Send + Sync for concurrent usage
    let tool = Arc::new(
        Tool::builder()
            .name(ToolId::new("concurrent_tool")?)
            .description("Tool for concurrency testing")
            .parameter(ToolParameter::new(
                "input",
                "Input parameter",
                ToolSchema::string(),
            ))
            .build()?,
    );

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let tool_clone = Arc::clone(&tool);
            thread::spawn(move || {
                // Each thread can access the tool
                let param = tool_clone.find_parameter("input");
                assert!(param.is_some());

                // Create tool calls in parallel
                let tool_call = ToolCall::new(tool_clone.name.clone())
                    .with_arguments(format!(r#"{{"thread": {i}}}"#));

                assert_eq!(tool_call.name, tool_clone.name);
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("thread should complete");
    }

    Ok(())
}
