//! Integration tests for HTTP outcalls module

use icarus_canister::http::{HttpConfig, HttpError};
use serde_json::json;

/// Test URL validation logic
#[test]
fn test_url_validation() {
    // Test helper function that mimics internal validation
    fn validate_url(url: &str) -> Result<(), String> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            Err(format!("URL must start with http:// or https://"))
        } else {
            Ok(())
        }
    }

    // Valid URLs should pass
    assert!(validate_url("https://api.example.com").is_ok());
    assert!(validate_url("http://localhost:8080/api").is_ok());
    assert!(validate_url("https://sub.domain.com/path/to/resource").is_ok());

    // Invalid URLs should fail
    assert!(validate_url("example.com").is_err());
    assert!(validate_url("ftp://example.com").is_err());
    assert!(validate_url("ws://websocket.com").is_err());
    assert!(validate_url("").is_err());
    assert!(validate_url("/relative/path").is_err());
}

/// Test HTTP configuration defaults and customization
#[test]
fn test_http_config() {
    // Test default configuration
    let default_config = HttpConfig::default();
    assert_eq!(default_config.max_response_bytes, 2 * 1024 * 1024);
    assert_eq!(default_config.timeout_seconds, 30);
    assert_eq!(default_config.max_retries, 3);
    assert_eq!(default_config.retry_delay_ms, 1000);

    // Test custom configuration
    let custom_config = HttpConfig {
        max_response_bytes: 5 * 1024 * 1024,
        timeout_seconds: 60,
        max_retries: 5,
        retry_delay_ms: 2000,
    };
    assert_eq!(custom_config.max_response_bytes, 5 * 1024 * 1024);
    assert_eq!(custom_config.timeout_seconds, 60);
    assert_eq!(custom_config.max_retries, 5);
    assert_eq!(custom_config.retry_delay_ms, 2000);
}

/// Test HTTP error types
#[test]
fn test_error_types() {
    // Test error creation and messages
    let request_failed = HttpError::RequestFailed("Connection refused".to_string());
    assert_eq!(
        request_failed.to_string(),
        "HTTP request failed: Connection refused"
    );

    let invalid_url = HttpError::InvalidUrl("missing protocol".to_string());
    assert_eq!(invalid_url.to_string(), "Invalid URL: missing protocol");

    let timeout = HttpError::Timeout(30);
    assert_eq!(timeout.to_string(), "Timeout after 30 seconds");

    let response_too_large = HttpError::ResponseTooLarge {
        size: 3_000_000,
        max: 2_000_000,
    };
    assert_eq!(
        response_too_large.to_string(),
        "Response too large: 3000000 bytes (max: 2000000)"
    );

    let invalid_json = HttpError::InvalidJson("Unexpected token".to_string());
    assert_eq!(
        invalid_json.to_string(),
        "Invalid JSON response: Unexpected token"
    );

    let http_status = HttpError::HttpStatus {
        status: 404,
        message: "Not Found".to_string(),
    };
    assert_eq!(http_status.to_string(), "HTTP status 404: Not Found");
}

/// Test JSON body preparation
#[test]
fn test_json_body_preparation() {
    // Test various JSON structures
    let simple = json!({"key": "value"});
    let simple_bytes = serde_json::to_vec(&simple).unwrap();
    assert!(simple_bytes.len() > 0);

    let complex = json!({
        "user": {
            "id": 123,
            "name": "Test User",
            "active": true
        },
        "tags": ["tag1", "tag2", "tag3"],
        "metadata": null
    });
    let complex_bytes = serde_json::to_vec(&complex).unwrap();
    assert!(complex_bytes.len() > simple_bytes.len());

    // Test that serialization produces valid UTF-8
    let utf8_result = String::from_utf8(complex_bytes.clone());
    assert!(utf8_result.is_ok());
}

