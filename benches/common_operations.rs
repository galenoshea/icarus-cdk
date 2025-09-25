//! Comprehensive benchmark suite for Icarus CDK common operations
//!
//! This benchmark suite measures performance of key operations including:
//! - MCP protocol operations (tool registration, execution, session management)
//! - Canister operations (storage, serialization, state management)
//! - Memory management (allocation, buffer pooling, streaming)
//! - Bridge operations (request routing, identity management)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark configuration and utilities
mod bench_utils {
    use super::*;

    pub struct BenchContext {
        pub runtime: Runtime,
        #[allow(dead_code)] // Reserved for future canister profiling features
        pub canister_id: String,
    }

    impl BenchContext {
        pub fn new() -> Self {
            let runtime = Runtime::new().unwrap();
            let canister_id = "rdmx6-jaaaa-aaaaa-aaadq-cai".to_string();

            Self {
                runtime,
                canister_id,
            }
        }
    }

    /// Generate test data for benchmarks
    pub fn generate_tool_data(size: usize) -> Vec<Value> {
        (0..size)
            .map(|i| {
                json!({
                    "name": format!("test_tool_{}", i),
                    "description": format!("Test tool number {}", i),
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "input": {"type": "string", "description": "Test input parameter"}
                        },
                        "required": ["input"]
                    }
                })
            })
            .collect()
    }

    /// Generate test messages for streaming benchmarks
    pub fn generate_messages(count: usize, message_size: usize) -> Vec<String> {
        let message = "x".repeat(message_size);
        (0..count).map(|_| message.clone()).collect()
    }
}

use bench_utils::*;

/// Benchmark MCP protocol operations
fn bench_mcp_operations(c: &mut Criterion) {
    let ctx = BenchContext::new();
    let mut group = c.benchmark_group("mcp_operations");

    // Tool registration performance
    group.bench_function("tool_registration_single", |b| {
        let tool_data = generate_tool_data(1);
        b.iter(|| {
            ctx.runtime.block_on(async {
                // Simulate tool registration process
                let _result = black_box(serde_json::to_string(&tool_data[0]).unwrap());
            })
        })
    });

    // Batch tool registration
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("tool_registration_batch", size),
            size,
            |b, &size| {
                let tool_data = generate_tool_data(size);
                b.iter(|| {
                    ctx.runtime.block_on(async {
                        for tool in &tool_data {
                            let _result = black_box(serde_json::to_string(tool).unwrap());
                        }
                    })
                })
            },
        );
    }

    // Tool execution simulation
    group.bench_function("tool_execution_basic", |b| {
        b.iter(|| {
            ctx.runtime.block_on(async {
                let request = json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": {
                        "name": "test_tool",
                        "arguments": {"input": "test_data"}
                    }
                });
                let _result = black_box(serde_json::to_string(&request).unwrap());
            })
        })
    });

    // Session management
    group.bench_function("session_lifecycle", |b| {
        b.iter(|| {
            ctx.runtime.block_on(async {
                let session_id = format!("session_{}", fastrand::u64(..));
                let _create = black_box(session_id.clone());
                let _update = black_box(format!("{}_updated", session_id));
                black_box(());
            })
        })
    });

    group.finish();
}

/// Benchmark streaming and memory operations
fn bench_streaming_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_operations");

    // Message throughput benchmarks
    for message_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*message_count as u64));
        group.bench_with_input(
            BenchmarkId::new("streaming_throughput", message_count),
            message_count,
            |b, &count| {
                let messages = generate_messages(count, 100); // 100 byte messages
                b.iter(|| {
                    for msg in &messages {
                        let _response = black_box("simulated streaming response");
                        let _serialized = black_box(msg.as_bytes());
                    }
                })
            },
        );
    }

    // Buffer pool performance
    group.bench_function("buffer_pool_allocation", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let buffer: Vec<u8> = black_box(Vec::with_capacity(4096));
                let _result = black_box(buffer);
            }
        })
    });

    // Serialization performance
    for data_size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("json_serialization", data_size),
            data_size,
            |b, &size| {
                let data: HashMap<String, String> = (0..size)
                    .map(|i| (format!("key_{}", i), format!("value_{}", i)))
                    .collect();

                b.iter(|| {
                    let _json = black_box(serde_json::to_string(&data).unwrap());
                })
            },
        );
    }

    group.finish();
}

/// Benchmark canister storage operations
fn bench_storage_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_operations");

    // Stable memory simulation (using Vec as proxy)
    group.bench_function("stable_memory_write", |b| {
        b.iter(|| {
            let mut storage = Vec::new();
            for i in 0..1000 {
                let data = format!("data_entry_{}", i);
                storage.push(black_box(data));
            }
            black_box(storage);
        })
    });

    group.bench_function("stable_memory_read", |b| {
        let storage: Vec<String> = (0..1000).map(|i| format!("data_entry_{}", i)).collect();

        b.iter(|| {
            for item in &storage {
                let _read = black_box(item.clone());
            }
        })
    });

    // State serialization
    group.bench_function("state_serialization", |b| {
        let state = json!({
            "sessions": {},
            "tools": {},
            "config": {
                "max_concurrent_tools": 10,
                "timeout_ms": 30000
            },
            "metrics": {
                "total_requests": 12345,
                "total_errors": 42,
                "uptime_ms": 3600000
            }
        });

        b.iter(|| {
            let _serialized = black_box(serde_json::to_vec(&state).unwrap());
        })
    });

    group.finish();
}

