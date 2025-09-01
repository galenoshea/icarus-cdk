//! Bridge server that handles WebSocket connections from Claude Desktop
//! 
//! Accepts MCP protocol over WebSocket and translates to ICP calls

use anyhow::Result;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use candid::Principal;

use crate::bridge::translator::{ProtocolTranslator, McpRequest};

/// Configuration for the bridge server
pub struct BridgeConfig {
    pub canister_id: Principal,
    pub port: u16,
    pub ic_host: String,
}

/// Bridge server that handles MCP connections
pub struct BridgeServer {
    config: BridgeConfig,
}

impl BridgeServer {
    /// Create a new bridge server
    pub fn new(config: BridgeConfig) -> Self {
        Self { config }
    }
    
    /// Run the bridge server
    pub async fn run(self) -> Result<()> {
        let addr = format!("127.0.0.1:{}", self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        
        println!("🌉 Bridge server listening on ws://{}", addr);
        println!("📡 Connected to canister: {}", self.config.canister_id);
        println!("🔌 Ready for Claude Desktop connections");
        
        // Initialize protocol translator
        let translator = ProtocolTranslator::new(self.config.canister_id).await?;
        
        loop {
            let (stream, addr) = listener.accept().await?;
            println!("New connection from: {}", addr);
            
            // Handle each connection in a new task
            let translator = translator.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, translator).await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }
}

/// Handle a single WebSocket connection
async fn handle_connection(
    stream: tokio::net::TcpStream,
    translator: ProtocolTranslator,
) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    while let Some(msg) = ws_receiver.next().await {
        let msg = msg?;
        
        match msg {
            Message::Text(text) => {
                // Parse MCP request
                let request: McpRequest = serde_json::from_str(&text)?;
                
                // Translate and execute
                let response = translator.handle_mcp_request(request).await;
                
                // Send response
                let response_text = serde_json::to_string(&response)?;
                ws_sender.send(Message::Text(response_text.into())).await?;
            }
            Message::Close(_) => {
                println!("Client disconnected");
                break;
            }
            _ => {}
        }
    }
    
    Ok(())
}

// Make translator cloneable for multi-connection support
impl Clone for ProtocolTranslator {
    fn clone(&self) -> Self {
        // For MVP, create a new instance
        // In production, would share the metadata
        Self {
            canister_client: crate::bridge::canister_client::CanisterClient::new(
                Principal::from_text(&self.metadata.canister_id).unwrap()
            ),
            metadata: self.metadata.clone(),
        }
    }
}