//! Canister endpoints for tool metadata discovery
//!
//! In the clean architecture, canisters don't handle MCP protocol.
//! They only provide metadata for bridge discovery.

use crate::state::STATE;
use candid::{CandidType, Deserialize, Principal};
use icarus_core::protocol::{IcarusMetadata, ToolMetadata};

/// Query canister metadata for tool discovery
pub fn icarus_metadata() -> IcarusMetadata {
    STATE.with(|s| {
        let state = s.borrow();
        if let Some(state) = state.as_ref() {
            IcarusMetadata {
                version: state.config.get().version.clone(),
                canister_id: ic_cdk::api::canister_self(),
                tools: state
                    .tools
                    .iter()
                    .map(|entry| {
                        let (name, tool_state) = (entry.key().clone(), entry.value());
                        ToolMetadata {
                            name: name.clone(),
                            candid_method: name.clone(), // Method name matches tool name
                            is_query: tool_state.is_query,
                            description: tool_state.description.clone(),
                            parameters: tool_state
                                .parameters
                                .iter()
                                .map(|p| icarus_core::protocol::ParameterMetadata {
                                    name: p.name.clone(),
                                    candid_type: p.param_type.clone(),
                                    description: p.description.clone(),
                                    required: p.required,
                                })
                                .collect(),
                        }
                    })
                    .collect(),
            }
        } else {
            IcarusMetadata {
                version: "1.0.0".to_string(),
                canister_id: ic_cdk::api::canister_self(),
                tools: vec![],
            }
        }
    })
}

/// HTTP request type for canister HTTP gateway
#[derive(CandidType, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// HTTP response type for canister HTTP gateway
#[derive(CandidType)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Handle HTTP requests from the IC HTTP gateway
pub fn http_request(_req: HttpRequest) -> HttpResponse {
    let metadata = icarus_metadata();

    // Generate HTML showing available tools
    let tools_html: String = metadata
        .tools
        .iter()
        .map(|tool| {
            format!(
                r#"<div class="tool">
                <strong>{}</strong> ({})
                <br><small>{}</small>
            </div>"#,
                tool.name,
                if tool.is_query { "query" } else { "update" },
                tool.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Icarus Canister</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        .container {{
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            margin-bottom: 10px;
        }}
        .info {{
            background: #e7f3ff;
            padding: 15px;
            border-radius: 5px;
            border-left: 4px solid #2196F3;
            margin: 20px 0;
        }}
        code {{
            background: #f0f0f0;
            padding: 2px 5px;
            border-radius: 3px;
            font-family: "Courier New", monospace;
        }}
        .tool {{
            margin: 10px 0;
            padding: 10px;
            background: #f9f9f9;
            border-radius: 5px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Icarus Canister</h1>
        <p>This is a standard ICP canister built with Icarus SDK.</p>
        
        <div class="info">
            <strong>Canister ID:</strong> <code>{}</code><br>
            <strong>Version:</strong> <code>{}</code><br>
            <strong>Status:</strong> <span style="color: green;">✓ Running</span>
        </div>
        
        <h2>Available Tools</h2>
        {}
        
        <h2>Connect with Claude Desktop</h2>
        <p>To use this canister with Claude Desktop, run:</p>
        <code>icarus bridge start --canister-id {}</code>
        
        <hr style="margin-top: 40px; border: none; border-top: 1px solid #eee;">
        <p style="text-align: center; color: #666; font-size: 14px;">
            Powered by <a href="https://icarus.dev" style="color: #2196F3;">Icarus SDK</a>
        </p>
    </div>
</body>
</html>"#,
        ic_cdk::api::canister_self().to_text(),
        metadata.version,
        tools_html,
        ic_cdk::api::canister_self().to_text()
    );

    HttpResponse {
        status_code: 200,
        headers: vec![
            (
                "Content-Type".to_string(),
                "text/html; charset=UTF-8".to_string(),
            ),
            ("Cache-Control".to_string(), "no-cache".to_string()),
        ],
        body: html.into_bytes(),
    }
}

/// Get the current owner of the canister
pub fn get_owner() -> Principal {
    crate::state::get_owner()
}
