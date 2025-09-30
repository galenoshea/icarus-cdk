//! Performance benchmarks for icarus-core crate.
//!
//! These benchmarks measure the performance of critical code paths and ensure
//! they meet the performance targets specified in `rust_best_practices.md`.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use icarus_core::{
    error::{IcarusError, JsonRpcError},
    newtypes::{SessionId, Timestamp, ToolId, UserId},
    protocol::{JsonRpcRequest, JsonRpcResponse, ToolCall, ToolResult},
    tool::{Tool, ToolParameter, ToolSchema},
};
use std::borrow::Cow;

// Helper function to create test data
fn create_test_tool() -> Tool {
    Tool::builder()
        .name(ToolId::new("benchmark_tool").expect("benchmark test data should be valid"))
        .description("A tool for benchmarking performance")
        .parameter(ToolParameter::new(
            "input",
            "Input parameter",
            ToolSchema::string(),
        ))
        .parameter(ToolParameter::new(
            "count",
            "Count parameter",
            ToolSchema::integer(),
        ))
        .parameter(ToolParameter::optional(
            "optional",
            "Optional parameter",
            ToolSchema::boolean(),
        ))
        .build()
        .expect("benchmark test data should be valid")
}

fn create_test_tool_call() -> ToolCall<'static> {
    ToolCall::new(ToolId::new("test_tool").expect("benchmark test data should be valid"))
        .with_arguments(r#"{"input": "test_value", "count": 42, "optional": true}"#)
        .with_session(SessionId::generate())
        .with_metadata(r#"{"source": "benchmark"}"#)
}

// Benchmark newtype creation and validation
fn bench_newtype_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("newtype_creation");

    // ToolId creation
    group.bench_function("tool_id_valid", |b| {
        b.iter(|| {
            let tool_id = ToolId::new(black_box("valid_tool_name"));
            black_box(tool_id).expect("benchmark test data should be valid");
        });
    });

    group.bench_function("tool_id_invalid", |b| {
        b.iter(|| {
            let result = ToolId::new(black_box("invalid tool name"));
            black_box(result).expect_err("should fail");
        });
    });

    // UserId creation
    group.bench_function("user_id_creation", |b| {
        b.iter(|| {
            let user_id = UserId::new(black_box("user_123"));
            black_box(user_id).expect("benchmark test data should be valid");
        });
    });

    // SessionId generation (critical for performance)
    group.bench_function("session_id_generation", |b| {
        b.iter(|| {
            let session_id = SessionId::generate();
            black_box(session_id)
        });
    });

    // Timestamp operations
    group.bench_function("timestamp_now", |b| {
        b.iter(|| {
            let timestamp = Timestamp::now();
            black_box(timestamp)
        });
    });

    group.bench_function("timestamp_conversions", |b| {
        let timestamp = Timestamp::from_nanos(1_500_000_000_000);
        b.iter(|| {
            let secs = black_box(timestamp).as_secs();
            let millis = black_box(timestamp).as_millis();
            let nanos = black_box(timestamp).as_nanos();
            black_box((secs, millis, nanos))
        });
    });

    group.finish();
}

// Benchmark serialization performance
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    let tool = create_test_tool();
    let tool_call = create_test_tool_call();
    let tool_result = ToolResult::success_with_metadata("result_data", r#"{"meta": "data"}"#);

    // Tool serialization
    group.bench_function("tool_serialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&tool));
            black_box(json).expect("benchmark test data should be valid");
        });
    });

    group.bench_function("tool_deserialize", |b| {
        let json = serde_json::to_string(&tool).expect("benchmark test data should be valid");
        b.iter(|| {
            let deserialized: Tool = serde_json::from_str(black_box(&json))
                .expect("benchmark test data should be valid");
            black_box(deserialized)
        });
    });

    // ToolCall serialization
    group.bench_function("tool_call_serialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&tool_call));
            black_box(json).expect("benchmark test data should be valid");
        });
    });

    group.bench_function("tool_call_deserialize", |b| {
        let json = serde_json::to_string(&tool_call).expect("benchmark test data should be valid");
        b.iter(|| {
            let deserialized: ToolCall<'_> = serde_json::from_str(black_box(&json))
                .expect("benchmark test data should be valid");
            black_box(deserialized)
        });
    });

    // ToolResult serialization
    group.bench_function("tool_result_serialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&tool_result));
            black_box(json).expect("benchmark test data should be valid");
        });
    });

    group.bench_function("tool_result_deserialize", |b| {
        let json =
            serde_json::to_string(&tool_result).expect("benchmark test data should be valid");
        b.iter(|| {
            let deserialized: ToolResult = serde_json::from_str(black_box(&json))
                .expect("benchmark test data should be valid");
            black_box(deserialized)
        });
    });

    group.finish();
}

