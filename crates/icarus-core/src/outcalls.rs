//! HTTP outcalls support for external API integration

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::Result;

/// HTTP method types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

/// HTTP request for outcalls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    /// HTTP method
    pub method: HttpMethod,
    /// Target URL
    pub url: String,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body (for POST, PUT, PATCH)
    pub body: Option<Vec<u8>>,
    /// Maximum response size in bytes
    pub max_response_bytes: Option<u64>,
}

/// HTTP response from outcalls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Vec<u8>,
}

/// Trait for making HTTP outcalls
#[async_trait]
pub trait HttpOutcalls: Send + Sync {
    /// Make an HTTP request
    async fn request(&self, request: HttpRequest) -> Result<HttpResponse>;
    
    /// Convenience method for GET requests
    async fn get(&self, url: &str) -> Result<HttpResponse> {
        self.request(HttpRequest {
            method: HttpMethod::GET,
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
            max_response_bytes: None,
        }).await
    }
    
    /// Convenience method for POST requests with JSON
    async fn post_json(&self, url: &str, json: &serde_json::Value) -> Result<HttpResponse> {
        let body = serde_json::to_vec(json)
            .map_err(crate::error::IcarusError::Serialization)?;
            
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        self.request(HttpRequest {
            method: HttpMethod::POST,
            url: url.to_string(),
            headers,
            body: Some(body),
            max_response_bytes: None,
        }).await
    }
}

/// Configuration for HTTP outcalls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcallsConfig {
    /// Maximum number of concurrent outcalls
    pub max_concurrent: u32,
    /// Default timeout in seconds
    pub timeout_secs: u64,
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
    /// Allowed domains (empty = all allowed)
    pub allowed_domains: Vec<String>,
}

impl Default for OutcallsConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 5,
            timeout_secs: 30,
            verify_ssl: true,
            allowed_domains: Vec::new(),
        }
    }
}

/// Builder for HTTP requests
pub struct HttpRequestBuilder {
    method: HttpMethod,
    url: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
    max_response_bytes: Option<u64>,
}

impl HttpRequestBuilder {
    /// Create a new request builder
    pub fn new(method: HttpMethod, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: HashMap::new(),
            body: None,
            max_response_bytes: None,
        }
    }
    
    /// Add a header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    
    /// Set the request body
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
    
    /// Set JSON body
    pub fn json_body(mut self, json: &serde_json::Value) -> Result<Self> {
        let body = serde_json::to_vec(json)
            .map_err(crate::error::IcarusError::Serialization)?;
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        Ok(self.body(body))
    }
    
    /// Set maximum response size
    pub fn max_response_bytes(mut self, max: u64) -> Self {
        self.max_response_bytes = Some(max);
        self
    }
    
    /// Build the request
    pub fn build(self) -> HttpRequest {
        HttpRequest {
            method: self.method,
            url: self.url,
            headers: self.headers,
            body: self.body,
            max_response_bytes: self.max_response_bytes,
        }
    }
}

/// Extension methods for HTTP responses
impl HttpResponse {
    /// Check if the response is successful (2xx status)
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }
    
    /// Get response body as string
    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.body.clone())
            .map_err(|e| crate::error::IcarusError::Canister(format!("Invalid UTF-8 in response: {}", e)))
    }
    
    /// Parse response body as JSON
    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        serde_json::from_slice(&self.body)
            .map_err(crate::error::IcarusError::Serialization)
    }
    
    /// Get a header value
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }
}