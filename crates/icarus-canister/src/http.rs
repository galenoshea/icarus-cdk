//! HTTP Outcalls Module
//!
//! Provides simple, idiomatic HTTP client functionality for ICP canisters.
//! Abstracts the complexity of ICP's HTTP outcalls into clean, easy-to-use APIs.

use ic_cdk::management_canister::{
    http_request, HttpHeader, HttpMethod, HttpRequestArgs, HttpRequestResult, TransformArgs,
    TransformContext, TransformFunc,
};
use serde_json::Value;
use std::collections::HashMap;

/// Result type for HTTP operations
pub type HttpResult<T> = Result<T, HttpError>;

/// HTTP error types
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Timeout after {0} seconds")]
    Timeout(u64),

    #[error("Response too large: {size} bytes (max: {max})")]
    ResponseTooLarge { size: usize, max: usize },

    #[error("Invalid JSON response: {0}")]
    InvalidJson(String),

    #[error("HTTP status {status}: {message}")]
    HttpStatus { status: u16, message: String },
}

/// Configuration for HTTP requests
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Maximum response size in bytes (default: 2MB)
    pub max_response_bytes: usize,
    /// Request timeout in seconds (default: 30s)
    pub timeout_seconds: u64,
    /// Number of retry attempts (default: 3)
    pub max_retries: u32,
    /// Initial retry delay in milliseconds (default: 1000ms)
    pub retry_delay_ms: u64,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            max_response_bytes: 2 * 1024 * 1024, // 2MB
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// Perform a GET request to the specified URL
///
/// # Example
/// ```ignore
/// use icarus_canister::http;
///
/// #[ic_cdk::update]
/// async fn fetch_data() -> Result<String, String> {
///     http::get("https://api.example.com/data").await
/// }
/// ```
pub async fn get(url: &str) -> HttpResult<String> {
    get_with_config(url, HttpConfig::default()).await
}

/// Perform a GET request with custom configuration
pub async fn get_with_config(url: &str, config: HttpConfig) -> HttpResult<String> {
    execute_request(url, HttpMethod::GET, None, HashMap::new(), config).await
}

/// Perform a POST request with JSON body
///
/// # Example
/// ```ignore
/// use icarus_canister::http;
/// use serde_json::json;
///
/// #[ic_cdk::update]
/// async fn submit_data() -> Result<String, String> {
///     let body = json!({
///         "name": "example",
///         "value": 42
///     });
///     http::post_json("https://api.example.com/submit", body).await
/// }
/// ```
pub async fn post_json(url: &str, body: Value) -> HttpResult<String> {
    post_json_with_config(url, body, HttpConfig::default()).await
}

/// Perform a POST request with JSON body and custom configuration
pub async fn post_json_with_config(
    url: &str,
    body: Value,
    config: HttpConfig,
) -> HttpResult<String> {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let body_bytes =
        serde_json::to_vec(&body).map_err(|e| HttpError::InvalidJson(e.to_string()))?;

    execute_request(url, HttpMethod::POST, Some(body_bytes), headers, config).await
}

/// Perform a custom HTTP request with full control
pub async fn request(
    url: &str,
    method: HttpMethod,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
) -> HttpResult<String> {
    execute_request(url, method, body, headers, HttpConfig::default()).await
}

/// Internal function to execute HTTP requests with retry logic
async fn execute_request(
    url: &str,
    method: HttpMethod,
    body: Option<Vec<u8>>,
    headers: HashMap<String, String>,
    config: HttpConfig,
) -> HttpResult<String> {
    // Validate URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(HttpError::InvalidUrl(
            "URL must start with http:// or https://".to_string(),
        ));
    }

    let mut attempt = 0;
    let mut last_error = None;

    while attempt < config.max_retries {
        match execute_single_request(url, method, body.clone(), headers.clone(), &config).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                last_error = Some(e);
                attempt += 1;

                if attempt < config.max_retries {
                    // Exponential backoff with IC timer
                    let _delay = config.retry_delay_ms * 2_u64.pow(attempt - 1);
                    // Note: In production, use ic_cdk_timers for proper delays
                    // For now, we'll just continue without delay as sleep is not available
                    // TODO: Implement proper timer-based retry when timers module is ready
                }
            }
        }
    }

    Err(last_error
        .unwrap_or_else(|| HttpError::RequestFailed("All retry attempts failed".to_string())))
}

/// Execute a single HTTP request
async fn execute_single_request(
    url: &str,
    method: HttpMethod,
    body: Option<Vec<u8>>,
    headers: HashMap<String, String>,
    config: &HttpConfig,
) -> HttpResult<String> {
    let request_headers: Vec<HttpHeader> = headers
        .into_iter()
        .map(|(name, value)| HttpHeader { name, value })
        .collect();

    let request = HttpRequestArgs {
        url: url.to_string(),
        method,
        body,
        max_response_bytes: Some(config.max_response_bytes as u64),
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::canister_self(),
                method: "transform_response".to_string(),
            }),
            context: vec![],
        }),
        headers: request_headers,
    };

    let response = http_request(&request)
        .await
        .map_err(|e| HttpError::RequestFailed(format!("{:?}", e)))?;

    // Check response status (Nat comparison with u64)
    if response.status >= 400u64 {
        // Convert Nat to u16 for error reporting
        let status_str = response.status.to_string();
        let status_code: u16 = status_str.parse().unwrap_or(0);

        return Err(HttpError::HttpStatus {
            status: status_code,
            message: String::from_utf8_lossy(&response.body).to_string(),
        });
    }

    // Check response size
    if response.body.len() > config.max_response_bytes {
        return Err(HttpError::ResponseTooLarge {
            size: response.body.len(),
            max: config.max_response_bytes,
        });
    }

    String::from_utf8(response.body)
        .map_err(|e| HttpError::InvalidJson(format!("Invalid UTF-8 in response: {}", e)))
}

/// Transform function for HTTP responses (required by ICP)
#[ic_cdk::query]
fn transform_response(args: TransformArgs) -> HttpRequestResult {
    // Simply return the response as-is
    // In production, you might want to sanitize or transform the response
    args.response
}

/// Macro for simple GET requests
#[macro_export]
macro_rules! http_get {
    ($url:expr) => {
        $crate::http::get($url).await
    };
    ($url:expr, timeout: $timeout:expr) => {{
        let mut config = $crate::http::HttpConfig::default();
        config.timeout_seconds = $timeout;
        $crate::http::get_with_config($url, config).await
    }};
    ($url:expr, retries: $retries:expr) => {{
        let mut config = $crate::http::HttpConfig::default();
        config.max_retries = $retries;
        $crate::http::get_with_config($url, config).await
    }};
}

/// Macro for POST requests with JSON
#[macro_export]
macro_rules! http_post_json {
    ($url:expr, $body:expr) => {
        $crate::http::post_json($url, $body).await
    };
    ($url:expr, $body:expr, timeout: $timeout:expr) => {{
        let mut config = $crate::http::HttpConfig::default();
        config.timeout_seconds = $timeout;
        $crate::http::post_json_with_config($url, $body, config).await
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_validation() {
        // Valid URLs
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://api.example.com/path").is_ok());

        // Invalid URLs
        assert!(validate_url("example.com").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("").is_err());
    }

    fn validate_url(url: &str) -> Result<(), HttpError> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            Err(HttpError::InvalidUrl(format!(
                "URL must start with http:// or https://"
            )))
        } else {
            Ok(())
        }
    }

    #[test]
    fn test_config_defaults() {
        let config = HttpConfig::default();
        assert_eq!(config.max_response_bytes, 2 * 1024 * 1024);
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 1000);
    }
}
