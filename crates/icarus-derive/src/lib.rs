//! Procedural macros for the Icarus SDK
//! 
//! This crate provides derive macros and attribute macros to reduce
//! boilerplate when building MCP servers for ICP.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for creating MCP tools
/// 
/// # Example
/// ```ignore
/// #[derive(IcarusTool)]
/// #[icarus_tool(name = "calculator", description = "Perform calculations")]
/// struct CalculatorTool;
/// ```
#[proc_macro_derive(IcarusTool, attributes(icarus_tool))]
pub fn derive_icarus_tool(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Extract tool metadata from attributes
    let mut name = None;
    let mut description = None;
    
    for attr in &input.attrs {
        if attr.path().is_ident("icarus_tool") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    name = Some(meta.value()?.parse::<syn::LitStr>()?.value());
                    Ok(())
                } else if meta.path.is_ident("description") {
                    description = Some(meta.value()?.parse::<syn::LitStr>()?.value());
                    Ok(())
                } else {
                    Err(meta.error("unsupported icarus_tool attribute"))
                }
            }).expect("Failed to parse icarus_tool attribute");
        }
    }
    
    let struct_name = &input.ident;
    let tool_name = name.unwrap_or_else(|| struct_name.to_string());
    let tool_desc = description.unwrap_or_else(|| format!("{} tool", tool_name));
    
    // Generate the implementation
    let expanded = quote! {
        #[async_trait::async_trait]
        impl icarus_core::tool::IcarusTool for #struct_name {
            fn info(&self) -> icarus_core::tool::ToolInfo {
                icarus_core::tool::ToolInfo {
                    name: #tool_name.to_string(),
                    description: #tool_desc.to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    }),
                }
            }
            
            fn to_rmcp_tool(&self) -> rmcp::model::Tool {
                use std::borrow::Cow;
                use std::sync::Arc;
                
                let schema = serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                });
                
                rmcp::model::Tool {
                    name: Cow::Borrowed(#tool_name),
                    description: Some(Cow::Borrowed(#tool_desc)),
                    input_schema: Arc::new(schema.as_object().unwrap().clone()),
                    annotations: None,
                }
            }
            
            async fn execute(&self, args: serde_json::Value) -> icarus_core::error::Result<serde_json::Value> {
                // Default implementation - override in your tool
                Ok(serde_json::json!({
                    "error": "Tool execution not implemented"
                }))
            }
        }
    };
    
    TokenStream::from(expanded)
}

/// Attribute macro for MCP server setup
/// 
/// # Example
/// ```ignore
/// #[icarus_server(name = "my-server", version = "1.0.0")]
/// pub struct MyServer {
///     tools: Vec<Box<dyn IcarusTool>>,
/// }
/// ```
#[proc_macro_attribute]
pub fn icarus_server(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Parse the server metadata from the attribute args
    let mut server_name = None;
    let mut server_version = None;
    
    // Parse the arguments
    let args_tokens = TokenStream::from(args);
    if !args_tokens.is_empty() {
        let args_parser = syn::meta::parser(|meta| {
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
        parse_macro_input!(args_tokens with args_parser);
    }
    
    let struct_name = &input.ident;
    let name = server_name.unwrap_or_else(|| "icarus-server".to_string());
    let version = server_version.unwrap_or_else(|| "0.1.0".to_string());
    
    // Generate the expanded code
    let expanded = quote! {
        #input
        
        impl #struct_name {
            /// Create server configuration
            pub fn config() -> icarus_canister::state::ServerConfig {
                icarus_canister::state::ServerConfig {
                    name: #name.to_string(),
                    version: #version.to_string(),
                    canister_id: ic_cdk::api::canister_self(),
                }
            }
        }
        
        // Generate canister init function
        #[ic_cdk_macros::init]
        fn __icarus_init() {
            let config = #struct_name::config();
            icarus_canister::state::IcarusCanisterState::init(config);
        }
        
        // Generate canister query/update methods
        #[ic_cdk_macros::update]
        async fn icarus_mcp_request(request: icarus_core::protocol::IcarusMcpRequest) -> icarus_core::protocol::IcarusMcpResponse {
            icarus_canister::endpoints::icarus_mcp_request(request).await
        }
        
        #[ic_cdk_macros::query]
        fn icarus_capabilities() -> icarus_core::protocol::IcarusServerCapabilities {
            icarus_canister::endpoints::icarus_capabilities()
        }
        
        // Generate pre/post upgrade hooks
        #[ic_cdk_macros::pre_upgrade]
        fn __icarus_pre_upgrade() {
            // State is automatically preserved in stable memory
        }
        
        #[ic_cdk_macros::post_upgrade]
        fn __icarus_post_upgrade() {
            __icarus_init();
        }
    };
    
    TokenStream::from(expanded)
}

/// Derive macro for ICP storable types
/// 
/// # Example
/// ```ignore
/// #[derive(IcarusStorable)]
/// struct SessionState {
///     id: String,
///     created_at: u64,
/// }
/// ```
#[proc_macro_derive(IcarusStorable)]
pub fn derive_icarus_storable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    
    // For now, assume we're working with structs and use Candid serialization
    let expanded = quote! {
        impl ic_stable_structures::Storable for #struct_name {
            fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
                std::borrow::Cow::Owned(
                    candid::encode_one(self).expect("Failed to encode to Candid")
                )
            }
            
            fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
                candid::decode_one(&bytes).expect("Failed to decode from Candid")
            }
            
            const BOUND: ic_stable_structures::storable::Bound = 
                ic_stable_structures::storable::Bound::Bounded {
                    max_size: 1024 * 1024, // 1MB default max size
                    is_fixed_size: false,
                };
        }
        
        impl icarus_core::state::Storable for #struct_name {
            fn to_bytes(&self) -> icarus_core::error::Result<Vec<u8>> {
                candid::encode_one(self)
                    .map_err(|e| icarus_core::error::IcarusError::Canister(e.to_string()))
            }
            
            fn from_bytes(bytes: &[u8]) -> icarus_core::error::Result<Self> {
                candid::decode_one(bytes)
                    .map_err(|e| icarus_core::error::IcarusError::Canister(e.to_string()))
            }
            
            const MAX_SIZE: u32 = 1024 * 1024; // 1MB
            const FIXED_SIZE: Option<u32> = None;
        }
    };
    
    TokenStream::from(expanded)
}