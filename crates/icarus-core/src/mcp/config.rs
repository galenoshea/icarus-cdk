//! Configuration for the MCP server

#[cfg(feature = "mcp")]
use candid::Principal;
#[cfg(feature = "mcp")]
use std::time::Duration;

/// Configuration for the MCP server
#[cfg(feature = "mcp")]
#[derive(Debug, Clone)]
pub struct McpConfig {
    /// The canister ID to connect to
    pub canister_id: Principal,

    /// IC network URL
    pub ic_url: String,

    /// Connection timeout for canister calls
    pub timeout: Duration,

    /// Whether to fetch root key (for local development)
    pub fetch_root_key: bool,

    /// Maximum number of concurrent requests
    pub max_concurrent_requests: usize,
}

/// Builder for McpConfig with ergonomic API and validation
#[cfg(feature = "mcp")]
#[derive(Debug, Clone)]
pub struct McpConfigBuilder {
    canister_id: Option<Principal>,
    ic_url: Option<String>,
    timeout: Option<Duration>,
    fetch_root_key: Option<bool>,
    max_concurrent_requests: Option<usize>,
}

/// Error type for configuration building
#[cfg(feature = "mcp")]
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Canister ID was not provided when required
    #[error("Canister ID is required")]
    MissingCanisterId,
    /// IC URL was not provided when required
    #[error("IC URL is required")]
    MissingIcUrl,
    /// Timeout value is outside the valid range
    #[error("Timeout must be between 1 and 300 seconds, got {0}")]
    InvalidTimeout(u64),
    /// Max concurrent requests value is outside the valid range
    #[error("Max concurrent requests must be between 1 and 100, got {0}")]
    InvalidConcurrency(usize),
}

#[cfg(feature = "mcp")]
impl McpConfig {
    /// Create a new configuration (kept for backward compatibility)
    pub fn new(canister_id: Principal, ic_url: String) -> Self {
        let fetch_root_key = ic_url.contains("localhost") || ic_url.contains("127.0.0.1");

        Self {
            canister_id,
            ic_url,
            timeout: Duration::from_secs(30),
            fetch_root_key,
            max_concurrent_requests: 10,
        }
    }

    /// Configure for local development (kept for backward compatibility)
    pub fn local(canister_id: Principal) -> Self {
        Self::new(canister_id, "http://localhost:4943".to_string())
    }

    /// Configure for mainnet (kept for backward compatibility)
    pub fn mainnet(canister_id: Principal) -> Self {
        Self::new(canister_id, "https://ic0.app".to_string())
    }

    /// Set connection timeout (kept for backward compatibility)
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set maximum concurrent requests (kept for backward compatibility)
    pub fn with_max_concurrent_requests(mut self, max: usize) -> Self {
        self.max_concurrent_requests = max;
        self
    }

    /// Create a new builder instance
    #[inline]
    pub fn builder() -> McpConfigBuilder {
        McpConfigBuilder::new()
    }
}

#[cfg(feature = "mcp")]
impl McpConfigBuilder {
    /// Create a new builder
    #[inline]
    pub fn new() -> Self {
        Self {
            canister_id: None,
            ic_url: None,
            timeout: None,
            fetch_root_key: None,
            max_concurrent_requests: None,
        }
    }

    /// Set the canister ID (required)
    #[inline]
    pub fn canister_id(mut self, canister_id: Principal) -> Self {
        self.canister_id = Some(canister_id);
        self
    }

    /// Set the IC URL (required)
    #[inline]
    pub fn ic_url<S: Into<String>>(mut self, url: S) -> Self {
        self.ic_url = Some(url.into());
        self
    }

    /// Configure for local development
    #[inline]
    pub fn local(mut self) -> Self {
        self.ic_url = Some("http://localhost:4943".to_string());
        self.fetch_root_key = Some(true);
        self
    }

    /// Configure for mainnet
    #[inline]
    pub fn mainnet(mut self) -> Self {
        self.ic_url = Some("https://ic0.app".to_string());
        self.fetch_root_key = Some(false);
        self
    }

