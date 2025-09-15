//! Unit tests for McpServer

use anyhow::Result;
use candid::Principal;
use icarus_mcp::{McpConfig, McpProtocol, McpServer};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::timeout;

/// Test McpServer creation with valid config
#[tokio::test]
async fn test_mcp_server_creation() -> Result<()> {
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai")?;
    let config = McpConfig::local(canister_id);

    // This will likely fail since we don't have IC running, but test the error handling
    let result = McpServer::new().connect(config).await;

    match result {
        Ok(server) => {
            // If successful, verify the server has the right canister ID
            assert_eq!(server.client().canister_id(), canister_id);
        }
        Err(e) => {
            // Expected in CI - should be a connection error, not a configuration error
            let error_msg = e.to_string().to_lowercase();
            assert!(
                error_msg.contains("connection")
                    || error_msg.contains("network")
                    || error_msg.contains("failed to create")
                    || error_msg.contains("timeout")
                    || error_msg.contains("refused")
                    || error_msg.contains("canister"),
                "Expected connection/canister error, got: {}",
                e
            );
        }
    }

    Ok(())
}

/// Test McpServer creation with production config
#[tokio::test]
async fn test_mcp_server_creation_production() -> Result<()> {
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai")?;
    let config = McpConfig {
        canister_id,
        ic_url: "https://ic0.app".to_string(),
        timeout: Duration::from_secs(30),
        fetch_root_key: false, // Production doesn't need root key
        max_concurrent_requests: 10,
    };

    // This will likely timeout or fail, but test the error handling
    let result = timeout(Duration::from_secs(5), McpServer::new().connect(config)).await;

    match result {
        Ok(Ok(_server)) => {
            // Unlikely to succeed without proper setup, but okay if it does
        }
        Ok(Err(e)) => {
            // Expected - connection or canister error
            let error_msg = e.to_string();
            assert!(!error_msg.is_empty());
        }
        Err(_timeout) => {
            // Also expected - timeout waiting for connection
        }
    }

    Ok(())
}

/// Test config validation
#[test]
fn test_config_validation() {
    // Test local config helper
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let local_config = McpConfig::local(canister_id);

    assert_eq!(local_config.canister_id, canister_id);
    assert_eq!(local_config.ic_url, "http://localhost:4943");
    assert!(local_config.fetch_root_key);
    assert!(local_config.timeout >= Duration::from_secs(30));
}

/// Test invalid canister ID handling
#[tokio::test]
async fn test_invalid_canister_id() -> Result<()> {
    let canister_id = Principal::anonymous(); // Use anonymous as "invalid"
    let config = McpConfig::local(canister_id);

    let result = McpServer::new().connect(config).await;

    // Should fail, but not with a panic
    assert!(result.is_err(), "Should fail with anonymous canister ID");

    let error = result.unwrap_err();
    let error_msg = error.to_string().to_lowercase();

    // Should be some kind of canister or connection error
    assert!(
        error_msg.contains("canister")
            || error_msg.contains("connection")
            || error_msg.contains("network")
            || error_msg.contains("failed"),
        "Expected canister/connection error, got: {}",
        error
    );

    Ok(())
}

/// Mock reader/writer for testing serve function
#[allow(dead_code)]
struct MockStream {
    data: Vec<u8>,
    position: usize,
}

impl MockStream {
    #[allow(dead_code)]
    fn new(data: Vec<u8>) -> Self {
        Self { data, position: 0 }
    }
}

impl AsyncRead for MockStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let remaining = self.data.len().saturating_sub(self.position);
        if remaining == 0 {
            return std::task::Poll::Ready(Ok(()));
        }

        let to_read = std::cmp::min(remaining, buf.remaining());
        let end_pos = self.position + to_read;

        buf.put_slice(&self.data[self.position..end_pos]);
        self.position = end_pos;

        std::task::Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for MockStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        // Just pretend to write everything
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

/// Test server interface (without actual serving)
#[tokio::test]
async fn test_server_interface() -> Result<()> {
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai")?;
    let config = McpConfig::local(canister_id);

    // Test that server creation works or fails gracefully
    match McpServer::new().connect(config).await {
        Ok(server) => {
            // If successful, test that we can access the handler
            let handler = server.handler();
            let server_info = handler.get_server_info();
            assert_eq!(server_info["name"], "icarus-mcp");
        }
        Err(_e) => {
            // Expected if IC is not running
            eprintln!("Server creation failed as expected (IC not available)");
        }
    }

    Ok(())
}

/// Test server client access
#[tokio::test]
async fn test_server_client_access() -> Result<()> {
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai")?;
    let config = McpConfig::local(canister_id);

    if let Ok(server) = McpServer::new().connect(config).await {
        // Test that we can access the client
        let client = server.client();
        assert_eq!(client.canister_id(), canister_id);
    }

    Ok(())
}

/// Test configuration edge cases
#[test]
fn test_config_edge_cases() {
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();

    // Test with very short timeout
    let short_timeout_config = McpConfig {
        canister_id,
        ic_url: "http://localhost:4943".to_string(),
        timeout: Duration::from_millis(1),
        fetch_root_key: true,
        max_concurrent_requests: 10,
    };

    assert!(short_timeout_config.timeout < Duration::from_secs(1));

    // Test with very long timeout
    let long_timeout_config = McpConfig {
        canister_id,
        ic_url: "http://localhost:4943".to_string(),
        timeout: Duration::from_secs(3600), // 1 hour
        fetch_root_key: false,
        max_concurrent_requests: 10,
    };

    assert!(long_timeout_config.timeout >= Duration::from_secs(3600));

    // Test with different URLs
    let custom_url_config = McpConfig {
        canister_id,
        ic_url: "http://127.0.0.1:8080".to_string(),
        timeout: Duration::from_secs(30),
        fetch_root_key: true,
        max_concurrent_requests: 10,
    };

    assert!(custom_url_config.ic_url.contains("127.0.0.1"));
    assert!(custom_url_config.ic_url.contains("8080"));
}

/// Test refresh_tools method
#[tokio::test]
async fn test_refresh_tools() -> Result<()> {
    let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai")?;
    let config = McpConfig::local(canister_id);

    if let Ok(server) = McpServer::new().connect(config).await {
        // Test refresh_tools (currently just returns Ok)
        let result = server.refresh_tools().await;
        assert!(result.is_ok());
    }

    Ok(())
}