// Benchmark JSON-RPC protocol operations
fn bench_json_rpc_protocol(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_rpc_protocol");

    // Request creation
    group.bench_function("request_creation", |b| {
        b.iter(|| {
            let request = JsonRpcRequest::new(
                black_box("2.0"),
                black_box("test_method"),
                black_box(Some(r#"{"param": "value"}"#)).map(Cow::Borrowed),
                black_box(Some("req-123")).map(Cow::Borrowed),
            );
            black_box(request).expect("benchmark test data should be valid");
        });
    });

    // Response creation
    group.bench_function("response_creation_success", |b| {
        b.iter(|| {
            let response = JsonRpcResponse::success(black_box("result_data"), black_box("req-123"));
            black_box(response)
        });
    });

    group.bench_function("response_creation_error", |b| {
        b.iter(|| {
            let error = JsonRpcError::internal_error(black_box("Test error"));
            let response = JsonRpcResponse::error(error, black_box("req-123"));
            black_box(response)
        });
    });

    // Parameter extraction
    let request = JsonRpcRequest::new(
        "2.0",
        "test_method",
        Some(Cow::Borrowed(r#"{"name": "test", "value": 42}"#)),
        Some(Cow::Borrowed("req-123")),
    )
    .expect("benchmark test data should be valid");

    group.bench_function("parameter_extraction", |b| {
        b.iter(|| {
            let params: Result<serde_json::Value, _> = black_box(&request).extract_params();
            black_box(params).expect("benchmark test data should be valid")
        });
    });

    group.finish();
}

// Benchmark tool validation performance
fn bench_tool_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_validation");

    // Simple tool validation
    let simple_tool = Tool::builder()
        .name(ToolId::new("simple_tool").expect("benchmark test data should be valid"))
        .description("Simple tool")
        .parameter(ToolParameter::new(
            "param",
            "Parameter",
            ToolSchema::string(),
        ))
        .build()
        .expect("benchmark test data should be valid");

    group.bench_function("simple_tool_validation", |b| {
        b.iter(|| {
            let result = black_box(&simple_tool).validate();
            black_box(result).expect("benchmark test data should be valid");
        });
    });

    // Complex tool validation (many parameters)
    let mut complex_builder = Tool::builder()
        .name(ToolId::new("complex_tool").expect("benchmark test data should be valid"))
        .description("Complex tool with many parameters");

    for i in 0..20 {
        complex_builder = complex_builder.parameter(ToolParameter::new(
            format!("param_{i}"),
            format!("Parameter {i}"),
            match i % 4 {
                0 => ToolSchema::string(),
                1 => ToolSchema::number(),
                2 => ToolSchema::integer(),
                _ => ToolSchema::boolean(),
            },
        ));
    }

    let complex_tool = complex_builder
        .build()
        .expect("benchmark test data should be valid");

    group.bench_function("complex_tool_validation", |b| {
        b.iter(|| {
            let result = black_box(&complex_tool).validate();
            black_box(result).expect("benchmark test data should be valid");
        });
    });

    // Schema validation
    let complex_schema = ToolSchema::object(
        [
            (
                "name".to_string(),
                ToolSchema::string_with_length(Some(1), Some(100)),
            ),
            (
                "age".to_string(),
                ToolSchema::integer_range(Some(0), Some(150)),
            ),
            ("tags".to_string(), ToolSchema::array(ToolSchema::string())),
            (
                "metadata".to_string(),
                ToolSchema::object([("key".to_string(), ToolSchema::string())], ["key"]),
            ),
        ],
        ["name", "age"],
    );

    group.bench_function("complex_schema_validation", |b| {
        b.iter(|| {
            let result = black_box(&complex_schema).validate();
            black_box(result).expect("benchmark test data should be valid");
        });
    });

    group.finish();
}

// Benchmark error handling performance
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    let tool_id = ToolId::new("test_tool").expect("benchmark test data should be valid");
    let user_id = UserId::new("test_user").expect("benchmark test data should be valid");

    // Error creation
    group.bench_function("error_creation_tool_not_found", |b| {
        b.iter(|| {
            let error = IcarusError::tool_not_found(black_box(tool_id.clone()));
            black_box(error)
        });
    });

    group.bench_function("error_creation_rate_limited", |b| {
        b.iter(|| {
            let error = IcarusError::rate_limit_exceeded(
                black_box(user_id.clone()),
                black_box("Too many requests"),
            );
            black_box(error)
        });
    });

    // Error chaining
    group.bench_function("error_chaining", |b| {
        b.iter(|| {
            let inner = IcarusError::internal_error(black_box("Inner error"));
            let outer = IcarusError::tool_execution_failed(black_box(tool_id.clone()), inner);
            black_box(outer)
        });
    });

    // Error conversion
    group.bench_function("error_conversion_json", |b| {
        b.iter(|| {
            let json_error = serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "test error",
            ));
            let icarus_error: IcarusError = black_box(json_error).into();
            black_box(icarus_error)
        });
    });

    group.finish();
}

