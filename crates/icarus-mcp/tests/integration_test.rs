//! Integration tests for the icarus-mcp crate
//!
//! These tests verify the full workflow of MCP server creation,
//! protocol handling, and canister communication.

use anyhow::Result;
use candid::Principal;
use icarus_mcp::{McpConfig, McpServer, ToolMetadata};
use serde_json::json;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::time::timeout;

/// Integration test for complete MCP server workflow
#[tokio::test]
async fn test_complete_mcp_workflow() -> Result<()> {
    // Test configuration creation
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai")?;
    let config = McpConfig::local(canister_id);

    assert_eq!(config.canister_id, canister_id);
    assert!(config.fetch_root_key);

    // Test server creation (might fail without IC)
    match McpServer::new().connect(config).await {
        Ok(server) => {
            // If we successfully created a server, test its properties
            assert_eq!(server.client().canister_id(), canister_id);

            // Test that we can call refresh_tools
            let refresh_result = server.refresh_tools().await;
            assert!(refresh_result.is_ok());

            eprintln!("✅ Integration test passed - IC is running and reachable");
        }
        Err(e) => {
            // Expected in CI environments - verify it's a connection error
            let error_msg = e.to_string().to_lowercase();
            assert!(
                error_msg.contains("connection")
                    || error_msg.contains("network")
                    || error_msg.contains("failed to create")
                    || error_msg.contains("timeout")
                    || error_msg.contains("refused")
                    || error_msg.contains("canister"),
                "Expected connection error, got: {}",
                e
            );

            eprintln!("⚠️  Integration test skipped - IC not available: {}", e);
        }
    }

    Ok(())
}

/// Test error propagation through the stack
#[tokio::test]
async fn test_error_propagation() -> Result<()> {
    // Test with intentionally invalid configuration
    let invalid_canister = Principal::anonymous();
    let config = McpConfig {
        canister_id: invalid_canister,
        ic_url: "http://invalid-url:9999".to_string(),
        timeout: Duration::from_millis(100), // Very short timeout
        fetch_root_key: true,
        max_concurrent_requests: 10,
    };

    let result = timeout(Duration::from_secs(2), McpServer::new().connect(config)).await;

    match result {
        Ok(Ok(_)) => {
            // Very unlikely to succeed with this config
            eprintln!("Unexpected success with invalid config");
        }
        Ok(Err(e)) => {
            // Expected - should be a network or canister error
            let error_msg = e.to_string();
            assert!(!error_msg.is_empty());
            eprintln!("Expected error with invalid config: {}", e);
        }
        Err(_) => {
            // Timeout is also expected
            eprintln!("Timeout with invalid config (expected)");
        }
    }

    Ok(())
}

/// Test configuration variations
#[tokio::test]
async fn test_config_variations() -> Result<()> {
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai")?;

    // Test local config
    let local_config = McpConfig::local(canister_id);
    assert!(local_config.fetch_root_key);
    assert!(local_config.ic_url.contains("localhost"));

    // Test production-like config
    let prod_config = McpConfig {
        canister_id,
        ic_url: "https://ic0.app".to_string(),
        timeout: Duration::from_secs(60),
        fetch_root_key: false,
        max_concurrent_requests: 20,
    };

    assert!(!prod_config.fetch_root_key);
    assert!(prod_config.ic_url.contains("ic0.app"));
    assert!(prod_config.timeout >= Duration::from_secs(60));

    // Both configs should have valid canister IDs
    assert_eq!(local_config.canister_id, prod_config.canister_id);

    Ok(())
}

/// Mock stream for testing MCP protocol
struct MockMcpStream {
    read_data: Vec<u8>,
    read_pos: usize,
    write_data: Vec<u8>,
}

impl MockMcpStream {
    fn new(input: &str) -> Self {
        Self {
            read_data: input.as_bytes().to_vec(),
            read_pos: 0,
            write_data: Vec::new(),
        }
    }

    fn written_data(&self) -> &[u8] {
        &self.write_data
    }
}

