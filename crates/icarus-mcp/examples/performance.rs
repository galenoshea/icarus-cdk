//! Performance Optimization Example
//!
//! Demonstrates performance optimization techniques including profiling,
//! SIMD operations, zero-copy serialization, and efficient allocators.
//!
//! Run with: cargo run --example performance --features=all --release

use anyhow::Result;
use icarus_mcp::storage::profile::PerformanceProfiler;

#[cfg(feature = "storage")]
use icarus_mcp::storage::{allocator::get_pooled_buffer, zerocopy::ZeroCopySerializer};

#[cfg(feature = "simd")]
use icarus_mcp::storage::simd::SimdProcessor;

#[cfg(feature = "streaming")]
use icarus_mcp::storage::streaming::{Large, StreamingResponse};

use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkData {
    id: u64,
    name: String,
    values: Vec<f64>,
    metadata: std::collections::HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("performance=info")
        .init();

    println!("⚡ Performance Optimization Examples");

    // Initialize profiler
    let profiler = PerformanceProfiler::new();

    // Generate test data
    let test_data = generate_test_data(1000);
    println!("Generated {} test records", test_data.len());

    // Benchmark 1: Serialization performance
    println!("\n🔄 Benchmark 1: Serialization Performance");
    benchmark_serialization(&test_data, &profiler).await?;

    // Benchmark 2: Memory allocation patterns
    #[cfg(feature = "storage")]
    {
        println!("\n💾 Benchmark 2: Memory Allocation Performance");
        benchmark_allocation(&profiler).await?;
    }

    // Benchmark 3: SIMD operations
    #[cfg(feature = "simd")]
    {
        println!("\n⚡ Benchmark 3: SIMD Operations");
        benchmark_simd_operations(&profiler).await?;
    }

    // Benchmark 4: Streaming performance
    #[cfg(feature = "streaming")]
    {
        println!("\n🌊 Benchmark 4: Streaming Performance");
        benchmark_streaming(&test_data, &profiler).await?;
    }

    // Generate performance report
    println!("\n📊 Performance Report");
    let report = profiler.get_statistics();

    println!("Function Performance (top 5):");
    let mut sorted_functions: Vec<_> = report.function_stats.iter().collect();
    sorted_functions.sort_by(|a, b| b.1.avg_time_nanos.cmp(&a.1.avg_time_nanos));

    for (name, stats) in sorted_functions.iter().take(5) {
        println!(
            "  {}: {:.2}ms avg ({} calls)",
            name,
            stats.avg_time_nanos as f64 / 1_000_000.0,
            stats.call_count
        );
    }

    if !report.allocation_summary.is_empty() {
        println!("\nAllocation Patterns:");
        for (func, alloc) in &report.allocation_summary {
            println!(
                "  {}: {} allocations, {:.2}KB avg size",
                func,
                alloc.allocation_count,
                alloc.avg_size as f64 / 1024.0
            );
        }
    }

    if !report.hot_path_summary.is_empty() {
        println!("\nHot Paths:");
        for (path, stats) in &report.hot_path_summary {
            let avg_time_ms = stats.total_time_nanos as f64 / 1_000_000.0;
            println!(
                "  {}: {:.2}ms total ({} executions)",
                path, avg_time_ms, stats.execution_count
            );
        }
    }

    println!("\n✨ Performance benchmarks completed!");

    Ok(())
}

async fn benchmark_serialization(
    test_data: &[BenchmarkData],
    profiler: &PerformanceProfiler,
) -> Result<()> {
    // Standard serde_json
    let start = Instant::now();
    let _guard = profiler.start_timer("serialize_standard");
    let mut total_size = 0;

    for data in test_data {
        let serialized = serde_json::to_vec(data)?;
        total_size += serialized.len();
    }

    drop(_guard);
    let standard_duration = start.elapsed();

    // Zero-copy serialization (if available)
    #[cfg(feature = "storage")]
    {
        let start = Instant::now();
        let _guard = profiler.start_timer("serialize_zerocopy");
        let mut serializer = ZeroCopySerializer::new().compact();
        let mut zero_copy_size = 0;

        for data in test_data {
            let serialized = serializer.serialize(data)?;
            zero_copy_size += serialized.len();
        }

        drop(_guard);
        let zero_copy_duration = start.elapsed();

        println!(
            "Standard serde_json: {:?} ({} bytes)",
            standard_duration, total_size
        );
        println!(
            "Zero-copy: {:?} ({} bytes)",
            zero_copy_duration, zero_copy_size
        );
        println!(
            "Speedup: {:.2}x",
            standard_duration.as_secs_f64() / zero_copy_duration.as_secs_f64()
        );
    }

    #[cfg(not(feature = "storage"))]
    {
        println!(
            "Standard serde_json: {:?} ({} bytes)",
            standard_duration, total_size
        );
        println!("Zero-copy serialization disabled - enable 'storage' feature");
    }

    Ok(())
}

