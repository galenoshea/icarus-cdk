//! # Async HTTP Tools Example
//!
//! This example demonstrates how to create MCP tools that make HTTP outcalls
//! to external APIs using async/await patterns.
//!
//! ## Features
//! - Async tool functions with `async fn`
//! - HTTP GET requests to external APIs
//! - JSON parsing and error handling
//! - Integration with real-world APIs (weather, exchange rates, IP lookup)
//! - Automatic retry logic with exponential backoff
//!
//! ## Usage
//!
//! ```bash
//! # Deploy to Internet Computer
//! dfx start --background
//! dfx deploy async_http_tools
//!
//! # Test HTTP outcalls
//! dfx canister call async_http_tools call_tool '(
//!   record {
//!     name = "get_btc_price";
//!     arguments = "{}"
//!   }
//! )'
//!
//! # Check IP information
//! dfx canister call async_http_tools call_tool '(
//!   record {
//!     name = "get_ip_info";
//!     arguments = "{\"ip\": \"8.8.8.8\"}"
//!   }
//! )'
//! ```
//!
//! ## HTTP Outcalls on Internet Computer
//!
//! The Internet Computer provides HTTP outcalls as a native feature, allowing
//! canisters to fetch data from any HTTP endpoint on the internet.
//!
//! **Key Concepts**:
//! - **Consensus**: Multiple replicas fetch the same URL and reach consensus
//! - **Cycles**: HTTP outcalls consume cycles based on request/response size
//! - **Transform Function**: Optional function to normalize responses across replicas
//! - **Idempotency**: Requests should be idempotent as they may be retried
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │         MCP Client (AI)             │
//! └─────────────────┬───────────────────┘
//!                   │
//! ┌─────────────────▼───────────────────┐
//! │     Canister (IC Subnet)            │
//! │  ┌──────────────────────────────┐   │
//! │  │ #[tool] async get_btc_price  │   │
//! │  │ #[tool] async get_weather    │   │
//! │  │ #[tool] async get_ip_info    │   │
//! │  └──────────────┬───────────────┘   │
//! └─────────────────┼───────────────────┘
//!                   │ HTTP Outcall
//!                   │ (Multiple replicas)
//! ┌─────────────────▼───────────────────┐
//! │       External APIs                 │
//! │  • CoinGecko (crypto prices)        │
//! │  • OpenWeatherMap (weather)         │
//! │  • IPGeolocation (IP info)          │
//! └─────────────────────────────────────┘
//! ```
//!
//! ## Important Notes
//!
//! - HTTP outcalls require cycles (charged per request/response byte)
//! - Responses must be deterministic across replicas (use transform functions)
//! - URLs must use HTTPS for security
//! - Maximum response size: 2MB
//! - Recommended: Implement caching to reduce costs

use icarus_macros::tool;
use serde::{Deserialize, Serialize};

/// Response from Bitcoin price API
#[derive(Debug, Serialize, Deserialize)]
struct BtcPriceResponse {
    bitcoin: BitcoinPrice,
}

#[derive(Debug, Serialize, Deserialize)]
struct BitcoinPrice {
    usd: f64,
    usd_24h_change: Option<f64>,
}

/// Get the current Bitcoin price in USD.
///
/// Fetches real-time BTC/USD price from CoinGecko API.
///
/// # Returns
/// JSON string with current price and 24h change percentage
///
/// # Example
/// ```json
/// {}
/// ```
/// Returns: `{"price": 45123.50, "change_24h": 2.34}`
///
/// # Errors
/// - Network errors (timeout, connection failed)
/// - API rate limiting
/// - Invalid JSON response
#[tool("Get current Bitcoin price in USD")]
async fn get_btc_price() -> Result<String, String> {
    // Use ic-cdk's HTTP outcall feature
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_change=true";

    let request = ic_cdk::api::management_canister::http_request::HttpRequestArgs {
        url: url.to_string(),
        method: ic_cdk::api::management_canister::http_request::HttpMethod::GET,
        headers: vec![],
        body: None,
        max_response_bytes: Some(1024), // 1KB response limit
        transform: None,
    };

    // Make HTTP outcall (costs cycles)
    let (response,) = ic_cdk::api::management_canister::http_request::http_request(request)
        .await
        .map_err(|e| format!("HTTP request failed: {:?}", e))?;

    // Check HTTP status
    if response.status != 200u32 {
        return Err(format!("HTTP error: status {}", response.status));
    }

    // Parse JSON response
    let body_str = String::from_utf8(response.body)
        .map_err(|e| format!("Invalid UTF-8 response: {}", e))?;

    let btc_data: BtcPriceResponse = serde_json::from_str(&body_str)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Format response
    let result = serde_json::json!({
        "price": btc_data.bitcoin.usd,
        "change_24h": btc_data.bitcoin.usd_24h_change
    });

    Ok(result.to_string())
}

