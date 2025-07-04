//! Procedural macros for the Icarus SDK
//! 
//! This crate provides derive macros and attribute macros to reduce
//! boilerplate when building MCP servers for ICP.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod server;
mod tools;

/// Derive macro for creating MCP tools
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
/// pub struct MyServer {
///     tools: Vec<Box<dyn IcarusTool>>,
/// }
/// ```
#[proc_macro_attribute]
pub fn icarus_server(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = TokenStream2::from(args);
    let input = parse_macro_input!(input as DeriveInput);
    server::expand_icarus_server(args, input).into()
}

/// Derive macro for ICP storable types
#[proc_macro_derive(IcarusStorable)]
pub fn derive_icarus_storable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    
    // Extract generics if any
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    // For now, assume we're working with structs and use Candid serialization
    let expanded = quote! {
        impl #impl_generics ic_stable_structures::Storable for #struct_name #ty_generics #where_clause {
            fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
                // Note: Using expect here is acceptable as Storable trait doesn't support errors
                // Ensure your types are always serializable
                std::borrow::Cow::Owned(
                    candid::encode_one(self).expect("Failed to encode to Candid")
                )
            }
            
            fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
                // Note: Using expect here is acceptable as Storable trait doesn't support errors
                candid::decode_one(&bytes).expect("Failed to decode from Candid")
            }
            
            const BOUND: ic_stable_structures::storable::Bound = 
                ic_stable_structures::storable::Bound::Bounded {
                    max_size: 1024 * 1024, // 1MB default max size
                    is_fixed_size: false,
                };
        }
        
        impl #impl_generics icarus_core::state::Storable for #struct_name #ty_generics #where_clause {
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

/// Attribute macro for marking impl blocks that contain tool methods
#[proc_macro_attribute]
pub fn icarus_tools(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = TokenStream2::from(attr);
    let input = parse_macro_input!(item as syn::ItemImpl);
    tools::expand_icarus_tools(attr, input).into()
}

/// Attribute macro for individual tool methods
#[proc_macro_attribute]
pub fn icarus_tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as syn::ItemFn);
    
    // Parse attributes using parse_nested_meta
    let mut tool_name = input_fn.sig.ident.to_string();
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            let value: syn::LitStr = meta.value()?.parse()?;
            tool_name = value.value();
            Ok(())
        } else {
            Err(meta.error("unsupported attribute"))
        }
    });
    
    let _ = parse_macro_input!(attr with parser);
    
    // For now, just pass through the function as-is
    // The actual serialization handling would be done by the framework
    // when integrating with the canister and MCP bridge
    let expanded = quote! {
        #input_fn
    };
    
    TokenStream::from(expanded)
}