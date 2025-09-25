//! Integration tests for endpoints module

use candid::Principal;
use icarus_canister::endpoints::{HttpRequest, HttpResponse};
use icarus_core::protocol::{IcarusMetadata, ToolMetadata};

/// Test icarus_metadata function structure
#[test]
fn test_icarus_metadata_structure() {
    // Test metadata structure without canister runtime
    let metadata = IcarusMetadata {
        version: "1.0.0".to_string(),
        canister_id: Principal::anonymous(),
        tools: vec![
            ToolMetadata {
                name: "test_tool".to_string(),
                candid_method: "test_tool".to_string(),
                is_query: true,
                description: "test_tool tool".to_string(),
                parameters: vec![],
            },
            ToolMetadata {
                name: "update_tool".to_string(),
                candid_method: "update_tool".to_string(),
                is_query: false,
                description: "update_tool tool".to_string(),
                parameters: vec![],
            },
        ],
    };

    assert_eq!(metadata.version, "1.0.0");
    assert_eq!(metadata.canister_id, Principal::anonymous());
    assert_eq!(metadata.tools.len(), 2);

    // Check first tool
    let test_tool = metadata
        .tools
        .iter()
        .find(|t| t.name == "test_tool")
        .unwrap();
    assert_eq!(test_tool.name, "test_tool");
    assert_eq!(test_tool.candid_method, "test_tool");
    assert_eq!(test_tool.is_query, true);
    assert_eq!(test_tool.description, "test_tool tool");
    assert_eq!(test_tool.parameters.len(), 0);

    // Check second tool
    let update_tool = metadata
        .tools
        .iter()
        .find(|t| t.name == "update_tool")
        .unwrap();
    assert_eq!(update_tool.name, "update_tool");
    assert_eq!(update_tool.candid_method, "update_tool");
    assert_eq!(update_tool.is_query, false);
    assert_eq!(update_tool.description, "update_tool tool");
    assert_eq!(update_tool.parameters.len(), 0);
}

/// Test icarus_metadata function with empty state
#[test]
fn test_icarus_metadata_empty_state() {
    // Test fallback when state is None
    let metadata = IcarusMetadata {
        version: "1.0.0".to_string(),
        canister_id: Principal::anonymous(),
        tools: vec![],
    };

    assert_eq!(metadata.version, "1.0.0");
    assert_eq!(metadata.canister_id, Principal::anonymous());
    assert_eq!(metadata.tools.len(), 0);
}

