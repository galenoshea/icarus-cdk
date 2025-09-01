//! MCP server that communicates via stdin/stdout
//! This is used when the CLI is run directly by Claude Desktop

use anyhow::Result;
use candid::Principal;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use super::translator::{McpRequest, McpResponse};

/// Mock translator for testing
struct MockTranslator {
    canister_id: Principal,
}

impl MockTranslator {
    async fn handle_mcp_request(&self, request: McpRequest) -> McpResponse {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize().await,
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => self.handle_tool_call(request.params).await,
            _ => Err(anyhow::anyhow!("Unknown method: {}", request.method)),
        };
        
        match result {
            Ok(value) => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: Some(value),
                error: None,
            },
            Err(e) => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(json!({
                    "code": -32603,
                    "message": e.to_string(),
                    "data": null
                })),
            },
        }
    }
    
    async fn handle_initialize(&self) -> Result<Value> {
        Ok(json!({
            "protocolVersion": "1.0.0",
            "capabilities": {
                "tools": {},
                "resources": {}
            }
        }))
    }
    
    async fn handle_list_tools(&self) -> Result<Value> {
        // Return sample tools from the canister
        Ok(json!({
            "tools": [
                {
                    "name": "memorize",
                    "description": "Store a memory in the canister",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string",
                                "description": "The content to memorize"
                            },
                            "tags": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Optional tags for the memory"
                            }
                        },
                        "required": ["content"]
                    }
                },
                {
                    "name": "recall",
                    "description": "Recall memories by tag",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "tag": {
                                "type": "string",
                                "description": "Tag to search for"
                            }
                        },
                        "required": ["tag"]
                    }
                },
                {
                    "name": "list",
                    "description": "List all memories",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "number",
                                "description": "Maximum number of memories to return"
                            }
                        }
                    }
                }
            ]
        }))
    }
    
    async fn handle_tool_call(&self, params: Value) -> Result<Value> {
        let tool_name = params.get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;
        
        // For now, return mock responses
        match tool_name {
            "memorize" => Ok(json!({
                "success": true,
                "id": "mock-memory-id"
            })),
            "recall" => Ok(json!({
                "memories": []
            })),
            "list" => Ok(json!({
                "memories": []
            })),
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name))
        }
    }
}

use anyhow::anyhow;

/// Run the MCP server using stdin/stdout for communication
pub async fn run(canister_id_str: String) -> Result<()> {
    // Parse canister ID
    let canister_id = match Principal::from_text(&canister_id_str) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to parse canister ID: {}", e);
            return Err(e.into());
        }
    };
    
    // For now, create a mock translator since we can't connect to the canister
    // TODO: Implement proper canister connection
    eprintln!("Creating mock translator for canister: {}", canister_id);
    let translator = MockTranslator { canister_id };
    
    // Set up stdin/stdout for async operation
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    
    eprintln!("MCP server ready, waiting for messages...");
    
    // Process messages from stdin
    loop {
        match lines.next_line().await {
            Ok(Some(line)) => {
                if line.trim().is_empty() {
                    continue;
                }
                eprintln!("Received message: {}", line);
        
        // Parse JSON-RPC request
        match serde_json::from_str::<Value>(&line) {
            Ok(request_value) => {
                // Convert to McpRequest
                let response = if let Ok(request) = serde_json::from_value::<McpRequest>(request_value.clone()) {
                    // Handle the request
                    translator.handle_mcp_request(request).await
                } else {
                    // Invalid request format
                    McpResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request_value.get("id").cloned(),
                        result: None,
                        error: Some(json!({
                            "code": -32600,
                            "message": "Invalid Request"
                        })),
                    }
                };
                
                // Send response
                let response_str = serde_json::to_string(&response)?;
                eprintln!("Sending response: {}", response_str);
                stdout.write_all(response_str.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
                eprintln!("Response sent successfully");
            }
            Err(e) => {
                // Send parse error
                let error_response = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {
                        "code": -32700,
                        "message": "Parse error",
                        "data": e.to_string()
                    }
                });
                
                let response_str = serde_json::to_string(&error_response)?;
                stdout.write_all(response_str.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }
            }
            Ok(None) => {
                eprintln!("EOF reached, stdin closed");
                break;
            }
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                break;
            }
        }
    }
    
    eprintln!("MCP server shutting down");
    Ok(())
}