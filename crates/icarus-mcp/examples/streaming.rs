//! Streaming Response Example
//!
//! Demonstrates how to use streaming responses for large data processing
//! with configurable buffer sizes and SIMD optimizations.
//!
//! Run with: cargo run --example streaming --features=streaming,simd

use anyhow::Result;
use icarus_mcp::storage::streaming::{CustomSize, Large, ResponseStream, Small, StreamingResponse};
use icarus_mcp::storage::zerocopy::{ZeroCopyDeserializer, ZeroCopySerializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct LargeDataSet {
    id: String,
    data: Vec<String>,
    metadata: std::collections::HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("streaming=debug")
        .init();

    println!("🌊 Streaming Response Examples");

    // Example 1: Small buffer streaming
    println!("\n📦 Example 1: Small Buffer Streaming (4KB)");
    let mut small_response = StreamingResponse::<Small>::new();

    // Add some test data
    for i in 0..100 {
        let data = format!("chunk_{}: {}\n", i, "x".repeat(40));
        small_response.extend_from_slice(data.as_bytes());
    }

    println!(
        "Small buffer size: {} bytes",
        StreamingResponse::<Small>::buffer_size()
    );
    println!("Data processed: {} bytes", small_response.bytes_read());

    // Example 2: Large buffer streaming
    println!("\n📦 Example 2: Large Buffer Streaming (256KB)");
    let mut large_response = StreamingResponse::<Large>::new();

    // Process larger chunks
    for i in 0..10 {
        let data = format!("large_chunk_{}: {}\n", i, "y".repeat(1000));
        #[cfg(feature = "simd")]
        large_response.extend_from_slice_simd(data.as_bytes());
        #[cfg(not(feature = "simd"))]
        large_response.extend_from_slice(data.as_bytes());
    }

    println!(
        "Large buffer size: {} bytes",
        StreamingResponse::<Large>::buffer_size()
    );
    println!("Data processed: {} bytes", large_response.bytes_read());

    // Example 3: Custom buffer size
    println!("\n📦 Example 3: Custom Buffer Size (128KB)");
    const CUSTOM_SIZE: usize = 128 * 1024;
    let mut custom_response = StreamingResponse::<CustomSize<CUSTOM_SIZE>>::new();

    // Test JSON parsing with streaming
    let test_data = LargeDataSet {
        id: "test-123".to_string(),
        data: (0..100).map(|i| format!("item_{}", i)).collect(),
        metadata: [
            ("source".to_string(), "streaming_example".to_string()),
            ("timestamp".to_string(), "2024-01-01T00:00:00Z".to_string()),
        ]
        .into_iter()
        .collect(),
    };

    // Serialize with zero-copy serializer
    let mut serializer = ZeroCopySerializer::new().compact();
    let serialized = serializer.serialize(&test_data)?;
    custom_response.extend_from_slice(&serialized);

    println!(
        "Custom buffer size: {} bytes",
        StreamingResponse::<CustomSize<CUSTOM_SIZE>>::buffer_size()
    );
    println!("JSON data size: {} bytes", serialized.len());

    // Try parsing the JSON
    #[cfg(feature = "simd")]
    let parsed_json = custom_response.try_parse_json_simd()?;
    #[cfg(not(feature = "simd"))]
    let parsed_json = custom_response.try_parse_json()?;

    if let Some(_json) = parsed_json {
        let deserialized: LargeDataSet = ZeroCopyDeserializer::deserialize(&serialized)?;
        println!("✅ Successfully parsed JSON with ID: {}", deserialized.id);
        println!("   Data items: {}", deserialized.data.len());
        println!("   Metadata keys: {}", deserialized.metadata.len());
    }

    // Example 4: Stream processing
    println!("\n🔄 Example 4: Stream Processing");
    let stream_response = StreamingResponse::<Large>::new();
    let _response_stream = ResponseStream::new(stream_response);

    // In a real application, you'd populate the stream with actual data
    // For this example, we'll just demonstrate the collection
    println!("Stream created - ready for data processing");

    // Example 5: Performance features
    #[cfg(feature = "simd")]
    {
        println!("\n⚡ Example 5: SIMD Features");
        let test_data = b"Hello, world! This is a test string for SIMD operations.";
        let mut response = StreamingResponse::<Large>::new();
        response.extend_from_slice(test_data);

        let checksum = response.checksum();
        println!("SIMD checksum: {}", checksum);

        if let Some(pos) = response.find_pattern(b"world") {
            println!("Found 'world' at position: {}", pos);
        }

        let comparison_data = b"Hello, world! This is a test string for SIMD operations.";
        let is_equal = response.fast_equals(comparison_data);
        println!("Data comparison result: {}", is_equal);
    }

    #[cfg(not(feature = "simd"))]
    {
        println!("\n📝 SIMD features disabled - compile with --features=simd to enable");
    }

    println!("\n✨ Streaming examples completed!");
    Ok(())
}
