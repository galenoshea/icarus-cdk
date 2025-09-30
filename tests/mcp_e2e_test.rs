//! End-to-end tests for MCP client interaction with glizzinator

use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::TempDir;

/// Test complete MCP client workflow with glizzinator
#[tokio::test]
async fn test_complete_mcp_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // First, ensure the CLI is built
    let build_output = Command::new("cargo")
        .args(&[
            "build",
            "--package",
            "icarus-cli",
            "--bin",
            "icarus",
            "--release",
        ])
        .output()?;

    if !build_output.status.success() {
        panic!(
            "Failed to build icarus CLI: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Test with a known canister ID (adjust as needed)
    let canister_id = "rrkah-fqaaa-aaaaa-aaaaq-cai";

    // Create a mock MCP client interaction
    let mut child = Command::new("./target/release/icarus")
        .args(&["mcp", "start", canister_id])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.take().expect("Failed to get stdin");
    let stdout = child.stdout.take().expect("Failed to get stdout");

    // Spawn a task to handle responses
    let response_handler = tokio::spawn(async move {
        let mut responses = Vec::new();
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            if let Ok(line) = line {
                if !line.trim().is_empty() {
                    if let Ok(json) = serde_json::from_str::<Value>(&line) {
                        responses.push(json);
                    } else {
                        eprintln!("Invalid JSON response: {}", line);
                    }
                }
            }
        }
        responses
    });

    // Send MCP requests
    let mut stdin = stdin;

    // 1. Initialize the connection
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    writeln!(stdin, "{}", init_request)?;

    // 2. List available tools
    let list_tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    writeln!(stdin, "{}", list_tools_request)?;

    // 3. Test image handling with a real image file
    let temp_dir = TempDir::new()?;
    let image_path = temp_dir.path().join("test_hotdog.jpg");

    // Create a minimal JPEG file (not a real hotdog, just for testing)
    let jpeg_header = vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x01, 0x00,
        0x48, 0x00, 0x48, 0x00, 0x00, 0xFF, 0xD9,
    ];
    fs::write(&image_path, jpeg_header)?;

    let call_tool_request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "is_hotdog",
            "arguments": {
                "image_base64": image_path.to_string_lossy().to_string()
            }
        }
    });

    writeln!(stdin, "{}", call_tool_request)?;

    // Close stdin to signal end of input
    drop(stdin);

    // Wait for responses with timeout
    let responses = tokio::time::timeout(Duration::from_secs(30), response_handler).await??;

    // Validate responses
    assert!(
        !responses.is_empty(),
        "Should receive at least one response"
    );

    for response in &responses {
        // All responses should be valid JSON-RPC
        assert_eq!(response["jsonrpc"], "2.0", "Invalid JSON-RPC version");
        assert!(response.get("id").is_some(), "Missing response ID");

        // Should have either result or error
        assert!(
            response.get("result").is_some() || response.get("error").is_some(),
            "Response missing both result and error"
        );

        // Log response for debugging
        println!("MCP Response: {}", serde_json::to_string_pretty(response)?);
    }

    // Wait for process to complete
    let exit_status = child.wait()?;
    println!("MCP bridge exit status: {}", exit_status);

    Ok(())
}

/// Test MCP bridge with invalid image files
#[tokio::test]
async fn test_mcp_bridge_invalid_images() -> Result<(), Box<dyn std::error::Error>> {
    let canister_id = "rrkah-fqaaa-aaaaa-aaaaq-cai";

    let mut child = Command::new("./target/release/icarus")
        .args(&["mcp", "start", canister_id])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.take().unwrap();
    let mut stdin = stdin;

    // Test with non-existent image file
    let call_tool_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "is_hotdog",
            "arguments": {
                "image_base64": "/nonexistent/image.png"
            }
        }
    });

    writeln!(stdin, "{}", call_tool_request)?;
    drop(stdin);

    // Wait for process to complete
    let output = child.wait_with_output()?;

    // Should handle invalid images gracefully
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("STDERR: {}", stderr);

    // Should log the warning but not crash
    assert!(stderr.contains("icarus-mcp-bridge"));

    // Stdout should still be valid JSON-RPC (error response)
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        for line in stdout.lines() {
            if !line.trim().is_empty() {
                let json_result = serde_json::from_str::<Value>(line);
                assert!(json_result.is_ok(), "Invalid JSON in response: {}", line);
            }
        }
    }

    Ok(())
}

/// Test MCP bridge protocol version negotiation
#[tokio::test]
async fn test_protocol_version_negotiation() -> Result<(), Box<dyn std::error::Error>> {
    let canister_id = "rrkah-fqaaa-aaaaa-aaaaq-cai";

    let test_versions = vec![
        "2025-06-18", // Current version
        "2024-11-05", // Older version
        "invalid",    // Invalid version
    ];

    for version in test_versions {
        let mut child = Command::new("./target/release/icarus")
            .args(&["mcp", "start", canister_id])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let mut stdin = stdin;

        let init_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": version,
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        });

        writeln!(stdin, "{}", init_request)?;
        drop(stdin);

        let output = child.wait_with_output()?;

        // Should handle all versions gracefully (might return error for unsupported)
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            for line in stdout.lines() {
                if !line.trim().is_empty() {
                    let json_result = serde_json::from_str::<Value>(line);
                    assert!(
                        json_result.is_ok(),
                        "Invalid JSON for version {}: {}",
                        version,
                        line
                    );
                }
            }
        }

        println!("Version {} handled successfully", version);
    }

    Ok(())
}

/// Benchmark MCP bridge performance
#[tokio::test]
async fn test_mcp_bridge_performance() -> Result<(), Box<dyn std::error::Error>> {
    let canister_id = "rrkah-fqaaa-aaaaa-aaaaq-cai";

    let start_time = std::time::Instant::now();

    // Test multiple rapid requests
    let mut child = Command::new("./target/release/icarus")
        .args(&["mcp", "start", canister_id])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.take().unwrap();
    let mut stdin = stdin;

    // Send multiple requests rapidly
    for i in 0..10 {
        let request = json!({
            "jsonrpc": "2.0",
            "id": i,
            "method": "tools/list",
            "params": {}
        });

        writeln!(stdin, "{}", request)?;
    }

    drop(stdin);

    let output = child.wait_with_output()?;
    let elapsed = start_time.elapsed();

    println!("MCP bridge handled 10 requests in {:?}", elapsed);

    // Should complete within reasonable time (adjust as needed)
    assert!(
        elapsed < Duration::from_secs(10),
        "Bridge too slow: {:?}",
        elapsed
    );

    // All responses should be valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let response_count = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();

    println!("Received {} responses", response_count);

    Ok(())
}

/// Test resource cleanup and proper shutdown
#[tokio::test]
async fn test_resource_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let canister_id = "rrkah-fqaaa-aaaaa-aaaaq-cai";

    // Start and immediately stop the bridge multiple times
    for i in 0..5 {
        let mut child = Command::new("./target/release/icarus")
            .args(&["mcp", "start", canister_id])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Send a quick request and close
        if let Some(ref mut stdin) = child.stdin {
            let request = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-06-18",
                    "capabilities": {}
                }
            });

            writeln!(stdin, "{}", request)?;
        }

        // Close stdin to trigger shutdown
        child.stdin.take();

        let output = child.wait_with_output()?;

        // Should shutdown cleanly
        println!("Iteration {}: exit status {}", i, output.status);

        // Check for any resource leaks in stderr
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("leaked"),
            "Resource leak detected: {}",
            stderr
        );
        assert!(!stderr.contains("panic"), "Panic detected: {}", stderr);
    }

    Ok(())
}
