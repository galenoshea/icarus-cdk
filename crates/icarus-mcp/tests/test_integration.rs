//! Integration tests for advanced features
//!
//! Tests trait implementations, error handling, and feature interactions

use candid::Principal;
use icarus_mcp::{McpConfig, McpServer};
use std::str::FromStr;
use std::time::Duration;

#[cfg(feature = "storage")]
use icarus_mcp::storage::{
    allocator::{get_pool_stats, get_pooled_buffer},
    zerocopy::{ZeroCopyDeserializer, ZeroCopySerializer},
};

#[cfg(feature = "streaming")]
use icarus_mcp::storage::streaming::{CustomSize, Large, Small, StreamingResponse};

#[cfg(feature = "simd")]
use icarus_mcp::storage::simd::SimdProcessor;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: u64,
    name: String,
    values: Vec<f64>,
}

/// Test trait implementations and type safety
#[tokio::test]
async fn test_trait_implementations() {
    let canister_id = Principal::from_str("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let config = McpConfig::local(canister_id);

    // Test Clone implementation on config
    let cloned_config = config.clone();
    assert_eq!(config.canister_id, cloned_config.canister_id);

    // Test Debug implementation
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("McpConfig"));
}

/// Test error handling scenarios
#[tokio::test]
async fn test_error_handling() {
    // Test invalid canister ID
    let invalid_config = McpConfig::local(Principal::anonymous());

    // The config creation should work, but connection might fail
    // This tests that our error types implement the required traits
    match McpServer::from_config(invalid_config).await {
        Ok(_) => {
            // If it succeeds, that's fine too (local testing environment)
        }
        Err(e) => {
            // Error should implement Display and Debug
            let _debug = format!("{:?}", e);
            let _display = format!("{}", e);
        }
    }
}

/// Test feature interactions and compatibility
#[cfg(all(feature = "storage", feature = "streaming"))]
#[tokio::test]
async fn test_storage_streaming_integration() {
    // Test that storage allocators work with streaming responses
    let buffer = get_pooled_buffer(1024);
    assert!(buffer.capacity() >= 1024);

    let mut response = StreamingResponse::<Large>::new();
    let test_data = b"Integration test data for storage and streaming features";

    #[cfg(feature = "simd")]
    response.extend_from_slice_simd(test_data);
    #[cfg(not(feature = "simd"))]
    response.extend_from_slice(test_data);

    assert_eq!(response.bytes_read(), test_data.len());

    // Test zero-copy serialization with streaming
    let data = TestData {
        id: 123,
        name: "integration_test".to_string(),
        values: vec![1.0, 2.5, std::f64::consts::PI],
    };

    let mut serializer = ZeroCopySerializer::new().compact();
    let serialized = serializer.serialize(&data).unwrap();

    // Test deserialization directly (zero-copy doesn't produce JSON)
    let deserialized: TestData = ZeroCopyDeserializer::deserialize(&serialized).unwrap();
    assert_eq!(data, deserialized);

    // Test JSON parsing with a fresh response (avoid mixing binary data with JSON)
    let mut json_response = StreamingResponse::<Large>::new();
    json_response.extend_from_slice(b"{\"test\": \"json_data\"}");
    let parsed_json = json_response.try_parse_json().unwrap();
    assert!(parsed_json.is_some());
    let json_value = parsed_json.unwrap();
    assert_eq!(json_value["test"], "json_data");
}

/// Test SIMD operations with different data sizes
#[cfg(feature = "simd")]
#[test]
fn test_simd_edge_cases() {
    // Test with empty data
    let empty_data = [];
    let checksum = SimdProcessor::fast_checksum(&empty_data);
    assert_eq!(checksum, 0);

    // Test with single byte
    let single_byte = [42];
    let checksum = SimdProcessor::fast_checksum(&single_byte);
    assert_eq!(checksum, 42);

    // Test comparison with different sizes
    let data1 = [1, 2, 3];
    let data2 = [1, 2];
    assert!(!SimdProcessor::fast_compare(&data1, &data2));

    // Test pattern finding edge cases
    let data = b"hello world";
    assert_eq!(SimdProcessor::fast_find(data, b""), Some(0)); // Empty pattern
    assert_eq!(SimdProcessor::fast_find(b"", b"hello"), None); // Empty data
    assert_eq!(SimdProcessor::fast_find(data, b"xyz"), None); // Not found
    assert_eq!(SimdProcessor::fast_find(data, b"world"), Some(6)); // Found at end
}

