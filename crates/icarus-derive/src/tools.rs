//! Tools macro implementation

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemImpl, ImplItem};

pub fn expand_icarus_tools(_attr: TokenStream, input: ItemImpl) -> TokenStream {
    let self_ty = &input.self_ty;
    
    // Collect all methods marked with icarus_tool
    let mut tool_registrations = vec![];
    let mut tool_implementations = vec![];
    let mut tool_methods = vec![];
    
    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            // Check if method has icarus_tool attribute
            for attr in &method.attrs {
                if attr.path().is_ident("icarus_tool") {
                    let method_name = &method.sig.ident;
                    let tool_name = extract_tool_name(attr).unwrap_or_else(|| method_name.to_string());
                    
                    // Store the full method for later processing
                    tool_methods.push((tool_name.clone(), method.clone()));
                    
                    // Determine if this is a query or update based on receiver type
                    let is_query = method.sig.receiver().map(|r| {
                        // If it takes &self, it's a query. If &mut self, it's an update
                        match r {
                            syn::Receiver { mutability: None, .. } => true,
                            syn::Receiver { mutability: Some(_), .. } => false,
                        }
                    }).unwrap_or(true); // Default to query if no receiver
                    
                    // Register tool in canister state
                    tool_registrations.push(quote! {
                        icarus_canister::state::STATE.with(|s| {
                            if let Some(state) = s.borrow_mut().as_mut() {
                                state.tools.insert(
                                    #tool_name.to_string(),
                                    icarus_canister::state::ToolState {
                                        name: #tool_name.to_string(),
                                        enabled: true,
                                        call_count: 0,
                                        is_query: #is_query,
                                    }
                                );
                            }
                        });
                    });
                    
                    // Generate tool dispatcher - for now just store the tool name
                    // The actual dispatch will need to be handled differently
                    tool_implementations.push((tool_name.clone(), method_name.clone()));
                }
            }
        }
    }
    
    // Generate wrapper methods for each tool with hardcoded parameter extraction
    let tool_wrappers = tool_methods.iter().map(|(tool_name, method)| {
        let method_name = &method.sig.ident;
        let wrapper_name = syn::Ident::new(&format!("__tool_{}", method_name), method_name.span());
        
        // For MVP, generate specific parsing for known tools
        let parse_and_call = match tool_name.as_str() {
            "memorize" => quote! {
                let content = params.get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| icarus_core::error::ToolError::InvalidInput("Missing content parameter".to_string()))?
                    .to_string();
                let tags = params.get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
                self.memorize(content, tags).await
            },
            "forget" => quote! {
                let id = params.get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| icarus_core::error::ToolError::InvalidInput("Missing id parameter".to_string()))?
                    .to_string();
                self.forget(id).await
            },
            "recall" => quote! {
                let query = params.get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| icarus_core::error::ToolError::InvalidInput("Missing query parameter".to_string()))?
                    .to_string();
                self.recall(query).await
            },
            "list" => quote! {
                let limit = params.get("limit")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize);
                self.list(limit).await
            },
            _ => quote! {
                // Generic fallback - just return error
                Err(icarus_core::error::ToolError::NotFound(format!("Tool {} not implemented", #tool_name)))
            }
        };
        
        quote! {
            pub async fn #wrapper_name(&mut self, params: serde_json::Value) -> Result<serde_json::Value, icarus_core::error::ToolError> {
                #parse_and_call
            }
        }
    });
    
    // Generate match arms for each tool
    let tool_match_arms = tool_implementations.iter().map(|(tool_name, method_name)| {
        let wrapper_name = syn::Ident::new(&format!("__tool_{}", method_name), method_name.span());
        quote! {
            #tool_name => {
                match self.#wrapper_name(params).await {
                    Ok(result) => Ok(serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string())
                        }]
                    }).to_string()),
                    Err(e) => Err(icarus_core::protocol::IcarusMcpError {
                        code: -32603,
                        message: e.to_string(),
                        data: None,
                    })
                }
            }
        }
    });
    
    // Generate the implementation with tool registration
    quote! {
        #input
        
        impl #self_ty {
            fn __register_tools() {
                #(#tool_registrations)*
            }
            
            /// Get list of available tools
            pub fn __get_tools() -> Vec<String> {
                icarus_canister::state::STATE.with(|s| {
                    if let Some(state) = s.borrow().as_ref() {
                        state.tools.iter().map(|(name, _)| name).collect()
                    } else {
                        vec![]
                    }
                })
            }
            
            /// Dispatch tool calls
            pub async fn __dispatch_tool(&mut self, tool_name: &str, params: serde_json::Value) -> Result<String, icarus_core::protocol::IcarusMcpError> {
                match tool_name {
                    #(#tool_match_arms)*
                    _ => Err(icarus_core::protocol::IcarusMcpError {
                        code: -32601,
                        message: format!("Unknown tool: {}", tool_name),
                        data: None,
                    })
                }
            }
            
            // Tool wrapper methods
            #(#tool_wrappers)*
        }
    }
}

fn extract_tool_name(attr: &syn::Attribute) -> Option<String> {
    let mut name = None;
    
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("name") {
            name = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            Ok(())
        } else {
            Ok(()) // Ignore other attributes
        }
    });
    
    name
}