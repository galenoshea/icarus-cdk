//! HTTP Fetcher Example
//!
//! This example demonstrates how to use HTTP outcalls in an ICP canister
//! to fetch external data from APIs. It showcases:
//! - Simple GET requests
//! - POST requests with JSON bodies
//! - Custom configurations for timeout and retries
//! - Error handling

use icarus::prelude::*;

/// MCP module containing all HTTP fetching tools
#[icarus_module]
mod tools {
    use super::*;

    /// Fetch data from a URL using GET request
    ///
    /// Example: fetch_url("https://api.github.com/repos/icarus-dev/icarus-sdk")
    #[update]
    #[icarus_tool("Fetch data from a URL using GET request")]
    pub async fn fetch_url(url: String) -> Result<String, String> {
        // Validate URL format
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err("URL must start with http:// or https://".to_string());
        }

        // Perform the HTTP GET request
        http::get(&url)
            .await
            .map_err(|e| format!("Failed to fetch {}: {}", url, e))
    }

    /// Fetch JSON data and parse it
    ///
    /// Example: fetch_json("https://api.github.com/repos/icarus-dev/icarus-sdk")
    #[update]
    #[icarus_tool("Fetch and parse JSON data from a URL")]
    pub async fn fetch_json(url: String) -> Result<String, String> {
        // Fetch the data
        let response = http::get(&url)
            .await
            .map_err(|e| format!("Failed to fetch {}: {}", url, e))?;

        // Parse and pretty-print the JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&response).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        Ok(serde_json::to_string_pretty(&parsed).unwrap_or_else(|_| response))
    }

    /// Post JSON data to a URL
    ///
    /// Example: post_data("https://httpbin.org/post", "{\"key\": \"value\"}")
    #[update]
    #[icarus_tool("Post JSON data to a URL")]
    pub async fn post_data(url: String, json_data: String) -> Result<String, String> {
        // Parse the JSON string
        let body: serde_json::Value =
            serde_json::from_str(&json_data).map_err(|e| format!("Invalid JSON data: {}", e))?;

        // Send the POST request
        http::post_json(&url, body)
            .await
            .map_err(|e| format!("Failed to post to {}: {}", url, e))
    }

    /// Fetch with custom timeout (in seconds)
    ///
    /// Example: fetch_with_timeout("https://slow-api.example.com", 60)
    #[update]
    #[icarus_tool("Fetch URL with custom timeout")]
    pub async fn fetch_with_timeout(url: String, timeout_seconds: u64) -> Result<String, String> {
        let mut config = http::HttpConfig::default();
        config.timeout_seconds = timeout_seconds;

        http::get_with_config(&url, config)
            .await
            .map_err(|e| format!("Failed to fetch {}: {}", url, e))
    }

    /// Fetch with custom retry attempts
    ///
    /// Example: fetch_with_retries("https://flaky-api.example.com", 5)
    #[update]
    #[icarus_tool("Fetch URL with custom retry attempts")]
    pub async fn fetch_with_retries(url: String, max_retries: u32) -> Result<String, String> {
        let mut config = http::HttpConfig::default();
        config.max_retries = max_retries;

        http::get_with_config(&url, config)
            .await
            .map_err(|e| format!("Failed to fetch {}: {}", url, e))
    }

    /// Check if a URL is reachable (returns status code)
    ///
    /// Example: check_url("https://example.com")
    #[update]
    #[icarus_tool("Check if a URL is reachable")]
    pub async fn check_url(url: String) -> Result<String, String> {
        // Try to fetch with minimal configuration
        let mut config = http::HttpConfig::default();
        config.max_response_bytes = 1024; // Only need headers
        config.timeout_seconds = 10;
        config.max_retries = 1;

        match http::get_with_config(&url, config).await {
            Ok(_) => Ok("URL is reachable (status < 400)".to_string()),
            Err(http::HttpError::HttpStatus { status, .. }) => {
                Ok(format!("URL returned status: {}", status))
            }
            Err(e) => Err(format!("URL is not reachable: {}", e)),
        }
    }

    /// Fetch cryptocurrency price from CoinGecko API
    ///
    /// Example: fetch_crypto_price("bitcoin")
    #[update]
    #[icarus_tool("Fetch cryptocurrency price from CoinGecko")]
    pub async fn fetch_crypto_price(coin_id: String) -> Result<String, String> {
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
            coin_id
        );

        let response = http::get(&url)
            .await
            .map_err(|e| format!("Failed to fetch price: {}", e))?;

        // Parse the response
        let data: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Extract the price
        if let Some(price) = data.get(&coin_id).and_then(|c| c.get("usd")) {
            Ok(format!("{} price: ${}", coin_id, price))
        } else {
            Err(format!("Price not found for {}", coin_id))
        }
    }

    /// Fetch weather data for a city
    ///
    /// Note: Requires API key for production use
    /// Example: fetch_weather("London")
    #[update]
    #[icarus_tool("Fetch weather data for a city")]
    pub async fn fetch_weather(city: String) -> Result<String, String> {
        // Using a free weather API (may have rate limits)
        let url = format!("https://wttr.in/{}?format=j1", city);

        let response = http::get(&url)
            .await
            .map_err(|e| format!("Failed to fetch weather: {}", e))?;

        // Parse and extract key information
        let data: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse weather data: {}", e))?;

        if let Some(current) = data.get("current_condition").and_then(|c| c.get(0)) {
            let temp = current
                .get("temp_C")
                .and_then(|t| t.as_str())
                .unwrap_or("N/A");
            let desc = current
                .get("weatherDesc")
                .and_then(|d| d.get(0))
                .and_then(|d| d.get("value"))
                .and_then(|v| v.as_str())
                .unwrap_or("N/A");

            Ok(format!("{}: {}°C, {}", city, temp, desc))
        } else {
            Err("Weather data not available".to_string())
        }
    }
}

// Export the Candid interface for the canister
ic_cdk::export_candid!();