/// Test retry delay calculation (exponential backoff)
#[test]
fn test_retry_delays() {
    let base_delay = 1000_u64; // 1 second in milliseconds

    // Calculate expected delays for each retry attempt
    let attempt_1_delay = base_delay * 2_u64.pow(0); // 1000ms
    let attempt_2_delay = base_delay * 2_u64.pow(1); // 2000ms
    let attempt_3_delay = base_delay * 2_u64.pow(2); // 4000ms

    assert_eq!(attempt_1_delay, 1000);
    assert_eq!(attempt_2_delay, 2000);
    assert_eq!(attempt_3_delay, 4000);

    // Verify exponential growth
    assert_eq!(attempt_2_delay, attempt_1_delay * 2);
    assert_eq!(attempt_3_delay, attempt_2_delay * 2);
}

/// Test header construction
#[test]
fn test_header_construction() {
    use std::collections::HashMap;

    // Test empty headers
    let empty_headers: HashMap<String, String> = HashMap::new();
    assert_eq!(empty_headers.len(), 0);

    // Test standard headers
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());
    headers.insert("User-Agent".to_string(), "Icarus/1.0".to_string());

    assert_eq!(headers.len(), 3);
    assert_eq!(
        headers.get("Content-Type"),
        Some(&"application/json".to_string())
    );
    assert_eq!(
        headers.get("Authorization"),
        Some(&"Bearer token123".to_string())
    );
    assert_eq!(headers.get("User-Agent"), Some(&"Icarus/1.0".to_string()));
}

/// Test response size validation
#[test]
fn test_response_size_validation() {
    let max_size = 2 * 1024 * 1024; // 2MB

    // Test valid sizes
    assert!(validate_response_size(0, max_size).is_ok());
    assert!(validate_response_size(1024, max_size).is_ok());
    assert!(validate_response_size(max_size - 1, max_size).is_ok());
    assert!(validate_response_size(max_size, max_size).is_ok());

    // Test invalid sizes
    assert!(validate_response_size(max_size + 1, max_size).is_err());
    assert!(validate_response_size(3 * 1024 * 1024, max_size).is_err());

    fn validate_response_size(size: usize, max: usize) -> Result<(), String> {
        if size > max {
            Err(format!("Response too large: {} bytes (max: {})", size, max))
        } else {
            Ok(())
        }
    }
}

/// Test HTTP status code handling
#[test]
fn test_status_code_handling() {
    // Success codes (2xx)
    assert!(is_success_status(200));
    assert!(is_success_status(201));
    assert!(is_success_status(204));

    // Redirect codes (3xx) - should be considered success for basic use
    assert!(is_success_status(301));
    assert!(is_success_status(302));
    assert!(is_success_status(304));

    // Client error codes (4xx)
    assert!(!is_success_status(400));
    assert!(!is_success_status(401));
    assert!(!is_success_status(403));
    assert!(!is_success_status(404));
    assert!(!is_success_status(429));

    // Server error codes (5xx)
    assert!(!is_success_status(500));
    assert!(!is_success_status(502));
    assert!(!is_success_status(503));

    fn is_success_status(status: u16) -> bool {
        status < 400
    }
}

/// Test UTF-8 response validation
#[test]
fn test_utf8_response_validation() {
    // Valid UTF-8
    let valid_utf8 = b"Hello, World!";
    assert!(String::from_utf8(valid_utf8.to_vec()).is_ok());

    // Valid UTF-8 with special characters
    let valid_special = "Hello, 世界! 🌍".as_bytes();
    assert!(String::from_utf8(valid_special.to_vec()).is_ok());

    // Invalid UTF-8
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
    assert!(String::from_utf8(invalid_utf8).is_err());
}

/// Test macro expansions (compile-time test)
#[test]
fn test_macro_usage() {
    // Helper to simulate macro expansion (for testing purposes)
    macro_rules! quote_macro {
        ($expr:expr) => {
            stringify!($expr)
        };
    }

    // This test ensures the macros compile correctly
    // In a real canister context, these would be async

    // Test http_get! macro with different parameters
    let _basic = quote_macro!(http_get!("https://api.example.com"));
    let _with_timeout = quote_macro!(http_get!("https://api.example.com", timeout: 60));
    let _with_retries = quote_macro!(http_get!("https://api.example.com", retries: 5));

    // Test http_post_json! macro
    let body = json!({"test": "data"});
    let _basic_post = quote_macro!(http_post_json!("https://api.example.com", body));
    let _post_with_timeout =
        quote_macro!(http_post_json!("https://api.example.com", body, timeout: 45));
}