/// Benchmark authentication and security operations
fn bench_auth_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("auth_operations");

    // Principal validation simulation
    group.bench_function("principal_validation", |b| {
        let principals = vec![
            "rdmx6-jaaaa-aaaaa-aaadq-cai",
            "rrkah-fqaaa-aaaaa-aaaaq-cai",
            "rno2w-sqaaa-aaaah-qcnwa-cai",
        ];

        b.iter(|| {
            for principal in &principals {
                let _valid = black_box(principal.len() == 27); // Simplified validation
            }
        })
    });

    // Session token generation simulation
    group.bench_function("session_token_generation", |b| {
        b.iter(|| {
            let session_id = black_box(format!("session_{}", fastrand::u64(..)));
            let timestamp = black_box(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            let _token = black_box(format!("{}:{}", session_id, timestamp));
        })
    });

    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    let ctx = BenchContext::new();

    // Simulate concurrent tool executions
    group.bench_function("concurrent_tool_calls", |b| {
        b.iter(|| {
            ctx.runtime.block_on(async {
                let tasks: Vec<_> = (0..10)
                    .map(|i| {
                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_millis(1)).await;
                            black_box(format!("result_{}", i))
                        })
                    })
                    .collect();

                let _results: Vec<_> = futures::future::join_all(tasks)
                    .await
                    .into_iter()
                    .collect::<Vec<_>>();
            })
        })
    });

    // Concurrent session management
    group.bench_function("concurrent_sessions", |b| {
        b.iter(|| {
            ctx.runtime.block_on(async {
                let sessions: Vec<_> = (0..50).map(|i| format!("session_{}", i)).collect();

                let tasks: Vec<_> = sessions
                    .iter()
                    .map(|session| {
                        let session = session.clone();
                        tokio::spawn(async move {
                            // Simulate session operation
                            tokio::time::sleep(Duration::from_micros(100)).await;
                            black_box(format!("{}_processed", session))
                        })
                    })
                    .collect();

                let _results: Vec<_> = futures::future::join_all(tasks)
                    .await
                    .into_iter()
                    .collect::<Vec<_>>();
            })
        })
    });

    group.finish();
}

/// Benchmark error handling and recovery
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    // Error creation and formatting
    group.bench_function("error_creation", |b| {
        b.iter(|| {
            for i in 0..100 {
                let error = black_box(format!("Error #{}: Something went wrong", i));
                let _json_error = black_box(json!({
                    "jsonrpc": "2.0",
                    "id": i,
                    "error": {
                        "code": -32603,
                        "message": error
                    }
                }));
            }
        })
    });

    // Error recovery simulation
    group.bench_function("error_recovery", |b| {
        b.iter(|| {
            for _ in 0..100 {
                // Simulate failed operation
                let result: std::result::Result<String, &str> = Err("Simulated error");

                let _recovered = black_box(match result {
                    Ok(val) => val,
                    Err(e) => format!("Recovered from: {}", e),
                });
            }
        })
    });

    group.finish();
}

/// Real-world scenario benchmarks
fn bench_end_to_end_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end_scenarios");
    let ctx = BenchContext::new();

    // Complete MCP request/response cycle
    group.bench_function("full_mcp_cycle", |b| {
        b.iter(|| {
            ctx.runtime.block_on(async {
                // 1. Parse request
                let request = black_box(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/list"
                }));

                // 2. Process request
                let _parsed = black_box(serde_json::to_string(&request).unwrap());

                // 3. Execute (simulated)
                let tools = black_box(vec!["tool1", "tool2", "tool3"]);

                // 4. Format response
                let response = black_box(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": {
                        "tools": tools
                    }
                }));

                // 5. Serialize response
                let _serialized = black_box(serde_json::to_string(&response).unwrap());
            })
        })
    });

    // Canister upgrade simulation
    group.bench_function("canister_upgrade_simulation", |b| {
        b.iter(|| {
            ctx.runtime.block_on(async {
                // 1. Save current state
                let state = black_box(json!({
                    "sessions": {"session1": "data1"},
                    "tools": {"tool1": "metadata1"}
                }));
                let _serialized_state = black_box(serde_json::to_vec(&state).unwrap());

                // 2. Simulate upgrade
                tokio::time::sleep(Duration::from_micros(10)).await;

                // 3. Restore state
                let _restored_state =
                    black_box(serde_json::from_slice::<Value>(&_serialized_state).unwrap());
            })
        })
    });

    group.finish();
}

// Criterion benchmark groups
criterion_group!(
    benches,
    bench_mcp_operations,
    bench_streaming_operations,
    bench_storage_operations,
    bench_auth_operations,
    bench_concurrent_operations,
    bench_error_handling,
    bench_end_to_end_scenarios
);

criterion_main!(benches);