#[cfg(feature = "storage")]
async fn benchmark_allocation(profiler: &PerformanceProfiler) -> Result<()> {
    let iterations = 10000;

    // Standard allocation
    let start = Instant::now();
    let _guard = profiler.start_timer("allocate_standard");

    for _ in 0..iterations {
        let mut buffer = Vec::with_capacity(1024);
        buffer.extend_from_slice(&vec![0u8; 1024]);
        // Buffer is dropped here
    }

    drop(_guard);
    let standard_duration = start.elapsed();

    // Pooled allocation
    let start = Instant::now();
    let _guard = profiler.start_timer("allocate_pooled");

    for _ in 0..iterations {
        let mut buffer = get_pooled_buffer(1024);
        buffer.extend_from_slice(&vec![0u8; 1024]);
        // Buffer is automatically returned to pool when dropped
    }

    drop(_guard);
    let pooled_duration = start.elapsed();

    println!("Standard allocation: {:?}", standard_duration);
    println!("Pooled allocation: {:?}", pooled_duration);
    println!(
        "Speedup: {:.2}x",
        standard_duration.as_secs_f64() / pooled_duration.as_secs_f64()
    );

    Ok(())
}

#[cfg(feature = "simd")]
async fn benchmark_simd_operations(profiler: &PerformanceProfiler) -> Result<()> {
    // Generate large test data
    let data_size = 1024 * 1024; // 1MB
    let test_data: Vec<u8> = (0..data_size).map(|i| (i % 256) as u8).collect();
    let mut target = vec![0u8; data_size];

    // SIMD copy performance
    let start = Instant::now();
    let _guard = profiler.start_timer("simd_copy");

    SimdProcessor::fast_copy(&test_data, &mut target)?;

    drop(_guard);
    let simd_duration = start.elapsed();

    // Standard copy for comparison
    let start = Instant::now();
    let _guard = profiler.start_timer("standard_copy");

    target.copy_from_slice(&test_data);

    drop(_guard);
    let standard_duration = start.elapsed();

    println!("Data size: {} KB", data_size / 1024);
    println!("SIMD copy: {:?}", simd_duration);
    println!("Standard copy: {:?}", standard_duration);

    if simd_duration.as_nanos() > 0 {
        println!(
            "Speedup: {:.2}x",
            standard_duration.as_secs_f64() / simd_duration.as_secs_f64()
        );
    }

    // SIMD checksum performance
    let start = Instant::now();
    let _guard = profiler.start_timer("simd_checksum");
    let checksum = SimdProcessor::fast_checksum(&test_data);
    drop(_guard);
    let simd_checksum_duration = start.elapsed();

    // Standard checksum
    let start = Instant::now();
    let _guard = profiler.start_timer("standard_checksum");
    let std_checksum: u64 = test_data.iter().map(|&b| b as u64).sum();
    drop(_guard);
    let std_checksum_duration = start.elapsed();

    println!(
        "SIMD checksum: {} in {:?}",
        checksum, simd_checksum_duration
    );
    println!(
        "Standard checksum: {} in {:?}",
        std_checksum, std_checksum_duration
    );

    Ok(())
}

#[cfg(feature = "streaming")]
async fn benchmark_streaming(
    test_data: &[BenchmarkData],
    profiler: &PerformanceProfiler,
) -> Result<()> {
    let _guard = profiler.start_timer("streaming_processing");

    let mut response = StreamingResponse::<Large>::new();
    let start = Instant::now();

    // Process data in streaming fashion
    for data in test_data {
        let serialized = serde_json::to_vec(data)?;

        #[cfg(feature = "simd")]
        response.extend_from_slice_simd(&serialized);
        #[cfg(not(feature = "simd"))]
        response.extend_from_slice(&serialized);

        // Process chunks as they become available
        while let Some(_chunk) = response.next_chunk() {
            // In a real application, you'd process these chunks
        }
    }

    let streaming_duration = start.elapsed();
    println!("Streaming processing: {:?}", streaming_duration);
    println!("Total bytes processed: {}", response.bytes_read());

    // Calculate throughput
    let throughput =
        response.bytes_read() as f64 / streaming_duration.as_secs_f64() / (1024.0 * 1024.0);
    println!("Throughput: {:.2} MB/s", throughput);

    Ok(())
}

fn generate_test_data(count: usize) -> Vec<BenchmarkData> {
    (0..count)
        .map(|i| BenchmarkData {
            id: i as u64,
            name: format!("benchmark_item_{}", i),
            values: vec![
                i as f64 * 1.5,
                (i as f64).sqrt(),
                (i as f64).powi(2) / 1000.0,
            ],
            metadata: [
                ("category".to_string(), format!("category_{}", i % 10)),
                (
                    "timestamp".to_string(),
                    format!("2024-01-01T{}:00:00Z", i % 24),
                ),
                ("source".to_string(), "performance_benchmark".to_string()),
            ]
            .into_iter()
            .collect(),
        })
        .collect()
}
