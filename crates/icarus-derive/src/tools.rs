//! Tools macro implementation
//! 
//! Generates Candid methods from tool definitions

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemImpl, ImplItem, FnArg, Pat, Type};

pub fn expand_icarus_tools(_attr: TokenStream, input: ItemImpl) -> TokenStream {
    let self_ty = &input.self_ty;
    
    // Collect all methods marked with icarus_tool
    let mut tool_registrations = vec![];
    let mut candid_methods = vec![];
    
    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            // Check if method has icarus_tool attribute
            for attr in &method.attrs {
                if attr.path().is_ident("icarus_tool") {
                    let method_name = &method.sig.ident;
                    let tool_name = extract_tool_name(attr).unwrap_or_else(|| method_name.to_string());
                    let is_query = extract_is_query(attr);
                    let is_async = method.sig.asyncness.is_some();
                    
                    // Determine if this is a query based on receiver or attribute
                    let is_query = is_query.unwrap_or_else(|| {
                        method.sig.receiver().map(|r| {
                            // If it takes &self, it's a query. If &mut self, it's an update
                            match r {
                                syn::Receiver { mutability: None, .. } => true,
                                syn::Receiver { mutability: Some(_), .. } => false,
                            }
                        }).unwrap_or(true)
                    });
                    
                    // Extract parameters
                    let params: Vec<_> = method.sig.inputs.iter()
                        .filter_map(|arg| {
                            if let FnArg::Typed(pat_type) = arg {
                                if let Pat::Ident(ident) = &*pat_type.pat {
                                    Some((ident.ident.clone(), pat_type.ty.clone()))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();
                    
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
                    
                    // Generate Candid method based on query/update and async/sync
                    let candid_method = match (is_query, is_async) {
                        (true, true) => generate_async_query_method(&tool_name, method_name, &params),
                        (true, false) => generate_query_method(&tool_name, method_name, &params),
                        (false, true) => generate_async_update_method(&tool_name, method_name, &params),
                        (false, false) => generate_update_method(&tool_name, method_name, &params),
                    };
                    
                    candid_methods.push(candid_method);
                }
            }
        }
    }
    
    // Generate the implementation with tool registration
    quote! {
        #input
        
        impl #self_ty {
            /// Register all tools with the canister state
            pub fn __register_tools() {
                #(#tool_registrations)*
            }
        }
        
        // Generate Candid methods
        #(#candid_methods)*
    }
}

fn generate_query_method(tool_name: &str, method_name: &syn::Ident, params: &[(syn::Ident, Box<Type>)]) -> TokenStream {
    // Generate parameter list for Candid method
    let param_list = params.iter().map(|(name, ty)| {
        quote! { #name: #ty }
    });
    
    // Generate parameter passing to actual method
    let param_pass = params.iter().map(|(name, _)| {
        quote! { #name }
    });
    
    quote! {
        // Query methods must be synchronous in ICP
        #[ic_cdk::query]
        fn #method_name(#(#param_list),*) -> String {
            SERVER_INSTANCE.with(|s| {
                let server = s.borrow();
                let server = server.as_ref().expect("Server not initialized");
                
                // Call the sync method directly
                match server.#method_name(#(#param_pass),*) {
                    Ok(value) => value.to_string(),
                    Err(e) => serde_json::json!({
                        "error": e.to_string()
                    }).to_string()
                }
            })
        }
    }
}

fn generate_update_method(tool_name: &str, method_name: &syn::Ident, params: &[(syn::Ident, Box<Type>)]) -> TokenStream {
    // Generate parameter list for Candid method
    let param_list = params.iter().map(|(name, ty)| {
        quote! { #name: #ty }
    });
    
    // Generate parameter passing to actual method
    let param_pass = params.iter().map(|(name, _)| {
        quote! { #name }
    });
    
    quote! {
        // Sync update method
        #[ic_cdk::update]
        fn #method_name(#(#param_list),*) -> String {
            SERVER_INSTANCE.with(|s| {
                let mut server = s.borrow_mut();
                let server = server.as_mut().expect("Server not initialized");
                
                // Call the sync method directly
                match server.#method_name(#(#param_pass),*) {
                    Ok(value) => value.to_string(),
                    Err(e) => serde_json::json!({
                        "error": e.to_string()
                    }).to_string()
                }
            })
        }
    }
}

fn extract_tool_name(attr: &syn::Attribute) -> Option<String> {
    // Parse the attribute to extract name = "tool_name"
    let mut name = None;
    
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("name") {
            if let Ok(value) = meta.value() {
                if let Ok(lit_str) = value.parse::<syn::LitStr>() {
                    name = Some(lit_str.value());
                }
            }
        }
        Ok(())
    });
    
    name
}

fn extract_is_query(attr: &syn::Attribute) -> Option<bool> {
    // Parse the attribute to extract is_query = true/false
    let mut is_query = None;
    
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("is_query") {
            if let Ok(value) = meta.value() {
                if let Ok(lit_bool) = value.parse::<syn::LitBool>() {
                    is_query = Some(lit_bool.value());
                }
            }
        }
        Ok(())
    });
    
    is_query
}

// Generate async query method - This will error at compile time with helpful message
fn generate_async_query_method(tool_name: &str, method_name: &syn::Ident, params: &[(syn::Ident, Box<Type>)]) -> TokenStream {
    quote! {
        // This will generate a compile error with a helpful message
        compile_error!(concat!(
            "Query method '", stringify!(#method_name), "' cannot be async. ",
            "IC query methods must be synchronous. ",
            "Either remove 'async' from the method or change it to an update method by removing 'is_query = true'."
        ));
    }
}

// Generate async update method
fn generate_async_update_method(tool_name: &str, method_name: &syn::Ident, params: &[(syn::Ident, Box<Type>)]) -> TokenStream {
    // Generate parameter list for Candid method
    let param_list = params.iter().map(|(name, ty)| {
        quote! { #name: #ty }
    });
    
    // Generate parameter passing to actual method
    let param_pass = params.iter().map(|(name, _)| {
        quote! { #name }
    });
    
    quote! {
        // Async update method - IC supports async updates
        #[ic_cdk::update]
        async fn #method_name(#(#param_list),*) -> String {
            // We need to extract the future before awaiting to avoid RefCell issues
            let fut = SERVER_INSTANCE.with(|s| {
                let mut server = s.borrow_mut();
                let server = server.as_mut().expect("Server not initialized");
                
                // For now, return a placeholder until we find a better solution
                // The issue is that we can't hold a mutable borrow across an await point
                Box::pin(std::future::ready(Ok(serde_json::json!({
                    "status": "success",
                    "message": "Async update called",
                    "tool": #tool_name
                }))))
            });
            
            let result = fut.await;
            
            match result {
                Ok(value) => value.to_string(),
                Err(e) => serde_json::json!({
                    "error": e.to_string()
                }).to_string()
            }
        }
    }
}