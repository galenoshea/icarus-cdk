//! Streaming performance benchmarks
//!
//! Benchmarks for different streaming buffer sizes and operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use icarus_mcp::*;
use rand::RngCore;
use std::time::Duration;

/// Generate test data of various sizes
fn generate_test_data(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut data = vec![0u8; size];
    rng.fill_bytes(&mut data);
    data
}

/// Benchmark streaming response creation with different buffer sizes
fn benchmark_response_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_response_creation");

    // Test different data sizes
    for &size in &[1024, 4096, 16384, 65536, 262144] {
        let data = generate_test_data(size);

        group.throughput(Throughput::Bytes(size as u64));

        // Small buffer
        group.bench_with_input(BenchmarkId::new("small_buffer", size), &data, |b, data| {
            b.iter(|| {
                let mut response = black_box(StreamingResponse::<Small>::new());
                response.extend_from_slice(black_box(data));
                response
            });
        });

        // Default buffer
        group.bench_with_input(
            BenchmarkId::new("default_buffer", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let mut response = black_box(StreamingResponse::<DefaultBuffer>::new());
                    response.extend_from_slice(black_box(data));
                    response
                });
            },
        );

        // Large buffer
        group.bench_with_input(BenchmarkId::new("large_buffer", size), &data, |b, data| {
            b.iter(|| {
                let mut response = black_box(StreamingResponse::<Large>::new());
                response.extend_from_slice(black_box(data));
                response
            });
        });

        // Custom 1MB buffer
        group.bench_with_input(
            BenchmarkId::new("custom_1mb_buffer", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let mut response =
                        black_box(StreamingResponse::<CustomSize<{ 1024 * 1024 }>>::new());
                    response.extend_from_slice(black_box(data));
                    response
                });
            },
        );
    }

    group.finish();
}

/// Benchmark chunk reading performance
fn benchmark_chunk_reading(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_reading");
    group.measurement_time(Duration::from_secs(10));

    for &size in &[4096, 16384, 65536, 262144, 1048576] {
        let data = generate_test_data(size);

        group.throughput(Throughput::Bytes(size as u64));

        // Small buffer chunks
        group.bench_with_input(BenchmarkId::new("small_chunks", size), &data, |b, data| {
            b.iter(|| {
                let mut response = StreamingResponse::<Small>::new();
                response.extend_from_slice(data);

                let mut total_read = 0;
                while let Some(chunk) = response.next_chunk() {
                    total_read += black_box(chunk.len());
                }
                total_read
            });
        });

        // Large buffer chunks
        group.bench_with_input(BenchmarkId::new("large_chunks", size), &data, |b, data| {
            b.iter(|| {
                let mut response = StreamingResponse::<Large>::new();
                response.extend_from_slice(data);

                let mut total_read = 0;
                while let Some(chunk) = response.next_chunk() {
                    total_read += black_box(chunk.len());
                }
                total_read
            });
        });
    }

    group.finish();
}

/// Benchmark JSON parsing performance
fn benchmark_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    // Create test JSON data of various complexities
    let simple_json = r#"{"key": "value", "number": 42, "bool": true}"#;
    let complex_json = serde_json::json!({
        "users": [
            {"id": 1, "name": "Alice", "email": "alice@example.com", "metadata": {"role": "admin", "permissions": ["read", "write", "delete"]}},
            {"id": 2, "name": "Bob", "email": "bob@example.com", "metadata": {"role": "user", "permissions": ["read"]}},
            {"id": 3, "name": "Charlie", "email": "charlie@example.com", "metadata": {"role": "moderator", "permissions": ["read", "write"]}}
        ],
        "config": {
            "version": "1.2.3",
            "features": ["auth", "logging", "metrics"],
            "limits": {"max_users": 1000, "max_requests_per_minute": 100}
        }
    }).to_string();

    // Simple JSON parsing
    group.bench_function("simple_json", |b| {
        b.iter(|| {
            let mut response = StreamingResponse::<DefaultBuffer>::new();
            response.extend_from_slice(black_box(simple_json.as_bytes()));
            black_box(response.try_parse_json().unwrap())
        });
    });

    // Complex JSON parsing
    group.bench_function("complex_json", |b| {
        b.iter(|| {
            let mut response = StreamingResponse::<DefaultBuffer>::new();
            response.extend_from_slice(black_box(complex_json.as_bytes()));
            black_box(response.try_parse_json().unwrap())
        });
    });

    // Incremental JSON parsing (simulating streaming)
    group.bench_function("incremental_json", |b| {
        let json_bytes = complex_json.as_bytes();
        let chunk_size = 128;

        b.iter(|| {
            let mut response = StreamingResponse::<DefaultBuffer>::new();
            let mut result = None;

            for chunk in json_bytes.chunks(chunk_size) {
                response.extend_from_slice(black_box(chunk));
                if let Ok(Some(parsed)) = response.try_parse_json() {
                    result = Some(parsed);
                    break;
                }
            }
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark memory allocation patterns
fn benchmark_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    // Pre-allocated vs dynamic allocation
    group.bench_function("preallocated", |b| {
        b.iter(|| black_box(StreamingResponse::<DefaultBuffer>::with_capacity(65536)));
    });

    group.bench_function("dynamic_allocation", |b| {
        b.iter(|| black_box(StreamingResponse::<DefaultBuffer>::new()));
    });

    // Memory usage with known size
    group.bench_function("with_known_size", |b| {
        b.iter(|| black_box(StreamingResponse::<DefaultBuffer>::with_size(32768)));
    });

    group.finish();
}

criterion_group!(
    streaming_benches,
    benchmark_response_creation,
    benchmark_chunk_reading,
    benchmark_json_parsing,
    benchmark_memory_patterns
);

criterion_main!(streaming_benches);