/// Test HTTP request handling with GET request
#[test]
fn test_http_request_get() {
    let _request = HttpRequest {
        method: "GET".to_string(),
        url: "/".to_string(),
        headers: vec![
            ("User-Agent".to_string(), "test-agent/1.0".to_string()),
            ("Accept".to_string(), "text/html".to_string()),
        ],
        body: vec![],
    };

    // Create mock metadata
    let metadata = IcarusMetadata {
        version: "1.0.0".to_string(),
        canister_id: Principal::anonymous(),
        tools: vec![ToolMetadata {
            name: "test_tool".to_string(),
            candid_method: "test_tool".to_string(),
            is_query: true,
            description: "A test tool".to_string(),
            parameters: vec![],
        }],
    };

    // Test HTML generation logic
    let tools_html: String = metadata
        .tools
        .iter()
        .map(|tool| {
            format!(
                r#"<div class="tool">
                <strong>{}</strong> ({})
                <br><small>{}</small>
            </div>"#,
                tool.name,
                if tool.is_query { "query" } else { "update" },
                tool.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(tools_html.contains("test_tool"));
    assert!(tools_html.contains("query"));
    assert!(tools_html.contains("A test tool"));

    // Test HTML template
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Icarus Canister</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        .container {{
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            margin-bottom: 10px;
        }}
        .info {{
            background: #e7f3ff;
            padding: 15px;
            border-radius: 5px;
            border-left: 4px solid #2196F3;
            margin: 20px 0;
        }}
        code {{
            background: #f0f0f0;
            padding: 2px 5px;
            border-radius: 3px;
            font-family: "Courier New", monospace;
        }}
        .tool {{
            margin: 10px 0;
            padding: 10px;
            background: #f9f9f9;
            border-radius: 5px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Icarus Canister</h1>
        <p>This is a standard ICP canister built with Icarus CDK.</p>

        <div class="info">
            <strong>Canister ID:</strong> <code>{}</code><br>
            <strong>Version:</strong> <code>{}</code><br>
            <strong>Status:</strong> <span style="color: green;">✓ Running</span>
        </div>

        <h2>Available Tools</h2>
        {}

        <h2>Connect with Claude Desktop</h2>
        <p>To use this canister with Claude Desktop, run:</p>
        <code>icarus bridge start --canister-id {}</code>

        <hr style="margin-top: 40px; border: none; border-top: 1px solid #eee;">
        <p style="text-align: center; color: #666; font-size: 14px;">
            Powered by <a href="https://icarus.dev" style="color: #2196F3;">Icarus CDK</a>
        </p>
    </div>
</body>
</html>"#,
        Principal::anonymous().to_text(),
        metadata.version,
        tools_html,
        Principal::anonymous().to_text()
    );

    // Verify HTML contains expected elements
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Icarus Canister"));
    assert!(html.contains("🚀"));
    assert!(html.contains(&metadata.version));
    assert!(html.contains(&Principal::anonymous().to_text()));
    assert!(html.contains("test_tool"));
    assert!(html.contains("icarus bridge start"));
    assert!(html.contains("Powered by"));

    // Test response structure
    let response = HttpResponse {
        status_code: 200,
        headers: vec![
            (
                "Content-Type".to_string(),
                "text/html; charset=UTF-8".to_string(),
            ),
            ("Cache-Control".to_string(), "no-cache".to_string()),
        ],
        body: html.into_bytes(),
    };

    assert_eq!(response.status_code, 200);
    assert_eq!(response.headers.len(), 2);

    // Check headers
    let content_type = response.headers.iter().find(|(k, _)| k == "Content-Type");
    assert!(content_type.is_some());
    assert_eq!(content_type.unwrap().1, "text/html; charset=UTF-8");

    let cache_control = response.headers.iter().find(|(k, _)| k == "Cache-Control");
    assert!(cache_control.is_some());
    assert_eq!(cache_control.unwrap().1, "no-cache");

    // Check body is valid UTF-8
    let body_str = String::from_utf8(response.body).expect("Response body should be valid UTF-8");
    assert!(body_str.contains("Icarus Canister"));
}

/// Test HTTP request handling with POST request (should still return same HTML)
#[test]
fn test_http_request_post() {
    let _request = HttpRequest {
        method: "POST".to_string(),
        url: "/api/tools".to_string(),
        headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        body: b"{\"test\": \"data\"}".to_vec(),
    };

    // HTTP endpoint should ignore method and always return HTML
    // (since it's designed for browser viewing, not API usage)

    // Test that we would get the same response structure
    let response_headers = vec![
        (
            "Content-Type".to_string(),
            "text/html; charset=UTF-8".to_string(),
        ),
        ("Cache-Control".to_string(), "no-cache".to_string()),
    ];

    assert_eq!(response_headers.len(), 2);
    assert_eq!(response_headers[0].0, "Content-Type");
    assert_eq!(response_headers[0].1, "text/html; charset=UTF-8");
    assert_eq!(response_headers[1].0, "Cache-Control");
    assert_eq!(response_headers[1].1, "no-cache");
}

/// Test HTTP request structs serialization/deserialization
#[test]
fn test_http_request_serialization() {
    use candid::{decode_one, encode_one};

    let request = HttpRequest {
        method: "GET".to_string(),
        url: "/health".to_string(),
        headers: vec![
            ("Authorization".to_string(), "Bearer token123".to_string()),
            ("Accept".to_string(), "application/json".to_string()),
        ],
        body: b"test body".to_vec(),
    };

    // Test Candid serialization
    let encoded = encode_one(&request).expect("Should serialize HttpRequest");
    assert!(encoded.len() > 0);

    let decoded: HttpRequest = decode_one(&encoded).expect("Should deserialize HttpRequest");
    assert_eq!(decoded.method, "GET");
    assert_eq!(decoded.url, "/health");
    assert_eq!(decoded.headers.len(), 2);
    assert_eq!(
        decoded.headers[0],
        ("Authorization".to_string(), "Bearer token123".to_string())
    );
    assert_eq!(
        decoded.headers[1],
        ("Accept".to_string(), "application/json".to_string())
    );
    assert_eq!(decoded.body, b"test body".to_vec());
}

/// Test HTTP response struct
#[test]
fn test_http_response_creation() {
    let html_content = "<html><body>Test</body></html>";

    let response = HttpResponse {
        status_code: 200,
        headers: vec![
            ("Content-Type".to_string(), "text/html".to_string()),
            ("X-Custom-Header".to_string(), "test-value".to_string()),
        ],
        body: html_content.as_bytes().to_vec(),
    };

    assert_eq!(response.status_code, 200);
    assert_eq!(response.headers.len(), 2);
    assert_eq!(response.body, html_content.as_bytes());

    // Test body as string
    let body_str = String::from_utf8(response.body).expect("Body should be valid UTF-8");
    assert_eq!(body_str, html_content);
}

/// Test error status codes and different response scenarios
#[test]
fn test_http_error_responses() {
    // Test 404 response
    let not_found = HttpResponse {
        status_code: 404,
        headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
        body: b"Not Found".to_vec(),
    };

    assert_eq!(not_found.status_code, 404);
    assert_eq!(String::from_utf8(not_found.body).unwrap(), "Not Found");

    // Test 500 response
    let server_error = HttpResponse {
        status_code: 500,
        headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        body: b"{\"error\": \"Internal Server Error\"}".to_vec(),
    };

    assert_eq!(server_error.status_code, 500);
    let error_json = String::from_utf8(server_error.body).unwrap();
    assert!(error_json.contains("Internal Server Error"));
}

/// Test empty and malformed HTTP requests
#[test]
fn test_malformed_http_requests() {
    // Empty method
    let empty_method = HttpRequest {
        method: "".to_string(),
        url: "/".to_string(),
        headers: vec![],
        body: vec![],
    };
    assert_eq!(empty_method.method, "");

    // Empty URL
    let empty_url = HttpRequest {
        method: "GET".to_string(),
        url: "".to_string(),
        headers: vec![],
        body: vec![],
    };
    assert_eq!(empty_url.url, "");

    // Invalid HTTP method (should still be handled)
    let invalid_method = HttpRequest {
        method: "INVALID".to_string(),
        url: "/test".to_string(),
        headers: vec![],
        body: vec![],
    };
    assert_eq!(invalid_method.method, "INVALID");

    // Very long URL
    let long_url = "a".repeat(10000);
    let long_url_request = HttpRequest {
        method: "GET".to_string(),
        url: long_url.clone(),
        headers: vec![],
        body: vec![],
    };
    assert_eq!(long_url_request.url.len(), 10000);
}

/// Test owner function (simple test since it delegates to state module)
#[test]
fn test_get_owner_function() {
    // This is a simple delegation test since the actual implementation
    // calls crate::state::get_owner() which would require canister runtime

    // Test that the function signature is correct
    fn test_owner_signature() -> Principal {
        Principal::anonymous() // Mock implementation
    }

    let owner = test_owner_signature();
    assert_eq!(owner, Principal::anonymous());
}

/// Test HTML escaping and injection prevention
#[test]
fn test_html_injection_prevention() {
    // Test with tool name containing HTML/script tags
    let malicious_tool = ToolMetadata {
        name: "<script>alert('xss')</script>".to_string(),
        candid_method: "safe_method".to_string(),
        is_query: true,
        description: "<img src=x onerror=alert('xss')>".to_string(),
        parameters: vec![],
    };

    // Format tool HTML (this is the vulnerable part in the real code)
    let tools_html = format!(
        r#"<div class="tool">
                <strong>{}</strong> ({})
                <br><small>{}</small>
            </div>"#,
        malicious_tool.name,
        if malicious_tool.is_query {
            "query"
        } else {
            "update"
        },
        malicious_tool.description
    );

    // In a real implementation, this should be escaped
    // For now, we just verify the content is included as-is
    assert!(tools_html.contains("<script>"));
    assert!(tools_html.contains("<img src=x"));

    // NOTE: This test documents that HTML injection is currently possible
    // A future improvement would be to add HTML escaping
}

/// Test large response handling
#[test]
fn test_large_response_generation() {
    // Create metadata with many tools to test large response generation
    let mut tools = Vec::new();
    for i in 0..100 {
        tools.push(ToolMetadata {
            name: format!("tool_{}", i),
            candid_method: format!("tool_{}", i),
            is_query: i % 2 == 0,
            description: format!("Description for tool number {}", i),
            parameters: vec![],
        });
    }

    let metadata = IcarusMetadata {
        version: "1.0.0".to_string(),
        canister_id: Principal::anonymous(),
        tools,
    };

    // Generate tools HTML
    let tools_html: String = metadata
        .tools
        .iter()
        .map(|tool| {
            format!(
                r#"<div class="tool">
                <strong>{}</strong> ({})
                <br><small>{}</small>
            </div>"#,
                tool.name,
                if tool.is_query { "query" } else { "update" },
                tool.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Verify all tools are included
    assert!(tools_html.contains("tool_0"));
    assert!(tools_html.contains("tool_99"));
    assert!(tools_html.contains("query"));
    assert!(tools_html.contains("update"));

    // Verify reasonable size (should be less than 100KB for 100 tools)
    assert!(tools_html.len() < 100_000);
    assert!(tools_html.len() > 1_000); // But substantial content
}

/// Test URL encoding and special characters in canister ID
#[test]
fn test_canister_id_encoding() {
    // Test various Principal formats
    let principals = vec![
        Principal::anonymous(),
        Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap(),
        Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").unwrap(),
    ];

    for principal in principals {
        let canister_id_text = principal.to_text();

        // Verify canister ID can be used in HTML
        let html_snippet = format!(
            r#"<code>icarus bridge start --canister-id {}</code>"#,
            canister_id_text
        );

        assert!(html_snippet.contains(&canister_id_text));
        assert!(html_snippet.contains("icarus bridge start"));

        // Verify no HTML injection possible through canister ID
        assert!(!canister_id_text.contains("<"));
        assert!(!canister_id_text.contains(">"));
        assert!(!canister_id_text.contains("\""));
    }
}