    /// Set connection timeout (1-300 seconds)
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set connection timeout in seconds (1-300 seconds)
    #[inline]
    pub fn timeout_secs(mut self, seconds: u64) -> Self {
        self.timeout = Some(Duration::from_secs(seconds));
        self
    }

    /// Explicitly set whether to fetch root key
    #[inline]
    pub fn fetch_root_key(mut self, fetch: bool) -> Self {
        self.fetch_root_key = Some(fetch);
        self
    }

    /// Set maximum concurrent requests (1-100)
    #[inline]
    pub fn max_concurrent_requests(mut self, max: usize) -> Self {
        self.max_concurrent_requests = Some(max);
        self
    }

    /// Build the configuration with validation
    pub fn build(self) -> Result<McpConfig, ConfigError> {
        let canister_id = self.canister_id.ok_or(ConfigError::MissingCanisterId)?;
        let ic_url = self.ic_url.ok_or(ConfigError::MissingIcUrl)?;

        // Validate timeout range
        let timeout = self.timeout.unwrap_or(Duration::from_secs(30));
        let timeout_secs = timeout.as_secs();
        if !(1..=300).contains(&timeout_secs) {
            return Err(ConfigError::InvalidTimeout(timeout_secs));
        }

        // Validate concurrency range
        let max_concurrent_requests = self.max_concurrent_requests.unwrap_or(10);
        if !(1..=100).contains(&max_concurrent_requests) {
            return Err(ConfigError::InvalidConcurrency(max_concurrent_requests));
        }

        // Auto-detect root key fetching if not explicitly set
        let fetch_root_key = self
            .fetch_root_key
            .unwrap_or_else(|| ic_url.contains("localhost") || ic_url.contains("127.0.0.1"));

        Ok(McpConfig {
            canister_id,
            ic_url,
            timeout,
            fetch_root_key,
            max_concurrent_requests,
        })
    }
}

#[cfg(feature = "mcp")]
impl Default for McpConfigBuilder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "mcp"))]
mod tests {
    use super::*;
    use candid::Principal;
    use std::time::Duration;

    #[test]
    fn test_local_config() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let config = McpConfig::local(canister_id);