// Benchmark zero-copy operations
fn bench_zero_copy_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("zero_copy_operations");

    let borrowed_str = "test_borrowed_string";
    let owned_str = "test_owned_string".to_string();

    // Compare borrowed vs owned Cow operations
    group.bench_function("cow_borrowed", |b| {
        b.iter(|| {
            let cow = Cow::Borrowed(black_box(borrowed_str));
            let tool_call =
                ToolCall::new(ToolId::new("test").expect("benchmark test data should be valid"))
                    .with_arguments(cow);
            black_box(tool_call)
        });
    });

    group.bench_function("cow_owned", |b| {
        b.iter(|| {
            let cow = Cow::Owned(black_box(owned_str.clone()));
            let tool_call =
                ToolCall::new(ToolId::new("test").expect("benchmark test data should be valid"))
                    .with_arguments(cow);
            black_box(tool_call)
        });
    });

    // JSON-RPC request with zero-copy
    group.bench_function("json_rpc_borrowed", |b| {
        b.iter(|| {
            let request = JsonRpcRequest::new(
                black_box("2.0"),
                black_box("method"),
                black_box(Some(Cow::Borrowed(r#"{"test": true}"#))),
                black_box(Some(Cow::Borrowed("id-123"))),
            );
            black_box(request).expect("benchmark test data should be valid");
        });
    });

    group.bench_function("json_rpc_owned", |b| {
        b.iter(|| {
            let request = JsonRpcRequest::new(
                black_box("2.0".to_string()),
                black_box("method".to_string()),
                black_box(Some(Cow::Owned(r#"{"test": true}"#.to_string()))),
                black_box(Some(Cow::Owned("id-123".to_string()))),
            );
            black_box(request).expect("benchmark test data should be valid");
        });
    });

    group.finish();
}

// Benchmark memory allocation patterns
fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    // Tool creation (measures allocation overhead)
    group.bench_function("tool_creation_minimal", |b| {
        b.iter(|| {
            let tool = Tool::builder()
                .name(ToolId::new(black_box("tool")).expect("benchmark test data should be valid"))
                .description(black_box("Description"))
                .build();
            black_box(tool).expect("benchmark test data should be valid")
        });
    });

    group.bench_function("tool_creation_with_params", |b| {
        b.iter(|| {
            let tool = Tool::builder()
                .name(ToolId::new(black_box("tool")).expect("benchmark test data should be valid"))
                .description(black_box("Description"))
                .parameter(ToolParameter::new("p1", "Param 1", ToolSchema::string()))
                .parameter(ToolParameter::new("p2", "Param 2", ToolSchema::number()))
                .parameter(ToolParameter::new("p3", "Param 3", ToolSchema::boolean()))
                .build();
            black_box(tool).expect("benchmark test data should be valid")
        });
    });

    // Collection operations
    group.bench_function("parameter_collection", |b| {
        let tool = create_test_tool();
        b.iter(|| {
            let required = black_box(&tool).required_parameters();
            let optional = black_box(&tool).optional_parameters();
            black_box((required, optional))
        });
    });

    group.finish();
}

// Scalability benchmarks with different data sizes
fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");

    // Test tool validation with varying numbers of parameters
    for param_count in &[1, 5, 10, 25, 50] {
        group.bench_with_input(
            BenchmarkId::new("tool_validation", param_count),
            param_count,
            |b, &param_count| {
                let mut builder = Tool::builder()
                    .name(
                        ToolId::new("scalability_tool")
                            .expect("benchmark test data should be valid"),
                    )
                    .description("Tool for scalability testing");

                for i in 0..param_count {
                    builder = builder.parameter(ToolParameter::new(
                        format!("param_{i}"),
                        format!("Parameter {i}"),
                        ToolSchema::string(),
                    ));
                }

                let tool = builder
                    .build()
                    .expect("benchmark test data should be valid");

                b.iter(|| {
                    let result = black_box(&tool).validate();
                    black_box(result).expect("benchmark test data should be valid");
                });
            },
        );
    }

    // Test JSON serialization with varying data sizes
    for size in &[100, 500, 1000, 5000] {
        group.bench_with_input(
            BenchmarkId::new("json_serialization", size),
            size,
            |b, &size| {
                let large_data = "x".repeat(size);
                let tool_result = ToolResult::success(&large_data);

                b.iter(|| {
                    let json = serde_json::to_string(black_box(&tool_result));
                    black_box(json).expect("benchmark test data should be valid");
                });
            },
        );
    }

    group.finish();
}

// Benchmark real-world usage patterns
fn bench_real_world_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_patterns");

    // Complete MCP tool call workflow
    group.bench_function("complete_tool_call_workflow", |b| {
        b.iter(|| {
            // 1. Create tool call
            let tool_call = ToolCall::new(
                ToolId::new(black_box("workflow_tool"))
                    .expect("benchmark test data should be valid"),
            )
            .with_arguments(black_box(r#"{"input": "test", "count": 42}"#))
            .with_session(SessionId::generate());

            // 2. Create JSON-RPC request
            let params_json =
                serde_json::to_string(&tool_call).expect("benchmark test data should be valid");
            let request = JsonRpcRequest::new(
                "2.0",
                "tools/call",
                Some(Cow::Borrowed(params_json.as_str())),
                Some(Cow::Borrowed("req-123")),
            )
            .expect("benchmark test data should be valid");

            // 3. Extract parameters - just test the deserialization
            let params_str = request
                .params
                .as_ref()
                .expect("benchmark test data should be valid");
            let _extracted_call: ToolCall<'_> =
                serde_json::from_str(params_str).expect("benchmark test data should be valid");

            // 4. Create successful result
            let result = ToolResult::success("workflow_result");

            // 5. Create response
            let response = JsonRpcResponse::success(
                serde_json::to_string(&result).expect("benchmark test data should be valid"),
                "req-123",
            );

            black_box(response)
        });
    });

    // Error handling workflow
    group.bench_function("error_handling_workflow", |b| {
        b.iter(|| {
            // 1. Create error
            let tool_id = ToolId::new(black_box("failing_tool"))
                .expect("benchmark test data should be valid");
            let inner_error = IcarusError::internal_error(black_box("Something went wrong"));
            let error = IcarusError::tool_execution_failed(tool_id, inner_error);

            // 2. Convert to JSON-RPC error
            let json_rpc_error = JsonRpcError::internal_error(error.to_string());

            // 3. Create error response
            let response = JsonRpcResponse::error(json_rpc_error, black_box("req-123"));

            // 4. Serialize response
            let json =
                serde_json::to_string(&response).expect("benchmark test data should be valid");

            black_box(json)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_newtype_creation,
    bench_serialization,
    bench_json_rpc_protocol,
    bench_tool_validation,
    bench_error_handling,
    bench_zero_copy_operations,
    bench_memory_allocation,
    bench_scalability,
    bench_real_world_patterns,
);

criterion_main!(benches);
