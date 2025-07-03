//! Bridge configuration types

use serde::{Deserialize, Serialize};

/// Configuration for the Icarus bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// ICP network URL
    pub ic_url: String,
    
    /// Local port for WebSocket/HTTP server
    pub port: u16,
    
    /// Whether to fetch root key (false in production)
    pub fetch_root_key: bool,
    
    /// Session timeout in seconds
    pub session_timeout: u64,
    
    /// Maximum request size in bytes
    pub max_request_size: u64,
    
    /// Enable debug logging
    pub debug: bool,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            ic_url: "http://localhost:8080".to_string(),
            port: 3000,
            fetch_root_key: true, // For local development
            session_timeout: 3600, // 1 hour
            max_request_size: 10 * 1024 * 1024, // 10MB
            debug: false,
        }
    }
}

impl BridgeConfig {
    /// Create config for local development
    pub fn local() -> Self {
        Self {
            ic_url: "http://localhost:8080".to_string(),
            fetch_root_key: true,
            debug: true,
            ..Default::default()
        }
    }
    
    /// Create config for IC mainnet
    pub fn mainnet() -> Self {
        Self {
            ic_url: "https://ic0.app".to_string(),
            fetch_root_key: false,
            debug: false,
            ..Default::default()
        }
    }
    
    /// Load config from environment variables
    pub fn from_env() -> Self {
        Self {
            ic_url: std::env::var("ICARUS_IC_URL")
                .unwrap_or_else(|_| Self::default().ic_url),
            port: std::env::var("ICARUS_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(Self::default().port),
            fetch_root_key: std::env::var("ICARUS_FETCH_ROOT_KEY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(Self::default().fetch_root_key),
            session_timeout: std::env::var("ICARUS_SESSION_TIMEOUT")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(Self::default().session_timeout),
            max_request_size: std::env::var("ICARUS_MAX_REQUEST_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Self::default().max_request_size),
            debug: std::env::var("ICARUS_DEBUG")
                .ok()
                .and_then(|d| d.parse().ok())
                .unwrap_or(Self::default().debug),
        }
    }
}