        assert_eq!(config.canister_id, canister_id);
        assert_eq!(config.ic_url, "http://localhost:4943");
        assert!(config.fetch_root_key);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_concurrent_requests, 10);
    }

    #[test]
    fn test_mainnet_config() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let config = McpConfig::mainnet(canister_id);

        assert_eq!(config.canister_id, canister_id);
        assert_eq!(config.ic_url, "https://ic0.app");
        assert!(!config.fetch_root_key);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_concurrent_requests, 10);
    }

    #[test]
    fn test_with_timeout() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let config = McpConfig::local(canister_id).with_timeout(Duration::from_secs(60));

        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_with_max_concurrent_requests() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let config = McpConfig::local(canister_id).with_max_concurrent_requests(20);

        assert_eq!(config.max_concurrent_requests, 20);
    }

    // Builder pattern tests
    #[test]
    fn test_builder_basic() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let config = McpConfig::builder()
            .canister_id(canister_id)
            .ic_url("https://ic0.app")
            .build()
            .unwrap();

        assert_eq!(config.canister_id, canister_id);
        assert_eq!(config.ic_url, "https://ic0.app");
        assert!(!config.fetch_root_key); // Should be false for non-localhost
        assert_eq!(config.timeout, Duration::from_secs(30)); // Default
        assert_eq!(config.max_concurrent_requests, 10); // Default
    }

    #[test]
    fn test_builder_local() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let config = McpConfig::builder()
            .canister_id(canister_id)
            .local()
            .build()
            .unwrap();

        assert_eq!(config.canister_id, canister_id);
        assert_eq!(config.ic_url, "http://localhost:4943");
        assert!(config.fetch_root_key);
    }

    #[test]
    fn test_builder_mainnet() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let config = McpConfig::builder()
            .canister_id(canister_id)
            .mainnet()
            .build()
            .unwrap();

        assert_eq!(config.canister_id, canister_id);
        assert_eq!(config.ic_url, "https://ic0.app");
        assert!(!config.fetch_root_key);
    }

    #[test]
    fn test_builder_full_configuration() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let config = McpConfig::builder()
            .canister_id(canister_id)
            .ic_url("https://custom.ic.network")
            .timeout_secs(120)
            .max_concurrent_requests(25)
            .fetch_root_key(true)
            .build()
            .unwrap();

        assert_eq!(config.canister_id, canister_id);
        assert_eq!(config.ic_url, "https://custom.ic.network");
        assert_eq!(config.timeout, Duration::from_secs(120));
        assert_eq!(config.max_concurrent_requests, 25);
        assert!(config.fetch_root_key);
    }

    #[test]
    fn test_builder_missing_canister_id() {
        let result = McpConfig::builder().ic_url("https://ic0.app").build();

        match result {
            Err(ConfigError::MissingCanisterId) => (),
            _ => panic!("Expected MissingCanisterId error"),
        }
    }

    #[test]
    fn test_builder_missing_ic_url() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let result = McpConfig::builder().canister_id(canister_id).build();

        match result {
            Err(ConfigError::MissingIcUrl) => (),
            _ => panic!("Expected MissingIcUrl error"),
        }
    }

    #[test]
    fn test_builder_invalid_timeout() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let result = McpConfig::builder()
            .canister_id(canister_id)
            .ic_url("https://ic0.app")
            .timeout_secs(0) // Invalid: too low
            .build();

        match result {
            Err(ConfigError::InvalidTimeout(0)) => (),
            _ => panic!("Expected InvalidTimeout error"),
        }

        let result = McpConfig::builder()
            .canister_id(canister_id)
            .ic_url("https://ic0.app")
            .timeout_secs(500) // Invalid: too high
            .build();

        match result {
            Err(ConfigError::InvalidTimeout(500)) => (),
            _ => panic!("Expected InvalidTimeout error"),
        }
    }

    #[test]
    fn test_builder_invalid_concurrency() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let result = McpConfig::builder()
            .canister_id(canister_id)
            .ic_url("https://ic0.app")
            .max_concurrent_requests(0) // Invalid: too low
            .build();

        match result {
            Err(ConfigError::InvalidConcurrency(0)) => (),
            _ => panic!("Expected InvalidConcurrency error"),
        }

        let result = McpConfig::builder()
            .canister_id(canister_id)
            .ic_url("https://ic0.app")
            .max_concurrent_requests(200) // Invalid: too high
            .build();

        match result {
            Err(ConfigError::InvalidConcurrency(200)) => (),
            _ => panic!("Expected InvalidConcurrency error"),
        }
    }

    #[test]
    fn test_builder_auto_detect_localhost() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();

        // Test localhost detection
        let config = McpConfig::builder()
            .canister_id(canister_id)
            .ic_url("http://localhost:8080")
            .build()
            .unwrap();
        assert!(config.fetch_root_key);

        // Test 127.0.0.1 detection
        let config = McpConfig::builder()
            .canister_id(canister_id)
            .ic_url("http://127.0.0.1:4943")
            .build()
            .unwrap();
        assert!(config.fetch_root_key);

        // Test non-localhost URL
        let config = McpConfig::builder()
            .canister_id(canister_id)
            .ic_url("https://ic0.app")
            .build()
            .unwrap();
        assert!(!config.fetch_root_key);
    }

    #[test]
    fn test_builder_string_into_conversion() {
        let canister_id = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();

        // Test String conversion
        let config = McpConfig::builder()
            .canister_id(canister_id)
            .ic_url("https://ic0.app".to_string())
            .build()
            .unwrap();
        assert_eq!(config.ic_url, "https://ic0.app");

        // Test &str conversion
        let config = McpConfig::builder()
            .canister_id(canister_id)
            .ic_url("https://ic0.app")
            .build()
            .unwrap();
        assert_eq!(config.ic_url, "https://ic0.app");
    }

    #[test]
    fn test_builder_default() {
        let builder1 = McpConfigBuilder::default();
        let builder2 = McpConfigBuilder::new();

        // Both should behave the same way (fail due to missing required fields)
        let result1 = builder1.build();
        let result2 = builder2.build();

        assert!(result1.is_err());
        assert!(result2.is_err());
    }
}