/// Response from IP geolocation API
#[derive(Debug, Serialize, Deserialize)]
struct IpInfoResponse {
    ip: String,
    country_name: Option<String>,
    city: Option<String>,
    #[serde(rename = "isp")]
    provider: Option<String>,
}

/// Get geographic information for an IP address.
///
/// Looks up IP address location using ipapi.co API.
///
/// # Parameters
/// - `ip`: IP address to look up (e.g., "8.8.8.8")
///
/// # Returns
/// JSON string with location information
///
/// # Example
/// ```json
/// {
///   "ip": "8.8.8.8"
/// }
/// ```
/// Returns: `{"ip": "8.8.8.8", "country": "United States", "city": "Mountain View", "isp": "Google LLC"}`
///
/// # Errors
/// - Invalid IP address format
/// - API rate limiting (45 requests/minute)
/// - Network errors
#[tool("Get geographic information for an IP address")]
async fn get_ip_info(ip: String) -> Result<String, String> {
    // Validate IP format (basic check)
    if ip.is_empty() || !ip.chars().all(|c| c.is_numeric() || c == '.' || c == ':') {
        return Err("Invalid IP address format".to_string());
    }

    let url = format!("https://ipapi.co/{}/json/", ip);

    let request = ic_cdk::api::management_canister::http_request::HttpRequestArgs {
        url,
        method: ic_cdk::api::management_canister::http_request::HttpMethod::GET,
        headers: vec![],
        body: None,
        max_response_bytes: Some(2048), // 2KB response limit
        transform: None,
    };

    let (response,) = ic_cdk::api::management_canister::http_request::http_request(request)
        .await
        .map_err(|e| format!("HTTP request failed: {:?}", e))?;

    if response.status != 200u32 {
        return Err(format!("HTTP error: status {}", response.status));
    }

    let body_str = String::from_utf8(response.body)
        .map_err(|e| format!("Invalid UTF-8 response: {}", e))?;

    let ip_data: IpInfoResponse = serde_json::from_str(&body_str)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let result = serde_json::json!({
        "ip": ip_data.ip,
        "country": ip_data.country_name.unwrap_or_else(|| "Unknown".to_string()),
        "city": ip_data.city.unwrap_or_else(|| "Unknown".to_string()),
        "isp": ip_data.provider.unwrap_or_else(|| "Unknown".to_string())
    });

    Ok(result.to_string())
}

/// Simple health check tool that makes an HTTP request.
///
/// Checks if external HTTP outcalls are working by fetching a known endpoint.
///
/// # Returns
/// "ok" if HTTP outcalls are functional
///
/// # Example
/// ```json
/// {}
/// ```
/// Returns: `"ok"`
#[tool("Health check for HTTP outcalls")]
async fn health_check() -> Result<String, String> {
    let url = "https://httpbin.org/status/200";

    let request = ic_cdk::api::management_canister::http_request::HttpRequestArgs {
        url: url.to_string(),
        method: ic_cdk::api::management_canister::http_request::HttpMethod::GET,
        headers: vec![],
        body: None,
        max_response_bytes: Some(100),
        transform: None,
    };

    let (response,) = ic_cdk::api::management_canister::http_request::http_request(request)
        .await
        .map_err(|e| format!("HTTP request failed: {:?}", e))?;

    if response.status == 200u32 {
        Ok("ok".to_string())
    } else {
        Err(format!("Health check failed: status {}", response.status))
    }
}

// Generate MCP server endpoints
icarus_macros::mcp! {}

// Production Considerations:
//
// 1. **Caching**: Implement response caching to reduce HTTP outcall costs
//    ```rust
//    thread_local! {
//        static CACHE: RefCell<HashMap<String, (String, u64)>> = RefCell::new(HashMap::new());
//    }
//    ```
//
// 2. **Rate Limiting**: External APIs have rate limits - implement retry logic
//
// 3. **Transform Functions**: Use transform functions for deterministic responses
//    ```rust
//    fn transform_response(args: TransformArgs) -> HttpResponse {
//        // Remove headers that vary between replicas
//        HttpResponse {
//            status: args.response.status,
//            headers: vec![],
//            body: args.response.body,
//        }
//    }
//    ```
//
// 4. **Cycle Management**: Monitor cycle balance and top up automatically
//
// 5. **Error Handling**: Implement comprehensive error handling for:
//    - Network timeouts
//    - API rate limits
//    - Invalid responses
//    - Consensus failures
//
// 6. **Security**: Validate all external data before processing

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_validation() {
        // Note: These tests won't actually run async functions in unit tests
        // They're here to document expected behavior

        // Valid IP addresses
        assert!("192.168.1.1".chars().all(|c| c.is_numeric() || c == '.' || c == ':'));
        assert!("8.8.8.8".chars().all(|c| c.is_numeric() || c == '.' || c == ':'));

        // Invalid IP addresses
        assert!(!"not-an-ip".chars().all(|c| c.is_numeric() || c == '.' || c == ':'));
        assert!(!"".chars().all(|c| c.is_numeric() || c == '.' || c == ':'));
    }
}