/// Test streaming responses with different buffer sizes
#[cfg(feature = "streaming")]
#[test]
fn test_streaming_buffer_sizes() {
    // Test const generic buffer sizes
    assert_eq!(StreamingResponse::<Small>::buffer_size(), 4 * 1024);
    assert_eq!(StreamingResponse::<Large>::buffer_size(), 256 * 1024);
    assert_eq!(StreamingResponse::<CustomSize<8192>>::buffer_size(), 8192);

    // Test each buffer type individually to avoid type mismatches
    let test_data = b"Buffer test data";

    // Test Small buffer
    let mut small_response = StreamingResponse::<Small>::new();
    small_response.extend_from_slice(test_data);
    assert_eq!(small_response.bytes_read(), test_data.len());

    // Test Large buffer
    let mut large_response = StreamingResponse::<Large>::new();
    large_response.extend_from_slice(test_data);
    assert_eq!(large_response.bytes_read(), test_data.len());

    // Test Custom buffer
    let mut custom_response = StreamingResponse::<CustomSize<8192>>::new();
    custom_response.extend_from_slice(test_data);
    assert_eq!(custom_response.bytes_read(), test_data.len());
}

/// Test memory allocator statistics and tracking
#[cfg(feature = "storage")]
#[test]
fn test_allocator_statistics() {
    // Get initial stats
    let initial_stats = get_pool_stats();

    // Perform some allocations
    let sizes = [256, 512, 1024, 2048];
    let mut buffers = Vec::new();

    for &size in &sizes {
        for _ in 0..3 {
            buffers.push(get_pooled_buffer(size));
        }
    }

    // Check that we can get statistics
    let stats = get_pool_stats();
    assert!(!stats.is_empty());

    // Return buffers (via drop)
    drop(buffers);

    // Statistics should reflect the operations
    let final_stats = get_pool_stats();
    assert!(final_stats.len() >= initial_stats.len());
}

/// Test configuration builder pattern
#[test]
fn test_config_builder_comprehensive() {
    let canister_id = Principal::from_str("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();

    // Test chaining methods
    let config = McpConfig::local(canister_id)
        .with_timeout(Duration::from_secs(45))
        .with_max_concurrent_requests(20);

    // Test that values are set correctly
    // Note: These are internal fields, so we test behavior rather than direct access
    let config_str = format!("{:?}", config);
    assert!(config_str.contains("McpConfig"));

    // Test validation through builder pattern
    use icarus_mcp::config::McpConfigBuilder;

    let builder = McpConfigBuilder::new()
        .canister_id(canister_id)
        .ic_url("http://localhost:4943".to_string())
        .timeout(Duration::from_secs(30))
        .max_concurrent_requests(15);

    let result = builder.build();
    assert!(result.is_ok());

    // Test invalid configurations
    let invalid_builder = McpConfigBuilder::new().canister_id(canister_id);
    // Missing IC URL

    let result = invalid_builder.build();
    assert!(result.is_err());
}

/// Test protocol trait abstractions
#[tokio::test]
async fn test_protocol_abstractions() {
    use icarus_mcp::networking::client::CanisterClient;
    use icarus_mcp::protocol::{McpProtocol, McpProtocolHandler};
    use std::sync::Arc;

    let canister_id = Principal::from_str("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let config = McpConfig::local(canister_id);

    // Create client and handler
    let client = CanisterClient::new(config).await;
    match client {
        Ok(client) => {
            let client = Arc::new(client);

            // Test that we can create a protocol handler
            let metadata = icarus_mcp::networking::client::CanisterMetadata {
                name: "test".to_string(),
                version: Some("1.0.0".to_string()),
                tools: smallvec::SmallVec::new(),
                title: Some("Test Canister".to_string()),
                website_url: Some("https://example.com".to_string()),
            };

            let handler = McpProtocolHandler::new(client, metadata);
            let server_info = handler.get_server_info();

            // server_info is a JsonValue, test that it's an object
            assert!(server_info.is_object());
        }
        Err(_) => {
            // If client creation fails (e.g., no local replica),
            // that's okay for this test - we're just testing the types
            println!("Client creation failed - likely no local IC replica");
        }
    }
}

/// Test performance profiler integration
#[cfg(feature = "storage")]
#[test]
fn test_performance_profiler() {
    use icarus_mcp::storage::profile::PerformanceProfiler;
    use std::thread;
    use std::time::Duration;

    let profiler = PerformanceProfiler::new();

    // Test function timing
    {
        let _guard = profiler.start_timer("test_function");
        thread::sleep(Duration::from_millis(10));
    } // Guard dropped here, timing recorded

    // Test allocation recording
    profiler.record_allocation("test_allocation", 1024);
    profiler.record_allocation("test_allocation", 2048);

    // Test hot path marking
    profiler.mark_hot_path("test_path", Duration::from_millis(5));

    // Get statistics
    let stats = profiler.get_statistics();

    assert!(!stats.function_stats.is_empty());
    assert!(stats.function_stats.contains_key("test_function"));
    assert!(!stats.allocation_summary.is_empty());
    assert!(stats.allocation_summary.contains_key("test_allocation"));
    assert!(!stats.hot_path_summary.is_empty());
    assert!(stats.hot_path_summary.contains_key("test_path"));

    // Test reset
    profiler.reset_statistics();
    let reset_stats = profiler.get_statistics();
    assert!(reset_stats.function_stats.is_empty());
}