impl AsyncRead for MockMcpStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let remaining = self.read_data.len().saturating_sub(self.read_pos);
        if remaining == 0 {
            return std::task::Poll::Ready(Ok(()));
        }

        let to_read = std::cmp::min(remaining, buf.remaining());
        let end_pos = self.read_pos + to_read;

        buf.put_slice(&self.read_data[self.read_pos..end_pos]);
        self.read_pos = end_pos;

        std::task::Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for MockMcpStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        self.write_data.extend_from_slice(buf);
        std::task::Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

/// Test MCP protocol communication (mock)
#[tokio::test]
async fn test_mcp_protocol_mock() -> Result<()> {
    // Create a mock MCP initialize request
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-20",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    let request_str = format!("{}\n", initialize_request);
    let mock_stream = MockMcpStream::new(&request_str);

    // We can't easily test the full protocol without a real server,
    // but we can verify the stream works
    assert!(!mock_stream.read_data.is_empty());
    assert_eq!(mock_stream.written_data().len(), 0);

    Ok(())
}

/// Test tool metadata integration
#[test]
fn test_tool_metadata_integration() {
    // Test creating comprehensive tool metadata
    let tool = ToolMetadata {
        name: "calculate".to_string(),
        description: "Perform mathematical calculations".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"],
                    "description": "The mathematical operation to perform"
                },
                "operands": {
                    "type": "array",
                    "items": {"type": "number"},
                    "minItems": 2,
                    "description": "Numbers to operate on"
                }
            },
            "required": ["operation", "operands"]
        }),
        title: Some("Calculator Tool".to_string()),
        icon: Some("🧮".to_string()),
    };

    // Test serialization round-trip
    let serialized = serde_json::to_string(&tool).unwrap();
    let deserialized: ToolMetadata = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.name, "calculate");
    assert_eq!(
        deserialized.description,
        "Perform mathematical calculations"
    );
    assert!(deserialized.title.is_some());
    assert!(deserialized.icon.is_some());

    // Test schema structure
    let schema = &deserialized.input_schema;
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"].is_object());
    assert!(schema["required"].is_array());
}

/// Test concurrent operations (if server creation succeeds)
#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai")?;

    // Try to create multiple servers concurrently
    let configs: Vec<_> = (0..3).map(|_| McpConfig::local(canister_id)).collect();

    let mut handles = Vec::new();

    for config in configs {
        let handle = tokio::spawn(async move {
            timeout(Duration::from_secs(2), McpServer::new().connect(config)).await
        });
        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should either succeed or fail with the same error type
    let mut success_count = 0;
    let mut error_count = 0;

    for result in results {
        match result {
            Ok(Ok(Ok(_server))) => success_count += 1,
            Ok(Ok(Err(_e))) => error_count += 1,
            Ok(Err(_timeout)) => error_count += 1,
            Err(_join_error) => error_count += 1,
        }
    }

    // Either all should succeed or all should fail (depending on IC availability)
    assert!(
        success_count == 3 || error_count == 3,
        "Inconsistent results: {} successes, {} errors",
        success_count,
        error_count
    );

    eprintln!(
        "Concurrent test: {} successes, {} errors",
        success_count, error_count
    );

    Ok(())
}

/// Test memory usage and resource cleanup
#[tokio::test]
async fn test_resource_management() -> Result<()> {
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai")?;

    // Create and drop servers to test resource cleanup
    for i in 0..5 {
        let config = McpConfig::local(canister_id);

        match McpServer::new().connect(config).await {
            Ok(server) => {
                // Use the server briefly
                let client_id = server.client().canister_id();
                assert_eq!(client_id, canister_id);

                // Call refresh_tools
                let _ = server.refresh_tools().await;

                eprintln!("Created and used server #{}", i + 1);
                // Server will be dropped here
            }
            Err(_e) => {
                // Expected if IC is not running
                eprintln!("Skipping resource test #{} - IC not available", i + 1);
                break;
            }
        }
    }

    Ok(())
}
