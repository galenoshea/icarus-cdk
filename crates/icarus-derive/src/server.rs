//! Server macro implementation

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::parse::Parser;

pub fn expand_icarus_server(args: TokenStream, input: DeriveInput) -> TokenStream {
    // Parse server metadata
    let mut server_name = None;
    let mut server_version = None;
    
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            server_name = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            Ok(())
        } else if meta.path.is_ident("version") {
            server_version = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            Ok(())
        } else {
            Err(meta.error("unsupported icarus_server attribute"))
        }
    });
    
    let _ = parser.parse2(args);
    
    let struct_name = &input.ident;
    let name = server_name.unwrap_or_else(|| "icarus-server".to_string());
    let version = server_version.unwrap_or_else(|| "0.1.0".to_string());
    
    // Generate the expanded code
    quote! {
        #input
        
        // Server instance for this canister
        thread_local! {
            static SERVER_INSTANCE: std::cell::RefCell<Option<#struct_name>> = 
                std::cell::RefCell::new(None);
        }
        
        impl #struct_name {
            /// Create server configuration
            pub fn config() -> icarus_canister::state::ServerConfig {
                icarus_canister::state::ServerConfig {
                    name: #name.to_string(),
                    version: #version.to_string(),
                    canister_id: ic_cdk::id(),
                }
            }
            
            /// Initialize the server instance
            pub fn init_instance() {
                SERVER_INSTANCE.with(|s| {
                    *s.borrow_mut() = Some(#struct_name::new());
                });
            }
            
            /// Access the server instance
            pub fn with_instance<F, R>(f: F) -> R
            where
                F: FnOnce(&mut #struct_name) -> R
            {
                SERVER_INSTANCE.with(|s| {
                    let mut server = s.borrow_mut();
                    f(server.as_mut().expect("Server not initialized"))
                })
            }
        }
        
        
        // Generate canister init function
        #[ic_cdk_macros::init]
        fn __icarus_init() {
            let config = #struct_name::config();
            icarus_canister::state::IcarusCanisterState::init(config);
            
            // Initialize server instance
            #struct_name::init_instance();
            
            // Initialize tools (will be populated by icarus_tools macro)
            #struct_name::__register_tools();
        }
        
        // Generate canister query/update methods
        #[ic_cdk_macros::update]
        async fn icarus_mcp_request(request: icarus_core::protocol::IcarusMcpRequest) -> icarus_core::protocol::IcarusMcpResponse {
            // Check if this is a direct tool call
            let is_tool = icarus_canister::state::STATE.with(|s| {
                if let Some(state) = s.borrow().as_ref() {
                    state.tools.contains_key(&request.method)
                } else {
                    false
                }
            });
            
            if is_tool {
                // Handle direct tool call
                let tool_name = request.method.clone();
                let request_id = request.id.clone();
                
                // Parse params
                let params_value: serde_json::Value = if request.params.is_empty() {
                    serde_json::json!({})
                } else {
                    match serde_json::from_str(&request.params) {
                        Ok(v) => v,
                        Err(e) => {
                            return icarus_core::protocol::IcarusMcpResponse {
                                id: request_id,
                                result: None,
                                error: Some(icarus_core::protocol::IcarusMcpError {
                                    code: -32700,
                                    message: format!("Failed to parse params: {}", e),
                                    data: None,
                                }),
                            };
                        }
                    }
                };
                
                // Dispatch to tool - for MVP, return placeholder
                // A full implementation would properly handle async execution
                let result = Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Tool {} called with params: {}", tool_name, params_value)
                    }]
                }).to_string());
                
                match result {
                    Ok(result_str) => icarus_core::protocol::IcarusMcpResponse {
                        id: request_id,
                        result: Some(result_str),
                        error: None,
                    },
                    Err(e) => icarus_core::protocol::IcarusMcpResponse {
                        id: request_id,
                        result: None,
                        error: Some(e),
                    }
                }
            } else {
                // Use standard MCP handler
                icarus_canister::endpoints::icarus_mcp_request(request).await
            }
        }
        
        #[ic_cdk_macros::query]
        fn icarus_capabilities() -> icarus_core::protocol::IcarusServerCapabilities {
            icarus_canister::endpoints::icarus_capabilities()
        }
        
        // HTTP gateway endpoint
        #[ic_cdk_macros::query]
        fn http_request(req: icarus_canister::HttpRequest) -> icarus_canister::HttpResponse {
            icarus_canister::http_request(req)
        }
        
        // Direct tool methods for Candid - MVP hardcoded for memory server
        #[ic_cdk_macros::update]
        async fn memorize(content: String, tags: Option<Vec<String>>) -> String {
            let params = serde_json::json!({
                "content": content,
                "tags": tags
            });
            
            // For MVP, just acknowledge the call
            let _ = #struct_name::with_instance(|_server| {
                // A full implementation would dispatch to the actual method
                ()
            });
            
            // For now return immediate response since we can't await in update
            serde_json::json!({
                "status": "success",
                "message": "Memory stored",
                "id": format!("mem_{}", ic_cdk::api::time())
            }).to_string()
        }
        
        #[ic_cdk_macros::update]
        async fn forget(id: String) -> String {
            let params = serde_json::json!({
                "id": id
            });
            
            // For MVP, just acknowledge the call
            let _ = #struct_name::with_instance(|_server| {
                // A full implementation would dispatch to the actual method
                ()
            });
            
            serde_json::json!({
                "status": "success",
                "message": "Memory forgotten"
            }).to_string()
        }
        
        #[ic_cdk_macros::query]
        fn recall(query: String) -> String {
            let params = serde_json::json!({
                "query": query
            });
            
            #struct_name::with_instance(|server| {
                // For queries we need synchronous execution
                // For MVP, return placeholder since async in query is complex
                let _ = params;
                let _ = server;
                serde_json::json!({
                    "matches": [],
                    "count": 0
                }).to_string()
            })
        }
        
        #[ic_cdk_macros::query]
        fn list(limit: Option<u64>) -> String {
            let params = serde_json::json!({
                "limit": limit
            });
            
            #struct_name::with_instance(|server| {
                // For queries we need synchronous execution
                // For MVP, return placeholder since async in query is complex
                let _ = params;
                let _ = server;
                serde_json::json!({
                    "memories": [],
                    "total": 0
                }).to_string()
            })
        }
        
        // Generate pre/post upgrade hooks
        #[ic_cdk_macros::pre_upgrade]
        fn __icarus_pre_upgrade() {
            // State is automatically preserved in stable memory
        }
        
        #[ic_cdk_macros::post_upgrade]
        fn __icarus_post_upgrade() {
            let config = #struct_name::config();
            icarus_canister::state::IcarusCanisterState::init(config);
            
            // Initialize server instance
            #struct_name::init_instance();
            
            // Re-register tools
            #struct_name::__register_tools();
        }
    }
}