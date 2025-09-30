//! MCP Bridge Server infrastructure for connecting canisters to AI clients
//! These are core infrastructure components that will be used as the CLI expands

#![allow(dead_code)]

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::config::mcp::McpConfig;

/// MCP Bridge Server trait
#[async_trait]
pub(crate) trait McpBridgeServer: Send {
    async fn run(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
}

/// Simple MCP Bridge Server implementation
pub(crate) struct SimpleBridgeServer {
    host: String,
    port: u16,
    config: Arc<RwLock<McpConfig>>,
    running: Arc<RwLock<bool>>,
}

impl SimpleBridgeServer {
    pub(crate) fn new(host: &str, port: u16, config: McpConfig) -> Result<Self> {
        Ok(Self {
            host: host.to_string(),
            port,
            config: Arc::new(RwLock::new(config)),
            running: Arc::new(RwLock::new(false)),
        })
    }

    async fn handle_connection(&self, stream: tokio::net::TcpStream) -> Result<()> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let peer_addr = stream.peer_addr()?;
        info!("New connection from: {}", peer_addr);

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;

            if bytes_read == 0 {
                info!("Connection closed by client: {}", peer_addr);
                break;
            }

            let trimmed_line = line.trim();
            if trimmed_line.is_empty() {
                continue;
            }

            info!("Received from {}: {}", peer_addr, trimmed_line);

            // Parse and handle MCP request
            let response = self.handle_mcp_request(trimmed_line).await;

            match response {
                Ok(resp) => {
                    writer.write_all(resp.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                    writer.flush().await?;
                }
                Err(e) => {
                    error!("Error handling MCP request: {}", e);
                    let error_response = format!(r#"{{"error": "{}"}}"#, e);
                    writer.write_all(error_response.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                    writer.flush().await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_mcp_request(&self, request: &str) -> Result<String> {
        // Try to parse as JSON
        let request_json: serde_json::Value =
            serde_json::from_str(request).map_err(|_| anyhow!("Invalid JSON request"))?;

        let method = request_json
            .get("method")
            .and_then(|m| m.as_str())
            .ok_or_else(|| anyhow!("Missing method in request"))?;

        match method {
            "list_tools" => self.handle_list_tools().await,
            "call_tool" => self.handle_call_tool(&request_json).await,
            "get_server_info" => self.handle_get_server_info().await,
            "ping" => Ok(r#"{"result": "pong"}"#.to_string()),
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    async fn handle_list_tools(&self) -> Result<String> {
        let config = self.config.read().await;
        let servers = &config.servers;

        let mut tools = Vec::new();
        for server in servers {
            if server.enabled {
                tools.push(serde_json::json!({
                    "name": server.name,
                    "canister_id": server.canister_id,
                    "network": server.network,
                    "url": server.url
                }));
            }
        }

        let response = serde_json::json!({
            "result": {
                "tools": tools
            }
        });

        Ok(serde_json::to_string(&response)?)
    }

    async fn handle_call_tool(&self, request: &serde_json::Value) -> Result<String> {
        let params = request
            .get("params")
            .ok_or_else(|| anyhow!("Missing params in call_tool request"))?;

        let tool_name = params
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| anyhow!("Missing tool name"))?;

        let args = params.get("arguments").unwrap_or(&serde_json::Value::Null);

        // Find the server for this tool
        let config = self.config.read().await;
        let server = config
            .servers
            .iter()
            .find(|s| s.name == tool_name && s.enabled)
            .ok_or_else(|| anyhow!("Tool not found: {}", tool_name))?;

        // Make HTTP request to the canister
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/call", server.url))
            .json(&serde_json::json!({
                "method": "mcp_call_tool",
                "args": args
            }))
            .send()
            .await
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Canister returned error: {}", response.status()));
        }

        let canister_result = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read response: {}", e))?;

        let result = serde_json::json!({
            "result": {
                "content": canister_result
            }
        });

        Ok(serde_json::to_string(&result)?)
    }

    async fn handle_get_server_info(&self) -> Result<String> {
        let config = self.config.read().await;

        let response = serde_json::json!({
            "result": {
                "name": "Icarus MCP Bridge",
                "version": env!("CARGO_PKG_VERSION"),
                "servers": config.servers.len(),
                "enabled_servers": config.enabled_servers().len()
            }
        });

        Ok(serde_json::to_string(&response)?)
    }
}

#[async_trait]
impl McpBridgeServer for SimpleBridgeServer {
    async fn run(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).await?;

        {
            let mut running = self.running.write().await;
            *running = true;
        }

        info!("MCP Bridge Server listening on {}", addr);

        let config = self.config.clone();
        let running = self.running.clone();

        loop {
            // Check if we should stop
            {
                let is_running = *running.read().await;
                if !is_running {
                    info!("Bridge server stopping...");
                    break;
                }
            }

            // Accept connections with timeout
            let accept_result =
                tokio::time::timeout(std::time::Duration::from_millis(1000), listener.accept())
                    .await;

            match accept_result {
                Ok(Ok((stream, addr))) => {
                    info!("Accepted connection from: {}", addr);

                    // Clone for the task
                    let server_clone = SimpleBridgeServer {
                        host: self.host.clone(),
                        port: self.port,
                        config: config.clone(),
                        running: running.clone(),
                    };

                    // Handle connection in a separate task
                    tokio::spawn(async move {
                        if let Err(e) = server_clone.handle_connection(stream).await {
                            error!("Error handling connection: {}", e);
                        }
                    });
                }
                Ok(Err(e)) => {
                    error!("Error accepting connection: {}", e);
                }
                Err(_) => {
                    // Timeout, continue loop to check if we should stop
                    continue;
                }
            }
        }

        info!("MCP Bridge Server stopped");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        info!("Bridge server stop requested");
        Ok(())
    }

    fn is_running(&self) -> bool {
        // This is a synchronous check, so we can't use the async lock
        // In a real implementation, you might want to use a different synchronization mechanism
        true // Simplified implementation
    }
}

/// HTTP-based MCP Bridge Server
pub(crate) struct HttpBridgeServer {
    host: String,
    port: u16,
    config: Arc<RwLock<McpConfig>>,
    running: Arc<RwLock<bool>>,
}

impl HttpBridgeServer {
    pub(crate) fn new(host: &str, port: u16, config: McpConfig) -> Result<Self> {
        Ok(Self {
            host: host.to_string(),
            port,
            config: Arc::new(RwLock::new(config)),
            running: Arc::new(RwLock::new(false)),
        })
    }

    async fn handle_http_request(
        &self,
        req: hyper::Request<hyper::body::Incoming>,
    ) -> Result<hyper::Response<String>, hyper::Error> {
        use hyper::{Method, Response, StatusCode};

        let response = match (req.method(), req.uri().path()) {
            (&Method::GET, "/health") => Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/plain")
                .body("OK".to_string())
                .unwrap(),

            (&Method::GET, "/mcp/tools") => match self.handle_list_tools_http().await {
                Ok(tools_json) => Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(tools_json)
                    .unwrap(),
                Err(e) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(format!("Error: {}", e))
                    .unwrap(),
            },

            (&Method::POST, "/mcp/call") => {
                // Handle tool calls
                match self.handle_call_tool_http(req).await {
                    Ok(result) => Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "application/json")
                        .body(result)
                        .unwrap(),
                    Err(e) => Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(format!("Error: {}", e))
                        .unwrap(),
                }
            }

            _ => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Not Found".to_string())
                .unwrap(),
        };

        Ok(response)
    }

    async fn handle_list_tools_http(&self) -> Result<String> {
        let config = self.config.read().await;
        let tools: Vec<_> = config
            .enabled_servers()
            .iter()
            .map(|server| {
                serde_json::json!({
                    "name": server.name,
                    "canister_id": server.canister_id,
                    "network": server.network,
                    "url": server.url
                })
            })
            .collect();

        Ok(serde_json::to_string(
            &serde_json::json!({ "tools": tools }),
        )?)
    }

    async fn handle_call_tool_http(
        &self,
        _req: hyper::Request<hyper::body::Incoming>,
    ) -> Result<String> {
        // This would need to parse the HTTP body and handle the tool call
        // For now, return a placeholder
        Ok(serde_json::to_string(&serde_json::json!({
            "result": "Tool call not implemented in HTTP bridge yet"
        }))?)
    }
}

#[async_trait]
impl McpBridgeServer for HttpBridgeServer {
    async fn run(&mut self) -> Result<()> {
        warn!("HTTP Bridge Server is not fully implemented yet");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        false // Not implemented yet
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::mcp::{McpConfig, McpServerConfig};
    use chrono::Utc;

    fn create_test_config() -> McpConfig {
        use crate::types::{CanisterId, Network, ServerName};
        let mut config = McpConfig::default();

        let server = McpServerConfig {
            name: ServerName::new("test-server").unwrap(),
            canister_id: CanisterId::new("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap(),
            network: Network::Local,
            url: "http://localhost:3000/mcp".to_string(),
            client: "claude-desktop".to_string(),
            port: Some(3000),
            enabled: true,
            created_at: Utc::now(),
            last_updated: Utc::now(),
        };

        config.servers.push(server);
        config
    }

    #[tokio::test]
    async fn test_bridge_server_creation() {
        let config = create_test_config();
        let result = SimpleBridgeServer::new("127.0.0.1", 0, config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_request_parsing() {
        let config = create_test_config();
        let server = SimpleBridgeServer::new("127.0.0.1", 0, config).unwrap();

        // Test list_tools request
        let request = r#"{"method": "list_tools"}"#;
        let result = server.handle_mcp_request(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("tools"));
    }

    #[tokio::test]
    async fn test_invalid_json_request() {
        let config = create_test_config();
        let server = SimpleBridgeServer::new("127.0.0.1", 0, config).unwrap();

        let request = "invalid json";
        let result = server.handle_mcp_request(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ping_request() {
        let config = create_test_config();
        let server = SimpleBridgeServer::new("127.0.0.1", 0, config).unwrap();

        let request = r#"{"method": "ping"}"#;
        let result = server.handle_mcp_request(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("pong"));
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let config = create_test_config();
        let server = SimpleBridgeServer::new("127.0.0.1", 0, config).unwrap();

        let request = r#"{"method": "unknown_method"}"#;
        let result = server.handle_mcp_request(request).await;
        assert!(result.is_err());
    }
}
